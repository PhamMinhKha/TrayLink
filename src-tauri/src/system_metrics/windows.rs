use wmi::{COMLibrary, WMIConnection};

use super::types::*;

pub fn read_cpu_temperature() -> MetricTemperature {
    match try_read_cpu_temperature() {
        Ok(Some(c)) => metric_ok_temperature(c),
        Ok(None) => temperature_unsupported(
            "Không có cảm biến nhiệt độ CPU qua WMI trên máy này.",
        ),
        Err(e) => temperature_error(&format!("Lỗi đọc nhiệt độ CPU: {e}")),
    }
}

fn try_read_cpu_temperature() -> Result<Option<f64>, String> {
    let com = COMLibrary::new().map_err(|e| e.to_string())?;
    let wmi = WMIConnection::with_namespace_path("root\\WMI", com)
        .map_err(|e| e.to_string())?;

    #[derive(serde::Deserialize)]
    struct ThermalZone {
        #[serde(rename = "CurrentTemperature")]
        current_temperature: Option<u32>,
    }

    let zones: Vec<ThermalZone> = wmi
        .raw_query("SELECT CurrentTemperature FROM MSAcpi_ThermalZoneTemperature")
        .map_err(|e| e.to_string())?;

    let mut temps = Vec::new();
    for zone in zones {
        if let Some(raw) = zone.current_temperature {
            // Tenths of Kelvin → Celsius
            let c = (raw as f64 / 10.0) - 273.15;
            if c.is_finite() && (-50.0..150.0).contains(&c) {
                temps.push(c);
            }
        }
    }

    if temps.is_empty() {
        return Ok(None);
    }
    Ok(Some(temps.into_iter().fold(f64::MIN, f64::max)))
}

pub fn read_battery_temperature() -> MetricTemperature {
    match try_read_battery_temperature() {
        Ok(Some(c)) => metric_ok_temperature(c),
        Ok(None) => temperature_unsupported(
            "Không có pin hoặc cảm biến nhiệt pin trên máy này.",
        ),
        Err(e) => temperature_error(&format!("Lỗi đọc nhiệt pin: {e}")),
    }
}

fn try_read_battery_temperature() -> Result<Option<f64>, String> {
    let com = COMLibrary::new().map_err(|e| e.to_string())?;
    let wmi = WMIConnection::new(com).map_err(|e| e.to_string())?;

    #[derive(serde::Deserialize)]
    struct Battery {
        #[serde(rename = "EstimatedChargeRemaining")]
        estimated_charge_remaining: Option<u16>,
        #[serde(rename = "Temperature")]
        temperature: Option<u16>,
    }

    let batteries: Vec<Battery> = wmi
        .raw_query("SELECT EstimatedChargeRemaining, Temperature FROM Win32_Battery")
        .map_err(|e| e.to_string())?;

    if batteries.is_empty() {
        return Ok(None);
    }

    for bat in batteries {
        if let Some(temp) = bat.temperature {
            if temp > 0 {
                // Win32_Battery Temperature is tenths of Kelvin in some docs, Celsius*10 in others.
                // Values > 500 are tenths Kelvin.
                let c = if temp > 500 {
                    (temp as f64 / 10.0) - 273.15
                } else {
                    temp as f64 / 10.0
                };
                if c.is_finite() && (-20.0..80.0).contains(&c) {
                    return Ok(Some(c));
                }
            }
        }
        let _ = bat.estimated_charge_remaining;
    }

    Ok(None)
}

pub fn read_fan_speed() -> MetricFan {
    match try_read_fan_speed() {
        Ok(Some(rpm)) => metric_ok_fan(rpm),
        Ok(None) => fan_unsupported(
            "Hầu hết laptop Windows không expose tốc độ quạt qua WMI chuẩn.",
        ),
        Err(e) => fan_error(&format!("Lỗi đọc quạt: {e}")),
    }
}

fn try_read_fan_speed() -> Result<Option<u64>, String> {
    let com = COMLibrary::new().map_err(|e| e.to_string())?;
    let wmi = WMIConnection::with_namespace_path("root\\WMI", com)
        .map_err(|e| e.to_string())?;

    #[derive(serde::Deserialize)]
    struct Fan {
        #[serde(rename = "DesiredSpeed")]
        desired_speed: Option<u64>,
        #[serde(rename = "Active")]
        active: Option<bool>,
    }

    let fans: Vec<Fan> = wmi
        .raw_query("SELECT DesiredSpeed, Active FROM MSAcpi_Fan")
        .unwrap_or_default();

    for fan in fans {
        if fan.active.unwrap_or(true) {
            if let Some(rpm) = fan.desired_speed {
                if rpm > 0 {
                    return Ok(Some(rpm));
                }
            }
        }
    }

    Ok(None)
}
