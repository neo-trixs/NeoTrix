//! Audit logging for governance enforcement (R-P141, R-P142).
//!
//! Records enforcement decisions for compliance tracking and forensic
//! analysis. Supports queries by agent, policy, and recency.

use serde::{Deserialize, Serialize};

use super::enforcer::EnforcementResult;

/// A single audit log entry recording an enforcement decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry identifier.
    pub id: String,
    /// ID of the agent that performed the action.
    pub agent_id: String,
    /// The action that was evaluated.
    pub action: String,
    /// The enforcement result for this action.
    pub result: EnforcementResult,
    /// Unix timestamp of when this entry was recorded.
    pub timestamp: u64,
    /// ID of the policy that was checked (empty if multiple).
    pub policy_id: String,
}

/// In-memory audit log for enforcement decisions.
///
/// Stores audit entries and supports queries by agent, policy, and recency.
/// Entries are stored in insertion order.
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    max_entries: usize,
}

impl AuditLog {
    /// Create a new audit log with a maximum entry capacity.
    ///
    /// When the capacity is reached, oldest entries are evicted.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries.min(1024)),
            max_entries,
        }
    }

    /// Create a new audit log with default capacity (10,000 entries).
    pub fn with_default_capacity() -> Self {
        Self::new(10_000)
    }

    /// Log an enforcement result.
    ///
    /// Generates a unique ID and records the entry with the current timestamp.
    pub fn log(&mut self, result: &EnforcementResult, action: &str, agent_id: &str) -> &AuditEntry {
        let entry = AuditEntry {
            id: format!("audit_{}_{}", agent_id, self.entries.len()),
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            result: result.clone(),
            timestamp: timestamp_now(),
            policy_id: String::new(),
        };

        // Evict oldest if at capacity
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }

        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    /// Get the N most recent audit entries.
    pub fn get_recent(&self, n: usize) -> Vec<&AuditEntry> {
        let start = self.entries.len().saturating_sub(n);
        self.entries[start..].iter().collect()
    }

    /// Get all entries for a specific agent.
    pub fn get_by_agent(&self, agent_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .collect()
    }

    /// Get all entries related to a specific policy.
    pub fn get_by_policy(&self, policy_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.policy_id == policy_id || {
                    // Also match violations that reference this policy
                    e.result.violations.iter().any(|v| v.policy_id == policy_id)
                }
            })
            .collect()
    }

    /// Get the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get entries within a time range (inclusive).
    pub fn get_by_time_range(&self, from: u64, to: u64) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect()
    }

    /// Get only entries with blocking violations.
    pub fn get_blocked(&self) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| !e.result.allowed).collect()
    }
}

fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|e| {
            tracing::warn!("SystemTime before UNIX_EPOCH, falling back to 0: {}", e);
            std::time::Duration::ZERO
        })
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l6_meta::coordination::governance::enforcement::enforcer::Violation;
    use crate::l6_meta::coordination::governance::enforcement::policy::RuleSeverity;

    fn allowed_result() -> EnforcementResult {
        EnforcementResult::allowed()
    }

    fn blocked_result() -> EnforcementResult {
        EnforcementResult::blocked(
            vec![Violation {
                policy_id: "p1".into(),
                rule_id: "r1".into(),
                message: "violation".into(),
                severity: RuleSeverity::High,
            }],
            vec![],
        )
    }

    #[test]
    fn test_log_and_get_recent() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "deploy", "agent_1");
        log.log(&blocked_result(), "push", "agent_2");

        let recent = log.get_recent(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].action, "deploy");
        assert_eq!(recent[1].action, "push");
    }

    #[test]
    fn test_get_by_agent() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "action1", "agent_1");
        log.log(&allowed_result(), "action2", "agent_2");
        log.log(&allowed_result(), "action3", "agent_1");

        let entries = log.get_by_agent("agent_1");
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.agent_id == "agent_1"));
    }

    #[test]
    fn test_get_by_policy() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "action1", "agent_1");
        log.log(&blocked_result(), "action2", "agent_1");

        let entries = log.get_by_policy("p1");
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].result.allowed);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut log = AuditLog::new(3);
        log.log(&allowed_result(), "a1", "agent_1");
        log.log(&allowed_result(), "a2", "agent_1");
        log.log(&allowed_result(), "a3", "agent_1");
        log.log(&allowed_result(), "a4", "agent_1");

        assert_eq!(log.len(), 3);
        // Oldest entry (a1) should be evicted
        let recent = log.get_recent(10);
        assert_eq!(recent[0].action, "a2");
    }

    #[test]
    fn test_is_empty() {
        let log = AuditLog::new(100);
        assert!(log.is_empty());

        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a", "agent_1");
        assert!(!log.is_empty());
    }

    #[test]
    fn test_get_blocked() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a1", "agent_1");
        log.log(&blocked_result(), "a2", "agent_1");
        log.log(&allowed_result(), "a3", "agent_1");

        let blocked = log.get_blocked();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].action, "a2");
    }

    #[test]
    fn test_get_recent_limits_results() {
        let mut log = AuditLog::new(100);
        for i in 0..10 {
            log.log(&allowed_result(), &format!("action_{}", i), "agent_1");
        }

        let recent = log.get_recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].action, "action_7");
    }

    #[test]
    fn test_get_by_time_range() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a1", "agent_1");
        let ts = log.entries[0].timestamp;
        log.log(&allowed_result(), "a2", "agent_1");

        let entries = log.get_by_time_range(ts, ts);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "a1");
    }

    #[test]
    fn test_audit_entry_has_unique_ids() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a", "agent_1");
        log.log(&allowed_result(), "b", "agent_1");

        assert_ne!(log.entries[0].id, log.entries[1].id);
    }

    #[test]
    fn test_audit_entry_records_timestamp() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a", "agent_1");

        let entry = &log.entries[0];
        assert!(entry.timestamp > 0, "timestamp should be non-zero");
    }

    #[test]
    fn test_with_default_capacity() {
        let log = AuditLog::with_default_capacity();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_log_returns_reference_to_entry() {
        let mut log = AuditLog::new(100);
        let entry = log.log(&allowed_result(), "test_action", "agent_1");
        assert_eq!(entry.action, "test_action");
        assert_eq!(entry.agent_id, "agent_1");
    }

    #[test]
    fn test_get_recent_with_empty_log() {
        let log = AuditLog::new(100);
        let recent = log.get_recent(5);
        assert!(recent.is_empty());
    }

    #[test]
    fn test_get_by_agent_no_match() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a", "agent_1");
        let entries = log.get_by_agent("agent_2");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_by_policy_no_match() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a", "agent_1");
        let entries = log.get_by_policy("nonexistent");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_by_time_range_no_match() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a", "agent_1");
        let entries = log.get_by_time_range(0, 1);
        // Should match since timestamp is likely > 1
        // But let's test with a future range
        let entries = log.get_by_time_range(u64::MAX - 1, u64::MAX);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_blocked_empty_log() {
        let log = AuditLog::new(100);
        assert!(log.get_blocked().is_empty());
    }

    #[test]
    fn test_capacity_one_eviction() {
        let mut log = AuditLog::new(1);
        log.log(&allowed_result(), "first", "a");
        log.log(&allowed_result(), "second", "a");
        assert_eq!(log.len(), 1);
        assert_eq!(log.entries[0].action, "second");
    }

    #[test]
    fn test_multiple_agents_filtering() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a1", "agent_alpha");
        log.log(&allowed_result(), "a2", "agent_beta");
        log.log(&allowed_result(), "a3", "agent_alpha");
        log.log(&allowed_result(), "a4", "agent_gamma");
        log.log(&allowed_result(), "a5", "agent_beta");

        assert_eq!(log.get_by_agent("agent_alpha").len(), 2);
        assert_eq!(log.get_by_agent("agent_beta").len(), 2);
        assert_eq!(log.get_by_agent("agent_gamma").len(), 1);
    }

    #[test]
    fn test_entry_ids_are_sequential() {
        let mut log = AuditLog::new(100);
        log.log(&allowed_result(), "a", "agent_1");
        log.log(&allowed_result(), "b", "agent_1");
        log.log(&allowed_result(), "c", "agent_1");

        // IDs should contain sequential numbers
        assert!(log.entries[0].id.contains("0"));
        assert!(log.entries[1].id.contains("1"));
        assert!(log.entries[2].id.contains("2"));
    }
}
