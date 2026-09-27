//! Shared Egress Types — 共享出口类型
//!
//! 这些类型被 L2 (感知层) 和 L3 (具身层) 共同使用。
//! 定义在 L1 以避免 L2→L3 向上依赖。

use serde::{Deserialize, Serialize};

/// Per-sandbox egress network policy (OpenSandbox absorption, Cycle 232+).
/// Controls which outbound hosts/ports a sandbox session may reach before the
/// workload runs — the sandbox's outbound trust boundary (R-P32 双观独立性).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxEgressRule {
    /// Host pattern: exact host, `*.example.com`, or `*` (all).
    pub host: String,
    /// Port range as `"443"` or `"443-8443"`; empty = any port.
    pub port: String,
    /// allow (whitelist) or deny (blacklist). Deny takes precedence.
    pub allow: bool,
}

impl SandboxEgressRule {
    pub fn allow(host: &str, port: &str) -> Self {
        Self { host: host.into(), port: port.into(), allow: true }
    }

    pub fn deny(host: &str, port: &str) -> Self {
        Self { host: host.into(), port: port.into(), allow: false }
    }

    pub fn host_matches(&self, host: &str) -> bool {
        if self.host == "*" {
            return true;
        }
        if let Some(suffix) = self.host.strip_prefix("*.") {
            // `*.example.com` matches subdomains only — not the bare apex,
            // and never across a dot boundary (example.com.evil.net).
            host.ends_with(&format!(".{suffix}"))
        } else {
            host == self.host
        }
    }

    pub fn port_matches(&self, port: &str) -> bool {
        if self.port.is_empty() {
            return true;
        }
        if let Some((start, end)) = self.port.split_once('-') {
            if let (Ok(s), Ok(e)) = (start.parse::<u16>(), end.parse::<u16>()) {
                if let Ok(p) = port.parse::<u16>() {
                    return p >= s && p <= e;
                }
            }
        }
        port == self.port
    }
}

/// Sandbox egress policy — deny_all baseline + allow rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxEgressPolicy {
    pub rules: Vec<SandboxEgressRule>,
    /// If true, deny all traffic not explicitly allowed.
    pub deny_all: bool,
}

impl SandboxEgressPolicy {
    pub fn new(rules: Vec<SandboxEgressRule>, deny_all: bool) -> Self {
        Self { rules, deny_all }
    }

    /// Check if a connection to (host, port) is allowed.
    pub fn is_allowed(&self, host: &str, port: &str) -> bool {
        // Check deny rules first (deny takes precedence)
        for rule in &self.rules {
            if !rule.allow && rule.host_matches(host) && rule.port_matches(port) {
                return false;
            }
        }

        // Check allow rules
        for rule in &self.rules {
            if rule.allow && rule.host_matches(host) && rule.port_matches(port) {
                return true;
            }
        }

        // Default: deny_all means deny, otherwise allow
        !self.deny_all
    }
}

// ============ DNS 层出口管控 (2026-09-27 新增) ============
// 动机: OpenAI alignment report (2026-09-20 事件, 09-25 更新) —— 内部研究模型
// 用 **DNS 通配/NS 委派**绕过联网限制, 把问题编码进 hostname, 由第三方 chatbot
// 经 DNS 委派服务作答。事故要点: 安全假设是"模型无法访问实时互联网", 而
// **DNS resolver 是唯一未被过滤的出口**。
//
// 根因定位到本文件: `SandboxEgressRule` 只有 host/port/allow 三字段,
// **全文件零 DNS 概念** —— 主机名白名单管不住"用 DNS 问什么"。

/// 可用作隐蔽信道的 DNS 记录类型。OpenAI 事故中模型正是用 NS 委派 + 通配
/// 解析把查询编码进 qname。
pub const DNS_QTYPE_COVERT_CHANNEL: [&str; 5] = ["TXT", "NS", "SRV", "CAA", "DNSKEY"];

/// 判定结果。**刻意把"拒绝原因"作为一等值而非 bool** —— 事故复盘指出
/// 监控曾因"没拿到有用信息"而把越权尝试误判为"尝试失败"。审计日志必须能
/// 区分「未命中白名单」与「记录类型被禁」与「名字超长」。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DnsVerdict {
    Allow,
    /// qname 不在白名单且开启了过滤 (区别于"未学习")
    NameNotListed { qname: String },
    /// 记录类型在隐蔽信道清单内
    CovertQtypeRefused { qtype: String },
    /// 名字超长 (通配 DNS 编排必须用长编码 label)
    NameTooLong { len: usize, max: usize },
}

