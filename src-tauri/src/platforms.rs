use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

const FETCH_TIMEOUT: Duration = Duration::from_secs(8);

// ─── Data Types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub score: i64,
    pub platform: String,
    pub category: String,
    pub timestamp: Option<String>,
    pub geos: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusteredItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub score: i64,
    pub platform: String,
    pub platforms: Vec<String>,
    pub platform_details: Vec<PlatformDetail>,
    pub mentions: usize,
    pub category: String,
    pub timestamp: Option<String>,
    pub geos: Option<Vec<String>>,
    pub normalized_score: i64,
    pub composite_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformDetail {
    pub platform: String,
    pub score: i64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStats {
    pub count: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingData {
    pub items: Vec<ClusteredItem>,
    pub by_category: HashMap<String, Vec<ClusteredItem>>,
    pub platform_stats: HashMap<String, PlatformStats>,
    pub total_items: usize,
    pub fetched_at: String,
    pub elapsed: u64,
}

// ─── Categories ──────────────────────────────────────────────

const CATEGORIES: &[(&str, &str)] = &[
    ("all", "All"),
    ("tech", "Technology"),
    ("entertainment", "Entertainment"),
    ("politics", "Politics"),
    ("business", "Business"),
    ("science", "Science"),
    ("sports", "Sports"),
    ("general", "General"),
];

fn categorize_by_title(title: &str) -> &'static str {
    let t = title.to_lowercase();
    let checks: &[(&str, &[&str])] = &[
        ("tech", &["ai", "apple", "google", "microsoft", "openai", "chatgpt", "software", "app", "tech", "crypto", "bitcoin", "programming", "nvidia"]),
        ("sports", &["nba", "nfl", "soccer", "football", "basketball", "tennis", "championship", "league", "cricket", "olympics"]),
        ("politics", &["president", "election", "congress", "senate", "government", "trump", "biden", "war ", "political", "parliament"]),
        ("entertainment", &["movie", "film", "album", "song", "netflix", "disney", "concert", "actor", "actress", "anime", "manga", "trailer", "box office"]),
        ("science", &["nasa", "climate", "research", "study", "space", "earthquake", "species", "planet", "virus"]),
        ("business", &["stock", "market", "economy", "ceo", "company", "revenue", "ipo", "billion"]),
    ];
    for (cat, keywords) in checks {
        if keywords.iter().any(|kw| t.contains(kw)) {
            return cat;
        }
    }
    "general"
}

// ─── Platform weights ────────────────────────────────────────

fn platform_weight(platform: &str) -> f64 {
    match platform {
        "youtube" => 1.4,     // highest — real view counts
        "google" => 1.2,      // search interest
        "reddit" => 1.2,      // real upvotes
        "hackernews" => 0.9,
        "wikipedia" => 0.9,   // bumped — will have summaries
        "mastodon" => 0.6,    // social noise
        "bluesky" => 0.6,     // social noise
        "news" => 0.3,        // validation only
        _ => 1.0,
    }
}

// ─── Main fetch ──────────────────────────────────────────────

pub async fn fetch_all(
    youtube_key: Option<&str>,
) -> Result<TrendingData> {
    let start = Instant::now();
    let client = Client::builder()
        .timeout(FETCH_TIMEOUT)
        .user_agent("LightTrend/1.0 (trending dashboard)")
        .build()?;

    // Fetch all platforms in parallel
    let (reddit, hn, google, youtube, wikipedia, news, mastodon, bluesky) = tokio::join!(
        fetch_reddit(&client),
        fetch_hackernews(&client),
        fetch_google(&client),
        fetch_youtube(&client, youtube_key),
        fetch_wikipedia(&client),
        fetch_news(&client),
        fetch_mastodon(&client),
        fetch_bluesky(&client),
    );

    let mut all_items: Vec<TrendItem> = Vec::new();
    for result in [reddit, hn, google, youtube, wikipedia, news, mastodon, bluesky] {
        match result {
            Ok(items) => all_items.extend(items),
            Err(e) => eprintln!("[Platform error] {e:#}"),
        }
    }

    let total_items = all_items.len();
    let normalized = normalize_scores(&all_items);
    let mut clustered = cluster_topics(&normalized);
    clustered.sort_by(|a, b| b.composite_score.cmp(&a.composite_score));

    // After clustering, enrich empty descriptions from other items
    for cluster in &mut clustered {
        if cluster.description.is_some() { continue; }
        let kw = extract_keywords(&cluster.title);
        if kw.is_empty() { continue; }

        let mut best: Option<&str> = None;
        let mut best_overlap = 0;
        for item in &all_items {
            if let Some(ref desc) = item.description {
                if desc.len() < 20 { continue; }
                let item_kw = extract_keywords(&item.title);
                let overlap = kw.iter().filter(|k| item_kw.contains(*k)).count();
                if overlap >= 2 && overlap > best_overlap {
                    best_overlap = overlap;
                    best = Some(desc);
                }
            }
        }
        if let Some(desc) = best {
            cluster.description = Some(desc.to_string());
        }
    }

    // Build category views
    let mut by_category: HashMap<String, Vec<ClusteredItem>> = HashMap::new();
    for (cat_id, _) in CATEGORIES {
        let items = if *cat_id == "all" {
            clustered.iter().take(30).cloned().collect()
        } else {
            clustered.iter().filter(|i| i.category == *cat_id).take(20).cloned().collect()
        };
        by_category.insert(cat_id.to_string(), items);
    }

    // Platform stats
    let mut platform_stats: HashMap<String, PlatformStats> = HashMap::new();
    for item in &all_items {
        let entry = platform_stats.entry(item.platform.clone()).or_insert(PlatformStats {
            count: 0,
            name: item.platform.clone(),
        });
        entry.count += 1;
    }

    let elapsed = start.elapsed().as_millis() as u64;

    Ok(TrendingData {
        items: clustered.into_iter().take(50).collect(),
        by_category,
        platform_stats,
        total_items,
        fetched_at: chrono::Utc::now().to_rfc3339(),
        elapsed,
    })
}

// ─── Normalization ───────────────────────────────────────────

struct NormalizedItem {
    item: TrendItem,
    normalized_score: i64,
}

fn normalize_scores(items: &[TrendItem]) -> Vec<NormalizedItem> {
    let mut by_platform: HashMap<&str, Vec<&TrendItem>> = HashMap::new();
    for item in items {
        by_platform.entry(&item.platform).or_default().push(item);
    }

    let mut result = Vec::new();
    for (platform, platform_items) in &by_platform {
        let max_score = platform_items.iter().map(|i| i.score).max().unwrap_or(1).max(1);
        let all_zero = max_score <= 1;
        let weight = platform_weight(platform);
        let count = platform_items.len() as f64;

        for (idx, item) in platform_items.iter().enumerate() {
            let raw_norm = if all_zero {
                (100.0 - idx as f64 * (100.0 / count.max(1.0))).max(0.0)
            } else {
                (item.score as f64 / max_score as f64) * 100.0
            };
            let normalized = ((raw_norm * weight).round() as i64).min(100);
            result.push(NormalizedItem {
                item: (*item).clone(),
                normalized_score: normalized,
            });
        }
    }
    result
}

// ─── Clustering ──────────────────────────────────────────────

fn extract_keywords(title: &str) -> HashSet<String> {
    let stopwords: HashSet<&str> = [
        "the","a","an","is","are","was","were","be","been","have","has","had",
        "do","does","did","will","would","could","should","may","might","to",
        "of","in","for","on","with","at","by","from","as","into","through",
        "and","but","or","if","it","its","this","that","what","which","who",
        "not","only","so","than","too","very","just","about","up","get","like",
        "one","two","first","also","after","now","still","even","new","says","said",
    ].into_iter().collect();

    title.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|w| w.to_string())
        .collect()
}

