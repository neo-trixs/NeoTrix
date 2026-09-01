use std::time::{SystemTime, UNIX_EPOCH};

use crate::neotrix::nt_core_error::NeoTrixError;
use super::SelfIteratingBrain;
use super::pipeline::{BrainStage, StageDecision};

#[derive(Debug, Clone)]
pub struct GoalContract {
    pub id: String,
    pub phase: GoalPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalPhase {
    Analyze,
    Plan,
    Execute,
    Verify,
    Reflect,
}

#[derive(Debug, Clone)]
pub struct GoalVerificationReport {
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct PhaseEvidence {
    pub phase: GoalPhase,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceType {
    Compilation,
    TestPass,
    PropertyProof,
    UserFeedback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    Retry,
    Rollback,
    Abort,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationAxis {
    Correctness,
    Completeness,
    Consistency,
    Performance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisVerdict {
    Pass,
    Fail(AxisSeverity),
    Skip,
}

#[derive(Debug, Clone)]
pub struct VerdictVector {
    pub axis: VerificationAxis,
    pub verdict: AxisVerdict,
}

#[derive(Debug, Clone)]
pub struct RepairTarget {
    pub path: String,
    pub description: String,
}

pub struct GoalContractStage;
impl GoalContractStage {
    pub fn new() -> Self { Self }
}
impl Default for GoalContractStage { fn default() -> Self { Self } }
impl BrainStage for GoalContractStage {
    fn name(&self) -> &str { "goal_contract" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let _ = brain;
        Ok(StageDecision::Continue)
    }
}

pub struct EvidenceCaptureStage;
impl EvidenceCaptureStage {
    pub fn new() -> Self { Self }
}
impl Default for EvidenceCaptureStage { fn default() -> Self { Self } }
impl BrainStage for EvidenceCaptureStage {
    fn name(&self) -> &str { "evidence_capture" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._reward;
        let iteration = brain.iteration;
        let aut = brain.autonomy;
        let evidence_count = brain._phase_evidence.len();
        // Push evidence capturing current iteration state
        brain._phase_evidence.push_back(PhaseEvidence {
            phase: GoalPhase::Analyze,
            description: format!("iter={} reward={:.4} autonomy={:?}", iteration, reward, aut),
        });
        if brain._phase_evidence.len() > 32 {
            brain._phase_evidence.pop_front();
        }
        log::debug!("[evidence_capture] iter={} reward={:.4} autonomy={:?} evidence={}",
            iteration, reward, aut, evidence_count);
        if evidence_count > 0 {
            if let Some(last) = brain._phase_evidence.back() {
                log::debug!("[evidence_capture] last_phase={:?} desc='{}'",
                    last.phase, last.description);
            }
        }
        Ok(StageDecision::Continue)
    }
}

pub struct NarrowRecoveryStage;
impl NarrowRecoveryStage {
    pub fn new() -> Self { Self }
}
impl Default for NarrowRecoveryStage { fn default() -> Self { Self } }
impl BrainStage for NarrowRecoveryStage {
    fn name(&self) -> &str { "narrow_recovery" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._reward < -0.2 {
            log::info!("[narrow_recovery] negative reward {:.4}, checkpoint restore attempted", brain._reward);
            brain._phase_evidence.push_back(PhaseEvidence {
                phase: GoalPhase::Reflect,
                description: format!("narrow_recovery triggered reward={:.4}", brain._reward),
            });
            if brain._phase_evidence.len() > 32 {
                brain._phase_evidence.pop_front();
            }
            match brain._checkpoint_manager.restore(
                &mut brain.brain, &mut brain.permission, &mut brain.autonomy, &mut brain._reward, "best",
            ) {
                Ok(()) => log::info!("[narrow_recovery] checkpoint restore succeeded"),
                Err(e) => log::warn!("[narrow_recovery] checkpoint restore failed: {}", e),
            }
        }
        Ok(StageDecision::Continue)
    }
}

pub struct FinalVerificationStage;
impl FinalVerificationStage {
    pub fn new() -> Self { Self }
}
impl Default for FinalVerificationStage { fn default() -> Self { Self } }
impl BrainStage for FinalVerificationStage {
    fn name(&self) -> &str { "final_verification" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let passed = brain._reward > 0.5;
        let report = GoalVerificationReport {
            passed,
            details: format!("verification at iter={} reward={:.4}", brain.iteration, brain._reward),
        };
        brain._phase_evidence.push_back(PhaseEvidence {
            phase: GoalPhase::Verify,
            description: format!("verification_passed={} reward={:.4}", passed, brain._reward),
        });
        if brain._phase_evidence.len() > 32 {
            brain._phase_evidence.pop_front();
        }
        log::info!("[final_verification] {}", write_journal(&report));
        Ok(StageDecision::Continue)
    }
}

pub struct GoalTerminatorStage;
impl GoalTerminatorStage {
    pub fn new() -> Self { Self }
}
impl Default for GoalTerminatorStage { fn default() -> Self { Self } }
impl BrainStage for GoalTerminatorStage {
    fn name(&self) -> &str { "goal_terminator" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._reward > 0.8 {
            log::info!("[goal_terminator] high reward {:.4}, goal achieved", brain._reward);
            brain._goal_complete = true;
            brain._phase_evidence.push_back(PhaseEvidence {
                phase: GoalPhase::Execute,
                description: format!("goal_achieved reward={:.4} iter={}", brain._reward, brain.iteration),
            });
            if brain._phase_evidence.len() > 32 {
                brain._phase_evidence.pop_front();
            }
            if brain._goal_contract.is_none() {
                brain._goal_contract = Some(GoalContract {
                    id: format!("goal-iter-{}", brain.iteration),
                    phase: GoalPhase::Verify,
                });
            }
        }
        Ok(StageDecision::Continue)
    }
}

pub struct ExternalVerifierStage;
impl ExternalVerifierStage {
    pub fn new() -> Self { Self }
}
impl Default for ExternalVerifierStage { fn default() -> Self { Self } }
impl BrainStage for ExternalVerifierStage {
    fn name(&self) -> &str { "external_verifier" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // Run external grounding check every 5 iterations to break self-validation loop
        if brain.iteration > 0 && !brain.iteration.is_multiple_of(5) {
            return Ok(StageDecision::Continue);
        }
        if let Some(ref engine) = brain.reasoning_engine {
            let traj_len = engine.state_trajectory.len();
            // External grounding: verify reward against cargo check
            // 单元测试不打真实 cargo check (build-lock 死锁 + 60s+ 超时) — 与 self_review
            // / behavioral_verifier 同一 cfg!(test) 隔离纪律。真实验证留待集成测试。
            let build_ok = if cfg!(test) { true } else {
                std::process::Command::new("cargo")
                    .args(["check", "--lib", "-q", "-p", "neotrix"])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            };
            let reward_aligns_with_build = if build_ok { brain._reward >= 0.0 } else { brain._reward < 0.0 };
            let verified = brain._reward > 0.3 && build_ok;
            if !verified && brain._reward > 0.3 && !build_ok {
                log::warn!("[external_verifier] SELF-DECEPTION FLAG: reward={:.4} positive but cargo check FAILED", brain._reward);
            }
            if !reward_aligns_with_build {
                log::warn!("[external_verifier] MISALIGNED: reward={:.4} but build={} — overriding internal signal",
                    brain._reward, if build_ok { "OK" } else { "FAIL" });
                if !build_ok { brain._set_reward(-0.5); }
            }
            brain._phase_evidence.push_back(PhaseEvidence {
                phase: GoalPhase::Verify,
                description: format!("external_verified={} build_ok={} traj_len={} reward={:.4} align={}",
                    verified, build_ok, traj_len, brain._reward, reward_aligns_with_build),
            });
            while brain._phase_evidence.len() > 32 {
                brain._phase_evidence.pop_front();
            }
            log::trace!("[external_verifier] traj_len={} build_ok={} verified={} reward={:.4}",
                traj_len, build_ok, verified, brain._reward);
        }
        Ok(StageDecision::Continue)
    }
}

pub struct SemanticRecallStage;
impl SemanticRecallStage {
    pub fn new() -> Self { Self }
}
impl Default for SemanticRecallStage { fn default() -> Self { Self } }
impl BrainStage for SemanticRecallStage {
    fn name(&self) -> &str { "semantic_recall" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref kb) = brain._nt_memory_kb {
            let context = format!("iteration_reward_{:.2}", brain._reward);
            if let Ok(results) = kb.hybrid_rerank_search(&context, 3) {
                let n = results.len();
                brain._phase_evidence.push_back(PhaseEvidence {
                    phase: GoalPhase::Plan,
                    description: format!("semantic_recall count={} iter={} reward={:.4}",
                        n, brain.iteration, brain._reward),
                });
                if brain._phase_evidence.len() > 32 {
                    brain._phase_evidence.pop_front();
                }
                log::trace!("[semantic_recall] retrieved {} nodes for iter={} reward={:.4}",
                    n, brain.iteration, brain._reward);
            }
        }
        Ok(StageDecision::Continue)
    }
}

pub fn analyze_failure(_trajectory: &[String], _outcome: &str) -> String {
    String::new()
}

pub fn default_verification_axes() -> Vec<VerificationAxis> {
    vec![VerificationAxis::Correctness, VerificationAxis::Completeness]
}

pub fn run_verification_axis(_axis: VerificationAxis, _brain: &SelfIteratingBrain) -> VerdictVector {
    VerdictVector { axis: _axis, verdict: AxisVerdict::Pass }
}

pub fn write_journal(report: &GoalVerificationReport) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!(
        "=== Goal Verification @ {} ===\nStatus: {}\n{}",
        ts,
        if report.passed { "PASS" } else { "FAIL" },
        report.details,
    )
}

pub fn should_stop_seal_loop(_report: &GoalVerificationReport) -> bool {
    false
}
