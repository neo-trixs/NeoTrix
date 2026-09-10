use serde::{Deserialize, Serialize};

/// 写前检查裁决 — Allow 放行 / RequiresApproval 需人工 / Reject 硬阻断。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteGuardVerdict {
    Allow,
    RequiresApproval,
    Reject(Vec<String>),
}

impl WriteGuardVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, WriteGuardVerdict::Allow)
    }

    pub fn requires_approval(&self) -> bool {
        matches!(self, WriteGuardVerdict::RequiresApproval)
    }

    pub fn reasons(&self) -> Vec<String> {
        match self {
            WriteGuardVerdict::Reject(rs) => rs.clone(),
            WriteGuardVerdict::RequiresApproval => vec!["需要人工审批 (force 未置位)".into()],
            WriteGuardVerdict::Allow => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriteGuardStats {
    pub total: usize,
    pub allowed: usize,
    pub requires_approval: usize,
    pub rejected: usize,
    pub rejected_actions: std::collections::BTreeMap<String, usize>,
    pub anomalies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteGuardEvidence {
    pub key: String,
    pub action: String,
    pub verdict: WriteGuardVerdict,
    pub executed: bool,
    pub ts_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_allow_roundtrip() {
        let v = WriteGuardVerdict::Allow;
        let json = serde_json::to_string(&v).unwrap();
        let back: WriteGuardVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, WriteGuardVerdict::Allow);
    }

    #[test]
    fn test_verdict_reject_roundtrip() {
        let v = WriteGuardVerdict::Reject(vec!["bad".into()]);
        let json = serde_json::to_string(&v).unwrap();
        let back: WriteGuardVerdict = serde_json::from_str(&json).unwrap();
        match back {
            WriteGuardVerdict::Reject(rs) => assert_eq!(rs, vec!["bad"]),
            _ => panic!("expected Reject"),
        }
    }

    #[test]
    fn test_verdict_requires_approval() {
        let v = WriteGuardVerdict::RequiresApproval;
        assert!(!v.is_allowed());
        assert!(v.requires_approval());
        assert_eq!(v.reasons().len(), 1);
    }
}