fn trigrams(s: &str) -> HashSet<String> {
    let padded = format!("  {}  ", s.to_lowercase());
    let chars: Vec<char> = padded.chars().collect();
    let mut set = HashSet::new();
    for i in 0..chars.len().saturating_sub(2) {
        set.insert(chars[i..i+3].iter().collect());
    }
    set
}

fn dice_coefficient(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    let intersection = a.iter().filter(|t| b.contains(*t)).count();
    if a.len() + b.len() == 0 { return 0.0; }
    (2 * intersection) as f64 / (a.len() + b.len()) as f64
}

fn recency_bonus(timestamp: &Option<String>) -> i64 {
    let Some(ts) = timestamp else { return 0 };
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) else { return 0 };
    let age_hours = (chrono::Utc::now() - dt.to_utc()).num_hours();
    if age_hours < 1 { 15 }
    else if age_hours < 6 { 10 }
    else if age_hours < 12 { 5 }
    else { 0 }
}

fn cluster_topics(items: &[NormalizedItem]) -> Vec<ClusteredItem> {
    let keywords: Vec<HashSet<String>> = items.iter().map(|i| extract_keywords(&i.item.title)).collect();
    let tri: Vec<HashSet<String>> = items.iter().map(|i| trigrams(&i.item.title)).collect();

    let mut clusters: Vec<ClusteredItem> = Vec::new();
    let mut used: HashSet<usize> = HashSet::new();

    for i in 0..items.len() {
        if used.contains(&i) { continue; }

        let mut cluster = ClusteredItem {
            id: items[i].item.id.clone(),
            title: items[i].item.title.clone(),
            description: items[i].item.description.clone(),
            url: items[i].item.url.clone(),
            score: items[i].item.score,
            platform: items[i].item.platform.clone(),
            platforms: vec![items[i].item.platform.clone()],
            platform_details: vec![PlatformDetail {
                platform: items[i].item.platform.clone(),
                score: items[i].item.score,
                url: items[i].item.url.clone(),
            }],
            mentions: 1,
            category: items[i].item.category.clone(),
            timestamp: items[i].item.timestamp.clone(),
            geos: items[i].item.geos.clone(),
            normalized_score: items[i].normalized_score,
            composite_score: 0,
        };

        for j in (i + 1)..items.len() {
            if used.contains(&j) { continue; }
            if cluster.platforms.contains(&items[j].item.platform) { continue; }

            let overlap = keywords[i].iter().filter(|k| keywords[j].contains(*k)).count();
            let similar = overlap >= 3 || (overlap >= 2 && dice_coefficient(&tri[i], &tri[j]) > 0.25);

            if similar {
                cluster.platforms.push(items[j].item.platform.clone());
                cluster.platform_details.push(PlatformDetail {
                    platform: items[j].item.platform.clone(),
                    score: items[j].item.score,
                    url: items[j].item.url.clone(),
                });
                cluster.mentions += 1;
                cluster.normalized_score = cluster.normalized_score.max(items[j].normalized_score);

                if let Some(ref geos) = items[j].item.geos {
                    let mut existing = cluster.geos.unwrap_or_default();
                    existing.extend(geos.clone());
                    cluster.geos = Some(existing);
                }
                if cluster.description.is_none() {
                    cluster.description = items[j].item.description.clone();
                }

                used.insert(j);
            }
        }

        cluster.composite_score = cluster.normalized_score
            + (cluster.mentions as i64 - 1) * 20
            + recency_bonus(&cluster.timestamp);

        // Content richness penalty
        let richness = match &cluster.description {
            Some(d) if d.len() >= 50 => 0,
            Some(_) => -5,
            None => -10,
        };
        cluster.composite_score += richness;

        used.insert(i);
        clusters.push(cluster);
    }
    clusters
}

