use std::net::IpAddr;
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

    async fn flush_dns() {
        let _ = Command::new("dscacheutil").arg("-flushcache").output().await;
        let _ = Command::new("killall").args(["-HUP", "mDNSResponder"]).output().await;
    }

    async fn suspend_shadowrocket() {
        // Per "intelligent lifecycle" principle: suspend Shadowrocket cleanly
        // instead of brute-force fighting its TUN interface
        let _ = Command::new("scutil")
            .args(["--nc", "stop", "Shadowrocket"])
            .output()
            .await;
        // Set DNS to real servers on Wi-Fi (NOT "Shadowrocket" — that's the app name)
        let _ = Command::new("networksetup")
            .args(["-setdnsservers", "Wi-Fi", "1.1.1.1", "8.8.8.8", "114.114.114.114"])
            .output()
            .await;
        // Also try Ethernet interface names in case user is wired
        let output = Command::new("ifconfig").arg("utun4").output().await;
        if let Ok(o) = output {
            if String::from_utf8_lossy(&o.stdout).contains("198.18") {
                eprintln!("[network] Shadowrocket reconnected, brute-force kill");
                let _ = Command::new("sudo")
                    .args(["ifconfig", "utun4", "down"])
                    .output()
                    .await;
            }
        }
    }

    pub async fn restore_shadowrocket() {
        let _ = Command::new("networksetup")
            .args(["-setdnsservers", "Wi-Fi", "Empty"])
            .output()
            .await;
    }

    pub async fn check_dns_quality() -> bool {
        let addrs = match tokio::net::lookup_host("www.baidu.com:80").await {
            Ok(a) => a,
            Err(_) => return false,
        };
        for addr in addrs {
            let ip = addr.ip();
            if is_private_ip(ip) || ip.is_loopback() || ip.to_string().starts_with("198.18.") {
                return false;
            }
        }
        true
    }

    pub async fn test_connectivity() -> bool {
        let hosts = [("www.baidu.com", 80u16), ("www.qq.com", 80), ("www.taobao.com", 80)];
        for (host, port) in &hosts {
            match tokio::time::timeout(
                Duration::from_secs(3),
                TcpStream::connect(format!("{}:{}", host, port)),
            )
            .await
            {
                Ok(Ok(_)) => return true,
                _ => continue,
            }
        }
        false
    }

    async fn wifi_toggle(&mut self) {
        if self.last_wifi_toggle.elapsed() < WIFI_TOGGLE_COOLDOWN {
            eprintln!("[network] wifi toggle skipped (cooldown 5m)");
            return;
        }
        let iface = &self.wifi_interface;
        eprintln!("[network] wifi {} OFF...", iface);
        let _ = Command::new("sudo")
            .args(["ifconfig", iface, "down"])
            .output()
            .await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        eprintln!("[network] wifi {} ON...", iface);
        let _ = Command::new("sudo")
            .args(["ifconfig", iface, "up"])
            .output()
            .await;
        self.last_wifi_toggle = Instant::now();
    }

    pub async fn tick(&mut self) {
        Self::flush_dns().await;
        Self::suspend_shadowrocket().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        Self::flush_dns().await;

        let dns_clean = Self::check_dns_quality().await;
        if !dns_clean {
            eprintln!("[network] DNS still poisoned, re-suspending Shadowrocket");
            Self::suspend_shadowrocket().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            Self::flush_dns().await;
        }

        let connected = Self::test_connectivity().await;
        if connected {
            self.consecutive_failures = 0;
        } else {
            self.consecutive_failures += 1;
            eprintln!(
                "[network] connectivity fail #{}/3",
                self.consecutive_failures
            );
            if self.consecutive_failures >= 3 {
                eprintln!("[network] 3 consecutive failures → wifi toggle");
                self.wifi_toggle().await;
                self.consecutive_failures = 0;
            }
        }
    }
}
