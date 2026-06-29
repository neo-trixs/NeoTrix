use serde::{Deserialize, Serialize};
use serde_json;
use std::net::IpAddr;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RuleOrigin {
    #[default]
    Local,
    External(String),
    Temporary,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RuleCondition {
    DomainSuffix(String),
    DomainExact(String),
    UrlPathPrefix(String),
    Cidr(std::net::IpAddr, u8),
    SchemeMatch(String),
    Always,
}

impl RuleCondition {
    pub fn from_cidr(cidr: &str) -> Option<Self> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        let prefix_len: u8 = match parts[1].parse() {
            Ok(n) if n <= 128 => n,
            Ok(_) => return None,
            Err(e) => {
                log::warn!("[rules] parse prefix: {}", e);
                return None;
            }
        };
        let network_ip: std::net::IpAddr = match parts[0].parse() {
            Ok(ip) => ip,
            Err(e) => {
                log::warn!("[rules] parse IP: {}", e);
                return None;
            }
        };
        Some(RuleCondition::Cidr(network_ip, prefix_len))
    }
}

impl RuleCondition {
    pub fn matches(&self, url: &Url) -> bool {
        match self {
            RuleCondition::DomainSuffix(suffix) => url
                .host_str()
                .map(|h| h == suffix.trim_start_matches('.') || h.ends_with(suffix))
                .unwrap_or(false),
            RuleCondition::DomainExact(domain) => url.host_str() == Some(domain.as_str()),
            RuleCondition::UrlPathPrefix(prefix) => url.path().starts_with(prefix),
            RuleCondition::Cidr(ref net_ip, prefix_len) => url
                .host_str()
                .and_then(|h| {
                    h.parse::<IpAddr>()
                        .inspect_err(|e| log::warn!("[rules] parse host: {}", e))
                        .ok()
                })
                .map(|ip| ip_is_in_cidr_pre(&ip, *net_ip, *prefix_len))
                .unwrap_or(false),
            RuleCondition::SchemeMatch(scheme) => url.scheme() == scheme,
            RuleCondition::Always => true,
        }
    }

    pub fn matches_host(&self, host: &str) -> bool {
        match self {
            RuleCondition::DomainSuffix(suffix) => {
                host == suffix.trim_start_matches('.') || host.ends_with(suffix)
            }
            RuleCondition::DomainExact(domain) => host == domain,
            RuleCondition::Cidr(ref net_ip, prefix_len) => host
                .parse::<IpAddr>()
                .inspect_err(|e| log::warn!("[rules] parse host: {}", e))
                .ok()
                .map(|ip| ip_is_in_cidr_pre(&ip, *net_ip, *prefix_len))
                .unwrap_or(false),
            RuleCondition::Always => true,
            _ => false,
        }
    }

    pub fn matches_ip(&self, ip: IpAddr) -> bool {
        match self {
            RuleCondition::Cidr(ref net_ip, prefix_len) => {
                ip_is_in_cidr_pre(&ip, *net_ip, *prefix_len)
            }
            RuleCondition::Always => true,
            _ => false,
        }
    }
}

