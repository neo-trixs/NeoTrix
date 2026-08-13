//! nt_memory::unlearn::forget — 选择性遗忘执行
//!
//! 节点: nt_memory::unlearn::forget (L0)
//! Provides: unlearning_guardrail, selective_forgetting

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForgetRequest {
    pub target: String,
    pub reason: String,
    pub requested_by: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForgottenRecord {
    pub target: String,
    pub reason: String,
    pub requested_by: String,
    pub at: u64,
}

/// 遗忘引擎 — 只接受带作用域+理由的遗忘请求, 被遗忘条目进入可审计记录
#[derive(Debug, Clone, Default)]
pub struct ForgetEngine {
    forgotten: Vec<ForgottenRecord>,
    next_seq: u64,
}

impl ForgetEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// 执行遗忘 (护栏: 目标非空 + 理由非空 + 请求方非空)
    pub fn forget(
        &mut self,
        req: ForgetRequest,
        now: u64,
    ) -> Result<ForgottenRecord, NeoTrixError> {
        if req.target.trim().is_empty() {
            return Err(NeoTrixError::InvalidInput("遗忘目标不能为空".into()));
        }
        if req.reason.trim().is_empty() {
            return Err(NeoTrixError::InvalidInput(
                "遗忘必须有理由 (禁止无理由清除)".into(),
            ));
        }
        if req.requested_by.trim().is_empty() {
            return Err(NeoTrixError::InvalidInput(
                "遗忘必须登记请求方 (禁止匿名清除)".into(),
            ));
        }
        self.next_seq += 1;
        let rec = ForgottenRecord {
            target: req.target,
            reason: req.reason,
            requested_by: req.requested_by,
            at: now,
        };
        self.forgotten.push(rec.clone());
        Ok(rec)
    }

    pub fn forgotten_count(&self) -> usize {
        self.forgotten.len()
    }

    pub fn records(&self) -> &[ForgottenRecord] {
        &self.forgotten
    }
}

impl CapabilityNode for ForgetEngine {
    fn node_id(&self) -> &str {
        "nt_memory::unlearn::forget"
    }
    fn provides(&self) -> Vec<String> {
        vec!["unlearning_guardrail".into(), "selective_forgetting".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec![]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Golden]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for ForgetEngine {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut f = ForgetEngine::new();
        let rec = f
            .forget(
                ForgetRequest {
                    target: "user-42-email".into(),
                    reason: "数据删除权请求".into(),
                    requested_by: "nt-shield".into(),
                },
                100,
            )
            .map_err(|e| vec![e.to_string()])?;
        assert_eq!(rec.target, "user-42-email");
        assert_eq!(f.forgotten_count(), 1);
        assert!(f
            .forget(
                ForgetRequest {
                    target: "x".into(),
                    reason: "  ".into(),
                    requested_by: "y".into()
                },
                101
            )
            .is_err());
        assert!(f
            .forget(
                ForgetRequest {
                    target: "x".into(),
                    reason: "r".into(),
                    requested_by: " ".into()
                },
                101
            )
            .is_err());
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_memory_unlearn_forget"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forget_requires_scope() {
        let mut f = ForgetEngine::new();
        assert!(f
            .forget(
                ForgetRequest {
                    target: "".into(),
                    reason: "r".into(),
                    requested_by: "u".into()
                },
                1
            )
            .is_err());
        assert!(f
            .forget(
                ForgetRequest {
                    target: "t".into(),
                    reason: "".into(),
                    requested_by: "u".into()
                },
                1
            )
            .is_err());
        assert!(f
            .forget(
                ForgetRequest {
                    target: "t".into(),
                    reason: "r".into(),
                    requested_by: "".into()
                },
                1
            )
            .is_err());
        assert_eq!(f.forgotten_count(), 0);
    }

    #[test]
    fn test_valid_forget_records() {
        let mut f = ForgetEngine::new();
        let rec = f
            .forget(
                ForgetRequest {
                    target: "namespace::key".into(),
                    reason: "隐私删除".into(),
                    requested_by: "audit".into(),
                },
                5,
            )
            .unwrap();
        assert_eq!(rec.at, 5);
        assert_eq!(f.forgotten_count(), 1);
    }

    #[test]
    fn test_multiple_records() {
        let mut f = ForgetEngine::new();
        f.forget(
            ForgetRequest {
                target: "a".into(),
                reason: "r1".into(),
                requested_by: "u".into(),
            },
            1,
        )
        .unwrap();
        f.forget(
            ForgetRequest {
                target: "b".into(),
                reason: "r2".into(),
                requested_by: "u".into(),
            },
            2,
        )
        .unwrap();
        assert_eq!(f.records().len(), 2);
    }
}