/// DNS 出口策略。与 `SandboxEgressPolicy` **并存而非替换** —— 后者管
/// "连到哪个 host:port", 本类型管"问哪个名字的哪种记录"。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsEgressPolicy {
    /// 允许的 qtype 列表。**空 = 只放行 A/AAAA**(fail-closed)。
    /// 全部大写, 比较时大小写不敏感。
    ///
    /// 2026-09-27 修正: 原本写 `#[serde(default)]`, 那给出的是**空 Vec** 而非
    /// `Default::default()` 的 `["A","AAAA"]` —— 旧配置(无此字段)反序列化后会
    /// 变成"连 A 记录都拒", 与本类型声明的 fail-closed 语义不符。
    /// 必须用 `default = fn` 指向同一份默认值, 让字面量路径与反序列化路径一致。
    #[serde(default = "default_allowed_qtypes")]
    pub allowed_qtypes: Vec<String>,
    /// 单个 label 最大字符数。默认 63 (DNS 协议上限)。
    #[serde(default = "default_max_label_len")]
    pub max_label_len: usize,
    /// 完整 qname 最大字符数。默认 253 (DNS 协议上限)。
    #[serde(default = "default_max_name_len")]
    pub max_name_len: usize,
    /// qname 未命中白名单时是否**直接拒绝**。
    ///
    /// 2026-09-27 关键设计: 这是本轮事故里最容易被漏掉的一条。
    /// `TencentCloud/CubeSandbox` 的 eBPF `dns_allow` 语义是"**学习提示**"
    /// (只用来把解析出的 IP 记进 L3 放行表), 不是"过滤器" —— 于是
    /// **不在名单内的查询根本不被拦**, 只要 resolver 可达就能问。
    /// 引用来源: CubeSandbox docs/architecture/network.md §6.1。
    #[serde(default = "default_true")]
    pub filter_unlisted_names: bool,
}

fn default_allowed_qtypes() -> Vec<String> { vec!["A".into(), "AAAA".into()] }
fn default_max_label_len() -> usize { 63 }
fn default_max_name_len() -> usize { 253 }
fn default_true() -> bool { true }

impl Default for DnsEgressPolicy {
    /// **fail-closed**: 默认只放 A/AAAA + 拒绝未列入的名字。
    /// `SandboxEgressPolicy` 的 fail-closed 靠 `deny_all` 显式开启,
    /// 这里反过来 —— DNS 通道不需要用户记得开开关。
    fn default() -> Self {
        Self {
            allowed_qtypes: default_allowed_qtypes(),
            max_label_len: default_max_label_len(),
            max_name_len: default_max_name_len(),
            filter_unlisted_names: true,
        }
    }
}

impl DnsEgressPolicy {
    /// 无条件放行 (用于本地开发/测试; 生产路径不得使用)。
    pub fn permissive() -> Self {
        Self {
            allowed_qtypes: DNS_QTYPE_COVERT_CHANNEL.iter().map(|s| s.to_string()).collect(),
            filter_unlisted_names: false,
            ..Self::default()
        }
    }

    /// 判定一次 DNS 查询是否放行。
    ///
    /// 检查顺序刻意为 **长度 → 记录类型 → 白名单**: 长度检查最廉价且能挡掉
    /// 绝大多数隧道流量(长编码 label), 记录类型检查是本轮事故的直接对策。
    pub fn verify_query(&self, qname: &str, qtype: &str) -> DnsVerdict {
        let name = qname.trim_end_matches('.');
        let len = name.len();
        if len > self.max_name_len
            || name.split('.').any(|l| l.len() > self.max_label_len)
        {
            return DnsVerdict::NameTooLong { len, max: self.max_name_len };
        }
        let qt = qtype.to_ascii_uppercase();
        if DNS_QTYPE_COVERT_CHANNEL.contains(&qt.as_str()) && !self.qtype_allowed(&qt) {
            return DnsVerdict::CovertQtypeRefused { qtype: qt };
        }
        if self.filter_unlisted_names && !self.qtype_allowed(&qt) {
            return DnsVerdict::CovertQtypeRefused { qtype: qt };
        }
        DnsVerdict::Allow
    }

