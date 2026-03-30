# Light Trend — Specification

> Version: 2.0 (macOS Native)
> Date: 2026-03-30
> Status: Implemented

---

## 1. Overview

A macOS menu bar app that aggregates trending topics from 8 platforms into a unified, ranked feed with cross-platform clustering. Click the tray icon to open, click again to hide. Native HudWindow vibrancy, zero API keys required (YouTube optional).

## 2. Goals

- **Multi-source trending**: Surface what's trending across Reddit, HN, GitHub, Google Trends, YouTube, Wikipedia, News, and Product Hunt
- **Zero friction**: Menu bar icon, one click to open/hide
- **Cross-platform intelligence**: Cluster same topics from different platforms, composite scoring
- **Lightweight**: Native macOS app via Tauri v2
- **Offline-first**: Cached data loads instantly, refresh on demand

## 3. Non-Goals

- No push notifications
- No user accounts or authentication
- No cross-platform support (macOS only)
- No Anthropic API key required

---

## 4. Architecture

### 4.1 Tech Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Desktop framework | Tauri v2 | Native macOS window, tray icon, vibrancy |
| Frontend | React 19 + TypeScript | Component model, fast dev |
| Styling | Tailwind CSS v4 | Utility-first |
| Backend (Rust) | reqwest + futures | Parallel HTTP fetching for all 8 platforms |
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
│  │  App.tsx │              │  lib.rs (tray)   │  │
│  └──────────┘              └───────┬──────────┘  │
│       │                            │              │
│   localStorage                     │              │
│   (cache)                          │              │
└────────────────────────────────────┼──────────────┘
                     ┌───────────────┼───────────────┐
                     ▼               ▼               ▼
              reddit.com      github.com       7 more APIs
              (JSON API)     (Search API)    (parallel fetch)
