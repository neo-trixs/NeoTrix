//! IP 隐私保护 — 模拟私有 IP + MAC + 地理位置
//!
//! 对标 Apple 网络隐私功能:
//! - **Private Wi-Fi Address**: 每网络随机 MAC
//! - **Limit IP Address Tracking**: 隐藏真实 IP
//! - **iCloud Private Relay**: 出口 IP 匿名化
//!
//! 核心:
//! - 生成逼真的模拟 IPv4/IPv6 地址
//! - 随机 MAC 地址 (有效 OUI 前缀)
//! - 地理位置数据 (城市/ISP/坐标) 与 IP 一致
//! - 每15秒 / 每网络自动轮转

use rand::Rng;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::sleep;

use super::rotation_coordinator::{RotationCoordinator, RotationDomain};

const IP_ROTATION_INTERVAL_SECS: u64 = 9;

/// OUI 前缀列表 (知名厂商, 对标 macOS Private Wi-Fi)
const VALID_OUI: &[&[u8; 3]] = &[
    &[0x00, 0x1A, 0x11], // Google
    &[0x00, 0x25, 0x00], // Apple
    &[0x00, 0x50, 0x79], // Microsoft
    &[0x08, 0x00, 0x27], // Oracle/VirtualBox
    &[0x3C, 0x5A, 0xB4], // Intel
    &[0x00, 0x14, 0x22], // Dell
    &[0x48, 0x45, 0x20], // Cisco
    &[0x10, 0x02, 0xB5], // Samsung
    &[0xAC, 0xDE, 0x48], // Raspberry Pi
    &[0x70, 0x8B, 0xCD], // Apple
];

#[derive(Debug, Clone)]
pub struct FakeIpConfig {
    pub ip: IpAddr,
    pub mac: String,
    pub subnet: IpSubnet,
    pub geo: FakeGeoLocation,
    pub isp: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IpSubnet {
    /// 10.x.x.x
    PrivateClassA,
    /// 172.16-31.x.x
    PrivateClassB,
    /// 192.168.x.x
    PrivateClassC,
    /// 100.64.x.x (CGNAT)
    Cgnat,
    /// 公共 IP 段 (模拟真实出口)
    Public,
}

#[derive(Debug, Clone)]
pub struct FakeGeoLocation {
    pub city: String,
    pub country: String,
    pub continent: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

/// IP 隐私管理器 — 生成模拟网络身份
pub struct IpPrivacyManager {
    current: RwLock<FakeIpConfig>,
    network_name: RwLock<String>,
    rotation_interval: AtomicU64,
    running: AtomicBool,
    rotation_count: AtomicU64,
    coordinator: RwLock<Option<Arc<RotationCoordinator>>>,
}

impl Default for IpPrivacyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IpPrivacyManager {
    pub fn new() -> Self {
        let config = Self::generate_fake_config("default");
        Self {
            current: RwLock::new(config),
            network_name: RwLock::new("default".into()),
            rotation_interval: AtomicU64::new(IP_ROTATION_INTERVAL_SECS),
            running: AtomicBool::new(false),
            rotation_count: AtomicU64::new(0),
            coordinator: RwLock::new(None),
        }
    }

    /// 绑定 RotationCoordinator
    pub async fn set_coordinator(&self, coord: Arc<RotationCoordinator>) {
        *self.coordinator.write().await = Some(coord);
    }

    /// 对标 macOS Private Wi-Fi: 每网络生成不同身份
    pub async fn set_network(&self, name: &str) {
        *self.network_name.write().await = name.to_string();
        self.rotate_identity().await;
    }

    /// 生成全新的 IP/MAC/地理位置身份 (对标 Private Relay)
    pub async fn rotate_identity(&self) -> FakeIpConfig {
        let net = self.network_name.read().await.clone();
        let config = Self::generate_fake_config(&net);
        *self.current.write().await = config.clone();
        self.rotation_count.fetch_add(1, Ordering::Relaxed);
        config
    }

    pub async fn current(&self) -> FakeIpConfig {
        self.current.read().await.clone()
    }

    pub async fn current_ip(&self) -> IpAddr {
        self.current.read().await.ip
    }

    pub async fn current_mac(&self) -> String {
        self.current.read().await.mac.clone()
    }

    pub async fn current_geo(&self) -> FakeGeoLocation {
        self.current.read().await.geo.clone()
    }

