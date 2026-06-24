use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Rect,
};

use crate::api::server::restart_server;

#[cfg(target_os = "macos")]
pub fn hide_from_dock(app: &AppHandle) {
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    let _ = app.set_dock_visibility(false);
}

#[cfg(not(target_os = "macos"))]
pub fn hide_from_dock(_app: &AppHandle) {}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    hide_from_dock(app);
}

pub fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    hide_from_dock(app);
}

fn position_usage_popup(window: &tauri::WebviewWindow, anchor: Option<Rect>) {
    if let Some(rect) = anchor {
        let (x, y, height) = match (rect.position, rect.size) {
            (tauri::Position::Physical(pos), tauri::Size::Physical(size)) => {
                (pos.x, pos.y, size.height as i32)
            }
            (tauri::Position::Logical(pos), tauri::Size::Logical(size)) => {
                (pos.x as i32, pos.y as i32, size.height as i32)
            }
            (tauri::Position::Physical(pos), tauri::Size::Logical(size)) => {
                (pos.x, pos.y, size.height as i32)
            }
            (tauri::Position::Logical(pos), tauri::Size::Physical(size)) => {
                (pos.x as i32, pos.y as i32, size.height as i32)
            }
        };
        let _ = window.set_position(PhysicalPosition::new(x, y + height + 4));
        return;
    }

    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen = monitor.size();
        let width = window.outer_size().map(|size| size.width).unwrap_or(340);
        let x = screen.width as i32 - width as i32 - 12;
        let _ = window.set_position(PhysicalPosition::new(x, 24));
    }
}

pub fn toggle_usage_popup(app: &AppHandle, anchor: Option<Rect>) {
    let Some(window) = app.get_webview_window("usage-popup") else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    position_usage_popup(&window, anchor);
    let _ = window.show();
    let _ = window.set_focus();
    hide_from_dock(app);
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let usage_item = MenuItem::with_id(app, "usage", "Usage", true, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", "Open Dashboard", true, None::<&str>)?;
    let restart_item = MenuItem::with_id(app, "restart", "Restart Server", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&usage_item, &open_item, &restart_item, &quit_item])?;

    let icon = app
        .default_window_icon()
        .ok_or("missing default window icon")?
        .clone();

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("TrayLink")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "usage" => toggle_usage_popup(app, None),
            "open" => show_main_window(app),
            "restart" => {
                if let Some(state) = app.try_state::<std::sync::Arc<crate::state::AppState>>() {
                    let app_handle = app.clone();
                    let state_clone = state.inner().clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(err) = restart_server(app_handle, state_clone).await {
                            eprintln!("Failed to restart server: {err}");
                        }
                    });
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                toggle_usage_popup(tray.app_handle(), Some(rect));
            }
        })
        .build(app)?;

    Ok(())
}
