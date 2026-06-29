use crate::neotrix::nt_io_http_factory::TlsFingerprint;

/// TLS fingerprint configuration with browser-specific parameters
#[derive(Debug, Clone)]
pub struct TlsFingerprintConfig {
    pub fingerprint: TlsFingerprint,
    pub http_version: HttpVersionPreference,
    pub alpn: Vec<String>,
    pub header_order: Vec<&'static str>,
}

impl Default for TlsFingerprintConfig {
    fn default() -> Self {
        let fp = TlsFingerprint::default();
        Self {
            fingerprint: fp,
            http_version: HttpVersionPreference::Http2,
            alpn: vec!["h2".into(), "http/1.1".into()],
            header_order: fp.default_header_order().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersionPreference {
    Http1Only,
    Http2,
    Http2PriorKnowledge,
}

impl TlsFingerprint {
    pub fn ja3_fingerprint(&self) -> String {
        match self {
            TlsFingerprint::Chrome116 => {
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0".into()
            }
            TlsFingerprint::Chrome120 => {
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0".into()
            }
            TlsFingerprint::Firefox117 => {
                "771,4865-4867-4866-49195-49199-52393-52392-49200-49196-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0".into()
            }
            TlsFingerprint::Firefox120 => {
                "771,4865-4867-4866-49195-49199-52393-52392-49200-49196-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0".into()
            }
            TlsFingerprint::Safari17 => {
                "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0".into()
            }
            TlsFingerprint::Edge120 => {
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0".into()
            }
            TlsFingerprint::Custom(ja3) => ja3.to_string(),
        }
    }

    pub fn http2_settings(&self) -> Vec<(u16, u32)> {
        match self {
            TlsFingerprint::Chrome116 | TlsFingerprint::Chrome120 | TlsFingerprint::Edge120 => {
                vec![
                    (1, 65536),
                    (2, 0),
                    (3, 1000),
                    (4, 6291456),
                    (5, 15663105),
                    (6, 262144),
                ]
            }
            TlsFingerprint::Firefox117 | TlsFingerprint::Firefox120 => {
                vec![
                    (1, 65536),
                    (2, 0),
                    (3, 1000),
                    (4, 131072),
                    (5, 16384),
                    (6, 131072),
                ]
            }
            TlsFingerprint::Safari17 => {
                vec![
                    (1, 65536),
                    (2, 0),
                    (3, 100),
                    (4, 2097152),
                    (5, 16384),
                    (6, 65536),
                ]
            }
            TlsFingerprint::Custom(_) => {
                vec![
                    (1, 65536),
                    (2, 0),
                    (3, 1000),
                    (4, 6291456),
                    (5, 15663105),
                    (6, 262144),
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_fingerprints_have_non_empty_ja3() {
        let cases = vec![
            TlsFingerprint::Chrome116,
            TlsFingerprint::Chrome120,
            TlsFingerprint::Firefox117,
            TlsFingerprint::Firefox120,
            TlsFingerprint::Safari17,
            TlsFingerprint::Edge120,
        ];
        for fp in cases {
            let ja3 = fp.ja3_fingerprint();
            assert!(!ja3.is_empty(), "JA3 for {:?} should not be empty", fp);
            assert!(
                ja3.split(',').count() >= 5,
                "JA3 for {:?} malformed: {}",
                fp,
                ja3
            );
        }
    }

    #[test]
    fn test_custom_fingerprint_returns_stored_string() {
        let fp = TlsFingerprint::Custom("771,4865-4866,0-23,29,0");
        assert_eq!(fp.ja3_fingerprint(), "771,4865-4866,0-23,29,0");
    }

    #[test]
    fn test_http2_settings_not_empty() {
        let fp = TlsFingerprint::Chrome116;
        let settings = fp.http2_settings();
        assert!(!settings.is_empty());
        for (key, val) in &settings {
            assert!(*key >= 1 && *key <= 6, "Invalid H2 setting key: {}", key);
            assert!(*val > 0, "H2 setting {} should be positive", key);
        }
    }

    #[test]
    fn test_firefox_settings_differ_from_chrome() {
        let chrome = TlsFingerprint::Chrome116.http2_settings();
        let ff = TlsFingerprint::Firefox120.http2_settings();
        assert_ne!(
            chrome, ff,
            "Chrome and Firefox should have different H2 settings"
        );
    }

    #[test]
    fn test_safari_settings_have_lower_stream_limit() {
        let settings = TlsFingerprint::Safari17.http2_settings();
        let max_streams = settings
            .iter()
            .find(|(k, _)| *k == 3)
            .map(|(_, v)| *v)
            .unwrap_or(0);
        assert!(
            max_streams <= 100,
            "Safari should limit concurrent streams to <=100"
        );
    }

    #[test]
    fn test_config_default() {
        let cfg = TlsFingerprintConfig::default();
        assert_eq!(cfg.fingerprint, TlsFingerprint::Chrome116);
        assert_eq!(cfg.http_version, HttpVersionPreference::Http2);
        assert!(cfg.alpn.contains(&"h2".to_string()));
    }
}