// ─── Platform Fetchers ───────────────────────────────────────

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    // Try CDATA first, then plain text
    let cdata_pattern = format!("<{tag}");
    if let Some(start_idx) = xml.find(&cdata_pattern) {
        let after_tag = &xml[start_idx..];
        if let Some(end_idx) = after_tag.find(&format!("</{tag}>")) {
            let content = &after_tag[..end_idx];
            // Find content after >
            if let Some(gt) = content.find('>') {
                let inner = &content[gt + 1..];
                let cleaned = inner
                    .replace("<![CDATA[", "")
                    .replace("]]>", "")
                    .trim()
                    .to_string();
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' { in_tag = true; }
        else if c == '>' { in_tag = false; }
        else if !in_tag { result.push(c); }
    }
    // Decode common entities
    result
        .replace("&#8217;", "\u{2019}")  // right single quote '
        .replace("&#8216;", "\u{2018}")  // left single quote '
        .replace("&#8220;", "\u{201C}")  // left double quote "
        .replace("&#8221;", "\u{201D}")  // right double quote "
        .replace("&#8212;", "\u{2014}")  // em dash —
        .replace("&#8211;", "\u{2013}")  // en dash –
        .replace("&#160;", " ")          // non-breaking space
        .replace("&mdash;", "\u{2014}")
        .replace("&ndash;", "\u{2013}")
        .replace("&rsquo;", "\u{2019}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&ldquo;", "\u{201C}")
        .replace("&nbsp;", " ")
        .replace("&hellip;", "\u{2026}")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// ─── Smart sentence splitting ────────────────────────────────

fn smart_split(text: &str) -> (String, String) {
    let text = text.trim();
    let desc = text.chars().take(500).collect::<String>();

    // Find sentence boundary in first 120 chars
    let boundary = text.char_indices()
        .take_while(|(i, _)| *i < 120)
        .filter(|(_, c)| matches!(c, '.' | '!' | '?' | '\n'))
        .map(|(i, _)| i + 1)
        .last();

    let title = if let Some(pos) = boundary {
        text[..pos].trim().to_string()
    } else if text.len() <= 100 {
        text.to_string()
    } else {
        // Break at last space before 100
        let cut = text.char_indices()
            .take_while(|(i, _)| *i < 100)
            .filter(|(_, c)| c.is_whitespace())
            .map(|(i, _)| i)
            .last()
            .unwrap_or(100);
        format!("{}\u{2026}", text[..cut].trim())
    };

    (title, desc)
}

// --- Reddit ---

async fn fetch_reddit(client: &Client) -> Result<Vec<TrendItem>> {
    let subreddits = ["popular","all","technology","worldnews","science","movies","gaming","music","television","entertainment"];
    let mut items = Vec::new();

    // Fetch in batches of 4
    for chunk in subreddits.chunks(4) {
        let futures: Vec<_> = chunk.iter().map(|sub| {
            let client = client.clone();
            let sub = sub.to_string();
            async move {
                let url = format!("https://www.reddit.com/r/{}/hot.json?limit=15&raw_json=1", sub);
                let res = client.get(&url)
                    .header("User-Agent", "LightTrend/1.0 (trending dashboard)")
                    .send().await?;
                let data: serde_json::Value = res.json().await?;
                Ok::<_, anyhow::Error>((sub, data))
            }
        }).collect();

        let results = futures::future::join_all(futures).await;
        for (_sub, data) in results.into_iter().flatten() {
            if let Some(children) = data["data"]["children"].as_array() {
                for child in children {
                    let post = &child["data"];
                    let title = post["title"].as_str().unwrap_or_default();
                    let id = post["id"].as_str().unwrap_or_default();
                    if title.is_empty() || id.is_empty() { continue; }

                    let subreddit = post["subreddit"].as_str().unwrap_or_default();
                    let category = match subreddit.to_lowercase().as_str() {
                        "technology" | "programming" => "tech",
                        "worldnews" | "politics" => "politics",
                        "science" | "space" => "science",
                        "movies" | "gaming" | "music" | "television" | "entertainment" => "entertainment",
                        _ => categorize_by_title(title),
                    };

                    items.push(TrendItem {
                        id: format!("reddit_{id}"),
                        title: title.to_string(),
                        description: post["selftext"].as_str()
                            .filter(|s: &&str| !s.is_empty())
                            .map(|s: &str| s.chars().take(200).collect()),
                        url: format!("https://www.reddit.com{}", post["permalink"].as_str().unwrap_or("")),
                        score: post["ups"].as_i64().unwrap_or(0),
                        platform: "reddit".to_string(),
                        category: category.to_string(),
                        timestamp: post["created_utc"].as_f64().and_then(|t| {
                            chrono::DateTime::from_timestamp(t as i64, 0)
                                .map(|dt| dt.to_rfc3339())
                        }),
                        geos: None,
                    });
                }
            }
        }
    }

    // Deduplicate by ID
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.id.clone()));
    Ok(items)
}

