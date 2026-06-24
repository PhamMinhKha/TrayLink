export function usageBarColor(value: number): string {
  if (value < 50) return "bg-emerald-500";
  if (value < 80) return "bg-amber-500";
  return "bg-rose-500";
}

export function UsageCompactBar({
  label,
  usedPercent,
}: {
  label: string;
  usedPercent: number;
}) {
  const percent = Math.max(0, Math.min(100, usedPercent));

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between gap-2 text-xs">
        <span className="text-muted-foreground">{label}</span>
        <span className="font-medium tabular-nums">{percent.toFixed(0)}%</span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-muted">
        <div
          className={`h-full rounded-full transition-all ${usageBarColor(percent)}`}
          style={{ width: `${percent}%` }}
        />
      </div>
    </div>
  );
}
