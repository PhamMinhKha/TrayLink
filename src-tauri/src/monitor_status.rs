use chrono::Utc;
use serde::Serialize;

use crate::codex_usage;
use crate::config::SystemMetricsPreferences;
use crate::system_metrics;
use crate::usage_monitor;
use crate::state::APP_VERSION;

#[derive(Debug, Clone, Serialize)]
pub struct MonitorStatusResponse {
    pub online: bool,
    pub version: String,
    pub updated_at: String,
    pub claude: usage_monitor::UsageMonitorResponse,
    pub codex: codex_usage::UsageMonitorResponse,
    pub system: system_metrics::SystemMetricsResponse,
}

pub async fn get_all_status(
    claude_enabled: bool,
    codex_enabled: bool,
    system_prefs: &SystemMetricsPreferences,
) -> MonitorStatusResponse {
    let (claude, codex) = tokio::join!(
        usage_monitor::get_usage_status(claude_enabled),
        codex_usage::get_usage_status(codex_enabled),
    );

    let claude = claude.unwrap_or_else(|error| usage_monitor::UsageMonitorResponse {
        enabled: claude_enabled,
        provider: "claude".to_string(),
        updated_at: None,
        session_5h: None,
        weekly_7d: None,
        ok: false,
        error: Some(error),
    });

    let system = system_metrics::get_status(system_prefs);

    MonitorStatusResponse {
        online: true,
        version: APP_VERSION.to_string(),
        updated_at: Utc::now().to_rfc3339(),
        claude,
        codex,
        system,
    }
}
