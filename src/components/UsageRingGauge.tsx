export function usageAccentColor(value: number): string {
  if (value < 50) return "#38bdf8";
  if (value < 80) return "#fbbf24";
  return "#fb7185";
}

export function UsageRingGauge({
  percent,
  label,
  hint,
  compact = false,
}: {
  percent: number;
  label: string;
  hint?: string;
  compact?: boolean;
}) {
  const value = Math.max(0, Math.min(100, percent));
  const size = compact ? 62 : 84;
  const radius = compact ? 25 : 34;
  const stroke = compact ? 4 : 5;
  const center = size / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (value / 100) * circumference;
  const color = usageAccentColor(value);

  return (
    <div className="flex flex-col items-center gap-1">
      <div className="relative" style={{ width: size, height: size }}>
        <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="-rotate-90">
          <circle
            cx={center}
            cy={center}
            r={radius}
            fill="none"
            className="usage-ring-track"
            strokeWidth={stroke}
          />
          <circle
            cx={center}
            cy={center}
            r={radius}
            fill="none"
            stroke={color}
            strokeWidth={stroke}
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={offset}
            className="transition-[stroke-dashoffset] duration-500 ease-out"
          />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span
            className={`font-semibold tabular-nums leading-none text-foreground ${compact ? "text-sm" : "text-lg"}`}
          >
            {value.toFixed(0)}
            <span className="text-[9px] font-normal text-muted-foreground">%</span>
          </span>
        </div>
      </div>
      <div className="text-center">
        <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">{label}</p>
        {hint ? (
          <p className="max-w-[108px] truncate text-[9px] text-muted-foreground" title={hint}>
            {hint}
          </p>
        ) : null}
      </div>
    </div>
  );
}

function formatResetTime(minutes: number): string {
  if (minutes >= 24 * 60) {
    const days = Math.max(1, Math.round(minutes / (24 * 60)));
    return `~${days}d`;
  }
  if (minutes >= 60) {
    const hours = Math.max(1, Math.round(minutes / 60));
    return `~${hours}h`;
  }
  return `~${Math.max(1, Math.round(minutes))}m`;
}

export function formatWindowHint(resetMinutes: number, remainingPercent: number): string {
  return `${formatResetTime(resetMinutes)} · ${remainingPercent.toFixed(0)}% left`;
}
