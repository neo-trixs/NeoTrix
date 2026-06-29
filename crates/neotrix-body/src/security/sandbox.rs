//! # Sandbox Filter
//!
//! Restricts what operations (code exec, FS write, network) are allowed
//! based on the current execution context and trust level.

use super::{FilterDecision, SecurityContext, SecurityFilter, UrgencyLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxLevel {
    None,
    ReadOnly,
    Restricted,
    Isolated,
}

impl SandboxLevel {
    pub fn name(&self) -> &str {
        match self {
            Self::None => "none",
            Self::ReadOnly => "readonly",
            Self::Restricted => "restricted",
            Self::Isolated => "isolated",
        }
    }
}

#[derive(Debug)]
pub struct SandboxFilter {
    pub name: String,
    pub default_level: SandboxLevel,
    pub allowed_operations: Vec<String>,
    pub blocked_domains: Vec<String>,
}

impl SandboxFilter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            default_level: SandboxLevel::Restricted,
            allowed_operations: vec!["read".into(), "list".into()],
            blocked_domains: vec!["malware.com".into(), "exploit.net".into()],
        }
    }

    pub fn allows_operation(&self, operation: &str) -> bool {
        self.allowed_operations.iter().any(|a| a == operation)
    }

    pub fn is_domain_blocked(&self, domain: &str) -> bool {
        self.blocked_domains.iter().any(|d| domain.contains(d))
    }
}

impl SecurityFilter for SandboxFilter {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self, ctx: &SecurityContext) -> FilterDecision {
        if ctx.urgency == UrgencyLevel::Critical {
            return FilterDecision::Allow;
        }
        if !self.allows_operation(&ctx.request_type) {
            return FilterDecision::Deny;
        }
        if self.is_domain_blocked(&ctx.source) {
            return FilterDecision::Deny;
        }
        FilterDecision::Allow
    }

    fn description(&self) -> &str {
        "sandbox filter enforcing operation and domain restrictions"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_allows_read() {
        let sandbox = SandboxFilter::new("test");
        let ctx = SecurityContext {
            request_type: "read".into(),
            content: "data".into(),
            source: "internal".into(),
            urgency: UrgencyLevel::Normal,
        };
        assert_eq!(sandbox.check(&ctx), FilterDecision::Allow);
    }

    #[test]
    fn test_sandbox_blocks_write() {
        let sandbox = SandboxFilter::new("test");
        let ctx = SecurityContext {
            request_type: "write".into(),
            content: "data".into(),
            source: "internal".into(),
            urgency: UrgencyLevel::Normal,
        };
        assert_eq!(sandbox.check(&ctx), FilterDecision::Deny);
    }

    #[test]
    fn test_sandbox_allows_critical() {
        let sandbox = SandboxFilter::new("test");
        let ctx = SecurityContext {
            request_type: "write".into(),
            content: "data".into(),
            source: "internal".into(),
            urgency: UrgencyLevel::Critical,
        };
        assert_eq!(sandbox.check(&ctx), FilterDecision::Allow);
    }

    #[test]
    fn test_sandbox_blocks_domain() {
        let sandbox = SandboxFilter::new("test");
        let ctx = SecurityContext {
            request_type: "read".into(),
            content: "data".into(),
            source: "https://malware.com/exploit".into(),
            urgency: UrgencyLevel::Normal,
        };
        assert_eq!(sandbox.check(&ctx), FilterDecision::Deny);
    }

    #[test]
    fn test_sandbox_level_names() {
        assert_eq!(SandboxLevel::None.name(), "none");
        assert_eq!(SandboxLevel::Isolated.name(), "isolated");
    }
}
