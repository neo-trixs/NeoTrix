//! 自动更新规则引擎 — 多源 GeoIP + 域名规则
//!
//! 数据源评测:
//! [19.7k⭐] Loyalsoldier/v2ray-rules-dat  — 每天更新，中国IP+域名最全
//! [25.3k⭐] gfwlist/gfwlist               — 被墙域名规范列表
//! [8.6k⭐ ] v2fly/domain-list-community    — 社区域名分类
//! [726⭐  ] mayaxcn/china-ip-list          — 每小时 APNIC 原始数据
//!
//! 架构: GeoDatabase(IP范围) + DomainRules(域名集合) → RuleUpdater(自动更新)

use std::collections::HashSet;
use std::fs;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

use base64::Engine;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::RwLock;

/// DNS Geo 缓存: domain → (is_china, timestamp)
/// TTL 300s，超时自动驱逐。
const GEO_CACHE_TTL_SECS: u64 = 300;
static GEO_CACHE: OnceLock<std::sync::Mutex<HashMap<String, (bool, Instant)>>> = OnceLock::new();

fn geo_cache_get(domain: &str) -> Option<bool> {
    let cache = GEO_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    let mut map = match cache.lock() {
        Ok(m) => m,
        Err(e) => {
            log::warn!("[geo] cache lock: {}", e);
            return None;
        }
    };
    if let Some((is_china, ts)) = map.get(domain) {
        if ts.elapsed().as_secs() < GEO_CACHE_TTL_SECS {
            return Some(*is_china);
        }
        map.remove(domain);
    }
    None
}

fn geo_cache_set(domain: &str, is_china: bool) {
    let cache = GEO_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    if let Ok(mut map) = cache.lock() {
        map.insert(domain.to_string(), (is_china, Instant::now()));
    }
}

// ==================== 数据源 ====================

/// Tier1: 最全面的中国 IP + 域名规则 (每日更新)
const LOYALSOLDIER_CHINA_LIST: &str =
    "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/china-list.txt";
const LOYALSOLDIER_DIRECT_LIST: &str =
    "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/direct-list.txt";
const LOYALSOLDIER_GFW_LIST: &str =
    "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/gfw.txt";

/// Tier2: APNIC 原始分配数据 (每小时更新)
const MAYAXCN_CHINA_IP: &str =
    "https://raw.githubusercontent.com/mayaxcn/china-ip-list/master/chn_ip.txt";

/// Tier3: GFWList 被墙域名 (持续更新)
const GFWLIST_URL: &str =
    "https://raw.githubusercontent.com/gfwlist/gfwlist/master/gfwlist.txt";

// ==================== GeoDatabase ====================

/// 地理位置数据库 — 中国 IP 范围
pub struct GeoDatabase {
    ranges: Vec<(u32, u32)>,
    loaded_from: Option<String>,
}

impl GeoDatabase {
    pub fn new() -> Self {
        Self { ranges: Vec::new(), loaded_from: None }
    }

    pub fn is_china_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => {
                let ip = u32::from(*v4);
                // partition_point: 找到最后一个 start <= ip 的区间
                let idx = self.ranges.partition_point(|&(start, _)| start <= ip);
                if idx == 0 {
                    return false;
                }
                let (_start, end) = self.ranges[idx - 1];
                ip <= end
            }
            IpAddr::V6(_) => false,
        }
    }

    pub fn load_cidr_text(&mut self, text: &str, source: &str) -> usize {
        self.ranges = parse_cidr_list(text);
        self.loaded_from = Some(source.to_string());
        self.ranges.len()
    }

    pub fn count(&self) -> usize {
        self.ranges.len()
    }

    pub fn loaded_source(&self) -> Option<&str> {
        self.loaded_from.as_deref()
    }
}

impl Default for GeoDatabase {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== DomainRules ====================

/// 域名规则 — 直连/代理判定
#[derive(Default)]
pub struct DomainRules {
    /// 应直连的域名（中国网站）
    pub direct_domains: HashSet<String>,
    /// 应走代理的域名（被墙网站）
    pub proxy_domains: HashSet<String>,
}

impl DomainRules {
    pub fn new() -> Self {
        Self {
            direct_domains: HashSet::new(),
            proxy_domains: HashSet::new(),
        }
    }

