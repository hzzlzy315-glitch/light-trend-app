# Light Trend — Specification

> Version: 2.1 (macOS Native)
> Date: 2026-03-30
> Status: Implemented

---

## 1. Overview

A macOS menu bar app that aggregates trending topics from 8 platforms into a unified, ranked feed with cross-platform clustering and content enrichment. Click the tray icon to open, click again to hide. Native HudWindow vibrancy, designed for content creators who need to know what the world is talking about.

## 2. Goals

- **Content-rich trending**: Not just titles — context, summaries, related news for every topic
- **Cross-platform intelligence**: Cluster same topics from different platforms, composite scoring
- **Detective enrichment**: Use data from one platform to fill gaps in another
- **Zero friction**: Menu bar icon, one click to open/hide
- **Lightweight**: Native macOS app via Tauri v2

## 3. Non-Goals

- No push notifications
- No user accounts or authentication
- No cross-platform support (macOS only)
- No developer-focused platforms (removed GitHub, Product Hunt)

---

## 4. Architecture

### 4.1 Tech Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Desktop framework | Tauri v2 | Native macOS window, tray icon, vibrancy |
| Frontend | React 19 + TypeScript | Component model, fast dev |
| Styling | Tailwind CSS v4 | Utility-first |
| Backend (Rust) | reqwest + futures + tokio | Parallel HTTP fetching for all 8 platforms |
| Env loading | dotenvy | Loads .env for API keys |
| Window effects | window-vibrancy | Native macOS HudWindow material |
| Build tool | Vite 7 | Fast HMR, Tauri-compatible |

### 4.2 System Diagram

```
                    macOS Menu Bar
                         │
                    [Tray Icon] ← click to toggle
                         │
┌────────────────────────┴─────────────────────────┐
│                  Tauri Shell                      │
│           (HudWindow vibrancy, 16px radius)       │
│                                                   │
│  ┌──────────┐     IPC      ┌──────────────────┐  │
│  │  React   │◄────────────►│    Rust Core     │  │
│  │  WebView │  (commands)  │                  │  │
│  │          │              │  platforms.rs    │  │
│  │  App.tsx │              │  (8 fetchers +   │  │
│  │          │              │   enrichment)    │  │
│  └──────────┘              └───────┬──────────┘  │
│       │                            │              │
│   localStorage                  .env file         │
│   (cache)                    (API keys)           │
└────────────────────────────────────┼──────────────┘
                     ┌───────────────┼───────────────┐
                     ▼               ▼               ▼
              reddit.com      youtube.com       6 more APIs
              (JSON API)     (Data API v3)    (parallel fetch)
```

---

## 5. Data Pipeline

### 5.1 Platform Sources (8 total)

All platforms fetched **in parallel** via `tokio::join!`. Each has an 8-second timeout.

| Platform | API | Auth | Weight | Content Quality |
|----------|-----|------|--------|----------------|
| YouTube | Data API v3 | API key (env) | **1.4** | Title + description + views |
| Google Trends | RSS feed | None | **1.2** | Topic + related news article titles (enriched) |
| Reddit | Public JSON | None | **1.2** | Title + selftext + upvotes |
| Hacker News | Firebase | None | 0.9 | Title + post text (enriched) |
| Wikipedia | Wikimedia | None | **0.9** | Title + page summary (enriched via API) |
| Mastodon | Public API | None | 0.6 | Smart sentence-split title + full post |
| Bluesky | AT Protocol | None | 0.6 | Smart sentence-split title + full post |
| News RSS | RSS feeds | None | 0.3 | Title + description (validation only) |

**Removed platforms**: GitHub (dev niche), Product Hunt (startup niche), Spotify (needs paid credentials)

### 5.2 Content Enrichment Engine

The "detective" approach — use already-fetched data to fill content gaps:

1. **Google Trends**: RSS contains `<ht:news_item>` blocks with linked news article titles. Parsed and concatenated as description: `"Related: 'Article title 1' | 'Article title 2'"`

2. **Wikipedia**: After fetching top 30 articles, parallel batch requests (3 × 10) to `en.wikipedia.org/api/rest_v1/page/summary/{title}` for 1-2 sentence plain-text summaries.

3. **Mastodon / Bluesky**: Smart sentence-boundary splitting instead of hard 100-char cut. Finds first `.!?\n` within 120 chars for title, full text for description.

4. **Hacker News**: Extracts `text` field from post JSON (for Ask HN, Show HN posts).

