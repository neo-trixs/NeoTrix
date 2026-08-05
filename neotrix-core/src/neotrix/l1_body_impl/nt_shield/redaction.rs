//! 隐私脱敏模块 (Redaction) — 复活自 archive_star OutputScreener (R-P42 强化现有节点)
//!
//! 功能:
//!   1. `redact()` — 扫描文本并替换 secrets/PII 为占位符 (可还原污染? 否，单向)。
//!      (注: 非对称地，脱敏不可逆向 — 与 `key_encryption` 的可逆加密互补。)
//!   2. `analyze()` — 风险分级 Safe / Suspicious / Dangerous。
//!   3. `is_safe()` — 快速布尔判定。
//!   4. 秘密规则覆盖 AWS/GitHub/Stripe/OpenAI/私钥/Slack/GitLab/通用 key/password/secret。
//!   5. PII 规则覆盖 email/phone/IPv4/IPv6/私钥块。
//!
//! 挂载点: EventBus 持久化 (`nt_core_event_bus::emit`) — 落盘前净化。
//! CLI: `/redact <text>`。

use regex::Regex;
use serde::Serialize;

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RiskLevel {
    Safe,
    Suspicious,
    Dangerous,
}

/// 脱敏器 (编译期构建正则集)
pub struct Redactor {
    secret_regexes: Vec<(&'static str, Regex)>,
    secret_strs: Vec<&'static str>,
    pii_regexes: Vec<(&'static str, Regex)>,
}

impl Default for Redactor {
    fn default() -> Self {
        Self {
            secret_regexes: vec![
                ("openai", Regex::new(r"sk-[a-zA-Z0-9_-]{20,}").expect("literal regex")),
                ("stripe", Regex::new(r"sk-[a-fA-F0-9]{32,}").expect("literal regex")),
                ("github-pat", Regex::new(r"ghp_[a-zA-Z0-9]{36}").expect("literal regex")),
                ("github-pat-fine", Regex::new(r"github_pat_[a-zA-Z0-9]{36}").expect("literal regex")),
                ("github-oauth", Regex::new(r"gho_[a-zA-Z0-9]{36}").expect("literal regex")),
                ("github-app", Regex::new(r"ghu_[a-zA-Z0-9]{36}").expect("literal regex")),
                ("github-user", Regex::new(r"ghs_[a-zA-Z0-9]{36}").expect("literal regex")),
                ("github-refresh", Regex::new(r"ghr_[a-zA-Z0-9]{36}").expect("literal regex")),
                ("aws-key", Regex::new(r"AKIA[0-9A-Z]{16}").expect("literal regex")),
                ("aws-secret", Regex::new(r#"(?i)aws_secret_access_key\s*[:=]\s*['"]?[a-zA-Z0-9/+]{40}"#).expect("literal regex")),
                ("private-key", Regex::new(r"-----BEGIN (RSA |EC )?PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----").expect("literal regex")),
                ("slack", Regex::new(r"xox[abpors]-[a-zA-Z0-9]{10,}").expect("literal regex")),
                ("gitlab", Regex::new(r"glpat-[a-zA-Z0-9\-]{20,}").expect("literal regex")),
                ("jwt", Regex::new(r"[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}").expect("literal regex")),
                ("generic-key", Regex::new(r#"(?i)(?:api[_-]?key|secret)\s*[:=]\s*['"]?[a-zA-Z0-9_\-]{16,}"#).expect("literal regex")),
                ("password", Regex::new(r#"(?i)password\s*[:=]\s*['"]?[^'"\s]{8,}"#).expect("literal regex")),
            ],
            secret_strs: vec!["sk-", "api_key", "api-key", "apikey", "BEGIN PRIVATE KEY"],
            pii_regexes: vec![
                ("email", Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").expect("literal regex")),
                ("phone", Regex::new(r"\b\+?1?\d{10,15}\b").expect("literal regex")),
                ("ipv4", Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").expect("literal regex")),
                ("ipv6", Regex::new(r"[a-fA-F0-9]{1,4}:[a-fA-F0-9:]{2,}:").expect("literal regex")),
                ("home-path", Regex::new(r"/home/[a-zA-Z0-9_]+").expect("literal regex")),
            ],
        }
    }
}

impl Redactor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 分析文本风险等级 + 命中的规则描述
    pub fn analyze(&self, text: &str) -> (RiskLevel, Vec<String>) {
        for &(desc, ref re) in &self.secret_regexes {
            if re.is_match(text) {
                return (RiskLevel::Dangerous, vec![desc.to_string()]);
            }
        }
        for &p in &self.secret_strs {
            if text.contains(p) {
                return (RiskLevel::Dangerous, vec![p.to_string()]);
            }
        }
        let mut matches = Vec::new();
        for &(desc, ref re) in &self.pii_regexes {
            if re.is_match(text) {
                matches.push(desc.to_string());
            }
        }
        if !matches.is_empty() {
            return (RiskLevel::Suspicious, matches);
        }
        (RiskLevel::Safe, matches)
    }

    /// 是否安全 (无可脱敏内容)
    pub fn is_safe(&self, text: &str) -> bool {
        matches!(self.analyze(text).0, RiskLevel::Safe)
    }

    /// 将文本中的 secrets 替换为占位符 [REDACTED] (PII 也替换，默认严格脱敏)
    pub fn redact(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (_, re) in &self.secret_regexes {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
        for &p in &self.secret_strs {
            result = result.replace(p, "[REDACTED]");
        }
        for (_, re) in &self.pii_regexes {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
        result
    }

    /// 仅替换 secrets (保留 PII) — 用于需保留 email 等但屏蔽密钥的场景
    pub fn redact_secrets_only(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (_, re) in &self.secret_regexes {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
        for &p in &self.secret_strs {
            result = result.replace(p, "[REDACTED]");
        }
        result
    }
}

/// 便捷函数: 全局脱敏 API
pub fn redact(text: &str) -> String {
    Redactor::new().redact(text)
}

/// 便捷函数: 仅 secrets
pub fn redact_secrets(text: &str) -> String {
    Redactor::new().redact_secrets_only(text)
}

/// 便捷函数: 风险分析
pub fn analyze(text: &str) -> (RiskLevel, Vec<String>) {
    Redactor::new().analyze(text)
}

/// 便捷函数: 判定安全
pub fn is_safe(text: &str) -> bool {
    Redactor::new().is_safe(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_api_key() {
        let r = Redactor::new();
        let redacted = r.redact("OpenAI key sk-abcdef1234567890 was used");
        assert!(!redacted.contains("sk-abcdef"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_aws_key() {
        let r = Redactor::new();
        let redacted = r.redact("AKIAIOSFODNN7EXAMPLE123456");
        assert!(!redacted.contains("AKIA"));
    }

    #[test]
    fn test_analyze_dangerous_on_secret() {
        let r = Redactor::new();
        let (level, matched) = r.analyze(&format!("token ghp_{}", "A".repeat(36)));
        assert_eq!(level, RiskLevel::Dangerous);
        assert!(!matched.is_empty());
    }

    #[test]
    fn test_analyze_suspicious_on_email() {
        let r = Redactor::new();
        let (level, matched) = r.analyze("contact me at user@example.com");
        assert_eq!(level, RiskLevel::Suspicious);
        assert!(matched.contains(&"email".to_string()));
    }

    #[test]
    fn test_safe_text() {
        let r = Redactor::new();
        assert!(r.is_safe("plain harmless sentence"));
        assert!(!r.is_safe("api_key: abcdef1234567890name"));
    }

    #[test]
    fn test_redact_secrets_only_keeps_email() {
        let r = Redactor::new();
        let out = r.redact_secrets_only("email user@example.com api_key=abcdef1234567890xxx");
        assert!(out.contains("user@example.com"));
        assert!(!out.contains("abcdef1234567890xxx"));
    }

    #[test]
    fn test_redact_private_key_block() {
        let r = Redactor::new();
        let pk = "-----BEGIN RSA PRIVATE KEY-----\nMIICXAIBAAKBgQC\n-----END RSA PRIVATE KEY-----";
        let out = r.redact(pk);
        assert!(!out.contains("PRIVATE KEY"));
    }
}