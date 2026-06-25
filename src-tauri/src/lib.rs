mod api;
mod apps;
mod config;
mod launcher;
mod monitor_status;
mod net;
#[cfg(target_os = "macos")]
mod macos;
mod remote_icons;
#[cfg(desktop)]
mod shortcuts;
mod tls;
mod usage_monitor;
mod codex_usage;
mod state;
mod system_metrics;
mod tray;
mod uploads;

use std::sync::Arc;

use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

use crate::api::server::{restart_server, start_server};
use crate::config::store::{load_config, save_config};
use crate::state::AppState;

#[tauri::command]
fn show_dashboard(app: tauri::AppHandle, tab: Option<String>) {
    tray::show_main_window(&app);
    if let Some(tab) = tab.filter(|value| !value.is_empty()) {
        let _ = app.emit("navigate-tab", tab);
    }
}

#[tauri::command]
fn get_config(state: tauri::State<'_, Arc<AppState>>) -> config::AppConfig {
    state.config.read().unwrap().clone()
}

#[tauri::command]
fn update_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    config: config::AppConfig,
) -> Result<(), String> {
    {
        let mut cfg = state.config.write().unwrap();
        *cfg = config.clone();
    }
    save_config(&app, &config)?;
    #[cfg(desktop)]
    shortcuts::sync_hotkeys(&app, &config);
    Ok(())
}

#[tauri::command]
async fn restart_api_server(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    restart_server(app, state.inner().clone()).await
}

#[tauri::command]
fn get_request_logs(state: tauri::State<'_, Arc<AppState>>) -> Vec<state::LogEntry> {
    state.logs.read().unwrap().clone()
}

#[tauri::command]
fn regenerate_token(app: tauri::AppHandle, state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let token = uuid::Uuid::new_v4().to_string();
    {
        let mut cfg = state.config.write().unwrap();
        cfg.token = token.clone();
        save_config(&app, &cfg)?;
    }
    Ok(token)
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, state: tauri::State<'_, Arc<AppState>>, enabled: bool) -> Result<(), String> {
    sync_autostart(&app, enabled)?;

    {
        let mut cfg = state.config.write().unwrap();
        cfg.autostart = enabled;
        save_config(&app, &cfg)?;
    }

    Ok(())
}

fn sync_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_server_uptime(state: tauri::State<'_, Arc<AppState>>) -> i64 {
    state.uptime_seconds()
}

#[tauri::command]
async fn get_claude_usage_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<usage_monitor::UsageMonitorResponse, String> {
    let enabled = state.config.read().unwrap().usage_monitor_enabled;
    usage_monitor::get_usage_status(enabled).await
}

#[tauri::command]
fn get_claude_diagnostics() -> usage_monitor::ClaudeDiagnosticsResponse {
    usage_monitor::diagnose_claude()
}

#[tauri::command]
async fn get_codex_usage_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<codex_usage::UsageMonitorResponse, String> {
    let enabled = {
        let cfg = state.config.read().unwrap();
        cfg.codex_usage_monitor_enabled
    };
    Ok(codex_usage::get_usage_status(enabled).await)
}

#[tauri::command]
fn get_system_metrics_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> system_metrics::SystemMetricsResponse {
    let prefs = state.config.read().unwrap().system_metrics.clone();
    system_metrics::get_status(&prefs)
}

#[tauri::command]
async fn get_monitor_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<monitor_status::MonitorStatusResponse, String> {
    let (claude_enabled, codex_enabled, system_prefs) = {
        let cfg = state.config.read().unwrap();
        (
            cfg.usage_monitor_enabled,
            cfg.codex_usage_monitor_enabled,
            cfg.system_metrics.clone(),
        )
    };
    Ok(
        monitor_status::get_all_status(claude_enabled, codex_enabled, &system_prefs).await,
    )
}

#[tauri::command]
fn test_open_app(state: tauri::State<'_, Arc<AppState>>, app_key: String) -> Result<String, String> {
    let apps = state.config.read().unwrap().apps.clone();
    launcher::open_app::open_app(&app_key, &apps, None).map_err(|err| err.to_string())?;
    Ok(format!("Đã mở app '{app_key}'"))
}

#[tauri::command]
fn list_browser_profiles(path: String) -> Vec<launcher::browser_profiles::BrowserProfile> {
    launcher::browser_profiles::list_profiles(&path)
}

#[tauri::command]
fn get_app_icon(path: String) -> Option<String> {
    apps::get_app_icon_data_url(&path)
}

