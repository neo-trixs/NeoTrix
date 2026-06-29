use super::super::pipeline::{BrainStage, StageDecision};
use super::types::*;
use super::write_journal;
use super::SelfIteratingBrain;
use crate::core::MicroEdit;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_world_journal_index::JournalIndex;
use std::collections::VecDeque;

/// Analyze micro-edits and stage results to determine root cause of failure
pub fn analyze_failure(
    _brain: &SelfIteratingBrain,
    failed_stage: &str,
    reward_delta: f64,
    edits: &[MicroEdit],
) -> RecoveryAction {
    if edits.is_empty() && reward_delta < -0.1 {
        return RecoveryAction::RetryStage;
    }
    if reward_delta < -0.3 {
        let dim = edits
            .last()
            .and_then(|e: &MicroEdit| e.dimension_name().map(String::from))
            .unwrap_or_else(|| "unknown".into());
        return RecoveryAction::NarrowEdit {
            dimension: dim,
            description: format!(
                "阶段 '{}' 导致奖励下降 {:.2}，回滚最近编辑",
                failed_stage, reward_delta
            ),
        };
    }
    RecoveryAction::RetryStage
}

/// Stage: initialize or advance the goal contract on task boundaries
pub struct GoalContractStage;

impl Default for GoalContractStage {
    fn default() -> Self {
        Self
    }
}
impl GoalContractStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for GoalContractStage {
    fn name(&self) -> &str {
        "goal_contract"
    }
    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.task_scratch.current_task.is_empty() {
            return Ok(StageDecision::Skip("no active task".into()));
        }

        if let Some(ref contract) = brain.goal_state.goal_contract {
            if contract.completed || contract.all_phases_done() {
                return Ok(StageDecision::Skip(
                    "goal contract already completed".into(),
                ));
            }
            if contract.phases.is_empty() {
                return Ok(StageDecision::Skip("empty goal contract".into()));
            }
            if let Some(current) = contract.current_phase() {
                if current.verified {
                    let mut c = match brain.goal_state.goal_contract.take() {
                        Some(c) => c,
                        None => {
                            log::warn!("goal_contract: missing contract");
                            return Ok(StageDecision::Skip("missing contract".into()));
                        }
                    };
                    c.advance_phase();
                    let phase_name = c
                        .phases
                        .get(c.current_phase)
                        .map(|p| p.id.clone())
                        .unwrap_or_else(|| "complete".into());
                    log::info!("[goal_contract] advancing to phase: {}", phase_name);
                    brain.goal_state.goal_contract = Some(c);
                }
            }
        } else {
            let contract = GoalContract::decompose(
                &brain.task_scratch.current_task,
                brain.task_scratch.current_task_type,
            );
            log::info!(
                "[goal_contract] decomposed '{}' into {} phases: {:?}",
                brain.task_scratch.current_task,
                contract.phases.len(),
                contract
                    .phases
                    .iter()
                    .map(|p| p.id.as_str())
                    .collect::<Vec<_>>()
            );
            brain.goal_state.goal_contract = Some(contract);
            brain.goal_state.phase_evidence = VecDeque::with_capacity(32);
        }

        Ok(StageDecision::Continue)
    }
}

/// Stage: capture concrete evidence after work stages execute
pub struct EvidenceCaptureStage;

impl Default for EvidenceCaptureStage {
    fn default() -> Self {
        Self
    }
}
impl EvidenceCaptureStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for EvidenceCaptureStage {
    fn name(&self) -> &str {
        "evidence_capture"
    }
    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let contract = match brain.goal_state.goal_contract.as_ref() {
            Some(c) if !c.phases.is_empty() => c,
            _ => return Ok(StageDecision::Continue),
        };

        let current_phase_id = contract
            .phases
            .get(contract.current_phase)
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "unknown".into());

        let evidence_id = format!("ev-{}-{}", current_phase_id, brain.iteration);

        let reward_delta = brain.task_scratch.reward;
        let edit_count = brain.task_scratch.micro_edits.len();
        let tool_count = brain.tool_call_count;
        let has_edits = edit_count > 0;

        let evidence = PhaseEvidence {
            id: evidence_id.clone(),
            phase_id: current_phase_id.clone(),
            stage_name: "evidence_capture".into(),
            evidence_type: EvidenceType::RewardDelta(reward_delta),
            description: format!(
                "阶段 '{}' 迭代 {}: 奖励 {:.3}, 编辑 {} 个, 工具调用 {} 次",
                current_phase_id, brain.iteration, reward_delta, edit_count, tool_count
            ),
            success: reward_delta >= 0.0,
            details: format!(
                "reward={:.3} edits={} tools={}",
                reward_delta, edit_count, tool_count
            ),
            iteration: brain.iteration,
        };

        brain.goal_state.phase_evidence.push_back(evidence);

        if has_edits && reward_delta >= 0.0 {
            if let Some(ref mut contract) = brain.goal_state.goal_contract {
                if let Some(phase) = contract.phases.get_mut(contract.current_phase) {
                    if !phase.evidence_ids.contains(&evidence_id) {
                        phase.evidence_ids.push(evidence_id);
                    }
                }
            }
        }

        Ok(StageDecision::Continue)
    }
}

