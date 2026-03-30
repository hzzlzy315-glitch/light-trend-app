# Light Trend -- Neumorphic UI Redesign Specification

Complete design specification for the neumorphic / soft UI redesign of the Light Trend macOS menu bar app. Every value in this document is implementation-ready.

---

## 1. Vibrancy Decision

**Switch from HudWindow vibrancy to an opaque light background.**

Neumorphism requires a consistent, flat background color to make the embossed/debossed shadow illusion work. macOS `HudWindow` material is a dark translucent blur that will fight against light neumorphic shadows and destroy the effect. The shadows become invisible or inverted on a translucent dark surface.

**Rust change required** in `src-tauri/src/lib.rs`:

Replace `NSVisualEffectMaterial::HudWindow` with either:
- Option A (recommended): Remove vibrancy entirely. Set window background to opaque via CSS. The `transparent: true` in `tauri.conf.json` allows the CSS background to be the surface.
- Option B: Use `NSVisualEffectMaterial::ContentBackground` which is light, but still introduces translucency that competes with neumorphic shadows.

**Recommendation: Option A.** Remove the `apply_vibrancy` call. Set the root div to the solid neumorphic background color. Keep `transparent: true` and `shadow: true` in `tauri.conf.json` so the window itself still has the native macOS shadow and rounded corners, but the content is opaque light.

---

## 2. Color System

### Base Surface

| Token | Hex | Usage |
|-------|-----|-------|
| `--bg-base` | `#e8e8ee` | Main window background. The "canvas" for neumorphism. |
| `--bg-surface` | `#e8e8ee` | Cards and raised elements (same as base -- depth comes from shadow, not color difference). |
| `--bg-inset` | `#dfe0e6` | Pressed/inset surfaces (active pills, pressed buttons). Slightly darker to reinforce the "pushed in" illusion. |

### Text Hierarchy

| Token | Hex | Usage |
|-------|-----|-------|
| `--text-primary` | `#2d2d3a` | Titles, primary content. High contrast on light bg. |
| `--text-secondary` | `#6b6b80` | Descriptions, secondary labels. |
| `--text-tertiary` | `#9a9ab0` | Timestamps, metadata, subtle info. |
| `--text-muted` | `#b8b8cc` | Disabled states, very subtle text. |

### Accent Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--accent-primary` | `#7c5cfc` | Primary accent (purple). Active states, links, primary buttons. |
| `--accent-primary-soft` | `#ece7ff` | Light purple background for tags/badges. |
| `--accent-secondary` | `#e84393` | Secondary accent (pink/magenta). Score highlights, hot items. |
| `--accent-secondary-soft` | `#fce4f0` | Light pink background for hot badges. |

### Semantic Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-success` | `#00b894` | Positive indicators. |
| `--color-warning` | `#fdcb6e` | Medium-range scores. |
| `--color-error` | `#e17055` | Error states. |
| `--color-info` | `#74b9ff` | Informational. |

### Score Colors (for composite score display)

| Range | Color | Token |
|-------|-------|-------|
| 80-150 (hot) | `#e84393` | `--score-hot` |
| 50-79 (warm) | `#7c5cfc` | `--score-warm` |
| 0-49 (cool) | `#9a9ab0` | `--score-cool` |

### Platform Tag Colors (preserved from current, adjusted for light bg)

```typescript
const PLATFORM_COLORS: Record<string, { text: string; bg: string }> = {
  reddit:      { text: "#d63031", bg: "#ffeaea" },
  hackernews:  { text: "#cc5500", bg: "#fff3e6" },
  github:      { text: "#586069", bg: "#eef0f2" },
  google:      { text: "#1a73e8", bg: "#e8f0fe" },
  youtube:     { text: "#cc0000", bg: "#ffe0e0" },
  wikipedia:   { text: "#72777d", bg: "#f0f0f2" },
  news:        { text: "#b8860b", bg: "#fff8e1" },
  producthunt: { text: "#da552f", bg: "#fde8e2" },
  mastodon:    { text: "#5850ec", bg: "#eeedff" },
  bluesky:     { text: "#0066cc", bg: "#e0f0ff" },
};
```

---

## 3. Neumorphic Shadow System

The core of neumorphism is two opposing shadows on a flat-colored element: a light shadow (top-left, suggesting light source) and a dark shadow (bottom-right, suggesting depth). The element has the SAME background color as the surface it sits on.

### Shadow Values

