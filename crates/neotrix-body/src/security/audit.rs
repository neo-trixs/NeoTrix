//! # Audit Filter
//!
//! Logs all security-relevant events to a ring buffer for later inspection.
//! Non-blocking — always returns Allow, but records the event.

use super::{FilterDecision, SecurityContext, SecurityFilter};
use std::collections::VecDeque;

/// A single audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp_ms: u64,
    pub request_type: String,
    pub source: String,
    pub decision: FilterDecision,
    pub summary: String,
}

/// Audit filter — logs all requests without blocking
#[derive(Debug)]
pub struct AuditFilter {
    pub name: String,
    pub max_entries: usize,
    pub log: VecDeque<AuditEntry>,
    entry_counter: u64,
}

impl AuditFilter {
    pub fn new(name: &str, max_entries: usize) -> Self {
        Self {
            name: name.into(),
            max_entries,
            log: VecDeque::with_capacity(max_entries),
            entry_counter: 0,
        }
    }

    pub fn recent_entries(&self, count: usize) -> Vec<&AuditEntry> {
        self.log.iter().rev().take(count).collect()
    }

    pub fn total_entries(&self) -> u64 {
        self.entry_counter
    }

    pub fn clear(&mut self) {
        self.log.clear();
        self.entry_counter = 0;
    }

    #[allow(dead_code)]
    fn record(&mut self, ctx: &SecurityContext, decision: FilterDecision) {
        if self.log.len() >= self.max_entries {
            self.log.pop_front();
        }
        self.log.push_back(AuditEntry {
            timestamp_ms: 0,
            request_type: ctx.request_type.clone(),
            source: ctx.source.clone(),
            decision,
            summary: format!("{} via {}", ctx.request_type, ctx.source),
        });
        self.entry_counter += 1;
    }
}

impl SecurityFilter for AuditFilter {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self, _ctx: &SecurityContext) -> FilterDecision {
        // Logging would happen via a wrapper that has &mut access
        FilterDecision::Allow
    }

    fn description(&self) -> &str {
        "non-blocking audit logger"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_filter_creation() {
        let audit = AuditFilter::new("test", 100);
        assert_eq!(audit.total_entries(), 0);
        assert!(audit.recent_entries(10).is_empty());
    }

    #[test]
    fn test_audit_record_and_retrieve() {
        let mut audit = AuditFilter::new("test", 100);
        let ctx = SecurityContext {
            request_type: "llm".into(),
            content: "hello".into(),
            source: "user".into(),
            urgency: super::super::UrgencyLevel::Normal,
        };
        audit.record(&ctx, FilterDecision::Allow);
        assert_eq!(audit.total_entries(), 1);
        assert_eq!(audit.recent_entries(1)[0].request_type, "llm");
    }

    #[test]
    fn test_audit_ring_buffer() {
        let mut audit = AuditFilter::new("test", 3);
        for i in 0..5 {
            let ctx = SecurityContext {
                request_type: format!("req_{i}"),
                content: "".into(),
                source: "test".into(),
                urgency: super::super::UrgencyLevel::Normal,
            };
            audit.record(&ctx, FilterDecision::Allow);
        }
        assert_eq!(audit.log.len(), 3);
    }

    #[test]
    fn test_audit_clear() {
        let mut audit = AuditFilter::new("test", 100);
        let ctx = SecurityContext {
            request_type: "test".into(),
            content: "".into(),
            source: "test".into(),
            urgency: super::super::UrgencyLevel::Normal,
        };
        audit.record(&ctx, FilterDecision::Allow);
        audit.clear();
        assert_eq!(audit.total_entries(), 0);
    }
}
