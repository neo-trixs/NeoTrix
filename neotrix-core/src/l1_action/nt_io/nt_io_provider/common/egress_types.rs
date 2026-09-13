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
            host == suffix || host.ends_with(&format!(".{suffix}"))
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