```css
/* Raised surface -- cards at rest, stat cards, filter bar */
--shadow-raised: 6px 6px 12px #c8c8d2, -6px -6px 12px #ffffff;

/* Raised surface (subtle) -- topic rows */
--shadow-raised-sm: 3px 3px 6px #c8c8d2, -3px -3px 6px #ffffff;

/* Pressed/inset surface -- active filter pills, pressed buttons */
--shadow-inset: inset 3px 3px 6px #c8c8d2, inset -3px -3px 6px #ffffff;

/* Hover state -- slightly more pronounced raise */
--shadow-hover: 8px 8px 16px #c0c0cc, -8px -8px 16px #ffffff;

/* Flat (no shadow) -- for seamless inline elements */
--shadow-flat: none;
```

### Shadow Color Rationale

The dark shadow (`#c8c8d2`) is the base color `#e8e8ee` darkened by approximately 13%. The light shadow is pure `#ffffff`. This creates the embossed look on the `#e8e8ee` background. If the base background changes, both shadow colors must be recalculated.

### Applied as Tailwind Arbitrary Values

Since Tailwind v4 supports arbitrary values directly:

```html
<!-- Raised card -->
<div class="rounded-2xl" style="background: #e8e8ee; box-shadow: 6px 6px 12px #c8c8d2, -6px -6px 12px #ffffff;">

<!-- Inset/pressed -->
<button class="rounded-xl" style="background: #dfe0e6; box-shadow: inset 3px 3px 6px #c8c8d2, inset -3px -3px 6px #ffffff;">
```

Or define CSS custom properties in `index.css` and reference them (recommended approach, detailed in section 11).

---

## 4. Typography System

Using the system font stack already in place (`-apple-system, BlinkMacSystemFont, 'SF Pro Text'`). This is correct for a macOS menu bar app.

| Element | Size | Weight | Color Token | Line Height |
|---------|------|--------|-------------|-------------|
| App title ("Light Trend") | 16px | 600 (semibold) | `--text-primary` | 1.2 |
| Stat card number | 28px | 700 (bold) | `--text-primary` | 1.0 |
| Stat card label | 11px | 500 (medium) | `--text-tertiary` | 1.2 |
| Stat card sub-label | 10px | 400 (normal) | `--text-muted` | 1.2 |
| Filter pill text | 13px | 500 (medium) | `--text-secondary` (inactive), `--accent-primary` (active) | 1.2 |
| Search input | 14px | 400 (normal) | `--text-primary` | 1.4 |
| Search placeholder | 14px | 400 (normal) | `--text-muted` | 1.4 |
| Topic rank number | 13px | 600 (semibold) | `--text-muted` (normal), `#daa520` (top 3) | 1.0 |
| Topic title | 14px | 500 (medium) | `--text-primary` | 1.4 |
| Topic description | 12px | 400 (normal) | `--text-secondary` | 1.3 |
| Topic meta (platform tags) | 11px | 500 (medium) | Per-platform color | 1.2 |
| Topic timestamp | 11px | 400 (normal) | `--text-tertiary` | 1.2 |
| Topic score number | 14px | 700 (bold) | Score color per range | 1.0 |
| Status bar text | 11px | 400 (normal) | `--text-tertiary` | 1.2 |
| Detail section header | 11px | 600 (semibold) | `--text-tertiary`, uppercase, tracking-wide | 1.2 |
| Detail body text | 13px | 400 (normal) | `--text-secondary` | 1.5 |

---

## 5. Layout Architecture (500x780)

```
+--------------------------------------------------+ 0px
|  HEADER (drag region)                      48px  |
|  "Light Trend"          [time]  [Refresh]        |
+--------------------------------------------------+ 48px
|  STAT CARDS ROW                            88px  |
|  [ Topics ]  [ Sources ]  [ Score ]              |
|   count        count        avg/max              |
+--------------------------------------------------+ 136px
|  FILTER PILLS                              44px  |
|  [All] [Tech] [Entertainment] ... (scroll)       |
+--------------------------------------------------+ 180px
|  SEARCH BAR                                44px  |
|  [ Filter topics...  / ]                         |
+--------------------------------------------------+ 224px
|                                                  |
|  TOPIC LIST (scrollable)                  ~520px |
|  +-----------------------------------------+     |
|  | #1  Topic Title              Score  [=] |     |
|  |     description...                      |     |
|  |     [reddit] [github]  2 src  3m        |     |
|  +-----------------------------------------+     |
|  | #2  ...                                 |     |
|  +-----------------------------------------+     |
|  | ...                                     |     |
|                                                  |
+--------------------------------------------------+ ~744px
|  STATUS BAR                                36px  |
|  8 sources    42 items    1.2s                   |
+--------------------------------------------------+ 780px
```

