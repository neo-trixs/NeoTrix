//! 系统级代理 — 全局流量路由 (macOS/Linux/Windows)
//!
//! 对标:
//! - **ProxyChains-NG**: 系统级流量劫持
//! - **V2Ray/Xray TUN**: 虚拟网卡代理
//! - **mitmproxy**: 透明代理
//!
//! 能力:
//! - 设置 HTTP_PROXY / HTTPS_PROXY / ALL_PROXY / NO_PROXY 环境变量
//! - macOS: networksetup 配置网络接口代理 (SOCKS + HTTP + HTTPS)
//! - Linux: gsettings / 环境变量全局生效
//! - 集成动态代理链出口 (每15秒刷新)
//! - 快速开关 (启用/禁用/状态查询)

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::sleep;

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

use super::proxy_chain::DynamicProxyChain;

const SYSTEM_PROXY_CHECK_INTERVAL_SECS: u64 = 9;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OsType {
    MacOS,
    Linux,
    Windows,
    Other,
}

impl OsType {
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        { OsType::MacOS }
        #[cfg(target_os = "linux")]
        { OsType::Linux }
        #[cfg(target_os = "windows")]
        { OsType::Windows }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        { OsType::Other }
    }
}

#[derive(Debug, Clone)]
pub struct SystemProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub socks_proxy: Option<String>,
    pub no_proxy: Vec<String>,
    pub auto_detect: bool,
}

impl Default for SystemProxyConfig {
    fn default() -> Self {
        Self {
            http_proxy: None,
            https_proxy: None,
            socks_proxy: None,
            no_proxy: vec!["localhost".into(), "127.0.0.1".into(), "::1".into()],
            auto_detect: true,
        }
    }
}

/// 系统代理管理器 — 控制 OS 级流量转发
pub struct SystemProxyManager {
    config: RwLock<SystemProxyConfig>,
    enabled: AtomicBool,
    os: OsType,
    env_backup: RwLock<HashMap<String, String>>,
    dynamic_chain: RwLock<Option<Arc<DynamicProxyChain>>>,
}

