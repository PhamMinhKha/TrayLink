use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub const CREDENTIALS_FILE: &str = ".credentials.json";
pub const CLAUDE_JSON_FILE: &str = ".claude.json";
pub const BASE_CREDENTIAL_TARGETS: &[&str] =
    &["Claude Code-credentials", "Claude Code", "Claude"];

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn claude_config_dir() -> Option<PathBuf> {
    std::env::var("CLAUDE_CONFIG_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".claude")))
}

pub fn credentials_path() -> Option<PathBuf> {
    claude_config_dir().map(|dir| dir.join(CREDENTIALS_FILE))
}

pub fn claude_json_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(CLAUDE_JSON_FILE))
}

/// Claude Code v2 namespaced credential target: `Claude Code-credentials-<sha256(dir)[:8]>`.
pub fn credential_target_names() -> Vec<String> {
    let mut targets: Vec<String> = BASE_CREDENTIAL_TARGETS
        .iter()
        .map(|name| (*name).to_string())
        .collect();

    if let Some(dir) = claude_config_dir() {
        if let Some(suffix) = hashed_config_suffix(&dir) {
            targets.push(format!("Claude Code-credentials-{suffix}"));
        }
    }

    targets
}

pub fn hashed_config_suffix(dir: &Path) -> Option<String> {
    let abs = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(abs.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    Some(format!(
        "{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    ))
}

pub fn read_token_from_env() -> Option<String> {
    std::env::var("CLAUDE_CODE_OAUTH_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn extract_access_token(blob: &str) -> Option<String> {
    let blob = blob.trim();
    if blob.is_empty() {
        return None;
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(blob) {
        if let Some(token) = token_from_json_value(&value) {
            return Some(token);
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
        && blob.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '~' | '+' | '/' | '=')
        })
    {
        return Some(blob.to_string());
    }

    None
}

fn token_from_json_value(value: &serde_json::Value) -> Option<String> {
    if let Some(oauth) = value.get("claudeAiOauth") {
        if let Some(token) = token_field(oauth) {
            return Some(token);
        }
    }

    if let Some(token) = token_field(value) {
        return Some(token);
    }

    if let Some(obj) = value.as_object() {
        for nested in obj.values() {
            if let Some(token) = token_from_json_value(nested) {
                return Some(token);
            }
        }
    }

    None
}

fn token_field(value: &serde_json::Value) -> Option<String> {
    value
        .get("accessToken")
        .or_else(|| value.get("access_token"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|token| !token.is_empty())
}

pub fn read_token_from_file() -> Result<Option<String>, String> {
    read_token_at_path(credentials_path())
}

pub fn read_token_from_claude_json() -> Result<Option<String>, String> {
    read_token_at_path(claude_json_path())
}

fn read_token_at_path(path: Option<PathBuf>) -> Result<Option<String>, String> {
    let path = match path {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_claude_ai_oauth_token() {
        let raw = r#"{"claudeAiOauth":{"accessToken":"test-token-123","refreshToken":"r"}}"#;
        assert_eq!(
            extract_access_token(raw).as_deref(),
            Some("test-token-123")
        );
    }

    #[test]
    fn extracts_top_level_access_token() {
        let raw = r#"{"accessToken":"legacy-token"}"#;
        assert_eq!(
            extract_access_token(raw).as_deref(),
            Some("legacy-token")
        );
    }

    #[test]
    fn hashed_suffix_is_eight_hex_chars() {
        let suffix = hashed_config_suffix(Path::new("/tmp/.claude")).expect("suffix");
        assert_eq!(suffix.len(), 8);
        assert!(suffix.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn credential_targets_include_namespaced_entry() {
        let targets = credential_target_names();
        assert!(targets.contains(&"Claude Code-credentials".to_string()));
        assert!(targets.iter().any(|name| name.starts_with("Claude Code-credentials-")));
    }
}