// --- Hacker News ---

async fn fetch_hackernews(client: &Client) -> Result<Vec<TrendItem>> {
    let res = client.get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send().await?;
    let ids: Vec<i64> = res.json().await?;
    let top_ids: Vec<i64> = ids.into_iter().take(30).collect();

    let mut items = Vec::new();
    for chunk in top_ids.chunks(15) {
        let futures: Vec<_> = chunk.iter().map(|id| {
            let client = client.clone();
            let id = *id;
            async move {
                let url = format!("https://hacker-news.firebaseio.com/v0/item/{id}.json");
                let res = client.get(&url).send().await?;
                let data: serde_json::Value = res.json().await?;
                Ok::<_, anyhow::Error>(data)
            }
        }).collect();

        let results = futures::future::join_all(futures).await;
        for post in results.into_iter().flatten() {
            let title = post["title"].as_str().unwrap_or_default();
            if title.is_empty() { continue; }
            let id = post["id"].as_i64().unwrap_or(0);
            items.push(TrendItem {
                id: format!("hn_{id}"),
                title: title.to_string(),
                description: post["text"].as_str()
                    .filter(|s| !s.is_empty())
                    .map(|s| strip_html(s).chars().take(300).collect()),
                url: post["url"].as_str()
                    .unwrap_or(&format!("https://news.ycombinator.com/item?id={id}"))
                    .to_string(),
                score: post["score"].as_i64().unwrap_or(0),
                platform: "hackernews".to_string(),
                category: "tech".to_string(),
                timestamp: post["time"].as_i64().and_then(|t| {
                    chrono::DateTime::from_timestamp(t, 0).map(|dt| dt.to_rfc3339())
                }),
                geos: None,
            });
        }
    }
    Ok(items)
}

