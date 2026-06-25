import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { Activity, AlertCircle, RefreshCw, Settings2 } from "lucide-react";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import claudeSymbol from "@/assets/claude-ai-symbol.svg";
import codexSymbol from "@/assets/codex-color.png";
import { formatWindowHint, UsageRingGauge } from "@/components/UsageRingGauge";
import { ThemeToggle } from "@/components/ThemeToggle";
import { usageBarColor } from "@/components/UsageCompactBar";
import {
  getConfig,
  getMonitorStatus,
  showDashboard,
  systemMetricsAnyEnabled,
  type AppConfig,
  type SystemMetricsResponse,
  type UsageMonitorResponse,
  type UsageWindow,
} from "@/lib/tauri";

const REFRESH_MS = 60_000;
const POPUP_WIDTH = 340;
const POPUP_HEIGHT_PADDING = 4;

function syncPopupWindowSize(root: HTMLElement) {
  const height = Math.ceil(root.scrollHeight) + POPUP_HEIGHT_PADDING;
  void getCurrentWindow().setSize(new LogicalSize(POPUP_WIDTH, height));
}

function formatBps(value: number): string {
  if (value >= 1024 ** 2) {
    return `${(value / 1024 ** 2).toFixed(1)} MB/s`;
  }
  if (value >= 1024) {
    return `${(value / 1024).toFixed(1)} KB/s`;
  }
  return `${value.toFixed(0)} B/s`;
}

function LoadingRings() {
  return (
    <div className="flex justify-center gap-5 py-1">
      {[0, 1].map((key) => (
        <div key={key} className="flex flex-col items-center gap-1.5">
          <div className="size-[62px] animate-pulse rounded-full bg-muted" />
          <div className="h-2 w-8 animate-pulse rounded bg-muted" />
        </div>
      ))}
    </div>
  );
}

function ProviderError({ message }: { message: string }) {
  return (
    <div className="flex gap-2 rounded-lg border border-rose-500/20 bg-rose-500/[0.08] px-2.5 py-2">
      <AlertCircle className="mt-0.5 size-3.5 shrink-0 text-rose-400" />
      <div className="min-w-0">
        <p className="text-[11px] font-medium text-rose-300">Không lấy được quota</p>
        <p className="mt-0.5 line-clamp-2 text-[10px] leading-snug text-muted-foreground">{message}</p>
      </div>
    </div>
  );
}

function WindowGauges({ session, weekly }: { session: UsageWindow; weekly: UsageWindow }) {
  return (
    <div className="flex justify-around gap-1 px-0.5">
      <UsageRingGauge
        compact
        percent={session.used_percent}
        label="5 giờ"
        hint={formatWindowHint(session.reset_minutes, session.remaining_percent)}
      />
      <UsageRingGauge
        compact
        percent={weekly.used_percent}
        label="7 ngày"
        hint={formatWindowHint(weekly.reset_minutes, weekly.remaining_percent)}
      />
    </div>
  );
}

