//! 增强型 Stealth HTTP Client — SOCKS5 + 客户端池 + 动态指纹按 RotationCoordinator 轮转
//!
//! 对标开源项目:
//! - **Obscura**: 反检测 HTTP 客户端
//! - **curl-impersonate**: TLS 指纹伪造 + JA3 分片
//! - **V2Ray/Xray**: Mux 连接池化
//!
//! 安全修复:
//! - P0: DNS 泄露防护 — 强制 SOCKS5 远端解析 (no_local_resolve)
//! - P1: 客户端池化 — 按代理 URL 缓存 reqwest::Client, 轮转时重建
//! - P2: 不 panic — build() 返回 Result, 调用方处理
//! - P3: 非周期轮转 — RotationCoordinator 统一时钟, 各域独立相位偏移

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use log;
use tokio::sync::RwLock;

use super::rotation_coordinator::{RotationCoordinator, RotationDomain};
use super::bandit::{FingerprintBandit, ComboArm};
use super::system_fingerprint::{SystemFingerprint, SystemFingerprintConfig, SystemFingerprintGenerator};
use super::proxy_chain::DynamicProxyChain;
use super::rules::RuleEngine;
use super::lan_router::LanRouter;
use super::ip_privacy::IpPrivacyManager;
use crate::neotrix::http_factory::build_async_client;

mod config;
mod pool;
mod request;

pub use config::{ProxyConfig, EntropyBudget, STEALTH_USER_AGENT};
pub use request::Response;

use config::proxy_url_string;
use pool::ClientEntry;

pub struct StealthHttpClient {
    pub(super) pool: RwLock<Vec<ClientEntry>>,
    pub(super) extra_headers: RwLock<HashMap<String, String>>,
    pub(super) check_tracker: bool,
    pub(super) fingerprint_gen: SystemFingerprintGenerator,
    pub(super) current_fingerprint: RwLock<SystemFingerprint>,
    pub(super) last_fingerprint_rotation: RwLock<Instant>,
    pub(super) proxy_config: RwLock<ProxyConfig>,
    pub(super) rule_engine: RwLock<Option<Arc<RuleEngine>>>,
    pub(super) lan_router: RwLock<Option<Arc<LanRouter>>>,
    pub(super) ip_privacy: RwLock<Option<Arc<IpPrivacyManager>>>,
    pub(super) tls_insecure: AtomicBool,
    pub(super) coordinator: RwLock<Option<Arc<RotationCoordinator>>>,
    pub(super) current_combo_arm: RwLock<ComboArm>,
    #[allow(dead_code)]
    pub(super) global_bandit: FingerprintBandit,
    pub(super) bandits: RwLock<HashMap<String, Arc<FingerprintBandit>>>,
    pub(super) isolation_map: RwLock<HashMap<String, Arc<DynamicProxyChain>>>,
    pub(super) entropy_budget: RwLock<EntropyBudget>,
    pub(super) detection_streak: AtomicU64,
}

impl Default for StealthHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl StealthHttpClient {
    pub fn new() -> Self {
        Self::build_or_fallback(ProxyConfig::None, "new")
    }

    pub fn with_proxy(proxy_url: Option<&str>) -> Self {
        let config = match proxy_url {
            Some(url) => ProxyConfig::Static(url.to_string()),
            None => ProxyConfig::None,
        };
        Self::build_or_fallback(config, "with_proxy")
    }

    pub fn with_socks5(socks_url: &str) -> Self {
        Self::build_or_fallback(ProxyConfig::Static(socks_url.to_string()), "with_socks5")
    }

    pub fn with_tor() -> Self {
        Self::build_or_fallback(ProxyConfig::Tor, "with_tor")
    }

    pub fn with_dynamic_chain(chain: Arc<DynamicProxyChain>) -> Self {
        Self::build_or_fallback(ProxyConfig::DynamicChain(chain), "with_dynamic_chain")
    }

    fn build_or_fallback(proxy: ProxyConfig, label: &str) -> Self {
        match Self::build(proxy) {
            Ok(client) => client,
            Err(e) => {
                log::warn!("StealthHttpClient::{} failed (fallback to no-proxy): {}", label, e);
                Self::build(ProxyConfig::None).unwrap_or_else(|_| {
                    let client = build_async_client();
                    let fp = SystemFingerprintGenerator::new().generate(&SystemFingerprintConfig::default());
                    let headers = SystemFingerprintGenerator::to_headers(&fp);
                    Self {
                        pool: RwLock::new(vec![ClientEntry {
                            client,
                            proxy_url: String::new(),
                            created_at: Instant::now(),
                        }]),
                        extra_headers: RwLock::new(headers),
                        check_tracker: true,
                        fingerprint_gen: SystemFingerprintGenerator::new(),
                        current_fingerprint: RwLock::new(fp),
                        last_fingerprint_rotation: RwLock::new(Instant::now()),
                        proxy_config: RwLock::new(ProxyConfig::None),
                        rule_engine: RwLock::new(None),
                        lan_router: RwLock::new(None),
                        ip_privacy: RwLock::new(None),
                        tls_insecure: AtomicBool::new(true),
                        coordinator: RwLock::new(None),
                        current_combo_arm: RwLock::new(ComboArm::all()[0].clone()),
                        global_bandit: FingerprintBandit::load(),
                        bandits: RwLock::new(HashMap::new()),
                        isolation_map: RwLock::new(HashMap::new()),
                        entropy_budget: RwLock::new(EntropyBudget::new(20.0)),
                        detection_streak: AtomicU64::new(0),
                    }
                })
            }
        }
    }

