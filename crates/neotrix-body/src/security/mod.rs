//! # SecurityBus — Filter Chain
//!
//! Architecture: SecurityBus ← FilterChain ← Filter trait
//! Each request passes through the chain: prompt_guard → sandbox → audit → protect.
//! Filters can allow, deny, or modify requests.

use std::fmt;

pub type SecurityResult<T> = Result<T, SecurityError>;

#[derive(Debug, Clone)]
pub enum SecurityError {
    Blocked(String),
    RateLimited { reason: String, retry_after_ms: u64 },
    Violation(String),
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocked(r) => write!(f, "blocked: {r}"),
            Self::RateLimited { reason, retry_after_ms } => {
                write!(f, "rate limited ({retry_after_ms}ms): {reason}")
            }
            Self::Violation(v) => write!(f, "violation: {v}"),
        }
    }
}

/// Result of a security filter check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDecision {
    Allow,
    Deny,
    Modify,
}

/// Security context — information about the request being filtered
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub request_type: String,
    pub content: String,
    pub source: String,
    pub urgency: UrgencyLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrgencyLevel {
    Low,
    Normal,
    High,
    Critical,
}

/// A single security filter in the chain
pub trait SecurityFilter: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, ctx: &SecurityContext) -> FilterDecision;
    fn description(&self) -> &str;
}

/// SecurityBus — runs the filter chain on each request
#[derive(Debug)]
pub struct SecurityBus {
    filters: Vec<Box<dyn SecurityFilter>>,
    name: String,
}

impl SecurityBus {
    pub fn new(name: &str) -> Self {
        Self {
            filters: Vec::new(),
            name: name.into(),
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn SecurityFilter>) {
        self.filters.push(filter);
    }

    pub fn remove_filter(&mut self, name: &str) {
        self.filters.retain(|f| f.name() != name);
    }

    pub fn check(&self, ctx: &SecurityContext) -> SecurityResult<()> {
        for filter in &self.filters {
            match filter.check(ctx) {
                FilterDecision::Deny => {
                    return Err(SecurityError::Blocked(format!("blocked by {}", filter.name())));
                }
                FilterDecision::Modify | FilterDecision::Allow => {}
            }
        }
        Ok(())
    }

    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    pub fn report(&self) -> String {
        let names: Vec<&str> = self.filters.iter().map(|f| f.name()).collect();
        format!("sec:{}_filters_{}", self.name, names.join(","))
    }
}

impl Clone for SecurityBus {
    fn clone(&self) -> Self {
        Self {
            filters: Vec::new(),
            name: self.name.clone(),
        }
    }
}

/// Allow-all filter (for testing / passthrough)
#[derive(Debug)]
pub struct AllowAllFilter;

impl SecurityFilter for AllowAllFilter {
    fn name(&self) -> &str {
        "allow_all"
    }
    fn check(&self, _ctx: &SecurityContext) -> FilterDecision {
        FilterDecision::Allow
    }
    fn description(&self) -> &str {
        "allows all requests (passthrough)"
    }
}

pub mod prompt_guard;
pub mod sandbox;
pub mod audit;
pub mod prompt;
pub mod sandbox_v2;
pub mod audit_v2;
pub mod sentry;
pub mod proxy_rotator;
pub mod tls_fingerprint;
pub mod captcha_handler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_bus_allow_all() {
        let mut bus = SecurityBus::new("test");
        bus.add_filter(Box::new(AllowAllFilter));
        let ctx = SecurityContext {
            request_type: "llm".into(),
            content: "test".into(),
            source: "internal".into(),
            urgency: UrgencyLevel::Normal,
        };
        assert!(bus.check(&ctx).is_ok());
    }

    #[test]
    fn test_security_bus_reject() {
        #[derive(Debug)]
        struct DenyAll;
        impl SecurityFilter for DenyAll {
            fn name(&self) -> &str { "deny_all" }
            fn check(&self, _ctx: &SecurityContext) -> FilterDecision { FilterDecision::Deny }
            fn description(&self) -> &str { "denies all" }
        }
        let mut bus = SecurityBus::new("test");
        bus.add_filter(Box::new(DenyAll));
        let ctx = SecurityContext {
            request_type: "llm".into(),
            content: "test".into(),
            source: "internal".into(),
            urgency: UrgencyLevel::Normal,
        };
        assert!(bus.check(&ctx).is_err());
    }

    #[test]
    fn test_filter_count() {
        let mut bus = SecurityBus::new("test");
        bus.add_filter(Box::new(AllowAllFilter));
        bus.add_filter(Box::new(AllowAllFilter));
        assert_eq!(bus.filter_count(), 2);
    }
}
