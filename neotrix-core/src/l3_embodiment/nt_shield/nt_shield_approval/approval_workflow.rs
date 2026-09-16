use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub id: String,
    pub action: String,
    pub requester: String,
    pub reason: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub status: ApprovalStatus,
}

#[derive(Debug, Clone)]
pub struct ApprovalRule {
    pub action_pattern: String,
    pub auto_approve: bool,
    pub max_risk_level: u8,
}

pub struct ApprovalWorkflow {
    requests: HashMap<String, ApprovalRequest>,
    rules: Vec<ApprovalRule>,
}

impl ApprovalWorkflow {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: ApprovalRule) {
        self.rules.push(rule);
    }

    pub fn request_approval(
        &mut self,
        id: &str,
        action: &str,
        requester: &str,
        reason: &str,
    ) -> ApprovalStatus {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let auto = self
            .rules
            .iter()
            .any(|r| action.contains(&r.action_pattern) && r.auto_approve);
        let status = if auto {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Pending
        };
        self.requests.insert(
            id.to_string(),
            ApprovalRequest {
                id: id.to_string(),
                action: action.to_string(),
                requester: requester.to_string(),
                reason: reason.to_string(),
                created_at: now,
                expires_at: None,
                status: status.clone(),
            },
        );
        status
    }

    pub fn approve(&mut self, id: &str) -> bool {
        if let Some(r) = self.requests.get_mut(id) {
            r.status = ApprovalStatus::Approved;
            true
        } else {
            false
        }
    }

    pub fn reject(&mut self, id: &str) -> bool {
        if let Some(r) = self.requests.get_mut(id) {
            r.status = ApprovalStatus::Rejected;
            true
        } else {
            false
        }
    }

    pub fn status(&self, id: &str) -> Option<&ApprovalStatus> {
        self.requests.get(id).map(|r| &r.status)
    }

    pub fn pending_count(&self) -> usize {
        self.requests
            .values()
            .filter(|r| r.status == ApprovalStatus::Pending)
            .count()
    }
}

impl Default for ApprovalWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_approve() {
        let mut w = ApprovalWorkflow::new();
        w.add_rule(ApprovalRule {
            action_pattern: "read".into(),
            auto_approve: true,
            max_risk_level: 3,
        });
        let s = w.request_approval("r1", "read file", "agent", "need data");
        assert_eq!(s, ApprovalStatus::Approved);
    }

    #[test]
    fn test_manual_approve() {
        let mut w = ApprovalWorkflow::new();
        let s = w.request_approval("r2", "delete file", "agent", "cleanup");
        assert_eq!(s, ApprovalStatus::Pending);
        assert!(w.approve("r2"));
        assert_eq!(w.status("r2"), Some(&ApprovalStatus::Approved));
    }

    #[test]
    fn test_reject() {
        let mut w = ApprovalWorkflow::new();
        w.request_approval("r3", "execute", "agent", "");
        assert!(w.reject("r3"));
        assert_eq!(w.status("r3"), Some(&ApprovalStatus::Rejected));
    }
}
