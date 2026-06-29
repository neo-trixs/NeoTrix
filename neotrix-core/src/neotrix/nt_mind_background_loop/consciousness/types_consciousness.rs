use core::sync::atomic::AtomicBool;
use std::sync::{Mutex, OnceLock};

use crate::core::nt_core_consciousness::inner_critic::CritiqueResult;

/// Global stats snapshot written by the real brain's CI each tick,
/// read by the desktop bridge for live dashboard display.
pub static GLOBAL_CONSCIOUSNESS_STATS: OnceLock<Mutex<ExperienceStats>> = OnceLock::new();

/// Flag: has the GLOBAL_CONSCIOUSNESS_STATS been written at least once?
pub static GLOBAL_STATS_READY: AtomicBool = AtomicBool::new(false);

// ── Local stubs (still in use) ──

pub struct SarDiagnosticStub {
    cycle: u64,
}
impl SarDiagnosticStub {
    pub fn new() -> Self {
        Self { cycle: 0 }
    }
    pub fn diagnose(
        &mut self,
        vitals: crate::core::nt_core_experience::ConsciousnessVitals,
    ) -> crate::core::nt_core_experience::SarReport {
        self.cycle += 1;
        let mut signals = Vec::new();
        if vitals.coherence < 0.3 {
            signals.push("低相干性");
        }
        if vitals.arousal < 0.2 {
            signals.push("低唤醒");
        }
        if vitals.cognitive_load > 0.8 {
            signals.push("高负荷");
        }
        if vitals.negentropy_slope < -0.01 {
            signals.push("负熵下降");
        }
        if vitals.meta_accuracy < 0.5 {
            signals.push("元精度不足");
        }
        if vitals.goal_drift > 0.3 {
            signals.push("目标漂移");
        }
        let finding = if signals.is_empty() {
            "稳态运行".to_string()
        } else {
            signals.join("; ")
        };
        crate::core::nt_core_experience::SarReport {
            timestamp: self.cycle,
            setting: format!(
                "coherence={:.2}, arousal={:.2}",
                vitals.coherence, vitals.arousal
            ),
            analytical_finding: finding,
            recommendation: "继续当前周期".to_string(),
            confidence: 0.8,
        }
    }
}

pub struct ReliabilityGateStub;
impl ReliabilityGateStub {
    pub fn new() -> Self {
        Self
    }
    pub fn record_outcome(
        &mut self,
        _agent: &str,
        _outcome: crate::core::nt_core_experience::EditOutcome,
    ) {
    }
    pub fn gate_value(&self, _agent: &str) -> f64 {
        1.0
    }
    pub fn report(&self) -> crate::core::nt_core_experience::ReliabilityReport {
        crate::core::nt_core_experience::ReliabilityReport { agents: vec![] }
    }
}

// ── Core exported types ──

#[derive(Debug, Clone)]
pub struct ExperienceStats {
    pub c_score: f64,
    pub sp_coherence: f64,
    pub nm_da: f64,
    pub nm_ne: f64,
    pub nm_ht: f64,
    pub nm_ach: f64,
    pub critic_pass_rate: f64,
    pub load_mode: u64,
    pub vsa_buffer_size: usize,
    pub text_feed_total: usize,
    pub reflexivity: f64,
    pub emotion: String,
    pub critic_issued: u64,
    pub cycle: u64,
    pub last_critique: CritiqueResult,
}
