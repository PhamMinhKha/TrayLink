import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";

export interface AppHotkeyBinding {
  id: string;
  name: string;
  accelerator: string;
  /** `open` = mở/focus app · `keys` = gửi phím tắt vào app */
  action?: "open" | "keys";
  /** `fa:floppy-disk` · `custom:uuid.png` */
  icon?: string | null;
}

export interface AppEntry {
  path: string;
  name?: string;
  args?: string[];
  url?: string;
  /** Bật mở app kèm URL (cho trình duyệt không tự nhận diện). */
  url_enabled?: boolean;
  /** Profile trình duyệt — Chromium: `Default` / `Profile 1`, Firefox: `firefox:...` */
  browser_profile?: string;
  hotkeys?: AppHotkeyBinding[];
}

export interface ExecEntry {
  internal?: boolean;
  win?: string;
  mac?: string;
  linux?: string;
}

export interface RemoteDeckLayout {
  display_order: string[];
  app_order: string[];
  command_order: string[];
  hidden_apps: string[];
  hidden_commands: string[];
  custom_icons: Record<string, string>;
}

export interface SystemMetricsPreferences {
  cpu?: boolean;
  memory?: boolean;
  disk?: boolean;
  network?: boolean;
  cpu_temperature?: boolean;
  battery_temperature?: boolean;
  fan_speed?: boolean;
}

export function systemMetricsAnyEnabled(prefs?: SystemMetricsPreferences | null): boolean {
  if (!prefs) return false;
  return Boolean(
    prefs.cpu ||
      prefs.memory ||
      prefs.disk ||
      prefs.network ||
      prefs.cpu_temperature ||
      prefs.battery_temperature ||
      prefs.fan_speed,
  );
}

export interface AppConfig {
  port: number;
  token: string;
  require_token?: boolean;
  allow_get?: boolean;
  usage_monitor_enabled?: boolean;
  codex_usage_monitor_enabled?: boolean;
  system_metrics?: SystemMetricsPreferences;
  autostart: boolean;
  apps: Record<string, AppEntry>;
  commands: Record<string, ExecEntry>;
  remote_deck?: RemoteDeckLayout;
}

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  duration_ms: number;
  client_ip: string;
}

export interface StatusResponse {
  online: boolean;
  version: string;
  port: number;
  https_port: number;
  lan_ip?: string | null;
  error?: string | null;
}

export interface UsageWindow {
  label: string;
  used_percent: number;
  remaining_percent: number;
  reset_minutes: number;
  status: string;
}

export interface UsageMonitorResponse {
  enabled: boolean;
  provider: string;
  plan?: string | null;
  account_id?: string | null;
  updated_at?: string | null;
  session_5h?: UsageWindow | null;
  weekly_7d?: UsageWindow | null;
  credits?: {
    has_credits: boolean;
    unlimited: boolean;
    balance?: number | null;
  } | null;
  ok: boolean;
  error?: string | null;
}

export type MetricStatus = "ok" | "unsupported" | "error";

export interface MetricCpu {
  status: MetricStatus;
  message?: string | null;
  usage_percent?: number | null;
}

export interface MetricMemory {
  status: MetricStatus;
  message?: string | null;
  used_percent?: number | null;
  used_bytes?: number | null;
  total_bytes?: number | null;
}

export interface MetricDisk {
  status: MetricStatus;
  message?: string | null;
  name?: string | null;
  mount_point?: string | null;
  used_percent?: number | null;
  available_bytes?: number | null;
  total_bytes?: number | null;
}

export interface MetricNetwork {
  status: MetricStatus;
  message?: string | null;
  upload_bps?: number | null;
  download_bps?: number | null;
}

export interface MetricTemperature {
  status: MetricStatus;
  message?: string | null;
  celsius?: number | null;
}

export interface MetricFan {
  status: MetricStatus;
  message?: string | null;
  rpm?: number | null;
}

export interface SystemMetricsResponse {
  enabled: boolean;
  ok: boolean;
  updated_at?: string | null;
  error?: string | null;
  cpu?: MetricCpu | null;
  memory?: MetricMemory | null;
  disk?: MetricDisk | null;
  network?: MetricNetwork | null;
  cpu_temperature?: MetricTemperature | null;
  battery_temperature?: MetricTemperature | null;
  fan?: MetricFan | null;
}

export interface MonitorStatusResponse {
  online: boolean;
  version: string;
  updated_at: string;
  claude: UsageMonitorResponse;
  codex: UsageMonitorResponse;
  system: SystemMetricsResponse;
}

export interface ClaudeDiagnosticItem {
  label: string;
  status: string;
  detail: string;
}

