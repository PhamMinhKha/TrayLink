use std::path::PathBuf;

use super::auth::{credential_target_names, extract_access_token};

pub fn read_token_from_secure_store() -> Result<Option<String>, String> {
    for target in credential_target_names() {
        for account in credential_account_candidates() {
            if let Some(token) = read_keyring_entry(&target, &account)? {
                return Ok(Some(token));
            }
        }
    }

    Ok(None)
}

fn credential_account_candidates() -> Vec<String> {
    let mut accounts = Vec::new();
    let push_unique = |accounts: &mut Vec<String>, value: String| {
        if !value.is_empty() && !accounts.iter().any(|existing| existing == &value) {
            accounts.push(value);
        }
    };

    if let Ok(user) = std::env::var("USERNAME") {
        push_unique(&mut accounts, user.clone());
        if let Ok(domain) = std::env::var("USERDOMAIN") {
            push_unique(&mut accounts, format!("{domain}\\{user}"));
            push_unique(&mut accounts, format!("{user}@{domain}"));
        }
    }

    if let Ok(user) = std::env::var("USER") {
        push_unique(&mut accounts, user);
    }

    accounts.push(String::new());
    accounts
}

fn read_keyring_entry(target: &str, account: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(target, account).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(raw) => Ok(extract_access_token(&raw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

pub fn secure_store_targets() -> Vec<String> {
    credential_target_names()
}

pub fn secure_store_label() -> &'static str {
    "Credential Manager"
}

pub fn claude_desktop_installed() -> (bool, bool) {
    let candidates = [
        local_app_data().map(|base| base.join("Programs").join("claude").join("Claude.exe")),
        local_app_data().map(|base| base.join("AnthropicClaude").join("Claude.exe")),
        local_app_data().map(|base| base.join("Claude").join("Claude.exe")),
        program_files().map(|base| base.join("Claude").join("Claude.exe")),
    ];

    for path in candidates.into_iter().flatten() {
        if path.exists() {
            return (true, true);
        }
    }

    let app_data = app_data_dir();
    let desktop_hint = app_data
        .map(|base| base.join("Claude"))
        .is_some_and(|path| path.exists());
    (desktop_hint, desktop_hint)
}

fn local_app_data() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

fn app_data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

fn program_files() -> Option<PathBuf> {
    std::env::var_os("ProgramFiles").map(PathBuf::from)
}

pub fn claude_desktop_detail(installed: bool) -> String {
    if installed {
        "Phát hiện Claude Desktop trên Windows.".to_string()
    } else {
        "Không thấy Claude Desktop trong %LOCALAPPDATA%, %ProgramFiles% hoặc %APPDATA%.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_candidates_include_username() {
        let accounts = credential_account_candidates();
        assert!(accounts.iter().any(|account| account.is_empty()));
        if let Ok(user) = std::env::var("USERNAME") {
            assert!(accounts.iter().any(|account| account == &user));
        }
    }
}
