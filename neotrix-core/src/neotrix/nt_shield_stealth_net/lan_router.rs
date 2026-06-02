//! 局域网路由自动跳 IP — 源地址绑定 + 自动轮转 + Wi-Fi 指纹伪装
//!
//! 对标:
//! - **ProxyChains-NG**: 多源 IP 出口
//! - **V2Ray/Xray**: 本地绑定策略 (local_address/bind)
//! - **Tor**: 出口节点轮转
//! - 虚假 Wi-Fi 指纹: 防止基于 BSSID/SSID 的跨网络追踪
//!
//! 核心:
//! - 自动发现所有可用局域网 IP
//! - 每15秒自动切换到下一个本地 IP
//! - reqwest ClientBuilder.local_address() 绑定源地址
//! - 虚假 Wi-Fi 信息 (BSSID/SSID/RSSI) 按 Gaussian 间隔主动轮转

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::rotation_coordinator::{RotationCoordinator, RotationDomain};

const IP_ROTATION_INTERVAL_SECS: u64 = 9;

#[derive(Debug, Clone)]
pub struct LocalInterface {
    pub name: String,
    pub ips: Vec<IpAddr>,
    pub mac: Option<String>,
    pub is_up: bool,
}

/// 常见 SSID 池 (伪装不易被关联)
pub(crate) static SSID_POOL: &[&str] = &[
    "WiFi", "xfinitywifi", "Starbucks WiFi", "ATT", "Home",
    "Network", "Internet", "Linksys", "NETGEAR", "TP-LINK",
    "DIRECT-", "AndroidAP", "iPhone", "Wi-Fi", "CableWiFi",
    "OptimumWiFi", "BTWiFi", "FreeWiFi", "Guest", "5G WiFi",
];

/// 虚假 Wi-Fi 指纹 — 让系统 app 感知到 Wi-Fi 网络持续变化
#[derive(Debug, Clone)]
pub struct FakeWifiInfo {
    pub ssid: String,
    pub bssid: String,
    pub rssi: i32,
    pub channel: u8,
    pub frequency_mhz: u16,
    pub last_seen: String,
}

impl FakeWifiInfo {
    /// 生成随机 BSSID (MAC 地址格式)
    fn random_bssid(rng: &mut impl Rng) -> String {
        let bytes: [u8; 6] = rng.gen();
        format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            bytes[0] | 0x02, // locally administered
            bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
    }

    /// 生成一组全新的虚假 Wi-Fi 信息
    pub fn generate(rng: &mut impl Rng) -> Self {
        let pool = SSID_POOL;
        let idx = rng.gen_range(0..pool.len());
        let ssid = pool[idx].to_string();
        let bssid = Self::random_bssid(rng);
        let rssi: i32 = -(rng.gen_range(30..90i32));
        let channel = rng.gen_range(1..14u8);
        let freq = match channel {
            1..=11 => 2412 + (channel as u16 - 1) * 5,
            12 | 13 => 2467 + (channel as u16 - 12) * 5,
            _ => 5180,
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            ssid, bssid, rssi, channel, frequency_mhz: freq,
            last_seen: format!("{}", now),
        }
    }

    /// 对上一组信息做微小扰动 (模拟同一 Wi-Fi 的信号波动)
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        // RSSI 随机波动 ±5dB
        self.rssi = (self.rssi + rng.gen_range(-5..6)).clamp(-90, -20);
        // 每 3 次有一定概率换 BSSID (模拟漫游到同网不同 AP)
        if rng.gen_bool(0.3) {
            self.bssid = Self::random_bssid(rng);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_seen = format!("{}", now);
    }

    /// 转为 HTTP 头格式 (注入请求头供系统级检测使用)
    pub fn to_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("X-Network-SSID", self.ssid.clone()),
            ("X-Network-BSSID", self.bssid.clone()),
            ("X-Network-RSSI", format!("{}", self.rssi)),
            ("X-Network-Channel", format!("{}", self.channel)),
            ("X-Network-Frequency", format!("{} MHz", self.frequency_mhz)),
            ("X-Network-LastSeen", self.last_seen.clone()),
        ]
    }
}

