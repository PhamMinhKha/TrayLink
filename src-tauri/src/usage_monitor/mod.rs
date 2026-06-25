mod auth;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Serialize;

use auth::{
    claude_config_dir, claude_json_path, credentials_path, read_token_from_claude_json,
    read_token_from_env, read_token_from_file,
};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const ANTHROPIC_BETA: &str = "oauth-2025-04-20";
const USER_AGENT_VALUE: &str = "claude-code/2.1.5";
const MODEL: &str = "claude-haiku-4-5-20251001";

#[derive(Debug, Clone, Serialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub reset_minutes: i64,
    /// Unix timestamp (seconds) when the quota window resets; 0 if unknown.
    pub reset_at: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageMonitorResponse {
    pub enabled: bool,
    pub provider: String,
    pub updated_at: Option<String>,
    pub session_5h: Option<UsageWindow>,
    pub weekly_7d: Option<UsageWindow>,
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaudeDiagnosticItem {
    pub label: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaudeDiagnosticsResponse {
    pub claude_app_installed: bool,
    pub claude_app_executable: bool,
    pub claude_cli_on_path: bool,
    pub keychain_services_checked: Vec<String>,
    pub keychain_token_found: bool,
    pub credentials_path: Option<String>,
    pub credentials_file_exists: bool,
    pub credentials_token_found: bool,
    pub auth_store_label: String,
    pub summary: String,
    pub recommendation: String,
    pub findings: Vec<ClaudeDiagnosticItem>,
}

impl UsageMonitorResponse {
    fn disabled() -> Self {
        Self {
            enabled: false,
            provider: "claude".to_string(),
            updated_at: None,
            session_5h: None,
            weekly_7d: None,
            ok: true,
            error: None,
        }
    }
}

fn claude_cli_on_path() -> bool {
    #[cfg(windows)]
    {
        return Command::new("where")
            .arg("claude")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
    }

    #[cfg(not(windows))]
    {
        Command::new("sh")
            .args(["-lc", "command -v claude >/dev/null 2>&1"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

fn read_token_from_secure_store() -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::read_token_from_secure_store();
    }

    #[cfg(windows)]
    {
        return windows::read_token_from_secure_store();
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    {
        Ok(None)
    }
}

fn secure_store_targets() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        return macos::secure_store_targets();
    }

    #[cfg(windows)]
    {
        return windows::secure_store_targets();
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    {
        Vec::new()
    }
}

fn secure_store_label() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return macos::secure_store_label();
    }

    #[cfg(windows)]
    {
        return windows::secure_store_label();
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    {
        "Secure store"
    }
}

fn claude_desktop_installed() -> (bool, bool) {
    #[cfg(target_os = "macos")]
    {
        return macos::claude_desktop_installed();
    }

    #[cfg(windows)]
    {
        return windows::claude_desktop_installed();
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    {
        (false, false)
    }
}

fn claude_desktop_detail(installed: bool) -> String {
    #[cfg(target_os = "macos")]
    {
        return macos::claude_desktop_detail(installed);
    }

    #[cfg(windows)]
    {
        return windows::claude_desktop_detail(installed);
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    {
        if installed {
            "Phát hiện Claude Desktop.".to_string()
        } else {
            "Không thấy Claude Desktop.".to_string()
        }
    }
}

fn build_claude_diagnostics() -> ClaudeDiagnosticsResponse {
    let (claude_app_installed, claude_app_executable) = claude_desktop_installed();
    let secure_store_targets = secure_store_targets();
    let secure_store_token_found = read_token_from_secure_store()
        .ok()
        .flatten()
        .is_some();
    let env_token_found = read_token_from_env().is_some();
    let credentials_path = credentials_path().map(|path| path.display().to_string());
    let claude_json = claude_json_path().map(|path| path.display().to_string());
    let config_dir = claude_config_dir().map(|path| path.display().to_string());
    let credentials_file_exists = credentials_path
        .as_deref()
        .map(std::path::Path::new)
        .is_some_and(|path| path.exists());
    let credentials_token_found = read_token_from_file().ok().flatten().is_some();
    let claude_json_exists = claude_json
        .as_deref()
        .map(std::path::Path::new)
        .is_some_and(|path| path.exists());
    let claude_json_token_found = read_token_from_claude_json().ok().flatten().is_some();
    let claude_cli_on_path = claude_cli_on_path();
    let auth_store_label = secure_store_label().to_string();

    let mut findings = Vec::new();
    findings.push(ClaudeDiagnosticItem {
        label: "Claude Desktop".to_string(),
        status: if claude_app_installed {
            "detected".to_string()
        } else {
            "not found".to_string()
        },
        detail: claude_desktop_detail(claude_app_installed),
    });
    findings.push(ClaudeDiagnosticItem {
        label: "Claude CLI".to_string(),
        status: if claude_cli_on_path {
            "available".to_string()
        } else {
            "missing".to_string()
        },
        detail: if claude_cli_on_path {
            "Có lệnh `claude` trong PATH.".to_string()
        } else {
            "Không thấy `claude` CLI trong PATH. Cài Claude Code CLI rồi chạy `claude login`.".to_string()
        },
    });
    findings.push(ClaudeDiagnosticItem {
        label: auth_store_label.clone(),
        status: if secure_store_token_found {
            "token found".to_string()
        } else {
            "no token".to_string()
        },
        detail: format!(
            "Đã kiểm tra các target: {}.",
            secure_store_targets.join(", ")
        ),
    });
    findings.push(ClaudeDiagnosticItem {
        label: "Credentials file".to_string(),
        status: if credentials_file_exists {
            if credentials_token_found {
                "token found".to_string()
            } else {
                "file found".to_string()
            }
        } else {
            "missing".to_string()
        },
        detail: credentials_path
            .as_ref()
            .map(|path| format!("Đường dẫn kiểm tra: {path}"))
            .unwrap_or_else(|| "Không xác định được đường dẫn credentials.".to_string()),
    });
    findings.push(ClaudeDiagnosticItem {
        label: ".claude.json".to_string(),
        status: if claude_json_token_found {
            "token found".to_string()
        } else if claude_json_exists {
            "file found".to_string()
        } else {
            "missing".to_string()
        },
        detail: claude_json
            .as_ref()
            .map(|path| format!("Đường dẫn kiểm tra: {path}"))
            .unwrap_or_else(|| "Không xác định được đường dẫn .claude.json.".to_string()),
    });
    if env_token_found {
        findings.push(ClaudeDiagnosticItem {
            label: "CLAUDE_CODE_OAUTH_TOKEN".to_string(),
            status: "token found".to_string(),
            detail: "Biến môi trường CLAUDE_CODE_OAUTH_TOKEN đang được đặt.".to_string(),
        });
    }

    let has_token = secure_store_token_found
        || credentials_token_found
        || claude_json_token_found
        || env_token_found;
    let summary = if has_token {
        "Đã tìm thấy nguồn auth Claude Code phù hợp.".to_string()
    } else if claude_app_installed && !claude_cli_on_path {
        "Phát hiện Claude Desktop, nhưng chưa thấy token Claude Code CLI.".to_string()
    } else {
        "Không tìm thấy nguồn auth Claude Code nào trên máy này.".to_string()
    };

    let recommendation = if has_token {
        "Token đã sẵn sàng. Nếu quota vẫn lỗi, thử chạy `claude login` để làm mới token.".to_string()
    } else if claude_cli_on_path {
        "Mở terminal, chạy `claude login`, rồi bật lại theo dõi usage.".to_string()
    } else {
        #[cfg(windows)]
        let install_hint = "Cài Claude Code CLI trên Windows (npm i -g @anthropic-ai/claude-code hoặc installer chính thức), chạy `claude login`, rồi mở lại TrayLink. Nếu login báo thành công nhưng không có file credentials, xóa thư mục `*.lock` trong %USERPROFILE%\\.claude rồi login lại.";
        #[cfg(not(windows))]
        let install_hint = "Cài Claude Code CLI và đăng nhập trên máy này, rồi mở lại TrayLink.";

        if let Some(dir) = config_dir {
            format!("{install_hint} Thư mục cấu hình Claude: {dir}.")
        } else {
            install_hint.to_string()
        }
    };

    ClaudeDiagnosticsResponse {
        claude_app_installed,
        claude_app_executable,
        claude_cli_on_path,
        keychain_services_checked: secure_store_targets,
        keychain_token_found: secure_store_token_found,
        credentials_path,
        credentials_file_exists,
        credentials_token_found,
        auth_store_label,
        summary,
        recommendation,
        findings,
    }
}

pub fn diagnose_claude() -> ClaudeDiagnosticsResponse {
    build_claude_diagnostics()
}

fn build_missing_token_error() -> String {
    let diag = build_claude_diagnostics();
    format!("{} {}", diag.summary, diag.recommendation)
}

pub fn read_access_token() -> Result<String, String> {
    if let Some(token) = read_token_from_env() {
        return Ok(token);
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(token) = read_token_from_secure_store()? {
            return Ok(token);
        }
        if let Some(token) = read_token_from_file()? {
            return Ok(token);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(token) = read_token_from_file()? {
            return Ok(token);
        }
        if let Some(token) = read_token_from_claude_json()? {
            return Ok(token);
        }
        if let Some(token) = read_token_from_secure_store()? {
            return Ok(token);
        }
    }

    Err(build_missing_token_error())
}

fn parse_percent(value: Option<&HeaderValue>) -> f64 {
    let raw = value
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(0.0);
    let pct = if raw <= 1.0 { raw * 100.0 } else { raw };
    pct.clamp(0.0, 100.0)
}

fn parse_reset_at(value: Option<&HeaderValue>) -> i64 {
    let raw = match value.and_then(|v| v.to_str().ok()) {
        Some(raw) => raw.trim(),
        None => return 0,
    };

    match raw.parse::<f64>() {
        Ok(value) if value.is_finite() && value > 0.0 => value.floor() as i64,
        _ => 0,
    }
}

fn reset_minutes_from_timestamp(reset_at: i64) -> i64 {
    if reset_at <= 0 {
        return 0;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let minutes = ((reset_at - now) as f64 / 60.0).round() as i64;
    minutes.max(0)
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a HeaderValue> {
    HeaderName::from_bytes(name.as_bytes())
        .ok()
        .and_then(|header| headers.get(header))
}

fn window_from_headers(prefix: &str, headers: &HeaderMap, label: &str) -> UsageWindow {
    let used_percent = parse_percent(header_value(headers, &format!("{prefix}-utilization")));
    let status = header_value(headers, &format!("{prefix}-status"))
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let reset_header = header_value(headers, &format!("{prefix}-reset"));
    let reset_at = parse_reset_at(reset_header);

    UsageWindow {
        label: label.to_string(),
        used_percent,
        remaining_percent: (100.0 - used_percent).clamp(0.0, 100.0),
        reset_minutes: reset_minutes_from_timestamp(reset_at),
        reset_at,
        status,
    }
}

pub async fn collect_claude_usage() -> Result<UsageMonitorResponse, String> {
    let token = read_access_token()?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(|e| e.to_string())?,
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(USER_AGENT_VALUE),
    );
    headers.insert("anthropic-version", HeaderValue::from_static(ANTHROPIC_VERSION));
    headers.insert("anthropic-beta", HeaderValue::from_static(ANTHROPIC_BETA));

    let body = serde_json::json!({
        "model": MODEL,
        "max_tokens": 1,
        "messages": [
            { "role": "user", "content": "hi" }
        ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(API_URL)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(match status.as_u16() {
            401 | 403 => {
                "Token Claude Code không hợp lệ hoặc đã hết hạn. Chạy `claude login` để đăng nhập lại."
                    .to_string()
            }
            code => format!(
                "Không lấy được quota Claude Code (HTTP {code}). Thử lại sau."
            ),
        });
    }

    let headers = response.headers().clone();
    let updated_at: DateTime<Utc> = Utc::now();

    Ok(UsageMonitorResponse {
        enabled: true,
        provider: "claude".to_string(),
        updated_at: Some(updated_at.to_rfc3339()),
        session_5h: Some(window_from_headers("anthropic-ratelimit-unified-5h", &headers, "5 giờ")),
        weekly_7d: Some(window_from_headers("anthropic-ratelimit-unified-7d", &headers, "7 ngày")),
        ok: true,
        error: None,
    })
}

pub async fn get_usage_status(enabled: bool) -> Result<UsageMonitorResponse, String> {
    if !enabled {
        return Ok(UsageMonitorResponse::disabled());
    }

    match collect_claude_usage().await {
        Ok(status) => Ok(status),
        Err(error) => Ok(UsageMonitorResponse {
            enabled: true,
            provider: "claude".to_string(),
            updated_at: None,
            session_5h: None,
            weekly_7d: None,
            ok: false,
            error: Some(error),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::auth::extract_access_token;

    #[test]
    fn token_priority_supports_claude_ai_oauth_shape() {
        let raw = r#"{"claudeAiOauth":{"accessToken":"oauth-token"}}"#;
        assert_eq!(extract_access_token(raw).as_deref(), Some("oauth-token"));
    }
}
