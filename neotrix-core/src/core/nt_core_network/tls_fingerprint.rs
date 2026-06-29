use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TlsFingerprintProfile {
    Chrome,
    Firefox,
    Safari,
    SafariiOS,
    Edge,
}

impl TlsFingerprintProfile {
    pub fn variants() -> &'static [Self] {
        &[
            Self::Chrome,
            Self::Firefox,
            Self::Safari,
            Self::SafariiOS,
            Self::Edge,
        ]
    }

    pub fn default_config(&self) -> TlsFingerprintConfig {
        let (ja3, http2_settings, ua, h2_pref) = match self {
            Self::Chrome => (
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
                vec![
                    H2Setting { key: 1, value: 65536 },
                    H2Setting { key: 2, value: 0 },
                    H2Setting { key: 3, value: 1000 },
                    H2Setting { key: 4, value: 6291456 },
                    H2Setting { key: 5, value: 15663105 },
                    H2Setting { key: 6, value: 262144 },
                ],
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                HttpVersionPref::Http2,
            ),
            Self::Firefox => (
                "771,4865-4867-4866-49195-49199-52393-52392-49200-49196-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
                vec![
                    H2Setting { key: 1, value: 65536 },
                    H2Setting { key: 2, value: 0 },
                    H2Setting { key: 3, value: 1000 },
                    H2Setting { key: 4, value: 131072 },
                    H2Setting { key: 5, value: 16384 },
                    H2Setting { key: 6, value: 131072 },
                ],
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:120.0) Gecko/20100101 Firefox/120.0",
                HttpVersionPref::Http2,
            ),
            Self::Safari => (
                "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
                vec![
                    H2Setting { key: 1, value: 65536 },
                    H2Setting { key: 2, value: 0 },
                    H2Setting { key: 3, value: 100 },
                    H2Setting { key: 4, value: 2097152 },
                    H2Setting { key: 5, value: 16384 },
                    H2Setting { key: 6, value: 65536 },
                ],
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
                HttpVersionPref::Http2,
            ),
            Self::SafariiOS => (
                "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
                vec![
                    H2Setting { key: 1, value: 65536 },
                    H2Setting { key: 2, value: 0 },
                    H2Setting { key: 3, value: 100 },
                    H2Setting { key: 4, value: 2097152 },
                    H2Setting { key: 5, value: 16384 },
                    H2Setting { key: 6, value: 65536 },
                ],
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
                HttpVersionPref::Http2,
            ),
            Self::Edge => (
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
                vec![
                    H2Setting { key: 1, value: 65536 },
                    H2Setting { key: 2, value: 0 },
                    H2Setting { key: 3, value: 1000 },
                    H2Setting { key: 4, value: 6291456 },
                    H2Setting { key: 5, value: 15663105 },
                    H2Setting { key: 6, value: 262144 },
                ],
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
                HttpVersionPref::Http2,
            ),
        };

        TlsFingerprintConfig {
            profile: *self,
            ja3: ja3.to_string(),
            ja3n: ja3.to_string(),
            http2_settings,
            alpn: vec!["h2".into(), "http/1.1".into()],
            tls_versions: vec![TlsVersion::Tls12, TlsVersion::Tls13],
            http_version: h2_pref,
            user_agent: ua.to_string(),
            accept_invalid_certs: false,
            connect_timeout_secs: 10,
            request_timeout_secs: 30,
        }
    }

    pub fn header_order(&self) -> Vec<&'static str> {
        match self {
            Self::Chrome | Self::Edge => vec![
                "host", "connection", "sec-ch-ua", "sec-ch-ua-mobile", "sec-ch-ua-platform",
                "user-agent", "accept", "sec-fetch-site", "sec-fetch-mode", "sec-fetch-dest",
                "accept-encoding", "accept-language", "cookie",
            ],
            Self::Firefox => vec![
                "host", "user-agent", "accept", "accept-language", "accept-encoding",
                "connection", "cookie", "upgrade-insecure-requests",
                "sec-fetch-dest", "sec-fetch-mode", "sec-fetch-site", "priority",
            ],
            Self::Safari | Self::SafariiOS => vec![
                "host", "user-agent", "accept", "accept-language", "accept-encoding",
                "connection", "cookie",
            ],
        }
    }

    pub fn user_agent(&self) -> String {
        self.default_config().user_agent
    }
}

impl fmt::Display for TlsFingerprintProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chrome => write!(f, "Chrome"),
            Self::Firefox => write!(f, "Firefox"),
            Self::Safari => write!(f, "Safari"),
            Self::SafariiOS => write!(f, "SafariiOS"),
            Self::Edge => write!(f, "Edge"),
        }
    }
}

