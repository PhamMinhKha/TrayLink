import { useEffect, useMemo, useState } from "react";
import { RefreshCw } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import codexSymbol from "@/assets/codex-color.png";
import {
  apiBaseUrl,
  getCodexUsageStatus,
  getConfig,
  getServerStatus,
  updateConfig,
  type AppConfig,
  type UsageMonitorResponse,
  type UsageWindow,
} from "@/lib/tauri";

const REFRESH_MS = 60_000;

function usageBarColor(value: number): string {
  if (value < 50) return "bg-emerald-500";
  if (value < 80) return "bg-amber-500";
  return "bg-rose-500";
}

function formatResetTime(minutes: number): string {
  if (minutes >= 24 * 60) {
    const days = Math.max(1, Math.round(minutes / (24 * 60)));
    return `${days} ngày`;
  }

  if (minutes >= 60) {
    const hours = Math.max(1, Math.round(minutes / 60));
    return `${hours} giờ`;
  }

  return `${Math.max(1, Math.round(minutes))} phút`;
}

function UsageRow({ window }: { window: UsageWindow }) {
  const percent = Math.max(0, Math.min(100, window.used_percent));
  return (
    <div className="space-y-2 rounded-xl border bg-muted/20 p-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="font-medium">{window.label}</p>
          <p className="text-xs text-muted-foreground">
            Trạng thái: <span className="capitalize">{window.status}</span>
          </p>
        </div>
        <p className="text-lg font-semibold tabular-nums">{percent.toFixed(0)}%</p>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-muted">
        <div
          className={`h-full rounded-full transition-all ${usageBarColor(percent)}`}
          style={{ width: `${percent}%` }}
        />
      </div>
      <div className="flex flex-wrap justify-between gap-2 text-xs text-muted-foreground">
        <span>Còn lại {window.remaining_percent.toFixed(0)}%</span>
        <span>Reset ~{formatResetTime(window.reset_minutes)}</span>
      </div>
    </div>
  );
}

function usageSummary(status: UsageMonitorResponse | null): string {
  if (!status?.ok) {
    return status?.error ?? "Chưa có dữ liệu usage.";
  }

  if (!status.weekly_7d) {
    return "Đang chờ dữ liệu quota tuần từ Codex.";
  }

  return `Cập nhật ${status.updated_at ? new Date(status.updated_at).toLocaleString() : "gần đây"}`;
}

