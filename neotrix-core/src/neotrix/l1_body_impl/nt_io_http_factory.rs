use std::sync::LazyLock;
use std::time::Duration;

/// TLS 变体 — 改变 JA3/h2 指纹
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TlsVariant {
    ModernH2,
    LegacyHttp11,
    StrictVerify,
    LegacyStrict,
}

impl TlsVariant {
    pub fn all() -> &'static [TlsVariant] {
        &[TlsVariant::ModernH2, TlsVariant::LegacyHttp11, TlsVariant::StrictVerify, TlsVariant::LegacyStrict]
    }
}

/// H2 SETTINGS 参数组合
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum H2SettingsProfile {
    ChromeDefault,
    FirefoxDefault,
    SafariDefault,
    EdgeDefault,
}

impl H2SettingsProfile {
    pub fn all() -> &'static [H2SettingsProfile] {
        &[H2SettingsProfile::ChromeDefault, H2SettingsProfile::FirefoxDefault,
          H2SettingsProfile::SafariDefault, H2SettingsProfile::EdgeDefault]
    }
}

/// 连接池配置
pub const POOL_MAX_IDLE_PER_HOST: usize = 32;
pub const POOL_IDLE_TIMEOUT_SECS: u64 = 90;
pub const TCP_KEEPALIVE_SECS: u64 = 15;
pub const CONNECT_TIMEOUT_SECS: u64 = 10;
pub const REQUEST_TIMEOUT_SECS: u64 = 60;

/// 全局异步 HTTP 客户端（惰性初始化，自带连接池）
// TODO: inject via DI — pass reqwest::Client through subsystem constructors where feasible
pub fn global_client() -> &'static reqwest::Client {
    static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
        build_async_client_with_tls(TlsVariant::ModernH2, None)
    });
    &CLIENT
}

/// 全局阻塞 HTTP 客户端
// TODO: inject via DI — pass reqwest::blocking::Client through subsystem constructors
pub fn global_blocking_client() -> &'static reqwest::blocking::Client {
    static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
            .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
            .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS))
            .build()
            .unwrap_or_else(|e| {
                log::warn!("[http_factory] blocking client build failed: {}. Using default.", e);
                reqwest::blocking::Client::new()
            })
    });
    &CLIENT
}

/// 构建自定义异步客户端
pub fn build_async_client() -> reqwest::Client {
    build_async_client_with_tls(TlsVariant::ModernH2, None)
}

/// 按 TLS 变体 + 可选源地址构建异步客户端
pub fn build_async_client_with_tls(variant: TlsVariant, local_addr: Option<std::net::IpAddr>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
        .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS));

    match variant {
        TlsVariant::ModernH2 => {
            builder = builder.danger_accept_invalid_certs(true);
        }
        TlsVariant::LegacyHttp11 => {
            builder = builder.http1_only().danger_accept_invalid_certs(true);
        }
        TlsVariant::StrictVerify => {}
        TlsVariant::LegacyStrict => {
            builder = builder.http1_only();
        }
    }

    if let Some(addr) = local_addr {
        builder = builder.local_address(addr);
    }

    builder.build().unwrap_or_else(|_| global_client().clone())
}

/// 构建自定义阻塞客户端
pub fn build_blocking_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
        .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS))
        .build()
        .unwrap_or_else(|_| global_blocking_client().clone())
}

/// 从环境变量解析代理地址 (子母阵隐匿通信支持)
/// 优先级: NEOTRIX_PROXY_URL (通用代理) > NEOTRIX_TOR_PROXY (Tor SOCKS5)
pub fn proxy_from_env() -> Option<String> {
    if let Ok(url) = std::env::var("NEOTRIX_PROXY_URL") {
        if !url.is_empty() {
            return Some(url);
        }
    }
    if let Ok(url) = std::env::var("NEOTRIX_TOR_PROXY") {
        if !url.is_empty() {
            return Some(url);
        }
    }
    None
}

/// 构建带代理的异步客户端 (Proxied/Tor 子网格路由)
/// 未提供 proxy 地址时回退到标准构建，保持向后兼容
pub fn build_async_client_with_proxy(proxy_url: Option<&str>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
        .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS));
    if let Some(url) = proxy_url {
        match reqwest::Proxy::all(url) {
            Ok(proxy) => {
                builder = builder.proxy(proxy);
            }
            Err(e) => {
                log::warn!("[http_factory] invalid proxy '{}': {} — building without proxy", url, e);
            }
        }
    }
    builder.build().unwrap_or_else(|_| global_client().clone())
}

/// 构建带代理的阻塞客户端
pub fn build_blocking_client_with_proxy(proxy_url: Option<&str>) -> reqwest::blocking::Client {
    let mut builder = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
        .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS));
    if let Some(url) = proxy_url {
        match reqwest::Proxy::all(url) {
            Ok(proxy) => {
                builder = builder.proxy(proxy);
            }
            Err(e) => {
                log::warn!("[http_factory] invalid proxy '{}': {} — building without proxy", url, e);
            }
        }
    }
    builder.build().unwrap_or_else(|_| global_blocking_client().clone())
}