#[tauri::command]
fn set_deck_icon_from_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    item_type: String,
    key: String,
    source_path: String,
) -> Result<String, String> {
    let filename = remote_icons::save_custom_icon(&app, &source_path)?;
    let slot = remote_icons::slot_id(&item_type, &key);

    {
        let mut cfg = state.config.write().unwrap();
        if let Some(old) = cfg.remote_deck.custom_icons.insert(slot, filename.clone()) {
            let _ = remote_icons::delete_custom_icon(&app, &old);
        }
        save_config(&app, &cfg)?;
    }

    remote_icons::custom_icon_data_url(&app, &filename).ok_or_else(|| "Không đọc được icon".to_string())
}

#[tauri::command]
fn clear_deck_icon(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    item_type: String,
    key: String,
) -> Result<(), String> {
    let slot = remote_icons::slot_id(&item_type, &key);

    {
        let mut cfg = state.config.write().unwrap();
        if let Some(old) = cfg.remote_deck.custom_icons.remove(&slot) {
            remote_icons::delete_custom_icon(&app, &old)?;
        }
        save_config(&app, &cfg)?;
    }

    Ok(())
}

#[tauri::command]
fn get_deck_icon_data_url(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    item_type: String,
    key: String,
) -> Option<String> {
    let cfg = state.config.read().unwrap();
    let slot = remote_icons::slot_id(&item_type, &key);
    let filename = cfg.remote_deck.custom_icons.get(&slot)?;
    remote_icons::custom_icon_data_url(&app, filename)
}

#[tauri::command]
fn set_hotkey_icon_from_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    app_key: String,
    hotkey_id: String,
    source_path: String,
) -> Result<String, String> {
    let filename = remote_icons::save_custom_icon(&app, &source_path)?;
    let new_icon = format!("custom:{filename}");

    {
        let mut cfg = state.config.write().unwrap();
        let entry = cfg
            .apps
            .get_mut(&app_key)
            .ok_or_else(|| format!("App '{app_key}' không tồn tại"))?;
        let binding = entry
            .hotkeys
            .iter_mut()
            .find(|item| item.id == hotkey_id)
            .ok_or_else(|| format!("Phím tắt '{hotkey_id}' không tồn tại"))?;

        remote_icons::delete_hotkey_icon(&app, binding.icon.as_deref());
        binding.icon = Some(new_icon.clone());
        save_config(&app, &cfg)?;
    }

    Ok(new_icon)
}

#[tauri::command]
fn get_hotkey_icon_data_url(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    app_key: String,
    hotkey_id: String,
) -> Option<String> {
    let cfg = state.config.read().unwrap();
    let entry = cfg.apps.get(&app_key)?;
    let binding = entry.hotkeys.iter().find(|item| item.id == hotkey_id)?;
    let filename = binding.icon.as_deref().and_then(remote_icons::custom_icon_filename)?;
    remote_icons::custom_icon_data_url(&app, filename)
}

#[tauri::command]
fn cleanup_hotkey_icon(app: tauri::AppHandle, icon: String) -> Result<(), String> {
    if let Some(filename) = remote_icons::custom_icon_filename(&icon) {
        remote_icons::delete_custom_icon(&app, filename)?;
    }
    Ok(())
}

#[tauri::command]
fn list_installed_apps_cmd() -> Vec<apps::InstalledApp> {
    apps::list_installed_apps()
}

#[tauri::command]
fn resolve_launch_path_cmd(path: String) -> String {
    apps::resolve_launch_path(&path)
        .to_string_lossy()
        .into_owned()
}

