import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import {
  apiBaseUrl,
  apiGetUrl,
  getAutostartEnabled,
  getConfig,
  getServerStatus,
  regenerateToken,
  restartServer,
  setAutostart,
  updateConfig,
} from "@/lib/tauri";
import { AccessibilityPanel } from "@/components/AccessibilityPanel";
import { ClaudeUsagePanel } from "@/components/ClaudeUsagePanel";
import { CodexUsagePanel } from "@/components/CodexUsagePanel";
import { SystemMetricsPanel } from "@/components/SystemMetricsPanel";
import { ThemeModeSwitch } from "@/components/ThemeToggle";

function authHeader(requireToken: boolean, token: string, showToken: boolean) {
  if (!requireToken) {
    return "";
  }
  return `  -H "Authorization: Bearer ${showToken ? token : "<token>"}" \\\n`;
}

export function SettingsPanel() {
  const [port, setPort] = useState("8765");
  const [token, setToken] = useState("");
  const [requireToken, setRequireToken] = useState(false);
  const [allowGet, setAllowGet] = useState(true);
  const [showToken, setShowToken] = useState(false);
  const [autostart, setAutostartState] = useState(false);
  const [lanIp, setLanIp] = useState<string | null>(null);
  const [message, setMessage] = useState("");

  const load = async () => {
    const [config, status] = await Promise.all([getConfig(), getServerStatus()]);
    setPort(String(config.port));
    setToken(config.token);
    setRequireToken(config.require_token ?? false);
    setAllowGet(config.allow_get ?? true);
    setLanIp(status.lan_ip ?? null);
    const enabled = await getAutostartEnabled();
    setAutostartState(enabled);
  };

  useEffect(() => {
    load();
  }, []);

  const applyPort = async () => {
    const config = await getConfig();
    const nextPort = Number(port);
    if (!nextPort || nextPort < 1024 || nextPort > 65535) {
      setMessage("Port không hợp lệ (1024-65535)");
      return;
    }
    await updateConfig({ ...config, port: nextPort });
    await restartServer();
    setMessage("Đã cập nhật port và restart server");
  };

  const handleRequireToken = async (enabled: boolean) => {
    const config = await getConfig();
    let nextToken = config.token;
    if (enabled && !nextToken) {
      nextToken = await regenerateToken();
      setToken(nextToken);
    }
    await updateConfig({ ...config, require_token: enabled, token: nextToken });
    setRequireToken(enabled);
    setMessage(enabled ? "Đã bật xác thực token" : "Đã tắt token — API mở cho LAN");
  };

  const handleAllowGet = async (enabled: boolean) => {
    const config = await getConfig();
    await updateConfig({ ...config, allow_get: enabled });
    setAllowGet(enabled);
    setMessage(
      enabled
        ? "Đã bật GET — có thể gọi API bằng link URL"
        : "Đã tắt GET — chỉ dùng POST",
    );
  };

  const handleRegenerateToken = async () => {
    const newToken = await regenerateToken();
    setToken(newToken);
    setMessage("Đã tạo token mới");
  };

  const copyToken = async () => {
    await navigator.clipboard.writeText(token);
    setMessage("Đã copy token");
  };

  const handleAutostart = async (enabled: boolean) => {
    await setAutostart(enabled);
    setAutostartState(enabled);
    setMessage(enabled ? "Đã bật autostart" : "Đã tắt autostart");
  };

  const portNum = Number(port) || 8765;
  const tokenDisplay = showToken ? token : "<token>";
  const openAppGetUrl = apiGetUrl(
    portNum,
    "/open-app",
    { app: "my-app" },
    requireToken,
    tokenDisplay,
    lanIp,
  );
  const openFileGetUrl = apiGetUrl(
    portNum,
    "/open-file",
    { path: "/path/to/file.mp4" },
    requireToken,
    tokenDisplay,
    lanIp,
  );
  const execGetUrl = apiGetUrl(
    portNum,
    "/exec",
    { cmd: "restart_server" },
    requireToken,
    tokenDisplay,
    lanIp,
  );
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Cài đặt</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <Label>Giao diện</Label>
            <p className="text-sm text-muted-foreground">
              Light / Dark áp dụng cho dashboard và popup tray.
            </p>
            <ThemeModeSwitch />
          </div>

          <Separator />

          <div className="space-y-2">
            <Label htmlFor="port">Port</Label>
            <div className="flex gap-2">
              <Input id="port" value={port} onChange={(e) => setPort(e.target.value)} />
              <Button onClick={applyPort}>Apply</Button>
            </div>
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="allow-get">Cho phép GET (link URL)</Label>
              <p className="text-sm text-muted-foreground">
                Bật để gọi API bằng URL trong trình duyệt, Stream Deck, shortcut…
              </p>
            </div>
            <Switch id="allow-get" checked={allowGet} onCheckedChange={handleAllowGet} />
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="require-token">Yêu cầu token API</Label>
              <p className="text-sm text-muted-foreground">
                Tắt (mặc định) để dùng trên mạng LAN không cần Authorization header
              </p>
            </div>
            <Switch
              id="require-token"
              checked={requireToken}
              onCheckedChange={handleRequireToken}
            />
          </div>

          {requireToken && (
            <div className="space-y-2">
              <Label>API Token</Label>
              <div className="flex flex-wrap gap-2">
                <Input
                  readOnly
                  className="min-w-[200px] flex-1"
                  value={showToken ? token : "•".repeat(Math.min(token.length, 24))}
                />
                <Button variant="outline" onClick={() => setShowToken((v) => !v)}>
                  {showToken ? "Ẩn" : "Hiện"}
                </Button>
                <Button variant="outline" onClick={copyToken}>
                  Copy
                </Button>
                <Button variant="destructive" onClick={handleRegenerateToken}>
                  Regenerate
                </Button>
              </div>
              {allowGet && (
                <p className="text-xs text-muted-foreground">
                  Với GET, truyền token qua query: <code>?token=...</code>
                </p>
              )}
            </div>
          )}

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="autostart">Autostart khi boot</Label>
              <p className="text-sm text-muted-foreground">
                Tự khởi động TrayLink cùng hệ thống
              </p>
            </div>
            <Switch id="autostart" checked={autostart} onCheckedChange={handleAutostart} />
          </div>
        </CardContent>
      </Card>

      <ClaudeUsagePanel />

      <CodexUsagePanel />

      <SystemMetricsPanel />

      <AccessibilityPanel />

      <Card>
        <CardHeader>
          <CardTitle>Ví dụ API</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 font-mono text-xs">
          <p className="font-sans text-sm text-muted-foreground">
            Base URL LAN: <strong>{apiBaseUrl(portNum, lanIp)}</strong>
            {!lanIp && " (chưa phát hiện IP — dùng 127.0.0.1, chỉ máy này gọi được)"}
          </p>
          <p className="font-sans text-sm text-muted-foreground">GET — status</p>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl ${apiBaseUrl(portNum, lanIp)}/status`}</pre>

          <p className="font-sans text-sm text-muted-foreground">
            GET — monitor (Claude + Codex + System, theo toggle trong Settings)
          </p>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">
            {allowGet
              ? requireToken
                ? `curl "${apiBaseUrl(portNum, lanIp)}/monitor-status?token=${showToken ? token : "<token>"}"`
                : `curl "${apiBaseUrl(portNum, lanIp)}/monitor-status"`
              : `# Bật "Cho phép GET" hoặc dùng POST:\ncurl -X POST "${apiBaseUrl(portNum, lanIp)}/monitor-status"${requireToken ? ` \\\n  -H "Authorization: Bearer ${showToken ? token : "<token>"}"` : ""}`}
          </pre>

          {allowGet ? (
            <>
              <p className="font-sans text-sm text-muted-foreground">
                GET — mở link URL trực tiếp (dán vào trình duyệt / Stream Deck trên LAN)
              </p>
              <pre className="overflow-x-auto rounded-md bg-muted p-3">{openAppGetUrl}</pre>
              <pre className="overflow-x-auto rounded-md bg-muted p-3">{openFileGetUrl}</pre>
              <pre className="overflow-x-auto rounded-md bg-muted p-3">{execGetUrl}</pre>
            </>
          ) : (
            <p className="font-sans text-sm text-muted-foreground">
              GET đang tắt — bật &quot;Cho phép GET&quot; ở trên để dùng link URL.
            </p>
          )}

          <p className="font-sans text-sm text-muted-foreground">POST — curl</p>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl -X POST ${apiBaseUrl(portNum, lanIp)}/open-app \\
${authHeader(requireToken, token, showToken)}  -H "Content-Type: application/json" \\
  -d '{"app":"my-app"}'`}</pre>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl -X POST ${apiBaseUrl(portNum, lanIp)}/open-file \\
${authHeader(requireToken, token, showToken)}  -H "Content-Type: application/json" \\
  -d '{"path":"/path/to/file.mp4"}'`}</pre>
          <pre className="overflow-x-auto rounded-md bg-muted p-3">{`curl -X POST ${apiBaseUrl(portNum, lanIp)}/exec \\
${authHeader(requireToken, token, showToken)}  -H "Content-Type: application/json" \\
  -d '{"cmd":"restart_server"}'`}</pre>
        </CardContent>
      </Card>

      {message && <p className="text-sm text-muted-foreground">{message}</p>}
    </div>
  );
}