// --- Google Trends ---

async fn fetch_google(client: &Client) -> Result<Vec<TrendItem>> {
    let geos = ["US", "GB", "AU", "CA", "IN"];
    let futures: Vec<_> = geos.iter().map(|geo| {
        let client = client.clone();
        let geo = geo.to_string();
        async move {
            let url = format!("https://trends.google.com/trending/rss?geo={geo}&hl=en-US&hours=24&status=active&sort=search-volume");
            let res = client.get(&url).send().await?;
            let xml = res.text().await?;
            let mut items = Vec::new();
            for entry in xml.split("<item>").skip(1) {
                let title = match extract_xml_tag(entry, "title") {
                    Some(t) => t,
                    None => continue,
                };
                let link = extract_xml_tag(entry, "link").unwrap_or_default();
                let mut desc = extract_xml_tag(entry, "description").map(|d| strip_html(&d));
                let traffic = extract_xml_tag(entry, "ht:approx_traffic").unwrap_or_default();
                let score = parse_traffic(&traffic);

                // Extract <ht:news_item_title> blocks for richer description
                let mut news_titles: Vec<String> = Vec::new();
                let mut search_start = 0;
                while let Some(pos) = entry[search_start..].find("<ht:news_item>") {
                    let abs_pos = search_start + pos;
                    let block_end = entry[abs_pos..].find("</ht:news_item>")
                        .map(|e| abs_pos + e + "</ht:news_item>".len())
                        .unwrap_or(entry.len());
                    let block = &entry[abs_pos..block_end];
                    if let Some(news_title) = extract_xml_tag(block, "ht:news_item_title") {
                        news_titles.push(news_title);
                        if news_titles.len() >= 2 { break; }
                    }
                    search_start = abs_pos + "<ht:news_item>".len();
                    if search_start >= entry.len() { break; }
                }

                if !news_titles.is_empty() {
                    let related = news_titles.iter()
                        .map(|t| format!("'{t}'"))
                        .collect::<Vec<_>>()
                        .join(" | ");
                    let enriched = format!("Related: {related}");
                    if desc.as_ref().map(|d| d.is_empty()).unwrap_or(true) {
                        desc = Some(enriched);
                    } else {
                        desc = Some(format!("{} — {}", desc.unwrap(), enriched));
                    }
                }

                items.push(TrendItem {
                    id: format!("google_{geo}_{}", title.to_lowercase().replace(' ', "_").chars().take(40).collect::<String>()),
                    title: title.clone(),
                    description: desc.map(|d| d.chars().take(300).collect()),
                    url: if link.is_empty() { format!("https://trends.google.com/trending?geo={geo}") } else { link },
                    score,
                    platform: "google".to_string(),
                    category: categorize_by_title(&title).to_string(),
                    timestamp: extract_xml_tag(entry, "pubDate").and_then(|d| {
                        chrono::DateTime::parse_from_rfc2822(&d).ok().map(|dt| dt.to_rfc3339())
                    }),
                    geos: Some(vec![geo.clone()]),
                });
            }
            Ok::<_, anyhow::Error>(items)
        }
    }).collect();

    let results: Vec<Result<Vec<TrendItem>>> = futures::future::join_all(futures).await;
    let mut all: Vec<TrendItem> = results.into_iter().filter_map(|r: Result<Vec<TrendItem>>| r.ok()).flatten().collect();

    // Deduplicate by title, merge geos
    let mut map: HashMap<String, TrendItem> = HashMap::new();
    for item in all.drain(..) {
        let key = item.title.to_lowercase();
        if let Some(existing) = map.get_mut(&key) {
            if let Some(ref geos) = item.geos {
                let mut eg: Vec<String> = existing.geos.clone().unwrap_or_default();
                eg.extend(geos.iter().cloned());
                eg.sort(); eg.dedup();
                existing.geos = Some(eg);
            }
            existing.score = existing.score.max(item.score);
        } else {
            map.insert(key, item);
        }
    }
    Ok(map.into_values().collect())
}