fn ip_is_in_cidr_pre(ip: &IpAddr, network_ip: IpAddr, prefix_len: u8) -> bool {
    match (ip, network_ip) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            let mask = if prefix_len >= 32 {
                0u32
            } else {
                !0u32 << (32 - prefix_len)
            };
            (u32::from(*ip) & mask) == (u32::from(net) & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(net)) => {
            let mask = if prefix_len >= 128 {
                0u128
            } else {
                !0u128 << (128 - prefix_len)
            };
            (u128::from(*ip) & mask) == (u128::from(net) & mask)
        }
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub enum OutboundAction {
    Direct,
    Proxy(String),
    Tor,
    Block,
}

#[derive(Debug, Clone)]
pub struct OutboundRule {
    pub condition: RuleCondition,
    pub action: OutboundAction,
    pub priority: u32,
    pub enabled: bool,
    pub label: String,
    pub origin: RuleOrigin,
    pub ttl_secs: Option<u64>,
    pub created_at: std::time::Instant,
}

impl OutboundRule {
    pub fn new(label: &str, condition: RuleCondition, action: OutboundAction) -> Self {
        Self {
            condition,
            action,
            priority: 100,
            enabled: true,
            label: label.to_string(),
            origin: RuleOrigin::Local,
            ttl_secs: None,
            created_at: std::time::Instant::now(),
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn external(
        label: &str,
        condition: RuleCondition,
        action: OutboundAction,
        source: &str,
    ) -> Self {
        Self {
            condition,
            action,
            priority: 50,
            enabled: true,
            label: label.to_string(),
            origin: RuleOrigin::External(source.to_string()),
            ttl_secs: None,
            created_at: std::time::Instant::now(),
        }
    }

    pub fn matches(&self, url: &Url) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(ttl) = self.ttl_secs {
            if self.created_at.elapsed().as_secs() > ttl {
                return false;
            }
        }
        self.condition.matches(url)
    }

    pub fn matches_host(&self, host: &str) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(ttl) = self.ttl_secs {
            if self.created_at.elapsed().as_secs() > ttl {
                return false;
            }
        }
        self.condition.matches_host(host)
    }

    pub fn is_expired(&self) -> bool {
        self.ttl_secs
            .map(|t| self.created_at.elapsed().as_secs() > t)
            .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize)]
struct RuleSnapshot {
    label: String,
    condition_type: String,
    condition_value: String,
    action_type: String,
    action_param: Option<String>,
    priority: u32,
    enabled: bool,
    origin: String,
    ttl_secs: Option<u64>,
}

impl RuleSnapshot {
    fn from_rule(r: &OutboundRule) -> Self {
        let (condition_type, condition_value) = match &r.condition {
            RuleCondition::DomainSuffix(v) => ("domain_suffix".into(), v.clone()),
            RuleCondition::DomainExact(v) => ("domain_exact".into(), v.clone()),
            RuleCondition::UrlPathPrefix(v) => ("url_path_prefix".into(), v.clone()),
            RuleCondition::Cidr(ip, prefix) => ("cidr".into(), format!("{}/{}", ip, prefix)),
            RuleCondition::SchemeMatch(v) => ("scheme".into(), v.clone()),
            RuleCondition::Always => ("always".into(), String::new()),
        };
        let (action_type, action_param) = match &r.action {
            OutboundAction::Direct => ("direct".into(), None),
            OutboundAction::Block => ("block".into(), None),
            OutboundAction::Tor => ("tor".into(), None),
            OutboundAction::Proxy(u) => ("proxy".into(), Some(u.clone())),
        };
        let origin = match &r.origin {
            RuleOrigin::Local => "local".into(),
            RuleOrigin::External(s) => format!("external:{}", s),
            RuleOrigin::Temporary => "temporary".into(),
        };
        Self {
            label: r.label.clone(),
            condition_type,
            condition_value,
            action_type,
            action_param,
            priority: r.priority,
            enabled: r.enabled,
            origin,
            ttl_secs: r.ttl_secs,
        }
    }

    fn to_rule(&self) -> Option<OutboundRule> {
        let condition = match self.condition_type.as_str() {
            "domain_suffix" => RuleCondition::DomainSuffix(self.condition_value.clone()),
            "domain_exact" => RuleCondition::DomainExact(self.condition_value.clone()),
            "url_path_prefix" => RuleCondition::UrlPathPrefix(self.condition_value.clone()),
            "cidr" => RuleCondition::from_cidr(&self.condition_value)?,
            "scheme" => RuleCondition::SchemeMatch(self.condition_value.clone()),
            "always" => RuleCondition::Always,
            _ => return None,
        };
        let action = match self.action_type.as_str() {
            "direct" => OutboundAction::Direct,
            "block" => OutboundAction::Block,
            "tor" => OutboundAction::Tor,
            "proxy" => OutboundAction::Proxy(self.action_param.clone().unwrap_or_default()),
            _ => return None,
        };
        let origin = if self.origin == "local" {
            RuleOrigin::Local
        } else if let Some(s) = self.origin.strip_prefix("external:") {
            RuleOrigin::External(s.to_string())
        } else {
            RuleOrigin::Temporary
        };
        let mut rule = OutboundRule::new(&self.label, condition, action);
        rule.priority = self.priority;
        rule.enabled = self.enabled;
        rule.origin = origin;
        rule.ttl_secs = self.ttl_secs;
        Some(rule)
    }
}

pub struct RuleEngine {
    rules: Vec<OutboundRule>,
    default_action: OutboundAction,
    persistence_path: Option<PathBuf>,
}

impl RuleEngine {
    /// 创建引擎并加载内置 china_bypass 规则 + 用户持久化规则
    pub fn new() -> Self {
        let mut engine = Self {
            rules: china_bypass_rules(),
            default_action: OutboundAction::Direct,
            persistence_path: Some(Self::default_path()),
        };
        engine.rules.sort_by_key(|r| r.priority);
        // 合并来自 ~/.neotrix/rules.json 的用户自定义规则
        let _ = engine.load();
        engine
    }

    /// 创建空引擎（无内置规则、无持久化），主要用于测试
    pub fn new_empty() -> Self {
        Self {
            rules: Vec::new(),
            default_action: OutboundAction::Direct,
            persistence_path: None,
        }
    }

    pub fn with_persistence(mut self, path: PathBuf) -> Self {
        self.persistence_path = Some(path);
        self
    }

    pub fn no_persistence(mut self) -> Self {
        self.persistence_path = None;
        self
    }

    fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".neotrix").join("rules.json")
    }

    pub fn save(&self) -> Result<(), String> {
        let path = match self.persistence_path {
            Some(ref p) => p,
            None => return Ok(()),
        };
        let defaults = china_bypass_rules();
        let default_labels: std::collections::HashSet<&str> =
            defaults.iter().map(|r| r.label.as_str()).collect();
        let snapshots: Vec<RuleSnapshot> = self
            .rules
            .iter()
            .filter(|r| !default_labels.contains(r.label.as_str()))
            .map(RuleSnapshot::from_rule)
            .collect();
        let json =
            serde_json::to_string_pretty(&snapshots).map_err(|e| format!("serialize: {}", e))?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| format!("write rules: {}", e))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("rename rules: {}", e))?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        let path = match self.persistence_path {
            Some(ref p) => p,
            None => return Ok(()),
        };
        if !path.exists() {
            return Ok(());
        }
        let json = std::fs::read_to_string(path).map_err(|e| format!("read rules: {}", e))?;
        let snapshots: Vec<RuleSnapshot> =
            serde_json::from_str(&json).map_err(|e| format!("deserialize: {}", e))?;
        let persisted: Vec<OutboundRule> = snapshots.iter().filter_map(|s| s.to_rule()).collect();
        self.merge_persisted(persisted)
    }

    /// 从磁盘重新加载规则 — 清除已有用户规则后重新读取 rules.json
    pub fn reload_from_disk(&mut self) -> Result<String, String> {
        let defaults = china_bypass_rules();
        let default_labels: std::collections::HashSet<&str> =
            defaults.iter().map(|r| r.label.as_str()).collect();
        // 清除所有用户规则，仅保留内置默认规则
        self.rules
            .retain(|r| default_labels.contains(r.label.as_str()));
        let count_before = self.rules.len();
        self.load()?;
        let total = self.rules.len();
        let user_rules = total - count_before;
        Ok(format!(
            "{} defaults + {} user = {} rules",
            count_before, user_rules, total
        ))
    }

    fn merge_persisted(&mut self, persisted: Vec<OutboundRule>) -> Result<(), String> {
        let defaults = china_bypass_rules();
        let default_labels: std::collections::HashSet<&str> =
            defaults.iter().map(|r| r.label.as_str()).collect();
        let user_rules: Vec<OutboundRule> = persisted
            .into_iter()
            .filter(|r| !default_labels.contains(r.label.as_str()))
            .collect();
        if !user_rules.is_empty() {
            log::info!("[rules] merging {} user rules", user_rules.len());
            for p in user_rules {
                if let Some(pos) = self.rules.iter().position(|r| r.label == p.label) {
                    self.rules[pos] = p;
                } else {
                    self.rules.push(p);
                }
            }
            self.rules.sort_by_key(|r| r.priority);
        }
        Ok(())
    }

    pub fn with_default(mut self, action: OutboundAction) -> Self {
        self.default_action = action;
        self
    }

    pub fn add_rule(&mut self, rule: OutboundRule) {
        self.rules.push(rule);
        self.rules.sort_by_key(|r| r.priority);
        let _ = self.save();
    }

    pub fn remove_rule(&mut self, label: &str) {
        self.rules.retain(|r| r.label != label);
        let _ = self.save();
    }

    pub fn remove_expired(&mut self) {
        self.rules.retain(|r| !r.is_expired());
    }

    pub fn clear_all(&mut self) {
        let defaults = china_bypass_rules();
        let default_labels: std::collections::HashSet<&str> =
            defaults.iter().map(|r| r.label.as_str()).collect();
        self.rules
            .retain(|r| default_labels.contains(r.label.as_str()));
    }

    pub fn clear_external(&mut self) {
        self.rules
            .retain(|r| !matches!(r.origin, RuleOrigin::External(_)));
    }

    pub fn load_rules(&mut self, rules: Vec<OutboundRule>) {
        self.rules = rules;
        self.rules.sort_by_key(|r| r.priority);
    }

    pub fn evaluate(&self, url: &Url) -> &OutboundAction {
        for rule in &self.rules {
            if rule.matches(url) {
                return &rule.action;
            }
        }
        &self.default_action
    }

    pub fn evaluate_host(&self, host: &str) -> Option<&OutboundAction> {
        for rule in &self.rules {
            if rule.matches_host(host) {
                return Some(&rule.action);
            }
        }
        None
    }

    pub fn evaluate_ip(&self, ip: IpAddr) -> &OutboundAction {
        for rule in &self.rules {
            if rule.enabled && rule.condition.matches_ip(ip) {
                return &rule.action;
            }
        }
        &self.default_action
    }

    pub fn rules(&self) -> &[OutboundRule] {
        &self.rules
    }

    pub fn export(&self) -> Vec<OutboundRule> {
        self.rules.clone()
    }

    pub fn default_action(&self) -> &OutboundAction {
        &self.default_action
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn external_rule_count(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| matches!(r.origin, RuleOrigin::External(_)))
            .count()
    }

    pub fn has_rule(&self, label: &str) -> bool {
        self.rules.iter().any(|r| r.label == label)
    }

    pub fn get_rule_by_label(&self, label: &str) -> Option<&OutboundRule> {
        self.rules.iter().find(|r| r.label == label)
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn china_bypass_rules() -> Vec<OutboundRule> {
    vec![
        // ===== 中国主流网站（检测海外IP即屏蔽） =====
        OutboundRule::new(
            "direct-linuxdo",
            RuleCondition::DomainExact("linux.do".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-v2ex",
            RuleCondition::DomainSuffix(".v2ex.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-zhihu",
            RuleCondition::DomainSuffix(".zhihu.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-bilibili",
            RuleCondition::DomainSuffix(".bilibili.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-weibo",
            RuleCondition::DomainSuffix(".weibo.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-douban",
            RuleCondition::DomainSuffix(".douban.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-12306",
            RuleCondition::DomainSuffix(".12306.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-cnblogs",
            RuleCondition::DomainSuffix(".cnblogs.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-csdn",
            RuleCondition::DomainSuffix(".csdn.net".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-juejin",
            RuleCondition::DomainSuffix(".juejin.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-nowcoder",
            RuleCondition::DomainSuffix(".nowcoder.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-51job",
            RuleCondition::DomainSuffix(".51job.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-zhipin",
            RuleCondition::DomainSuffix(".zhipin.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        // ===== 中国域名后缀 =====
        OutboundRule::new(
            "direct-cn",
            RuleCondition::DomainSuffix(".cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-com-cn",
            RuleCondition::DomainSuffix(".com.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-net-cn",
            RuleCondition::DomainSuffix(".net.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-org-cn",
            RuleCondition::DomainSuffix(".org.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-gov-cn",
            RuleCondition::DomainSuffix(".gov.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-edu-cn",
            RuleCondition::DomainSuffix(".edu.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        // ===== 中国大厂域名 =====
        OutboundRule::new(
            "direct-baidu",
            RuleCondition::DomainSuffix(".baidu.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-alibaba",
            RuleCondition::DomainSuffix(".alibaba.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-taobao",
            RuleCondition::DomainSuffix(".taobao.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-tmall",
            RuleCondition::DomainSuffix(".tmall.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-jd",
            RuleCondition::DomainSuffix(".jd.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-qq",
            RuleCondition::DomainSuffix(".qq.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-tencent",
            RuleCondition::DomainSuffix(".tencent.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-weixin",
            RuleCondition::DomainSuffix(".weixin.qq.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-netease",
            RuleCondition::DomainSuffix(".163.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-sina",
            RuleCondition::DomainSuffix(".sina.com.cn".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-xiaomi",
            RuleCondition::DomainSuffix(".xiaomi.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-huawei",
            RuleCondition::DomainSuffix(".huawei.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-bytedance",
            RuleCondition::DomainSuffix(".bytedance.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-douyin",
            RuleCondition::DomainSuffix(".douyin.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-meituan",
            RuleCondition::DomainSuffix(".meituan.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-didi",
            RuleCondition::DomainSuffix(".didiglobal.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        OutboundRule::new(
            "direct-pinduoduo",
            RuleCondition::DomainSuffix(".pinduoduo.com".into()),
            OutboundAction::Direct,
        )
        .with_priority(10),
        // ===== 中国IP段 =====
        OutboundRule::new(
            "direct-cn-ip",
            RuleCondition::Cidr(std::net::IpAddr::V4(std::net::Ipv4Addr::new(1, 0, 0, 0)), 8),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip2",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(14, 0, 0, 0)),
                8,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip3",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(36, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip4",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(39, 0, 0, 0)),
                8,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip5",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(42, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip6",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(58, 0, 0, 0)),
                6,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip7",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(101, 0, 0, 0)),
                8,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip8",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(103, 0, 0, 0)),
                8,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip9",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(106, 0, 0, 0)),
                8,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip10",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(110, 0, 0, 0)),
                6,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip11",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(114, 0, 0, 0)),
                8,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip12",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(116, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip13",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(118, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip14",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(120, 0, 0, 0)),
                6,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip15",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(124, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip16",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(202, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip17",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(210, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip18",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(218, 0, 0, 0)),
                6,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        OutboundRule::new(
            "direct-cn-ip19",
            RuleCondition::Cidr(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(222, 0, 0, 0)),
                7,
            ),
            OutboundAction::Direct,
        )
        .with_priority(9),
        // ===== Tor 仅用于 .onion =====
        OutboundRule::new(
            "tor-onion",
            RuleCondition::DomainSuffix(".onion".into()),
            OutboundAction::Tor,
        )
        .with_priority(5),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_suffix_match() {
        let cond = RuleCondition::DomainSuffix(".google.com".to_string());
        let url = Url::parse("https://api.google.com/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
        let url = Url::parse("https://google.com/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
        let url = Url::parse("https://googlex.com/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(!cond.matches(&url));
    }

    #[test]
    fn test_domain_exact_match() {
        let cond = RuleCondition::DomainExact("example.com".to_string());
        let url = Url::parse("https://example.com/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
        let url = Url::parse("https://sub.example.com/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(!cond.matches(&url));
    }

    #[test]
    fn test_url_path_prefix() {
        let cond = RuleCondition::UrlPathPrefix("/api/".to_string());
        let url = Url::parse("https://example.com/api/v1/data")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
        let url = Url::parse("https://example.com/other")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(!cond.matches(&url));
    }

    #[test]
    fn test_scheme_match() {
        let cond = RuleCondition::SchemeMatch("http".to_string());
        assert!(cond.matches(
            &Url::parse("http://example.com")
                .expect("Url::parse of hardcoded test URL should never fail")
        ));
        assert!(!cond.matches(
            &Url::parse("https://example.com")
                .expect("Url::parse of hardcoded test URL should never fail")
        ));
    }

    #[test]
    fn test_cidr_match_v4() {
        let cond = RuleCondition::from_cidr("10.0.0.0/8")
            .expect("from_cidr with valid /8 CIDR should succeed");
        let url = Url::parse("http://10.1.2.3/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
        let url = Url::parse("http://192.168.1.1/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(!cond.matches(&url));
    }

    #[test]
    fn test_always_match() {
        let cond = RuleCondition::Always;
        let url = Url::parse("https://anything.com/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(cond.matches(&url));
    }

    #[test]
    fn test_rule_sorting_by_priority() {
        let mut engine = RuleEngine::new_empty();
        engine.add_rule(
            OutboundRule::new("low", RuleCondition::Always, OutboundAction::Direct)
                .with_priority(200),
        );
        engine.add_rule(
            OutboundRule::new("high", RuleCondition::Always, OutboundAction::Tor).with_priority(10),
        );
        let rules = engine.rules();
        assert_eq!(rules[0].label, "high");
        assert_eq!(rules[1].label, "low");
    }

    #[test]
    fn test_engine_evaluate_first_match() {
        let mut engine = RuleEngine::new_empty();
        engine.add_rule(
            OutboundRule::new(
                "direct-cn",
                RuleCondition::DomainSuffix(".cn".into()),
                OutboundAction::Direct,
            )
            .with_priority(10),
        );
        engine.add_rule(
            OutboundRule::new("tor-else", RuleCondition::Always, OutboundAction::Tor)
                .with_priority(200),
        );

        let cn_url = Url::parse("https://baidu.cn/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(matches!(engine.evaluate(&cn_url), OutboundAction::Direct));
        let other_url = Url::parse("https://google.com/path")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(matches!(engine.evaluate(&other_url), OutboundAction::Tor));
    }

    #[test]
    fn test_default_bypass_rules() {
        let rules = china_bypass_rules();
        assert!(rules.len() >= 3);
        assert!(rules.iter().any(|r| r.label == "tor-onion"));
        assert!(rules.iter().any(|r| r.label.starts_with("direct-")));
    }

    #[test]
    fn test_rule_clear_all() {
        let mut engine = RuleEngine::new_empty();
        engine.add_rule(OutboundRule::new(
            "test",
            RuleCondition::Always,
            OutboundAction::Block,
        ));
        assert_eq!(engine.rules().len(), 1);
        engine.clear_all();
        assert_eq!(engine.rules().len(), 0);
    }

    #[test]
    fn test_remove_rule_by_label() {
        let mut engine = RuleEngine::new_empty();
        engine.add_rule(OutboundRule::new(
            "keep",
            RuleCondition::Always,
            OutboundAction::Direct,
        ));
        engine.add_rule(OutboundRule::new(
            "remove-me",
            RuleCondition::Always,
            OutboundAction::Block,
        ));
        engine.remove_rule("remove-me");
        assert_eq!(engine.rules().len(), 1);
        assert_eq!(engine.rules()[0].label, "keep");
    }

    #[test]
    fn test_load_rules_replaces_existing() {
        let mut engine = RuleEngine::new_empty();
        engine.add_rule(OutboundRule::new(
            "old",
            RuleCondition::Always,
            OutboundAction::Block,
        ));
        engine.load_rules(vec![OutboundRule::new(
            "new",
            RuleCondition::Always,
            OutboundAction::Tor,
        )]);
        assert_eq!(engine.rules().len(), 1);
        assert_eq!(engine.rules()[0].label, "new");
    }

    #[test]
    fn test_ip_in_cidr_v4() {
        assert!(ip_is_in_cidr_pre(
            &"10.0.0.1"
                .parse()
                .expect("hardcoded IP string 10.0.0.1 should parse as IpAddr"),
            "10.0.0.0"
                .parse()
                .expect("hardcoded IP string 10.0.0.0 should parse as IpAddr"),
            8
        ));
        assert!(!ip_is_in_cidr_pre(
            &"11.0.0.1"
                .parse()
                .expect("hardcoded IP string 11.0.0.1 should parse as IpAddr"),
            "10.0.0.0"
                .parse()
                .expect("hardcoded IP string 10.0.0.0 should parse as IpAddr"),
            8
        ));
        assert!(ip_is_in_cidr_pre(
            &"192.168.1.100"
                .parse()
                .expect("hardcoded IP string 192.168.1.100 should parse as IpAddr"),
            "192.168.0.0"
                .parse()
                .expect("hardcoded IP string 192.168.0.0 should parse as IpAddr"),
            16
        ));
        assert!(!ip_is_in_cidr_pre(
            &"192.167.255.255"
                .parse()
                .expect("hardcoded IP string 192.167.255.255 should parse as IpAddr"),
            "192.168.0.0"
                .parse()
                .expect("hardcoded IP string 192.168.0.0 should parse as IpAddr"),
            16
        ));
    }

    #[test]
    fn test_disabled_rule_does_not_match() {
        let mut rule = OutboundRule::new("disabled", RuleCondition::Always, OutboundAction::Block);
        rule.enabled = false;
        let url = Url::parse("https://example.com")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(!rule.matches(&url));
    }

    #[test]
    fn test_external_rule_origin() {
        let rule = OutboundRule::external(
            "ext-test",
            RuleCondition::Always,
            OutboundAction::Block,
            "test-app",
        );
        assert!(matches!(rule.origin, RuleOrigin::External(_)));
    }

    #[test]
    fn test_expired_rule_does_not_match() {
        let mut rule = OutboundRule::new("expired", RuleCondition::Always, OutboundAction::Block);
        // 设置 TTL 为 0 秒并使用 Instant::now() 已过去至少 1 秒
        rule.created_at = std::time::Instant::now() - std::time::Duration::from_secs(1);
        rule.ttl_secs = Some(0);
        let url = Url::parse("https://example.com")
            .expect("Url::parse of hardcoded test URL should never fail");
        assert!(!rule.matches(&url));
        assert!(rule.is_expired());
    }

    #[test]
    fn test_clear_external_preserves_local() {
        let mut engine = RuleEngine::new_empty();
        engine.add_rule(OutboundRule::new(
            "local",
            RuleCondition::Always,
            OutboundAction::Direct,
        ));
        engine.add_rule(OutboundRule::external(
            "ext",
            RuleCondition::Always,
            OutboundAction::Block,
            "app",
        ));
        assert_eq!(engine.rule_count(), 2);
        engine.clear_external();
        assert_eq!(engine.rule_count(), 1);
        assert_eq!(engine.rules()[0].label, "local");
    }

    #[test]
    fn test_evaluate_host() {
        let mut engine = RuleEngine::new_empty();
        engine.add_rule(OutboundRule::new(
            "block-google",
            RuleCondition::DomainExact("google.com".into()),
            OutboundAction::Block,
        ));
        assert!(matches!(
            engine.evaluate_host("google.com"),
            Some(OutboundAction::Block)
        ));
        assert!(engine.evaluate_host("example.com").is_none());
    }

    #[test]
    fn test_rule_counters() {
        let mut engine = RuleEngine::new_empty();
        assert_eq!(engine.rule_count(), 0);
        engine.add_rule(OutboundRule::new(
            "a",
            RuleCondition::Always,
            OutboundAction::Direct,
        ));
        assert_eq!(engine.rule_count(), 1);
        assert_eq!(engine.external_rule_count(), 0);
    }
}