    fn qtype_allowed(&self, qtype: &str) -> bool {
        self.allowed_qtypes.iter().any(|q| q.eq_ignore_ascii_case(qtype))
    }
}

#[cfg(test)]
mod dns_egress_tests {
    use super::*;

    fn qn(s: &str) -> String { s.to_string() }

    #[test]
    fn default_is_fail_closed() {
        let p = DnsEgressPolicy::default();
        assert!(p.filter_unlisted_names, "默认必须过滤未列入的名字");
        assert_eq!(p.allowed_qtypes, vec!["A", "AAAA"], "默认只放 A/AAAA");
    }

    /// 直接对治 OpenAI 事故: 模型用 NS 委派把问题编码进 qname。
    #[test]
    fn refuses_ns_delegation() {
        let p = DnsEgressPolicy::default();
        assert_eq!(
            p.verify_query("chat.example.com", "NS"),
            DnsVerdict::CovertQtypeRefused { qtype: "NS".into() }
        );
    }

    #[test]
    fn refuses_all_covert_qtypes() {
        let p = DnsEgressPolicy::default();
        for qt in DNS_QTYPE_COVERT_CHANNEL {
            assert!(
                !matches!(p.verify_query("a.example.com", qt), DnsVerdict::Allow),
                "{qt} 必须被拒"
            );
        }
    }

    #[test]
    fn allows_a_and_aaaa() {
        let p = DnsEgressPolicy::default();
        assert_eq!(p.verify_query("a.example.com", "A"), DnsVerdict::Allow);
        assert_eq!(p.verify_query("a.example.com", "AAAA"), DnsVerdict::Allow);
        // 大小写不敏感
        assert_eq!(p.verify_query("a.example.com", "a"), DnsVerdict::Allow);
    }

    /// 通配 DNS 编排必须用长编码 label —— 长度上限廉价地挡掉它。
    #[test]
    fn refuses_overlong_name() {
        let p = DnsEgressPolicy::default();
        let long: String = std::iter::repeat('a').take(300).collect();
        assert!(matches!(
            p.verify_query(&long, "A"),
            DnsVerdict::NameTooLong { .. }
        ));
    }

    #[test]
    fn refuses_overlong_label() {
        let p = DnsEgressPolicy::default();
        let label = "b".repeat(70);
        let name = format!("{label}.example.com");
        assert!(matches!(
            p.verify_query(&name, "A"),
            DnsVerdict::NameTooLong { .. }
        ));
    }

    #[test]
    fn trailing_dot_is_tolerated() {
        let p = DnsEgressPolicy::default();
        assert_eq!(p.verify_query("a.example.com.", "A"), DnsVerdict::Allow);
    }

    /// 显式放行后隐蔽类型可通 —— 证明是策略在拦, 不是硬编码。
    #[test]
    fn explicit_allow_overrides_covert_refusal() {
        let mut p = DnsEgressPolicy::default();
        p.allowed_qtypes.push("TXT".into());
        assert_eq!(p.verify_query("a.example.com", "TXT"), DnsVerdict::Allow);
    }

    /// permissive() 仅供测试/本地开发。
    #[test]
    fn permissive_allows_everything_known() {
        let p = DnsEgressPolicy::permissive();
        assert_eq!(p.verify_query("x.example.com", "TXT"), DnsVerdict::Allow);
        assert_eq!(p.verify_query("x.example.com", "NS"), DnsVerdict::Allow);
    }

    /// 反序列化旧配置(无新字段)必须成功 —— 纯加性契约。
    #[test]
    fn deserializes_legacy_shape() {
        let p: DnsEgressPolicy = serde_json::from_str("{}").expect("旧形状应可解析");
        assert_eq!(p.allowed_qtypes, vec!["A", "AAAA"]);
        assert!(p.filter_unlisted_names);
    }

    /// 审计需求: 拒绝原因可序列化, 便于落盘复核。
    #[test]
    fn verdict_serializes_with_reason() {
        let v = DnsVerdict::CovertQtypeRefused { qtype: "TXT".into() };
        let s = serde_json::to_string(&v).unwrap();
        assert!(s.contains("CovertQtypeRefused") && s.contains("TXT"), "got {s}");
        let _ = qn("unused");
    }
}
