use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::nt_core_self_test::SelfTest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum _PermissionAction {
    FileWrite,
    CommandExec,
    NetworkAccess,
    ModelCall,
    FileRead,
    BrowserAutomation,
}

impl _PermissionAction {
    pub fn variants() -> Vec<_PermissionAction> {
        vec![
            _PermissionAction::FileWrite,
            _PermissionAction::CommandExec,
            _PermissionAction::NetworkAccess,
            _PermissionAction::ModelCall,
            _PermissionAction::FileRead,
            _PermissionAction::BrowserAutomation,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum _PermissionStatus {
    Pending,
    Approved(String),
    Denied(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub action: _PermissionAction,
    pub target: String,
    pub details: String,
    pub timestamp: i64,
    pub status: _PermissionStatus,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyRule {
    Allow,
    Deny,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _PolicyEntry {
    pub action: _PermissionAction,
    pub rule: PolicyRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub request_id: String,
    pub action: _PermissionAction,
    pub target: String,
    pub timestamp: i64,
    pub resolution: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum _PermissionDecision {
    Allowed,
    Denied(String),
    NeedsApproval,
}

pub struct PermissionManager {
    policies: Mutex<HashMap<_PermissionAction, PolicyRule>>,
    pending_requests: Mutex<HashMap<String, PermissionRequest>>,
    audit_log: Mutex<Vec<AuditEntry>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        policies.insert(_PermissionAction::FileWrite, PolicyRule::Ask);
        policies.insert(_PermissionAction::CommandExec, PolicyRule::Ask);
        policies.insert(_PermissionAction::NetworkAccess, PolicyRule::Ask);
        policies.insert(_PermissionAction::ModelCall, PolicyRule::Ask);
        policies.insert(_PermissionAction::FileRead, PolicyRule::Allow);
        policies.insert(_PermissionAction::BrowserAutomation, PolicyRule::Ask);

        Self {
            policies: Mutex::new(policies),
            pending_requests: Mutex::new(HashMap::new()),
            audit_log: Mutex::new(Vec::new()),
        }
    }

    pub fn check(&self, request: &PermissionRequest) -> _PermissionDecision {
        let policies = match self.policies.lock() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("[permissions] policy lock poisoned: {}", e);
                return _PermissionDecision::Denied("Internal error: lock poisoned".into());
            }
        };
        match policies.get(&request.action) {
            Some(PolicyRule::Allow) => _PermissionDecision::Allowed,
            Some(PolicyRule::Deny) => _PermissionDecision::Denied("Blocked by policy".into()),
            Some(PolicyRule::Ask) | None => _PermissionDecision::NeedsApproval,
        }
    }

    pub fn request(&self, mut req: PermissionRequest) -> PermissionRequest {
        let decision = self.check(&req);
        match decision {
            _PermissionDecision::Allowed => {
                req.status = _PermissionStatus::Approved("Auto-approved by policy".into());
            }
            _PermissionDecision::Denied(reason) => {
                req.status = _PermissionStatus::Denied(reason);
            }
            _PermissionDecision::NeedsApproval => {
                req.status = _PermissionStatus::Pending;
                if let Ok(mut pending) = self.pending_requests.lock() {
                    pending.insert(req.id.clone(), req.clone());
                }
            }
        }
        req
    }

    pub fn approve(&self, request_id: &str, reason: String) -> Result<(), String> {
        let mut pending = self.pending_requests.lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        let req = pending.remove(request_id).ok_or_else(|| format!("Request {} not found", request_id))?;
        let entry = AuditEntry {
            request_id: request_id.to_string(),
            action: req.action.clone(),
            target: req.target.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            resolution: "approved".into(),
            reason: Some(reason),
        };
        let mut audit = self.audit_log.lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        audit.push(entry);
        Ok(())
    }

    pub fn deny(&self, request_id: &str, reason: String) -> Result<(), String> {
        let mut pending = self.pending_requests.lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        let req = pending.remove(request_id).ok_or_else(|| format!("Request {} not found", request_id))?;
        let entry = AuditEntry {
            request_id: request_id.to_string(),
            action: req.action.clone(),
            target: req.target.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            resolution: "denied".into(),
            reason: Some(reason),
        };
        let mut audit = self.audit_log.lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        audit.push(entry);
        Ok(())
    }

    pub fn set_policy(&self, action: _PermissionAction, rule: PolicyRule) {
        if let Ok(mut policies) = self.policies.lock() {
            policies.insert(action, rule);
        }
    }

    pub fn _policy_summary(&self) -> Vec<_PolicyEntry> {
        let policies = match self.policies.lock() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("[permissions] policy lock poisoned: {}", e);
                return Vec::new();
            }
        };
        let mut entries: Vec<_PolicyEntry> = policies
            .iter()
            .map(|(action, rule)| _PolicyEntry {
                action: action.clone(),
                rule: rule.clone(),
            })
            .collect();
        entries.sort_by(|a, b| format!("{:?}", a.action).cmp(&format!("{:?}", b.action)));
        entries
    }

    pub fn _get_audit_log(&self, count: usize) -> Vec<AuditEntry> {
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

    pub fn _get_pending_requests(&self) -> Vec<PermissionRequest> {
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

impl SelfTest for PermissionManager {
    fn name(&self) -> &str { "permission_manager" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let summary = self._policy_summary();
        if summary.len() != 6 {
            return Err(vec![format!("expected 6 policies, got {}", summary.len())]);
        }
        let file_read = summary.iter().find(|e| e.action == _PermissionAction::FileRead);
        match file_read {
            Some(entry) if entry.rule == PolicyRule::Allow => Ok(()),
            _ => Err(vec!["FileRead should be Allow by default".into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_request(action: _PermissionAction) -> PermissionRequest {
        PermissionRequest {
            id: Uuid::new_v4().to_string(),
            action,
            target: "/tmp/test".into(),
            details: "test request".into(),
            timestamp: chrono::Utc::now().timestamp(),
            status: _PermissionStatus::Pending,
            duration: None,
        }
    }

    #[test]
    fn test_new_manager_has_default_policies() {
        let m = PermissionManager::new();
        let summary = m._policy_summary();
        assert_eq!(summary.len(), 6);
        assert!(summary.iter().any(|e| e.action == _PermissionAction::FileRead && e.rule == PolicyRule::Allow));
        assert!(summary.iter().all(|e| e.rule != PolicyRule::Deny));
    }

    #[test]
    fn test_check_allowed_action() {
        let m = PermissionManager::new();
        let req = make_request(_PermissionAction::FileRead);
        assert_eq!(m.check(&req), _PermissionDecision::Allowed);
    }

    #[test]
    fn test_check_denied_action() {
        let m = PermissionManager::new();
        m.set_policy(_PermissionAction::CommandExec, PolicyRule::Deny);
        let req = make_request(_PermissionAction::CommandExec);
        assert_eq!(m.check(&req), _PermissionDecision::Denied("Blocked by policy".into()));
    }

    #[test]
    fn test_check_needs_approval() {
        let m = PermissionManager::new();
        let req = make_request(_PermissionAction::CommandExec);
        assert_eq!(m.check(&req), _PermissionDecision::NeedsApproval);
    }

    #[test]
    fn test_request_auto_approved_for_allowed() {
        let m = PermissionManager::new();
        let req = make_request(_PermissionAction::FileRead);
        let result = m.request(req);
        assert!(matches!(result.status, _PermissionStatus::Approved(_)));
    }

    #[test]
    fn test_request_auto_denied_when_policy_denies() {
        let m = PermissionManager::new();
        m.set_policy(_PermissionAction::NetworkAccess, PolicyRule::Deny);
        let req = make_request(_PermissionAction::NetworkAccess);
        let result = m.request(req);
        assert!(matches!(result.status, _PermissionStatus::Denied(_)));
    }

    #[test]
    fn test_request_stored_as_pending_when_ask() {
        let m = PermissionManager::new();
        let req = make_request(_PermissionAction::CommandExec);
        let result = m.request(req);
        assert_eq!(result.status, _PermissionStatus::Pending);
        assert_eq!(m._get_pending_requests().len(), 1);
    }

    #[test]
    fn test_approve_request() {
        let m = PermissionManager::new();
        let req = make_request(_PermissionAction::CommandExec);
        let id = req.id.clone();
        m.request(req);
        assert!(m.approve(&id, "User approved".into()).is_ok());
        assert!(m._get_pending_requests().is_empty());
        let audit = m._get_audit_log(10);
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].resolution, "approved");
    }

    #[test]
    fn test_deny_request() {
        let m = PermissionManager::new();
        let req = make_request(_PermissionAction::CommandExec);
        let id = req.id.clone();
        m.request(req);
        assert!(m.deny(&id, "Not needed".into()).is_ok());
        assert!(m._get_pending_requests().is_empty());
        let audit = m._get_audit_log(10);
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
        m.set_policy(_PermissionAction::ModelCall, PolicyRule::Deny);
        let summary = m._policy_summary();
        let entry = summary.iter().find(|e| e.action == _PermissionAction::ModelCall).expect("ModelCall policy should exist after set_policy");
        assert_eq!(entry.rule, PolicyRule::Deny);
    }

    #[test]
    fn test_audit_log_respects_count() {
        let m = PermissionManager::new();
        for i in 0..5 {
            let req = make_request(_PermissionAction::CommandExec);
            let id = req.id.clone();
            m.request(req);
            let _ = m.approve(&id, format!("approve {}", i));
        }
        assert_eq!(m._get_audit_log(3).len(), 3);
        assert_eq!(m._get_audit_log(10).len(), 5);
    }

    #[test]
    fn test_request_generates_uuid() {
        let req = make_request(_PermissionAction::BrowserAutomation);
        assert!(Uuid::parse_str(&req.id).is_ok());
    }

    #[test]
    fn test_permission_action_variants() {
        let variants = _PermissionAction::variants();
        assert_eq!(variants.len(), 6);
        assert!(variants.contains(&_PermissionAction::FileWrite));
        assert!(variants.contains(&_PermissionAction::BrowserAutomation));
    }

    #[test]
    fn test_policy_entry_roundtrip() {
        let entry = _PolicyEntry {
            action: _PermissionAction::FileWrite,
            rule: PolicyRule::Ask,
        };
        let json = serde_json::to_string(&entry).expect("serialize _PolicyEntry");
        let deserialized: _PolicyEntry = serde_json::from_str(&json).expect("deserialize _PolicyEntry");
        assert_eq!(entry.action, deserialized.action);
        assert_eq!(entry.rule, deserialized.rule);
    }
}
