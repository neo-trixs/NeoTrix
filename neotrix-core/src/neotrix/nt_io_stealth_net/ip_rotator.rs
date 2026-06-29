use log;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::sleep;

const DEFAULT_EXTERNAL_IP_URL: &str = "https://api.ipify.org";
const DEFAULT_INTERVAL_SECS: u64 = 30;
const DEFAULT_DHCP_WAIT_SECS: u64 = 5;
const DEFAULT_COOLDOWN_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct OsIpRotatorConfig {
    pub wifi_toggle: bool,
    pub verify_external_ip: bool,
    pub external_ip_check_url: String,
    pub interval_secs: u64,
    pub toggle_cooldown_secs: u64,
    pub dhcp_wait_secs: u64,
}

impl Default for OsIpRotatorConfig {
    fn default() -> Self {
        Self {
            wifi_toggle: true,
            verify_external_ip: true,
            external_ip_check_url: DEFAULT_EXTERNAL_IP_URL.into(),
            interval_secs: DEFAULT_INTERVAL_SECS,
            toggle_cooldown_secs: DEFAULT_COOLDOWN_SECS,
            dhcp_wait_secs: DEFAULT_DHCP_WAIT_SECS,
        }
    }
}

pub struct OsIpRotator {
    config: RwLock<OsIpRotatorConfig>,
    running: AtomicBool,
    rotation_count: AtomicU64,
    last_toggle: RwLock<Instant>,
    last_external_ip: RwLock<String>,
}

// ── Cross-platform detection ──────────────────────────────

fn has_command(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .is_some_and(|o| o.status.success())
}