export interface ClaudeDiagnosticsResponse {
  claude_app_installed: boolean;
  claude_app_executable: boolean;
  claude_cli_on_path: boolean;
  keychain_services_checked: string[];
  keychain_token_found: boolean;
  credentials_path?: string | null;
  credentials_file_exists: boolean;
  credentials_token_found: boolean;
  auth_store_label?: string;
  summary: string;
  recommendation: string;
  findings: ClaudeDiagnosticItem[];
}

export function apiHost(lanIp?: string | null): string {
  return lanIp || "127.0.0.1";
}

export function apiBaseUrl(port: number, lanIp?: string | null): string {
  return `http://${apiHost(lanIp)}:${port}`;
}

export function apiBaseUrlHttps(httpsPort: number, lanIp?: string | null): string {
  return `https://${apiHost(lanIp)}:${httpsPort}`;
}

export function runningInTauri(): boolean {
  return isTauri();
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      "Dashboard cần mở trong app TrayLink (icon tray → Open Dashboard), không phải trình duyệt.",
    );
  }
  return tauriInvoke<T>(cmd, args);
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function showDashboard(tab?: string): Promise<void> {
  return invoke("show_dashboard", { tab: tab ?? null });
}

export async function updateConfig(config: AppConfig): Promise<void> {
  return invoke("update_config", { config });
}

export async function restartServer(): Promise<void> {
  return invoke("restart_api_server");
}

export async function getRequestLogs(): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("get_request_logs");
}

export async function regenerateToken(): Promise<string> {
  return invoke<string>("regenerate_token");
}

export async function setAutostart(enabled: boolean): Promise<void> {
  return invoke("set_autostart", { enabled });
}

export async function getAutostartEnabled(): Promise<boolean> {
  return invoke<boolean>("get_autostart_enabled");
}

export async function getServerUptime(): Promise<number> {
  return invoke<number>("get_server_uptime");
}

export async function getClaudeUsageStatus(): Promise<UsageMonitorResponse> {
  return invoke<UsageMonitorResponse>("get_claude_usage_status");
}

export async function getClaudeDiagnostics(): Promise<ClaudeDiagnosticsResponse> {
  return invoke<ClaudeDiagnosticsResponse>("get_claude_diagnostics");
}

export async function getCodexUsageStatus(): Promise<UsageMonitorResponse> {
  return invoke<UsageMonitorResponse>("get_codex_usage_status");
}

export async function getSystemMetricsStatus(): Promise<SystemMetricsResponse> {
  return invoke<SystemMetricsResponse>("get_system_metrics_status");
}

export async function getMonitorStatus(): Promise<MonitorStatusResponse> {
  return invoke<MonitorStatusResponse>("get_monitor_status");
}

async function fetchStatusFromApi(port: number): Promise<StatusResponse> {
  const response = await fetch(`http://127.0.0.1:${port}/status`);
  if (!response.ok) {
    throw new Error(`API server không phản hồi trên port ${port}`);
  }
  return response.json();
}

export async function getServerStatus(port = 8765): Promise<StatusResponse> {
  if (isTauri()) {
    return invoke<StatusResponse>("get_server_status");
  }
  return fetchStatusFromApi(port);
}

export interface InstalledApp {
  name: string;
  path: string;
}

export interface BrowserProfile {
  id: string;
  name: string;
}

export async function listInstalledApps(): Promise<InstalledApp[]> {
  return invoke<InstalledApp[]>("list_installed_apps_cmd");
}

export async function listBrowserProfiles(path: string): Promise<BrowserProfile[]> {
  return invoke<BrowserProfile[]>("list_browser_profiles", { path });
}

export async function resolveLaunchPath(path: string): Promise<string> {
  return invoke<string>("resolve_launch_path_cmd", { path });
}

export async function browseAppPath(): Promise<string | null> {
  if (!isTauri()) {
    throw new Error("Chọn app chỉ dùng được trong app TrayLink.");
  }

  const { open } = await import("@tauri-apps/plugin-dialog");
  const isMac = navigator.userAgent.toLowerCase().includes("mac");

  if (isMac) {
    const selected = await open({
      title: "Chọn ứng dụng (.app)",
      multiple: false,
      directory: true,
      defaultPath: "/Applications",
    });
    return typeof selected === "string" ? selected : null;
  }

  const selected = await open({
    title: "Chọn ứng dụng",
    multiple: false,
    directory: false,
    filters: [
      {
        name: "Application",
        extensions: ["exe", "lnk", "desktop"],
      },
    ],
  });

  if (selected === null) {
    return null;
  }

  const raw = typeof selected === "string" ? selected : selected[0] ?? null;
  if (!raw) {
    return null;
  }

  return resolveLaunchPath(raw);
}