fn parse_traffic(s: &str) -> i64 {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == 'K' || *c == 'k' || *c == 'M' || *c == 'm').collect();
    if cleaned.contains('M') || cleaned.contains('m') {
        (cleaned.replace(['M', 'm'], "").parse::<f64>().unwrap_or(0.0) * 1_000_000.0) as i64
    } else if cleaned.contains('K') || cleaned.contains('k') {
        (cleaned.replace(['K', 'k'], "").parse::<f64>().unwrap_or(0.0) * 1_000.0) as i64
    } else {
        cleaned.parse::<i64>().unwrap_or(0)
    }
}

// --- YouTube ---

async fn fetch_youtube(client: &Client, api_key: Option<&str>) -> Result<Vec<TrendItem>> {
    let key = match api_key {
        Some(k) if !k.is_empty() => k,
        _ => return Ok(Vec::new()),
    };

    let regions = ["US", "GB"];
    let futures: Vec<_> = regions.iter().map(|region| {
        let client = client.clone();
        let key = key.to_string();
        let region = region.to_string();
        async move {
            let url = format!("https://youtube.googleapis.com/youtube/v3/videos?part=snippet,statistics&chart=mostPopular&regionCode={region}&maxResults=15&key={key}");
            let res = client.get(&url).send().await?;
            if !res.status().is_success() { return Ok(Vec::new()); }
            let data: serde_json::Value = res.json().await?;
            let items = data["items"].as_array().map(|arr| {
                arr.iter().map(|item| {
                    let video_id = item["id"].as_str().unwrap_or("");
                    let snippet = &item["snippet"];
                    let stats = &item["statistics"];
                    TrendItem {
                        id: format!("yt_{video_id}"),
                        title: snippet["title"].as_str().unwrap_or("").to_string(),
                        description: snippet["description"].as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.chars().take(200).collect()),
                        url: format!("https://youtube.com/watch?v={video_id}"),
                        score: stats["viewCount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0),
                        platform: "youtube".to_string(),
                        category: "entertainment".to_string(),
                        timestamp: snippet["publishedAt"].as_str().map(|s| s.to_string()),
                        geos: Some(vec![region.clone()]),
                    }
                }).collect::<Vec<_>>()
            }).unwrap_or_default();
            Ok::<_, anyhow::Error>(items)
        }
    }).collect();

    let results: Vec<Result<Vec<TrendItem>>> = futures::future::join_all(futures).await;
    let mut all: Vec<TrendItem> = results.into_iter().filter_map(|r: Result<Vec<TrendItem>>| r.ok()).flatten().collect();
    let mut seen = HashSet::new();
    all.retain(|item| seen.insert(item.url.clone()));
    Ok(all)
}

// --- Wikipedia ---

