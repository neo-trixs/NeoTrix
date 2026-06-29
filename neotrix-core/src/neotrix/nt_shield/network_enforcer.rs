#![allow(dead_code)]
use std::sync::OnceLock;
use std::sync::RwLock;

use super::network_egress::{global_egress, EgressAction};
use super::tool_permissions::NetworkPolicy;
use serde::{Deserialize, Serialize};

static GLOBAL_ENFORCER: OnceLock<NetworkEnforcer> = OnceLock::new();

/// Enforces NetworkPolicy before any outbound connection.
pub struct NetworkEnforcer {
    policy: RwLock<NetworkPolicy>,
}

impl Default for NetworkEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkEnforcer {
    pub fn new() -> Self {
        Self {
            policy: RwLock::new(NetworkPolicy::default()),
        }
    }

    /// Check if URL is allowed. Returns Ok(()) or an error message.
    pub fn check_url(&self, url: &str) -> Result<(), String> {
        if url.trim().is_empty() {
            return Err("Empty URL: request blocked".to_string());
        }

        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL '{}': {}", url, e))?;

        let host = parsed
            .host_str()
            .ok_or_else(|| format!("URL '{}' has no host", url))?;

        let policy = self
            .policy
            .read()
            .map_err(|e| format!("Policy lock error: {}", e))?;

        if !policy.check_access(url) {
            return Err(format!(
                "Network policy denied access to '{}' (host: {})",
                url, host
            ));
        }

        // Second layer: egress policy — zero-trust allowlist with default-deny
        let port = parsed
            .port()
            .or_else(|| {
                if parsed.scheme() == "https" {
                    Some(443)
                } else if parsed.scheme() == "http" {
                    Some(80)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let egress_action = global_egress()
            .write()
            .map_err(|e| format!("Egress lock error: {}", e))?
            .check(host, port);

        match egress_action {
            EgressAction::Allow | EgressAction::Log => Ok(()),
            EgressAction::Deny => Err(format!(
                "Egress policy denied access to '{}' (host: {}, port: {})",
                url, host, port
            )),
        }
    }

    /// Update the network policy at runtime.
    pub fn set_policy(&self, policy: NetworkPolicy) {
        if let Ok(mut p) = self.policy.write() {
            *p = policy;
        }
    }

    /// Get current policy.
    pub fn policy(&self) -> NetworkPolicy {
        self.policy
            .read()
            .map(|p| p.clone())
            .unwrap_or(NetworkPolicy::DefaultDeny)
    }
}

/// Global singleton enforcer.
pub fn global_enforcer() -> &'static NetworkEnforcer {
    GLOBAL_ENFORCER.get_or_init(|| {
        let enforcer = NetworkEnforcer::new();
        // Attempt to load persisted policy from config
        if let Some(home) = dirs::home_dir() {
            let path = home
                .join(".config")
                .join("neotrix")
                .join("network_policy.json");
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(policy) = serde_json::from_str::<NetworkPolicy>(&content) {
                    enforcer.set_policy(policy);
                }
            }
        }
        enforcer
    })
}

/// Persist current policy to disk.
pub fn persist_policy() {
    let enforcer = global_enforcer();
    let policy = enforcer.policy();
    if let Some(home) = dirs::home_dir() {
        let path = home
            .join(".config")
            .join("neotrix")
            .join("network_policy.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&policy) {
            let tmp = path.with_extension("tmp");
            let _ = std::fs::write(&tmp, json);
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

// ============================================================================
// Domain Allowlist with wildcard subdomain support + Blocklist + Traffic Log
// References:
//   - INNOQ "AI Sandboxing" (default-deny + allowlist pattern)
//   - Blaxel "Agent Security" (wildcard subdomain allowlist)
//   - CageClaw "Execution Isolation" (traffic monitoring + blocklist)
// ============================================================================

/// A structured domain allowlist with wildcard subdomain support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAllowlist {
    /// Exact domain matches (e.g., "api.openai.com")
    pub exact: Vec<String>,
    /// Wildcard subdomain patterns (e.g., "*.openai.com" matches "api.openai.com", "beta.openai.com")
    pub wildcard: Vec<String>,
}

impl DomainAllowlist {
    pub fn new() -> Self {
        Self {
            exact: Vec::new(),
            wildcard: Vec::new(),
        }
    }

    /// Check if a hostname matches the allowlist
    pub fn allows(&self, host: &str) -> bool {
        if self.exact.iter().any(|e| e == host) {
            return true;
        }
        for pattern in &self.wildcard {
            if let Some(domain) = pattern.strip_prefix("*.") {
                if host.ends_with(domain) && host != domain {
                    return true;
                }
            }
        }
        false
    }

    pub fn add_exact(&mut self, domain: &str) {
        if !self.exact.contains(&domain.to_string()) {
            self.exact.push(domain.to_string());
        }
    }

    pub fn add_wildcard(&mut self, pattern: &str) {
        if !self.wildcard.contains(&pattern.to_string()) {
            self.wildcard.push(pattern.to_string());
        }
    }

    pub fn remove(&mut self, domain: &str) {
        self.exact.retain(|d| d != domain);
        self.wildcard.retain(|d| d != domain);
    }

    pub fn bulk_add_exact(&mut self, domains: &[&str]) {
        for d in domains {
            self.add_exact(d);
        }
    }

    pub fn clear(&mut self) {
        self.exact.clear();
        self.wildcard.clear();
    }
}

impl Default for DomainAllowlist {
    fn default() -> Self {
        Self::new()
    }
}

/// Traffic log entry for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficEntry {
    pub timestamp: u64,
    pub url: String,
    pub host: String,
    pub action: String,
    pub reason: String,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficLog {
    pub entries: Vec<TrafficEntry>,
    pub max_entries: usize,
    pub total_blocked: u64,
    pub total_allowed: u64,
}

impl TrafficLog {
    pub fn new(max: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max),
            max_entries: max,
            total_blocked: 0,
            total_allowed: 0,
        }
    }

    pub fn record(&mut self, url: &str, host: &str, action: &str, reason: &str, blocked: bool) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(TrafficEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            url: url.to_string(),
            host: host.to_string(),
            action: action.to_string(),
            reason: reason.to_string(),
            blocked,
        });
        if blocked {
            self.total_blocked += 1;
        } else {
            self.total_allowed += 1;
        }
    }

    pub fn recent_blocked(&self, n: usize) -> Vec<&TrafficEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.blocked)
            .take(n)
            .collect()
    }

    pub fn block_ratio(&self) -> f64 {
        let total = self.total_blocked + self.total_allowed;
        if total == 0 {
            return 0.0;
        }
        self.total_blocked as f64 / total as f64
    }
}