export function slugifyAppKey(name: string): string {
  return name
    .replace(/\.app$/i, "")
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function slugifyHotkeyId(name: string): string {
  return slugifyAppKey(name) || "hotkey";
}

export async function getAppIcon(path: string): Promise<string | null> {
  return invoke<string | null>("get_app_icon", { path });
}

export async function browseDeckIconPath(): Promise<string | null> {
  if (!isTauri()) {
    throw new Error("Chọn icon chỉ dùng được trong app TrayLink.");
  }

  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    title: "Chọn hình icon",
    multiple: false,
    directory: false,
    filters: [
      {
        name: "Image",
        extensions: ["png", "jpg", "jpeg", "webp", "gif", "svg"],
      },
    ],
  });

  if (selected === null) {
    return null;
  }

  return typeof selected === "string" ? selected : selected[0] ?? null;
}

export async function setDeckIconFromFile(
  itemType: "app" | "cmd",
  key: string,
  sourcePath: string,
): Promise<string> {
  return invoke<string>("set_deck_icon_from_file", {
    itemType,
    key,
    sourcePath,
  });
}

export async function clearDeckIcon(itemType: "app" | "cmd", key: string): Promise<void> {
  return invoke("clear_deck_icon", { itemType, key });
}

export async function getDeckIconDataUrl(
  itemType: "app" | "cmd",
  key: string,
): Promise<string | null> {
  return invoke<string | null>("get_deck_icon_data_url", { itemType, key });
}

export async function getHotkeyIconDataUrl(
  appKey: string,
  hotkeyId: string,
): Promise<string | null> {
  return invoke<string | null>("get_hotkey_icon_data_url", { appKey, hotkeyId });
}

export async function setHotkeyIconFromFile(
  appKey: string,
  hotkeyId: string,
  sourcePath: string,
): Promise<string> {
  return invoke<string>("set_hotkey_icon_from_file", {
    appKey,
    hotkeyId,
    sourcePath,
  });
}

export async function cleanupHotkeyIcon(icon: string): Promise<void> {
  return invoke("cleanup_hotkey_icon", { icon });
}

export async function testOpenApp(appKey: string): Promise<string> {
  return invoke<string>("test_open_app", { appKey });
}

export interface AccessibilityStatus {
  supported: boolean;
  trusted: boolean;
  dev_build?: boolean;
  executable_path?: string | null;
  dev_app_path?: string | null;
  codesign_identifier?: string | null;
  stable_signature?: boolean;
  hint: string;
}

export async function getAccessibilityStatus(): Promise<AccessibilityStatus> {
  return invoke<AccessibilityStatus>("get_accessibility_status");
}

export async function promptAccessibilityPermission(): Promise<boolean> {
  return invoke<boolean>("prompt_accessibility_permission");
}

export async function openAccessibilitySettings(): Promise<void> {
  return invoke("open_accessibility_settings");
}

export async function signDevBinary(): Promise<string> {
  return invoke<string>("sign_dev_binary_cmd");
}

export async function openDevAppBundle(): Promise<string> {
  return invoke<string>("open_dev_app_bundle_cmd");
}

export async function resetAccessibility(): Promise<string> {
  return invoke<string>("reset_accessibility_cmd");
}

export async function testHotkeyInput(): Promise<string> {
  return invoke<string>("test_hotkey_input");
}

export function apiGetUrl(
  port: number,
  path: string,
  params: Record<string, string | undefined>,
  requireToken: boolean,
  token: string,
  lanIp?: string | null,
): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value) {
      search.set(key, value);
    }
  }
  if (requireToken && token) {
    search.set("token", token);
  }
  const query = search.toString();
  return `${apiBaseUrl(port, lanIp)}${path}${query ? `?${query}` : ""}`;
}

export function remoteDeckUrl(
  port: number,
  requireToken: boolean,
  token: string,
  lanIp?: string | null,
): string {
  return apiGetUrl(port, "/remote", {}, requireToken, token, lanIp);
}

export function remoteDeckHttpsUrl(
  httpsPort: number,
  requireToken: boolean,
  token: string,
  lanIp?: string | null,
): string {
  const search = new URLSearchParams();
  if (requireToken && token) search.set("token", token);
  const query = search.toString();
  return `${apiBaseUrlHttps(httpsPort, lanIp)}/remote${query ? `?${query}` : ""}`;
}

export function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  return `${h}h ${m}m ${s}s`;
}