fn detect_wifi_iface() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        for iface in &["en0", "en1"] {
            if let Ok(output) = std::process::Command::new("ifconfig").arg(iface).output() {
                let out = String::from_utf8_lossy(&output.stdout);
                if out.contains("airport") || out.contains("802.11") {
                    return Some(iface.to_string());
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let wireless = entry.path().join("wireless");
                if wireless.exists() {
                    return Some(name.to_string_lossy().to_string());
                }
            }
        }
        if has_command("iwconfig") {
            if let Ok(output) = std::process::Command::new("iwconfig").output() {
                let out = String::from_utf8_lossy(&output.stdout);
                for line in out.lines() {
                    if let Some(name) = line.split_whitespace().next() {
                        if name != "lo" && !out.contains("no wireless") {
                            return Some(name.trim_end_matches(':').to_string());
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
        {
            let out = String::from_utf8_lossy(&output.stdout);
            for line in out.lines() {
                let t = line.trim();
                if let Some(name) = t.strip_prefix("Name") {
                    if let Some(val) = name.split(':').nth(1) {
                        return Some(val.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

fn detect_wifi_service_name() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let iface = detect_wifi_iface()?;
        if let Ok(output) = std::process::Command::new("networksetup")
            .args(["-listallhardwareports"])
            .output()
        {
            let out = String::from_utf8_lossy(&output.stdout);
            let mut current_port = String::new();
            for line in out.lines() {
                let t = line.trim();
                if let Some(port) = t.strip_prefix("Hardware Port: ") {
                    current_port = port.to_string();
                } else if t.contains(&format!("Device: {}", iface)) {
                    return Some(current_port);
                }
            }
        }
    }
    None
}

enum WifiMethod {
    Osascript(String),
    Networksetup(String),
    Nmcli,
    IpLink(String),
    Netsh(String),
}

fn detect_wifi_method() -> Option<WifiMethod> {
    #[cfg(target_os = "macos")]
    {
        if has_command("osascript") {
            if let Some(service) = detect_wifi_service_name() {
                return Some(WifiMethod::Osascript(service));
            }
        }
        if has_command("networksetup") {
            if let Some(iface) = detect_wifi_iface() {
                return Some(WifiMethod::Networksetup(iface));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if has_command("nmcli") {
            if let Ok(output) = std::process::Command::new("nmcli")
                .args(["-t", "-f", "TYPE,DEVICE", "device", "status"])
                .output()
            {
                let out = String::from_utf8_lossy(&output.stdout);
                if out.lines().any(|l| l.starts_with("wifi:")) {
                    return Some(WifiMethod::Nmcli);
                }
            }
        }
        if has_command("ip") {
            if let Some(iface) = detect_wifi_iface() {
                return Some(WifiMethod::IpLink(iface));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if has_command("netsh") {
            if let Some(name) = detect_wifi_iface() {
                return Some(WifiMethod::Netsh(name));
            }
        }
    }

    None
}

// TODO(platform): macOS-only (osascript), needs cfg(target_os = "macos") guard + Linux fallback
async fn toggle_via_osascript(service: &str, up: bool) -> Result<(), String> {
    let action = if up { "true" } else { "false" };
    let escaped = service.replace('"', r#"\""#);
    let script = format!(
        r#"tell application "System Events"
    tell current location of network preferences
        set s to service "{}"
        if s exists then set enabled of s to {}
    end tell
end tell"#,
        escaped, action
    );

    let output = Command::new("osascript")
        .args([("-e"), &script])
        .output()
        .await
        .map_err(|e| format!("osascript: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// TODO(platform): macOS-only (networksetup), needs cfg(target_os = "macos") guard + Linux fallback
async fn toggle_via_networksetup(iface: &str, up: bool) -> Result<(), String> {
    let action = if up { "on" } else { "off" };
    let output = Command::new("networksetup")
        .args(["-setairportpower", iface, action])
        .output()
        .await
        .map_err(|e| format!("networksetup: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn toggle_via_nmcli(up: bool) -> Result<(), String> {
    let action = if up { "on" } else { "off" };
    let output = Command::new("nmcli")
        .args(["radio", "wifi", action])
        .output()
        .await
        .map_err(|e| format!("nmcli: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn toggle_via_ip_link(iface: &str, up: bool) -> Result<(), String> {
    let action = if up { "up" } else { "down" };
    let output = Command::new("ip")
        .args(["link", "set", iface, action])
        .output()
        .await
        .map_err(|e| format!("ip: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn toggle_via_netsh(name: &str, up: bool) -> Result<(), String> {
    let action = if up { "enable" } else { "disable" };
    let output = Command::new("netsh")
        .args(["interface", "set", "interface", name, action])
        .output()
        .await
        .map_err(|e| format!("netsh: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn toggle_wifi(up: bool) -> Result<String, String> {
    let method = detect_wifi_method().ok_or_else(|| {
        "No supported Wi-Fi control tool found. Install osascript (macOS), \
         nmcli/ip (Linux), or netsh (Windows)"
            .to_string()
    })?;

    let label = match &method {
        WifiMethod::Osascript(s) => format!("osascript({})", s),
        WifiMethod::Networksetup(i) => format!("networksetup({})", i),
        WifiMethod::Nmcli => "nmcli".into(),
        WifiMethod::IpLink(i) => format!("ip({})", i),
        WifiMethod::Netsh(n) => format!("netsh({})", n),
    };

    let result = match &method {
        WifiMethod::Osascript(s) => toggle_via_osascript(s, up).await,
        WifiMethod::Networksetup(i) => toggle_via_networksetup(i, up).await,
        WifiMethod::Nmcli => toggle_via_nmcli(up).await,
        WifiMethod::IpLink(i) => toggle_via_ip_link(i, up).await,
        WifiMethod::Netsh(n) => toggle_via_netsh(n, up).await,
    };

    result.map(|_| label)
}

// ── Platform-specific extras ──────────────────────────────

async fn flush_dns() {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("dscacheutil")
            .arg("-flushcache")
            .output()
            .await;
        let _ = Command::new("killall")
            .args(["-HUP", "mDNSResponder"])
            .output()
            .await;
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemd-resolve")
            .args(["--flush-caches"])
            .output()
            .await;
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("ipconfig").arg("/flushdns").output().await;
    }
}

async fn suspend_vpn() {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("scutil")
            .args(["--nc", "stop", "Shadowrocket"])
            .output()
            .await;
    }
}

async fn flush_arp() {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("arp").args(["-a", "-d"]).output().await;
    }
}

// ── Connectivity & IP ─────────────────────────────────────

async fn wait_for_connectivity(timeout_secs: u64) -> bool {
    let start = Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        let hosts = [("1.1.1.1", 53u16), ("8.8.8.8", 53)];
        for (ip, port) in &hosts {
            if tokio::time::timeout(
                Duration::from_secs(2),
                tokio::net::TcpStream::connect((*ip, *port)),
            )
            .await
            .ok()
            .and_then(|r| r.ok())
            .is_some()
            {
                return true;
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    false
}

async fn get_external_ip(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .no_proxy()
        .build()
        .ok()?;
    let resp = client.get(url).send().await.ok()?;
    let text = resp.text().await.ok()?;
    let s = text.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

// ── OsIpRotator ───────────────────────────────────────────

impl OsIpRotator {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(OsIpRotatorConfig::default()),
            running: AtomicBool::new(false),
            rotation_count: AtomicU64::new(0),
            last_toggle: RwLock::new(Instant::now() - Duration::from_secs(99999)),
            last_external_ip: RwLock::new(String::new()),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub async fn set_config(&self, config: OsIpRotatorConfig) {
        *self.config.write().await = config;
    }

    pub async fn rotate_ip(&self) -> Result<String, String> {
        let cfg = self.config.read().await.clone();

        let elapsed = self.last_toggle.read().await.elapsed();
        if elapsed < Duration::from_secs(cfg.toggle_cooldown_secs) {
            return Err(format!(
                "Cooldown: {:.0}s remaining",
                cfg.toggle_cooldown_secs as f64 - elapsed.as_secs_f64()
            ));
        }

        let old_ip = if cfg.verify_external_ip {
            get_external_ip(&cfg.external_ip_check_url).await
        } else {
            None
        };

        suspend_vpn().await;

        log::info!("[ip-rotator] turning Wi-Fi OFF...");
        let off_label = toggle_wifi(false)
            .await
            .map_err(|e| format!("Failed to turn Wi-Fi off: {}", e))?;

        sleep(Duration::from_secs(2)).await;

        log::info!("[ip-rotator] turning Wi-Fi ON...");
        let _on_label = toggle_wifi(true)
            .await
            .map_err(|e| format!("Failed to turn Wi-Fi on: {}", e))?;

        log::info!("[ip-rotator] waiting for connectivity...");
        let connected = wait_for_connectivity(cfg.dhcp_wait_secs + 10).await;
        if !connected {
            return Err("Network did not recover after Wi-Fi toggle".into());
        }

        flush_dns().await;
        flush_arp().await;

        if cfg.verify_external_ip {
            if let Some(new_ip) = get_external_ip(&cfg.external_ip_check_url).await {
                let mut stored = self.last_external_ip.write().await;
                if let Some(ref old) = old_ip {
                    if old == &new_ip {
                        log::info!("[ip-rotator] IP unchanged: {} (still same)", new_ip);
                    } else {
                        log::info!("[ip-rotator] IP changed: {} → {}", old, new_ip);
                    }
                } else {
                    log::info!("[ip-rotator] external IP: {}", new_ip);
                }
                *stored = new_ip;
            } else {
                log::warn!("[ip-rotator] could not verify external IP");
            }
        }

        *self.last_toggle.write().await = Instant::now();
        self.rotation_count.fetch_add(1, Ordering::Relaxed);
        log::info!(
            "[ip-rotator] rotation #{} complete via {}",
            self.rotation_count(),
            off_label
        );

        Ok(off_label)
    }

    pub async fn start_rotation(self: Arc<Self>) {
        if self.running.swap(true, Ordering::AcqRel) {
            return;
        }
        let cfg = self.config.read().await.clone();
        log::info!(
            "[ip-rotator] started (interval: {}s, wifi_toggle: {})",
            cfg.interval_secs,
            cfg.wifi_toggle
        );

        loop {
            if !self.running.load(Ordering::Acquire) {
                break;
            }
            sleep(Duration::from_secs(cfg.interval_secs)).await;

            if !self.config.read().await.wifi_toggle {
                continue;
            }

            let _ = self.rotate_ip().await;
        }
    }

    pub fn rotation_count(&self) -> u64 {
        self.rotation_count.load(Ordering::Relaxed)
    }

    pub async fn get_last_external_ip(&self) -> String {
        self.last_external_ip.read().await.clone()
    }

    pub async fn seconds_since_last_toggle(&self) -> u64 {
        self.last_toggle.read().await.elapsed().as_secs()
    }

    pub async fn stats(&self) -> OsIpRotatorStats {
        let config = self.config.read().await.clone();
        let last_ip = self.last_external_ip.read().await.clone();
        let last_toggle_ago = self.last_toggle.read().await.elapsed().as_secs();
        OsIpRotatorStats {
            running: self.running.load(Ordering::Acquire),
            rotation_count: self.rotation_count.load(Ordering::Relaxed),
            wifi_toggle: config.wifi_toggle,
            interval_secs: config.interval_secs,
            current_external_ip: last_ip,
            last_toggle_ago_secs: last_toggle_ago,
        }
    }
}

impl Default for OsIpRotator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct OsIpRotatorStats {
    pub running: bool,
    pub rotation_count: u64,
    pub wifi_toggle: bool,
    pub interval_secs: u64,
    pub current_external_ip: String,
    pub last_toggle_ago_secs: u64,
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
        assert!(cfg.wifi_toggle);
        assert!(cfg.verify_external_ip);
        assert_eq!(cfg.interval_secs, DEFAULT_INTERVAL_SECS);
        assert_eq!(cfg.dhcp_wait_secs, DEFAULT_DHCP_WAIT_SECS);
        assert_eq!(cfg.toggle_cooldown_secs, DEFAULT_COOLDOWN_SECS);
    }

    #[tokio::test]
    async fn test_config_update() {
        let rotator = OsIpRotator::new();
        let mut cfg = OsIpRotatorConfig::default();
        cfg.wifi_toggle = false;
        cfg.verify_external_ip = false;
        rotator.set_config(cfg).await;
        let stats = rotator.stats().await;
        assert!(!stats.wifi_toggle);
    }

    #[tokio::test]
    async fn test_last_external_ip_default() {
        let rotator = OsIpRotator::new();
        let ip = rotator.get_last_external_ip().await;
        assert_eq!(ip, "");
    }

    #[tokio::test]
    async fn test_seconds_since_last_toggle() {
        let rotator = OsIpRotator::new();
        let secs = rotator.seconds_since_last_toggle().await;
        assert!(secs > 0);
    }

    #[test]
    fn test_has_command_known() {
        #[cfg(target_os = "macos")]
        assert!(has_command("osascript"));
        #[cfg(target_os = "linux")]
        assert!(has_command("sh"));
    }

    #[test]
    fn test_has_command_unknown() {
        assert!(!has_command("nonexistent_xyz_123"));
    }
}
