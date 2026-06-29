use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const MAX_SOURCE_FAILURES: u32 = 3;
const SOURCE_COOLDOWN_SECS: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl ProxyProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyProtocol::Http => "http",
            ProxyProtocol::Https => "https",
            ProxyProtocol::Socks4 => "socks4",
            ProxyProtocol::Socks5 => "socks5",
        }
    }

    pub fn url_scheme(&self) -> &'static str {
        match self {
            ProxyProtocol::Http => "http://",
            ProxyProtocol::Https => "https://",
            ProxyProtocol::Socks4 => "socks4://",
            ProxyProtocol::Socks5 => "socks5://",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyQualityTier {
    S,
    A,
    B,
    C,
    D,
}

impl ProxyQualityTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyQualityTier::S => "S",
            ProxyQualityTier::A => "A",
            ProxyQualityTier::B => "B",
            ProxyQualityTier::C => "C",
            ProxyQualityTier::D => "D",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxySourceDef {
    pub name: &'static str,
    pub url: &'static str,
    pub protocols: &'static [ProxyProtocol],
    pub format: SourceFormat,
    pub weight: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourceFormat {
    PlainTxt,
    JsonArray,
    JsonApi,
}

pub const PROXIFLY_ALL: ProxySourceDef = ProxySourceDef {
    name: "proxifly-all",
    url: "https://cdn.jsdelivr.net/gh/proxifly/free-proxy-list@main/proxies/all/data.txt",
    protocols: &[
        ProxyProtocol::Http,
        ProxyProtocol::Https,
        ProxyProtocol::Socks4,
        ProxyProtocol::Socks5,
    ],
    format: SourceFormat::PlainTxt,
    weight: 100,
};

pub const PROXIFLY_HTTP: ProxySourceDef = ProxySourceDef {
    name: "proxifly-http",
    url:
        "https://cdn.jsdelivr.net/gh/proxifly/free-proxy-list@main/proxies/protocols/http/data.txt",
    protocols: &[ProxyProtocol::Http],
    format: SourceFormat::PlainTxt,
    weight: 30,
};

pub const PROXIFLY_HTTPS: ProxySourceDef = ProxySourceDef {
    name: "proxifly-https",
    url:
        "https://cdn.jsdelivr.net/gh/proxifly/free-proxy-list@main/proxies/protocols/https/data.txt",
    protocols: &[ProxyProtocol::Https],
    format: SourceFormat::PlainTxt,
    weight: 30,
};

pub const PROXIFLY_SOCKS4: ProxySourceDef = ProxySourceDef {
    name: "proxifly-socks4",
    url: "https://cdn.jsdelivr.net/gh/proxifly/free-proxy-list@main/proxies/protocols/socks4/data.txt",
    protocols: &[ProxyProtocol::Socks4],
    format: SourceFormat::PlainTxt,
    weight: 30,
};

pub const PROXIFLY_SOCKS5: ProxySourceDef = ProxySourceDef {
    name: "proxifly-socks5",
    url: "https://cdn.jsdelivr.net/gh/proxifly/free-proxy-list@main/proxies/protocols/socks5/data.txt",
    protocols: &[ProxyProtocol::Socks5],
    format: SourceFormat::PlainTxt,
    weight: 30,
};

pub const GEONODE: ProxySourceDef = ProxySourceDef {
    name: "geonode",
    url: "https://proxylist.geonode.com/api/proxy-list?limit=100&page=1&sort_by=lastChecked&sort_type=desc",
    protocols: &[ProxyProtocol::Http, ProxyProtocol::Https, ProxyProtocol::Socks4, ProxyProtocol::Socks5],
    format: SourceFormat::JsonApi,
    weight: 50,
};

pub const PROXY_LIST_DOWNLOAD: ProxySourceDef = ProxySourceDef {
    name: "proxy-list-download",
    url: "https://www.proxy-list.download/api/v1/get?type=http",
    protocols: &[ProxyProtocol::Http],
    format: SourceFormat::PlainTxt,
    weight: 20,
};

pub const PROXYSCRAPE: ProxySourceDef = ProxySourceDef {
    name: "proxyscrape",
    url: "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all",
    protocols: &[ProxyProtocol::Http],
    format: SourceFormat::PlainTxt,
    weight: 20,
};

pub const ALL_FREE_SOURCES: &[ProxySourceDef] =
    &[PROXIFLY_ALL, GEONODE, PROXY_LIST_DOWNLOAD, PROXYSCRAPE];

pub const ALL_PROXIFLY_SOURCES: &[ProxySourceDef] = &[
    PROXIFLY_HTTP,
    PROXIFLY_HTTPS,
    PROXIFLY_SOCKS4,
    PROXIFLY_SOCKS5,
];

#[derive(Debug, Clone)]
pub struct SourceHealth {
    pub consecutive_failures: u32,
    pub total_failures: u64,
    pub total_successes: u64,
    pub last_attempt: Option<Instant>,
    pub cooldown_until: Option<Instant>,
    pub last_error: Option<String>,
}

impl Default for SourceHealth {
    fn default() -> Self {
        Self {
            consecutive_failures: 0,
            total_failures: 0,
            total_successes: 0,
            last_attempt: None,
            cooldown_until: None,
            last_error: None,
        }
    }
}

impl SourceHealth {
    pub fn is_on_cooldown(&self) -> bool {
        self.cooldown_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false)
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.total_successes += 1;
        self.last_attempt = Some(Instant::now());
        self.cooldown_until = None;
    }

    pub fn record_failure(&mut self, error: String) {
        self.consecutive_failures += 1;
        self.total_failures += 1;
        self.last_attempt = Some(Instant::now());
        self.last_error = Some(error);
        if self.consecutive_failures >= MAX_SOURCE_FAILURES {
            self.cooldown_until = Some(Instant::now() + Duration::from_secs(SOURCE_COOLDOWN_SECS));
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_successes + self.total_failures;
        if total == 0 {
            0.5
        } else {
            self.total_successes as f64 / total as f64
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawProxy {
    pub ip: String,
    pub port: u16,
    pub protocol: ProxyProtocol,
    pub source: Option<&'static str>,
}

impl RawProxy {
    pub fn to_proxy_url(&self) -> String {
        format!("{}{}:{}", self.protocol.url_scheme(), self.ip, self.port)
    }

    pub fn to_host_port(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone)]
pub struct ProxyValidationResult {
    pub connect_ok: bool,
    pub targets_passed: usize,
    pub targets_total: usize,
    pub avg_latency_ms: Option<f64>,
}

impl ProxyValidationResult {
    pub fn pass_rate(&self) -> f64 {
        if self.targets_total == 0 {
            return if self.connect_ok { 1.0 } else { 0.0 };
        }
        self.targets_passed as f64 / self.targets_total as f64
    }

    pub fn classify_quality(&self) -> ProxyQualityTier {
        if !self.connect_ok {
            return ProxyQualityTier::D;
        }
        let rate = self.pass_rate();
        let latency = self.avg_latency_ms.unwrap_or(f64::MAX);
        match (rate, latency) {
            (r, l) if r >= 0.8 && l < 500.0 => ProxyQualityTier::S,
            (r, _) if r >= 0.8 => ProxyQualityTier::A,
            (r, _) if r >= 0.5 => ProxyQualityTier::B,
            (r, _) if r >= 0.2 => ProxyQualityTier::C,
            _ => ProxyQualityTier::D,
        }
    }
}

pub const DEFAULT_VALIDATION_TARGETS: &[(&str, u16)] =
    &[("httpbin.org", 80), ("example.com", 80), ("google.com", 80)];
