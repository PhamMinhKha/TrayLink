use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};

const REFRESH_ENDPOINT: &str = "https://auth.openai.com/oauth/token";
const USAGE_DEFAULT_BASE: &str = "https://chatgpt.com/backend-api";
const REFRESH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const REQUEST_TIMEOUT_SECONDS: u64 = 30;

#[derive(Debug, Clone, Serialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub reset_minutes: i64,
    /// Unix timestamp (seconds) when the quota window resets; 0 if unknown.
    pub reset_at: i64,
    pub status: String,
    pub limit_window_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditsBalance {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageMonitorResponse {
    pub enabled: bool,
    pub provider: String,
    pub plan: Option<String>,
    pub account_id: Option<String>,
    pub updated_at: Option<String>,
    pub session_5h: Option<UsageWindow>,
    pub weekly_7d: Option<UsageWindow>,
    pub credits: Option<CreditsBalance>,
    pub ok: bool,
    pub error: Option<String>,
}

impl UsageMonitorResponse {
    fn disabled() -> Self {
        Self {
            enabled: false,
            provider: "codex".to_string(),
            plan: None,
            account_id: None,
            updated_at: None,
            session_5h: None,
            weekly_7d: None,
            credits: None,
            ok: true,
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
struct AuthCredentials {
    access_token: String,
    refresh_token: String,
    id_token: Option<String>,
    account_id: Option<String>,
    last_refresh: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
struct AuthBackedIdentity {
    _email: Option<String>,
    _auth_subject: Option<String>,
    plan: Option<String>,
    provider_account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodexUsageResponse {
    plan_type: Option<String>,
    rate_limit: Option<RateLimitDetails>,
    credits: Option<CreditDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RateLimitDetails {
    allowed: Option<bool>,
    limit_reached: Option<bool>,
    primary_window: Option<UsageWindowResponse>,
    secondary_window: Option<UsageWindowResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UsageWindowResponse {
    #[serde(deserialize_with = "deserialize_f64_flexible")]
    used_percent: f64,
    #[serde(deserialize_with = "deserialize_f64_flexible")]
    reset_at: f64,
    #[serde(deserialize_with = "deserialize_i64_flexible")]
    limit_window_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreditDetails {
    has_credits: bool,
    unlimited: bool,
    #[serde(default, deserialize_with = "deserialize_option_f64_flexible")]
    balance: Option<f64>,
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn codex_home_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".codex"))
}

fn auth_path() -> Option<PathBuf> {
    codex_home_dir().map(|home| home.join("auth.json"))
}

fn config_path() -> Option<PathBuf> {
    codex_home_dir().map(|home| home.join("config.toml"))
}

fn string_value(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| v.as_str()).map(ToString::to_string)
}

fn parse_jwt(token: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let mut payload = parts[1].to_string();
    payload.push_str(&"=".repeat((4 - payload.len() % 4) % 4));
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    serde_json::from_slice(&decoded).ok()
}

fn normalize(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn normalize_any(value: Option<String>) -> Option<String> {
    value.and_then(|value| normalize(Some(&value)))
}

fn parse_account_id_from_id_token(id_token: &Option<String>) -> Option<String> {
    let payload = id_token.as_deref().and_then(parse_jwt)?;
    let auth = payload.get("https://api.openai.com/auth").and_then(|value| value.as_object());
    normalize_any(
        auth.and_then(|auth| auth.get("chatgpt_account_id").and_then(|value| value.as_str()).map(ToString::to_string))
            .or_else(|| payload.get("chatgpt_account_id").and_then(|value| value.as_str()).map(ToString::to_string)),
    )
}

fn identity_from_credentials(credentials: &AuthCredentials) -> AuthBackedIdentity {
    let payload = credentials.id_token.as_deref().and_then(parse_jwt);
    let auth = payload
        .as_ref()
        .and_then(|payload| payload.get("https://api.openai.com/auth"))
        .and_then(|value| value.as_object());
    let profile = payload
        .as_ref()
        .and_then(|payload| payload.get("https://api.openai.com/profile"))
        .and_then(|value| value.as_object());

    let email = normalize(
        payload
            .as_ref()
            .and_then(|payload| payload.get("email").and_then(|value| value.as_str()))
            .or_else(|| profile.and_then(|value| value.get("email").and_then(|value| value.as_str()))),
    );
    let auth_subject = normalize(
        payload
            .as_ref()
            .and_then(|payload| payload.get("sub").and_then(|value| value.as_str())),
    );
    let plan = normalize(
        auth.and_then(|auth| auth.get("chatgpt_plan_type").and_then(|value| value.as_str()))
            .or_else(|| payload.as_ref().and_then(|payload| payload.get("chatgpt_plan_type").and_then(|value| value.as_str()))),
    );
    let provider_account_id = normalize(
        credentials
            .account_id
            .as_deref()
            .or_else(|| auth.and_then(|auth| auth.get("chatgpt_account_id").and_then(|value| value.as_str())))
            .or_else(|| payload.as_ref().and_then(|payload| payload.get("chatgpt_account_id").and_then(|value| value.as_str()))),
    );

    AuthBackedIdentity {
        _email: email,
        _auth_subject: auth_subject,
        plan,
        provider_account_id,
    }
}

fn read_credentials() -> Result<AuthCredentials, String> {
    let auth_path = auth_path().ok_or_else(|| "Không xác định được Codex home.".to_string())?;
    let raw = std::fs::read_to_string(&auth_path).map_err(|e| e.to_string())?;
    let payload: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    if let Some(api_key) = payload.get("OPENAI_API_KEY").and_then(|v| v.as_str()) {
        let api_key = api_key.trim();
        if !api_key.is_empty() {
            return Ok(AuthCredentials {
                access_token: api_key.to_string(),
                refresh_token: String::new(),
                id_token: None,
                account_id: None,
                last_refresh: None,
            });
        }
    }

    let tokens = payload
        .get("tokens")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "The required token fields are missing from `auth.json`.".to_string())?;

    let access_token = string_value(&serde_json::Value::Object(tokens.clone()), "access_token")
        .ok_or_else(|| "The required token fields are missing from `auth.json`.".to_string())?;
    let refresh_token = string_value(&serde_json::Value::Object(tokens.clone()), "refresh_token").unwrap_or_default();
    let id_token = string_value(&serde_json::Value::Object(tokens.clone()), "id_token");
    let account_id = string_value(&serde_json::Value::Object(tokens.clone()), "account_id")
        .or_else(|| parse_account_id_from_id_token(&id_token));

    let last_refresh = payload
        .get("last_refresh")
        .and_then(|value| value.as_str())
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc));

    Ok(AuthCredentials {
        access_token,
        refresh_token,
        id_token,
        account_id,
        last_refresh,
    })
}

fn save_credentials(credentials: &AuthCredentials) -> Result<(), String> {
    let auth_path = auth_path().ok_or_else(|| "Không xác định được Codex home.".to_string())?;
    let mut payload = if let Ok(raw) = std::fs::read_to_string(&auth_path) {
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut tokens = serde_json::Map::new();
    tokens.insert(
        "access_token".to_string(),
        serde_json::Value::String(credentials.access_token.clone()),
    );
    tokens.insert(
        "refresh_token".to_string(),
        serde_json::Value::String(credentials.refresh_token.clone()),
    );
    if let Some(id_token) = &credentials.id_token {
        tokens.insert("id_token".to_string(), serde_json::Value::String(id_token.clone()));
    }
    if let Some(account_id) = &credentials.account_id {
        tokens.insert(
            "account_id".to_string(),
            serde_json::Value::String(account_id.clone()),
        );
    }

    if let Some(map) = payload.as_object_mut() {
        map.insert("tokens".to_string(), serde_json::Value::Object(tokens));
        map.insert(
            "last_refresh".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        );
    }

    let data = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
    std::fs::write(auth_path, data).map_err(|e| e.to_string())
}

fn refresh_due(credentials: &AuthCredentials) -> bool {
    match credentials.last_refresh {
        Some(last_refresh) => Utc::now().signed_duration_since(last_refresh).num_seconds() > 8 * 24 * 60 * 60,
        None => true,
    }
}

async fn refresh(credentials: &AuthCredentials) -> Result<AuthCredentials, String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "client_id": REFRESH_CLIENT_ID,
        "grant_type": "refresh_token",
        "refresh_token": credentials.refresh_token,
        "scope": "openid profile email",
    });

    let response = client
        .post(REFRESH_ENDPOINT)
        .header(CONTENT_TYPE, "application/json")
        .header("Cache-Control", "no-cache, no-store, max-age=0")
        .header("Pragma", "no-cache")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    if status.as_u16() == 401 {
        let code = extract_error_code(&text).to_lowercase();
        let message = match code.as_str() {
            "refresh_token_reused" => "The refresh token can no longer be reused. Sign in again for this account.",
            "refresh_token_invalidated" => "The refresh token was revoked. Sign in again for this account.",
            _ => "The refresh token has expired. Sign in again for this account.",
        };
        return Err(message.to_string());
    }

    if !status.is_success() {
        return Err("The Codex API response was not in the expected format.".to_string());
    }

    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let access_token = json
        .get("access_token")
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .unwrap_or_else(|| credentials.access_token.clone());
    let refresh_token = json
        .get("refresh_token")
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .unwrap_or_else(|| credentials.refresh_token.clone());
    let id_token = json
        .get("id_token")
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .or_else(|| credentials.id_token.clone());
    let account_id = credentials
        .account_id
        .clone()
        .or_else(|| parse_account_id_from_id_token(&id_token));

    Ok(AuthCredentials {
        access_token,
        refresh_token,
        id_token,
        account_id,
        last_refresh: Some(Utc::now()),
    })
}

fn resolve_usage_url() -> String {
    let configured_base = config_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|contents| parse_chatgpt_base_url(&contents));

    let mut base = configured_base.unwrap_or_else(|| USAGE_DEFAULT_BASE.to_string());
    while base.ends_with('/') {
        base.pop();
    }

    if base.starts_with("https://chatgpt.com") && !base.contains("/backend-api") {
        base.push_str("/backend-api");
    }
    if base.starts_with("https://chat.openai.com") && !base.contains("/backend-api") {
        base.push_str("/backend-api");
    }

    let path = if base.contains("/backend-api") {
        "/wham/usage"
    } else {
        "/api/codex/usage"
    };
    format!("{base}{path}")
}

fn parse_chatgpt_base_url(contents: &str) -> Option<String> {
    for raw_line in contents.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() != "chatgpt_base_url" {
            continue;
        }
        let mut value = right.trim().to_string();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value = value[1..value.len() - 1].to_string();
        }
        if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
            value = value[1..value.len() - 1].to_string();
        }
        return Some(value);
    }
    None
}

