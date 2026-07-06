//! Resource normalizers — convert raw external data into `DiscoveredResource`.
//!
//! Each normalizer handles a specific protocol or data format:
//! - `ProxyUrlNormalizer` — parses ss:// ssr:// vmess:// trojan:// vless:// socks5://
//! - `LlmProviderNormalizer` — converts model entries from free_catalog / discovery

use std::collections::HashMap;

use super::resource_types::{DiscoveredResource, ResourceKind};

/// Result of normalizing a single raw entry.
#[derive(Debug, Clone)]
pub struct NormalizedEntry {
    pub resource: DiscoveredResource,
    pub raw: String,
}

/// Trait for normalizers.
pub trait ResourceNormalizer: Send + Sync {
    fn kind(&self) -> ResourceKind;

    /// Parse a raw line/entry into zero or more `DiscoveredResource`s.
    fn normalize(&self, raw: &str) -> Vec<DiscoveredResource>;
}

/// Normalizer for proxy URLs (ss://, ssr://, vmess://, trojan://, vless://, socks5://, http://).
pub struct ProxyUrlNormalizer;

impl Default for ProxyUrlNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyUrlNormalizer {
    pub fn new() -> Self {
        Self
    }

    /// Extract host from any proxy URL for use as label.
    fn extract_host(url: &str) -> String {
        let after_scheme = url.split("://").nth(1).unwrap_or(url);
        // Remove credentials: user:pass@host → host
        let after_auth = after_scheme.split('@').next_back().unwrap_or(after_scheme);
        // Remove fragment: host#tag → host
        let no_frag = after_auth.split('#').next().unwrap_or(after_auth);
        // Extract host:port
        no_frag.split(':').next().unwrap_or(no_frag).to_string()
    }

    fn extract_tag(url: &str) -> String {
        url.split('#')
            .nth(1)
            .map(|s| s.to_string())
            .unwrap_or_else(|| Self::extract_host(url))
    }
}

impl ResourceNormalizer for ProxyUrlNormalizer {
    fn kind(&self) -> ResourceKind {
        ResourceKind::Proxy
    }

    fn normalize(&self, raw: &str) -> Vec<DiscoveredResource> {
        let line = raw.trim();
        if line.is_empty() {
            return Vec::new();
        }

        let supported = [
            "ss://", "ssr://", "vmess://", "trojan://", "vless://",
            "socks5://", "socks4://", "http://", "https://",
        ];
        if !supported.iter().any(|p| line.starts_with(p)) {
            return Vec::new();
        }

        let host = Self::extract_host(line);
        let tag = Self::extract_tag(line);
        let meta = {
            let mut m = HashMap::new();
            m.insert("host".to_string(), host.clone());
            if line.contains("vmess://") {
                m.insert("protocol".to_string(), "vmess".to_string());
            } else if line.contains("vless://") {
                m.insert("protocol".to_string(), "vless".to_string());
            } else if line.contains("trojan://") {
                m.insert("protocol".to_string(), "trojan".to_string());
            } else if line.contains("ssr://") {
                m.insert("protocol".to_string(), "ssr".to_string());
            } else if line.contains("ss://") {
                m.insert("protocol".to_string(), "ss".to_string());
            } else if line.contains("socks5://") {
                m.insert("protocol".to_string(), "socks5".to_string());
            } else if line.contains("socks4://") {
                m.insert("protocol".to_string(), "socks4".to_string());
            } else if line.contains("https://") {
                m.insert("protocol".to_string(), "https".to_string());
            } else {
                m.insert("protocol".to_string(), "http".to_string());
            }
            m.insert("tag".to_string(), tag.clone());
            m
        };

        vec![DiscoveredResource {
            kind: ResourceKind::Proxy,
            resource_id: line.to_string(),
            label: tag,
            source_url: None,
            is_free: true,
            requires_auth: false,
            meta,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_url_normalizer_http() {
        let n = ProxyUrlNormalizer::new();
        let r = n.normalize("http://proxy.example.com:8080");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ResourceKind::Proxy);
    }

    #[test]
    fn test_proxy_url_normalizer_vless() {
        let n = ProxyUrlNormalizer::new();
        let r = n.normalize("vless://uuid@example.com:443?security=tls#TAG");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].label, "TAG");
        assert_eq!(r[0].resource_id, "vless://uuid@example.com:443?security=tls#TAG");
    }

    #[test]
    fn test_proxy_url_normalizer_skip_empty() {
        let n = ProxyUrlNormalizer::new();
        assert!(n.normalize("").is_empty());
        assert!(n.normalize("not a proxy").is_empty());
    }
}
