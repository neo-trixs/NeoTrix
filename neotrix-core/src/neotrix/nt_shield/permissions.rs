use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

const MAX_AUDIT_LOG: usize = 1_000;
const PENDING_TTL_SECS: i64 = 3600; // 1 hour

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PermissionAction {
    FileWrite,
    CommandExec,
    NetworkAccess,
    ModelCall,
    FileRead,
    BrowserAutomation,
}

impl PermissionAction {
    pub fn variants() -> Vec<PermissionAction> {
        vec![
            PermissionAction::FileWrite,
            PermissionAction::CommandExec,
            PermissionAction::NetworkAccess,
            PermissionAction::ModelCall,
            PermissionAction::FileRead,
            PermissionAction::BrowserAutomation,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionStatus {
    Pending,
    Approved(String),
    Denied(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub action: PermissionAction,
    pub target: String,
    pub details: String,
    pub timestamp: i64,
    pub status: PermissionStatus,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyRule {
    Allow,
    Deny,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEntry {
    pub action: PermissionAction,
    pub rule: PolicyRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub request_id: String,
    pub action: PermissionAction,
    pub target: String,
    pub timestamp: i64,
    pub resolution: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionDecision {
    Allowed,
    Denied(String),
    NeedsApproval,
}

pub struct PermissionManager {
    policies: Mutex<HashMap<PermissionAction, PolicyRule>>,
    pending_requests: Mutex<HashMap<String, PermissionRequest>>,
    audit_log: Mutex<Vec<AuditEntry>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        policies.insert(PermissionAction::FileWrite, PolicyRule::Ask);
        policies.insert(PermissionAction::CommandExec, PolicyRule::Ask);
        policies.insert(PermissionAction::NetworkAccess, PolicyRule::Ask);
        policies.insert(PermissionAction::ModelCall, PolicyRule::Ask);
        policies.insert(PermissionAction::FileRead, PolicyRule::Allow);
        policies.insert(PermissionAction::BrowserAutomation, PolicyRule::Ask);

        Self {
            policies: Mutex::new(policies),
            pending_requests: Mutex::new(HashMap::new()),
            audit_log: Mutex::new(Vec::new()),
        }
    }

    pub fn check(&self, request: &PermissionRequest) -> PermissionDecision {
        let policies = match self.policies.lock() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("[permissions] policy lock poisoned: {}", e);
                return PermissionDecision::Denied("Internal error: lock poisoned".into());
            }
        };
        match policies.get(&request.action) {
            Some(PolicyRule::Allow) => PermissionDecision::Allowed,
            Some(PolicyRule::Deny) => PermissionDecision::Denied("Blocked by policy".into()),
            Some(PolicyRule::Ask) | None => PermissionDecision::NeedsApproval,
        }
    }

    pub fn request(&self, mut req: PermissionRequest) -> PermissionRequest {
        let decision = self.check(&req);
        match decision {
            PermissionDecision::Allowed => {
                req.status = PermissionStatus::Approved("Auto-approved by policy".into());
            }
            PermissionDecision::Denied(reason) => {
                req.status = PermissionStatus::Denied(reason);
            }
            PermissionDecision::NeedsApproval => {
                req.status = PermissionStatus::Pending;
                if let Ok(mut pending) = self.pending_requests.lock() {
                    pending.insert(req.id.clone(), req.clone());
                }
            }
        }
        req
    }

    pub fn approve(&self, request_id: &str, reason: String) -> Result<(), String> {
        let mut pending = self
            .pending_requests
            .lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        let req = pending
            .remove(request_id)
            .ok_or_else(|| format!("Request {} not found", request_id))?;
        let entry = AuditEntry {
            request_id: request_id.to_string(),
            action: req.action.clone(),
            target: req.target.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            resolution: "approved".into(),
            reason: Some(reason),
        };
        let mut audit = self
            .audit_log
            .lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        audit.push(entry);
        if audit.len() > MAX_AUDIT_LOG {
            audit.remove(0);
        }
        Ok(())
    }

    pub fn deny(&self, request_id: &str, reason: String) -> Result<(), String> {
        let mut pending = self
            .pending_requests
            .lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        let req = pending
            .remove(request_id)
            .ok_or_else(|| format!("Request {} not found", request_id))?;
        let entry = AuditEntry {
            request_id: request_id.to_string(),
            action: req.action.clone(),
            target: req.target.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            resolution: "denied".into(),
            reason: Some(reason),
        };
        let mut audit = self
            .audit_log
            .lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        audit.push(entry);
        if audit.len() > MAX_AUDIT_LOG {
            audit.remove(0);
        }
        Ok(())
    }
}

impl PermissionManager {
    pub fn set_policy(&self, action: PermissionAction, rule: PolicyRule) {
        if let Ok(mut policies) = self.policies.lock() {
            policies.insert(action, rule);
        }
    }

    pub fn policy_summary(&self) -> Vec<PolicyEntry> {
        let policies = match self.policies.lock() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("[permissions] policy lock poisoned: {}", e);
                return Vec::new();
            }
        };
        let mut entries: Vec<PolicyEntry> = policies
            .iter()
            .map(|(action, rule)| PolicyEntry {
                action: action.clone(),
                rule: rule.clone(),
            })
            .collect();
        entries.sort_by(|a, b| format!("{:?}", a.action).cmp(&format!("{:?}", b.action)));
        entries
    }

    pub fn evict_stale_pending(&self) {
        if let Ok(mut pending) = self.pending_requests.lock() {
            let cutoff = chrono::Utc::now().timestamp() - PENDING_TTL_SECS;
            pending.retain(|_, req| req.timestamp >= cutoff);
        }
    }

    pub fn get_audit_log(&self, count: usize) -> Vec<AuditEntry> {
        let audit = match self.audit_log.lock() {
            Ok(a) => a,
            Err(e) => {
                log::warn!("[permissions] audit log lock poisoned: {}", e);
                return Vec::new();
            }
        };
        let len = audit.len();
        let start = len.saturating_sub(count);
        audit[start..].to_vec()
    }

    pub fn get_pending_requests(&self) -> Vec<PermissionRequest> {
        self.evict_stale_pending();
        let pending = match self.pending_requests.lock() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("[permissions] pending_requests lock poisoned: {}", e);
                return Vec::new();
            }
        };
        pending.values().cloned().collect()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_request(action: PermissionAction) -> PermissionRequest {
        PermissionRequest {
            id: Uuid::new_v4().to_string(),
            action,
            target: "/tmp/test".into(),
            details: "test request".into(),
            timestamp: chrono::Utc::now().timestamp(),
            status: PermissionStatus::Pending,
            duration: None,
        }
    }

    #[test]
    fn test_new_manager_has_default_policies() {
        let m = PermissionManager::new();
        let summary = m.policy_summary();
        assert_eq!(summary.len(), 6);
        assert!(summary
            .iter()
            .any(|e| e.action == PermissionAction::FileRead && e.rule == PolicyRule::Allow));
        assert!(summary.iter().all(|e| e.rule != PolicyRule::Deny));
    }

    #[test]
    fn test_check_allowed_action() {
        let m = PermissionManager::new();
        let req = make_request(PermissionAction::FileRead);
        assert_eq!(m.check(&req), PermissionDecision::Allowed);
    }

    #[test]
    fn test_check_denied_action() {
        let m = PermissionManager::new();
        m.set_policy(PermissionAction::CommandExec, PolicyRule::Deny);
        let req = make_request(PermissionAction::CommandExec);
        assert_eq!(
            m.check(&req),
            PermissionDecision::Denied("Blocked by policy".into())
        );
    }

    #[test]
    fn test_check_needs_approval() {
        let m = PermissionManager::new();
        let req = make_request(PermissionAction::CommandExec);
        assert_eq!(m.check(&req), PermissionDecision::NeedsApproval);
    }

    #[test]
    fn test_request_auto_approved_for_allowed() {
        let m = PermissionManager::new();
        let req = make_request(PermissionAction::FileRead);
        let result = m.request(req);
        assert!(matches!(result.status, PermissionStatus::Approved(_)));
    }

    #[test]
    fn test_request_auto_denied_when_policy_denies() {
        let m = PermissionManager::new();
        m.set_policy(PermissionAction::NetworkAccess, PolicyRule::Deny);
        let req = make_request(PermissionAction::NetworkAccess);
        let result = m.request(req);
        assert!(matches!(result.status, PermissionStatus::Denied(_)));
    }

    #[test]
    fn test_request_stored_as_pending_when_ask() {
        let m = PermissionManager::new();
        let req = make_request(PermissionAction::CommandExec);
        let result = m.request(req);
        assert_eq!(result.status, PermissionStatus::Pending);
        assert_eq!(m.get_pending_requests().len(), 1);
    }

    #[test]
    fn test_approve_request() {
        let m = PermissionManager::new();
        let req = make_request(PermissionAction::CommandExec);
        let id = req.id.clone();
        m.request(req);
        assert!(m.approve(&id, "User approved".into()).is_ok());
        assert!(m.get_pending_requests().is_empty());
        let audit = m.get_audit_log(10);
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].resolution, "approved");
    }

    #[test]
    fn test_deny_request() {
        let m = PermissionManager::new();
        let req = make_request(PermissionAction::CommandExec);
        let id = req.id.clone();
        m.request(req);
        assert!(m.deny(&id, "Not needed".into()).is_ok());
        assert!(m.get_pending_requests().is_empty());
        let audit = m.get_audit_log(10);
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].resolution, "denied");
    }

    #[test]
    fn test_approve_nonexistent_request_fails() {
        let m = PermissionManager::new();
        let result = m.approve("nonexistent-id", "reason".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_set_policy_and_verify() {
        let m = PermissionManager::new();
        m.set_policy(PermissionAction::ModelCall, PolicyRule::Deny);
        let summary = m.policy_summary();
        let entry = summary
            .iter()
            .find(|e| e.action == PermissionAction::ModelCall)
            .expect("ModelCall policy should exist after set_policy");
        assert_eq!(entry.rule, PolicyRule::Deny);
    }

    #[test]
    fn test_audit_log_respects_count() {
        let m = PermissionManager::new();
        for i in 0..5 {
            let req = make_request(PermissionAction::CommandExec);
            let id = req.id.clone();
            m.request(req);
            let _ = m.approve(&id, format!("approve {}", i));
        }
        assert_eq!(m.get_audit_log(3).len(), 3);
        assert_eq!(m.get_audit_log(10).len(), 5);
    }

    #[test]
    fn test_request_generates_uuid() {
        let req = make_request(PermissionAction::BrowserAutomation);
        assert!(Uuid::parse_str(&req.id).is_ok());
    }

    #[test]
    fn test_permission_action_variants() {
        let variants = PermissionAction::variants();
        assert_eq!(variants.len(), 6);
        assert!(variants.contains(&PermissionAction::FileWrite));
        assert!(variants.contains(&PermissionAction::BrowserAutomation));
    }

    #[test]
    fn test_policy_entry_roundtrip() {
        let entry = PolicyEntry {
            action: PermissionAction::FileWrite,
            rule: PolicyRule::Ask,
        };
        let json = serde_json::to_string(&entry).expect("serialize PolicyEntry");
        let deserialized: PolicyEntry =
            serde_json::from_str(&json).expect("deserialize PolicyEntry");
        assert_eq!(entry.action, deserialized.action);
        assert_eq!(entry.rule, deserialized.rule);
    }
}
