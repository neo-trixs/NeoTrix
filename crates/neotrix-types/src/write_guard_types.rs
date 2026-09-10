use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WriteGuardVerdict {
    Allow,
    Warn { reasons: Vec<String> },
    Deny { reasons: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteGuardStats {
    pub nodes_checked: usize,
    pub edges_checked: usize,
    pub verdict: WriteGuardVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteGuardEvidence {
    pub field: String,
    pub issue: String,
    pub severity: u8,
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
    fn test_verdict_deny_roundtrip() {
        let v = WriteGuardVerdict::Deny { reasons: vec!["bad".into()] };
        let json = serde_json::to_string(&v).unwrap();
        let back: WriteGuardVerdict = serde_json::from_str(&json).unwrap();
        match back {
            WriteGuardVerdict::Deny { reasons } => assert_eq!(reasons, vec!["bad"]),
            _ => panic!("expected Deny"),
        }
    }

    #[test]
    fn test_write_guard_stats_creation() {
        let stats = WriteGuardStats {
            nodes_checked: 10,
            edges_checked: 5,
            verdict: WriteGuardVerdict::Allow,
        };
        assert_eq!(stats.nodes_checked, 10);
    }
}