/// Combined proxy sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySandboxConfig {
    pub allowlist: DomainAllowlist,
    pub blocklist: Vec<String>,
    pub mode: ProxySandboxMode,
    pub traffic_log: TrafficLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProxySandboxMode {
    /// Only allowlisted domains (default)
    AllowlistOnly,
    /// Allowlist takes priority, blocklist overrides
    Hybrid,
    /// Monitor only — log but don't block
    MonitorOnly,
}

impl Default for ProxySandboxConfig {
    fn default() -> Self {
        Self {
            allowlist: DomainAllowlist::new(),
            blocklist: Vec::new(),
            mode: ProxySandboxMode::AllowlistOnly,
            traffic_log: TrafficLog::new(1000),
        }
    }
}

impl ProxySandboxConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a URL is allowed by the sandbox config
    pub fn check_url(&mut self, url: &str) -> Result<(), String> {
        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        let host = parsed.host_str().unwrap_or("unknown");

        // Check blocklist first (explicit deny)
        if self.mode != ProxySandboxMode::MonitorOnly {
            if self.blocklist.iter().any(|b| host.contains(b)) {
                self.traffic_log
                    .record(url, host, "BLOCK", "blocklisted", true);
                return Err(format!("Blocklisted domain: {}", host));
            }
        }

        // Check allowlist
        let allowed = self.allowlist.allows(host);
        match self.mode {
            ProxySandboxMode::MonitorOnly => {
                self.traffic_log
                    .record(url, host, "MONITOR", "monitor mode", false);
                Ok(())
            }
            ProxySandboxMode::AllowlistOnly => {
                if allowed {
                    self.traffic_log
                        .record(url, host, "ALLOW", "allowlisted", false);
                    Ok(())
                } else {
                    self.traffic_log
                        .record(url, host, "BLOCK", "not in allowlist", true);
                    Err(format!("Domain not in allowlist: {}", host))
                }
            }
            ProxySandboxMode::Hybrid => {
                if allowed {
                    self.traffic_log
                        .record(url, host, "ALLOW", "allowlisted", false);
                    Ok(())
                } else {
                    self.traffic_log
                        .record(url, host, "BLOCK", "not in allowlist", true);
                    Err(format!("Domain not in allowlist: {}", host))
                }
            }
        }
    }

    pub fn add_to_allowlist(&mut self, domain: &str) {
        self.allowlist.add_exact(domain);
    }

    pub fn add_wildcard(&mut self, pattern: &str) {
        self.allowlist.add_wildcard(pattern);
    }

    pub fn block_domain(&mut self, domain: &str) {
        if !self.blocklist.contains(&domain.to_string()) {
            self.blocklist.push(domain.to_string());
        }
    }

    pub fn bulk_allow(&mut self, domains: &[&str]) {
        for d in domains {
            self.add_to_allowlist(d);
        }
    }

    pub fn bulk_block(&mut self, domains: &[&str]) {
        for d in domains {
            self.block_domain(d);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let enforcer = NetworkEnforcer::new();
        assert!(enforcer.check_url("https://api.openai.com/v1/chat").is_ok());
        assert!(enforcer
            .check_url("https://api.anthropic.com/v1/messages")
            .is_ok());
        assert!(enforcer.check_url("https://example.com").is_err());
    }

    #[test]
    fn test_allow_all() {
        let enforcer = NetworkEnforcer::new();
        enforcer.set_policy(NetworkPolicy::AllowAll);
        assert!(enforcer.check_url("https://evil.com/malware").is_ok());
    }

    #[test]
    fn test_deny_all() {
        let enforcer = NetworkEnforcer::new();
        enforcer.set_policy(NetworkPolicy::DenyAll);
        assert!(enforcer
            .check_url("https://api.openai.com/v1/chat")
            .is_err());
        assert!(enforcer.check_url("https://example.com").is_err());
    }

    #[test]
    fn test_set_policy_updates() {
        let enforcer = NetworkEnforcer::new();
        assert!(enforcer.check_url("https://api.openai.com/v1/chat").is_ok());
        enforcer.set_policy(NetworkPolicy::DenyAll);
        assert!(enforcer
            .check_url("https://api.openai.com/v1/chat")
            .is_err());
        enforcer.set_policy(NetworkPolicy::AllowAll);
        assert!(enforcer.check_url("https://api.openai.com/v1/chat").is_ok());
    }

    #[test]
    fn test_allow_list() {
        let enforcer = NetworkEnforcer::new();
        let allowed = vec!["my-custom-api.com".to_string()];
        enforcer.set_policy(NetworkPolicy::AllowList(allowed));
        assert!(enforcer
            .check_url("https://my-custom-api.com/endpoint")
            .is_ok());
        assert!(enforcer
            .check_url("https://api.openai.com/v1/chat")
            .is_err());
    }

    #[test]
    fn test_empty_url() {
        let enforcer = NetworkEnforcer::new();
        assert!(enforcer.check_url("").is_err());
        assert!(enforcer.check_url("   ").is_err());
    }

    #[test]
    fn test_invalid_url() {
        let enforcer = NetworkEnforcer::new();
        assert!(enforcer.check_url("not a url").is_err());
    }

    #[test]
    fn test_policy_persistence() {
        let enforcer = NetworkEnforcer::new();
        enforcer.set_policy(NetworkPolicy::AllowAll);
        assert_eq!(enforcer.policy(), NetworkPolicy::AllowAll);
        enforcer.set_policy(NetworkPolicy::DenyAll);
        assert_eq!(enforcer.policy(), NetworkPolicy::DenyAll);
    }

    #[test]
    fn test_default_deny_allows_openai() {
        let enforcer = NetworkEnforcer::new();
        assert!(enforcer
            .check_url("https://api.openai.com/v1/chat/completions")
            .is_ok());
    }

    #[test]
    fn test_default_deny_blocks_unknown() {
        let enforcer = NetworkEnforcer::new();
        assert!(enforcer
            .check_url("https://unknown-malicious-site.com/")
            .is_err());
    }

    #[test]
    fn test_global_enforcer_singleton() {
        let e1 = global_enforcer();
        let e2 = global_enforcer();
        assert!(std::ptr::eq(e1, e2));
    }

    // --- Domain allowlist tests ---

    #[test]
    fn test_domain_allowlist_exact() {
        let mut al = DomainAllowlist::new();
        al.add_exact("api.openai.com");
        assert!(al.allows("api.openai.com"));
        assert!(!al.allows("evil.openai.com"));
    }

    #[test]
    fn test_domain_allowlist_wildcard() {
        let mut al = DomainAllowlist::new();
        al.add_wildcard("*.openai.com");
        assert!(al.allows("api.openai.com"));
        assert!(al.allows("beta.openai.com"));
        assert!(!al.allows("openai.com"));
        assert!(!al.allows("evil.com"));
    }

    #[test]
    fn test_domain_allowlist_bulk_add() {
        let mut al = DomainAllowlist::new();
        al.bulk_add_exact(&["a.com", "b.com", "c.com"]);
        assert_eq!(al.exact.len(), 3);
    }

    #[test]
    fn test_domain_allowlist_remove() {
        let mut al = DomainAllowlist::new();
        al.add_exact("api.openai.com");
        al.remove("api.openai.com");
        assert!(!al.allows("api.openai.com"));
    }

    #[test]
    fn test_domain_allowlist_clear() {
        let mut al = DomainAllowlist::new();
        al.add_exact("a.com");
        al.add_wildcard("*.b.com");
        al.clear();
        assert!(al.exact.is_empty());
        assert!(al.wildcard.is_empty());
    }

    // --- Traffic log tests ---

    #[test]
    fn test_traffic_log_records() {
        let mut log = TrafficLog::new(10);
        log.record("https://good.com", "good.com", "ALLOW", "ok", false);
        log.record("https://evil.com", "evil.com", "BLOCK", "blocked", true);
        assert_eq!(log.total_allowed, 1);
        assert_eq!(log.total_blocked, 1);
    }

    #[test]
    fn test_traffic_log_max_entries() {
        let mut log = TrafficLog::new(3);
        for i in 0..10 {
            log.record(
                &format!("https://site{}.com", i),
                &format!("site{}.com", i),
                "ALLOW",
                "",
                false,
            );
        }
        assert_eq!(log.entries.len(), 3);
    }

    #[test]
    fn test_traffic_log_recent_blocked() {
        let mut log = TrafficLog::new(10);
        log.record("https://a.com", "a.com", "ALLOW", "", false);
        log.record("https://b.com", "b.com", "BLOCK", "", true);
        log.record("https://c.com", "c.com", "BLOCK", "", true);
        let blocked = log.recent_blocked(1);
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].host, "c.com");
    }

    #[test]
    fn test_traffic_log_block_ratio() {
        let mut log = TrafficLog::new(10);
        log.record("https://a.com", "a.com", "ALLOW", "", false);
        log.record("https://b.com", "b.com", "BLOCK", "", true);
        log.record("https://c.com", "c.com", "ALLOW", "", false);
        assert!((log.block_ratio() - 1.0 / 3.0).abs() < 1e-9);
    }

    // --- ProxySandboxConfig tests ---

    #[test]
    fn test_sandbox_allowlist_only_blocks_unknown() {
        let mut config = ProxySandboxConfig::new();
        config.add_to_allowlist("api.openai.com");
        assert!(config.check_url("https://api.openai.com/v1/chat").is_ok());
        assert!(config.check_url("https://evil.com").is_err());
    }

    #[test]
    fn test_sandbox_wildcard_allowlist() {
        let mut config = ProxySandboxConfig::new();
        config.add_wildcard("*.openai.com");
        assert!(config.check_url("https://api.openai.com").is_ok());
        assert!(config.check_url("https://beta.openai.com").is_ok());
    }

    #[test]
    fn test_sandbox_blocklist_overrides() {
        let mut config = ProxySandboxConfig::new();
        config.add_to_allowlist("api.openai.com");
        config.block_domain("openai.com");
        assert!(config.check_url("https://api.openai.com").is_err());
    }

    #[test]
    fn test_sandbox_monitor_mode() {
        let mut config = ProxySandboxConfig::new();
        config.mode = ProxySandboxMode::MonitorOnly;
        // In monitor mode, everything is allowed but logged
        assert!(config.check_url("https://evil.com").is_ok());
        assert!(config.traffic_log.total_allowed > 0);
    }

    #[test]
    fn test_sandbox_bulk_allow_block() {
        let mut config = ProxySandboxConfig::new();
        config.bulk_allow(&["good1.com", "good2.com"]);
        config.bulk_block(&["bad1.com", "bad2.com"]);
        assert_eq!(config.allowlist.exact.len(), 2);
        assert_eq!(config.blocklist.len(), 2);
    }
}