```

---

## 5. Data Pipeline

### 5.1 Platform Fetching

All 8 platforms fetched **in parallel** via `tokio::join!` in Rust. Each has an 8-second timeout. Individual failures don't block the aggregate.

| Platform | API | Auth | Items | Parallelism |
|----------|-----|------|-------|-------------|
| Reddit | Public JSON | None | ~150 | 4-batch subreddits |
| Hacker News | Firebase | None | 30 | 15-batch items |
| GitHub | Search API | None | 20 | Single request |
| Google Trends | RSS feed | None | ~45 | 5 geos parallel |
| YouTube | Data API v3 | Optional key | ~24 | 2 regions parallel |
| Wikipedia | Wikimedia | None | 30 | Single request |
| News | RSS feeds | None | ~60 | 7 feeds parallel |
| Product Hunt | GraphQL | None | 15 | Single request |

**Subreddits**: popular, all, technology, worldnews, science, movies, gaming, music, television, entertainment

**News RSS**: NYTimes, BBC, Guardian, Ars Technica, Variety, IGN, BBC Entertainment

**Google Trends geos**: US, GB, AU, CA, IN

### 5.2 Normalization

Per-platform scores normalized to 0–100, then multiplied by platform weight (capped at 100):

| Platform | Weight |
|----------|--------|
| Google Trends | 1.4 |
| YouTube | 1.3 |
| Reddit | 1.2 |
| Hacker News | 0.9 |
| Wikipedia | 0.8 |
| News | 0.7 |
| GitHub | 0.7 |
| Product Hunt | 0.6 |

### 5.3 Cross-Platform Clustering

Topics from different platforms about the same subject are merged using:
1. **Keyword overlap** (Set-based): 3+ matching keywords → merge
2. **Trigram similarity** (Dice coefficient): 2+ keywords AND similarity > 0.25 → merge

Same platform can only appear once per cluster.

### 5.4 Composite Scoring

```
compositeScore = normalizedScore + (mentions - 1) × 20 + recencyBonus
```

Recency bonus: +15 (<1h), +10 (<6h), +5 (<12h), 0 (older)

### 5.5 Categorization

8 categories via shared keyword dictionary:
All, Technology, Entertainment, Politics, Business, Science, Sports, General

---

## 6. IPC Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `fetch_trending` | `youtubeKey?: string` | `TrendingData` | Fetch all platforms, normalize, cluster, rank |

### TrendingData Schema (camelCase via serde)

```typescript
interface TrendingData {
  items: ClusteredItem[];           // Top 50 clustered topics
  byCategory: Record<string, ClusteredItem[]>;
  platformStats: Record<string, { count: number; name: string }>;
  totalItems: number;
  fetchedAt: string;                // ISO-8601
  elapsed: number;                  // ms
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

### 7.1 Design Principles

- Native macOS HudWindow vibrancy (frosted glass)
- Semi-transparent dark overlay `rgba(14,16,23,0.82)` for structure
- System font (-apple-system / SF Pro)
- 15px titles, 13px body text (macOS standard)
- Single accent color: blue-500
- Score colors: rose (>80), blue (>50), muted (below)
- Colored platform tags with per-platform identity

### 7.2 Window Properties

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
| Corner radius | 16px (via vibrancy) |

### 7.3 Layout

```
┌─────────────────────────────────────────────┐
│  Light Trend                    14:32  [Rfr] │ ← drag region
│─────────────────────────────────────────────│
│  All(30)  Tech(20)  Ent(20)  Pol(18)  ...    │ ← filter pills
│─────────────────────────────────────────────│
│  Filter topics…  /                           │ ← search
│─────────────────────────────────────────────│
│  1  Topic title that can wrap to            │
│     two lines if needed                      │
│     Description text in muted color…    85   │
│     [reddit] [hackernews] · 3 sources · 2h   │
│                                              │
│  2  Another trending topic across            │
│     multiple platforms                       │
│     Brief description here…              72   │
│     [google] [news] · 2 sources · 5h        │
│                                              │
│  ...                                         │
│─────────────────────────────────────────────│
│  7 sources          371 items          2.1s  │ ← status bar
└─────────────────────────────────────────────┘
```

### 7.4 States

| State | Display |
|-------|---------|
| Cached data | Instant display from localStorage |
| Loading (no cache) | Spinner + "Scanning networks…" |
| Error (no data) | Error message + Retry button |
| Refreshing (has data) | Button shows ↻, existing data stays |
| Empty category | "No topics in this category" |
| Empty search | "No match for …" |

### 7.5 Expanded Detail Panel

Clicking a topic expands inline:
- Platform distribution bars (colored per platform)
- Geo region tags (if available)
- Full description text
- Source links (per platform)

---

## 8. Caching

- **Key**: `lt_cache` in localStorage
- **Contents**: Full `TrendingData` JSON
- **Behavior**: Load from cache instantly on open, fetch fresh in background
- **Corruption**: Invalid JSON is detected and evicted
- **YouTube API key**: Stored in `youtube_api_key` localStorage key

---

## 9. File Structure

```
light-trend-app/
├── SPEC.md
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── index.html
├── src/
│   ├── main.tsx              # React entry
│   ├── App.tsx               # Main app (filter, search, card list)
│   ├── types.ts              # TypeScript interfaces
│   └── index.css             # Global styles
└── src-tauri/
    ├── Cargo.toml            # Rust dependencies
    ├── tauri.conf.json       # Window config, tray, permissions
    ├── capabilities/
    │   └── default.json      # Tauri v2 permissions
    ├── icons/
    │   ├── tray-icon.png     # Menu bar icon (template)
    │   ├── icon.png          # App icon 1024px
    │   ├── icon.icns         # macOS app icon
    │   └── ...               # Various sizes
    └── src/
        ├── main.rs           # Binary entry point
        ├── lib.rs            # Tauri setup, tray, vibrancy
        └── platforms.rs      # All 8 platform fetchers + clustering
```

---

## 10. Environment

| Variable | Required | Description |
|----------|----------|-------------|
| `YOUTUBE_API_KEY` | No | Set in localStorage via browser console. If absent, YouTube skipped. |

---

## 11. Running

```bash
# Development (hot reload)
npm run tauri dev

# Build for production
npm run tauri build
```

---

## 12. Evolution from v1

Light Trend v1 was a Node.js web server at localhost:3000. v2 migrates to:

| v1 (Web) | v2 (Tauri) |
|----------|-----------|
| Node.js HTTP server | Rust native backend |
| Browser at localhost | macOS menu bar tray icon |
| CSS glassmorphism | Native HudWindow vibrancy |
| JS fetch API | reqwest + tokio parallel |
| In-memory cache (TTL) | localStorage persistent |
| Inline HTML/CSS/JS | React + TypeScript + Tailwind |
| Platform logic in JS | Platform logic in Rust |

The web version is preserved at `/Users/meco/light-trend/` and on GitHub as the `light-trend` repo.
