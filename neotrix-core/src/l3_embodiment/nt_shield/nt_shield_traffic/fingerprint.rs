// P13: TlsFingerprint (吸收 lwthiker/curl-impersonate + apify/crawlee)
// 客户端级 TLS/HTTP2 指纹复刻 (JA3/JA4): 编译期签名库 + 浏览器风格 header 生成。
// 注入 HTTP fetch 层获得轻量级反指纹能力 (无需浏览器级指纹)。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsFingerprint {
    pub name: &'static str,
    pub ja3: &'static str,
    pub ja4: &'static str,
    /// HTTP/2 settings (帧名 → 值), 复刻真实浏览器握手
    pub http2_settings: &'static [(&'static str, u32)],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserHeaders {
    pub user_agent: &'static str,
    pub accept: &'static str,
    pub accept_language: &'static str,
    pub sec_ch_ua: &'static str,
}

impl TlsFingerprint {
    pub const CHROME_116: TlsFingerprint = TlsFingerprint {
        name: "chrome116",
        ja3: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
        ja4: "t13d1717h2_8daaf6152771_02713d6af862",
        http2_settings: &[
            ("HEADER_TABLE_SIZE", 65536),
            ("ENABLE_PUSH", 0),
            ("INITIAL_WINDOW_SIZE", 6291456),
            ("MAX_FRAME_SIZE", 16384),
        ],
    };

    pub const FIREFOX_128: TlsFingerprint = TlsFingerprint {
        name: "firefox128",
        ja3: "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-51-45-43-27-21,29-23-24,0",
        ja4: "t13d1717h2_002f16f0262a_02713d6af862",
        http2_settings: &[
            ("HEADER_TABLE_SIZE", 65536),
            ("ENABLE_PUSH", 0),
            ("INITIAL_WINDOW_SIZE", 131072),
            ("MAX_FRAME_SIZE", 16384),
            ("MAX_HEADER_LIST_SIZE", 262144),
        ],
    };

    pub const SAFARI_17: TlsFingerprint = TlsFingerprint {
        name: "safari17",
        ja3: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
        ja4: "t13d1717h2_8daaf6152771_02713d6af862",
        http2_settings: &[
            ("HEADER_TABLE_SIZE", 65536),
            ("ENABLE_PUSH", 0),
            ("INITIAL_WINDOW_SIZE", 4194304),
            ("MAX_FRAME_SIZE", 16384),
        ],
    };

    pub const EDGE_114: TlsFingerprint = TlsFingerprint {
        name: "edge114",
        ja3: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
        ja4: "t13d1717h2_8daaf6152771_02713d6af862",
        http2_settings: &[
            ("HEADER_TABLE_SIZE", 65536),
            ("ENABLE_PUSH", 0),
            ("INITIAL_WINDOW_SIZE", 6291456),
            ("MAX_FRAME_SIZE", 16384),
        ],
    };

    pub fn browser_headers(&self) -> BrowserHeaders {
        match self.name {
            "firefox128" => BrowserHeaders {
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0",
                accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                accept_language: "en-US,en;q=0.5",
                sec_ch_ua: "",
            },
            _ => BrowserHeaders {
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36",
                accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
                accept_language: "en-US,en;q=0.9",
                sec_ch_ua: "\"Chromium\";v=\"116\", \"Not)A;Brand\";v=\"24\", \"Google Chrome\";v=\"116\"",
            },
        }
    }
}

/// 浏览器签名库 (browsers.json 语义): 按名查找指纹。
pub struct FingerprintStore {
    map: HashMap<&'static str, TlsFingerprint>,
}

impl Default for FingerprintStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FingerprintStore {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        for fp in Self::all() {
            map.insert(fp.name, fp);
        }
        Self { map }
    }

    pub fn all() -> Vec<TlsFingerprint> {
        vec![
            TlsFingerprint::CHROME_116,
            TlsFingerprint::FIREFOX_128,
            TlsFingerprint::SAFARI_17,
            TlsFingerprint::EDGE_114,
        ]
    }

    pub fn by_name(&self, name: &str) -> Option<&TlsFingerprint> {
        self.map.get(name)
    }

    /// 生成随请求头的模拟 (curl_easy_impersonate 语义): 指纹 + 浏览器头。
    pub fn impersonate(&self, name: &str) -> Option<(TlsFingerprint, BrowserHeaders)> {
        let fp = self.by_name(name)?;
        Some((fp.clone(), fp.browser_headers()))
    }

    /// 零配置人类化指纹选择 (crawlee human-like fingerprints): 确定性伪随机。
    pub fn human_like(&self, seed: u64) -> TlsFingerprint {
        let all = Self::all();
        all[(seed as usize) % all.len()].clone()
    }
}

impl crate::core::nt_core_self_test::SelfTest for FingerprintStore {
    fn name(&self) -> &str {
        "nt_shield_tls_fingerprint"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let store = FingerprintStore::new();
        let fp = store.by_name("chrome116").ok_or_else(|| {
            vec!["chrome116 preset missing".into()]
        })?;
        if fp.ja3.is_empty() || fp.ja4.is_empty() {
            return Err(vec!["fingerprint must carry JA3 and JA4".into()]);
        }
        let (fp2, _h) = store.impersonate("firefox128").ok_or_else(|| {
            vec!["firefox128 impersonate failed".into()]
        })?;
        if fp2.name != "firefox128" {
            return Err(vec!["wrong fingerprint returned".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_store_has_all_presets() {
        let store = FingerprintStore::new();
        assert_eq!(store.map.len(), 4);
        for name in ["chrome116", "firefox128", "safari17", "edge114"] {
            assert!(store.by_name(name).is_some(), "missing {name}");
        }
    }

    #[test]
    fn test_impersonate_returns_headers() {
        let store = FingerprintStore::new();
        let (fp, h) = store.impersonate("chrome116").expect("chrome");
        assert!(h.user_agent.contains("Chrome/116"));
        assert!(!fp.ja3.is_empty());
        assert!(!h.accept_language.is_empty());
    }

    #[test]
    fn test_unknown_name_none() {
        let store = FingerprintStore::new();
        assert!(store.impersonate("nonexistent").is_none());
    }

    #[test]
    fn test_human_like_deterministic() {
        let store = FingerprintStore::new();
        let a = store.human_like(42);
        let b = store.human_like(42);
        assert_eq!(a.name, b.name);
    }

    #[test]
    fn test_firefox_headers_differ_from_chrome() {
        let store = FingerprintStore::new();
        let (_, hf) = store.impersonate("firefox128").unwrap();
        let (_, hc) = store.impersonate("chrome116").unwrap();
        assert_ne!(hf.user_agent, hc.user_agent);
        assert_ne!(hf.sec_ch_ua, hc.sec_ch_ua);
    }

    #[test]
    fn test_http2_settings_present() {
        let fp = TlsFingerprint::CHROME_116;
        assert!(fp.http2_settings.iter().any(|(k, _)| *k == "INITIAL_WINDOW_SIZE"));
    }

    #[test]
    fn test_selftest() {
        let store = FingerprintStore::new();
        assert!(store.self_test().is_ok());
    }
}