async fn fetch_wikipedia(client: &Client) -> Result<Vec<TrendItem>> {
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1)).format("%Y/%m/%d").to_string();
    let url = format!("https://wikimedia.org/api/rest_v1/metrics/pageviews/top/en.wikipedia/all-access/{yesterday}");
    let res = client.get(&url).send().await?;
    let data: serde_json::Value = res.json().await?;

    let skip: HashSet<&str> = ["Main_Page", "Special:Search", "-", "Wikipedia:Main_Page"].into_iter().collect();

    let articles = match data["items"][0]["articles"].as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let base_items: Vec<(String, String, i64, String)> = articles.iter()
        .filter(|a| {
            let article = a["article"].as_str().unwrap_or("");
            !skip.contains(article) && !article.starts_with("Special:") && !article.starts_with("Wikipedia:")
        })
        .take(30)
        .map(|a| {
            let article = a["article"].as_str().unwrap_or("").to_string();
            let title = article.replace('_', " ");
            let views = a["views"].as_i64().unwrap_or(0);
            let url = format!("https://en.wikipedia.org/wiki/{}", urlencoding_simple(&article));
            (article, title, views, url)
        })
        .collect();

    // Fetch summaries in batches of 10
    let mut items: Vec<TrendItem> = Vec::new();
    for chunk in base_items.chunks(10) {
        let summary_futures: Vec<_> = chunk.iter().map(|(article, title, views, wiki_url)| {
            let client = client.clone();
            let article = article.clone();
            let title = title.clone();
            let views = *views;
            let wiki_url = wiki_url.clone();
            async move {
                let summary_url = format!(
                    "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
                    urlencoding_simple(&article)
                );
                let description = match client.get(&summary_url).send().await {
                    Ok(res) if res.status().is_success() => {
                        match res.json::<serde_json::Value>().await {
                            Ok(json) => json["extract"].as_str()
                                .filter(|s| !s.is_empty())
                                .map(|s| s.chars().take(300).collect::<String>()),
                            Err(_) => None,
                        }
                    }
                    _ => None,
                };
                TrendItem {
                    id: format!("wiki_{article}"),
                    title: title.clone(),
                    description,
                    url: wiki_url,
                    score: views,
                    platform: "wikipedia".to_string(),
                    category: categorize_by_title(&title).to_string(),
                    timestamp: None,
                    geos: None,
                }
            }
        }).collect();

        let chunk_results = futures::future::join_all(summary_futures).await;
        items.extend(chunk_results);
    }

    Ok(items)
}

fn urlencoding_simple(s: &str) -> String {
    s.replace(' ', "_")
}

// --- News ---

async fn fetch_news(client: &Client) -> Result<Vec<TrendItem>> {
    let feeds = [
        ("https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml", "NYTimes", "general"),
        ("https://feeds.bbci.co.uk/news/rss.xml", "BBC", "general"),
        ("https://www.theguardian.com/world/rss", "Guardian", "politics"),
        ("https://feeds.arstechnica.com/arstechnica/index", "ArsTechnica", "tech"),
        ("https://variety.com/feed/", "Variety", "entertainment"),
        ("https://www.ign.com/articles.rss", "IGN", "entertainment"),
        ("https://feeds.bbci.co.uk/news/entertainment_and_arts/rss.xml", "BBC Ent", "entertainment"),
    ];

    let futures: Vec<_> = feeds.iter().map(|(url, source, category)| {
        let client = client.clone();
        let url = url.to_string();
        let source = source.to_string();
        let category = category.to_string();
        async move {
            let res = client.get(&url).send().await?;
            if !res.status().is_success() { return Ok(Vec::new()); }
            let xml = res.text().await?;
            let items: Vec<TrendItem> = xml.split("<item>").skip(1).take(10).enumerate().filter_map(|(idx, entry)| {
                let title = extract_xml_tag(entry, "title").unwrap_or_default();
                if title.is_empty() { return None; }
                let desc = extract_xml_tag(entry, "description").map(|d| strip_html(&d));
                let link = extract_xml_tag(entry, "link").unwrap_or_default();
                let pub_date = extract_xml_tag(entry, "pubDate");
                Some(TrendItem {
                    id: format!("news_{source}_{idx}"),
                    title,
                    description: desc.map(|d| d.chars().take(200).collect()),
                    url: if link.is_empty() { return None } else { link },
                    score: (20 - idx as i64 * 2).max(0),
                    platform: "news".to_string(),
                    category: category.clone(),
                    timestamp: pub_date.and_then(|d| {
                        chrono::DateTime::parse_from_rfc2822(&d).ok().map(|dt| dt.to_rfc3339())
                    }),
                    geos: None,
                })
            }).collect();
            Ok::<_, anyhow::Error>(items)
        }
    }).collect();

    let results: Vec<Result<Vec<TrendItem>>> = futures::future::join_all(futures).await;
    Ok(results.into_iter().filter_map(|r: Result<Vec<TrendItem>>| r.ok()).flatten().collect())
}