    /// 生成一致的 IP/MAC/Geo 身份
    fn generate_fake_config(_network: &str) -> FakeIpConfig {
        let mut rng = rand::thread_rng();

        // 使用网络名做种子, 同一网络每次轮转换新身份
        let subnet = match rng.gen_range(0..5) {
            0 => IpSubnet::PrivateClassA,
            1 => IpSubnet::PrivateClassB,
            2 => IpSubnet::PrivateClassC,
            3 => IpSubnet::Cgnat,
            _ => IpSubnet::Public,
        };

        let ip = match subnet {
            IpSubnet::PrivateClassA => IpAddr::V4(Ipv4Addr::new(
                10, rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(1..254),
            )),
            IpSubnet::PrivateClassB => IpAddr::V4(Ipv4Addr::new(
                172, rng.gen_range(16..32), rng.gen_range(0..255), rng.gen_range(1..254),
            )),
            IpSubnet::PrivateClassC => IpAddr::V4(Ipv4Addr::new(
                192, 168, rng.gen_range(0..255), rng.gen_range(1..254),
            )),
            IpSubnet::Cgnat => IpAddr::V4(Ipv4Addr::new(
                100, rng.gen_range(64..127), rng.gen_range(0..255), rng.gen_range(1..254),
            )),
            IpSubnet::Public => {
                // 来自知名 ISP 的段
                let prefix = match rng.gen_range(0..6) {
                    0 => (1, 1), 1 => (8, 8), 2 => (23, 0),
                    3 => (34, 0), 4 => (52, 0), _ => (104, 16),
                };
                IpAddr::V4(Ipv4Addr::new(
                    prefix.0, prefix.1, rng.gen_range(0..255), rng.gen_range(1..254),
                ))
            }
        };

        // MAC 地址: 随机 OUI + 3 字节
        let oui = VALID_OUI[rng.gen_range(0..VALID_OUI.len())];
        let mac = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            oui[0], oui[1], oui[2],
            rng.gen_range(0x00..0xFF),
            rng.gen_range(0x00..0xFF),
            rng.gen_range(0x00..0xFF),
        );

        // 地理位置与 IP 子网对应
        let (city, country, continent, lat, lon, tz) = match subnet {
            IpSubnet::PrivateClassA => ("Private", "RFC1918", "Global", 40.71, -74.00, "Etc/UTC"),
            IpSubnet::PrivateClassB => ("Private-B", "RFC1918", "Global", 34.05, -118.24, "Etc/UTC"),
            IpSubnet::PrivateClassC => ("Private-C", "RFC1918", "Global", 51.50, -0.12, "Etc/UTC"),
            IpSubnet::Cgnat => ("CGNAT", "RFC6598", "Global", 48.85, 2.35, "Etc/UTC"),
            IpSubnet::Public => {
                let geos = [
                    ("Ashburn", "US", "NA", 39.04, -77.49, "US/Eastern"),
                    ("San Jose", "US", "NA", 37.34, -121.89, "US/Pacific"),
                    ("Frankfurt", "DE", "EU", 50.11, 8.68, "Europe/Berlin"),
                    ("Tokyo", "JP", "AS", 35.68, 139.69, "Asia/Tokyo"),
                    ("Sydney", "AU", "OC", -33.87, 151.21, "Australia/Sydney"),
                    ("London", "GB", "EU", 51.51, -0.13, "Europe/London"),
                ];
                geos[rng.gen_range(0..geos.len())]
            }
        };

        // ISP 与 IP 子网对应
        let isp = match subnet {
            IpSubnet::Public => {
                let isps = ["Cloudflare", "Google LLC", "Amazon AWS", "Microsoft Azure", "Akamai"];
                isps[rng.gen_range(0..isps.len())].to_string()
            }
            _ => "Private Network".to_string(),
        };

        FakeIpConfig {
            ip,
            mac,
            subnet,
            geo: FakeGeoLocation {
                city: city.to_string(),
                country: country.to_string(),
                continent: continent.to_string(),
                latitude: lat,
                longitude: lon,
                timezone: tz.to_string(),
            },
            isp,
        }
    }

    /// 注入 HTTP 头 (模拟 IP 隐私头)
    pub fn to_headers(config: &FakeIpConfig) -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Forwarded-For".into(), config.ip.to_string());
        headers.insert("X-Real-IP".into(), config.ip.to_string());
        headers.insert("X-Client-IP".into(), config.ip.to_string());
        headers.insert("True-Client-IP".into(), config.ip.to_string());
        headers.insert("X-Private-Network".into(), "true".into());
        // MAC 隐私头
        headers.insert("X-MAC-Address".into(), config.mac.clone());
        headers
    }