fn extract_error_code(payload: &str) -> String {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(payload) else {
        return String::new();
    };
    let Some(map) = parsed.as_object() else {
        return String::new();
    };
    if let Some(error) = map.get("error").and_then(|value| value.as_object()) {
        return error
            .get("code")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
    }
    if let Some(error) = map.get("error").and_then(|value| value.as_str()) {
        return error.to_string();
    }
    map.get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string()
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

fn make_window(payload: &UsageWindowResponse) -> UsageWindow {
    let reset_at = if payload.reset_at.is_finite() && payload.reset_at > 0.0 {
        payload.reset_at.floor() as i64
    } else {
        0
    };

    UsageWindow {
        label: String::new(),
        used_percent: payload.used_percent.clamp(0.0, 100.0),
        remaining_percent: (100.0 - payload.used_percent).clamp(0.0, 100.0),
        reset_minutes: reset_minutes_from_timestamp(reset_at),
        reset_at,
        status: "unknown".to_string(),
        limit_window_seconds: payload.limit_window_seconds,
    }
}

fn credits_from(payload: &CreditDetails) -> CreditsBalance {
    CreditsBalance {
        has_credits: payload.has_credits,
        unlimited: payload.unlimited,
        balance: payload.balance,
    }
}

fn parse_usage_response_body(text: &str) -> Result<CodexUsageResponse, String> {
    let parsed: serde_json::Value = serde_json::from_str(text).map_err(|err| {
        format!(
            "Codex usage API returned a non-JSON response: {err}"
        )
    })?;

    let mut candidates = Vec::new();
    candidates.push(parsed.clone());
    if let Some(value) = parsed.get("data") {
        candidates.push(value.clone());
    }
    if let Some(value) = parsed.get("result") {
        candidates.push(value.clone());
    }
    if let Some(value) = parsed.get("response") {
        candidates.push(value.clone());
    }

    for candidate in candidates {
        if let Ok(response) = serde_json::from_value::<CodexUsageResponse>(candidate.clone()) {
            return Ok(response);
        }

        if let Some(obj) = candidate.as_object() {
            let keys = ["plan_type", "rate_limit", "credits"];
            if keys.iter().any(|key| obj.contains_key(*key)) {
                return serde_json::from_value::<CodexUsageResponse>(serde_json::Value::Object(
                    obj.clone(),
                ))
                .map_err(|err| format!("Codex usage API response had an unexpected shape: {err}"));
            }
        }
    }

    Err("Codex usage API response did not include quota fields.".to_string())
}

fn deserialize_f64_flexible<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;

    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| D::Error::custom("expected a finite f64")),
        serde_json::Value::String(value) => value
            .trim()
            .parse::<f64>()
            .map_err(|err| D::Error::custom(err.to_string())),
        serde_json::Value::Null => Err(D::Error::custom("expected f64, found null")),
        other => Err(D::Error::custom(format!("expected f64 or string, found {other}"))),
    }
}