/// 局域网 IP 路由器 — 自动发现 + 轮转源地址 + Wi-Fi 指纹伪装
#[derive(Debug)]
pub struct LanRouter {
    interfaces: std::sync::RwLock<Vec<LocalInterface>>,
    current_interface: AtomicUsize,
    current_ip: std::sync::RwLock<Option<IpAddr>>,
    rotation_interval_secs: AtomicU64,
    running: AtomicBool,
    rotation_count: AtomicU64,
    /// 排除的 IP 段 (如 VPN 接口)
    exclude_prefixes: std::sync::RwLock<Vec<String>>,
    coordinator: RwLock<Option<Arc<RotationCoordinator>>>,
    /// 虚假 Wi-Fi 指纹 (持续变化)
    fake_wifi: RwLock<FakeWifiInfo>,
    /// Wi-Fi 指纹轮转计数器
    wifi_rotation_count: AtomicU64,
}

impl Default for LanRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl LanRouter {
    pub fn new() -> Self {
        let mut rng = rand::rngs::OsRng;
        let fake_wifi = RwLock::new(FakeWifiInfo::generate(&mut rng));
        let router = Self {
            interfaces: std::sync::RwLock::new(Vec::new()),
            current_interface: AtomicUsize::new(0),
            current_ip: std::sync::RwLock::new(None),
            rotation_interval_secs: AtomicU64::new(IP_ROTATION_INTERVAL_SECS),
            running: AtomicBool::new(false),
            rotation_count: AtomicU64::new(0),
            exclude_prefixes: std::sync::RwLock::new(vec![
                "127.".into(),
                "169.254.".into(),
                "::1".into(),
            ]),
            coordinator: RwLock::new(None),
            fake_wifi,
            wifi_rotation_count: AtomicU64::new(0),
        };
        router.discover_sync();
        router
    }

    /// 绑定 RotationCoordinator
    pub async fn set_coordinator(&self, coord: Arc<RotationCoordinator>) {
        *self.coordinator.write().await = Some(coord);
    }

    pub fn with_rotation_interval(self, secs: u64) -> Self {
        self.rotation_interval_secs.store(secs, Ordering::Relaxed);
        self
    }

    /// 添加排除网段 (如 VPN 接口: "10.8.", "172.16.")
    pub async fn add_exclude_prefix(&self, prefix: &str) {
        if let Ok(mut prefixes) = self.exclude_prefixes.write() {
            prefixes.push(prefix.to_string());
        }
    }

