#![allow(dead_code)]
use std::sync::OnceLock;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

static GLOBAL_EGRESS: OnceLock<NetworkEgressPolicy> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EgressAction {
    Allow,
    Deny,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressRule {
    /// Glob pattern: "*.google.com", "api.openai.com"
    pub host_pattern: String,
    pub port: Option<u16>,
    /// "tcp", "udp", "http", "ws", or empty for any
    pub protocol: Vec<String>,
    pub action: EgressAction,
}

impl EgressRule {
    fn matches(&self, host: &str, port: u16) -> bool {
        if let Some(p) = self.port {
            if p != port {
                return false;
            }
        }
        pattern_matches(&self.host_pattern, host)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEgressPolicy {
    pub default_action: EgressAction,
    pub allowlist: Vec<EgressRule>,
    pub denylist: Vec<EgressRule>,
    pub enabled: bool,
    /// Counter: (blocked, allowed)
    pub stats: (u64, u64),
}

/// Maximum number of rules per list (defense-in-depth)
const MAX_RULES: usize = 1000;

impl Default for NetworkEgressPolicy {
    fn default() -> Self {
        Self {
            default_action: EgressAction::Deny,
            allowlist: default_allowlist(),
            denylist: Vec::new(),
            enabled: cfg!(not(debug_assertions)),
            stats: (0, 0),
        }
    }
}

impl NetworkEgressPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let mut policy = Self::default();
        if let Ok(extra) = std::env::var("NEOTRIX_ALLOW_NET_HOSTS") {
            for host in extra.split(',').take(MAX_RULES) {
                let host = host.trim();
                if !host.is_empty() {
                    policy.allowlist.push(EgressRule {
                        host_pattern: host.to_string(),
                        port: None,
                        protocol: vec!["tcp".to_string()],
                        action: EgressAction::Allow,
                    });
                }
            }
        }
        policy
    }

    pub fn check(&mut self, host: &str, port: u16) -> EgressAction {
        if !self.enabled {
            self.stats.1 += 1;
            return EgressAction::Allow;
        }
        // denylist checked first — explicit deny
        for rule in &self.denylist {
            if rule.matches(host, port) {
                self.stats.0 += 1;
                return EgressAction::Deny;
            }
        }
        // allowlist checked next — explicit allow
        for rule in &self.allowlist {
            if rule.matches(host, port) {
                self.stats.1 += 1;
                return EgressAction::Allow;
            }
        }
        // fallback to default action
        match self.default_action {
            EgressAction::Deny => self.stats.0 += 1,
            EgressAction::Allow => self.stats.1 += 1,
            EgressAction::Log => self.stats.1 += 1,
        }
        self.default_action.clone()
    }

    pub fn stats(&self) -> (u64, u64) {
        self.stats
    }

    pub fn add_allow_rule(&mut self, host_pattern: &str) {
        if self.allowlist.len() >= MAX_RULES {
            return;
        }
        self.allowlist.push(EgressRule {
            host_pattern: host_pattern.to_string(),
            port: None,
            protocol: vec!["tcp".to_string()],
            action: EgressAction::Allow,
        });
    }

    pub fn add_deny_rule(&mut self, host_pattern: &str) {
        if self.denylist.len() >= MAX_RULES {
            return;
        }
        self.denylist.push(EgressRule {
            host_pattern: host_pattern.to_string(),
            port: None,
            protocol: vec!["tcp".to_string()],
            action: EgressAction::Deny,
        });
    }
}

fn default_allowlist() -> Vec<EgressRule> {
    let domains = [
        "api.openai.com",
        "api.anthropic.com",
        "api.groq.com",
        "api.together.xyz",
        "deepseek.com",
        "gemini.googleapis.com",
        "generativelanguage.googleapis.com",
        "api.mistral.ai",
        "api.cohere.ai",
        "api.openrouter.ai",
        "api.fireworks.ai",
        "api.github.com",
        "github.com",
        "raw.githubusercontent.com",
        "pypi.org",
        "crates.io",
        "static.crates.io",
        "arxiv.org",
        "huggingface.co",
        "cdn.huggingface.co",
        "upload.wikimedia.org",
        "en.wikipedia.org",
    ];
    domains
        .iter()
        .map(|d| EgressRule {
            host_pattern: d.to_string(),
            port: Some(443),
            protocol: vec!["tcp".to_string()],
            action: EgressAction::Allow,
        })
        .collect()
}

fn pattern_matches(pattern: &str, host: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(domain) = pattern.strip_prefix("*.") {
        host.ends_with(domain) && host != domain
    } else {
        pattern == host
    }
}

pub fn global_egress() -> &'static RwLock<NetworkEgressPolicy> {
    static EGRESS: OnceLock<RwLock<NetworkEgressPolicy>> = OnceLock::new();
    EGRESS.get_or_init(|| RwLock::new(NetworkEgressPolicy::from_env()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy() -> NetworkEgressPolicy {
        let mut p = NetworkEgressPolicy::new();
        p.enabled = true;
        p.denylist.push(EgressRule {
            host_pattern: "*.evil.com".to_string(),
            port: None,
            protocol: vec![],
            action: EgressAction::Deny,
        });
        p
    }

    #[test]
    fn test_default_allow_openai() {
        let mut p = NetworkEgressPolicy::new();
        p.enabled = true;
        assert_eq!(p.check("api.openai.com", 443), EgressAction::Allow);
    }

    #[test]
    fn test_denylist_blocks_evil() {
        let mut p = test_policy();
        assert_eq!(p.check("sub.evil.com", 443), EgressAction::Deny);
    }

    #[test]
    fn test_default_deny_unknown() {
        let mut p = test_policy();
        assert_eq!(p.check("unknown-malware.com", 443), EgressAction::Deny);
    }

    #[test]
    fn test_disabled_policy_allows_all() {
        let mut p = test_policy();
        p.enabled = false;
        assert_eq!(p.check("evil.com", 443), EgressAction::Allow);
    }

    #[test]
    fn test_wildcard_pattern() {
        assert!(pattern_matches("*.google.com", "api.google.com"));
        assert!(!pattern_matches("*.google.com", "google.com"));
        assert!(pattern_matches("*", "anything.com"));
        assert!(pattern_matches("exact.com", "exact.com"));
        assert!(!pattern_matches("exact.com", "not-exact.com"));
    }

    #[test]
    fn test_from_env_parses() {
        std::env::set_var("NEOTRIX_ALLOW_NET_HOSTS", "my-api.com,*.custom.io");
        let p = NetworkEgressPolicy::from_env();
        assert!(p.allowlist.iter().any(|r| r.host_pattern == "my-api.com"));
        assert!(p.allowlist.iter().any(|r| r.host_pattern == "*.custom.io"));
        std::env::remove_var("NEOTRIX_ALLOW_NET_HOSTS");
    }

    #[test]
    fn test_tracks_stats() {
        let mut p = test_policy();
        let _ = p.check("api.openai.com", 443);
        let _ = p.check("sub.evil.com", 443);
        let _ = p.check("unknown.com", 443);
        assert_eq!(p.stats(), (2, 1));
    }

    #[test]
    fn test_port_matching() {
        let mut p = NetworkEgressPolicy::new();
        p.enabled = true;
        p.allowlist.push(EgressRule {
            host_pattern: "api.test.com".to_string(),
            port: Some(443),
            protocol: vec!["tcp".to_string()],
            action: EgressAction::Allow,
        });
        assert_eq!(p.check("api.test.com", 443), EgressAction::Allow);
        assert_eq!(p.check("api.test.com", 80), EgressAction::Deny);
    }

    #[test]
    fn test_denylist_priority_over_allowlist() {
        let mut p = NetworkEgressPolicy::new();
        p.enabled = true;
        p.allowlist.push(EgressRule {
            host_pattern: "api.openai.com".to_string(),
            port: None,
            protocol: vec![],
            action: EgressAction::Allow,
        });
        p.denylist.push(EgressRule {
            host_pattern: "api.openai.com".to_string(),
            port: None,
            protocol: vec![],
            action: EgressAction::Deny,
        });
        assert_eq!(p.check("api.openai.com", 443), EgressAction::Deny);
    }
}
