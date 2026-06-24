use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Serialize;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const CREDENTIALS_PATH: &str = ".claude/.credentials.json";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const ANTHROPIC_BETA: &str = "oauth-2025-04-20";
const USER_AGENT_VALUE: &str = "claude-code/2.1.5";
const MODEL: &str = "claude-haiku-4-5-20251001";
const KEYCHAIN_SERVICES: &[&str] = &["Claude Code-credentials", "Claude Code", "Claude"];
const CLAUDE_APP_PATH: &str = "/Applications/Claude.app";
const CLAUDE_APP_EXECUTABLE: &str = "/Applications/Claude.app/Contents/MacOS/Claude";

#[derive(Debug, Clone, Serialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub reset_minutes: i64,
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

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn credentials_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(CREDENTIALS_PATH))
}

fn extract_access_token(blob: &str) -> Option<String> {
    let blob = blob.trim();
    if blob.is_empty() {
        return None;
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(blob) {
        if let Some(token) = value.get("accessToken").and_then(|v| v.as_str()) {
            return Some(token.to_string());
        }

        if let Some(token) = value.get("access_token").and_then(|v| v.as_str()) {
            return Some(token.to_string());
        }

        if let Some(obj) = value.as_object() {
            for nested in obj.values() {
                if let Some(token) = nested.get("accessToken").and_then(|v| v.as_str()) {
                    return Some(token.to_string());
                }
                if let Some(token) = nested.get("access_token").and_then(|v| v.as_str()) {
                    return Some(token.to_string());
                }
            }
        }
    }

    let needle = "\"accessToken\":\"";
    if let Some(start) = blob.find(needle) {
        let rest = &blob[start + needle.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }

    if !blob.starts_with('{')
        && !blob.starts_with('[')
        && blob.len() > 20
        && blob
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '~' | '+' | '/' | '='))
    {
        return Some(blob.to_string());
    }

    None
}

fn read_token_from_keychain() -> Result<Option<String>, String> {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map_err(|e| e.to_string())?;

    for service in KEYCHAIN_SERVICES {
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                service,
                "-a",
                &user,
                "-w",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout);
            if let Some(token) = extract_access_token(&raw) {
                return Ok(Some(token));
            }
        }
    }

    Ok(None)
}

fn claude_cli_on_path() -> bool {
    Command::new("sh")
        .args(["-lc", "command -v claude >/dev/null 2>&1"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn build_claude_diagnostics() -> ClaudeDiagnosticsResponse {
    let claude_app_installed = std::path::Path::new(CLAUDE_APP_PATH).exists();
    let claude_app_executable = std::path::Path::new(CLAUDE_APP_EXECUTABLE).exists();
    let keychain_token_found = read_token_from_keychain().ok().flatten().is_some();
    let credentials_path = credentials_path().map(|path| path.display().to_string());
    let credentials_file_exists = credentials_path
        .as_deref()
        .map(std::path::Path::new)
        .is_some_and(|path| path.exists());
    let credentials_token_found = read_token_from_file().ok().flatten().is_some();
    let claude_cli_on_path = claude_cli_on_path();

    let mut findings = Vec::new();
    findings.push(ClaudeDiagnosticItem {
        label: "Claude.app".to_string(),
        status: if claude_app_installed {
            "detected".to_string()
        } else {
            "not found".to_string()
        },
        detail: if claude_app_installed {
            "Bạn đang dùng Claude Desktop app trên macOS.".to_string()
        } else {
            "Không thấy /Applications/Claude.app.".to_string()
        },
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
            "Không thấy `claude` CLI trong PATH.".to_string()
        },
    });
    findings.push(ClaudeDiagnosticItem {
        label: "Keychain".to_string(),
        status: if keychain_token_found {
            "token found".to_string()
        } else {
            "no token".to_string()
        },
        detail: format!(
            "Đã kiểm tra các service: {}.",
            KEYCHAIN_SERVICES.join(", ")
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

    let summary = if keychain_token_found || credentials_token_found {
        "Đã tìm thấy nguồn auth Claude Code phù hợp.".to_string()
    } else if claude_app_installed {
        "Phát hiện Claude Desktop, nhưng không tìm thấy token Claude Code CLI.".to_string()
    } else {
        "Không tìm thấy nguồn auth Claude Code nào trên máy này.".to_string()
    };

    let recommendation = if claude_app_installed {
        "TrayLink hiện đọc token Claude Code CLI để lấy usage. Nếu bạn chỉ dùng Claude Desktop app thì monitor này không có token để đọc. Hãy cài và đăng nhập Claude Code CLI, hoặc nếu muốn mình có thể giúp đổi panel này thành chỉ báo rằng Claude Desktop không được hỗ trợ.".to_string()
    } else if claude_cli_on_path {
        "Mở Claude Code CLI, chạy /login nếu cần, rồi bật lại theo dõi usage.".to_string()
    } else {
        "Cài Claude Code CLI và đăng nhập trên máy này, rồi mở lại TrayLink.".to_string()
    };

    ClaudeDiagnosticsResponse {
        claude_app_installed,
        claude_app_executable,
        claude_cli_on_path,
        keychain_services_checked: KEYCHAIN_SERVICES.iter().map(|s| s.to_string()).collect(),
        keychain_token_found,
        credentials_path,
        credentials_file_exists,
        credentials_token_found,
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

fn read_token_from_file() -> Result<Option<String>, String> {
    let path = match credentials_path() {
        Some(path) => path,
        None => return Ok(None),
    };

    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.to_string()),
    };

    Ok(extract_access_token(&raw))
}

pub fn read_access_token() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(token) = read_token_from_keychain()? {
            return Ok(token);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(token) = read_token_from_file()? {
            return Ok(token);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(token) = read_token_from_file()? {
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

fn parse_reset_minutes(value: Option<&HeaderValue>) -> i64 {
    let raw = match value.and_then(|v| v.to_str().ok()) {
        Some(raw) => raw.trim(),
        None => return 0,
    };

    let reset_ts = match raw.parse::<f64>() {
        Ok(value) => value,
        Err(_) => return 0,
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    let minutes = (reset_ts - now) / 60.0;
    if minutes <= 0.0 {
        0
    } else {
        minutes.round() as i64
    }
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

    UsageWindow {
        label: label.to_string(),
        used_percent,
        remaining_percent: (100.0 - used_percent).clamp(0.0, 100.0),
        reset_minutes: parse_reset_minutes(header_value(headers, &format!("{prefix}-reset"))),
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
