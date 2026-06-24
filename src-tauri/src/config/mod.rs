pub mod store;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn default_allow_get() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteDeckLayout {
    #[serde(default)]
    pub display_order: Vec<String>,
    #[serde(default)]
    pub app_order: Vec<String>,
    #[serde(default)]
    pub command_order: Vec<String>,
    #[serde(default)]
    pub hidden_apps: Vec<String>,
    #[serde(default)]
    pub hidden_commands: Vec<String>,
    #[serde(default)]
    pub custom_icons: HashMap<String, String>,
}

fn default_usage_monitor_enabled() -> bool {
    false
}

fn default_codex_usage_monitor_enabled() -> bool {
    false
}

impl RemoteDeckLayout {
    pub fn deck_items<'a>(
        app_keys: impl Iterator<Item = &'a str>,
        command_keys: impl Iterator<Item = &'a str>,
        layout: &RemoteDeckLayout,
    ) -> Vec<(String, String)> {
        use std::collections::HashSet;

        let apps: HashSet<&str> = app_keys.collect();
        let commands: HashSet<&str> = command_keys.collect();
        let hidden_apps: HashSet<&str> = layout.hidden_apps.iter().map(String::as_str).collect();
        let hidden_commands: HashSet<&str> = layout.hidden_commands.iter().map(String::as_str).collect();

        let mut result = Vec::new();
        let mut seen_apps = HashSet::new();
        let mut seen_cmds = HashSet::new();

        for slot in &layout.display_order {
            if let Some((item_type, key)) = slot.split_once(':') {
                match item_type {
                    "app" if apps.contains(key) && !hidden_apps.contains(key) => {
                        seen_apps.insert(key.to_string());
                        result.push(("app".to_string(), key.to_string()));
                    }
                    "cmd" if commands.contains(key) && !hidden_commands.contains(key) => {
                        seen_cmds.insert(key.to_string());
                        result.push(("cmd".to_string(), key.to_string()));
                    }
                    _ => {}
                }
            }
        }

        for key in Self::ordered_visible(apps.iter().copied(), &layout.app_order, &layout.hidden_apps) {
            if seen_apps.insert(key.clone()) {
                result.push(("app".to_string(), key));
            }
        }

        for key in Self::ordered_visible(
            commands.iter().copied(),
            &layout.command_order,
            &layout.hidden_commands,
        ) {
            if seen_cmds.insert(key.clone()) {
                result.push(("cmd".to_string(), key));
            }
        }

        result
    }

    pub fn ordered_visible<'a>(
        all_keys: impl Iterator<Item = &'a str>,
        order: &[String],
        hidden: &[String],
    ) -> Vec<String> {
        use std::collections::HashSet;

        let all: HashSet<&str> = all_keys.collect();
        let hidden_set: HashSet<&str> = hidden.iter().map(String::as_str).collect();
        let mut result = Vec::new();

        for key in order {
            if all.contains(key.as_str()) && !hidden_set.contains(key.as_str()) {
                result.push(key.clone());
            }
        }

        let mut remaining: Vec<String> = all
            .iter()
            .filter(|key| {
                !order.iter().any(|k| k == **key) && !hidden_set.contains(**key)
            })
            .map(|key| (*key).to_string())
            .collect();
        remaining.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        result.extend(remaining);
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppHotkeyBinding {
    pub id: String,
    pub name: String,
    pub accelerator: String,
    /// `open` = mở/focus app · `keys` = gửi tổ hợp phím vào app
    #[serde(default = "default_hotkey_action")]
    pub action: String,
    /// `fa:floppy-disk` · `custom:uuid.png`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

fn default_hotkey_action() -> String {
    "keys".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyHotkeyEntry {
    pub accelerator: String,
    pub target_type: String,
    pub target_key: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub token: String,
    #[serde(default)]
    pub require_token: bool,
    #[serde(default = "default_allow_get")]
    pub allow_get: bool,
    #[serde(default = "default_usage_monitor_enabled")]
    pub usage_monitor_enabled: bool,
    #[serde(default = "default_codex_usage_monitor_enabled")]
    pub codex_usage_monitor_enabled: bool,
    pub autostart: bool,
    pub apps: HashMap<String, AppEntry>,
    pub commands: HashMap<String, ExecEntry>,
    #[serde(default)]
    pub remote_deck: RemoteDeckLayout,
    /// Chỉ dùng khi migrate config cũ — không ghi ra file mới.
    #[serde(default, skip_serializing)]
    pub hotkeys: HashMap<String, LegacyHotkeyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub path: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    /// Cho phép mở app kèm URL (trình duyệt hoặc app hỗ trợ URL).
    #[serde(default)]
    pub url_enabled: bool,
    /// Profile trình duyệt: `Default` / `Profile 1` (Chromium) hoặc `firefox:...`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_profile: Option<String>,
    #[serde(default)]
    pub hotkeys: Vec<AppHotkeyBinding>,
    /// Migrate field cũ — một phím tắt duy nhất.
    #[serde(default, skip_serializing, alias = "hotkey")]
    legacy_hotkey: Option<String>,
}

impl AppEntry {
    pub fn migrate_legacy_fields(&mut self) -> bool {
        let mut changed = false;
        if let Some(accelerator) = self.legacy_hotkey.take() {
            if self.hotkeys.is_empty() {
                self.hotkeys.push(AppHotkeyBinding {
                    id: "open".to_string(),
                    name: "Mở app".to_string(),
                    accelerator,
                    action: "open".to_string(),
                    icon: None,
                });
                changed = true;
            }
        }
        changed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecEntry {
    #[serde(default)]
    pub internal: bool,
    #[serde(default)]
    pub win: Option<String>,
    #[serde(default)]
    pub mac: Option<String>,
    #[serde(default)]
    pub linux: Option<String>,
}
