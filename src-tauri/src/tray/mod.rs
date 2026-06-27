use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Rect,
};

use crate::api::server::restart_server;

const POPUP_MARGIN: i32 = 8;
const POPUP_GAP: i32 = 4;
const DEFAULT_POPUP_WIDTH: i32 = 340;
const DEFAULT_POPUP_HEIGHT: i32 = 480;

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

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if max < min {
        return min;
    }

    value.clamp(min, max)
}

fn rect_to_physical(rect: Rect) -> (i32, i32, i32, i32) {
    let (x, y) = match rect.position {
        tauri::Position::Physical(pos) => (pos.x, pos.y),
        tauri::Position::Logical(pos) => (pos.x as i32, pos.y as i32),
    };

    let (width, height) = match rect.size {
        tauri::Size::Physical(size) => (size.width as i32, size.height as i32),
        tauri::Size::Logical(size) => (size.width as i32, size.height as i32),
    };

    (x, y, width, height)
}

fn position_usage_popup(window: &tauri::WebviewWindow, anchor: Option<Rect>) {
    let popup_size = window.outer_size().ok();
    let popup_width = popup_size
        .as_ref()
        .map(|size| size.width as i32)
        .unwrap_or(DEFAULT_POPUP_WIDTH);
    let popup_height = popup_size
        .as_ref()
        .map(|size| size.height as i32)
        .unwrap_or(DEFAULT_POPUP_HEIGHT);

    if let Ok(Some(monitor)) = window.current_monitor() {
        let work_area = monitor.work_area();
        let left = work_area.position.x;
        let top = work_area.position.y;
        let right = left + work_area.size.width as i32;
        let bottom = top + work_area.size.height as i32;

        if let Some(rect) = anchor {
            let (x, y, width, height) = rect_to_physical(rect);
            let centered_x = x + (width / 2) - (popup_width / 2);
            let below_y = y + height + POPUP_GAP;
            let above_y = y - popup_height - POPUP_GAP;

            // Prefer opening above the tray icon when the popup would run into the taskbar.
            let position_y = if below_y + popup_height <= bottom - POPUP_MARGIN {
                below_y
            } else if above_y >= top + POPUP_MARGIN {
                above_y
            } else {
                clamp(below_y, top + POPUP_MARGIN, bottom - popup_height - POPUP_MARGIN)
            };

            let position_x = clamp(
                centered_x,
                left + POPUP_MARGIN,
                right - popup_width - POPUP_MARGIN,
            );

            let _ = window.set_position(PhysicalPosition::new(position_x, position_y));
            return;
        }

        let x = clamp(
            right - popup_width - POPUP_MARGIN,
            left + POPUP_MARGIN,
            right - popup_width - POPUP_MARGIN,
        );
        let y = top + POPUP_MARGIN;
        let _ = window.set_position(PhysicalPosition::new(x, y));
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
