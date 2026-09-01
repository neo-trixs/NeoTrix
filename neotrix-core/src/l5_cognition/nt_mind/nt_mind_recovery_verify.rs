#![allow(unused_imports)]

//! nt_mind_recovery_verify — 恢复验证与模式学习 (R-P42 吸收)
//!
//! 契约: recovery_verify / pattern_learn / known_trap_promote
//! 核心: 验证修复结果、学习成功模式、晋升重复失败为已知陷阱

use crate::l5_cognition::nt_mind::nt_mind_repair::{
    RecoveryVerifier, VerifyResult, VerificationInput, RepairHistory, FixResult,
    PatternUpdate,
};

/// 恢复验证服务 (生产接线: 由 autofixer / 背景循环调用)
pub struct RecoveryVerifyService {
    verifier: RecoveryVerifier,
    pattern_callbacks: Vec<Box<dyn Fn(&PatternUpdate) + Send + Sync>>,
    trap_callbacks: Vec<Box<dyn Fn(&str) + Send + Sync>>,
}

impl RecoveryVerifyService {
    pub fn new() -> Self {
        Self {
            verifier: RecoveryVerifier::new(),
            pattern_callbacks: Vec::new(),
            trap_callbacks: Vec::new(),
        }
    }

    /// 注册模式学习回调 (供外部消费, 如写入 KB)
    pub fn on_pattern_learned<F: Fn(&PatternUpdate) + Send + Sync + 'static>(&mut self, cb: F) {
        self.pattern_callbacks.push(Box::new(cb));
    }

    /// 注册陷阱晋升回调
    pub fn on_trap_promoted<F: Fn(&str) + Send + Sync + 'static>(&mut self, cb: F) {
        self.trap_callbacks.push(Box::new(cb));
    }

    /// 核心验证入口 (T3 生产接线)
    pub fn verify_recovery(
        &mut self,
        fix_result: &FixResult,
        verification: &VerificationInput,
        history: &RepairHistory,
    ) -> VerifyResult {
        let result = self.verifier.verify(fix_result, verification, history);

        // 触发回调
        if let Some(ref pu) = result.pattern_update {
            for cb in &self.pattern_callbacks {
                cb(pu);
            }
        }
        if let Some(ref trap) = result.known_trap_promoted {
            for cb in &self.trap_callbacks {
                cb(trap);
            }
        }

        result
    }
}

impl Default for RecoveryVerifyService {
    fn default() -> Self {
        Self::new()
    }
}

/// SelfTest
#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::nt_mind_repair::{FixResult, RepairPlan, RiskAssessment, RiskLevel};

    #[test]
    fn test_recovery_verify_service() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let mut svc = RecoveryVerifyService::new();
        let pattern_received = Arc::new(AtomicBool::new(false));
        let trap_received = Arc::new(AtomicBool::new(false));

        let pattern_received_clone = pattern_received.clone();
        svc.on_pattern_learned(move |_| {
            pattern_received_clone.store(true, Ordering::SeqCst);
        });

        let trap_received_clone = trap_received.clone();
        svc.on_trap_promoted(move |_| {
            trap_received_clone.store(true, Ordering::SeqCst);
        });

        let fix = FixResult {
            plan: RepairPlan {
                strategy: "CompileFix".into(),
                steps: vec![],
                risk: RiskAssessment { level: RiskLevel::Low, impact: "".into(), mitigations: vec![], rollback_plan: "".into() },
                rollback_point: None,
                confidence: 0.9,
                source_pattern: None,
            },
            executed_steps: vec![],
            tests_pass: true,
            health_delta: 0.5,
            artifacts_changed: vec!["src/lib.rs".into()],
        };
        let verify = VerificationInput {
            tests_pass: true,
            health: crate::l5_cognition::nt_mind::nt_mind_repair::HealthReport {
                healthy: true, dimensions: std::collections::HashMap::new(), alerts: vec![],
            },
        };
        let history = RepairHistory { same_pattern_count: 0, total_attempts: 1 };

        let result = svc.verify_recovery(&fix, &verify, &history);
        assert_eq!(result.verdict, crate::l5_cognition::nt_mind::nt_mind_repair::VerifyVerdict::Recovered);
        assert!(pattern_received.load(Ordering::SeqCst), "pattern callback should be called");
        assert!(!trap_received.load(Ordering::SeqCst), "trap callback should NOT be called for success");
    }
}