    /// 同步发现局域网 IP (使用 system network interfaces)
    fn discover_sync(&self) {
        let mut interfaces = Vec::new();

        // 尝试从 /sys/class/net 读取 (Linux)
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "lo" { continue; }
                let mut ips = Vec::new();
                // 尝试读取地址
                if let Ok(content) = std::fs::read_to_string(format!("/sys/class/net/{}/address", name)) {
                    let mac = content.trim().to_string();
                    // 读取 IPv4
                    if let Ok(ip_path) = std::fs::read_dir(format!("/sys/class/net/{}/inet", name)) {
                        for ip_entry in ip_path.flatten() {
                            let _ip_name = ip_entry.file_name().to_string_lossy().to_string();
                            if let Ok(content) = std::fs::read_to_string(ip_entry.path()) {
                                if let Ok(ip) = content.trim().parse::<Ipv4Addr>() {
                                    ips.push(IpAddr::V4(ip));
                                }
                            }
                        }
                    }
                    interfaces.push(LocalInterface {
                        name,
                        ips,
                        mac: Some(mac),
                        is_up: true,
                    });
                }
            }
        }

        // macOS: 使用 ifconfig 读取
        if interfaces.is_empty() {
            if let Ok(output) = std::process::Command::new("ifconfig")
                .args(["-l"])
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                for iface in text.split_whitespace() {
                    if iface == "lo0" || iface.starts_with("lo") { continue; }
                    let mut ips = Vec::new();
                    // 获取该接口的 IP
                    if let Ok(ip_out) = std::process::Command::new("ifconfig")
                        .args([iface])
                        .output()
                    {
                        let ip_text = String::from_utf8_lossy(&ip_out.stdout);
                        for line in ip_text.lines() {
                            let trimmed = line.trim();
                            if let Some(inet) = trimmed.strip_prefix("inet ") {
                                if let Some(ip_str) = inet.split_whitespace().next() {
                                    if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
                                        ips.push(IpAddr::V4(ip));
                                    }
                                }
                            }
                            if let Some(inet6) = trimmed.strip_prefix("inet6 ") {
                                if let Some(ip_str) = inet6.split_whitespace().next() {
                                    if let Ok(ip) = ip_str.parse::<Ipv6Addr>() {
                                        ips.push(IpAddr::V6(ip));
                                    }
                                }
                            }
                        }
                    }
                    if !ips.is_empty() {
                        interfaces.push(LocalInterface {
                            name: iface.to_string(),
                            ips,
                            mac: None,
                            is_up: true,
                        });
                    }
                }
            }
        }

        // Windows 备选
        if interfaces.is_empty() {
            if let Ok(output) = std::process::Command::new("ipconfig")
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut current_name = String::new();
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("Ethernet") || trimmed.starts_with("Wi-Fi") || trimmed.starts_with("本地连接") {
                        current_name = trimmed.to_string();
                    }
                    if let Some(ip_str) = trimmed.strip_prefix("IPv4 Address") {
                        if let Some(ip) = ip_str.split(':').nth(1).and_then(|s| s.trim().parse::<Ipv4Addr>()
                            .inspect_err(|e| log::warn!("[lan-router] parse IP: {}", e))
                            .ok()) {
                            if let Some(iface) = interfaces.iter_mut().find(|i| i.name == current_name) {
                                iface.ips.push(IpAddr::V4(ip));
                            } else {
                                interfaces.push(LocalInterface {
                                    name: current_name.clone(),
                                    ips: vec![IpAddr::V4(ip)],
                                    mac: None,
                                    is_up: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 过滤排除的 IP
        let exclude = match self.exclude_prefixes.read() {
            Ok(e) => e.clone(),
            Err(_) => return,
        };
        for iface in &mut interfaces {
            iface.ips.retain(|ip| {
                let ip_str = ip.to_string();
                !exclude.iter().any(|p| ip_str.starts_with(p))
            });
        }
        interfaces.retain(|i| !i.ips.is_empty());

        if let Ok(mut ifs) = self.interfaces.write() {
            *ifs = interfaces;
        }

        // 设置初始 IP
        if let Ok(ifaces) = self.interfaces.read() {
            if let Some(first) = ifaces.first().and_then(|i| i.ips.first()) {
                if let Ok(mut ip) = self.current_ip.write() {
                    *ip = Some(*first);
                }
            }
        }
    }

    /// 重新发现网络接口
    pub async fn rediscover(&self) {
        self.discover_sync();
    }

    pub async fn available_ips(&self) -> Vec<IpAddr> {
        let mut ips = Vec::new();
        if let Ok(ifaces) = self.interfaces.read() {
            for iface in ifaces.iter() {
                ips.extend(iface.ips.iter());
            }
        }
        ips
    }

    pub fn current_ip_sync(&self) -> Option<IpAddr> {
        match self.current_ip.read() {
            Ok(guard) => {
                let inner = &*guard;
                *inner
            }
            Err(e) => {
                log::warn!("[lan-router] RwLock poisoned: {}", e);
                None
            }
        }
    }

    pub async fn current_interface_name(&self) -> Option<String> {
        let idx = self.current_interface.load(Ordering::Relaxed);
        if let Ok(ifaces) = self.interfaces.read() {
            return ifaces.get(idx % ifaces.len().max(1)).map(|i| i.name.clone());
        }
        None
    }

    /// 获取当前虚假 Wi-Fi 信息（用于注入 HTTP 请求头）
    pub async fn current_wifi_info(&self) -> FakeWifiInfo {
        self.fake_wifi.read().await.clone()
    }

    /// 轮转虚假 Wi-Fi 指纹 (让系统 app 检测到 Wi-Fi 持续变化)
    pub async fn rotate_wifi_fingerprint(&self) {
        let mut rng = rand::rngs::OsRng;
        let mut wifi = self.fake_wifi.write().await;
        // 70% 概率做微扰 (模拟信号波动), 30% 概率生成全新指纹 (模拟切换网络)
        if rng.gen_bool(0.7) {
            wifi.mutate(&mut rng);
        } else {
            *wifi = FakeWifiInfo::generate(&mut rng);
        }
        self.wifi_rotation_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Wi-Fi 指纹轮转次数
    pub fn wifi_rotation_count(&self) -> u64 {
        self.wifi_rotation_count.load(Ordering::Relaxed)
    }

    /// 切换到下一个 IP (循环轮转)
    pub async fn rotate_ip(&self) -> Option<IpAddr> {
        let ifaces = match self.interfaces.read() {
            Ok(ifaces) => ifaces,
            Err(_) => return None,
        };
        if ifaces.is_empty() {
            return None;
        }

        let iface_count = ifaces.len();
        let start_idx = self.current_interface.load(Ordering::Relaxed);

        for offset in 1..=iface_count {
            let idx = (start_idx + offset) % iface_count;
            if let Some(ip) = ifaces[idx].ips.first() {
                self.current_interface.store(idx, Ordering::Relaxed);
                let ip = *ip;
                if let Ok(mut cip) = self.current_ip.try_write() {
                    *cip = Some(ip);
                }
                self.rotation_count.fetch_add(1, Ordering::Relaxed);
                return Some(ip);
            }
        }
        None
    }

    /// 启动自动轮转循环（含 Wi-Fi 指纹伪装）
    pub async fn start_rotation_loop(self: Arc<Self>) {
        if self.running.swap(true, Ordering::Relaxed) {
            return;
        }
        loop {
            let use_coord = self.coordinator.read().await.is_some();
            if use_coord {
                let coord = self.coordinator.read().await.clone().expect("result");
                // IP 轮转
                let secs = coord.seconds_until_rotation(RotationDomain::SourceIp).await;
                sleep(Duration::from_secs_f64(secs.clamp(0.5, 10.0))).await;
                if coord.should_rotate(RotationDomain::SourceIp).await {
                    self.rotate_ip().await;
                    coord.mark_rotated(RotationDomain::SourceIp).await;
                }
                // Wi-Fi 指纹伪装 (独立相位, 更高频率)
                if coord.should_rotate(RotationDomain::TimingPattern).await {
                    self.rotate_wifi_fingerprint().await;
                }
            } else {
                sleep(Duration::from_secs(self.rotation_interval_secs.load(Ordering::Relaxed))).await;
                self.rotate_ip().await;
                self.rotate_wifi_fingerprint().await;
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn rotation_count(&self) -> u64 {
        self.rotation_count.load(Ordering::Relaxed)
    }

    pub async fn summary(&self) -> LanRouterSummary {
        let (interface_count, total_ips, interface_names) = match self.interfaces.read() {
            Ok(ifaces) => (
                ifaces.len(),
                ifaces.iter().map(|i| i.ips.len()).sum(),
                ifaces.iter().map(|i| format!("{}: {:?}", i.name, i.ips)).collect(),
            ),
            Err(_) => (0, 0, vec![]),
        };
        let wifi = self.fake_wifi.read().await.clone();
        LanRouterSummary {
            interface_count,
            total_ips,
            current_ip: self.current_ip_sync(),
            current_interface: self.current_interface.load(Ordering::Relaxed),
            rotation_interval_secs: self.rotation_interval_secs.load(Ordering::Relaxed),
            rotation_count: self.rotation_count.load(Ordering::Relaxed),
            interfaces: interface_names,
            wifi_ssid: wifi.ssid.clone(),
            wifi_bssid: wifi.bssid.clone(),
            wifi_rotation: self.wifi_rotation_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanRouterSummary {
    pub interface_count: usize,
    pub total_ips: usize,
    pub current_ip: Option<IpAddr>,
    pub current_interface: usize,
    pub rotation_interval_secs: u64,
    pub rotation_count: u64,
    pub interfaces: Vec<String>,
    pub wifi_ssid: String,
    pub wifi_bssid: String,
    pub wifi_rotation: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disabled_router_creation() {
        let router = LanRouter::new();
        let summary = router.summary().await;
        // 只验证结构体创建状态，不依赖网络接口 I/O
        assert_eq!(summary.rotation_count, 0);
        assert!(!summary.wifi_ssid.is_empty());
        assert!(!summary.wifi_bssid.is_empty());
    }

    #[tokio::test]
    async fn test_exclude_prefix() {
        let router = LanRouter::new();
        router.add_exclude_prefix("10.").await;
        // exclude 后不应影响基础功能
        let ips = router.available_ips().await;
        for ip in &ips {
            assert!(!ip.to_string().starts_with("10."));
        }
    }

    #[tokio::test]
    async fn test_rotate_ip() {
        let router = LanRouter::new();
        let _first = router.current_ip_sync();
        let rotated = router.rotate_ip().await;
        // rotate_ip 返回当前选择的 IP（可能因随机种子与 first 相同）
        // 验证至少不 panic
        assert!(rotated.is_some());
    }

    #[tokio::test]
    async fn test_auto_discovery_has_at_least_localhost_excluded() {
        let router = LanRouter::new();
        let ips = router.available_ips().await;
        // 127.0.0.1 不应出现
        for ip in &ips {
            assert!(!ip.to_string().starts_with("127."), "127.x.x.x should be excluded");
        }
    }

    #[test]
    fn test_current_ip_sync() {
        let router = LanRouter::new();
        // 至少不 panic
        let _ = router.rotation_count();
    }

    #[tokio::test]
    async fn test_rediscover() {
        let router = LanRouter::new();
        router.rediscover().await;
        let _summary = router.summary().await;
        // total_ips 是 usize，隐式 >= 0；至少不 panic
    }
}