function ProviderCard({
  name,
  icon,
  iconAlt,
  accentClass,
  enabled,
  usage,
  loading,
}: {
  name: string;
  icon: string;
  iconAlt: string;
  accentClass: string;
  enabled: boolean;
  usage: UsageMonitorResponse | null;
  loading: boolean;
}) {
  if (!enabled) {
    return null;
  }

  return (
    <section className="usage-popup-card rounded-xl p-2.5">
      <div className="mb-2 flex items-center gap-2">
        <div
          className={`flex size-7 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br ${accentClass}`}
        >
          <img src={icon} alt={iconAlt} className="size-3.5 object-contain" />
        </div>
        <p className="min-w-0 flex-1 truncate text-xs font-semibold text-foreground">{name}</p>
        {usage?.ok && usage.updated_at ? (
          <span className="shrink-0 text-[9px] tabular-nums text-muted-foreground">
            {new Date(usage.updated_at).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </span>
        ) : null}
      </div>

      {usage?.ok && usage.session_5h && usage.weekly_7d ? (
        <WindowGauges session={usage.session_5h} weekly={usage.weekly_7d} />
      ) : usage && !usage.ok ? (
        <ProviderError message={usage.error ?? "Không lấy được dữ liệu quota."} />
      ) : loading ? (
        <LoadingRings />
      ) : null}
    </section>
  );
}

function SystemMetricRow({
  label,
  value,
  percent,
}: {
  label: string;
  value?: string;
  percent?: number;
}) {
  if (percent != null) {
    const p = Math.max(0, Math.min(100, percent));
    return (
      <div className="space-y-1">
        <div className="flex items-center justify-between gap-2 text-[10px]">
          <span className="text-muted-foreground">{label}</span>
          <span className="font-medium tabular-nums text-foreground">{p.toFixed(0)}%</span>
        </div>
        <div className="h-1 overflow-hidden rounded-full bg-muted">
          <div
            className={`h-full rounded-full ${usageBarColor(p)}`}
            style={{ width: `${p}%` }}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-between gap-2 text-[10px]">
      <span className="text-muted-foreground">{label}</span>
      <span className="truncate font-medium tabular-nums text-foreground">{value ?? "—"}</span>
    </div>
  );
}

function SystemSection({
  enabled,
  metrics,
  loading,
}: {
  enabled: boolean;
  metrics: SystemMetricsResponse | null;
  loading: boolean;
}) {
  if (!enabled) {
    return null;
  }

  const hasData =
    metrics &&
    (metrics.cpu ||
      metrics.memory ||
      metrics.disk ||
      metrics.network ||
      metrics.cpu_temperature ||
      metrics.battery_temperature ||
      metrics.fan);

  return (
    <section className="usage-popup-card rounded-xl p-2.5">
      <div className="mb-2 flex items-center gap-2">
        <div className="flex size-7 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-sky-500/30 to-blue-500/20 ring-1 ring-sky-400/20">
          <Activity className="size-3.5 text-sky-300" />
        </div>
        <p className="min-w-0 flex-1 truncate text-xs font-semibold text-foreground">System</p>
        {metrics?.updated_at ? (
          <span className="shrink-0 text-[9px] tabular-nums text-muted-foreground">
            {new Date(metrics.updated_at).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </span>
        ) : null}
      </div>

      {loading && !hasData ? (
        <div className="space-y-2 py-1">
          {[0, 1, 2].map((key) => (
            <div key={key} className="h-3 animate-pulse rounded bg-muted" />
          ))}
        </div>
      ) : metrics && !metrics.ok && metrics.error ? (
        <ProviderError message={metrics.error} />
      ) : hasData ? (
        <div className="space-y-2">
          {metrics.cpu?.status === "ok" && metrics.cpu.usage_percent != null ? (
            <SystemMetricRow label="CPU" percent={metrics.cpu.usage_percent} />
          ) : null}
          {metrics.memory?.status === "ok" && metrics.memory.used_percent != null ? (
            <SystemMetricRow label="RAM" percent={metrics.memory.used_percent} />
          ) : null}
          {metrics.disk?.status === "ok" && metrics.disk.used_percent != null ? (
            <SystemMetricRow label="Disk" percent={metrics.disk.used_percent} />
          ) : null}
          {metrics.network?.status === "ok" ? (
            <SystemMetricRow
              label="Mạng"
              value={`↓${formatBps(metrics.network.download_bps ?? 0)} ↑${formatBps(metrics.network.upload_bps ?? 0)}`}
            />
          ) : null}
          {metrics.cpu_temperature ? (
            <SystemMetricRow
              label="Nhiệt CPU"
              value={
                metrics.cpu_temperature.status === "ok" && metrics.cpu_temperature.celsius != null
                  ? `${metrics.cpu_temperature.celsius.toFixed(0)}°C`
                  : metrics.cpu_temperature.message ?? "N/A"
              }
            />
          ) : null}
          {metrics.battery_temperature ? (
            <SystemMetricRow
              label="Nhiệt pin"
              value={
                metrics.battery_temperature.status === "ok" &&
                metrics.battery_temperature.celsius != null
                  ? `${metrics.battery_temperature.celsius.toFixed(0)}°C`
                  : metrics.battery_temperature.message ?? "N/A"
              }
            />
          ) : null}
          {metrics.fan ? (
            <SystemMetricRow
              label="Quạt"
              value={
                metrics.fan.status === "ok" && metrics.fan.rpm != null
                  ? `${metrics.fan.rpm} RPM`
                  : metrics.fan.message ?? "N/A"
              }
            />
          ) : null}
        </div>
      ) : null}
    </section>
  );
}

export function UsagePopup() {
  const rootRef = useRef<HTMLDivElement>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [claudeUsage, setClaudeUsage] = useState<UsageMonitorResponse | null>(null);
  const [codexUsage, setCodexUsage] = useState<UsageMonitorResponse | null>(null);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetricsResponse | null>(null);
  const [loading, setLoading] = useState(false);

  const claudeEnabled = Boolean(config?.usage_monitor_enabled);
  const codexEnabled = Boolean(config?.codex_usage_monitor_enabled);
  const systemEnabled = systemMetricsAnyEnabled(config?.system_metrics);
  const anyEnabled = claudeEnabled || codexEnabled || systemEnabled;
  const sectionCount = Number(claudeEnabled) + Number(codexEnabled) + Number(systemEnabled);

  const refreshAll = useCallback(async () => {
    setLoading(true);
    try {
      const cfg = await getConfig();
      setConfig(cfg);
      const sysOn = systemMetricsAnyEnabled(cfg.system_metrics);
      const anyMonitor =
        cfg.usage_monitor_enabled || cfg.codex_usage_monitor_enabled || sysOn;

      if (!anyMonitor) {
        setClaudeUsage(null);
        setCodexUsage(null);
        setSystemMetrics(null);
        return;
      }

      const monitor = await getMonitorStatus();
      setClaudeUsage(cfg.usage_monitor_enabled ? monitor.claude : null);
      setCodexUsage(cfg.codex_usage_monitor_enabled ? monitor.codex : null);
      setSystemMetrics(sysOn ? monitor.system : null);
    } finally {
      setLoading(false);
      requestAnimationFrame(() => {
        if (rootRef.current) {
          syncPopupWindowSize(rootRef.current);
        }
      });
    }
  }, []);

  useEffect(() => {
    void refreshAll();
  }, [refreshAll]);

  useEffect(() => {
    if (!anyEnabled) {
      return;
    }

    const id = window.setInterval(() => {
      void refreshAll();
    }, REFRESH_MS);

    return () => window.clearInterval(id);
  }, [anyEnabled, refreshAll]);

  useEffect(() => {
    const popup = getCurrentWindow();
    let unlistenFocus: (() => void) | undefined;

    void popup.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        void refreshAll();
      } else {
        void popup.hide();
      }
    }).then((fn) => {
      unlistenFocus = fn;
    });

    return () => {
      unlistenFocus?.();
    };
  }, [refreshAll]);

  useLayoutEffect(() => {
    const root = rootRef.current;
    if (!root) {
      return;
    }

    const applySize = () => syncPopupWindowSize(root);
    applySize();
    const frame = requestAnimationFrame(applySize);

    const observer = new ResizeObserver(() => {
      applySize();
    });
    observer.observe(root);

    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, [anyEnabled, sectionCount, claudeUsage, codexUsage, systemMetrics, loading, config]);

  const openSettings = async () => {
    await showDashboard("settings");
    await getCurrentWindow().hide();
  };

  return (
    <div ref={rootRef} id="usage-popup-root" className="traylink-theme usage-popup-root text-foreground">
      <div className="flex flex-col p-3">
        <header className="mb-2.5 flex items-center justify-between gap-2">
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              TrayLink
            </p>
            <h1 className="text-sm font-semibold text-foreground">Monitor</h1>
          </div>
          <div className="flex items-center gap-1.5">
            <ThemeToggle compact />
            <button
              type="button"
              disabled={loading || !anyEnabled}
              onClick={() => void refreshAll()}
              aria-label="Làm mới"
              className="flex size-7 items-center justify-center rounded-lg border border-border bg-background/60 text-muted-foreground transition hover:bg-accent hover:text-accent-foreground disabled:opacity-40"
            >
              <RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />
            </button>
          </div>
        </header>

        <div className="flex flex-col gap-2">
          {!anyEnabled ? (
            <div className="usage-popup-card rounded-xl px-3 py-5 text-center">
              <div className="mx-auto mb-2 flex size-10 items-center justify-center rounded-xl bg-muted">
                <Settings2 className="size-4 text-muted-foreground" />
              </div>
              <p className="text-xs font-medium text-foreground">Chưa bật theo dõi</p>
              <p className="mt-1 text-[10px] leading-relaxed text-muted-foreground">
                Bật AI quota hoặc System metrics trong Settings.
              </p>
            </div>
          ) : (
            <>
              <ProviderCard
                name="Claude Code"
                icon={claudeSymbol}
                iconAlt="Claude"
                accentClass="from-orange-500/30 to-rose-500/20 ring-1 ring-orange-400/20"
                enabled={claudeEnabled}
                usage={claudeUsage}
                loading={loading}
              />
              <ProviderCard
                name="Codex"
                icon={codexSymbol}
                iconAlt="Codex"
                accentClass="from-emerald-500/30 to-teal-500/20 ring-1 ring-emerald-400/20"
                enabled={codexEnabled}
                usage={codexUsage}
                loading={loading}
              />
              <SystemSection enabled={systemEnabled} metrics={systemMetrics} loading={loading} />
            </>
          )}
        </div>

        <button
          type="button"
          onClick={() => void openSettings()}
          className="mt-2.5 flex w-full items-center justify-center gap-1.5 rounded-lg border border-border bg-background/60 py-2 text-[11px] font-medium text-muted-foreground transition hover:border-primary/30 hover:bg-accent hover:text-accent-foreground"
        >
          <Settings2 className="size-3.5" />
          Mở Settings
        </button>
      </div>
    </div>
  );
}