### Spacing Values

| Area | Horizontal Padding | Vertical Padding | Gap Between Elements |
|------|-------------------|------------------|---------------------|
| Header | 20px (px-5) | centered in 48px height | -- |
| Stat cards row | 16px (px-4) outer, 8px gap between cards | 12px top, 12px bottom | 8px gap |
| Filter pills | 20px (px-5) | 10px top/bottom | 6px gap |
| Search bar | 20px (px-5) | 8px top/bottom | -- |
| Topic list | 16px (px-4) | 12px bottom | 6px gap between rows |
| Status bar | 20px (px-5) | 8px top/bottom | -- |

---

## 6. Stat Cards Design

Three stat cards in a row. Each card is a neumorphic raised element.

### Card 1: Topics

| Property | Value |
|----------|-------|
| Number | Total count of items in current category view (e.g., "42") |
| Label | "Topics" |
| Sub-label | Category name if not "All" (e.g., "in Tech") |

### Card 2: Sources

| Property | Value |
|----------|-------|
| Number | Count of active platforms from `platformStats` (e.g., "8") |
| Label | "Sources" |
| Sub-label | "active" |

### Card 3: Top Score

| Property | Value |
|----------|-------|
| Number | Highest `compositeScore` in current view (e.g., "127") |
| Label | "Top Score" |
| Sub-label | Truncated title of the top item (e.g., "OpenAI...") |

### Card CSS Specification

```css
.stat-card {
  flex: 1;
  min-width: 0;
  padding: 14px 16px;
  border-radius: 16px;
  background: #e8e8ee;
  box-shadow: 6px 6px 12px #c8c8d2, -6px -6px 12px #ffffff;
  display: flex;
  flex-direction: column;
  gap: 2px;
  transition: box-shadow 0.2s ease;
}

.stat-card:hover {
  box-shadow: 8px 8px 16px #c0c0cc, -8px -8px 16px #ffffff;
}

.stat-card .number {
  font-size: 28px;
  font-weight: 700;
  color: #2d2d3a;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.stat-card .label {
  font-size: 11px;
  font-weight: 500;
  color: #9a9ab0;
  margin-top: 4px;
}

.stat-card .sub-label {
  font-size: 10px;
  font-weight: 400;
  color: #b8b8cc;
}
```

---

## 7. Filter Pills Design

Horizontally scrollable row. Each pill toggles between raised (inactive) and inset (active).

### Inactive Pill

```css
.pill {
  padding: 6px 14px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 500;
  color: #6b6b80;
  background: #e8e8ee;
  box-shadow: 3px 3px 6px #c8c8d2, -3px -3px 6px #ffffff;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  flex-shrink: 0;
}

.pill:hover {
  color: #7c5cfc;
}
```

### Active Pill (pressed in)

```css
.pill--active {
  color: #7c5cfc;
  background: #dfe0e6;
  box-shadow: inset 3px 3px 6px #c8c8d2, inset -3px -3px 6px #ffffff;
}
```

### Count Badge

The topic count appears as a small number next to the label. When active, it shares the accent color at reduced opacity.

```css
.pill .count {
  font-size: 10px;
  margin-left: 4px;
  color: #b8b8cc; /* inactive */
}

.pill--active .count {
  color: rgba(124, 92, 252, 0.5); /* accent at 50% */
}
```

---

## 8. Search Bar Design

Neumorphic inset input field.

```css
.search-input {
  width: 100%;
  height: 36px;
  padding: 0 16px;
  border-radius: 12px;
  border: none;
  font-size: 14px;
  font-weight: 400;
  color: #2d2d3a;
  background: #dfe0e6;
  box-shadow: inset 3px 3px 6px #c8c8d2, inset -3px -3px 6px #ffffff;
  outline: none;
  transition: box-shadow 0.2s ease;
}

.search-input::placeholder {
  color: #b8b8cc;
}

.search-input:focus {
  box-shadow: inset 3px 3px 6px #c8c8d2, inset -3px -3px 6px #ffffff, 0 0 0 2px rgba(124, 92, 252, 0.3);
}
```

---

## 9. Topic Card Design

Each topic row is a neumorphic raised element. At 500px width, the layout is:

```
+---+------------------------------------+--------+
| # | Title (up to 2 lines)              | Score  |
|   | Description (1 line, truncated)    | [arc]  |
|   | [platform] [platform]  2src  3m    |        |
+---+------------------------------------+--------+
```

### Score Visualization (Right Side) -- Recommendation

**Option C: Score number with a subtle SVG arc/gauge** is the best choice for 500px width.

Rationale:
- Option A (mini donut for platform distribution) requires at least 40px diameter to be readable, and showing 2-8 platform slices at that size is illegible.
- Option B (stacked bar) works but is visually boring and adds visual noise to every row.
- Option C (arc gauge) gives an at-a-glance "temperature" reading that is instantly scannable. The score number sits inside/above a 120-degree arc. The arc fills proportionally to the score (0-150 scale). Color follows the score range.

### Arc Gauge SVG Component

The gauge is a 36x22px SVG showing a 120-degree arc. The arc stroke fills from left to right based on score/150.

```typescript
function ScoreGauge({ score }: { score: number }) {
  const pct = Math.min(score / 150, 1);
  const color = score > 80 ? "#e84393" : score > 50 ? "#7c5cfc" : "#9a9ab0";
  // Arc: 120 degrees centered at bottom, radius 14, centered at (18, 18)
  const r = 14;
  const startAngle = -150 * (Math.PI / 180); // -150 deg
  const endAngle = -30 * (Math.PI / 180);    // -30 deg
  const sweepAngle = startAngle + (endAngle - startAngle) * pct;

  const x1 = 18 + r * Math.cos(startAngle);
  const y1 = 18 + r * Math.sin(startAngle);
  const x2 = 18 + r * Math.cos(sweepAngle);
  const y2 = 18 + r * Math.sin(sweepAngle);
  const largeArc = pct > 0.5 ? 1 : 0;

  const bgX2 = 18 + r * Math.cos(endAngle);
  const bgY2 = 18 + r * Math.sin(endAngle);

  return (
    <svg width="36" height="22" viewBox="0 0 36 22">
      {/* Background arc (track) */}
      <path
        d={`M ${x1} ${y1} A ${r} ${r} 0 1 1 ${bgX2} ${bgY2}`}
        fill="none"
        stroke="#d5d5e0"
        strokeWidth="3"
        strokeLinecap="round"
      />
      {/* Value arc */}
      {pct > 0.01 && (
        <path
          d={`M ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2}`}
          fill="none"
          stroke={color}
          strokeWidth="3"
          strokeLinecap="round"
        />
      )}
    </svg>
  );
}
```

### Topic Row CSS Specification

```css
.topic-row {
  display: flex;
  align-items: flex-start;
  padding: 12px 14px;
  border-radius: 16px;
  background: #e8e8ee;
  box-shadow: 3px 3px 6px #c8c8d2, -3px -3px 6px #ffffff;
  cursor: pointer;
  transition: all 0.2s ease;
  gap: 10px;
}

.topic-row:hover {
  box-shadow: 5px 5px 10px #c0c0cc, -5px -5px 10px #ffffff;
}

/* When expanded, connect visually to the detail panel */
.topic-row--expanded {
  border-radius: 16px 16px 0 0;
}
```

### Rank Number

```css
.topic-rank {
  font-size: 13px;
  font-weight: 600;
  color: #b8b8cc;
  width: 22px;
  text-align: right;
  flex-shrink: 0;
  padding-top: 2px;
  font-variant-numeric: tabular-nums;
}

.topic-rank--top3 {
  color: #daa520; /* Goldenrod for top 3 */
}
```

### Score Column (right side)

```css
.topic-score {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  padding-top: 2px;
}

.topic-score .number {
  font-size: 14px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  /* color set dynamically based on score range */
}
```

### Platform Tags

```css
.platform-tag {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 6px;
  /* color and background set per platform from PLATFORM_COLORS map */
}
```

### Top 3 Indicator

Replace the current amber left border with a subtle gold neumorphic glow:

```css
.topic-row--top3 {
  box-shadow: 3px 3px 6px #c8c8d2, -3px -3px 6px #ffffff, inset 0 0 0 1.5px rgba(218, 165, 32, 0.25);
}
```

---

## 10. Expanded Detail Panel

When a topic row is tapped, a detail panel appears directly below it, visually connected (the row loses its bottom border-radius, the panel has only bottom border-radius).

### Detail Panel CSS