#[derive(serde::Serialize)]
struct ServerStatusResponse {
    online: bool,
    version: String,
    port: u16,
    https_port: u16,
    lan_ip: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn get_server_status(state: tauri::State<'_, Arc<AppState>>) -> ServerStatusResponse {
    let port = state.config.read().unwrap().port;
    ServerStatusResponse {
        online: state.is_server_running(),
        version: state::APP_VERSION.to_string(),
        port,
        https_port: api::server::https_port(port),
        lan_ip: net::get_lan_ip(),
        error: state.server_error.read().unwrap().clone(),
    }
}

#[derive(serde::Serialize)]
struct AccessibilityStatusResponse {
    supported: bool,
    trusted: bool,
    dev_build: bool,
    executable_path: Option<String>,
    dev_app_path: Option<String>,
    codesign_identifier: Option<String>,
    stable_signature: bool,
    hint: String,
}

#[tauri::command]
fn get_accessibility_status() -> AccessibilityStatusResponse {
    #[cfg(target_os = "macos")]
    {
        let status = macos::accessibility::status();
        return AccessibilityStatusResponse {
            supported: status.supported,
            trusted: status.trusted,
            dev_build: status.dev_build,
            executable_path: status.executable_path,
            dev_app_path: status.dev_app_path,
            codesign_identifier: status.codesign_identifier,
            stable_signature: status.stable_signature,
            hint: status.hint,
        };
    }

    #[cfg(not(target_os = "macos"))]
    AccessibilityStatusResponse {
        supported: false,
        trusted: true,
        dev_build: false,
        executable_path: None,
        dev_app_path: None,
        codesign_identifier: None,
        stable_signature: true,
        hint: "Accessibility chỉ cần trên macOS.".to_string(),
    }
}

#[tauri::command]
fn prompt_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        return macos::accessibility::prompt_permission();
    }

    #[cfg(not(target_os = "macos"))]
    true
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return macos::accessibility::open_settings();
    }

    #[cfg(not(target_os = "macos"))]
    Ok(())
}

#[tauri::command]
fn sign_dev_binary_cmd() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::accessibility::sign_current_dev_binary();
    }

    #[cfg(not(target_os = "macos"))]
    Err("Chỉ hỗ trợ trên macOS.".to_string())
}

#[tauri::command]
fn open_dev_app_bundle_cmd() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::accessibility::open_dev_app_bundle();
    }
    #[cfg(not(target_os = "macos"))]
    Err("Chỉ hỗ trợ trên macOS.".to_string())
}

#[tauri::command]
fn reset_accessibility_cmd() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::accessibility::reset_accessibility_approval();
    }
    #[cfg(not(target_os = "macos"))]
    Err("Chỉ hỗ trợ trên macOS.".to_string())
}

#[tauri::command]
fn test_hotkey_input(app: tauri::AppHandle) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        if !macos::accessibility::is_trusted() {
            return Err(macos::accessibility::input_permission_error());
        }
        let (tx, rx) = std::sync::mpsc::channel();
        app.run_on_main_thread(move || {
            let result =
                launcher::hotkey::send_accelerator("CommandOrControl+Shift+Option+F15");
            let _ = tx.send(result);
        })
        .map_err(|err| err.to_string())?;

        rx.recv()
            .map_err(|_| "Không nhận được kết quả test phím".to_string())??;

        return Ok(
            "Đã gửi phím test (⌃⌥⇧⌘F15). Nếu không thấy lỗi, Accessibility hoạt động.".to_string(),
        );
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        Ok("macOS only".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    #[cfg(all(target_os = "macos", debug_assertions))]
    macos::accessibility::ensure_stable_dev_signature();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .setup(|app| {
            let config = load_config(app.handle())?;
            let state = Arc::new(AppState::new(config.clone(), app.handle().clone()));

            let _ = sync_autostart(app.handle(), config.autostart);

            app.manage(state.clone());

            #[cfg(desktop)]
            shortcuts::setup_plugin(app.handle())?;

            tray::setup_tray(app.handle())?;
            tray::hide_from_dock(app.handle());

            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        tray::hide_main_window(&app_handle);
                    }
                });
            }

            let app_handle = app.handle().clone();
            let state_clone = state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = start_server(app_handle, state_clone).await {
                    eprintln!("Failed to start API server: {err}");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_dashboard,
            get_config,
            update_config,
            restart_api_server,
            get_request_logs,
            regenerate_token,
            set_autostart,
            get_autostart_enabled,
            get_server_uptime,
            get_server_status,
            get_claude_usage_status,
            get_claude_diagnostics,
            get_codex_usage_status,
            get_system_metrics_status,
            get_monitor_status,
            list_installed_apps_cmd,
            resolve_launch_path_cmd,
            list_browser_profiles,
            get_app_icon,
            set_deck_icon_from_file,
            clear_deck_icon,
            get_deck_icon_data_url,
            set_hotkey_icon_from_file,
            get_hotkey_icon_data_url,
            cleanup_hotkey_icon,
            test_open_app,
            get_accessibility_status,
            prompt_accessibility_permission,
            open_accessibility_settings,
            sign_dev_binary_cmd,
            open_dev_app_bundle_cmd,
            reset_accessibility_cmd,
            test_hotkey_input,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, code, .. } if code.is_none() => {
                api.prevent_exit();
                tray::hide_main_window(&app_handle);
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                tray::show_main_window(&app_handle);
            }
            _ => {}
        });
}