export function CodexUsagePanel() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [usage, setUsage] = useState<UsageMonitorResponse | null>(null);
  const [port, setPort] = useState(8765);
  const [lanIp, setLanIp] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");

  const enabled = Boolean(config?.codex_usage_monitor_enabled);
  const canRefresh = enabled && !loading;

  const loadConfigAndUsage = async () => {
    const [cfg, status, server] = await Promise.all([
      getConfig(),
      getCodexUsageStatus(),
      getServerStatus(),
    ]);
    setConfig(cfg);
    setUsage(status);
    setPort(cfg.port);
    setLanIp(server.lan_ip ?? null);
  };

  const refreshUsage = async () => {
    setLoading(true);
    setMessage("");
    try {
      const status = await getCodexUsageStatus();
      setUsage(status);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadConfigAndUsage();
  }, []);

  useEffect(() => {
    if (!enabled) return;
    const id = window.setInterval(() => {
      void refreshUsage();
    }, REFRESH_MS);
    return () => window.clearInterval(id);
  }, [enabled]);

  const handleToggle = async (nextEnabled: boolean) => {
    if (!config) return;
    const nextConfig = { ...config, codex_usage_monitor_enabled: nextEnabled };
    setLoading(true);
    setMessage("");
    try {
      await updateConfig(nextConfig);
      setConfig(nextConfig);
      if (nextEnabled) {
        const status = await getCodexUsageStatus();
        setUsage(status);
      } else {
        setUsage((prev) => (prev ? { ...prev, enabled: false, error: null } : prev));
      }
      setMessage(
        nextEnabled
          ? "Đã bật theo dõi quota Codex."
          : "Đã tắt theo dõi quota Codex.",
      );
    } catch (err) {
      setMessage(String(err));
    } finally {
      setLoading(false);
    }
  };

  const summary = useMemo(() => usageSummary(usage), [usage]);
  const baseUrl = apiBaseUrl(port, lanIp);
  const usageUrl = `${baseUrl}/codex-usage`;
  const usageUrlWithToken = config?.require_token ? `${usageUrl}?token=<token>` : usageUrl;
  const curlGet = config?.require_token ? `curl "${usageUrlWithToken}"` : `curl "${usageUrl}"`;
  const curlPost = config?.require_token
    ? `curl -X POST "${usageUrl}" \\\n  -H "Authorization: Bearer <token>"`
    : `curl -X POST "${usageUrl}"`;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between gap-3">
          <span className="flex items-center gap-2">
            <img src={codexSymbol} alt="Codex" className="h-6 w-6 rounded-md" />
            Theo dõi giới hạn Codex
          </span>
          <Badge variant={enabled ? "default" : "secondary"}>
            {enabled ? "Enabled" : "Disabled"}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="codex-usage-monitor">Bật lấy % sử dụng</Label>
            <p className="max-w-2xl text-sm text-muted-foreground">
              Khi bật, TrayLink sẽ đọc `~/.codex/auth.json` rồi gọi OpenAI Codex usage API
              để lấy quota tuần. Cấu trúc này để sẵn cho các client ngoài gọi lại
              `/codex-usage` luôn.
            </p>
          </div>
          <Switch
            id="codex-usage-monitor"
            checked={enabled}
            disabled={loading || !config}
            onCheckedChange={handleToggle}
          />
        </div>

        <Separator />

        {!enabled ? (
          <p className="text-sm text-muted-foreground">
            Bật switch ở trên để bắt đầu đồng bộ quota Codex.
          </p>
        ) : usage?.ok ? (
          <div className="space-y-4">
            <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
              <span>
                Provider: <strong className="text-foreground">{usage.provider}</strong>
              </span>
              {usage.plan && (
                <>
                  <span>•</span>
                  <span>Plan: <strong className="text-foreground">{usage.plan}</strong></span>
                </>
              )}
              {usage.account_id && (
                <>
                  <span>•</span>
                  <span>Account: <strong className="text-foreground">{usage.account_id}</strong></span>
                </>
              )}
              <span>•</span>
              <span>{summary}</span>
            </div>
            {usage.weekly_7d && <UsageRow window={usage.weekly_7d} />}
          </div>
        ) : (
          <div className="space-y-3">
            <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-4">
              <p className="text-sm font-medium text-destructive">Không lấy được quota</p>
              <p className="mt-1 text-sm text-muted-foreground">
                {usage?.error ?? "Không lấy được dữ liệu quota."}
              </p>
            </div>
          </div>
        )}

        {enabled && (
          <Button variant="outline" size="sm" onClick={() => void refreshUsage()} disabled={!canRefresh}>
            <RefreshCw className="size-3.5" />
            Làm mới
          </Button>
        )}

        <Separator />

        <div className="space-y-2">
          <p className="text-sm font-medium">API cho client</p>
          <p className="text-sm text-muted-foreground">
            Client bên ngoài có thể gọi endpoint này để lấy trạng thái usage hiện tại. Nếu
            bật token, dùng query `?token=...` hoặc header `Authorization: Bearer ...`.
          </p>
          <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
{`GET  ${usageUrlWithToken}
POST ${usageUrl}`}
          </pre>
          <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs whitespace-pre-wrap">{`${curlGet}\n\n${curlPost}`}</pre>
          <p className="text-xs text-muted-foreground">
            Response: `enabled`, `provider`, `plan`, `account_id`, `updated_at`, `weekly_7d`,
            `credits`, `ok`, `error`.
          </p>
        </div>

        {message && <p className="text-sm text-muted-foreground">{message}</p>}
      </CardContent>
    </Card>
  );
}