impl FromStr for TlsFingerprintProfile {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "chrome" => Ok(Self::Chrome),
            "firefox" => Ok(Self::Firefox),
            "safari" => Ok(Self::Safari),
            "safariios" | "safari_ios" => Ok(Self::SafariiOS),
            "edge" => Ok(Self::Edge),
            _ => Err(format!("Unknown TLS fingerprint profile: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HttpVersionPref {
    Http11,
    Http2,
    Http2PriorKnowledge,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct H2Setting {
    pub key: u16,
    pub value: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TlsFingerprintConfig {
    pub profile: TlsFingerprintProfile,
    pub ja3: String,
    pub ja3n: String,
    pub http2_settings: Vec<H2Setting>,
    pub alpn: Vec<String>,
    pub tls_versions: Vec<TlsVersion>,
    pub http_version: HttpVersionPref,
    pub user_agent: String,
    pub accept_invalid_certs: bool,
    pub connect_timeout_secs: u64,
    pub request_timeout_secs: u64,
}

impl Default for TlsFingerprintConfig {
    fn default() -> Self {
        TlsFingerprintProfile::Chrome.default_config()
    }
}

impl TlsFingerprintConfig {
    pub fn build_reqwest_client(
        &self,
        proxy: Option<reqwest::Proxy>,
    ) -> Result<reqwest::Client, String> {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.request_timeout_secs))
            .connect_timeout(Duration::from_secs(self.connect_timeout_secs))
            .user_agent(&self.user_agent)
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(15))
            .no_proxy();

        if self.accept_invalid_certs {
            builder = builder.danger_accept_invalid_certs(true);
        }

        match self.http_version {
            HttpVersionPref::Http11 => {
                builder = builder.http1_only();
            }
            HttpVersionPref::Http2 => {}
            HttpVersionPref::Http2PriorKnowledge => {
                builder = builder.http2_prior_knowledge();
            }
        }

        if let Some(p) = proxy {
            builder = builder.proxy(p);
        }

        builder.build().map_err(|e| format!("reqwest client build failed: {}", e))
    }

    pub fn apply_to_headers(&self, headers: &mut [(&str, String)]) {
        let order = TlsFingerprintProfile::header_order(&self.profile);
        headers.sort_by(|a, b| {
            let a_pos = order
                .iter()
                .position(|h| h.eq_ignore_ascii_case(a.0))
                .unwrap_or(usize::MAX);
            let b_pos = order
                .iter()
                .position(|h| h.eq_ignore_ascii_case(b.0))
                .unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });
    }
}

pub fn default_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        TlsFingerprintConfig::default()
            .build_reqwest_client(None)
            .expect("default reqwest client")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiles_have_unique_ja3() {
        let mut ja3s = std::collections::HashSet::new();
        for p in TlsFingerprintProfile::variants() {
            let cfg = p.default_config();
            assert!(ja3s.insert(cfg.ja3), "duplicate JA3 for {:?}", p);
        }
    }

    #[test]
    fn test_profile_from_str() {
        assert_eq!("chrome".parse::<TlsFingerprintProfile>().unwrap(), TlsFingerprintProfile::Chrome);
        assert_eq!("Firefox".parse::<TlsFingerprintProfile>().unwrap(), TlsFingerprintProfile::Firefox);
        assert!("unknown".parse::<TlsFingerprintProfile>().is_err());
    }

    #[test]
    fn test_default_config_builds_client() {
        let cfg = TlsFingerprintConfig::default();
        let client = cfg.build_reqwest_client(None);
        assert!(client.is_ok());
    }

    #[test]
    fn test_chrome_vs_firefox_h2_settings_differ() {
        let chrome = TlsFingerprintProfile::Chrome.default_config();
        let ff = TlsFingerprintProfile::Firefox.default_config();
        assert_ne!(chrome.http2_settings, ff.http2_settings);
    }

    #[test]
    fn test_safari_header_order_compact() {
        let order = TlsFingerprintProfile::Safari.header_order();
        assert!(!order.contains(&"sec-ch-ua"));
    }

    #[test]
    fn test_apply_to_headers_reorders() {
        let cfg = TlsFingerprintProfile::Chrome.default_config();
        let mut headers = vec![
            ("accept", "text/html".into()),
            ("user-agent", "test".into()),
            ("host", "example.com".into()),
        ];
        cfg.apply_to_headers(&mut headers);
        assert_eq!(headers[0].0, "host");
    }

    #[test]
    fn test_firefox_has_priority_header() {
        let order = TlsFingerprintProfile::Firefox.header_order();
        assert!(order.contains(&"priority"));
    }

    #[test]
    fn test_edge_same_as_chrome_header_order() {
        assert_eq!(
            TlsFingerprintProfile::Chrome.header_order(),
            TlsFingerprintProfile::Edge.header_order(),
        );
    }
}