    /// 判断域名应直连 (true=直连, false=代理, None=未知)
    pub fn should_direct(&self, domain: &str) -> Option<bool> {
        if self.direct_domains.contains(domain) {
            return Some(true);
        }
        if self.proxy_domains.contains(domain) {
            return Some(false);
        }
        // 后缀匹配: domain 是否以 .stored_domain 结尾
        if self.direct_domains.iter().any(|s| domain == s || domain.ends_with(&format!(".{}", s))) {
            return Some(true);
        }
        if self.proxy_domains.iter().any(|s| domain == s || domain.ends_with(&format!(".{}", s))) {
            return Some(false);
        }
        None
    }

    pub fn load_direct_text(&mut self, text: &str) -> usize {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("@@") || line.starts_with('!') {
                continue;
            }
            let domain = line.trim_start_matches("||").trim_end_matches('^');
            if !domain.is_empty() && !domain.contains('*') && !domain.contains('/') {
                self.direct_domains.insert(domain.to_string());
            }
        }
        self.direct_domains.len()
    }

    pub fn load_proxy_text(&mut self, text: &str) -> usize {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') || line.starts_with('[') {
                continue;
            }
            let domain = line.trim_start_matches("||").trim_end_matches('^').trim_end_matches('/');
            if !domain.is_empty() && !domain.contains('*') && !domain.contains('/') {
                self.proxy_domains.insert(domain.to_string());
            }
        }
        self.proxy_domains.len()
    }
}

// ==================== 全局状态 ====================

static GLOBAL_GEO: OnceLock<Arc<RwLock<GeoDatabase>>> = OnceLock::new();
static GLOBAL_DOMAINS: OnceLock<Arc<RwLock<DomainRules>>> = OnceLock::new();

pub fn global_geo() -> Arc<RwLock<GeoDatabase>> {
    GLOBAL_GEO.get_or_init(|| Arc::new(RwLock::new(GeoDatabase::new()))).clone()
}

pub fn global_domains() -> Arc<RwLock<DomainRules>> {
    GLOBAL_DOMAINS.get_or_init(|| Arc::new(RwLock::new(DomainRules::new()))).clone()
}

// ==================== RuleUpdater ====================

/// 规则更新器 — 从多个 GitHub 源拉取最新规则
pub struct RuleUpdater {
    cache_dir: PathBuf,
}

