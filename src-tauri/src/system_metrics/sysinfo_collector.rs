use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::{Disks, Networks, System};

use super::types::*;

pub struct SysinfoCollector {
    system: System,
    networks: Networks,
    last_network: HashMap<String, (u64, u64, Instant)>,
}

impl SysinfoCollector {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_memory();
        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            last_network: HashMap::new(),
        }
    }

    pub fn collect_cpu(&mut self) -> MetricCpu {
        self.system.refresh_cpu_usage();
        thread::sleep(Duration::from_millis(200));
        self.system.refresh_cpu_usage();
        let usage = self.system.global_cpu_usage();
        metric_ok_cpu(f64::from(usage))
    }

    pub fn collect_memory(&mut self) -> MetricMemory {
        self.system.refresh_memory();
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        if total == 0 {
            return memory_unsupported("Không đọc được dung lượng RAM.");
        }
        let used_percent = (used as f64 / total as f64) * 100.0;
        metric_ok_memory(used_percent, used, total)
    }

    pub fn collect_disk(&mut self) -> MetricDisk {
        let disks = Disks::new_with_refreshed_list();
        let disk = disks
            .iter()
            .find(|d| d.is_removable() == false && is_boot_disk(d))
            .or_else(|| disks.iter().find(|d| !d.is_removable()))
            .or_else(|| disks.iter().next());

        let Some(disk) = disk else {
            return disk_unsupported("Không tìm thấy ổ đĩa hệ thống.");
        };

        let total = disk.total_space();
        let available = disk.available_space();
        if total == 0 {
            return disk_unsupported("Không đọc được dung lượng ổ đĩa.");
        }
        let used = total.saturating_sub(available);
        let used_percent = (used as f64 / total as f64) * 100.0;
        metric_ok_disk(
            disk.name().to_string_lossy().into_owned(),
            disk.mount_point().to_string_lossy().into_owned(),
            used_percent,
            available,
            total,
        )
    }

    pub fn collect_network(&mut self) -> MetricNetwork {
        self.networks.refresh(true);
        let now = Instant::now();

        let mut current_rx = 0u64;
        let mut current_tx = 0u64;
        for (name, data) in self.networks.iter() {
            if is_loopback(name) {
                continue;
            }
            current_rx = current_rx.saturating_add(data.total_received());
            current_tx = current_tx.saturating_add(data.total_transmitted());
        }

        let upload_bps;
        let download_bps;
        if let Some((prev_rx, prev_tx, prev_at)) = self.last_network.get("_total") {
            let elapsed = now.duration_since(*prev_at).as_secs_f64();
            if elapsed > 0.0 {
                download_bps = current_rx.saturating_sub(*prev_rx) as f64 / elapsed;
                upload_bps = current_tx.saturating_sub(*prev_tx) as f64 / elapsed;
            } else {
                download_bps = 0.0;
                upload_bps = 0.0;
            }
        } else {
            download_bps = 0.0;
            upload_bps = 0.0;
        }

        self.last_network
            .insert("_total".to_string(), (current_rx, current_tx, now));

        metric_ok_network(upload_bps, download_bps)
    }
}

fn is_loopback(name: &str) -> bool {
    name == "lo" || name.starts_with("lo")
}

fn is_boot_disk(disk: &sysinfo::Disk) -> bool {
    let mount = disk.mount_point().to_string_lossy();
    #[cfg(windows)]
    {
        mount.eq_ignore_ascii_case("C:\\")
    }
    #[cfg(not(windows))]
    {
        mount == "/"
    }
}

impl Default for SysinfoCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_returns_ok_range() {
        let mut collector = SysinfoCollector::new();
        let cpu = collector.collect_cpu();
        assert_eq!(cpu.status, MetricStatus::Ok);
        let usage = cpu.usage_percent.unwrap();
        assert!((0.0..=100.0).contains(&usage));
    }

    #[test]
    fn memory_returns_bytes() {
        let mut collector = SysinfoCollector::new();
        let mem = collector.collect_memory();
        assert_eq!(mem.status, MetricStatus::Ok);
        assert!(mem.total_bytes.unwrap() > 0);
    }
}