/// Stage: narrow recovery on failure — analyze root cause and apply minimal fix
pub struct NarrowRecoveryStage;

impl Default for NarrowRecoveryStage {
    fn default() -> Self {
        Self
    }
}
impl NarrowRecoveryStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for NarrowRecoveryStage {
    fn name(&self) -> &str {
        "narrow_recovery"
    }
    fn frequency(&self) -> usize {
        3
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.task_scratch.reward >= -0.05 {
            return Ok(StageDecision::Continue);
        }
        if brain.seal_rl.stage_results.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let worst_stage = brain
            .seal_rl
            .stage_results
            .iter()
            .min_by(|a, b| {
                a.efc
                    .partial_cmp(&b.efc)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.stage_name.clone())
            .unwrap_or_else(|| "unknown".into());

        let recovery = analyze_failure(
            brain,
            &worst_stage,
            brain.task_scratch.reward,
            &brain.task_scratch.micro_edits,
        );

        log::warn!(
            "[narrow_recovery] reward={:.3}, worst_stage={}, action={}",
            brain.task_scratch.reward,
            worst_stage,
            recovery.description()
        );

        match recovery {
            RecoveryAction::NarrowEdit { .. } => Ok(StageDecision::Skip(format!(
                "narrow_recovery: {}",
                recovery.description()
            ))),
            RecoveryAction::RetryStage => Ok(StageDecision::Continue),
            RecoveryAction::SkipPhase {
                phase_id: _,
                reason,
            } => {
                if let Some(ref mut contract) = brain.goal_state.goal_contract {
                    contract.advance_phase();
                }
                Ok(StageDecision::Skip(reason))
            }
            RecoveryAction::RollbackToCheckpoint { checkpoint_id } => {
                brain
                    .checkpoint_manager
                    .restore(
                        &mut brain.brain,
                        &mut brain.permission,
                        &mut brain.autonomy,
                        &mut brain.task_scratch.reward,
                        &checkpoint_id,
                    )
                    .ok();
                Ok(StageDecision::Skip(format!(
                    "rolled back to {}",
                    checkpoint_id
                )))
            }
            RecoveryAction::AbortGoal { reason } => {
                brain.goal_state.goal_contract = None;
                brain.goal_state.phase_evidence.clear();
                Ok(StageDecision::Skip(format!("aborted: {}", reason)))
            }
        }
    }
}

/// Stage: final verification — map evidence against original goal contract
pub struct FinalVerificationStage;

impl Default for FinalVerificationStage {
    fn default() -> Self {
        Self
    }
}
impl FinalVerificationStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for FinalVerificationStage {
    fn name(&self) -> &str {
        "final_verification"
    }
    fn frequency(&self) -> usize {
        5
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let contract = match brain.goal_state.goal_contract.as_ref() {
            Some(c) if c.all_phases_done() || brain.task_scratch.reward > 0.5 => c.clone(),
            _ => return Ok(StageDecision::Continue),
        };

        let report = contract.verification_report();
        let phase_desc = contract.progress_description();

        let evidence_count = brain.goal_state.phase_evidence.len();
        let edit_count = brain.task_scratch.micro_edits.len();
        let tool_count = brain.tool_call_count;

        log::info!(
            "[final_verification] {} | 证据: {}, 编辑: {}, 工具: {} | 标准: {}/{} 通过",
            phase_desc,
            evidence_count,
            edit_count,
            tool_count,
            report.met_criteria,
            report.total_criteria
        );

        if report.overall_success {
            log::info!(
                "[final_verification] ✅ 目标完成: {}",
                contract.original_goal
            );
        } else {
            log::warn!(
                "[final_verification] ⚠️ 目标未完成: {} | 未完成阶段: {:?}",
                contract.original_goal,
                report.unmet_phases
            );
        }

        if let Some(ref mut c) = brain.goal_state.goal_contract {
            c.final_verification = Some(report);
            c.completed = true;
        }

        Ok(StageDecision::Continue)
    }
}

/// Stage: set _goal_complete flag when all phases are verified.
/// This signals the outer SEAL loop to stop iterating — /goal semantics.
pub struct GoalTerminatorStage;

impl Default for GoalTerminatorStage {
    fn default() -> Self {
        Self
    }
}
impl GoalTerminatorStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for GoalTerminatorStage {
    fn name(&self) -> &str {
        "goal_terminator"
    }
    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let contract = match brain.goal_state.goal_contract.as_ref() {
            Some(c) if !c.completed => c,
            _ => return Ok(StageDecision::Continue),
        };

