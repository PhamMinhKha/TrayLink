use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MetricStatus {
    Ok,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricCpu {
    pub status: MetricStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricMemory {
    pub status: MetricStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricDisk {
    pub status: MetricStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mount_point: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricNetwork {
    pub status: MetricStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_bps: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricTemperature {
    pub status: MetricStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub celsius: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricFan {
    pub status: MetricStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetricsResponse {
    pub enabled: bool,
    pub ok: bool,
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<MetricCpu>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MetricMemory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<MetricDisk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<MetricNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_temperature: Option<MetricTemperature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_temperature: Option<MetricTemperature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fan: Option<MetricFan>,
}

impl SystemMetricsResponse {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ok: true,
            updated_at: None,
            error: None,
            cpu: None,
            memory: None,
            disk: None,
            network: None,
            cpu_temperature: None,
            battery_temperature: None,
            fan: None,
        }
    }
}

pub fn metric_ok_cpu(usage_percent: f64) -> MetricCpu {
    MetricCpu {
        status: MetricStatus::Ok,
        message: None,
        usage_percent: Some(usage_percent.clamp(0.0, 100.0)),
    }
}

pub fn metric_ok_memory(used_percent: f64, used_bytes: u64, total_bytes: u64) -> MetricMemory {
    MetricMemory {
        status: MetricStatus::Ok,
        message: None,
        used_percent: Some(used_percent.clamp(0.0, 100.0)),
        used_bytes: Some(used_bytes),
        total_bytes: Some(total_bytes),
    }
}

pub fn metric_ok_disk(
    name: String,
    mount_point: String,
    used_percent: f64,
    available_bytes: u64,
    total_bytes: u64,
) -> MetricDisk {
    MetricDisk {
        status: MetricStatus::Ok,
        message: None,
        name: Some(name),
        mount_point: Some(mount_point),
        used_percent: Some(used_percent.clamp(0.0, 100.0)),
        available_bytes: Some(available_bytes),
        total_bytes: Some(total_bytes),
    }
}

pub fn metric_ok_network(upload_bps: f64, download_bps: f64) -> MetricNetwork {
    MetricNetwork {
        status: MetricStatus::Ok,
        message: None,
        upload_bps: Some(upload_bps.max(0.0)),
        download_bps: Some(download_bps.max(0.0)),
    }
}

pub fn metric_ok_temperature(celsius: f64) -> MetricTemperature {
    MetricTemperature {
        status: MetricStatus::Ok,
        message: None,
        celsius: Some(celsius),
    }
}

pub fn metric_ok_fan(rpm: u64) -> MetricFan {
    MetricFan {
        status: MetricStatus::Ok,
        message: None,
        rpm: Some(rpm),
    }
}

pub fn cpu_unsupported(message: &str) -> MetricCpu {
    MetricCpu {
        status: MetricStatus::Unsupported,
        message: Some(message.to_string()),
        usage_percent: None,
    }
}

pub fn memory_unsupported(message: &str) -> MetricMemory {
    MetricMemory {
        status: MetricStatus::Unsupported,
        message: Some(message.to_string()),
        used_percent: None,
        used_bytes: None,
        total_bytes: None,
    }
}

pub fn disk_unsupported(message: &str) -> MetricDisk {
    MetricDisk {
        status: MetricStatus::Unsupported,
        message: Some(message.to_string()),
        name: None,
        mount_point: None,
        used_percent: None,
        available_bytes: None,
        total_bytes: None,
    }
}

pub fn network_unsupported(message: &str) -> MetricNetwork {
    MetricNetwork {
        status: MetricStatus::Unsupported,
        message: Some(message.to_string()),
        upload_bps: None,
        download_bps: None,
    }
}

pub fn temperature_unsupported(message: &str) -> MetricTemperature {
    MetricTemperature {
        status: MetricStatus::Unsupported,
        message: Some(message.to_string()),
        celsius: None,
    }
}

pub fn fan_unsupported(message: &str) -> MetricFan {
    MetricFan {
        status: MetricStatus::Unsupported,
        message: Some(message.to_string()),
        rpm: None,
    }
}

pub fn temperature_error(message: &str) -> MetricTemperature {
    MetricTemperature {
        status: MetricStatus::Error,
        message: Some(message.to_string()),
        celsius: None,
    }
}

pub fn fan_error(message: &str) -> MetricFan {
    MetricFan {
        status: MetricStatus::Error,
        message: Some(message.to_string()),
        rpm: None,
    }
}
