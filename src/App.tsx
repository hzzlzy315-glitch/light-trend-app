import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { TrendingData, ClusteredItem } from "./types";

const CATS = ["all","tech","entertainment","politics","business","science","sports","general"] as const;
const CAT_LABELS: Record<string, string> = {
  all:"All", tech:"Tech", entertainment:"Entertainment", politics:"Politics",
  business:"Business", science:"Science", sports:"Sports", general:"General"
};
const PC: Record<string, string> = {
  reddit:"#ff4500", hackernews:"#ff6600", github:"#8b949e", google:"#4285f4",
  youtube:"#ff0000", wikipedia:"#a1a1aa", news:"#f59e0b", producthunt:"#da552f"
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

export default function App() {
  const [data, setData] = useState<TrendingData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [cat, setCat] = useState("all");
  const [exp, setExp] = useState<string | null>(null);
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
      if (e.key === "Escape") setExp(null);
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);

  let items: ClusteredItem[] = data?.byCategory?.[cat] || [];
  if (q) {
    const ql = q.toLowerCase();
    items = items.filter(i => i.title.toLowerCase().includes(ql) || (i.description||"").toLowerCase().includes(ql) || i.platforms.some(p => p.includes(ql)));
  }

  return (
    <div className="h-full flex flex-col" style={{ background: "rgba(14,16,23,0.82)" }}>

      {/* Header */}
      <header data-tauri-drag-region className="flex items-center justify-between px-5 shrink-0 border-b border-white/[0.07]" style={{ height: 48 }}>
        <span data-tauri-drag-region className="text-[16px] font-semibold text-white/90">Light Trend</span>
        <div className="flex items-center gap-3">
          {data && <span className="text-[11px] text-white/25 tabular-nums">{new Date(data.fetchedAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>}
          <button
            onClick={() => { setBusy(true); fetch_(); }}
            disabled={busy}
            className="text-[12px] text-white/50 hover:text-white/80 px-3 py-1 rounded-lg bg-white/[0.06] hover:bg-white/[0.10] disabled:opacity-30 transition-colors"
          >
            {busy ? "\u21BB" : "Refresh"}
          </button>
        </div>
      </header>

      {/* Filter pills */}
      <div className="flex items-center gap-1.5 px-5 py-2.5 overflow-x-auto shrink-0 border-b border-white/[0.05]" style={{ scrollbarWidth: "none" }}>
        {CATS.map(c => {
          const n = (data?.byCategory?.[c]||[]).length;
          const on = cat === c;
          return (
            <button key={c} onClick={() => { setCat(c); setExp(null); }}
              className={`shrink-0 px-3 py-1.5 rounded-lg text-[13px] font-medium transition-colors ${
                on ? "bg-blue-500/20 text-blue-400" : "text-white/30 hover:text-white/50 hover:bg-white/[0.05]"
              }`}
            >
              {CAT_LABELS[c]}
              {n > 0 && <span className={`ml-1.5 text-[10px] tabular-nums ${on ? "text-blue-400/50" : "text-white/15"}`}>{n}</span>}
            </button>
          );
        })}
      </div>

      {/* Search */}
      <div className="px-5 py-2.5 shrink-0">
        <input id="si" type="search" value={q}
          onChange={e => { setQ(e.target.value); setExp(null); }}
          placeholder="Filter topics…  /"
          className="w-full h-9 px-4 rounded-lg text-[14px] bg-white/[0.05] border border-white/[0.08] text-white/90 placeholder:text-white/20 focus:outline-none focus:border-blue-500/40 transition-colors"
        />
      </div>

      {/* Content */}
      <main className="flex-1 overflow-y-auto px-4 pb-4">
        {loading && !data ? (
          <div className="flex flex-col items-center justify-center h-full gap-3">
            <div className="size-7 border-2 border-white/10 border-t-blue-500 rounded-full animate-spin" />
            <span className="text-[13px] text-white/30">Scanning networks…</span>
          </div>
        ) : error && !data ? (
          <div className="flex flex-col items-center justify-center h-full gap-3">
            <span className="text-[13px] text-red-400 selectable">{error}</span>
            <button onClick={fetch_} className="text-[13px] text-white/50 hover:text-white/80 px-4 py-1.5 rounded-lg bg-white/[0.06]">Retry</button>
          </div>
        ) : items.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <span className="text-[13px] text-white/30">{q ? `No match for "${q}"` : "No topics in this category"}</span>
          </div>
        ) : (
          <div className="flex flex-col gap-1.5">
            {items.map((item, i) => (
              <Card key={item.id} item={item} rank={i+1} open={exp === item.id} toggle={() => setExp(exp === item.id ? null : item.id)} />
            ))}
          </div>
        )}
      </main>

      {/* Status bar */}
      {data && (
        <div className="flex items-center justify-between px-5 py-1.5 text-[11px] text-white/20 tabular-nums shrink-0 border-t border-white/[0.05]">
          <span>{Object.keys(data.platformStats ?? {}).length} sources</span>
          <span>{data.totalItems ?? 0} items</span>
          <span>{(data.elapsed / 1000).toFixed(1)}s</span>
        </div>
      )}
    </div>
  );
}

/* ── Card ────────────────────────────────────────────────────── */

function Card({ item, rank, open, toggle }: {
  item: ClusteredItem; rank: number; open: boolean; toggle: () => void;
}) {
  const top3 = rank <= 3;
  const sc = item.compositeScore;

  return (
    <div>
      <button onClick={toggle} className={`w-full text-left rounded-xl transition-colors ${
        open ? "bg-white/[0.07] rounded-b-none" : "bg-white/[0.04] hover:bg-white/[0.07]"
      } ${top3 ? "border-l-2 border-l-amber-500/70" : ""}`}
        style={{ padding: "14px 18px" }}
      >
        <div className="flex items-start gap-3">
          {/* Rank */}
          <span className={`text-[14px] font-semibold tabular-nums shrink-0 w-6 text-right ${top3 ? "text-amber-400/80" : "text-white/15"}`}>
            {rank}
          </span>

          {/* Content */}
          <div className="flex-1 min-w-0">
            {/* Title — 15px, up to 2 lines */}
            <div className="text-[15px] font-medium text-white/85 leading-[1.4] line-clamp-2">
              {item.title}
            </div>

            {/* Description — 13px, 1 line */}
            {item.description && (
              <div className="text-[13px] text-white/35 truncate mt-1">{item.description}</div>
            )}

            {/* Meta row */}
            <div className="flex items-center gap-2 mt-2 flex-wrap">
              {item.platforms.slice(0, 3).map(p => (
                <span key={p} className="text-[11px] px-2 py-0.5 rounded-md" style={{ color: PC[p]||"#a1a1aa", background: `${PC[p]||"#a1a1aa"}18` }}>
                  {p}
                </span>
              ))}
              {item.platforms.length > 3 && <span className="text-[11px] text-white/20">+{item.platforms.length - 3}</span>}
              {item.mentions > 1 && <span className="text-[11px] text-blue-400/70 font-medium">{item.mentions} sources</span>}
              {item.timestamp && <span className="text-[11px] text-white/20 tabular-nums">{timeAgo(item.timestamp)}</span>}
            </div>
          </div>

          {/* Score */}
          <div className="shrink-0 flex flex-col items-end gap-1 pt-0.5">
            <span className={`text-[15px] font-semibold tabular-nums ${
              sc > 80 ? "text-rose-400" : sc > 50 ? "text-blue-400" : "text-white/30"
            }`}>
              {sc}
            </span>
            {/* Mini bar */}
            <div className="w-10 h-1 rounded-full bg-white/[0.06] overflow-hidden">
              <div className="h-full rounded-full" style={{
                width: `${Math.min(sc, 100)}%`,
                background: sc > 80 ? "#f43f5e" : sc > 50 ? "#3b82f6" : "rgba(255,255,255,0.15)"
              }} />
            </div>
          </div>
        </div>
      </button>

      {/* Detail */}
      {open && (
        <div className="bg-white/[0.05] rounded-b-xl border-t border-white/[0.05]" style={{ padding: "16px 18px 16px 48px" }}>
          {/* Distribution */}
          <div className="text-[11px] text-white/25 uppercase tracking-wide mb-3 font-medium">Platform Distribution</div>
          {item.platformDetails.map((d, i) => {
            const mx = Math.max(1, ...item.platformDetails.map(x => x.score));
            const pct = Math.round((d.score / mx) * 100);
            return (
              <div key={`${d.platform}-${i}`} className="flex items-center gap-3 mb-2">
                <span className="text-[12px] text-white/40 w-20">{d.platform}</span>
                <div className="flex-1 h-1.5 rounded-full bg-white/[0.06] overflow-hidden">
                  <div className="h-full rounded-full transition-all" style={{ width: `${pct}%`, background: PC[d.platform]||"#3b82f6" }} />
                </div>
                <span className="text-[11px] text-white/25 w-14 text-right tabular-nums">{fmtScore(d.score)}</span>
              </div>
            );
          })}

          {/* Geos */}
          {item.geos && item.geos.length > 0 && (
            <div className="flex gap-1.5 mt-3 flex-wrap">
              {[...new Set(item.geos)].map(g => (
                <span key={g} className="text-[10px] px-2 py-0.5 rounded-md bg-teal-500/10 text-teal-400/70">{g}</span>
              ))}
            </div>
          )}

          {/* Description */}
          {item.description && (
            <p className="text-[13px] text-white/35 mt-3 leading-relaxed selectable">{item.description}</p>
          )}

          {/* Links */}
          <div className="mt-3 flex flex-col gap-2">
            {item.platformDetails.map((d, i) => (
              <a key={`l-${d.platform}-${i}`} href={d.url} target="_blank" rel="noopener noreferrer"
                className="text-[12px] text-blue-400/60 hover:text-blue-400 truncate selectable transition-colors"
                onClick={e => e.stopPropagation()}
              >
                {d.platform} — {d.url}
              </a>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
