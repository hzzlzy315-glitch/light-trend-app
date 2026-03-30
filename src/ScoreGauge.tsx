/** SVG arc gauge showing composite score (0-150 scale) */

interface ScoreGaugeProps {
  score: number;
  size?: number;
}

export function ScoreGauge({ score, size = 40 }: ScoreGaugeProps) {
  const max = 150;
  const pct = Math.min(score / max, 1);
  const color = score > 80 ? "#e84393" : score > 50 ? "#7c5cfc" : "#b8b8cc";
  const trackColor = "#e0e1e8";

  // Arc parameters (120-degree arc at the top)
  const cx = size / 2;
  const cy = size * 0.6;
  const r = size * 0.38;
  const startAngle = -210;
  const endAngle = 30;
  const sweep = endAngle - startAngle; // 240 degrees

  const polarToCartesian = (angle: number) => {
    const rad = (angle * Math.PI) / 180;
    return { x: cx + r * Math.cos(rad), y: cy + r * Math.sin(rad) };
  };

  const arcPath = (start: number, end: number) => {
    const s = polarToCartesian(start);
    const e = polarToCartesian(end);
    const largeArc = Math.abs(end - start) > 180 ? 1 : 0;
    return `M ${s.x} ${s.y} A ${r} ${r} 0 ${largeArc} 1 ${e.x} ${e.y}`;
  };

  const fillAngle = startAngle + sweep * pct;

  return (
    <svg width={size} height={size * 0.7} viewBox={`0 0 ${size} ${size * 0.7}`}>
      {/* Track */}
      <path
        d={arcPath(startAngle, endAngle)}
        fill="none"
        stroke={trackColor}
        strokeWidth={3}
        strokeLinecap="round"
      />
      {/* Fill */}
      {pct > 0.01 && (
        <path
          d={arcPath(startAngle, fillAngle)}
          fill="none"
          stroke={color}
          strokeWidth={3}
          strokeLinecap="round"
        />
      )}
      {/* Score text */}
      <text
        x={cx}
        y={cy + 2}
        textAnchor="middle"
        fontSize={size * 0.26}
        fontWeight="700"
        fill={color}
        style={{ fontFamily: "inherit", fontVariantNumeric: "tabular-nums" }}
      >
        {score}
      </text>
    </svg>
  );
}