impl RuleUpdater {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub fn default_cache() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("geo")
    }

    /// 更新所有规则源
    pub async fn update_all(&self) -> UpdateReport {
        let mut report = UpdateReport::default();

        // 1. 中国 IP 范围 (Loyalsoldier 为主, mayaxcn 为备)
        if let Err(e) = self.update_china_ip(&mut report).await {
            report.errors.push(format!("china-ip: {}", e));
        }

        // 2. 直连域名
        if let Err(e) = self.update_direct_domains(&mut report).await {
            report.errors.push(format!("direct: {}", e));
        }

        // 3. 代理域名 (GFWList)
        if let Err(e) = self.update_proxy_domains(&mut report).await {
            report.errors.push(format!("proxy: {}", e));
        }

        report
    }

    async fn update_china_ip(&self, report: &mut UpdateReport) -> Result<(), String> {
        let geo_arc = global_geo();
        let mut geo = geo_arc.write().await;

        // Tier1: Loyalsoldier (最全)
        match fetch_text(LOYALSOLDIER_CHINA_LIST).await {
            Ok(text) => {
                let count = geo.load_cidr_text(&text, "Loyalsoldier/china-list");
                report.china_ip_count = count;
                report.china_ip_source = "Loyalsoldier".into();
                self.cache_write("china-list.txt", &text);
                return Ok(());
            }
            Err(e) => report.errors.push(format!("Loyalsoldier fallback: {}", e)),
        }

        // Tier2: mayaxcn (APNIC hourly)
        match fetch_text(MAYAXCN_CHINA_IP).await {
            Ok(text) => {
                let count = geo.load_cidr_text(&text, "mayaxcn/chn_ip");
                report.china_ip_count = count;
                report.china_ip_source = "mayaxcn".into();
                self.cache_write("chn_ip.txt", &text);
                Ok(())
            }
            Err(e) => Err(format!("mayaxcn also failed: {}", e)),
        }
    }

    async fn update_direct_domains(&self, report: &mut UpdateReport) -> Result<(), String> {
        let text = fetch_text(LOYALSOLDIER_DIRECT_LIST).await?;
        let dom_arc = global_domains();
        let mut domains = dom_arc.write().await;
        let count = domains.load_direct_text(&text);
        report.direct_domain_count = count;
        self.cache_write("direct-list.txt", &text);
        Ok(())
    }

    async fn update_proxy_domains(&self, report: &mut UpdateReport) -> Result<(), String> {
        let mut total = 0usize;

        // Tier1: Loyalsoldier GFW list
        if let Ok(text) = fetch_text(LOYALSOLDIER_GFW_LIST).await {
            let dom_arc = global_domains();
        let mut domains = dom_arc.write().await;
            total = domains.load_proxy_text(&text);
            self.cache_write("gfw.txt", &text);
        }

        // Tier2: GFWList (补充)
        if let Ok(b64) = fetch_text(GFWLIST_URL).await {
            let text = base64::engine::general_purpose::STANDARD
                .decode(b64.trim())
                .map(|v| String::from_utf8_lossy(&v).to_string())
                .unwrap_or_default();
            if !text.is_empty() {
                let dom_arc = global_domains();
        let mut domains = dom_arc.write().await;
                total += domains.load_proxy_text(&text);
                self.cache_write("gfwlist.txt", &text);
            }
        }

        report.proxy_domain_count = total;
        Ok(())
    }

    fn cache_write(&self, name: &str, data: &str) {
        let _ = fs::create_dir_all(&self.cache_dir);
        let path = self.cache_dir.join(name);
        let _ = fs::write(&path, data);
    }

    /// 从缓存加载（离线可用）
    pub fn load_cache(&self) -> UpdateReport {
        let mut report = UpdateReport::default();

        let cache_paths = [
            ("china-list.txt", "china_ip"),
            ("direct-list.txt", "direct_domains"),
            ("gfw.txt", "proxy_domains"),
        ];

        for (name, kind) in &cache_paths {
            let path = self.cache_dir.join(name);
            if let Ok(text) = fs::read_to_string(&path) {
                match *kind {
                    "china_ip" => {
                        if let Some(geo) = GLOBAL_GEO.get() {
                            if let Ok(mut g) = geo.try_write() {
                                let count = g.load_cidr_text(&text, "cache");
                                report.china_ip_count = count;
                                report.china_ip_source = "cache".into();
                            }
                        }
                    }
                    "direct_domains" => {
                        if let Some(dom) = GLOBAL_DOMAINS.get() {
                            if let Ok(mut d) = dom.try_write() {
                                let count = d.load_direct_text(&text);
                                report.direct_domain_count = count;
                            }
                        }
                    }
                    "proxy_domains" => {
                        if let Some(dom) = GLOBAL_DOMAINS.get() {
                            if let Ok(mut d) = dom.try_write() {
                                let count = d.load_proxy_text(&text);
                                report.proxy_domain_count = count;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        report
    }

    /// 启动后台自动更新循环 (默认每6小时)
    pub async fn start_auto_update(self: Arc<Self>, interval_hours: u64) {
        let interval = std::time::Duration::from_secs(interval_hours * 3600);
        loop {
            tokio::time::sleep(interval).await;
            let report = self.update_all().await;
            if !report.errors.is_empty() {
                log::warn!("[geo] auto-update errors: {:?}", report.errors);
            } else {
                log::info!(
                    "[geo] auto-update OK: {} IPs, {} direct, {} proxy",
                    report.china_ip_count, report.direct_domain_count, report.proxy_domain_count
                );
            }
        }
    }
}

// ==================== 公共 API ====================

/// 判断 IP 是否中国 (使用全局数据库)
pub fn is_china_ip(ip: &IpAddr) -> bool {
    if let Some(db) = GLOBAL_GEO.get() {
        // block_in_place 允许在 tokio runtime 内安全阻塞
        let result = tokio::task::block_in_place(|| {
            let guard = db.blocking_read();
            if guard.count() > 0 {
                Some(guard.is_china_ip(ip))
            } else {
                None
            }
        });
        if let Some(r) = result {
            return r;
        }
    }
    fallback_is_china_ip(ip)
}

/// DNS 解析 + IP 归属判断（带 300s TTL 缓存，5s 超时）
pub async fn domain_resolves_to_china(domain: &str) -> Result<bool, String> {
    if let Some(cached) = geo_cache_get(domain) {
        return Ok(cached);
    }
    let domain_owned = domain.to_string();
    let task = tokio::task::spawn_blocking(move || {
        (domain_owned.as_str(), 0)
            .to_socket_addrs()
            .map(|addrs| addrs.map(|a| a.ip()).collect::<Vec<_>>())
    });
    let addrs = tokio::time::timeout(Duration::from_secs(5), task).await
        .map_err(|_| format!("DNS resolve timeout for {}", domain))?
        .map_err(|e| format!("DNS resolve task failed: {}", e))?
        .map_err(|e| format!("DNS resolve failed: {}", e))?;
    let ips: Vec<IpAddr> = addrs;
    if ips.is_empty() {
        return Err(format!("No IP resolved for {}", domain));
    }
    let is_china = ips.iter().all(is_china_ip);
    geo_cache_set(domain, is_china);
    Ok(is_china)
}

/// 域名是否应直连 (规则引擎集成)
pub fn domain_should_direct(domain: &str) -> Option<bool> {
    if let Some(rules) = GLOBAL_DOMAINS.get() {
        let guard = tokio::task::block_in_place(|| rules.blocking_read());
        if guard.direct_domains.len() + guard.proxy_domains.len() > 0 {
            return guard.should_direct(domain);
        }
    }
    None
}

/// 判断错误是否为超时
pub fn is_timeout_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("connection reset")
        || lower.contains("connection refused")
}

/// 更新报告
#[derive(Debug, Default, Clone)]
pub struct UpdateReport {
    pub china_ip_count: usize,
    pub china_ip_source: String,
    pub direct_domain_count: usize,
    pub proxy_domain_count: usize,
    pub errors: Vec<String>,
}

// ==================== 内部工具 ====================

async fn fetch_text(url: &str) -> Result<String, String> {
    reqwest::get(url)
        .await
        .map_err(|e| format!("GET {} failed: {}", url, e))?
        .text()
        .await
        .map_err(|e| format!("read {} failed: {}", url, e))
}

fn parse_cidr_list(text: &str) -> Vec<(u32, u32)> {
    let mut ranges: Vec<(u32, u32)> = text
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            cidr_to_range(line)
        })
        .collect();
    ranges.sort_by_key(|&(start, _)| start);
    ranges
}

fn cidr_to_range(cidr: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    let ip: u32 = match parts[0].parse::<std::net::Ipv4Addr>() {
        Ok(ip) => ip.into(),
        Err(e) => {
            log::warn!("[geo] parse IP: {}", e);
            return None;
        }
    };
    let prefix: u8 = match parts[1].parse() {
        Ok(n) if n <= 32 => n,
        Ok(_) => return None,
        Err(e) => {
            log::warn!("[geo] parse prefix: {}", e);
            return None;
        }
    };
    let mask = if prefix == 0 { 0u32 } else { !0u32 << (32 - prefix) };
    Some((ip & mask, ip | !mask))
}

/// Fallback — 42 个主要 /8 段
const CHINA_V4_FALLBACK: &[(u32, u32)] = &[
    (0x0E000000, 0x0EFFFFFF), (0x1B000000, 0x1BFFFFFF),
    (0x24000000, 0x24FFFFFF), (0x27000000, 0x27FFFFFF),
    (0x2A000000, 0x2AFFFFFF), (0x31000000, 0x31FFFFFF),
    (0x3A000000, 0x3AFFFFFF), (0x3B000000, 0x3BFFFFFF),
    (0x3C000000, 0x3CFFFFFF), (0x3D000000, 0x3DFFFFFF),
    (0x65000000, 0x65FFFFFF), (0x67000000, 0x67FFFFFF),
    (0x6A000000, 0x6AFFFFFF), (0x6E000000, 0x6EFFFFFF),
    (0x6F000000, 0x6FFFFFFF), (0x70000000, 0x70FFFFFF),
    (0x71000000, 0x71FFFFFF), (0x72000000, 0x72FFFFFF),
    (0x73000000, 0x73FFFFFF), (0x74000000, 0x74FFFFFF),
    (0x75000000, 0x75FFFFFF), (0x76000000, 0x76FFFFFF),
    (0x77000000, 0x77FFFFFF), (0x78000000, 0x78FFFFFF),
    (0x79000000, 0x79FFFFFF), (0x7A000000, 0x7AFFFFFF),
    (0x7B000000, 0x7BFFFFFF), (0x7C000000, 0x7CFFFFFF),
    (0x7D000000, 0x7DFFFFFF), (0xAF000000, 0xAFFFFFFF),
    (0xB4000000, 0xB4FFFFFF), (0xB6000000, 0xB6FFFFFF),
    (0xB7000000, 0xB7FFFFFF), (0xCA000000, 0xCAFFFFFF),
    (0xD2000000, 0xD2FFFFFF), (0xD3000000, 0xD3FFFFFF),
    (0xDA000000, 0xDAFFFFFF), (0xDB000000, 0xDBFFFFFF),
    (0xDC000000, 0xDCFFFFFF), (0xDD000000, 0xDDFFFFFF),
    (0xDE000000, 0xDEFFFFFF),
];

fn fallback_is_china_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let ip = u32::from(*v4);
            let idx = CHINA_V4_FALLBACK.partition_point(|&(s, _)| s <= ip);
            if idx == 0 {
                return false;
            }
            let (_, e) = CHINA_V4_FALLBACK[idx - 1];
            ip <= e
        }
        _ => false,
    }
}

