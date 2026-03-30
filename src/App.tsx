import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { TrendingData, ClusteredItem } from "./types";
import { ScoreGauge } from "./ScoreGauge";
import { PlatformPie } from "./PlatformPie";

const CATS = ["all","tech","entertainment","politics","business","science","sports","general"] as const;
const CAT_LABELS: Record<string, string> = {
  all:"All", tech:"Tech", entertainment:"Entertainment", politics:"Politics",
  business:"Business", science:"Science", sports:"Sports", general:"General"
};
const PC: Record<string, { text: string; bg: string }> = {
  reddit:     { text: "#d63031", bg: "#ffeaea" },
  hackernews: { text: "#cc5500", bg: "#fff3e6" },
  google:     { text: "#1a73e8", bg: "#e8f0fe" },
  youtube:    { text: "#cc0000", bg: "#ffe0e0" },
  wikipedia:  { text: "#72777d", bg: "#f0f0f2" },
  news:       { text: "#b8860b", bg: "#fff8e1" },
  mastodon:   { text: "#5850ec", bg: "#eeedff" },
  bluesky:    { text: "#0066cc", bg: "#e0f0ff" },
};

function timeAgo(ts: string | null): string {
  if (!ts) return "";
  const d = Date.now() - new Date(ts).getTime();
  if (d < 0) return "now";
  const m = Math.floor(d / 60000);
  if (m < 1) return "now";
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h`;
  return `${Math.floor(h / 24)}d`;
}

function fmtScore(n: number): string {
  if (n >= 1e6) return `${(n/1e6).toFixed(1)}M`;
  if (n >= 1e3) return `${(n/1e3).toFixed(1)}K`;
  return String(n);
}

// Tauri external link handler
async function openLink(url: string) {
  try { await openUrl(url); } catch { window.open(url, "_blank"); }
}

export default function App() {
  const [data, setData] = useState<TrendingData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [cat, setCat] = useState("all");
  const [selected, setSelected] = useState<ClusteredItem | null>(null);
  const [q, setQ] = useState("");
  const [busy, setBusy] = useState(false);

  const fetch_ = useCallback(async () => {
    try {
      setError(null);
      const youtubeKey = localStorage.getItem("youtube_api_key") || undefined;
      const r = await invoke<TrendingData>("fetch_trending", { youtubeKey });
      setData(r);
      localStorage.setItem("lt_cache", JSON.stringify(r));
    } catch (e) { setError(String(e)); }
    finally { setLoading(false); setBusy(false); }
  }, []);

  useEffect(() => {
    const c = localStorage.getItem("lt_cache");
    if (c) { try { setData(JSON.parse(c)); setLoading(false); } catch { localStorage.removeItem("lt_cache"); } }
    fetch_();
  }, [fetch_]);

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (e.key === "/" && !e.ctrlKey && !e.metaKey && document.activeElement?.tagName !== "INPUT") {
        e.preventDefault(); document.getElementById("si")?.focus();
      }
      if (e.key === "Escape") setSelected(null);
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);

  let items: ClusteredItem[] = data?.byCategory?.[cat] || [];
  if (q) {
    const ql = q.toLowerCase();
    items = items.filter(i => i.title.toLowerCase().includes(ql) || (i.description||"").toLowerCase().includes(ql) || i.platforms.some(p => p.includes(ql)));
  }

  const topScore = items[0]?.compositeScore || 0;
  const avgScore = items.length > 0 ? Math.round(items.reduce((s, i) => s + i.compositeScore, 0) / items.length) : 0;

  return (
    <div className="h-full flex items-center justify-center rounded-3xl" style={{ background: "var(--bg)", padding: "20px 32px" }}>
    <div className="w-full h-full flex flex-col">

      {/* ── Header ──────────────────────────────── */}
      <header data-tauri-drag-region className="flex items-center justify-between px-8 shrink-0" style={{ height: 56 }}>
        <span data-tauri-drag-region className="text-[20px] font-bold" style={{ color: "var(--hot)" }}>
          Light Trend
        </span>
        <div className="flex items-center gap-4 mr-4">
          {data && <span className="text-[12px] tabular-nums font-medium" style={{ color: "var(--text-4)" }}>
            Updated {new Date(data.fetchedAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
          </span>}
          <button
            onClick={() => { setBusy(true); fetch_(); }}
            disabled={busy}
            className="text-[13px] font-semibold px-5 py-2 rounded-2xl neu-sm neu-hover disabled:opacity-30"
            style={{ color: "var(--accent)" }}
          >
            {busy ? "Refreshing..." : "Refresh"}
          </button>
        </div>
      </header>

      {/* ── Stat Cards ──────────────────────────── */}
      <div className="flex gap-4 px-8 py-4 shrink-0">
        <StatCard number={items.length} label="Topics" sub={cat === "all" ? "All categories" : CAT_LABELS[cat]} />
        <StatCard number={Object.keys(data?.platformStats ?? {}).length} label="Sources" sub="Active platforms" />
        <StatCard number={topScore} label="Top Score" sub="Highest trending" accent />
        <StatCard number={avgScore} label="Avg Score" sub="Mean composite" />
        <StatCard number={data?.totalItems ?? 0} label="Raw Items" sub="Before clustering" />
      </div>

      {/* ── Filter + Search ─────────────────────── */}
      <div className="flex items-center gap-4 px-8 py-3 shrink-0">
        <div className="flex items-center gap-2 overflow-x-auto" style={{ scrollbarWidth: "none" }}>
          {CATS.map(c => {
            const n = (data?.byCategory?.[c]||[]).length;
            const on = cat === c;
            const colors: Record<string, { active: string; bg: string }> = {
              all:           { active: "#7c5cfc", bg: "#ece7ff" },
              tech:          { active: "#1a73e8", bg: "#e8f0fe" },
              entertainment: { active: "#e84393", bg: "#fce4f0" },
              politics:      { active: "#e17055", bg: "#ffeae4" },
              business:      { active: "#00b894", bg: "#e6fcf5" },
              science:       { active: "#0984e3", bg: "#e0f3ff" },
              sports:        { active: "#fdcb6e", bg: "#fff8e1" },
              general:       { active: "#6b6b80", bg: "#f0f0f2" },
            };
            const col = colors[c] || colors.general;
            return (
              <button key={c}
                onClick={() => { setCat(c); setSelected(null); }}
                className={`shrink-0 px-5 py-2.5 rounded-3xl text-[14px] font-bold transition-all ${on ? "neu-inset" : "neu-sm neu-hover"}`}
                style={{
                  color: col.active,
                  background: col.bg,
                }}
              >
                {CAT_LABELS[c]}
                {n > 0 && <span className="ml-2 text-[11px] tabular-nums font-semibold" style={{ color: col.active, opacity: 0.6 }}>{n}</span>}
              </button>
            );
          })}
        </div>
        <div className="w-48 shrink-0 ml-auto">
          <input id="si" type="search" value={q}
            onChange={e => { setQ(e.target.value); setSelected(null); }}
            placeholder="Filter topics…"
            className="w-full h-10 px-4 rounded-2xl text-[14px] font-medium neu-inset focus:outline-none"
            style={{ color: "var(--text-1)", border: "none" }}
          />
        </div>
      </div>

      {/* ── Main: Topic List + Right Panel ──────── */}
      <div className="flex-1 flex gap-4 px-8 py-4 overflow-hidden">

        {/* Left: Topic List */}
        <div className="flex-1 overflow-y-auto pr-2">
          {loading && !data ? (
            <div className="flex flex-col items-center justify-center h-full gap-4">
              <div className="size-8 border-3 rounded-full anim-spin" style={{ borderColor: "var(--text-4)", borderTopColor: "var(--accent)" }} />
              <span className="text-[14px] font-medium" style={{ color: "var(--text-3)" }}>Scanning networks…</span>
            </div>
          ) : error && !data ? (
            <div className="flex flex-col items-center justify-center h-full gap-4">
              <span className="text-[14px] font-medium selectable" style={{ color: "#e17055" }}>{error}</span>
              <button onClick={fetch_} className="text-[14px] font-semibold px-5 py-2 rounded-2xl neu-sm neu-hover" style={{ color: "var(--accent)" }}>Retry</button>
            </div>
          ) : items.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <span className="text-[14px] font-medium" style={{ color: "var(--text-3)" }}>{q ? `No match for "${q}"` : "No topics"}</span>
            </div>
          ) : (
            <div className="flex flex-col gap-2.5">
              {items.map((item, i) => (
                <TopicRow key={item.id} item={item} rank={i+1} isSelected={selected?.id === item.id}
                  onSelect={() => setSelected(selected?.id === item.id ? null : item)} idx={i} />
              ))}
            </div>
          )}
        </div>

        {/* Right: Detail + Visualizations */}
        <div className="w-[340px] shrink-0 flex flex-col gap-4 overflow-y-auto">
          {/* Platform Distribution Pie */}
          {data?.platformStats && (
            <div className="neu-raised rounded-3xl p-5 anim-fade-up">
              <h3 className="text-[13px] font-bold uppercase tracking-wide mb-4 text-center" style={{ color: "var(--text-3)" }}>
                Platform Distribution
              </h3>
              <PlatformPie platforms={data.platformStats} />
            </div>
          )}

          {/* Selected Topic Detail */}
          {selected ? (
            <div className="neu-raised rounded-3xl px-10 py-8 anim-fade-up">
              <h3 className="text-[13px] font-bold uppercase tracking-wide mb-5" style={{ color: "var(--text-3)" }}>
                Topic Detail
              </h3>

              {/* Score + Title */}
              <div className="flex items-start gap-4 mb-6">
                <ScoreGauge score={selected.compositeScore} size={72} />
                <div className="flex-1 min-w-0">
                  <div className="text-[16px] font-bold leading-snug line-clamp-2" style={{ color: "var(--text-1)" }}>{selected.title}</div>
                  {selected.timestamp && <div className="text-[11px] mt-2 font-medium" style={{ color: "var(--text-3)" }}>{timeAgo(selected.timestamp)} ago</div>}
                </div>
              </div>

              {/* Description */}
              {selected.description && (
                <p className="text-[13px] leading-[1.7] mb-6 selectable" style={{ color: "var(--text-2)" }}>{selected.description}</p>
              )}

              {/* Platform bars */}
              <div className="text-[11px] font-bold uppercase tracking-wide mb-3" style={{ color: "var(--text-4)" }}>Sources</div>
              <div className="flex flex-col gap-3 mb-6">
                {selected.platformDetails.map((d, i) => {
                  const mx = Math.max(1, ...selected.platformDetails.map(x => x.score));
                  const pct = Math.round((d.score / mx) * 100);
                  const c = PC[d.platform] || { text: "#72777d", bg: "#f0f0f2" };
                  return (
                    <div key={`${d.platform}-${i}`} className="flex items-center gap-3">
                      <span className="text-[12px] font-semibold w-20 truncate" style={{ color: "var(--text-2)" }}>{d.platform}</span>
                      <div className="flex-1 h-2.5 rounded-full overflow-hidden" style={{ background: "var(--bg-inset)" }}>
                        <div className="h-full rounded-full transition-all" style={{ width: `${pct}%`, background: c.text }} />
                      </div>
                      <span className="text-[11px] font-semibold w-14 text-right tabular-nums" style={{ color: "var(--text-3)" }}>{fmtScore(d.score)}</span>
                    </div>
                  );
                })}
              </div>

              {/* Geos */}
              {selected.geos && selected.geos.length > 0 && (
                <div className="mb-6">
                  <div className="text-[11px] font-bold uppercase tracking-wide mb-3" style={{ color: "var(--text-4)" }}>Regions</div>
                  <div className="flex gap-2 flex-wrap">
                    {[...new Set(selected.geos)].map(g => (
                      <span key={g} className="text-[11px] font-semibold px-3 py-1.5 rounded-2xl" style={{ color: "#00b894", background: "#e6fcf5" }}>{g}</span>
                    ))}
                  </div>
                </div>
              )}

              {/* Links */}
              <div className="text-[11px] font-bold uppercase tracking-wide mb-3" style={{ color: "var(--text-4)" }}>Open Source</div>
              <div className="flex flex-col gap-3">
                {selected.platformDetails.map((d, i) => (
                  <button key={`l-${d.platform}-${i}`}
                    onClick={() => openLink(d.url)}
                    className="text-[12px] font-semibold text-left truncate selectable transition-colors hover:underline py-1"
                    style={{ color: "var(--accent)" }}
                  >
                    {d.platform} ↗
                  </button>
                ))}
              </div>
            </div>
          ) : (
            <div className="neu-raised rounded-3xl p-5 flex items-center justify-center" style={{ minHeight: 200 }}>
              <span className="text-[13px] font-medium text-center" style={{ color: "var(--text-4)" }}>
                Select a topic to see details
              </span>
            </div>
          )}
        </div>
      </div>

      {/* ── Status Bar ──────────────────────────── */}
      {data && (
        <div className="flex items-center justify-center gap-6 px-8 py-2 text-[11px] font-semibold tabular-nums shrink-0" style={{ color: "var(--text-4)" }}>
          <span>{Object.keys(data.platformStats ?? {}).length} sources</span>
          <span>•</span>
          <span>{data.totalItems ?? 0} raw items</span>
          <span>•</span>
          <span>{(data.elapsed / 1000).toFixed(1)}s fetch</span>
        </div>
      )}
    </div>
    </div>
  );
}

/* ── Stat Card ────────────────────────────────────────────── */

function StatCard({ number, label, sub, accent }: { number: number; label: string; sub: string; accent?: boolean }) {
  return (
    <div className="flex-1 min-w-0 px-6 py-5 rounded-3xl neu-raised anim-fade-up text-center">
      <div className="text-[28px] font-extrabold tabular-nums leading-none" style={{ color: accent ? "var(--accent)" : "var(--text-1)" }}>
        {number}
      </div>
      <div className="text-[12px] font-semibold mt-2" style={{ color: "var(--text-3)" }}>{label}</div>
      <div className="text-[10px] font-medium truncate mt-0.5" style={{ color: "var(--text-4)" }}>{sub}</div>
    </div>
  );
}

/* ── Topic Row ────────────────────────────────────────────── */

function TopicRow({ item, rank, isSelected, onSelect, idx }: {
  item: ClusteredItem; rank: number; isSelected: boolean; onSelect: () => void; idx: number;
}) {
  const top3 = rank <= 3;
  const sc = item.compositeScore;

  return (
    <button
      onClick={onSelect}
      className={`anim-fade-up w-full text-left rounded-3xl transition-all ${isSelected ? "neu-inset" : "neu-sm neu-hover"}`}
      style={{ padding: "16px 20px", animationDelay: `${Math.min(idx * 25, 400)}ms` }}
    >
      <div className="flex items-center gap-4">
        {/* Rank */}
        <span className="text-[15px] font-bold tabular-nums shrink-0 w-6 text-center"
          style={{ color: top3 ? "#daa520" : "var(--text-4)" }}>
          {rank}
        </span>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="text-[15px] font-semibold leading-snug line-clamp-2" style={{ color: "var(--text-1)" }}>
            {item.title}
          </div>
          {item.description && (
            <div className="text-[12px] font-medium truncate mt-1" style={{ color: "var(--text-2)" }}>{item.description}</div>
          )}
          <div className="flex items-center gap-2 mt-2 flex-wrap">
            {item.platforms.slice(0, 3).map(p => {
              const c = PC[p] || { text: "#72777d", bg: "#f0f0f2" };
              return <span key={p} className="text-[10px] font-semibold px-2 py-0.5 rounded-lg" style={{ color: c.text, background: c.bg }}>{p}</span>;
            })}
            {item.platforms.length > 3 && <span className="text-[10px] font-medium" style={{ color: "var(--text-4)" }}>+{item.platforms.length - 3}</span>}
            {item.mentions > 1 && <span className="text-[10px] font-bold" style={{ color: "var(--accent)" }}>{item.mentions} sources</span>}
            {item.timestamp && <span className="text-[10px] font-medium tabular-nums" style={{ color: "var(--text-3)" }}>{timeAgo(item.timestamp)}</span>}
          </div>
        </div>

        {/* Score Gauge */}
        <div className="shrink-0">
          <ScoreGauge score={sc} size={48} />
        </div>
      </div>
    </button>
  );
}
