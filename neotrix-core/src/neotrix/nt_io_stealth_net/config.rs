//! 配置中心 — 从 ~/.neotrix/config.toml 加载所有可调参数
//!
//! 替代散落各处的硬编码常量。
//! 文件不存在时自动创建默认配置。

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

static INSTANCE: OnceLock<RwLock<Arc<NeoTrixConfig>>> = OnceLock::new();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeoTrixConfig {
    pub proxy: ProxyConfig,
    pub rotation: RotationConfig,
    pub tor: TorConfigSection,
    pub pool: PoolConfig,
    pub bandit: BanditConfig,
    pub nt_world_browse: BrowserConfig,
    #[serde(default)]
    pub firewall: FirewallConfigSection,
    #[serde(default)]
    pub rule_api: RuleApiConfigSection,
    #[serde(default)]
    pub ip_rotation: IpRotationConfigSection,
    #[serde(default)]
    pub transit: TransitConfigSection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FirewallConfigSection {
    pub enabled: bool,
    pub divert_to_port: u16,
    pub sync_interval_secs: u64,
    pub auto_apply_rules: bool,
}

impl Default for FirewallConfigSection {
    fn default() -> Self {
        Self {
            enabled: true,
            divert_to_port: 11081,
            sync_interval_secs: 15,
            auto_apply_rules: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuleApiConfigSection {
    pub enabled: bool,
    pub auto_start: bool,
    pub max_external_rules: u32,
}

impl Default for RuleApiConfigSection {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_start: true,
            max_external_rules: 100,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpRotationConfigSection {
    pub enabled: bool,
    pub wifi_toggle: bool,
    pub verify_external_ip: bool,
    pub external_ip_check_url: String,
    pub interval_secs: u64,
    pub toggle_cooldown_secs: u64,
    pub dhcp_wait_secs: u64,
}

impl Default for IpRotationConfigSection {
    fn default() -> Self {
        Self {
            enabled: false,
            wifi_toggle: true,
            verify_external_ip: true,
            external_ip_check_url: "https://api.ipify.org".into(),
            interval_secs: 30,
            toggle_cooldown_secs: 30,
            dhcp_wait_secs: 5,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransitConfigSection {
    pub enabled: bool,
    pub mode: String,
    pub listen_port: u16,
    pub per_conn_ip_rotation: bool,
    pub fingerprint_rotation_interval_secs: u64,
    pub padding_enabled: bool,
    pub timing_obfuscation_enabled: bool,
    pub max_concurrent_connections: usize,
}

impl Default for TransitConfigSection {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "pf_divert".into(),
            listen_port: 11081,
            per_conn_ip_rotation: true,
            fingerprint_rotation_interval_secs: 30,
            padding_enabled: true,
            timing_obfuscation_enabled: true,
            max_concurrent_connections: 1024,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxyConfig {
    pub local_port: u16,
    pub socks_port: u16,
    pub direct_timeout_secs: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RotationConfig {
    pub gaussian_mean_secs: f64,
    pub gaussian_std_dev_secs: f64,
    pub max_interval_secs: f64,
    pub min_interval_secs: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TorConfigSection {
    pub auto_start: bool,
    pub circuit_rotate_interval: u64,
    pub socks_addr: String,
    pub control_addr: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolConfig {
    pub min_nodes: u32,
    pub health_check_interval_secs: u64,
    /// 节点选择策略: auto / fastest / least_latency / least_failure / weighted_random / round_robin / adaptive / geo:JP
    #[serde(default = "default_selection_strategy")]
    pub selection_strategy: String,
    /// Tier 1 relay pool configuration
    #[serde(default)]
    pub tier1: TierPoolConfig,
    /// Tier 2 obfuscation pool configuration
    #[serde(default)]
    pub tier2: TierPoolConfig,
    /// Multi-hop chain count (default 3). Falls back automatically: N→2→1→Tor→direct
    #[serde(default = "default_multi_hop_count")]
    pub multi_hop_count: usize,
}

fn default_multi_hop_count() -> usize {
    3
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TierPoolConfig {
    pub enabled: bool,
    pub min_nodes: u32,
    pub max_latency_ms: f64,
    pub preferred_geo: Vec<String>,
}

impl Default for TierPoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_nodes: 3,
            max_latency_ms: 1000.0,
            preferred_geo: Vec::new(),
        }
    }
}

fn default_selection_strategy() -> String {
    "auto".into()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BanditConfig {
    pub persistence_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for NeoTrixConfig {
    fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                local_port: 11080,
                socks_port: crate::core::nt_core_util::TOR_SOCKS_PORT,
                direct_timeout_secs: 3,
            },
            rotation: RotationConfig {
                gaussian_mean_secs: 7.5,
                gaussian_std_dev_secs: 2.5,
                max_interval_secs: 9.0,
                min_interval_secs: 0.5,
            },
            tor: TorConfigSection {
                auto_start: true,
                circuit_rotate_interval: 300,
                socks_addr: crate::core::nt_core_util::TOR_SOCKS_ADDR.into(),
                control_addr: crate::core::nt_core_util::TOR_CONTROL_ADDR.into(),
            },
            pool: PoolConfig {
                min_nodes: 5,
                health_check_interval_secs: 60,
                selection_strategy: "auto".into(),
                tier1: TierPoolConfig::default(),
                tier2: TierPoolConfig::default(),
                multi_hop_count: 3,
            },
            bandit: BanditConfig {
                persistence_path: "~/.neotrix/bandit.json".into(),
            },
            nt_world_browse: BrowserConfig {
                headless: true,
                window_width: 1920,
                window_height: 1080,
            },
            firewall: FirewallConfigSection::default(),
            rule_api: RuleApiConfigSection::default(),
            ip_rotation: IpRotationConfigSection::default(),
            transit: TransitConfigSection::default(),
        }
    }
}

fn config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".neotrix").join("config.toml")
}

fn default_config_content() -> String {
    r#"# NeoTrix StealthNet Configuration
[proxy]
local_port = 11080
socks_port = 9050
direct_timeout_secs = 3

[rotation]
gaussian_mean_secs = 7.5
gaussian_std_dev_secs = 2.5
max_interval_secs = 9.0
min_interval_secs = 0.5

[tor]
auto_start = true
circuit_rotate_interval = 300
socks_addr = "127.0.0.1:9050"
control_addr = "127.0.0.1:9051"

[pool]
min_nodes = 5
health_check_interval_secs = 60
selection_strategy = "auto"

[pool.tier1]
enabled = true
min_nodes = 3
max_latency_ms = 1000.0
preferred_geo = []

[pool.tier2]
enabled = true
min_nodes = 3
max_latency_ms = 3000.0
preferred_geo = []

[bandit]
persistence_path = "~/.neotrix/bandit.json"

[nt_world_browse]
headless = true
window_width = 1920
window_height = 1080

[firewall]
enabled = true
divert_to_port = 11081
sync_interval_secs = 15
auto_apply_rules = true

[rule_api]
enabled = true
auto_start = true
max_external_rules = 100

[ip_rotation]
enabled = false
wifi_toggle = true
verify_external_ip = true
external_ip_check_url = "https://api.ipify.org"
interval_secs = 30
toggle_cooldown_secs = 30
dhcp_wait_secs = 5

[transit]
enabled = false
mode = "pf_divert"
listen_port = 11081
per_conn_ip_rotation = true
fingerprint_rotation_interval_secs = 30
padding_enabled = true
timing_obfuscation_enabled = true
max_concurrent_connections = 1024
"#
    .to_string()
}

/// 加载配置（惰性初始化）
pub fn load() -> Arc<NeoTrixConfig> {
    let lock = INSTANCE.get_or_init(|| RwLock::new(Arc::new(init_config())));
    lock.read().unwrap_or_else(|e| e.into_inner()).clone()
}

fn init_config() -> NeoTrixConfig {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if !path.exists() {
        let _ = fs::write(&path, default_config_content());
        log::info!("[config] created default at {:?}", path);
        return NeoTrixConfig::default();
    }
    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<NeoTrixConfig>(&content) {
            Ok(cfg) => {
                log::info!("[config] loaded from {:?}", path);
                cfg
            }
            Err(e) => {
                log::warn!("[config] parse error: {}, using defaults", e);
                NeoTrixConfig::default()
            }
        },
        Err(e) => {
            log::warn!("[config] read error: {}, using defaults", e);
            NeoTrixConfig::default()
        }
    }
}

/// 热重载配置 — 重新读取 config.toml 并原子替换
pub fn reload() -> Result<(), String> {
    let path = config_path();
    let content = fs::read_to_string(&path).map_err(|e| format!("read error: {}", e))?;
    let cfg: NeoTrixConfig = toml::from_str(&content).map_err(|e| format!("parse error: {}", e))?;
    let lock = INSTANCE.get_or_init(|| RwLock::new(Arc::new(NeoTrixConfig::default())));
    *lock.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(cfg);
    log::info!("[config] hot-reloaded from {:?}", path);
    Ok(())
}

/// 返回配置路径字符串（供外部日志/调试）
pub fn config_file_path() -> String {
    config_path().display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid_toml() {
        let content = default_config_content();
        let cfg: NeoTrixConfig =
            toml::from_str(&content).expect("default config should be valid toml");
        assert_eq!(cfg.proxy.local_port, 11080);
        assert_eq!(cfg.rotation.gaussian_mean_secs, 7.5);
        assert_eq!(cfg.pool.min_nodes, 5);
        assert!(cfg.nt_world_browse.headless);
    }

    #[test]
    fn test_config_all_sections_present() {
        let cfg = NeoTrixConfig::default();
        assert_eq!(cfg.proxy.local_port, 11080);
        assert_eq!(cfg.rotation.gaussian_std_dev_secs, 2.5);
        assert_eq!(cfg.tor.circuit_rotate_interval, 300);
        assert_eq!(cfg.bandit.persistence_path, "~/.neotrix/bandit.json");
        assert_eq!(cfg.pool.health_check_interval_secs, 60);
        assert!(cfg.pool.tier1.enabled);
        assert!(cfg.pool.tier2.enabled);
        assert_eq!(cfg.pool.tier1.min_nodes, 3);
        assert_eq!(cfg.pool.tier2.max_latency_ms, 3000.0);
    }

    #[test]
    fn test_config_path_is_valid() {
        let p = config_path();
        assert!(p.to_string_lossy().contains(".neotrix/config.toml"));
    }

    #[test]
    fn test_load_uses_default_when_no_file() {
        // 没有文件时不应 panic，返回默认
        let cfg = load();
        assert_eq!(cfg.proxy.local_port, 11080);
    }

    #[test]
    fn test_save_load_roundtrip() {
        use std::io::Write;
        let tmp = std::env::temp_dir().join("neotrix_test_config.toml");
        let content = default_config_content();
        let mut f = fs::File::create(&tmp).expect("failed to create temp config file");
        write!(f, "{}", content).expect("failed to write config content");

        let read_back = fs::read_to_string(&tmp).expect("failed to read back config file");
        let cfg: NeoTrixConfig =
            toml::from_str(&read_back).expect("read config should be valid toml");
        assert_eq!(cfg.proxy.local_port, 11080);
        assert_eq!(cfg.pool.min_nodes, 5);
        let _ = fs::remove_file(&tmp);
    }
}