    fn build(proxy: ProxyConfig) -> Result<Self, String> {
        let fp_gen = SystemFingerprintGenerator::new();
        let fp_config = SystemFingerprintConfig::default();
        let fingerprint = fp_gen.generate(&fp_config);

        let mut headers = HashMap::new();
        headers.extend(SystemFingerprintGenerator::to_headers(&fingerprint));

        let client = Self::build_client_for_proxy(&proxy, true, None, None)?;

        let mut pool = Vec::new();
        let proxy_url = proxy_url_string(&proxy);
        pool.push(ClientEntry {
            client,
            proxy_url: proxy_url.clone(),
            created_at: Instant::now(),
        });

        Ok(Self {
            pool: RwLock::new(pool),
            extra_headers: RwLock::new(headers),
            check_tracker: true,
            fingerprint_gen: fp_gen,
            current_fingerprint: RwLock::new(fingerprint),
            last_fingerprint_rotation: RwLock::new(Instant::now()),
            proxy_config: RwLock::new(proxy),
            rule_engine: RwLock::new(None),
            lan_router: RwLock::new(None),
            ip_privacy: RwLock::new(None),
            tls_insecure: AtomicBool::new(true),
            coordinator: RwLock::new(None),
            current_combo_arm: RwLock::new(ComboArm::all()[0].clone()),
            global_bandit: FingerprintBandit::load(),
            bandits: RwLock::new(HashMap::new()),
            isolation_map: RwLock::new(HashMap::new()),
            entropy_budget: RwLock::new(EntropyBudget::new(20.0)),
            detection_streak: AtomicU64::new(0),
        })
    }

    pub fn with_tracker_check(mut self, enable: bool) -> Self {
        self.check_tracker = enable;
        self
    }

    pub fn with_tls_insecure(mut self, insecure: bool) -> Self {
        self.tls_insecure = AtomicBool::new(insecure);
        self
    }

    pub async fn set_tls_insecure(&self, insecure: bool) {
        let prev = self.tls_insecure.load(Ordering::Relaxed);
        self.tls_insecure.store(insecure, Ordering::Relaxed);
        if prev != insecure {
            let mut pool = self.pool.write().await;
            pool.clear();
        }
    }

    pub async fn set_system_fingerprint(&self, fp: SystemFingerprint) {
        let headers = SystemFingerprintGenerator::to_headers(&fp);
        let mut extra = self.extra_headers.write().await;
        extra.clear();
        extra.extend(headers);
        *self.current_fingerprint.write().await = fp;
    }

    pub async fn set_coordinator(&self, coord: Arc<RotationCoordinator>) {
        *self.coordinator.write().await = Some(coord);
    }

    pub async fn rotate_tls_variant(&self) {
        self.rotate_tls_variant_for_host(None).await;
    }

    pub async fn rotate_tls_variant_for_host(&self, host: Option<&str>) {
        let bandit = self.get_bandit(host).await;
        let geo = match host {
            Some(h) => self.current_geo_for_host(h).await,
            None => self.current_geo().await,
        };
        let new_arm = bandit.select_arm(geo.as_deref());
        *self.current_combo_arm.write().await = new_arm.clone();
        let config = SystemFingerprintConfig {
            platform: Some(new_arm.platform),
            h2_profile: Some(new_arm.h2_profile),
            ..Default::default()
        };
        let fp = self.fingerprint_gen.generate(&config);
        self.set_system_fingerprint(fp).await;
        self.pool.write().await.clear();

        if let Some(ref coord) = *self.coordinator.read().await {
            let conf = bandit.confidence();
            let base_mean = 7500.0_f64;
            let min_factor = 0.5;
            let effective_mean = base_mean * (min_factor + conf * (1.0 - min_factor)).max(min_factor);
            let effective_std = effective_mean * 0.3;
            coord.set_domain_params(RotationDomain::TlsFingerprint, effective_mean, effective_std).await;
        }
    }