fn deserialize_i64_flexible<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;

    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| D::Error::custom("expected an integer")),
        serde_json::Value::String(value) => value
            .trim()
            .parse::<i64>()
            .map_err(|err| D::Error::custom(err.to_string())),
        serde_json::Value::Null => Err(D::Error::custom("expected i64, found null")),
        other => Err(D::Error::custom(format!("expected i64 or string, found {other}"))),
    }
}

fn deserialize_option_f64_flexible<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(number)) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("expected a finite f64")),
        Some(serde_json::Value::String(value)) => value
            .trim()
            .parse::<f64>()
            .map(Some)
            .map_err(|err| serde::de::Error::custom(err.to_string())),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected option f64 or string, found {other}"
        ))),
    }
}

fn role_for_window(window: &UsageWindow) -> &'static str {
    match window.limit_window_seconds {
        18_000 => "session",
        604_800 => "weekly",
        _ => "unknown",
    }
}

fn normalize_windows(
    primary: Option<UsageWindow>,
    secondary: Option<UsageWindow>,
) -> (Option<UsageWindow>, Option<UsageWindow>) {
    match (primary, secondary) {
        (Some(primary), Some(secondary)) => {
            let primary_role = role_for_window(&primary);
            let secondary_role = role_for_window(&secondary);

            if matches!((primary_role, secondary_role), ("weekly", "session") | ("weekly", "unknown")) {
                (Some(secondary), Some(primary))
            } else {
                (Some(primary), Some(secondary))
            }
        }
        (Some(window), None) => {
            if role_for_window(&window) == "weekly" {
                (None, Some(window))
            } else {
                (Some(window), None)
            }
        }
        (None, Some(window)) => {
            if role_for_window(&window) == "weekly" {
                (None, Some(window))
            } else {
                (Some(window), None)
            }
        }
        (None, None) => (None, None),
    }
}

