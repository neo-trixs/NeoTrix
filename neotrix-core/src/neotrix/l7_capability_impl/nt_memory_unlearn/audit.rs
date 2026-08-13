//! nt_memory::unlearn::audit — 遗忘审计 (范围确认 + 恢复凭据)
//!
//! 节点: nt_memory::unlearn::audit (L1)
//! Provides: unlearning_audit, recovery_credentials

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use crate::neotrix::l7_capability_impl::nt_memory_unlearn::forget::ForgottenRecord;

/// 遗忘审计 — 汇总被遗忘目标, 校验每次遗忘都有恢复凭据 (reason/requested_by)
#[derive(Debug, Clone, Default)]
pub struct UnlearningAudit;

impl UnlearningAudit {
    pub fn new() -> Self {
        Self
    }

    /// 审计: 所有遗忘记录都必须可追溯 (reason + requested_by 非空)
    pub fn verify(&self, records: &[ForgottenRecord]) -> bool {
        records
            .iter()
            .all(|r| !r.reason.trim().is_empty() && !r.requested_by.trim().is_empty())
    }

    /// 按请求方聚合统计
    pub fn by_requestor(
        &self,
        records: &[ForgottenRecord],
    ) -> std::collections::HashMap<String, usize> {
        let mut m = std::collections::HashMap::new();
        for r in records {
            *m.entry(r.requested_by.clone()).or_insert(0) += 1;
        }
        m
    }
}

impl CapabilityNode for UnlearningAudit {
    fn node_id(&self) -> &str {
        "nt_memory::unlearn::audit"
    }
    fn provides(&self) -> Vec<String> {
        vec!["unlearning_audit".into(), "recovery_credentials".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["unlearning_guardrail".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Alabaster, RuneSocket::Obsidian]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for UnlearningAudit {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let a = UnlearningAudit::new();
        let good = ForgottenRecord {
            target: "t".into(),
            reason: "r".into(),
            requested_by: "u".into(),
            at: 1,
        };
        assert!(a.verify(&[good.clone()]), "完整凭据应通过");
        let bad = ForgottenRecord {
            target: "t".into(),
            reason: "".into(),
            requested_by: "u".into(),
            at: 1,
        };
        assert!(!a.verify(&[bad]), "缺 reason 应失败");
        let agg = a.by_requestor(&[good]);
        assert_eq!(agg.get("u"), Some(&1));
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_memory_unlearn_audit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(requested_by: &str) -> ForgottenRecord {
        ForgottenRecord {
            target: "t".into(),
            reason: "r".into(),
            requested_by: requested_by.into(),
            at: 1,
        }
    }

    #[test]
    fn test_verify_all_traceable() {
        let a = UnlearningAudit::new();
        assert!(a.verify(&[rec("u1"), rec("u2")]));
    }

    #[test]
    fn test_verify_rejects_missing_reason() {
        let a = UnlearningAudit::new();
        let mut bad = rec("u");
        bad.reason = "   ".into();
        assert!(!a.verify(&[bad]));
    }

    #[test]
    fn test_aggregate_by_requestor() {
        let a = UnlearningAudit::new();
        let agg = a.by_requestor(&[rec("u1"), rec("u1"), rec("u2")]);
        assert_eq!(agg.len(), 2);
        assert_eq!(agg.get("u1"), Some(&2));
        assert_eq!(agg.get("u2"), Some(&1));
    }

    #[test]
    fn test_empty_ok() {
        let a = UnlearningAudit::new();
        assert!(a.verify(&[]));
        assert!(a.by_requestor(&[]).is_empty());
    }
}