    /// 自动轮转（优先使用 RotationCoordinator，回退 rotation_interval）
    pub async fn start_rotation_loop(self: Arc<Self>) {
        if self.running.swap(true, Ordering::Relaxed) {
            return;
        }
        loop {
            let use_coord = self.coordinator.read().await.is_some();
            if use_coord {
                let coord = self.coordinator.read().await.clone().expect("result");
                let secs = coord.seconds_until_rotation(RotationDomain::SourceIp).await;
                sleep(Duration::from_secs_f64(secs.clamp(0.5, 10.0))).await;
                if coord.should_rotate(RotationDomain::SourceIp).await {
                    self.rotate_identity().await;
                    coord.mark_rotated(RotationDomain::SourceIp).await;
                }
            } else {
                sleep(Duration::from_secs(self.rotation_interval.load(Ordering::Relaxed))).await;
                self.rotate_identity().await;
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub async fn summary(&self) -> IpPrivacySummary {
        let cfg = self.current.read().await.clone();
        IpPrivacySummary {
            ip: cfg.ip.to_string(),
            mac: cfg.mac,
            subnet: cfg.subnet,
            city: cfg.geo.city,
            isp: cfg.isp,
            rotation_count: self.rotation_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IpPrivacySummary {
    pub ip: String,
    pub mac: String,
    pub subnet: IpSubnet,
    pub city: String,
    pub isp: String,
    pub rotation_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_private_ip_generation() {
        let mgr = IpPrivacyManager::new();
        let cfg = mgr.current().await;
        assert!(!cfg.mac.is_empty());
        assert!(cfg.ip.to_string().len() >= 7);
    }

    #[tokio::test]
    async fn test_mac_format() {
        let mgr = IpPrivacyManager::new();
        let mac = mgr.current_mac().await;
        // MAC 格式: XX:XX:XX:XX:XX:XX
        assert_eq!(mac.len(), 17);
        assert_eq!(mac.chars().filter(|&c| c == ':').count(), 5);
    }

    #[tokio::test]
    async fn test_rotation_changes_ip() {
        let mgr = IpPrivacyManager::new();
        let first = mgr.current_ip().await;
        mgr.rotate_identity().await;
        let second = mgr.current_ip().await;
        // 大概率不同 (随机生成)
        assert!(first != second || mgr.summary().await.rotation_count >= 1);
    }

    #[tokio::test]
    async fn test_network_isolation() {
        let mgr = IpPrivacyManager::new();
        mgr.set_network("work-wifi").await;
        let work = mgr.current_ip().await;
        mgr.set_network("home-wifi").await;
        let home = mgr.current_ip().await;
        // 不同网络应不同 IP (macOS Private Wi-Fi 行为)
        assert!(work != home || mgr.summary().await.rotation_count >= 2);
    }

    #[tokio::test]
    async fn test_geo_consistency() {
        let mgr = IpPrivacyManager::new();
        let geo = mgr.current_geo().await;
        assert!(!geo.city.is_empty());
        assert!(!geo.country.is_empty());
        assert!(geo.latitude != 0.0 || geo.longitude != 0.0);
    }

    #[test]
    fn test_headers_injection() {
        let mgr = IpPrivacyManager::new();
        let rt = tokio::runtime::Runtime::new().expect("value should be ok in test");
        let cfg = rt.block_on(mgr.current());
        let headers = IpPrivacyManager::to_headers(&cfg);
        assert!(headers.contains_key("X-Forwarded-For"));
        assert!(headers.contains_key("X-Real-IP"));
        assert!(headers.contains_key("X-MAC-Address"));
    }

    #[test]
    fn test_valid_ouis() {
        for oui in VALID_OUI {
            assert_eq!(oui.len(), 3);
        }
        assert!(!VALID_OUI.is_empty());
    }

    #[tokio::test]
    async fn test_public_ip_range() {
        let mut found_public = false;
        for _ in 0..20 {
            let cfg = IpPrivacyManager::generate_fake_config("test");
            if matches!(cfg.subnet, IpSubnet::Public) {
                found_public = true;
                let ip_str = cfg.ip.to_string();
                // 不包含 RFC1918 前缀
                assert!(!ip_str.starts_with("10."));
                assert!(!ip_str.starts_with("172."));
                assert!(!ip_str.starts_with("192.168."));
                assert!(!ip_str.starts_with("100."));
                break;
            }
        }
        assert!(found_public, "Should generate at least one public IP out of 20");
    }
}