impl Default for SystemProxyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemProxyManager {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(SystemProxyConfig::default()),
            enabled: AtomicBool::new(false),
            os: OsType::detect(),
            env_backup: RwLock::new(HashMap::new()),
            dynamic_chain: RwLock::new(None),
        }
    }

    pub fn with_dynamic_chain(chain: Arc<DynamicProxyChain>) -> Self {
        Self {
            config: RwLock::new(SystemProxyConfig::default()),
            enabled: AtomicBool::new(false),
            os: OsType::detect(),
            env_backup: RwLock::new(HashMap::new()),
            dynamic_chain: RwLock::new(Some(chain)),
        }
    }

    /// 注册 SIGTERM/SIGINT 处理 — 进程退出时自动恢复系统代理
    pub async fn install_shutdown_handler(self: Arc<Self>) {
        #[cfg(unix)]
        {
            let mut term = match signal(SignalKind::terminate()) {
                Ok(s) => s,
                Err(_) => return,
            };
            let mut int = match signal(SignalKind::interrupt()) {
                Ok(s) => s,
                Err(_) => return,
            };
            let proxy = self.clone();
            tokio::spawn(async move {
                tokio::select! {
                    _ = term.recv() => {}
                    _ = int.recv() => {}
                }
                log::info!("[system-proxy] signal received, restoring proxy settings...");
                let _ = proxy.disable().await;
                std::process::exit(0);
            });
        }
    }

    pub fn os(&self) -> OsType {
        self.os
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// 绑定动态代理链（每次切换出口自动更新系统代理）
    pub async fn bind_dynamic_chain(&self, chain: Arc<DynamicProxyChain>) {
        *self.dynamic_chain.write().await = Some(chain);
    }

    /// 设置静态代理配置
    pub async fn set_config(&self, config: SystemProxyConfig) {
        *self.config.write().await = config;
    }

    /// 获取当前出口代理 URL (使用 display_url, 不含凭证, 防子进程泄露)
    async fn current_proxy_url(&self) -> String {
        if let Some(ref chain) = *self.dynamic_chain.read().await {
            if let Some(exit_url) = chain.current_exit_url().await {
                if let Ok(parsed) = url::Url::parse(&exit_url) {
                    let scheme = parsed.scheme();
                    let host = parsed.host_str().unwrap_or("127.0.0.1");
                    let port = parsed.port().unwrap_or(9050);
                    return format!("{}://{}:{}", scheme, host, port);
                }
            }
        }
        let config = self.config.read().await;
        config.https_proxy.clone()
            .or_else(|| config.http_proxy.clone())
            .or_else(|| config.socks_proxy.clone())
            .unwrap_or_else(|| "http://127.0.0.1:11080".into())
    }

    /// 检查代理端口是否可达（TCP 连接测试）
    async fn proxy_is_reachable(host: &str, port: u16) -> bool {
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            tokio::net::TcpStream::connect(format!("{}:{}", host, port)),
        )
        .await
        .is_ok()
    }

    /// 启用系统代理 — 自动检测可达性 + 重试 + 智能降级
    /// 1. 用 TCP 探测确认代理后端已运行
    /// 2. 若不可达，等待重试最多 3 次
    /// 3. 仍不可达则只设环境变量，跳过 networksetup
    /// 4. 后续 retry_enable 自动修复
    pub async fn enable(&self) -> Result<(), String> {
        if self.enabled.swap(true, Ordering::Relaxed) {
            return Ok(());
        }

        let proxy_url = self.current_proxy_url().await;
        let parsed = url::Url::parse(&proxy_url)
            .map_err(|e| format!("Invalid proxy URL: {}", e))?;
        let host = parsed.host_str().unwrap_or("127.0.0.1").to_string();
        let port = parsed.port().unwrap_or(9050);

        // 检查代理后端是否可达，最多重试 3 次
        let mut reachable = false;
        for attempt in 1..=3 {
            if Self::proxy_is_reachable(&host, port).await {
                reachable = true;
                break;
            }
            if attempt < 3 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }

        // 备份环境变量
        let mut backup = self.env_backup.write().await;
        for var in &["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY",
                     "http_proxy", "https_proxy", "all_proxy", "no_proxy"] {
            if let Ok(val) = std::env::var(var) {
                backup.insert(var.to_string(), val);
            }
        }

        let config = self.config.read().await;
        let no_proxy_str = config.no_proxy.join(",");
        let is_socks = proxy_url.starts_with("socks");

        if is_socks {
            std::env::set_var("ALL_PROXY", &proxy_url);
            std::env::set_var("all_proxy", &proxy_url);
            std::env::set_var("SOCKS5_PROXY", &proxy_url);
            std::env::set_var("socks5_proxy", &proxy_url);
            std::env::remove_var("HTTP_PROXY");
            std::env::remove_var("HTTPS_PROXY");
            std::env::remove_var("http_proxy");
            std::env::remove_var("https_proxy");
        } else {
            std::env::set_var("HTTP_PROXY", &proxy_url);
            std::env::set_var("HTTPS_PROXY", &proxy_url);
            std::env::set_var("http_proxy", &proxy_url);
            std::env::set_var("https_proxy", &proxy_url);
        }
        std::env::set_var("NO_PROXY", &no_proxy_str);
        std::env::set_var("no_proxy", &no_proxy_str);

        // 仅在后端可达时设置 OS 级代理（networksetup）
        // 否则跳过，后续 retry_enable 自动修复
        if reachable {
            self.set_os_proxy(&proxy_url).await?;
            println!("[proxy] system proxy enabled via networksetup ({})", proxy_url);
        } else {
            println!("[proxy] proxy backend {host}:{port} unreachable, env vars set (networksetup skipped)");
            // 标记为已启用但 OS 代理未设置 → retry_enable 会修复
            self.enabled.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    /// 重试启用 OS 级系统代理（由 BackgroundLoop 定期调用）
    /// 用于代理后端延迟启动的场景（如 Tor 启动较慢）
    pub async fn retry_enable(&self) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }
        let proxy_url = self.current_proxy_url().await;
        if let Ok(parsed) = url::Url::parse(&proxy_url) {
            let host = parsed.host_str().unwrap_or("127.0.0.1");
            let port = parsed.port().unwrap_or(9050);
            if Self::proxy_is_reachable(host, port).await
                && self.set_os_proxy(&proxy_url).await.is_ok() {
                    println!("[proxy] system proxy activated via retry ({})", proxy_url);
                    return true;
                }
        }
        false
    }

    /// 禁用系统代理 — 恢复环境变量 + 清除系统配置
    pub async fn disable(&self) -> Result<(), String> {
        if !self.enabled.swap(false, Ordering::Relaxed) {
            return Ok(());
        }

        let backup = self.env_backup.read().await;
        for var in &["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY",
                     "http_proxy", "https_proxy", "all_proxy", "no_proxy"] {
            if backup.contains_key(*var) {
                if let Some(val) = backup.get(*var) {
                    std::env::set_var(var, val);
                }
            } else {
                std::env::remove_var(var);
            }
        }

        self.clear_os_proxy().await?;
        Ok(())
    }

    /// 切换出口代理（刷新环境变量 + 系统配置）
    pub async fn switch_exit(&self) -> Result<(), String> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        if let Some(ref chain) = *self.dynamic_chain.read().await {
            chain.rotate_all().await;
        }

        let proxy_url = self.current_proxy_url().await;
        if proxy_url.starts_with("socks") {
            std::env::set_var("ALL_PROXY", &proxy_url);
            std::env::set_var("SOCKS5_PROXY", &proxy_url);
        } else {
            std::env::set_var("HTTP_PROXY", &proxy_url);
            std::env::set_var("HTTPS_PROXY", &proxy_url);
            std::env::set_var("ALL_PROXY", &proxy_url);
        }

        self.set_os_proxy(&proxy_url).await
    }

    /// 获取 macOS 活跃网络接口（自动发现，支持 USB/Thunderbolt 等）
    fn get_active_services_macos() -> Vec<String> {
        let mut services = Vec::new();
        if let Ok(output) = std::process::Command::new("networksetup")
            .args(["-listallnetworkservices"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    let s = line.trim();
                    if !s.is_empty()
                        && !s.contains('*')
                        && s != "An asterisk (*) denotes that a network service is disabled."
                    {
                        services.push(s.to_string());
                    }
                }
            }
        }
        if services.is_empty() {
            vec!["Wi-Fi".into(), "Ethernet".into()]
        } else {
            services
        }
    }

    /// macOS: 用 networksetup 设置系统代理
    /// Chrome 需要 HTTP + HTTPS 代理同时设置才能拦截所有流量
    async fn set_os_proxy(&self, proxy_url: &str) -> Result<(), String> {
        match self.os {
            OsType::MacOS => {
                let parsed = url::Url::parse(proxy_url)
                    .map_err(|e| format!("Invalid proxy URL: {}", e))?;
                let host = parsed.host_str().unwrap_or("127.0.0.1");
                let port = &parsed.port().unwrap_or(9050).to_string();
                let scheme = parsed.scheme();
                let services = Self::get_active_services_macos();

                let _ = self.clear_os_proxy().await;

                for service in &services {
                    // 跳过 VPN/代理接口，只改物理网络
                    if service.contains("Shadowrocket") || service.contains("VPN") || service.contains("Tunnel") {
                        continue;
                    }
                    match scheme {
                        "socks5" | "socks4" | "socks" => {
                            let _ = std::process::Command::new("networksetup")
                                .args(["-setsocksfirewallproxy", service, host, port])
                                .output();
                            let _ = std::process::Command::new("networksetup")
                                .args(["-setsocksfirewallproxystate", service, "on"])
                                .output();
                        }
                        _ => {
                            // HTTP 或 HTTPS 代理: 同时设置 HTTP + HTTPS
                            // Chrome 浏览器需要 HTTPS proxy 设置才能代理 HTTPS 流量
                            let _ = std::process::Command::new("networksetup")
                                .args(["-setwebproxy", service, host, port])
                                .output();
                            let _ = std::process::Command::new("networksetup")
                                .args(["-setwebproxystate", service, "on"])
                                .output();
                            let _ = std::process::Command::new("networksetup")
                                .args(["-setsecurewebproxy", service, host, port])
                                .output();
                            let _ = std::process::Command::new("networksetup")
                                .args(["-setsecurewebproxystate", service, "on"])
                                .output();
                        }
                    }
                }
                Ok(())
            }
            OsType::Linux => {
                if let Ok(output) = std::process::Command::new("gsettings")
                    .args(["--version"])
                    .output()
                {
                    if output.status.success() {
                        let parsed = url::Url::parse(proxy_url)
                            .map_err(|e| format!("Invalid proxy URL for Linux gsettings: {}", e))?;
                        let proxy_host = parsed.host_str().unwrap_or("127.0.0.1").to_string();
                        let proxy_port = parsed.port().unwrap_or(8080).to_string();

                        if let Err(e) = std::process::Command::new("gsettings")
                            .args(["set", "org.gnome.system.proxy", "mode", "manual"])
                            .output() {
                            log::warn!("[system-proxy] gsettings set mode manual failed: {}", e);
                        }
                        for (schema, _key) in &[("socks", "socks"), ("http", "http"), ("https", "https")] {
                            let _ = std::process::Command::new("gsettings")
                                .args(["set", &format!("org.gnome.system.proxy.{}", schema), "host", &proxy_host])
                                .output();
                            let _ = std::process::Command::new("gsettings")
                                .args(["set", &format!("org.gnome.system.proxy.{}", schema), "port", &proxy_port])
                                .output();
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// 清除 OS 级系统代理（全部类型）
    async fn clear_os_proxy(&self) -> Result<(), String> {
        match self.os {
            OsType::MacOS => {
                let services = Self::get_active_services_macos();
                for service in &services {
                    let _ = std::process::Command::new("networksetup")
                        .args(["-setsocksfirewallproxystate", service, "off"])
                        .output();
                    let _ = std::process::Command::new("networksetup")
                        .args(["-setwebproxystate", service, "off"])
                        .output();
                    let _ = std::process::Command::new("networksetup")
                        .args(["-setsecurewebproxystate", service, "off"])
                        .output();
                }
                Ok(())
            }
            OsType::Linux => {
                if let Ok(output) = std::process::Command::new("gsettings")
                    .args(["--version"])
                    .output()
                {
                    if output.status.success() {
                        if let Err(e) = std::process::Command::new("gsettings")
                            .args(["set", "org.gnome.system.proxy", "mode", "none"])
                            .output() {
                            log::warn!("[system-proxy] gsettings set mode none failed: {}", e);
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// 自动刷新循环：每15秒同步动态链出口到系统代理
    pub async fn start_auto_sync(self: Arc<Self>) {
        loop {
            sleep(Duration::from_secs(SYSTEM_PROXY_CHECK_INTERVAL_SECS)).await;
            if self.enabled.load(Ordering::Relaxed) {
                if let Some(ref chain) = *self.dynamic_chain.read().await {
                    let exit_url = chain.current_exit_url().await;
                    if let Some(url) = exit_url {
                        let safe_url = if let Ok(parsed) = url::Url::parse(&url) {
                            let scheme = parsed.scheme();
                            let host = parsed.host_str().unwrap_or("127.0.0.1");
                            let port = parsed.port().unwrap_or(9050);
                            format!("{}://{}:{}", scheme, host, port)
                        } else {
                            url.clone()
                        };
                        std::env::set_var("HTTP_PROXY", &safe_url);
                        std::env::set_var("HTTPS_PROXY", &safe_url);
                        std::env::set_var("ALL_PROXY", &safe_url);
                        chain.rotate_all().await;
                    }
                }
            }
        }
    }

    pub async fn status(&self) -> SystemProxyStatus {
        SystemProxyStatus {
            os: self.os,
            enabled: self.enabled.load(Ordering::Relaxed),
            http_proxy: std::env::var("HTTP_PROXY").ok(),
            https_proxy: std::env::var("HTTPS_PROXY").ok(),
            socks_proxy: std::env::var("ALL_PROXY").ok(),
            no_proxy: std::env::var("NO_PROXY").ok(),
            dynamic_chain_bound: self.dynamic_chain.read().await.is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemProxyStatus {
    pub os: OsType,
    pub enabled: bool,
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub socks_proxy: Option<String>,
    pub no_proxy: Option<String>,
    pub dynamic_chain_bound: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    async fn with_env_lock<F, R>(f: F) -> R
    where
        F: std::future::Future<Output = R>,
    {
        let _lock = match ENV_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let proxy_vars = &["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "SOCKS5_PROXY",
                     "http_proxy", "https_proxy", "all_proxy", "socks5_proxy"];
        for var in proxy_vars {
            std::env::remove_var(var);
        }
        let result = f.await;
        let all_vars = &["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY", "SOCKS5_PROXY",
                     "http_proxy", "https_proxy", "all_proxy", "no_proxy", "socks5_proxy"];
        for var in all_vars {
            std::env::remove_var(var);
        }
        result
    }

    #[tokio::test]
    async fn test_system_proxy_manager_creation() {
        with_env_lock(async {
            let mgr = SystemProxyManager::new();
            assert!(!mgr.is_enabled());
        }).await;
    }

    #[tokio::test]
    async fn test_enable_disable_proxy() {
        with_env_lock(async {
            let mgr = SystemProxyManager::new();
            mgr.enable().await.expect("enable proxy in test should succeed");
            assert!(mgr.is_enabled());
            mgr.disable().await.expect("disable proxy in test should succeed");
            assert!(!mgr.is_enabled());
        }).await;
    }

    #[tokio::test]
    async fn test_proxy_env_vars_set_on_enable() {
        with_env_lock(async {
            let mgr = SystemProxyManager::new();
            mgr.enable().await.expect("enable proxy in test_proxy_env_vars should succeed");
            // 默认 HTTP 代理 → 设 HTTP_PROXY + HTTPS_PROXY
            assert!(std::env::var("HTTP_PROXY").is_ok());
            assert!(std::env::var("HTTPS_PROXY").is_ok());
            mgr.disable().await.expect("disable proxy in test_proxy_env_vars should succeed");
        }).await;
    }

    #[tokio::test]
    async fn test_env_restored_on_disable() {
        with_env_lock(async {
            std::env::set_var("HTTP_PROXY", "http://original:8080");
            let mgr = SystemProxyManager::new();
            mgr.enable().await.expect("enable proxy in test_env_restored should succeed");
            mgr.disable().await.expect("disable proxy in test_env_restored should succeed");
            assert_eq!(std::env::var("HTTP_PROXY").expect("HTTP_PROXY should be restored to original value"), "http://original:8080");
            std::env::remove_var("HTTP_PROXY");
        }).await;
    }

    #[tokio::test]
    async fn test_status() {
        let mgr = SystemProxyManager::new();
        let status = mgr.status().await;
        assert!(!status.enabled);
    }

    #[test]
    fn test_os_detection() {
        let os = OsType::detect();
        #[cfg(target_os = "macos")]
        assert_eq!(os, OsType::MacOS);
        #[cfg(target_os = "linux")]
        assert_eq!(os, OsType::Linux);
    }

    #[test]
    fn test_config_default_no_proxy() {
        let config = SystemProxyConfig::default();
        assert!(config.no_proxy.contains(&"localhost".to_string()));
        assert!(config.no_proxy.contains(&"127.0.0.1".to_string()));
    }
}
