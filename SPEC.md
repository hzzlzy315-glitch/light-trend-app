# Light Trend — Specification

> Version: 3.0 (Neumorphic Dashboard)
> Date: 2026-03-30
> Status: Implemented

---

## 1. Overview

A macOS menu bar app that aggregates trending topics from 8 platforms into a unified, ranked feed with cross-platform clustering, content enrichment, and a neumorphic dashboard UI. Click the tray icon to open, click outside to auto-hide. Designed for content creators who need to know what the world is talking about.

## 2. Goals

- **Content-rich trending**: Context, summaries, related news for every topic
- **Cross-platform intelligence**: Cluster same topics, composite scoring, detective enrichment
- **Beautiful dashboard**: Neumorphic soft UI with pie charts, score gauges, and spacious layout
- **Zero friction**: Menu bar icon, auto-hide on focus loss
- **8 free platforms**: No paid APIs required (YouTube optional)

## 3. Non-Goals

- No push notifications
- No user accounts
- No cross-platform (macOS only)
- No developer-focused platforms (removed GitHub, Product Hunt, Spotify)

---

## 4. Architecture

### 4.1 Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | Tauri v2 |
| Frontend | React 19 + TypeScript + Tailwind CSS v4 |
| Backend | Rust (reqwest + futures + tokio) |
| Env | dotenvy (.env loading) |
| Font | Nunito (Google Fonts) |
| Build | Vite 7 |

### 4.2 Platforms (8 total)

| Platform | Weight | Content Enrichment |
|----------|--------|-------------------|
| YouTube | **1.4** | Title + description + views |
| Google Trends | **1.2** | Topic + parsed `<ht:news_item>` article titles |
| Reddit | **1.2** | Title + selftext + upvotes |
| Hacker News | 0.9 | Title + extracted post text |
| Wikipedia | **0.9** | Title + page summary API (parallel batch) |
| Mastodon | 0.6 | Smart sentence-split title + full post |
| Bluesky | 0.6 | Smart sentence-split title + full post |
| News RSS | 0.3 | Validation-only (not standalone) |

---

## 5. Scoring Algorithm

```
compositeScore = normalizedScore + (mentions-1)×20 + recencyBonus + richnessModifier
```

- Normalized: per-platform 0-100 × weight (capped at 100)
- Cross-platform: +20 per additional platform
- Recency: +15 (<1h), +10 (<6h), +5 (<12h)
- Richness: -10 (no description), -5 (sparse), 0 (rich)
- Detective backfill: empty descriptions filled by keyword-matching other platforms

---

## 6. UI Design — Neumorphic Dashboard

### 6.1 Design Language

- **Style**: Neumorphism / Soft UI — elements embossed from surface via dual shadows
- **Background**: `#dcdee6` (cool blue-gray)
- **Card surface**: `#ecedf3` (lighter, raised from background)
- **Inset surface**: `#d4d6de` (darker, pressed into background)
- **Font**: Nunito (rounded, soft)
- **Accent**: Purple `#7c5cfc`, Pink/Hot `#e84393`

### 6.2 Shadow System

| Level | CSS |
|-------|-----|
| Raised | `8px 8px 20px #d1d3da, -8px -8px 20px #ffffff` |
| Small | `4px 4px 12px #d1d3da, -4px -4px 12px #ffffff` |
| Inset | `inset 2px 2px 6px #d1d3da, inset -2px -2px 6px #ffffff` |
| Hover | `10px 10px 24px #d1d3da, -10px -10px 24px #ffffff` |

### 6.3 Window

| Property | Value |
|----------|-------|
| Default size | 1340 × 860 |
| Min size | 900 × 640 |
| Corner radius | 24px (rounded-3xl) |
| Decorations | None (custom drag region) |
| Always on top | Yes |
| Auto-hide | On focus loss |
| Dock icon | None (Accessory policy) |
| Vibrancy | None (opaque neumorphic background) |
| Content padding | 20px top/bottom, 32px left/right |

