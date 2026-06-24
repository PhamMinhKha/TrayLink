mod sysinfo_collector;
#[cfg(target_os = "macos")]
mod macos;
mod types;
#[cfg(windows)]
mod windows;

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use chrono::Utc;

use crate::config::SystemMetricsPreferences;

pub use types::*;

struct CacheEntry {
    prefs_key: u8,
    at: Instant,
    response: SystemMetricsResponse,
}

static CACHE: Mutex<Option<CacheEntry>> = Mutex::new(None);
static SYSINFO: OnceLock<Mutex<sysinfo_collector::SysinfoCollector>> = OnceLock::new();

fn sysinfo_collector() -> &'static Mutex<sysinfo_collector::SysinfoCollector> {
    SYSINFO.get_or_init(|| Mutex::new(sysinfo_collector::SysinfoCollector::new()))
}

const CACHE_TTL: Duration = Duration::from_secs(3);

fn prefs_cache_key(prefs: &SystemMetricsPreferences) -> u8 {
    let mut key = 0u8;
    if prefs.cpu {
        key |= 1;
    }
    if prefs.memory {
        key |= 2;
    }
    if prefs.disk {
        key |= 4;
    }
    if prefs.network {
        key |= 8;
    }
    if prefs.cpu_temperature {
        key |= 16;
    }
    if prefs.battery_temperature {
        key |= 32;
    }
    if prefs.fan_speed {
        key |= 64;
    }
    key
}

pub fn get_status(prefs: &SystemMetricsPreferences) -> SystemMetricsResponse {
    if !prefs.any_enabled() {
        return SystemMetricsResponse::disabled();
    }

    let key = prefs_cache_key(prefs);
    if let Ok(guard) = CACHE.lock() {
        if let Some(entry) = guard.as_ref() {
            if entry.prefs_key == key && entry.at.elapsed() < CACHE_TTL {
                return entry.response.clone();
            }
        }
    }

    let response = collect(prefs);
    if let Ok(mut guard) = CACHE.lock() {
        *guard = Some(CacheEntry {
            prefs_key: key,
            at: Instant::now(),
            response: response.clone(),
        });
    }
    response
}

fn collect(prefs: &SystemMetricsPreferences) -> SystemMetricsResponse {
    let mut ok = true;
    let mut cpu = None;
    let mut memory = None;
    let mut disk = None;
    let mut network = None;
    let mut cpu_temperature = None;
    let mut battery_temperature = None;
    let mut fan = None;

    if prefs.cpu || prefs.memory || prefs.disk || prefs.network {
        if let Ok(mut collector) = sysinfo_collector().lock() {
            if prefs.cpu {
                cpu = Some(collector.collect_cpu());
            }
            if prefs.memory {
                memory = Some(collector.collect_memory());
            }
            if prefs.disk {
                disk = Some(collector.collect_disk());
            }
            if prefs.network {
                network = Some(collector.collect_network());
            }
        }
    }

    if prefs.cpu_temperature {
        cpu_temperature = Some(read_cpu_temperature());
    }
    if prefs.battery_temperature {
        battery_temperature = Some(read_battery_temperature());
    }
    if prefs.fan_speed {
        fan = Some(read_fan_speed());
    }

    for metric in [
        cpu.as_ref().map(|m| m.status.clone()),
        memory.as_ref().map(|m| m.status.clone()),
        disk.as_ref().map(|m| m.status.clone()),
        network.as_ref().map(|m| m.status.clone()),
        cpu_temperature.as_ref().map(|m| m.status.clone()),
        battery_temperature.as_ref().map(|m| m.status.clone()),
        fan.as_ref().map(|m| m.status.clone()),
    ]
    .into_iter()
    .flatten()
    {
        if metric == MetricStatus::Error {
            ok = false;
        }
    }

    SystemMetricsResponse {
        enabled: true,
        ok,
        updated_at: Some(Utc::now().to_rfc3339()),
        error: if ok {
            None
        } else {
            Some("Một số metric gặp lỗi đọc dữ liệu.".into())
        },
        cpu,
        memory,
        disk,
        network,
        cpu_temperature,
        battery_temperature,
        fan,
    }
}

fn read_cpu_temperature() -> MetricTemperature {
    #[cfg(windows)]
    {
        return windows::read_cpu_temperature();
    }
    #[cfg(target_os = "macos")]
    {
        return macos::read_cpu_temperature();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        temperature_unsupported("Nền tảng này chưa hỗ trợ nhiệt độ CPU.")
    }
}

fn read_battery_temperature() -> MetricTemperature {
    #[cfg(windows)]
    {
        return windows::read_battery_temperature();
    }
    #[cfg(target_os = "macos")]
    {
        return macos::read_battery_temperature();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        temperature_unsupported("Nền tảng này chưa hỗ trợ nhiệt pin.")
    }
}

fn read_fan_speed() -> MetricFan {
    #[cfg(windows)]
    {
        return windows::read_fan_speed();
    }
    #[cfg(target_os = "macos")]
    {
        return macos::read_fan_speed();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        fan_unsupported("Nền tảng này chưa hỗ trợ tốc độ quạt.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_when_no_prefs() {
        let prefs = SystemMetricsPreferences::default();
        let resp = get_status(&prefs);
        assert!(!resp.enabled);
    }

    #[test]
    fn cache_returns_same_within_ttl() {
        let prefs = SystemMetricsPreferences {
            cpu: true,
            ..Default::default()
        };
        let a = get_status(&prefs);
        let b = get_status(&prefs);
        assert_eq!(a.updated_at, b.updated_at);
    }

    #[test]
    fn response_serializes() {
        let prefs = SystemMetricsPreferences {
            memory: true,
            ..Default::default()
        };
        let resp = get_status(&prefs);
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"memory\""));
    }
}
