use log;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::process::Command;

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 10
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 192 && o[1] == 168)
                || (o[0] == 127)
                || (o[0] == 169 && o[1] == 254)
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified() || v6.is_multicast(),
    }
}

const WIFI_TOGGLE_COOLDOWN: Duration = Duration::from_secs(300);

pub struct NetworkMonitor {
    last_wifi_toggle: Instant,
    consecutive_failures: u8,
    wifi_interface: String,
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self {
            last_wifi_toggle: Instant::now() - WIFI_TOGGLE_COOLDOWN,
            consecutive_failures: 0,
            wifi_interface: Self::detect_wifi_interface(),
        }
    }
}

impl NetworkMonitor {
    #[cfg(target_os = "macos")]
    fn detect_wifi_interface() -> String {
        for iface in &["en0", "en1"] {
            if let Ok(output) = std::process::Command::new("ifconfig").arg(iface).output() {
                let out = String::from_utf8_lossy(&output.stdout);
                if out.contains("airport") || out.contains("802.11") {
                    return iface.to_string();
                }
            }
        }
        "en0".to_string()
    }

    #[cfg(not(target_os = "macos"))]
    fn detect_wifi_interface() -> String {
        "wlan0".to_string()
    }

    #[cfg(target_os = "macos")]
    async fn flush_dns() {
        let _ = Command::new("dscacheutil")
            .arg("-flushcache")
            .output()
            .await;
        let _ = Command::new("killall")
            .args(["-HUP", "mDNSResponder"])
            .output()
            .await;
    }

    #[cfg(not(target_os = "macos"))]
    async fn flush_dns() {}

    #[cfg(target_os = "macos")]
    async fn suspend_shadowrocket() {
        let _ = Command::new("scutil")
            .args(["--nc", "stop", "Shadowrocket"])
            .output()
            .await;
        let _ = Command::new("networksetup")
            .args([
                "-setdnsservers",
                "Wi-Fi",
                "1.1.1.1",
                "8.8.8.8",
                "114.114.114.114",
            ])
            .output()
            .await;
        let output = Command::new("ifconfig").arg("utun4").output().await;
        if let Ok(o) = output {
            if String::from_utf8_lossy(&o.stdout).contains("198.18") {
                log::warn!("[network] Shadowrocket reconnected, brute-force kill");
                let _ = Command::new("sudo")
                    .args(["ifconfig", "utun4", "down"])
                    .output()
                    .await;
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    async fn suspend_shadowrocket() {}

    #[cfg(target_os = "macos")]
    // TODO(platform): restore_shadowrocket already gated, but networksetup references should be extracted to a helper
    pub async fn restore_shadowrocket() {
        let _ = Command::new("networksetup")
            .args(["-setdnsservers", "Wi-Fi", "Empty"])
            .output()
            .await;
    }

    #[cfg(not(target_os = "macos"))]
    pub async fn restore_shadowrocket() {}

    pub async fn check_dns_quality() -> bool {
        Self::check_dns_quality_cached(None).await
    }

    pub async fn check_dns_quality_cached(
        dns_cache: Option<&mut crate::core::nt_core_network::dns_cache::VsaDnsCache>,
    ) -> bool {
        let domain = "www.baidu.com";
        let addrs = if let Some(cache) = dns_cache {
            if let Some(ip) = cache.resolve(
                domain,
                crate::core::nt_core_network::dns_cache::AddressFamily::V4,
            ) {
                vec![SocketAddr::new(ip, 0)]
            } else {
                match tokio::net::lookup_host(format!("{}:80", domain)).await {
                    Ok(a) => {
                        let addrs: Vec<_> = a.collect();
                        if let Some(first) = addrs.first() {
                            cache.insert(
                                domain,
                                first.ip(),
                                crate::core::nt_core_network::dns_cache::AddressFamily::V4,
                            );
                        }
                        addrs
                    }
                    Err(_) => return false,
                }
            }
        } else {
            match tokio::net::lookup_host(format!("{}:80", domain)).await {
                Ok(a) => a.collect(),
                Err(_) => return false,
            }
        };
        for addr in &addrs {
            let ip = addr.ip();
            if is_private_ip(ip) || ip.is_loopback() || ip.to_string().starts_with("198.18.") {
                return false;
            }
        }
        true
    }

    pub async fn test_connectivity() -> bool {
        Self::test_connectivity_cached(None).await
    }

    pub async fn test_connectivity_cached(
        mut dns_cache: Option<&mut crate::core::nt_core_network::dns_cache::VsaDnsCache>,
    ) -> bool {
        let hosts = [
            ("www.baidu.com", 80u16),
            ("www.qq.com", 80),
            ("www.taobao.com", 80),
        ];
        for (host, port) in &hosts {
            let connect_str = if let Some(cache) = dns_cache.as_deref_mut() {
                if let Some(ip) = cache.resolve(
                    host,
                    crate::core::nt_core_network::dns_cache::AddressFamily::V4,
                ) {
                    format!("{}:{}", ip, port)
                } else {
                    format!("{}:{}", host, port)
                }
            } else {
                format!("{}:{}", host, port)
            };
            match tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(&connect_str))
                .await
            {
                Ok(Ok(_)) => return true,
                _ => continue,
            }
        }
        false
    }

    #[cfg(target_os = "macos")]
    async fn wifi_toggle(&mut self) {
        if self.last_wifi_toggle.elapsed() < WIFI_TOGGLE_COOLDOWN {
            log::info!("[network] wifi toggle skipped (cooldown 5m)");
            return;
        }
        let iface = &self.wifi_interface;
        log::info!("[network] wifi {} OFF...", iface);
        let _ = Command::new("sudo")
            .args(["ifconfig", iface, "down"])
            .output()
            .await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        log::info!("[network] wifi {} ON...", iface);
        let _ = Command::new("sudo")
            .args(["ifconfig", iface, "up"])
            .output()
            .await;
        self.last_wifi_toggle = Instant::now();
    }

    #[cfg(not(target_os = "macos"))]
    async fn wifi_toggle(&mut self) {
        log::info!("[network] wifi toggle not supported on this platform");
    }

    pub async fn tick(&mut self) {
        Self::flush_dns().await;
        Self::suspend_shadowrocket().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        Self::flush_dns().await;

        let dns_clean = Self::check_dns_quality().await;
        if !dns_clean {
            log::warn!("[network] DNS still poisoned, re-suspending Shadowrocket");
            Self::suspend_shadowrocket().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            Self::flush_dns().await;
        }

        let connected = Self::test_connectivity().await;
        if connected {
            self.consecutive_failures = 0;
        } else {
            self.consecutive_failures += 1;
            log::warn!(
                "[network] connectivity fail #{}/3",
                self.consecutive_failures
            );
            if self.consecutive_failures >= 3 {
                log::warn!("[network] 3 consecutive failures → wifi toggle");
                self.wifi_toggle().await;
                self.consecutive_failures = 0;
            }
        }
    }
}
