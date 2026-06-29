use base64::Engine;
use lru::LruCache;
use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::core::nt_core_network::dns_cache::{AddressFamily, VsaDnsCache};

/// IP 归属地结果
#[derive(Debug, Clone)]
pub struct GeoResult {
    pub ip: String,
    pub country_code: String,
    pub timezone: String,
}

impl GeoResult {
    pub fn tag(&self) -> String {
        self.country_code.clone()
    }
}

/// 基于 ip-api.com 的 IP 地理定位器
pub struct IpGeoLocator {
    cache: Arc<Mutex<LruCache<String, GeoResult>>>,
    client: reqwest::Client,
}

impl Default for IpGeoLocator {
    fn default() -> Self {
        Self::new()
    }
}

impl IpGeoLocator {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(1024).expect("IpGeoLocator: 1024 is a valid non-zero capacity"),
            ))),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap_or_default(),
        }
    }

    /// 获取 IP 归属地（缓存命中跳过网络请求）
    pub async fn lookup(&self, ip: &str) -> Option<GeoResult> {
        {
            let mut cache = self.cache.lock().await;
            if let Some(r) = cache.get(ip) {
                return Some(r.clone());
            }
        }

        let url = format!(
            "http://ip-api.com/json/{}?fields=query,countryCode,timezone",
            ip
        );
        let resp = self.client.get(&url).send().await.ok()?;
        let body = resp.text().await.ok()?;
        let json: serde_json::Value = serde_json::from_str(&body).ok()?;

        let status = json.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status == "fail" {
            return None;
        }

        let result = GeoResult {
            ip: json
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or(ip)
                .to_string(),
            country_code: json
                .get("countryCode")
                .and_then(|v| v.as_str())
                .unwrap_or("??")
                .to_string(),
            timezone: json
                .get("timezone")
                .and_then(|v| v.as_str())
                .unwrap_or("UTC")
                .to_string(),
        };

        let mut cache = self.cache.lock().await;
        cache.put(ip.to_string(), result.clone());
        Some(result)
    }

    /// 从 proxy URL 解析 host，解析为 IP，然后地理定位
    pub async fn lookup_url(&self, url: &str) -> Option<GeoResult> {
        let host = Self::extract_host(url)?;
        let ip = Self::resolve_to_ip(&host).await?;
        self.lookup(&ip).await
    }

    /// 从 proxy URL 解析 host，解析为 IP（带缓存），然后地理定位
    pub async fn lookup_url_cached(
        &self,
        url: &str,
        dns_cache: Option<&mut VsaDnsCache>,
    ) -> Option<GeoResult> {
        let host = Self::extract_host(url)?;
        let ip = Self::resolve_to_ip_cached(&host, dns_cache).await?;
        self.lookup(&ip).await
    }

    /// 从 proxy URL 解析 host（支持 vmess:// base64 内嵌格式）
    pub fn extract_host(url: &str) -> Option<String> {
        // vmess://base64(json) — host in JSON's "add" field
        if let Some(b64) = url.strip_prefix("vmess://") {
            // strip "vmess://"
            return Self::extract_vmess_host(b64);
        }

        // ssr://base64 — host encoded in base64 payload
        if let Some(b64) = url.strip_prefix("ssr://") {
            return Self::extract_ssr_host(b64);
        }

        let after_proto = url.split("://").nth(1)?;
        let without_fragment = after_proto.split('#').next()?;
        let without_auth = without_fragment.split('@').next_back()?;
        let host = without_auth.split(':').next()?;
        if host.is_empty() {
            None
        } else {
            Some(host.to_string())
        }
    }

    fn extract_vmess_host(b64: &str) -> Option<String> {
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
        json.get("add")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn extract_ssr_host(b64: &str) -> Option<String> {
        // SSR format: base64(host:port:protocol:method:obfs:password/?params)
        let decoded = base64::engine::general_purpose::URL_SAFE.decode(b64).ok()?;
        let text = String::from_utf8(decoded).ok()?;
        text.split(':').next().map(|s| s.to_string())
    }

    /// 解析域名到 IP
    pub async fn resolve_to_ip(host: &str) -> Option<String> {
        Self::resolve_to_ip_cached(host, None).await
    }

    /// 解析域名到 IP，可选 VsaDnsCache
    pub async fn resolve_to_ip_cached(
        host: &str,
        mut dns_cache: Option<&mut VsaDnsCache>,
    ) -> Option<String> {
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Some(ip.to_string());
        }
        if let Some(cache) = dns_cache.as_deref_mut() {
            if let Some(ip) = cache.resolve(host, AddressFamily::V4) {
                return Some(ip.to_string());
            }
        }
        let addr = tokio::time::timeout(
            Duration::from_secs(3),
            tokio::net::lookup_host(format!("{}:0", host)),
        )
        .await
        .ok()?
        .ok()?
        .next()?;
        let ip_str = addr.ip().to_string();
        if let Some(cache) = dns_cache {
            cache.insert(host, addr.ip(), AddressFamily::V4);
        }
        Some(ip_str)
    }

    /// 批量地理定位（节省 API 配额）
    pub async fn lookup_batch(&self, ips: &[String]) -> Vec<(String, GeoResult)> {
        if ips.is_empty() {
            return vec![];
        }

        let mut uncached = Vec::new();
        let mut results = Vec::new();

        {
            let mut cache = self.cache.lock().await;
            for ip in ips {
                if let Some(r) = cache.get(ip) {
                    results.push((ip.clone(), r.clone()));
                } else {
                    uncached.push(ip.clone());
                }
            }
        }

        if uncached.is_empty() {
            return results;
        }

        for chunk in uncached.chunks(100) {
            let body = serde_json::to_string(chunk).unwrap_or_default();
            if let Ok(resp) = self
                .client
                .post("http://ip-api.com/batch")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
            {
                if let Ok(text) = resp.text().await {
                    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                        let mut cache = self.cache.lock().await;
                        for entry in arr {
                            let ip = entry
                                .get("query")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let status = entry.get("status").and_then(|v| v.as_str()).unwrap_or("");
                            if status == "fail" || ip.is_empty() {
                                continue;
                            }
                            let geo = GeoResult {
                                ip: ip.clone(),
                                country_code: entry
                                    .get("countryCode")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("??")
                                    .to_string(),
                                timezone: entry
                                    .get("timezone")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("UTC")
                                    .to_string(),
                            };
                            cache.put(ip.clone(), geo.clone());
                            results.push((ip.clone(), geo));
                        }
                    }
                }
            }
        }

        results
    }
}
