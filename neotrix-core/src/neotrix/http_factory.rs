use std::sync::OnceLock;
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
pub fn global_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        build_async_client_with_tls(TlsVariant::ModernH2, None)
    })
}

/// 全局阻塞 HTTP 客户端
pub fn global_blocking_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
            .pool_idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
            .tcp_keepalive(Duration::from_secs(TCP_KEEPALIVE_SECS))
            .build()
            .expect("global blocking reqwest Client build failed")
    })
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