5. **Cross-platform backfill**: After clustering, any topic still missing a description gets keyword-matched against ALL fetched items. If another item shares 2+ keywords and has a description ≥20 chars, that description is borrowed.

### 5.3 Scoring Algorithm

#### Step 1: Normalize (per-platform, 0-100)
```
normalizedScore = (item.score / platform_max_score) × 100 × platform_weight
```
Capped at 100. Platforms without real engagement metrics (News) use position-based scoring.

#### Step 2: Cluster (cross-platform)
- Keyword overlap: 3+ matching keywords → merge
- Trigram similarity (Dice coefficient): 2+ keywords AND similarity > 0.25 → merge
- Same platform can only appear once per cluster

#### Step 3: Composite Score
```
compositeScore = normalizedScore
               + (mentions - 1) × 20        // cross-platform bonus
               + recencyBonus               // +15/+10/+5/+0
               + richnessModifier           // 0/-5/-10
```

- **Cross-platform bonus**: +20 per additional platform
- **Recency**: +15 (<1h), +10 (<6h), +5 (<12h)
- **Content richness penalty**: description ≥50 chars → 0, <50 chars → -5, none → -10

### 5.4 Categorization

8 categories via shared keyword dictionary:
All, Technology, Entertainment, Politics, Business, Science, Sports, General

---

## 6. IPC Commands

| Command | Parameters | Returns |
|---------|-----------|---------|
| `fetch_trending` | `youtubeKey?: string` | `TrendingData` |

YouTube API key resolution order:
1. Frontend parameter (from localStorage)
2. Environment variable `YOUTUBE_API_KEY`
3. `.env` file (loaded via dotenvy at startup)

### Data Schema (camelCase via serde)

```typescript
interface TrendingData {
  items: ClusteredItem[];
  byCategory: Record<string, ClusteredItem[]>;
  platformStats: Record<string, { count: number; name: string }>;
  totalItems: number;
  fetchedAt: string;
  elapsed: number;
}

interface ClusteredItem {
  id: string;
  title: string;
  description: string | null;
  url: string;
  score: number;
  platform: string;
  platforms: string[];
  platformDetails: { platform: string; score: number; url: string }[];
  mentions: number;
  category: string;
  timestamp: string | null;
  geos: string[] | null;
  normalizedScore: number;
  compositeScore: number;
}
```

---

## 7. UI Specification

### 7.1 Window Properties

| Property | Value |
|----------|-------|
| Default size | 500 × 780 |
| Min size | 400 × 520 |
| Max size | 700 × 960 |
| Resizable | Yes |
| Always on top | Yes |
| Decorations | None (custom drag region) |
| Transparent | Yes (native vibrancy) |
| Dock icon | None (Accessory policy) |

### 7.2 Design

- Semi-transparent dark overlay on native HudWindow vibrancy
- System font (-apple-system / SF Pro)
- 15px titles (2-line wrap), 13px descriptions
- Score colors: rose (>80), blue (>50), muted (below)
- Colored platform tags per platform identity
- Top 3 items have amber left border accent

### 7.3 Layout

Header (drag region) → Filter pills → Search → Topic list → Status bar

### 7.4 Expanded Detail

- Platform distribution bars (colored)
- Geo region tags
- Full description
- Source links per platform

---

## 8. Environment

| Variable | Required | Source |
|----------|----------|-------|
| `YOUTUBE_API_KEY` | Optional | `.env` file or environment variable |

YouTube quota: 10,000 units/day (free). Each refresh = 2 units. Monitor at [Google Cloud Console](https://console.cloud.google.com/apis/api/youtube.googleapis.com/quotas).

---

## 9. File Structure

```
light-trend-app/
├── SPEC.md
├── .env                   # YouTube API key (gitignored)
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── src/
│   ├── main.tsx
│   ├── App.tsx            # Main app (filter, search, cards)
│   ├── types.ts           # TypeScript interfaces
│   └── index.css          # Global styles
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/default.json
    ├── icons/
    └── src/
        ├── main.rs
        ├── lib.rs          # Tauri setup, tray, vibrancy, dotenv
        └── platforms.rs    # 8 fetchers + enrichment + clustering
```

---

## 10. Running

```bash
# Development
npm run tauri dev

# Production build
npm run tauri build
```

---

## 11. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-30 | Web version (Node.js server at localhost:3000) |
| 2.0 | 2026-03-30 | Tauri v2 macOS native, 11 platforms |
| 2.1 | 2026-03-30 | Content enrichment engine, removed GitHub/PH/Spotify, 8 platforms, detective cross-fill, richness penalty |
