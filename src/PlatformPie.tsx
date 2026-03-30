/** SVG donut chart showing platform distribution */

interface PlatformPieProps {
  platforms: Record<string, { count: number; name: string }>;
  size?: number;
}

const COLORS: Record<string, string> = {
  reddit: "#ff4500",
  hackernews: "#ff6600",
  google: "#4285f4",
  youtube: "#cc0000",
  wikipedia: "#72777d",
  news: "#daa520",
  mastodon: "#5850ec",
  bluesky: "#0066cc",
};

export function PlatformPie({ platforms, size = 160 }: PlatformPieProps) {
  const entries = Object.entries(platforms).sort((a, b) => b[1].count - a[1].count);
  const total = entries.reduce((s, [, v]) => s + v.count, 0);
  if (total === 0) return null;

  const cx = size / 2;
  const cy = size / 2;
  const r = size * 0.35;
  const strokeWidth = size * 0.15;
  const circumference = 2 * Math.PI * r;

  let offset = 0;
  const segments = entries.map(([name, { count }]) => {
    const pct = count / total;
    const dash = circumference * pct;
    const gap = circumference - dash;
    const rotation = (offset / total) * 360 - 90;
    offset += count;
    return { name, count, pct, dash, gap, rotation, color: COLORS[name] || "#9a9ab0" };
  });

  return (
    <div className="flex flex-col items-center gap-3">
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        {segments.map(s => (
          <circle
            key={s.name}
            cx={cx} cy={cy} r={r}
            fill="none"
            stroke={s.color}
            strokeWidth={strokeWidth}
            strokeDasharray={`${s.dash} ${s.gap}`}
            transform={`rotate(${s.rotation} ${cx} ${cy})`}
            strokeLinecap="round"
            opacity={0.85}
          />
        ))}
        {/* Center text */}
        <text x={cx} y={cy - 4} textAnchor="middle" fontSize={size * 0.16} fontWeight="800" fill="#3a3a4a" style={{ fontFamily: "inherit" }}>
          {entries.length}
        </text>
        <text x={cx} y={cy + 12} textAnchor="middle" fontSize={size * 0.08} fontWeight="500" fill="#a0a0b8" style={{ fontFamily: "inherit" }}>
          sources
        </text>
      </svg>
      {/* Legend */}
      <div className="flex flex-wrap justify-center gap-x-3 gap-y-1">
        {segments.map(s => (
          <div key={s.name} className="flex items-center gap-1.5">
            <div className="size-2.5 rounded-full" style={{ background: s.color }} />
            <span className="text-[11px] font-medium" style={{ color: "#8a8aa0" }}>{s.name}</span>
            <span className="text-[10px] tabular-nums" style={{ color: "#c0c0d4" }}>{s.count}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
