//! 配置中心 — 从 ~/.neotrix/config.toml 加载所有可调参数
//!
//! 替代散落各处的硬编码常量。
//! 文件不存在时自动创建默认配置。

use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock, Arc};

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
        Self { enabled: true, auto_start: true, max_external_rules: 100 }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpRotationConfigSection {
    pub enabled: bool,
    pub auto_add_alias_ips: bool,
    pub rotate_gateway: bool,
    pub alias_prefix: String,
    pub interval_secs: u64,
}

impl Default for IpRotationConfigSection {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_add_alias_ips: false,
            rotate_gateway: false,
            alias_prefix: "10.0.0".into(),
            interval_secs: 30,
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
}

fn default_selection_strategy() -> String { "auto".into() }

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
                socks_port: 9050,
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
                socks_addr: "127.0.0.1:9050".into(),
                control_addr: "127.0.0.1:9051".into(),
            },
            pool: PoolConfig {
                min_nodes: 5,
                health_check_interval_secs: 60,
                selection_strategy: "auto".into(),
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
auto_add_alias_ips = false
rotate_gateway = false
alias_prefix = "10.0.0"
interval_secs = 30
"#.to_string()
}

/// 加载配置（惰性初始化）
pub fn load() -> Arc<NeoTrixConfig> {
    let lock = INSTANCE.get_or_init(|| {
        RwLock::new(Arc::new(init_config()))
    });
    lock.read().expect("config lock poisoned").clone()
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
        Ok(content) => {
            match toml::from_str::<NeoTrixConfig>(&content) {
                Ok(cfg) => {
                    log::info!("[config] loaded from {:?}", path);
                    cfg
                }
                Err(e) => {
                    log::warn!("[config] parse error: {}, using defaults", e);
                    NeoTrixConfig::default()
                }
            }
        }
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
    let lock = INSTANCE.get_or_init(|| {
        RwLock::new(Arc::new(NeoTrixConfig::default()))
    });
    *lock.write().expect("config lock poisoned") = Arc::new(cfg);
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
        let cfg: NeoTrixConfig = toml::from_str(&content).expect("default config should be valid toml");
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
        let cfg: NeoTrixConfig = toml::from_str(&read_back).expect("read config should be valid toml");
        assert_eq!(cfg.proxy.local_port, 11080);
        assert_eq!(cfg.pool.min_nodes, 5);
        let _ = fs::remove_file(&tmp);
    }
}
