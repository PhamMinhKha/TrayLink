import { useEffect, useMemo, useState } from "react";
import { Activity, RefreshCw } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { UsageCompactBar } from "@/components/UsageCompactBar";
import {
  apiBaseUrl,
  getConfig,
  getServerStatus,
  getSystemMetricsStatus,
  systemMetricsAnyEnabled,
  updateConfig,
  type AppConfig,
  type SystemMetricsPreferences,
  type SystemMetricsResponse,
} from "@/lib/tauri";

const REFRESH_MS = 60_000;

type MetricKey = keyof SystemMetricsPreferences;

const METRIC_TOGGLES: {
  key: MetricKey;
  label: string;
  description: string;
  badge?: string;
}[] = [
  {
    key: "cpu",
    label: "CPU",
    description: "Mức sử dụng CPU toàn hệ thống (%).",
  },
  {
    key: "memory",
    label: "RAM",
    description: "Mức sử dụng bộ nhớ (% và dung lượng).",
  },
  {
    key: "disk",
    label: "Ổ đĩa",
    description: "Dung lượng ổ boot (C: hoặc /).",
  },
  {
    key: "network",
    label: "Mạng",
    description: "Tốc độ upload/download (bytes/giây).",
  },
  {
    key: "cpu_temperature",
    label: "Nhiệt CPU",
    description: "Best-effort qua WMI (Windows) — macOS thường unsupported.",
    badge: "Windows/macOS",
  },
  {
    key: "battery_temperature",
    label: "Nhiệt pin",
    description: "Laptop có pin — desktop thường unsupported.",
    badge: "Windows/macOS",
  },
  {
    key: "fan_speed",
    label: "Quạt (RPM)",
    description: "Best-effort — nhiều laptop không expose sensor.",
    badge: "Windows/macOS",
  },
];