async fn fetch_usage(access_token: &str, account_id: Option<&str>) -> Result<CodexUsageResponse, String> {
    let url = resolve_usage_url();
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {access_token}")).map_err(|e| e.to_string())?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("codex-cli"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache, no-store, max-age=0"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));

    if let Some(account_id) = account_id.filter(|value| !value.is_empty()) {
        headers.insert(
            HeaderName::from_static("chatgpt-account-id"),
            HeaderValue::from_str(account_id).map_err(|e| e.to_string())?,
        );
    }

    let response = client
        .get(url)
        .headers(headers)
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    match response.status().as_u16() {
        200..=299 => {
            let text = response.text().await.map_err(|e| e.to_string())?;
            parse_usage_response_body(&text)
        }
        401 | 403 => Err(
            "Token Codex không hợp lệ hoặc đã hết hạn. Đăng nhập lại Codex rồi thử lại.".to_string(),
        ),
        code => {
            let text = response.text().await.unwrap_or_default();
            if text.is_empty() {
                Err(format!("Codex API error {code}."))
            } else {
                Err(format!("Codex API error {code}: {}", text.chars().take(160).collect::<String>()))
            }
        }
    }
}

fn window_from_response(
    value: &UsageWindowResponse,
    label: &str,
    status: &str,
) -> UsageWindow {
    let mut window = make_window(value);
    window.label = label.to_string();
    window.status = status.to_string();
    window
}

