import { useEffect, useMemo, useState } from "react";
import { RefreshCw } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import claudeSymbol from "@/assets/claude-ai-symbol.svg";
import {
  apiBaseUrl,
  getClaudeDiagnostics,
  getClaudeUsageStatus,
  getConfig,
  getServerStatus,
  updateConfig,
  type ClaudeDiagnosticsResponse,
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

  if (!status.session_5h || !status.weekly_7d) {
    return "Đang chờ dữ liệu quota từ Claude Code.";
  }

  return `Cập nhật ${status.updated_at ? new Date(status.updated_at).toLocaleString() : "gần đây"}`;
}

export function ClaudeUsagePanel() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [usage, setUsage] = useState<UsageMonitorResponse | null>(null);
  const [port, setPort] = useState(8765);
  const [lanIp, setLanIp] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const [diagnostics, setDiagnostics] = useState<ClaudeDiagnosticsResponse | null>(null);
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const [diagnosticsLoading, setDiagnosticsLoading] = useState(false);
  const [diagnosticsError, setDiagnosticsError] = useState("");

  const enabled = Boolean(config?.usage_monitor_enabled);
  const canRefresh = enabled && !loading;

  const loadConfigAndUsage = async () => {
    const [cfg, status, server] = await Promise.all([
      getConfig(),
      getClaudeUsageStatus(),
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
      const status = await getClaudeUsageStatus();
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
    if (!enabled) {
      return;
    }

    const id = window.setInterval(() => {
      void refreshUsage();
    }, REFRESH_MS);

    return () => window.clearInterval(id);
  }, [enabled]);

  const handleToggle = async (nextEnabled: boolean) => {
    if (!config) return;

    const nextConfig = { ...config, usage_monitor_enabled: nextEnabled };
    setLoading(true);
    setMessage("");
    try {
      await updateConfig(nextConfig);
      setConfig(nextConfig);
      if (nextEnabled) {
        const status = await getClaudeUsageStatus();
        setUsage(status);
      } else {
        setUsage((prev) => (prev ? { ...prev, enabled: false, error: null } : prev));
      }
      setMessage(
        nextEnabled
          ? "Đã bật theo dõi quota Claude Code."
          : "Đã tắt theo dõi quota Claude Code.",
      );
    } catch (err) {
      setMessage(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleDiagnose = async () => {
    setDiagnosticsOpen(true);
    setDiagnosticsLoading(true);
    setDiagnosticsError("");
    try {
      const result = await getClaudeDiagnostics();
      setDiagnostics(result);
    } catch (err) {
      setDiagnostics(null);
      setDiagnosticsError(String(err));
    } finally {
      setDiagnosticsLoading(false);
    }
  };

  const summary = useMemo(() => usageSummary(usage), [usage]);
  const baseUrl = apiBaseUrl(port, lanIp);
  const usageUrl = `${baseUrl}/claude-usage`;
  const usageUrlWithToken = config?.require_token ? `${usageUrl}?token=<token>` : usageUrl;
  const curlGet = config?.require_token
    ? `curl "${usageUrlWithToken}"`
    : `curl "${usageUrl}"`;
  const curlPost = config?.require_token
    ? `curl -X POST "${usageUrl}" \\
  -H "Authorization: Bearer <token>"`
    : `curl -X POST "${usageUrl}"`;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between gap-3">
          <span className="flex items-center gap-2">
            <img src={claudeSymbol} alt="Claude" className="h-6 w-6" />
            Theo dõi giới hạn Claude Code
          </span>
          <Badge variant={enabled ? "default" : "secondary"}>
            {enabled ? "Enabled" : "Disabled"}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="space-y-1">
            <Label htmlFor="claude-usage-monitor">Bật lấy % sử dụng</Label>
            <p className="max-w-2xl text-sm text-muted-foreground">
              Khi bật, TrayLink sẽ đọc token Claude Code local rồi gọi Anthropic để lấy mức
              sử dụng 5 giờ và 7 ngày. Cấu trúc này để sẵn để sau này thêm Codex không phải
              đổi lại UI.
            </p>
          </div>
          <div className="flex flex-col items-end gap-2">
            <Switch
              id="claude-usage-monitor"
              checked={enabled}
              disabled={loading || !config}
              onCheckedChange={handleToggle}
            />
            <Button variant="outline" size="sm" onClick={() => void handleDiagnose()}>
              Chẩn đoán Claude
            </Button>
          </div>
        </div>

        <Separator />

        {!enabled ? (
          <p className="text-sm text-muted-foreground">
            Bật switch ở trên để bắt đầu đồng bộ quota Claude Code.
          </p>
        ) : usage?.ok ? (
          <div className="space-y-4">
            <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
              <span>Provider: <strong className="text-foreground">{usage.provider}</strong></span>
              <span>•</span>
              <span>{summary}</span>
            </div>
            {usage.session_5h && <UsageRow window={usage.session_5h} />}
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
          <Button
            variant="outline"
            size="sm"
            onClick={() => void refreshUsage()}
            disabled={!canRefresh}
          >
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
            Response: `enabled`, `provider`, `updated_at`, `session_5h`, `weekly_7d`, `ok`,
            `error`.
          </p>
        </div>

        {message && <p className="text-sm text-muted-foreground">{message}</p>}
      </CardContent>

      <Dialog open={diagnosticsOpen} onOpenChange={setDiagnosticsOpen}>
        <DialogContent className="max-h-[85vh] max-w-2xl overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Chẩn đoán Claude</DialogTitle>
            <DialogDescription>
              Mình kiểm tra nguồn auth trên máy để biết vì sao TrayLink không đọc được token.
            </DialogDescription>
          </DialogHeader>

          {diagnosticsLoading ? (
            <p className="text-sm text-muted-foreground">Đang kiểm tra...</p>
          ) : diagnosticsError ? (
            <div className="space-y-2">
              <p className="text-sm text-destructive">{diagnosticsError}</p>
            </div>
          ) : diagnostics ? (
            <div className="space-y-4 text-sm">
              <div className="rounded-lg border bg-muted/20 p-4">
                <p className="font-medium">Kết luận</p>
                <p className="mt-1 text-muted-foreground">{diagnostics.summary}</p>
                <p className="mt-2">{diagnostics.recommendation}</p>
              </div>

              <div className="space-y-2">
                {diagnostics.findings.map((item) => (
                  <div key={item.label} className="rounded-lg border p-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <p className="font-medium">{item.label}</p>
                      <Badge variant="secondary">{item.status}</Badge>
                    </div>
                    <p className="mt-1 text-muted-foreground">{item.detail}</p>
                  </div>
                ))}
              </div>

              <div className="rounded-lg border bg-muted/20 p-4">
                <p className="font-medium">Tín hiệu đã kiểm tra</p>
                <p className="mt-1 text-muted-foreground">
                  Keychain: {diagnostics.keychain_services_checked.join(", ")}
                </p>
                <p className="mt-1 text-muted-foreground">
                  Claude.app: {diagnostics.claude_app_installed ? "Có" : "Không"}
                  {" · "}
                  CLI: {diagnostics.claude_cli_on_path ? "Có" : "Không"}
                </p>
                <p className="mt-1 text-muted-foreground">
                  Credentials file: {diagnostics.credentials_path ?? "Không xác định"}
                  {" · "}
                  Tồn tại: {diagnostics.credentials_file_exists ? "Có" : "Không"}
                </p>
              </div>
            </div>
          ) : null}
        </DialogContent>
      </Dialog>
    </Card>
  );
}