function formatBytes(value: number): string {
  if (value >= 1024 ** 3) {
    return `${(value / 1024 ** 3).toFixed(1)} GB`;
  }
  if (value >= 1024 ** 2) {
    return `${(value / 1024 ** 2).toFixed(0)} MB`;
  }
  return `${(value / 1024).toFixed(0)} KB`;
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

function defaultPrefs(): SystemMetricsPreferences {
  return {
    cpu: false,
    memory: false,
    disk: false,
    network: false,
    cpu_temperature: false,
    battery_temperature: false,
    fan_speed: false,
  };
}

function MetricsPreview({ metrics }: { metrics: SystemMetricsResponse }) {
  return (
    <div className="space-y-3 rounded-xl border bg-muted/20 p-4">
      {metrics.cpu?.status === "ok" && metrics.cpu.usage_percent != null ? (
        <UsageCompactBar label="CPU" usedPercent={metrics.cpu.usage_percent} />
      ) : null}

      {metrics.memory?.status === "ok" && metrics.memory.used_percent != null ? (
        <div className="space-y-1">
          <UsageCompactBar label="RAM" usedPercent={metrics.memory.used_percent} />
          {metrics.memory.used_bytes != null && metrics.memory.total_bytes != null ? (
            <p className="text-xs text-muted-foreground">
              {formatBytes(metrics.memory.used_bytes)} / {formatBytes(metrics.memory.total_bytes)}
            </p>
          ) : null}
        </div>
      ) : null}

      {metrics.disk?.status === "ok" && metrics.disk.used_percent != null ? (
        <div className="space-y-1">
          <UsageCompactBar
            label={`Ổ đĩa${metrics.disk.mount_point ? ` (${metrics.disk.mount_point})` : ""}`}
            usedPercent={metrics.disk.used_percent}
          />
        </div>
      ) : null}

      {metrics.network?.status === "ok" ? (
        <div className="flex flex-wrap gap-3 text-xs">
          <span>
            ↓ {formatBps(metrics.network.download_bps ?? 0)}
          </span>
          <span>
            ↑ {formatBps(metrics.network.upload_bps ?? 0)}
          </span>
        </div>
      ) : null}

      {metrics.cpu_temperature ? (
        <MetricValueLine
          label="Nhiệt CPU"
          metric={metrics.cpu_temperature}
          format={(v) => `${v.toFixed(1)}°C`}
        />
      ) : null}

      {metrics.battery_temperature ? (
        <MetricValueLine
          label="Nhiệt pin"
          metric={metrics.battery_temperature}
          format={(v) => `${v.toFixed(1)}°C`}
        />
      ) : null}

      {metrics.fan ? (
        <MetricValueLine label="Quạt" metric={metrics.fan} format={(v) => `${Math.round(v)} RPM`} />
      ) : null}

      {metrics.updated_at ? (
        <p className="text-xs text-muted-foreground">
          Cập nhật {new Date(metrics.updated_at).toLocaleString()}
        </p>
      ) : null}
    </div>
  );
}

function MetricValueLine({
  label,
  metric,
  format,
}: {
  label: string;
  metric: { status: string; message?: string | null; celsius?: number | null; rpm?: number | null };
  format: (value: number) => string;
}) {
  const value = metric.celsius ?? metric.rpm;
  return (
    <div className="flex items-center justify-between gap-2 text-xs">
      <span className="text-muted-foreground">{label}</span>
      {metric.status === "ok" && value != null ? (
        <span className="font-medium tabular-nums">{format(value)}</span>
      ) : (
        <span className="text-muted-foreground">
          {metric.message ?? (metric.status === "unsupported" ? "Không hỗ trợ" : "Lỗi")}
        </span>
      )}
    </div>
  );
}

export function SystemMetricsPanel() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [metrics, setMetrics] = useState<SystemMetricsResponse | null>(null);
  const [port, setPort] = useState(8765);
  const [lanIp, setLanIp] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");

  const prefs = config?.system_metrics ?? defaultPrefs();
  const enabled = systemMetricsAnyEnabled(prefs);

  const loadConfigAndMetrics = async () => {
    const [cfg, status, server] = await Promise.all([
      getConfig(),
      getSystemMetricsStatus(),
      getServerStatus(),
    ]);
    setConfig(cfg);
    setMetrics(status);
    setPort(cfg.port);
    setLanIp(server.lan_ip ?? null);
  };

  const refreshMetrics = async () => {
    setLoading(true);
    setMessage("");
    try {
      const status = await getSystemMetricsStatus();
      setMetrics(status);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadConfigAndMetrics();
  }, []);

  useEffect(() => {
    if (!enabled) return;
    const id = window.setInterval(() => {
      void refreshMetrics();
    }, REFRESH_MS);
    return () => window.clearInterval(id);
  }, [enabled]);

  const handleToggle = async (key: MetricKey, next: boolean) => {
    if (!config) return;
    const nextPrefs = { ...defaultPrefs(), ...config.system_metrics, [key]: next };
    const nextConfig = { ...config, system_metrics: nextPrefs };
    setLoading(true);
    setMessage("");
    try {
      await updateConfig(nextConfig);
      setConfig(nextConfig);
      const status = await getSystemMetricsStatus();
      setMetrics(status);
      setMessage(next ? `Đã bật metric ${key}.` : `Đã tắt metric ${key}.`);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setLoading(false);
    }
  };

  const baseUrl = apiBaseUrl(port, lanIp);
  const metricsUrl = `${baseUrl}/system-metrics`;
  const metricsUrlWithToken = config?.require_token ? `${metricsUrl}?token=<token>` : metricsUrl;
  const curlGet = config?.require_token ? `curl "${metricsUrlWithToken}"` : `curl "${metricsUrl}"`;
  const curlPost = config?.require_token
    ? `curl -X POST "${metricsUrl}" \\\n  -H "Authorization: Bearer <token>"`
    : `curl -X POST "${metricsUrl}"`;

  const summary = useMemo(() => {
    if (!enabled) return "Chưa bật metric nào.";
    if (!metrics?.ok && metrics?.error) return metrics.error;
    if (metrics?.updated_at) {
      return `Cập nhật ${new Date(metrics.updated_at).toLocaleString()}`;
    }
    return "Đang thu thập thông số hệ thống…";
  }, [enabled, metrics]);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between gap-3">
          <span className="flex items-center gap-2">
            <Activity className="size-5 text-sky-500" />
            Thông số hệ thống
          </span>
          <Badge variant={enabled ? "default" : "secondary"}>
            {enabled ? "Enabled" : "Disabled"}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Bật từng loại metric cần theo dõi. Hiển thị trong popup tray và qua API{" "}
          <code className="rounded bg-muted px-1">/system-metrics</code>.
        </p>

        <div className="space-y-3">
          {METRIC_TOGGLES.map((item) => (
            <div
              key={item.key}
              className="flex flex-wrap items-start justify-between gap-4 rounded-lg border bg-muted/10 p-3"
            >
              <div className="min-w-0 space-y-1">
                <div className="flex flex-wrap items-center gap-2">
                  <Label htmlFor={`metric-${item.key}`}>{item.label}</Label>
                  {item.badge ? (
                    <Badge variant="outline" className="text-[10px]">
                      {item.badge}
                    </Badge>
                  ) : null}
                </div>
                <p className="text-sm text-muted-foreground">{item.description}</p>
              </div>
              <Switch
                id={`metric-${item.key}`}
                checked={Boolean(prefs[item.key])}
                disabled={loading || !config}
                onCheckedChange={(checked) => void handleToggle(item.key, checked)}
              />
            </div>
          ))}
        </div>

        <Separator />

        <div className="flex flex-wrap items-center justify-between gap-2">
          <p className="text-sm text-muted-foreground">{summary}</p>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={!enabled || loading}
            onClick={() => void refreshMetrics()}
          >
            <RefreshCw className={`mr-2 size-4 ${loading ? "animate-spin" : ""}`} />
            Làm mới
          </Button>
        </div>

        {enabled && metrics ? <MetricsPreview metrics={metrics} /> : null}

        {message ? <p className="text-sm text-muted-foreground">{message}</p> : null}

        {enabled ? (
          <>
            <Separator />
            <div className="space-y-2 font-mono text-xs">
              <p className="font-sans text-sm text-muted-foreground">GET — system metrics</p>
              <pre className="overflow-x-auto rounded-md bg-muted p-3">{curlGet}</pre>
              <p className="font-sans text-sm text-muted-foreground">POST — system metrics</p>
              <pre className="overflow-x-auto rounded-md bg-muted p-3">{curlPost}</pre>
            </div>
          </>
        ) : null}
      </CardContent>
    </Card>
  );
}
