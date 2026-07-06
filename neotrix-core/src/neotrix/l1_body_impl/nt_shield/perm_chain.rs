use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    Plan,
    AcceptEdits,
    BypassPermissions,
}

impl PermissionMode {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "plan" => Self::Plan,
            "accept-edits" | "accept" | "acceptEdits" => Self::AcceptEdits,
            "bypass" | "bypass-permissions" | "bypassPermissions" => Self::BypassPermissions,
            _ => Self::AcceptEdits,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Plan => "plan",
            Self::AcceptEdits => "acceptEdits",
            Self::BypassPermissions => "bypassPermissions",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionResult {
    Allowed,
    Logged(String),
    Blocked(String),
    AuditTrail(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEntry {
    pub action: String,
    pub target: String,
    pub timestamp: i64,
    pub mode: PermissionMode,
}

pub struct PermissionChain {
    mode: Mutex<PermissionMode>,
    audit_trail: Mutex<VecDeque<AuditTrailEntry>>,
    max_audit: usize,
}

impl PermissionChain {
    pub fn new(mode: PermissionMode) -> Self {
        Self {
            mode: Mutex::new(mode),
            audit_trail: Mutex::new(VecDeque::new()),
            max_audit: 1000,
        }
    }

    pub fn mode(&self) -> PermissionMode {
        *self.mode.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn set_mode(&self, mode: PermissionMode) {
        *self.mode.lock().unwrap_or_else(|e| e.into_inner()) = mode;
    }

    pub fn check(&self, action: &str, target: &str) -> PermissionResult {
        let mode = *self.mode.lock().unwrap_or_else(|e| e.into_inner());
        self.log_audit(action, target, mode);
        match mode {
            PermissionMode::Plan => PermissionResult::Logged(format!(
                "[plan] would execute: {} on {}",
                action, target
            )),
            PermissionMode::AcceptEdits => {
                if is_safe_action(action) {
                    PermissionResult::Allowed
                } else {
                    PermissionResult::Blocked(format!(
                        "[acceptEdits] risky action blocked: {} on {}",
                        action, target
                    ))
                }
            }
            PermissionMode::BypassPermissions => {
                PermissionResult::AuditTrail(format!(
                    "[bypassPermissions] executed: {} on {} (audited)",
                    action, target
                ))
            }
        }
    }

    fn log_audit(&self, action: &str, target: &str, mode: PermissionMode) {
        if let Ok(mut trail) = self.audit_trail.lock() {
            trail.push_back(AuditTrailEntry {
                action: action.to_string(),
                target: target.to_string(),
                timestamp: Utc::now().timestamp(),
                mode,
            });
            while trail.len() > self.max_audit {
                trail.pop_front();
            }
        }
    }

    pub fn audit_trail(&self) -> Vec<AuditTrailEntry> {
        self.audit_trail
            .lock()
            .map(|t| t.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn clear_audit(&self) {
        if let Ok(mut trail) = self.audit_trail.lock() {
            trail.clear();
        }
    }

    pub fn summary(&self) -> String {
        let mode = *self.mode.lock().unwrap_or_else(|e| e.into_inner());
        let count = self
            .audit_trail
            .lock()
            .map(|t| t.len())
            .unwrap_or(0);
        format!(
            "PermissionChain: mode={} audit_entries={}",
            mode.label(),
            count
        )
    }
}

fn is_safe_action(action: &str) -> bool {
    matches!(
        action,
        "file_read"
            | "read_file"
            | "list_dir"
            | "search"
            | "grep"
            | "glob"
            | "status"
            | "get_info"
            | "query"
            | "fetch"
            | "model_call"
            | "think"
            | "reason"
            | "plan"
    )
}

impl Default for PermissionChain {
    fn default() -> Self {
        Self::new(PermissionMode::AcceptEdits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_mode_logs_no_exec() {
        let chain = PermissionChain::new(PermissionMode::Plan);
        let result = chain.check("file_write", "/tmp/test.txt");
        assert!(matches!(result, PermissionResult::Logged(_)));
        let msg = match result {
            PermissionResult::Logged(s) => s,
            _ => unreachable!(),
        };
        assert!(msg.contains("[plan]"));
        assert!(msg.contains("would execute"));
    }

    #[test]
    fn test_accept_edits_allows_safe() {
        let chain = PermissionChain::new(PermissionMode::AcceptEdits);
        let result = chain.check("file_read", "/tmp/test.txt");
        assert_eq!(result, PermissionResult::Allowed);
    }

    #[test]
    fn test_accept_edits_blocks_risky() {
        let chain = PermissionChain::new(PermissionMode::AcceptEdits);
        let result = chain.check("file_write", "/etc/passwd");
        assert!(matches!(result, PermissionResult::Blocked(_)));
    }

    #[test]
    fn test_bypass_permits_all() {
        let chain = PermissionChain::new(PermissionMode::BypassPermissions);
        let result = chain.check("file_write", "/etc/passwd");
        assert!(matches!(result, PermissionResult::AuditTrail(_)));
    }

    #[test]
    fn test_audit_trail_records_bypass() {
        let chain = PermissionChain::new(PermissionMode::BypassPermissions);
        chain.check("file_write", "/etc/shadow");
        let trail = chain.audit_trail();
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].action, "file_write");
        assert_eq!(trail[0].mode, PermissionMode::BypassPermissions);
    }

    #[test]
    fn test_set_mode_switches_behavior() {
        let chain = PermissionChain::new(PermissionMode::Plan);
        assert!(matches!(chain.check("file_write", "/x"), PermissionResult::Logged(_)));
        chain.set_mode(PermissionMode::AcceptEdits);
        assert!(matches!(chain.check("file_write", "/x"), PermissionResult::Blocked(_)));
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(PermissionMode::from_str("plan"), PermissionMode::Plan);
        assert_eq!(PermissionMode::from_str("bypass"), PermissionMode::BypassPermissions);
        assert_eq!(PermissionMode::from_str("acceptEdits"), PermissionMode::AcceptEdits);
        assert_eq!(PermissionMode::from_str("unknown"), PermissionMode::AcceptEdits);
    }

    #[test]
    fn test_mode_labels() {
        assert_eq!(PermissionMode::Plan.label(), "plan");
        assert_eq!(PermissionMode::AcceptEdits.label(), "acceptEdits");
        assert_eq!(PermissionMode::BypassPermissions.label(), "bypassPermissions");
    }

    #[test]
    fn test_summary_reflects_mode() {
        let chain = PermissionChain::new(PermissionMode::BypassPermissions);
        let s = chain.summary();
        assert!(s.contains("bypassPermissions"));
    }

    #[test]
    fn test_audit_trail_max_capped() {
        let max = 5;
        let chain = PermissionChain {
            max_audit: max,
            ..Default::default()
        };
        chain.set_mode(PermissionMode::BypassPermissions);
        for i in 0..10 {
            chain.check(&format!("action_{}", i), "target");
        }
        assert_eq!(chain.audit_trail().len(), max);
    }

    #[test]
    fn test_clear_audit() {
        let chain = PermissionChain::new(PermissionMode::BypassPermissions);
        chain.check("x", "y");
        assert!(!chain.audit_trail().is_empty());
        chain.clear_audit();
        assert!(chain.audit_trail().is_empty());
    }
}