```css
.topic-detail {
  background: #e2e2e8;   /* Slightly darker than base to show depth */
  border-radius: 0 0 16px 16px;
  padding: 14px 14px 14px 46px; /* 46px left to align with content after rank */
  border-top: 1px solid #d8d8e2;
}
```

### Platform Distribution (in detail panel)

Use horizontal bars (already in current design, works well). Style them neumorphically:

```css
.dist-bar-track {
  height: 6px;
  border-radius: 3px;
  background: #dcdce6;
  box-shadow: inset 1px 1px 2px #c8c8d2, inset -1px -1px 2px #ffffff;
  overflow: hidden;
}

.dist-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
  /* color set per platform */
}
```

### Links in Detail Panel

```css
.detail-link {
  font-size: 12px;
  color: #7c5cfc;
  opacity: 0.7;
  text-decoration: none;
  transition: opacity 0.15s ease;
}

.detail-link:hover {
  opacity: 1;
}
```

---

## 11. CSS Custom Properties (add to index.css)

All tokens defined as CSS custom properties for the Tailwind v4 + inline style approach:

```css
@import "tailwindcss";

:root {
  /* Neumorphic base */
  --bg-base: #e8e8ee;
  --bg-surface: #e8e8ee;
  --bg-inset: #dfe0e6;
  --bg-detail: #e2e2e8;

  /* Shadows */
  --shadow-raised: 6px 6px 12px #c8c8d2, -6px -6px 12px #ffffff;
  --shadow-raised-sm: 3px 3px 6px #c8c8d2, -3px -3px 6px #ffffff;
  --shadow-inset: inset 3px 3px 6px #c8c8d2, inset -3px -3px 6px #ffffff;
  --shadow-hover: 8px 8px 16px #c0c0cc, -8px -8px 16px #ffffff;

  /* Text */
  --text-primary: #2d2d3a;
  --text-secondary: #6b6b80;
  --text-tertiary: #9a9ab0;
  --text-muted: #b8b8cc;

  /* Accents */
  --accent-primary: #7c5cfc;
  --accent-primary-soft: #ece7ff;
  --accent-secondary: #e84393;
  --accent-secondary-soft: #fce4f0;

  /* Score */
  --score-hot: #e84393;
  --score-warm: #7c5cfc;
  --score-cool: #9a9ab0;

  /* Arc gauge track */
  --gauge-track: #d5d5e0;

  /* Distribution bar inset */
  --bar-track-bg: #dcdce6;
  --bar-track-shadow-dark: #c8c8d2;
  --bar-track-shadow-light: #ffffff;
}
```

---

## 12. Scrollbar Styling (light theme)

Replace the current dark scrollbar:

```css
::-webkit-scrollbar { width: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: rgba(0, 0, 0, 0.1); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: rgba(0, 0, 0, 0.18); }
```

---

## 13. Interaction States Summary

### Topic Row

| State | Box Shadow | Background | Other |
|-------|-----------|------------|-------|
| Rest | `--shadow-raised-sm` | `--bg-surface` | -- |
| Hover | `--shadow-hover` (scaled down: `5px 5px 10px #c0c0cc, -5px -5px 10px #fff`) | `--bg-surface` | Subtle transform: `translateY(-1px)` |
| Active/Pressed | `--shadow-inset` | `--bg-inset` | `transform: translateY(0)` |
| Expanded | `--shadow-raised-sm`, `border-radius: 16px 16px 0 0` | `--bg-surface` | Connected to detail panel below |

### Filter Pill

| State | Box Shadow | Background | Text Color |
|-------|-----------|------------|------------|
| Inactive | `--shadow-raised-sm` | `--bg-surface` | `--text-secondary` |
| Hover | `--shadow-raised-sm` | `--bg-surface` | `--accent-primary` |
| Active | `--shadow-inset` | `--bg-inset` | `--accent-primary` |

### Refresh Button

| State | Box Shadow | Background | Text Color |
|-------|-----------|------------|------------|
| Rest | `--shadow-raised-sm` | `--bg-surface` | `--text-secondary` |
| Hover | `5px 5px 10px ...` | `--bg-surface` | `--text-primary` |
| Disabled | `--shadow-flat` (none) | `--bg-inset` | `--text-muted`, opacity 0.5 |

### Search Input

| State | Box Shadow | Background |
|-------|-----------|------------|
| Rest | `--shadow-inset` | `--bg-inset` |
| Focus | `--shadow-inset` + `0 0 0 2px rgba(124,92,252,0.3)` | `--bg-inset` |

---

## 14. Loading and Empty States

### Loading Spinner

