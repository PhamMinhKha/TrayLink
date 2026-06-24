use std::time::Instant;

use axum::extract::{DefaultBodyLimit, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::api::auth::extract_token;
use crate::api::client_ip::ClientIp;
use crate::api::remote;
use crate::api::server::{https_port, restart_server};
use crate::api::upload;
use crate::uploads::MAX_UPLOAD_BYTES;
use crate::launcher::{exec_cmd, open_app, open_file, LauncherError};
#[cfg(not(target_os = "macos"))]
use crate::launcher::hotkey;
use crate::net;
use crate::state::{LogEntry, SharedState, APP_VERSION};
use crate::codex_usage;
use crate::usage_monitor;

#[derive(Serialize)]
pub struct StatusResponse {
    online: bool,
    version: &'static str,
    port: u16,
    https_port: u16,
    lan_ip: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAppRequest {
    app: String,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenAppQuery {
    app: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenFileRequest {
    path: String,
}

#[derive(Deserialize)]
pub struct OpenFileQuery {
    path: String,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct ExecRequest {
    cmd: String,
}

#[derive(Deserialize)]
pub struct ExecQuery {
    cmd: String,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct SendHotkeyQuery {
    app: String,
    hotkey: String,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct SendHotkeyRequest {
    app: String,
    hotkey: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    ok: bool,
    message: String,
}

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .merge(remote::routes())
        .merge(upload::routes())
        .route("/status", get(status))
        .route("/open-app", get(open_app_get).post(open_app_post))
    .route("/open-file", get(open_file_get).post(open_file_post))
    .route("/exec", get(exec_get).post(exec_post))
    .route("/send-hotkey", get(send_hotkey_get).post(send_hotkey_post))
        .route("/claude-usage", get(claude_usage_get).post(claude_usage_post))
        .route("/codex-usage", get(codex_usage_get).post(codex_usage_post))
        .route(
            "/system-metrics",
            get(system_metrics_get).post(system_metrics_post),
        )
    .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES as usize))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}

async fn status(State(state): State<SharedState>) -> Json<StatusResponse> {
    let port = state.config.read().unwrap().port;
    log_request(&state, "GET", "/status", 200, 0, "127.0.0.1");
    Json(StatusResponse {
        online: true,
        version: APP_VERSION,
        port,
        https_port: https_port(port),
        lan_ip: net::get_lan_ip(),
    })
}

fn reject_get_disabled() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        Json(serde_json::json!({
            "error": "GET API is disabled. Enable it in Dashboard -> Settings, or use POST."
        })),
    )
        .into_response()
}

async fn open_app_post(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<OpenAppRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/open-app",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_open_app(
        &state,
        "POST",
        started,
        &ip,
        &body.app,
        body.url.as_deref(),
    )
}

async fn open_app_get(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(query): Query<OpenAppQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/open-app",
            405,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/open-app",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_open_app(
        &state,
        "GET",
        started,
        &ip,
        &query.app,
        query.url.as_deref(),
    )
}

fn handle_open_app(
    state: &SharedState,
    method: &str,
    started: Instant,
    client_ip: &str,
    app: &str,
    url: Option<&str>,
) -> Response {
    let apps = state.config.read().unwrap().apps.clone();
    match open_app::open_app(app, &apps, url) {
        Ok(()) => {
            log_request(
                state,
                method,
                "/open-app",
                200,
                started.elapsed().as_millis() as u64,
                client_ip,
            );
            let detail = url
                .filter(|value| !value.is_empty())
                .map(|value| format!(" with url '{value}'"))
                .unwrap_or_default();
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("opened app '{app}{detail}'"),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(state, method, "/open-app", started, client_ip, err),
    }
}

async fn open_file_post(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<OpenFileRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/open-file",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_open_file(&state, "POST", started, &ip, &body.path)
}

async fn open_file_get(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(query): Query<OpenFileQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/open-file",
            405,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/open-file",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_open_file(&state, "GET", started, &ip, &query.path)
}

fn handle_open_file(
    state: &SharedState,
    method: &str,
    started: Instant,
    client_ip: &str,
    path: &str,
) -> Response {
    match open_file::open_file(path) {
        Ok(()) => {
            log_request(
                state,
                method,
                "/open-file",
                200,
                started.elapsed().as_millis() as u64,
                client_ip,
            );
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("opened file '{path}'"),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(state, method, "/open-file", started, client_ip, err),
    }
}

async fn exec_post(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<ExecRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/exec",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_exec(&state, "POST", started, &ip, &body.cmd)
}

async fn exec_get(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(query): Query<ExecQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/exec",
            405,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/exec",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_exec(&state, "GET", started, &ip, &query.cmd)
}

async fn send_hotkey_post(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<SendHotkeyRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/send-hotkey",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_send_hotkey(&state, "POST", started, &ip, &body.app, &body.hotkey)
}

async fn send_hotkey_get(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(query): Query<SendHotkeyQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/send-hotkey",
            405,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/send-hotkey",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_send_hotkey(&state, "GET", started, &ip, &query.app, &query.hotkey)
}

#[derive(Deserialize)]
struct ClaudeUsageQuery {
    #[serde(default)]
    token: Option<String>,
}

async fn claude_usage_post(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/claude-usage",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_claude_usage(&state, "POST", started, &ip).await
}

async fn codex_usage_post(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/codex-usage",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_codex_usage(&state, "POST", started, &ip).await
}

async fn claude_usage_get(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(query): Query<ClaudeUsageQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/claude-usage",
            405,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/claude-usage",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_claude_usage(&state, "GET", started, &ip).await
}

#[derive(Deserialize)]
struct CodexUsageQuery {
    #[serde(default)]
    token: Option<String>,
}

async fn codex_usage_get(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(query): Query<CodexUsageQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/codex-usage",
            405,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/codex-usage",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_codex_usage(&state, "GET", started, &ip).await
}

async fn handle_claude_usage(
    state: &SharedState,
    method: &str,
    started: Instant,
    client_ip: &str,
) -> Response {
    let enabled = state.config.read().unwrap().usage_monitor_enabled;
    let result = usage_monitor::get_usage_status(enabled).await;

    match result {
        Ok(payload) => {
            log_request(
                state,
                method,
                "/claude-usage",
                200,
                started.elapsed().as_millis() as u64,
                client_ip,
            );
            (StatusCode::OK, Json(payload)).into_response()
        }
        Err(err) => {
            log_request(
                state,
                method,
                "/claude-usage",
                500,
                started.elapsed().as_millis() as u64,
                client_ip,
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err })),
            )
                .into_response()
        }
    }
}