### 6.4 Layout (Left-Right Dashboard)

```
┌──────────────────────────────────────────────────────────────┐
│  Light Trend                           Updated 14:32  [Rfr]  │
├──────────────────────────────────────────────────────────────┤
│  [Topics 42] [Sources 8] [Top Score 127] [Avg 65] [Raw 371] │
├──────────────────────────────────────────────────────────────┤
│  [All] [Tech] [Ent.] [Politics] [Biz] [Sci] [Sports] [Gen]  │
│                                              [Filter topics] │
├────────────────────────────────┬─────────────────────────────┤
│  Topic List (scrollable)       │  Platform Distribution      │
│                                │  [Donut Pie Chart]          │
│  1  Topic title here...   ◉85 │                              │
│  2  Another topic...      ◉72 │  Topic Detail                │
│  3  Third topic...        ◉68 │  [Score Gauge 72px]          │
│  ...                           │  Title + timestamp           │
│                                │  Description                 │
│                                │  [Platform bars]             │
│                                │  [Geo tags]                  │
│                                │  [Links ↗]                   │
├────────────────────────────────┴─────────────────────────────┤
│            8 sources  •  371 raw items  •  2.1s fetch        │
└──────────────────────────────────────────────────────────────┘
```

### 6.5 Category Filter Tabs

Each category has a permanent color — always visible, not just when active:

| Category | Color | Background |
|----------|-------|-----------|
| All | `#7c5cfc` | `#ece7ff` |
| Tech | `#1a73e8` | `#e8f0fe` |
| Entertainment | `#e84393` | `#fce4f0` |
| Politics | `#e17055` | `#ffeae4` |
| Business | `#00b894` | `#e6fcf5` |
| Science | `#0984e3` | `#e0f3ff` |
| Sports | `#fdcb6e` | `#fff8e1` |
| General | `#6b6b80` | `#f0f0f2` |

Active tab: inset shadow (pressed). Inactive: raised shadow (embossed).

### 6.6 Visualizations

- **Platform Distribution Pie**: SVG donut chart in right panel, showing item count per platform with legend
- **Score Arc Gauge**: 48px per topic row, 72px in detail panel. 240° arc, color-coded (pink >80, purple >50, gray below)

### 6.7 Interactions

- Click topic row → selects it, shows detail in right panel
- Click outside app → auto-hide (focus loss detection)
- Click tray icon → toggle show/hide
- "/" key → focus search
- Escape → deselect topic
- Links open in default browser via Tauri opener plugin

---

## 7. Environment

| Variable | Required | Source |
|----------|----------|-------|
| `YOUTUBE_API_KEY` | Optional | `.env` file or environment variable |

---

## 8. File Structure

```
light-trend-app/
├── SPEC.md
├── DESIGN_SPEC.md
├── .env                    # YouTube API key (gitignored)
├── package.json
├── vite.config.ts
├── index.html              # Loads Nunito font
├── src/
│   ├── main.tsx
│   ├── App.tsx             # Dashboard layout + all components
│   ├── ScoreGauge.tsx      # SVG arc gauge component
│   ├── PlatformPie.tsx     # SVG donut chart component
│   ├── types.ts            # TypeScript interfaces
│   └── index.css           # Neumorphic design tokens + utilities
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/default.json
    ├── icons/
    └── src/
        ├── main.rs
        ├── lib.rs           # Tauri setup, tray, auto-hide, dotenv
        └── platforms.rs     # 8 fetchers + enrichment + clustering
```

---

## 9. Version History

| Version | Changes |
|---------|---------|
| 1.0 | Web version (Node.js server) |
| 2.0 | Tauri v2 macOS native, 11 platforms |
| 2.1 | Content enrichment, removed GitHub/PH/Spotify, 8 platforms |
| **3.0** | **Neumorphic dashboard UI, left-right layout, pie chart, score gauges, Nunito font, auto-hide, colored category tabs, 1340×860 window** |
