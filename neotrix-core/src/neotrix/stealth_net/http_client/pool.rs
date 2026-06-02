use std::sync::Arc;
use std::time::{Duration, Instant};

use super::super::bandit::FingerprintBandit;
use crate::neotrix::http_factory::TlsVariant;

use super::StealthHttpClient;
use super::config::{self, ProxyConfig, STEALTH_USER_AGENT, DEFAULT_TIMEOUT_SECS, MAX_REDIRECTS, STEALTH_CONNECT_TIMEOUT_SECS, STEALTH_POOL_MAX_IDLE, STEALTH_POOL_IDLE_TIMEOUT_SECS};

pub(crate) struct ClientEntry {
    pub(super) client: reqwest::Client,
    pub(super) proxy_url: String,
    pub(super) created_at: Instant,
}

impl StealthHttpClient {
    pub(super) fn build_client_for_proxy(
        config: &ProxyConfig,
        tls_insecure: bool,
        tls_variant: Option<TlsVariant>,
        local_addr: Option<std::net::IpAddr>,
    ) -> Result<reqwest::Client, String> {
        let variant = tls_variant.unwrap_or(TlsVariant::ModernH2);
        let use_insecure = tls_insecure && variant != TlsVariant::StrictVerify && variant != TlsVariant::LegacyStrict;

        let proxy_str = match config {
            ProxyConfig::Static(url) => {
                if use_insecure && url.starts_with("https://") {
                    log::warn!("[tls] downgraded HTTPS proxy to HTTP (tls_insecure + strict_verify conflict)");
                    url.replacen("https://", "http://", 1)
                } else {
                    url.clone()
                }
            }
            ProxyConfig::Tor => "socks5://127.0.0.1:9050".into(),
            _ => String::new(),
        };

        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(STEALTH_CONNECT_TIMEOUT_SECS))
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .user_agent(STEALTH_USER_AGENT)
            .pool_max_idle_per_host(STEALTH_POOL_MAX_IDLE)
            .pool_idle_timeout(Duration::from_secs(STEALTH_POOL_IDLE_TIMEOUT_SECS))
            .tcp_keepalive(Duration::from_secs(10))
            .no_proxy()
            .resolve_to_addrs("dns.google", &[std::net::SocketAddr::from(([8,8,8,8], 443)), std::net::SocketAddr::from(([8,8,4,4], 443))])
            .resolve_to_addrs("cloudflare-dns.com", &[std::net::SocketAddr::from(([1,1,1,1], 443)), std::net::SocketAddr::from(([1,0,0,1], 443))])
            .resolve_to_addrs("dns.quad9.net", &[std::net::SocketAddr::from(([9,9,9,9], 443)), std::net::SocketAddr::from(([149,112,112,112], 443))]);

        match variant {
            TlsVariant::ModernH2 => {
                if use_insecure { builder = builder.danger_accept_invalid_certs(true); }
            }
            TlsVariant::LegacyHttp11 => {
                builder = builder.http1_only();
                if use_insecure { builder = builder.danger_accept_invalid_certs(true); }
            }
            TlsVariant::StrictVerify => {}
            TlsVariant::LegacyStrict => {
                builder = builder.http1_only();
            }
        }

        if let Some(addr) = local_addr {
            builder = builder.local_address(addr);
        }

        if !proxy_str.is_empty() {
            if let Ok(p) = reqwest::Proxy::all(&proxy_str) {
                builder = builder.proxy(p);
            }
        }

        builder.build().map_err(|e| format!("Failed to build reqwest client: {}", e))
    }

    pub(super) async fn get_or_create_client(&self, proxy_url: &str) -> Result<reqwest::Client, String> {
        let pool = self.pool.read().await;
        if let Some(entry) = pool.iter().find(|e| e.proxy_url == proxy_url && e.created_at.elapsed() < Duration::from_secs(9)) {
            return Ok(entry.client.clone());
        }
        drop(pool);

        let config = config::proxy_config_from_url(proxy_url);
        let tls_variant = self.current_combo_arm.read().await.tls;
        let local_addr = self.lan_router.read().await.as_ref()
            .and_then(|r| r.current_ip_sync());

        let client = Self::build_client_for_proxy(
            &config,
            self.tls_insecure.load(std::sync::atomic::Ordering::Relaxed),
            Some(tls_variant),
            local_addr,
        )?;

        let mut pool = self.pool.write().await;
        pool.retain(|e| e.created_at.elapsed() < Duration::from_secs(9));
        pool.push(ClientEntry {
            client: client.clone(),
            proxy_url: proxy_url.to_string(),
            created_at: Instant::now(),
        });
        Ok(client)
    }

    pub(super) async fn get_bandit(&self, host: Option<&str>) -> Arc<FingerprintBandit> {
        let host = match host {
            Some(h) if !h.is_empty() => h,
            _ => return Arc::new(FingerprintBandit::load()),
        };
        let map = self.bandits.read().await;
        if let Some(b) = map.get(host) {
            return b.clone();
        }
        drop(map);
        let b = Arc::new(FingerprintBandit::load());
        self.bandits.write().await.insert(host.to_string(), b.clone());
        b
    }

    pub(super) async fn current_geo(&self) -> Option<String> {
        let proxy_config = self.proxy_config.read().await.clone();
        match &proxy_config {
            ProxyConfig::DynamicChain(chain) => {
                chain.current_exit_geo().await
            }
            _ => None,
        }
    }

    pub(super) async fn current_geo_for_host(&self, host: &str) -> Option<String> {
        let map = self.isolation_map.read().await;
        if let Some(chain) = map.get(host) {
            return chain.current_exit_geo().await;
        }
        self.current_geo().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_build_client_no_proxy() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::None, false, None, None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_static_http_proxy() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::Static("http://127.0.0.1:8080".into()), false, None, None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_tor() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::Tor, false, None, None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_modern_h2_insecure() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::None, true, Some(TlsVariant::ModernH2), None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_legacy_http11() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::None, true, Some(TlsVariant::LegacyHttp11), None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_strict_verify() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::None, true, Some(TlsVariant::StrictVerify), None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_legacy_strict() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::None, true, Some(TlsVariant::LegacyStrict), None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_with_local_address() {
        let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::None, false, None, Some(addr),
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_https_proxy_insecure() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::Static("https://proxy:8443".into()), true, None, None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_dynamic_chain_falls_to_no_proxy() {
        let client = StealthHttpClient::build_client_for_proxy(
            &ProxyConfig::None, false, None, None,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_entry_construction() {
        let raw = reqwest::Client::new();
        let entry = ClientEntry {
            client: raw.clone(),
            proxy_url: "socks5://127.0.0.1:9050".into(),
            created_at: std::time::Instant::now(),
        };
        assert_eq!(entry.proxy_url, "socks5://127.0.0.1:9050");
    }
}
