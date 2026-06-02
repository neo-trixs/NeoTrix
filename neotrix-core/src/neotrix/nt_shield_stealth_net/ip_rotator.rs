use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::lan_router::LanRouter;

const ROTATION_INTERVAL_SECS: u64 = 9;

#[derive(Debug, Clone)]
pub struct OsIpRotatorConfig {
    pub auto_add_alias_ips: bool,
    pub rotate_gateway: bool,
    pub alias_prefix: String,
    pub alias_count: u32,
    pub interval_secs: u64,
}

impl Default for OsIpRotatorConfig {
    fn default() -> Self {
        Self {
            auto_add_alias_ips: false,
            rotate_gateway: false,
            alias_prefix: "10.0.0".into(),
            alias_count: 10,
            interval_secs: ROTATION_INTERVAL_SECS,
        }
    }
}

pub struct OsIpRotator {
    config: RwLock<OsIpRotatorConfig>,
    running: AtomicBool,
    rotation_count: AtomicU64,
    current_alias_idx: AtomicU64,
    lan_router: RwLock<Option<Arc<LanRouter>>>,
}

impl OsIpRotator {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(OsIpRotatorConfig::default()),
            running: AtomicBool::new(false),
            rotation_count: AtomicU64::new(0),
            current_alias_idx: AtomicU64::new(0),
            lan_router: RwLock::new(None),
        }
    }

    pub fn with_lan_router(self, router: Arc<LanRouter>) -> Self {
        let s = self;
        *s.lan_router.try_write().expect("lan_router RwLock should not be poisoned") = Some(router);
        s
    }

    pub fn is_running(&self) -> bool { self.running.load(Ordering::Relaxed) }

    pub async fn set_config(&self, config: OsIpRotatorConfig) {
        *self.config.write().await = config;
    }

    fn find_primary_interface() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            let output = match std::process::Command::new("route")
                .args(["get", "default"]).output() {
                Ok(o) => o,
                Err(e) => {
                    log::warn!("[ip-rotator] run route get: {}", e);
                    return None;
                }
            };
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.trim().starts_with("interface:") {
                    return Some(line.split(':').nth(1)?.trim().to_string());
                }
            }
            None
        }
        #[cfg(target_os = "linux")]
        {
            let output = match std::process::Command::new("ip")
                .args(["route", "show", "default"]).output() {
                Ok(o) => o,
                Err(e) => {
                    log::warn!("[ip-rotator] run ip route: {}", e);
                    return None;
                }
            };
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.split_whitespace().nth(4).map(|s| s.to_string())
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        { None }
    }

    fn add_iface_alias(iface: &str, ip: &str, _prefix: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("ifconfig")
                .args([iface, "alias", ip, "netmask", "0xffffff00"])
                .output();
            match output {
                Ok(o) if o.status.success() => Ok(()),
                Ok(o) => Err(format!("ifconfig alias: {}", String::from_utf8_lossy(&o.stderr))),
                Err(e) => Err(format!("ifconfig: {}", e)),
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("ip")
                .args(["addr", "add", &format!("{}/{}", ip, prefix), "dev", iface])
                .output();
            match output {
                Ok(o) if o.status.success() => Ok(()),
                Ok(o) => Err(format!("ip addr: {}", String::from_utf8_lossy(&o.stderr))),
                Err(e) => Err(format!("ip: {}", e)),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        { Ok(()) }
    }

    fn remove_iface_alias(iface: &str, ip: &str, _prefix: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("ifconfig")
                .args([iface, "-alias", ip])
                .output();
            match output {
                Ok(o) if o.status.success() => Ok(()),
                _ => Ok(()),
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("ip")
                .args(["addr", "del", &format!("{}/{}", ip, prefix), "dev", iface])
                .output();
            match output {
                Ok(o) if o.status.success() => Ok(()),
                _ => Ok(()),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        { Ok(()) }
    }

    fn change_default_gateway(gateway: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("route")
                .args(["change", "default", gateway]).output();
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("ip")
                .args(["route", "replace", "default", "via", gateway]).output();
            match output {
                Ok(o) if o.status.success() => Ok(()),
                Ok(o) => Err(format!("route: {}", String::from_utf8_lossy(&o.stderr))),
                Err(e) => Err(format!("ip route: {}", e)),
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        { Ok(()) }
    }

    fn get_current_gateway() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            let output = match std::process::Command::new("route")
                .args(["get", "default"]).output() {
                Ok(o) => o,
                Err(e) => {
                    log::warn!("[ip-rotator] run route get: {}", e);
                    return None;
                }
            };
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.trim().starts_with("gateway:") {
                    return Some(line.split(':').nth(1)?.trim().to_string());
                }
            }
            None
        }
        #[cfg(target_os = "linux")]
        {
            let output = match std::process::Command::new("ip")
                .args(["route", "show", "default"]).output() {
                Ok(o) => o,
                Err(e) => {
                    log::warn!("[ip-rotator] run ip route: {}", e);
                    return None;
                }
            };
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.split_whitespace().nth(2).map(|s| s.to_string())
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        { None }
    }

    pub fn rotate_alias_ip(&self) -> Result<(String, String), String> {
        let cfg = loop {
            if let Ok(c) = self.config.try_read() { break c.clone(); }
        };
        let iface = Self::find_primary_interface().ok_or("No primary interface found")?;
        let current_idx = self.current_alias_idx.fetch_add(1, Ordering::Relaxed);
        let ip = format!("{}.{}", cfg.alias_prefix, (current_idx % cfg.alias_count as u64) + 2);
        let prefix = "24";

        let old_ip = format!("{}.{}", cfg.alias_prefix, current_idx % cfg.alias_count as u64 + 1);
        let _ = Self::remove_iface_alias(&iface, &old_ip, prefix);

        Self::add_iface_alias(&iface, &ip, prefix)?;
        self.rotation_count.fetch_add(1, Ordering::Relaxed);
        println!("[ip-rotator] rotated alias IP to {}/{} on {}", ip, prefix, iface);
        Ok((iface, ip))
    }

    pub fn rotate_gateway(&self) -> Result<(), String> {
        let current = Self::get_current_gateway().ok_or("No current gateway")?;
        let parts: Vec<&str> = current.split('.').collect();
        if parts.len() == 4 {
            let last: u8 = parts[3].parse().unwrap_or(1);
            let alt_last = if last == 1 { 254 } else { last - 1 };
            let alt_gateway = format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], alt_last);
            let _ = Self::change_default_gateway(&alt_gateway);
            let result = Self::change_default_gateway(&alt_gateway);
            if result.is_ok() {
                println!("[ip-rotator] rotated gateway: {} -> {}", current, alt_gateway);
            }
            return result;
        }
        Err(format!("Unparseable gateway: {}", current))
    }

    pub async fn start_rotation(self: Arc<Self>) {
        if self.running.swap(true, Ordering::Relaxed) { return; }
        let cfg = self.config.read().await.clone();
        println!("[ip-rotator] started (interval: {}s, alias: {})", cfg.interval_secs, cfg.auto_add_alias_ips);
        loop {
            sleep(Duration::from_secs(cfg.interval_secs)).await;
            if cfg.auto_add_alias_ips {
                let _ = self.rotate_alias_ip();
            }
            if cfg.rotate_gateway {
                let _ = self.rotate_gateway();
            }
        }
    }

    pub fn rotation_count(&self) -> u64 { self.rotation_count.load(Ordering::Relaxed) }

    pub async fn stats(&self) -> OsIpRotatorStats {
        let config = self.config.read().await.clone();
        OsIpRotatorStats {
            running: self.running.load(Ordering::Relaxed),
            rotation_count: self.rotation_count.load(Ordering::Relaxed),
            auto_add_alias_ips: config.auto_add_alias_ips,
            rotate_gateway: config.rotate_gateway,
            alias_prefix: config.alias_prefix.clone(),
            interval_secs: config.interval_secs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsIpRotatorStats {
    pub running: bool,
    pub rotation_count: u64,
    pub auto_add_alias_ips: bool,
    pub rotate_gateway: bool,
    pub alias_prefix: String,
    pub interval_secs: u64,
}

impl Default for OsIpRotator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_ip_rotator_creation() {
        let rotator = OsIpRotator::new();
        assert!(!rotator.is_running());
        assert_eq!(rotator.rotation_count(), 0);
    }

    #[test]
    fn test_default_config() {
        let cfg = OsIpRotatorConfig::default();
        assert_eq!(cfg.interval_secs, ROTATION_INTERVAL_SECS);
        assert!(!cfg.auto_add_alias_ips);
    }

    #[tokio::test]
    async fn test_config_update() {
        let rotator = OsIpRotator::new();
        let mut cfg = OsIpRotatorConfig::default();
        cfg.auto_add_alias_ips = true;
        rotator.set_config(cfg).await;
        let stats = rotator.stats().await;
        assert!(stats.auto_add_alias_ips);
    }
}