fn response_from_codex(
    response: &CodexUsageResponse,
    identity: &AuthBackedIdentity,
) -> UsageMonitorResponse {
    let rate_limit = response.rate_limit.as_ref();
    let (primary, secondary) = if let Some(rate_limit) = rate_limit {
        let primary = rate_limit.primary_window.as_ref().map(|value| window_from_response(value, "5 giờ", "active"));
        let secondary = rate_limit.secondary_window.as_ref().map(|value| window_from_response(value, "7 ngày", "active"));
        normalize_windows(primary, secondary)
    } else {
        (None, None)
    };

    UsageMonitorResponse {
        enabled: true,
        provider: "codex".to_string(),
        plan: response.plan_type.clone().or_else(|| identity.plan.clone()),
        account_id: identity.provider_account_id.clone(),
        updated_at: Some(Utc::now().to_rfc3339()),
        session_5h: primary,
        weekly_7d: secondary,
        credits: response.credits.as_ref().map(credits_from),
        ok: true,
        error: None,
    }
}

pub async fn get_usage_status(enabled: bool) -> UsageMonitorResponse {
    if !enabled {
        return UsageMonitorResponse::disabled();
    }

    let mut credentials = match read_credentials() {
        Ok(credentials) => credentials,
        Err(error) => {
            return UsageMonitorResponse {
                enabled: true,
                provider: "codex".to_string(),
                plan: None,
                account_id: None,
                updated_at: None,
                session_5h: None,
                weekly_7d: None,
                credits: None,
                ok: false,
                error: Some(error),
            };
        }
    };
    let identity = identity_from_credentials(&credentials);

    if refresh_due(&credentials) && !credentials.refresh_token.is_empty() {
        match refresh(&credentials).await {
            Ok(refreshed) => {
                credentials = refreshed;
                if let Err(error) = save_credentials(&credentials) {
                    return UsageMonitorResponse {
                        enabled: true,
                        provider: "codex".to_string(),
                        plan: identity.plan,
                        account_id: identity.provider_account_id,
                        updated_at: None,
                        session_5h: None,
                        weekly_7d: None,
                        credits: None,
                        ok: false,
                        error: Some(error),
                    };
                }
            }
            Err(error) => {
                return UsageMonitorResponse {
                    enabled: true,
                    provider: "codex".to_string(),
                    plan: identity.plan,
                    account_id: identity.provider_account_id,
                    updated_at: None,
                    session_5h: None,
                    weekly_7d: None,
                    credits: None,
                    ok: false,
                    error: Some(error),
                };
            }
        }
    }

    let result = match fetch_usage(&credentials.access_token, credentials.account_id.as_deref()).await {
        Ok(response) => response_from_codex(&response, &identity),
        Err(err)
            if err == "The Codex usage API request returned unauthorized."
                && !credentials.refresh_token.is_empty() =>
        {
            match refresh(&credentials).await {
                Ok(refreshed) => {
                    credentials = refreshed;
                    if let Err(error) = save_credentials(&credentials) {
                        return UsageMonitorResponse {
                            enabled: true,
                            provider: "codex".to_string(),
                            plan: identity.plan,
                            account_id: identity.provider_account_id,
                            updated_at: None,
                            session_5h: None,
                            weekly_7d: None,
                            credits: None,
                            ok: false,
                            error: Some(error),
                        };
                    }
                    match fetch_usage(&credentials.access_token, credentials.account_id.as_deref()).await {
                        Ok(response) => response_from_codex(&response, &identity),
                        Err(err) => UsageMonitorResponse {
                            enabled: true,
                            provider: "codex".to_string(),
                            plan: identity.plan,
                            account_id: identity.provider_account_id,
                            updated_at: None,
                            session_5h: None,
                            weekly_7d: None,
                            credits: None,
                            ok: false,
                            error: Some(err),
                        },
                    }
                }
                Err(error) => UsageMonitorResponse {
                    enabled: true,
                    provider: "codex".to_string(),
                    plan: identity.plan,
                    account_id: identity.provider_account_id,
                    updated_at: None,
                    session_5h: None,
                    weekly_7d: None,
                    credits: None,
                    ok: false,
                    error: Some(error),
                },
            }
        }
        Err(err) => UsageMonitorResponse {
            enabled: true,
            provider: "codex".to_string(),
            plan: identity.plan,
            account_id: identity.provider_account_id,
            updated_at: None,
            session_5h: None,
            weekly_7d: None,
            credits: None,
            ok: false,
            error: Some(err),
        },
    };

    result
}