// --- Bluesky ---

async fn fetch_bluesky(client: &Client) -> Result<Vec<TrendItem>> {
    let url = "https://public.api.bsky.app/xrpc/app.bsky.feed.getFeed?feed=at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot&limit=30";
    let res = client.get(url).send().await?;
    if !res.status().is_success() { return Ok(Vec::new()); }
    let data: serde_json::Value = res.json().await?;

    let feed = match data["feed"].as_array() {
        Some(f) => f,
        None => return Ok(Vec::new()),
    };

    let mut items = Vec::new();
    for entry in feed {
        let post = &entry["post"];
        let text = post["record"]["text"].as_str().unwrap_or_default();
        if text.is_empty() { continue; }

        let uri = post["uri"].as_str().unwrap_or_default();
        // URI format: at://did:plc:xxx/app.bsky.feed.post/rkey
        let (did, rkey) = match parse_at_uri(uri) {
            Some(pair) => pair,
            None => continue,
        };

        let like_count = post["likeCount"].as_i64().unwrap_or(0);
        let repost_count = post["repostCount"].as_i64().unwrap_or(0);
        let score = like_count + repost_count;

        let (title, description) = smart_split(text);
        let bsky_url = format!("https://bsky.app/profile/{did}/post/{rkey}");
        let indexed_at = post["indexedAt"].as_str().map(|s| s.to_string());

        items.push(TrendItem {
            id: format!("bsky_{rkey}"),
            title,
            description: Some(description),
            url: bsky_url,
            score,
            platform: "bluesky".to_string(),
            category: categorize_by_title(text).to_string(),
            timestamp: indexed_at,
            geos: None,
        });
    }

    // Deduplicate by ID
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.id.clone()));
    Ok(items)
}

fn parse_at_uri(uri: &str) -> Option<(String, String)> {
    // at://did:plc:xxx/app.bsky.feed.post/rkey
    let stripped = uri.strip_prefix("at://")?;
    let parts: Vec<&str> = stripped.splitn(3, '/').collect();
    if parts.len() < 3 { return None; }
    let did = parts[0].to_string();
    let rkey = parts[2].to_string();
    Some((did, rkey))
}

// --- Mastodon ---

#[derive(Debug, Deserialize)]
struct MastodonStatus {
    id: String,
    content: String,
    url: String,
    reblogs_count: i64,
    favourites_count: i64,
    created_at: String,
}

async fn fetch_mastodon(client: &Client) -> Result<Vec<TrendItem>> {
    let res = client
        .get("https://mastodon.social/api/v1/trends/statuses?limit=20")
        .send()
        .await?;

    if !res.status().is_success() {
        return Ok(Vec::new());
    }

    let statuses: Vec<MastodonStatus> = res.json().await?;

    let items = statuses
        .into_iter()
        .map(|status| {
            let plain = strip_html(&status.content);
            let (title, description) = smart_split(&plain);
            let score = status.reblogs_count + status.favourites_count;
            let category = categorize_by_title(&title).to_string();
            let timestamp = chrono::DateTime::parse_from_rfc3339(&status.created_at)
                .ok()
                .map(|dt| dt.to_rfc3339());

            TrendItem {
                id: format!("masto_{}", status.id),
                title,
                description: Some(description),
                url: status.url,
                score,
                platform: "mastodon".to_string(),
                category,
                timestamp,
                geos: None,
            }
        })
        .collect();

    Ok(items)
}