        if contract.all_phases_done() {
            let report = contract.verification_report();
            if report.overall_success {
                brain.goal_state.goal_complete = true;
                log::info!(
                    "[goal_terminator] ✅ 目标完成，循环终止: {}",
                    contract.original_goal
                );
                if let Some(ref mut c) = brain.goal_state.goal_contract {
                    c.final_verification = Some(report);
                    c.completed = true;
                }
                if let Err(e) = write_journal(brain) {
                    log::warn!("[goal_terminator] journal write failed: {}", e);
                }
            }
        }

        Ok(StageDecision::Continue)
    }
}

/// Default verification axes for Rust projects
pub fn default_verification_axes(workspace: &str) -> Vec<VerificationAxis> {
    vec![
        VerificationAxis {
            name: "compile".into(),
            command: vec!["cargo".into(), "check".into(), "--quiet".into()],
            current_dir: Some(workspace.into()),
            severity: AxisSeverity::Critical,
            repair: RepairTarget::AutoFix(vec!["cargo".into(), "check".into()]),
        },
        VerificationAxis {
            name: "tests".into(),
            command: vec!["cargo".into(), "test".into(), "--quiet".into()],
            current_dir: Some(workspace.into()),
            severity: AxisSeverity::High,
            repair: RepairTarget::Retry,
        },
        VerificationAxis {
            name: "lint".into(),
            command: vec!["cargo".into(), "clippy".into(), "--quiet".into()],
            current_dir: Some(workspace.into()),
            severity: AxisSeverity::Medium,
            repair: RepairTarget::LogOnly,
        },
        VerificationAxis {
            name: "format".into(),
            command: vec![
                "cargo".into(),
                "fmt".into(),
                "--check".into(),
                "--quiet".into(),
            ],
            current_dir: Some(workspace.into()),
            severity: AxisSeverity::Low,
            repair: RepairTarget::AutoFix(vec!["cargo".into(), "fmt".into()]),
        },
    ]
}

/// Run a single verification axis and return its verdict
pub fn run_verification_axis(axis: &VerificationAxis, workspace: &str) -> AxisVerdict {
    let _cmd = if axis.command.is_empty() {
        return AxisVerdict {
            axis_name: axis.name.clone(),
            passed: true,
            details: "no command".into(),
            exit_code: None,
            severity: axis.severity,
        };
    };

    let program = &axis.command[0];
    let args: Vec<&str> = axis.command[1..].iter().map(|s| s.as_str()).collect();
    let dir = axis.current_dir.as_deref().unwrap_or(workspace);

    match std::process::Command::new(program)
        .args(&args)
        .current_dir(dir)
        .output()
    {
        Ok(output) => {
            let passed = output.status.success();
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if passed {
                "ok".into()
            } else {
                let lines: Vec<&str> = stderr.lines().chain(stdout.lines()).take(5).collect();
                lines.join("\n")
            };
            AxisVerdict {
                axis_name: axis.name.clone(),
                passed,
                details: detail,
                exit_code: output.status.code(),
                severity: axis.severity,
            }
        }
        Err(e) => AxisVerdict {
            axis_name: axis.name.clone(),
            passed: false,
            details: format!("cannot execute: {}", e),
            exit_code: None,
            severity: axis.severity,
        },
    }
}

/// Stage: run a vector of independent external checks (compile, test, lint, format).
/// Each axis is scored independently. Failed axes route to their own repair targets.
/// Writer never grades itself — this is the rater with multi-axis judgment.
pub struct ExternalVerifierStage {
    axes: Vec<VerificationAxis>,
}

impl Default for ExternalVerifierStage {
    fn default() -> Self {
        let workspace = std::env::var("NEOTRIX_WORKSPACE").unwrap_or_else(|_| ".".into());
        Self {
            axes: default_verification_axes(&workspace),
        }
    }
}
impl ExternalVerifierStage {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_axes(axes: Vec<VerificationAxis>) -> Self {
        Self { axes }
    }
}

