use std::process::Command;

use super::auth::{credential_target_names, extract_access_token};

pub fn read_token_from_secure_store() -> Result<Option<String>, String> {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map_err(|e| e.to_string())?;

    for service in credential_target_names() {
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                &service,
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

pub fn secure_store_targets() -> Vec<String> {
    credential_target_names()
}

pub fn secure_store_label() -> &'static str {
    "Keychain"
}

pub const CLAUDE_APP_PATH: &str = "/Applications/Claude.app";
pub const CLAUDE_APP_EXECUTABLE: &str = "/Applications/Claude.app/Contents/MacOS/Claude";

pub fn claude_desktop_installed() -> (bool, bool) {
    let installed = std::path::Path::new(CLAUDE_APP_PATH).exists();
    let executable = std::path::Path::new(CLAUDE_APP_EXECUTABLE).exists();
    (installed, executable)
}

pub fn claude_desktop_detail(installed: bool) -> String {
    if installed {
        "Bạn đang dùng Claude Desktop app trên macOS.".to_string()
    } else {
        "Không thấy /Applications/Claude.app.".to_string()
    }
}