// ==================== 测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_china_ip() {
        assert!(is_china_ip(&"114.114.114.114".parse().expect("parse 114.114.114.114")));
        assert!(is_china_ip(&"180.76.76.76".parse().expect("parse 180.76.76.76")));
        assert!(!is_china_ip(&"8.8.8.8".parse().expect("parse 8.8.8.8")));
        assert!(!is_china_ip(&"1.1.1.1".parse().expect("parse 1.1.1.1")));
        assert!(!is_china_ip(&"140.82.121.3".parse().expect("parse 140.82.121.3")));
    }

    #[test]
    fn test_parse_cidr() {
        let (start, end) = cidr_to_range("192.168.0.0/16").expect("parse 192.168.0.0/16");
        assert_eq!(start, 0xC0A80000);
        assert_eq!(end, 0xC0A8FFFF);
    }

    #[test]
    fn test_parse_cidr_list() {
        let ranges = parse_cidr_list("1.0.0.0/24\n# comment\n8.8.8.0/24\n");
        assert_eq!(ranges.len(), 2);
    }

    #[test]
    fn test_geo_ip_load_from_text() {
        let mut db = GeoDatabase::new();
        let n = db.load_cidr_text("114.114.114.0/24\n8.8.8.0/24\n", "test");
        assert_eq!(n, 2);
        assert!(db.is_china_ip(&"114.114.114.114".parse().expect("parse 114.114.114.114")));
        assert!(db.is_china_ip(&"8.8.8.8".parse().expect("parse 8.8.8.8")));
        assert!(!db.is_china_ip(&"9.9.9.9".parse().expect("parse 9.9.9.9")));
    }

    #[test]
    fn test_domain_rules_direct() {
        let mut rules = DomainRules::new();
        rules.direct_domains.insert("baidu.com".into());
        rules.proxy_domains.insert("google.com".into());
        assert_eq!(rules.should_direct("baidu.com"), Some(true));
        assert_eq!(rules.should_direct("google.com"), Some(false));
        assert_eq!(rules.should_direct("unknown.com"), None);
    }

    #[test]
    fn test_domain_rules_suffix() {
        let mut rules = DomainRules::new();
        rules.direct_domains.insert("baidu.com".into());
        assert_eq!(rules.should_direct("www.baidu.com"), Some(true));
        assert_eq!(rules.should_direct("api.baidu.com"), Some(true));
    }

    #[test]
    fn test_timeout_error() {
        assert!(is_timeout_error("request timed out"));
        assert!(!is_timeout_error("404 Not Found"));
    }

    #[test]
    fn test_fallback_ip() {
        assert!(fallback_is_china_ip(&"14.0.0.1".parse().expect("parse 14.0.0.1")));
        assert!(!fallback_is_china_ip(&"8.8.8.8".parse().expect("parse 8.8.8.8")));
    }

    #[cfg(feature = "network_tests")]
    #[tokio::test]
    async fn test_fetch_loyalsoldier_china_list() {
        let updater = RuleUpdater::new(RuleUpdater::default_cache());
        let report = updater.update_all().await;
        // 只要能拉到数据就算通过（网络不可用时不panic）
        if report.china_ip_count > 0 {
            assert!(report.china_ip_count > 5000, "expected >= 5000 CIDR");
            let geo_arc = global_geo();
            let geo = geo_arc.read().await;
            assert!(geo.is_china_ip(&"114.114.114.114".parse().expect("parse 114.114.114.114")));
        }
    }

    #[cfg(feature = "network_tests")]
    #[tokio::test]
    async fn test_fetch_direct_domains() {
        let updater = RuleUpdater::new(RuleUpdater::default_cache());
        let report = updater.update_all().await;
        if report.direct_domain_count > 0 {
            assert!(report.direct_domain_count > 10000, "expected >= 10k domains");
            let dom_arc = global_domains();
            let domains = dom_arc.read().await;
            assert_eq!(domains.should_direct("baidu.com"), Some(true));
        }
    }
}