async fn handle_codex_usage(
    state: &SharedState,
    method: &str,
    started: Instant,
    client_ip: &str,
) -> Response {
    let enabled = state.config.read().unwrap().codex_usage_monitor_enabled;
    let payload = codex_usage::get_usage_status(enabled).await;
    log_request(
        state,
        method,
        "/codex-usage",
        200,
        started.elapsed().as_millis() as u64,
        client_ip,
    );
    (StatusCode::OK, Json(payload)).into_response()
}

#[derive(Deserialize)]
struct SystemMetricsQuery {
    #[serde(default)]
    token: Option<String>,
}

async fn system_metrics_post(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/system-metrics",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_system_metrics(&state, "POST", started, &ip)
}

async fn system_metrics_get(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(query): Query<SystemMetricsQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/system-metrics",
            405,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/system-metrics",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    handle_system_metrics(&state, "GET", started, &ip)
}

fn handle_system_metrics(
    state: &SharedState,
    method: &str,
    started: Instant,
    client_ip: &str,
) -> Response {
    let prefs = state.config.read().unwrap().system_metrics.clone();
    let payload = crate::system_metrics::get_status(&prefs);
    log_request(
        state,
        method,
        "/system-metrics",
        200,
        started.elapsed().as_millis() as u64,
        client_ip,
    );
    (StatusCode::OK, Json(payload)).into_response()
}

fn handle_send_hotkey(
    state: &SharedState,
    method: &str,
    started: Instant,
    client_ip: &str,
    app_key: &str,
    hotkey_id: &str,
) -> Response {
    let config = state.config.read().unwrap();
    let apps = config.apps.clone();
    let entry = match apps.get(app_key) {
        Some(entry) => entry.clone(),
        None => {
            return error_response(
                state,
                method,
                "/send-hotkey",
                started,
                client_ip,
                LauncherError::AppNotAllowed(app_key.to_string()),
            );
        }
    };

    let binding = match entry.hotkeys.iter().find(|item| item.id == hotkey_id) {
        Some(binding) => binding.clone(),
        None => {
            return error_response(
                state,
                method,
                "/send-hotkey",
                started,
                client_ip,
                LauncherError::LaunchFailed(format!(
                    "hotkey '{hotkey_id}' not found for app '{app_key}'"
                )),
            );
        }
    };
    drop(config);

    let result = {
        #[cfg(target_os = "macos")]
        {
            crate::macos::executor::execute_binding(
                &state.app_handle,
                app_key,
                &binding,
                &apps,
            )
        }
        #[cfg(not(target_os = "macos"))]
        {
            hotkey::execute_binding(app_key, &binding, &apps)
        }
    };

    match result {
        Ok(()) => {
            log_request(
                state,
                method,
                "/send-hotkey",
                200,
                started.elapsed().as_millis() as u64,
                client_ip,
            );
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("executed hotkey '{}' for app '{app_key}'", binding.name),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(state, method, "/send-hotkey", started, client_ip, err),
    }
}

fn handle_exec(
    state: &SharedState,
    method: &str,
    started: Instant,
    client_ip: &str,
    cmd: &str,
) -> Response {
    let commands = state.config.read().unwrap().commands.clone();
    match exec_cmd::exec_command(cmd, &commands) {
        Ok(is_internal) => {
            if is_internal && cmd == "restart_server" {
                let app = state.app_handle.clone();
                let state_clone = state.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = restart_server(app, state_clone).await;
                });
            }

            log_request(
                state,
                method,
                "/exec",
                200,
                started.elapsed().as_millis() as u64,
                client_ip,
            );
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("executed command '{cmd}'"),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(state, method, "/exec", started, client_ip, err),
    }
}

fn error_response(
    state: &SharedState,
    method: &str,
    path: &str,
    started: Instant,
    client_ip: &str,
    err: LauncherError,
) -> Response {
    let status = match &err {
        LauncherError::AppNotAllowed(_) | LauncherError::CommandNotAllowed(_) => StatusCode::FORBIDDEN,
        LauncherError::PathNotAllowed(_) => StatusCode::FORBIDDEN,
        LauncherError::PathNotFound(_) => StatusCode::NOT_FOUND,
        LauncherError::LaunchFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    log_request(
        state,
        method,
        path,
        status.as_u16(),
        started.elapsed().as_millis() as u64,
        client_ip,
    );

    (
        status,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
        .into_response()
}

pub(crate) fn log_request(
    state: &SharedState,
    method: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
    client_ip: &str,
) {
    state.push_log(LogEntry {
        timestamp: Utc::now(),
        method: method.to_string(),
        path: path.to_string(),
        status,
        duration_ms,
        client_ip: client_ip.to_string(),
    });
}
