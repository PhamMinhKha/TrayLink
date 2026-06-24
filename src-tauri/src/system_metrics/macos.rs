use std::process::Command;

use super::types::*;

pub fn read_cpu_temperature() -> MetricTemperature {
    temperature_unsupported(
        "macOS không có API công khai ổn định cho nhiệt độ CPU (cần quyền SMC/IOKit).",
    )
}

pub fn read_battery_temperature() -> MetricTemperature {
    match try_read_battery_temperature() {
        Ok(Some(c)) => metric_ok_temperature(c),
        Ok(None) => temperature_unsupported(
            "Không đọc được nhiệt pin qua ioreg trên máy này.",
        ),
        Err(e) => temperature_error(&format!("Lỗi đọc nhiệt pin: {e}")),
    }
}

fn try_read_battery_temperature() -> Result<Option<f64>, String> {
    let output = Command::new("ioreg")
        .args(["-rn", "AppleSmartBattery"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"Temperature\"=") {
            if let Some(raw) = trimmed.split('=').nth(1) {
                let raw = raw.trim();
                // Temperature in 0.01°C units
                if let Ok(val) = raw.parse::<i64>() {
                    let c = val as f64 / 100.0;
                    if c.is_finite() && (-20.0..80.0).contains(&c) {
                        return Ok(Some(c));
                    }
                }
            }
        }
    }
    Ok(None)
}

pub fn read_fan_speed() -> MetricFan {
    match try_read_fan_speed() {
        Ok(Some(rpm)) => metric_ok_fan(rpm),
        Ok(None) => fan_unsupported(
            "Không đọc được tốc độ quạt qua ioreg/SMC trên model máy này.",
        ),
        Err(e) => fan_error(&format!("Lỗi đọc quạt: {e}")),
    }
}

fn try_read_fan_speed() -> Result<Option<u64>, String> {
    let output = Command::new("ioreg")
        .args(["-r", "-c", "AppleSMC"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"F0Ac\"=") || trimmed.contains("\"F1Ac\"=") {
            if let Some(raw) = trimmed.split('=').nth(1) {
                let raw = raw.trim();
                if let Ok(val) = raw.parse::<u64>() {
                    if val > 0 {
                        return Ok(Some(val));
                    }
                }
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_temp_is_unsupported_on_macos() {
        let t = read_cpu_temperature();
        assert_eq!(t.status, MetricStatus::Unsupported);
    }
}