impl BrainStage for ExternalVerifierStage {
    fn name(&self) -> &str {
        "external_verifier"
    }
    fn frequency(&self) -> usize {
        3
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.task_scratch.micro_edits.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let workspace = std::env::var("NEOTRIX_WORKSPACE").unwrap_or_else(|_| ".".into());
        let mut verdict = VerdictVector::new();

        for axis in &self.axes {
            let result = run_verification_axis(axis, &workspace);
            verdict.push(result.clone());

            let evidence = PhaseEvidence {
                id: format!("ver-{}-{}", axis.name, brain.iteration),
                phase_id: "external_verify".into(),
                stage_name: "external_verifier".into(),
                evidence_type: EvidenceType::Custom(axis.name.clone()),
                description: format!(
                    "axis '{}': {}",
                    axis.name,
                    if result.passed { "✅" } else { "❌" }
                ),
                success: result.passed,
                details: result.details.clone(),
                iteration: brain.iteration,
            };
            brain.goal_state.phase_evidence.push_back(evidence);

            if result.passed {
                log::info!("[verifier] {} ✅ {}", axis.name, result.details);
            } else {
                log::warn!(
                    "[verifier] {} ❌ {} (repair: {:?})",
                    axis.name,
                    result.details,
                    axis.repair
                );
            }
        }

        let penalty = verdict.penalty();
        if penalty > 0.0 {
            brain.task_scratch.reward -= penalty;
            log::warn!(
                "[verifier] penalty={:.3} (verdict: {})",
                penalty,
                verdict.summary()
            );
        } else {
            log::info!("[verifier] {}", verdict.summary());
        }

        Ok(StageDecision::Continue)
    }
    fn verify_step(
        &self,
        brain: &SelfIteratingBrain,
    ) -> Option<super::super::vsi_verifier::VsiStepVerdict> {
        let ev: Vec<&PhaseEvidence> = brain
            .goal_state
            .phase_evidence
            .iter()
            .filter(|e| e.stage_name == "external_verifier")
            .collect();
        if ev.is_empty() {
            return None;
        }
        let total = ev.len();
        let passed = ev.iter().filter(|e| e.success).count();
        let ratio = passed as f64 / total as f64;
        if ratio >= 0.5 {
            Some(super::super::vsi_verifier::VsiStepVerdict::Pass(ratio))
        } else {
            Some(super::super::vsi_verifier::VsiStepVerdict::Fail(
                format!("{}/{} verification axes passed", passed, total),
                ratio,
            ))
        }
    }
}

/// Stage: recall semantically similar past journal entries at pipeline start.
/// This enables cross-session memory — the model doesn't start each goal from scratch.
pub struct SemanticRecallStage;

impl Default for SemanticRecallStage {
    fn default() -> Self {
        Self
    }
}
impl SemanticRecallStage {
    pub fn new() -> Self {
        Self
    }
}

impl BrainStage for SemanticRecallStage {
    fn name(&self) -> &str {
        "semantic_recall"
    }
    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.task_scratch.current_task.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let idx = match JournalIndex::open() {
            Ok(idx) => idx,
            Err(_) => return Ok(StageDecision::Continue),
        };

        match idx.search(&brain.task_scratch.current_task, 3) {
            Ok(results) if !results.is_empty() => {
                log::info!(
                    "[recall] {} relevant past journal entries found",
                    results.len()
                );
                for (entry, score) in &results {
                    log::info!(
                        "[recall]   [{:.2}] {} ({})",
                        score,
                        entry.goal_text,
                        entry.timestamp
                    );
                    // Tag into current context via evidence log
                    let evidence = PhaseEvidence {
                        id: format!("recall-{}-{}", entry.id, brain.iteration),
                        phase_id: "recall".into(),
                        stage_name: "semantic_recall".into(),
                        evidence_type: EvidenceType::Custom("recall".into()),
                        description: format!(
                            "past goal '{}' (score={:.2}, success={})",
                            entry.goal_text, score, entry.success
                        ),
                        success: true,
                        details: format!("recalled from journal: {}", entry.id),
                        iteration: brain.iteration,
                    };
                    brain.goal_state.phase_evidence.push_back(evidence);
                }
            }
            Ok(_) => log::debug!("[recall] no relevant past journals found"),
            Err(e) => log::debug!("[recall] search failed: {}", e),
        }

        Ok(StageDecision::Continue)
    }
}