    pub async fn rotate_fingerprint(&self) {
        let arm = self.current_combo_arm.read().await.clone();
        let config = SystemFingerprintConfig {
            platform: Some(arm.platform),
            h2_profile: Some(arm.h2_profile),
            ..Default::default()
        };
        let fp = self.fingerprint_gen.generate(&config);
        self.set_system_fingerprint(fp).await;
        *self.last_fingerprint_rotation.write().await = Instant::now();
    }

    pub(super) async fn force_clear_fingerprint(&self) {
        let arm = self.current_combo_arm.read().await.clone();
        let config = SystemFingerprintConfig {
            platform: Some(arm.platform),
            h2_profile: Some(arm.h2_profile),
            ..Default::default()
        };
        let fp = self.fingerprint_gen.generate(&config);
        self.set_system_fingerprint(fp).await;
        *self.last_fingerprint_rotation.write().await = Instant::now();
        let mut pool = self.pool.write().await;
        pool.clear();
    }

    pub(super) async fn auto_rotate_fingerprint(&self) {
        if let Some(ref coord) = *self.coordinator.read().await {
            if coord.should_rotate(RotationDomain::HttpHeaders).await {
                self.rotate_fingerprint().await;
                coord.mark_rotated(RotationDomain::HttpHeaders).await;
            }
            if coord.should_rotate(RotationDomain::TlsFingerprint).await {
                self.rotate_tls_variant().await;
                coord.mark_rotated(RotationDomain::TlsFingerprint).await;
            }
        } else {
            let last = *self.last_fingerprint_rotation.read().await;
            if last.elapsed() >= Duration::from_secs(9) {
                self.rotate_fingerprint().await;
            }
        }
    }

    pub async fn set_rule_engine(&self, engine: Arc<RuleEngine>) {
        *self.rule_engine.write().await = Some(engine);
    }

    pub async fn clear_rule_engine(&self) {
        *self.rule_engine.write().await = None;
    }

    pub async fn set_dynamic_chain(&self, chain: Arc<DynamicProxyChain>) {
        *self.proxy_config.write().await = ProxyConfig::DynamicChain(chain);
    }

    pub async fn bind_lan_router(&self, router: Arc<LanRouter>) {
        *self.lan_router.write().await = Some(router.clone());
        tokio::spawn(async move {
            router.start_rotation_loop().await;
        });
    }

    pub async fn bind_ip_privacy(&self, mgr: Arc<IpPrivacyManager>) {
        *self.ip_privacy.write().await = Some(mgr.clone());
        tokio::spawn(async move {
            mgr.start_rotation_loop().await;
        });
    }

    pub async fn set_tor_proxy(&self) {
        *self.proxy_config.write().await = ProxyConfig::Tor;
    }

    pub async fn set_isolation_chain(&self, host: &str, chain: Arc<DynamicProxyChain>) {
        self.isolation_map.write().await.insert(host.to_string(), chain);
    }

    pub async fn clear_isolation_chain(&self, host: &str) {
        self.isolation_map.write().await.remove(host);
    }

    pub async fn set_extra_headers(&self, headers: HashMap<String, String>) {
        let mut extra = self.extra_headers.write().await;
        extra.clear();
        extra.extend(headers);
    }

    pub async fn extra_headers(&self) -> HashMap<String, String> {
        self.extra_headers.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_client_creation() {
        let client = StealthHttpClient::new();
        assert!(client.check_tracker);
    }

    #[test]
    fn test_client_with_proxy() {
        let client = StealthHttpClient::with_proxy(Some("http://proxy:8080"));
        assert!(client.check_tracker);
    }

    #[test]
    fn test_client_with_socks5() {
        let client = StealthHttpClient::with_socks5("socks5://127.0.0.1:9050");
        assert!(client.check_tracker);
    }

    #[test]
    fn test_client_with_tor() {
        let client = StealthHttpClient::with_tor();
        assert!(client.check_tracker);
    }

    #[test]
    fn test_client_disable_tracker_check() {
        let client = StealthHttpClient::new().with_tracker_check(false);
        assert!(!client.check_tracker);
    }

    #[test]
    fn test_response_text() {
        let resp = Response {
            url: Url::parse("https://example.com").expect("value should be ok in test"),
            status: 200,
            headers: HashMap::new(),
            body: b"hello world".to_vec(),
        };
        assert_eq!(resp.text().expect("value should be ok in test"), "hello world");
    }

    #[test]
    fn test_response_is_html() {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/html; charset=utf-8".into());
        let resp = Response {
            url: Url::parse("https://example.com").expect("value should be ok in test"),
            status: 200,
            headers,
            body: vec![],
        };
        assert!(resp.is_html());
    }

    #[test]
    fn test_response_not_html() {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "application/json".into());
        let resp = Response {
            url: Url::parse("https://example.com").expect("value should be ok in test"),
            status: 200,
            headers,
            body: vec![],
        };
        assert!(!resp.is_html());
    }
}
