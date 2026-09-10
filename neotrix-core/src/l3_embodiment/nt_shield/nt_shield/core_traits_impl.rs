//! L3 Shield → Core Traits 实现
//!
//! 将 L3 具体类型 (Redactor, PropagationGuard, scan_absorb_text, AgentReceipt)
//! 适配为 core::nt_core_traits 抽象接口, 供 L1 消费而不产生 L1→L3 直接依赖。

use crate::core::nt_core_traits::{
    AbsorbTextScanner, AbsorbVerdict, PropagationGuardLike, ReceiptEmitter, SecretRiskLevel,
    SecretScanner,
};

use super::redaction::{Redactor, RiskLevel};
use crate::l3_embodiment::nt_shield::nt_shield_propagation_guard::PropagationGuard;
use super::self_poison;
use super::receipt::AgentReceipt;

// ── SecretScanner for Redactor ──

impl SecretScanner for Redactor {
    fn analyze(&self, text: &str) -> (SecretRiskLevel, Vec<String>) {
        let (risk, hits) = Redactor::analyze(self, text);
        let risk = match risk {
            RiskLevel::Safe => SecretRiskLevel::Safe,
            RiskLevel::Suspicious => SecretRiskLevel::Suspicious,
            RiskLevel::Dangerous => SecretRiskLevel::Dangerous,
        };
        (risk, hits)
    }

    fn is_safe(&self, text: &str) -> bool {
        Redactor::is_safe(self, text)
    }
}

// ── PropagationGuardLike for PropagationGuard ──

impl PropagationGuardLike for PropagationGuard {
    fn is_enabled(&self) -> bool {
        PropagationGuard::is_enabled(self)
    }

    fn harden_system_prompt(&self, prompt: &str) -> String {
        PropagationGuard::harden_system_prompt(self, prompt)
    }
}

// ── AbsorbTextScanner (free function wrapper) ──

pub struct SelfPoisonScanner;

impl AbsorbTextScanner for SelfPoisonScanner {
    fn scan(&self, title: &str, summary: &Option<String>, content: &Option<String>) -> AbsorbVerdict {
        let v = self_poison::scan_absorb_text(title, summary, content);
        AbsorbVerdict {
            blocked: v.blocked,
            reasons: v.reasons,
        }
    }
}

// ── ReceiptEmitter (free function wrapper) ──

pub struct AgentReceiptEmitter;

impl ReceiptEmitter for AgentReceiptEmitter {
    fn emit_receipt(&self, run_id: &str, input: &str, output: &str) -> String {
        AgentReceipt::emit(run_id, input, output).signature
    }
}