Replace the current white/blue spinner with one that fits the neumorphic theme:

```css
.spinner {
  width: 28px;
  height: 28px;
  border: 2.5px solid #d5d5e0;
  border-top-color: #7c5cfc;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
```

Loading text: `color: --text-tertiary`, size 13px.

### Empty State

Text: `color: --text-tertiary`, size 13px. Centered in content area.

### Error State

Error text: `color: --color-error` (#e17055), size 13px. Retry button uses standard neumorphic raised button style.

---

## 15. Status Bar

```css
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 20px;
  font-size: 11px;
  font-weight: 400;
  color: #9a9ab0;
  border-top: 1px solid #dcdce6;
  font-variant-numeric: tabular-nums;
}
```

---

## 16. Chart Feasibility Analysis

### Historical Trend Sparklines -- NOT FEASIBLE (v1)

The app fetches a snapshot each time. There is no persisted history of scores over time. Adding sparklines would require:
1. A local SQLite database or file-based store to persist scores per topic per fetch
2. A data retention strategy (keep last N fetches? Last 24 hours?)
3. Topic identity resolution across fetches (titles change, topics merge)

**Recommendation:** Skip for v1. Consider for v2 by adding a Tauri-side persistence layer.

### Platform Distribution Mini Chart -- FEASIBLE but not recommended per-row

Each `ClusteredItem` has `platformDetails` with per-platform scores. A mini donut could be rendered as an SVG. However, at the 36-40px size needed to fit in a topic row, a donut with 2-8 segments is unreadable. Better to show this in the expanded detail panel (where there is room for the horizontal bars already designed).

### Score Arc Gauge -- FEASIBLE and RECOMMENDED

The SVG arc gauge described in section 9 is the recommended per-row visualization. It provides a clear visual indicator of score magnitude without requiring additional data.

### Stat Card Micro-Charts -- POSSIBLE for v2

The stat cards could include tiny inline sparklines if we persist fetch history. For v1, the bold number with label is sufficient and matches the reference design.

---

## 17. Transition Specifications

All transitions use `ease` timing:

| Element | Property | Duration |
|---------|----------|----------|
| Topic row hover | box-shadow, transform | 200ms |
| Filter pill state change | box-shadow, color, background | 200ms |
| Search focus ring | box-shadow | 150ms |
| Stat card hover | box-shadow | 200ms |
| Button hover | box-shadow, color | 150ms |
| Detail panel expand | -- (no animation, instant toggle for now) | 0ms |
| Distribution bar width | width | 300ms |

---

## 18. Summary of Rust-Side Changes Needed

1. **Remove vibrancy** in `lib.rs` -- delete the `apply_vibrancy` block entirely (or comment it out). The CSS will provide the opaque light background.
2. Keep `transparent: true` in `tauri.conf.json` -- the window chrome remains transparent so the rounded corners from `shadow: true` work. The CSS `background: #e8e8ee` on the root div makes the content opaque.
3. No other Rust changes needed.

---

## 19. File Change Summary

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Remove `apply_vibrancy` call |
| `src/index.css` | Replace entire file with neumorphic CSS custom properties, light scrollbar, light base styles |
| `src/App.css` | Can be deleted (all its styles are from the Vite/Tauri template and unused) |
| `src/App.tsx` | Full rewrite of JSX and class names to use neumorphic design system |
| (new) `src/ScoreGauge.tsx` | SVG arc gauge component |

---

## 20. Visual Reference Mapping

How the reference screenshot elements map to our app:

| Reference Element | Our Implementation |
|------|--------|
| Left sidebar nav | Removed. Horizontal filter pills replace navigation. |
| Top stat cards row | 3 stat cards: Topics, Sources, Top Score |
| "Today/Week/Month" pills | Category filter pills: All, Tech, Entertainment, etc. |
| Area chart panels | Not applicable in v1 (no time-series data). The topic list IS our main content. |
| Neumorphic raised cards | Applied to stat cards, topic rows, pills |
| Neumorphic pressed buttons | Applied to active pills, search input, active states |
| Purple accent | `#7c5cfc` used for active states, links, warm scores |
| Pink accent | `#e84393` used for hot scores |
| Light gray background | `#e8e8ee` base |
| Soft shadows | `6px 6px 12px #c8c8d2, -6px -6px 12px #ffffff` system |
| Clean sans-serif type | SF Pro Text system font stack |
| 16px rounded corners | `border-radius: 16px` on cards, `12px` on pills/inputs |
