use super::SelfIteratingBrain;
use crate::core::MicroEdit;
use super::pipeline::{BrainStage, StageDecision};
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_world_model::TaskType;
use crate::neotrix::nt_world_journal_index::JournalIndex;
use std::collections::VecDeque;

/// One phase in the goal decomposition with concrete done criteria
#[derive(Debug, Clone)]
pub struct GoalPhase {
    pub id: String,
    pub description: String,
    pub done_criteria: Vec<String>,
    pub verified: bool,
    pub evidence_ids: Vec<String>,
}

/// A decomposable goal contract: phases with checkable completion conditions
#[derive(Debug, Clone)]
pub struct GoalContract {
    pub original_goal: String,
    pub phases: Vec<GoalPhase>,
    pub current_phase: usize,
    pub completed: bool,
    pub final_verification: Option<GoalVerificationReport>,
}

impl GoalContract {
    /// Decompose a task description into phases based on task type
    pub fn decompose(task: &str, task_type: TaskType) -> Self {
        let phases = match task_type {
            TaskType::CodeGeneration | TaskType::CodeAnalysis | TaskType::Design | TaskType::UIDesign => {
                Self::code_phases(task)
            }
            TaskType::Debugging | TaskType::CodeReview => Self::debug_phases(task),
            TaskType::Learning | TaskType::Research => Self::research_phases(task),
            TaskType::Planning | TaskType::Security => Self::engineering_phases(task),
            TaskType::Reflection => Self::reflection_phases(task),
            _ => Self::general_phases(task),
        };
        Self {
            original_goal: task.to_string(),
            phases,
            current_phase: 0,
            completed: false,
            final_verification: None,
        }
    }

    fn code_phases(task: &str) -> Vec<GoalPhase> {
        let has_test = task.to_lowercase().contains("test");
        let mut phases = vec![
            GoalPhase {
                id: "understand".into(),
                description: "理解需求与现有代码结构".into(),
                done_criteria: vec!["识别出要修改/添加的文件".into(), "明确输入输出契约".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "implement".into(),
                description: "实现功能或修改".into(),
                done_criteria: vec!["代码编译通过".into(), "新增代码遵循现有模式".into()],
                verified: false, evidence_ids: vec![],
            },
        ];
        if has_test {
            phases.push(GoalPhase {
                id: "test".into(),
                description: "编写并通过测试".into(),
                done_criteria: vec!["新增测试覆盖关键路径".into(), "cargo test 通过".into()],
                verified: false, evidence_ids: vec![],
            });
        }
        phases.push(GoalPhase {
            id: "verify".into(),
            description: "验证完整性".into(),
            done_criteria: vec!["无 lint 错误".into(), "无功能回退".into()],
            verified: false, evidence_ids: vec![],
        });
        phases
    }

    fn debug_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "reproduce".into(),
                description: "复现问题".into(),
                done_criteria: vec!["稳定复现的步骤".into(), "确认问题在重现环境中出现".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "diagnose".into(),
                description: "定位根因".into(),
                done_criteria: vec!["找到根因文件/行号".into(), "理解为什么触发".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "fix".into(),
                description: "修复并验证".into(),
                done_criteria: vec!["最小变更解决问题".into(), "原问题场景不再出现".into()],
                verified: false, evidence_ids: vec![],
            },
        ]
    }

    fn research_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "gather".into(),
                description: "收集资料".into(),
                done_criteria: vec!["找到 ≥3 个相关信息源".into(), "提取关键信息".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "synthesize".into(),
                description: "综合理解".into(),
                done_criteria: vec!["形成结构化理解".into(), "信息交叉验证".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "conclude".into(),
                description: "产出结论".into(),
                done_criteria: vec!["回答原始问题".into(), "指出不确定性".into()],
                verified: false, evidence_ids: vec![],
            },
        ]
    }

    fn engineering_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "plan".into(),
                description: "方案设计".into(),
                done_criteria: vec!["技术选型明确".into(), "步骤拆分到可执行粒度".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "build".into(),
                description: "构建实现".into(),
                done_criteria: vec!["各步骤完成".into(), "构建通过".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "review".into(),
                description: "审查验证".into(),
                done_criteria: vec!["满足所有需求".into(), "无回归风险".into()],
                verified: false, evidence_ids: vec![],
            },
        ]
    }

    fn reflection_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "observe".into(),
                description: "回顾事实".into(),
                done_criteria: vec!["列出客观事实".into(), "排除主观解释".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "analyze".into(),
                description: "分析模式".into(),
                done_criteria: vec!["识别出重复模式".into(), "找出因果链".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "apply".into(),
                description: "提炼经验".into(),
                done_criteria: vec!["产出可复用原则".into(), "明确下次改进点".into()],
                verified: false, evidence_ids: vec![],
            },
        ]
    }

    fn general_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "understand".into(),
                description: "理解任务".into(),
                done_criteria: vec!["任务边界清晰".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "execute".into(),
                description: "执行任务".into(),
                done_criteria: vec!["产出符合预期".into()],
                verified: false, evidence_ids: vec![],
            },
            GoalPhase {
                id: "validate".into(),
                description: "验证结果".into(),
                done_criteria: vec!["结果可接受".into()],
                verified: false, evidence_ids: vec![],
            },
        ]
    }

    pub fn current_phase(&self) -> Option<&GoalPhase> {
        self.phases.get(self.current_phase)
    }

    pub fn advance_phase(&mut self) {
        if self.current_phase + 1 < self.phases.len() {
            self.current_phase += 1;
        } else {
            self.completed = true;
        }
    }

    pub fn all_phases_done(&self) -> bool {
        self.phases.iter().all(|p| p.verified)
    }

    pub fn progress_description(&self) -> String {
        let done = self.phases.iter().filter(|p| p.verified).count();
        let total = self.phases.len();
        format!("阶段进度: {}/{} (当前: {})", done, total,
            self.phases.get(self.current_phase).map_or("完成".into(), |p| p.id.clone()))
    }

    pub fn verification_report(&self) -> GoalVerificationReport {
        let mut total = 0;
        let mut met = 0;
        let mut failed = Vec::new();
        let mut unmet_phases = Vec::new();
        for phase in &self.phases {
            for criterion in &phase.done_criteria {
                total += 1;
                if phase.verified {
                    met += 1;
                } else {
                    failed.push((criterion.clone(), format!("阶段 '{}' 未完成", phase.id)));
                }
            }
            if !phase.verified {
                unmet_phases.push(phase.id.clone());
            }
        }
        let is_success = unmet_phases.is_empty();
        GoalVerificationReport {
            total_criteria: total,
            met_criteria: met,
            failed_criteria: failed,
            unmet_phases,
            overall_success: is_success,
        }
    }
}

/// Final verification report mapping original criteria to evidence
#[derive(Debug, Clone)]
pub struct GoalVerificationReport {
    pub total_criteria: usize,
    pub met_criteria: usize,
    pub failed_criteria: Vec<(String, String)>,
    pub unmet_phases: Vec<String>,
    pub overall_success: bool,
}

/// Concrete evidence captured from stage execution
#[derive(Debug, Clone)]
pub struct PhaseEvidence {
    pub id: String,
    pub phase_id: String,
    pub stage_name: String,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub success: bool,
    pub details: String,
    pub iteration: u64,
}

/// Types of evidence that can be captured
#[derive(Debug, Clone)]
pub enum EvidenceType {
    BuildPass,
    TestPass,
    FileDiff(String),
    ErrorCount(usize),
    WarningCount(usize),
    CommandOutput(String),
    RewardDelta(f64),
    Custom(String),
}

/// Recovery action for narrow failure recovery
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    RetryStage,
    NarrowEdit { dimension: String, description: String },
    SkipPhase { phase_id: String, reason: String },
    RollbackToCheckpoint { checkpoint_id: String },
    AbortGoal { reason: String },
}

impl RecoveryAction {
    pub fn description(&self) -> String {
        match self {
            RecoveryAction::RetryStage => "重试当前阶段".into(),
            RecoveryAction::NarrowEdit { dimension, description } => format!("窄修复: {} ({})", dimension, description),
            RecoveryAction::SkipPhase { phase_id, reason } => format!("跳过阶段 {}: {}", phase_id, reason),
            RecoveryAction::RollbackToCheckpoint { checkpoint_id } => format!("回滚到检查点 {}", checkpoint_id),
            RecoveryAction::AbortGoal { reason } => format!("放弃目标: {}", reason),
        }
    }
}

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
        let dim = edits.last()
            .and_then(|e: &MicroEdit| e.dimension_name().map(String::from))
            .unwrap_or_else(|| "unknown".into());
        return RecoveryAction::NarrowEdit {
            dimension: dim,
            description: format!("阶段 '{}' 导致奖励下降 {:.2}，回滚最近编辑", failed_stage, reward_delta),
        };
    }
    RecoveryAction::RetryStage
}

/// Stage: initialize or advance the goal contract on task boundaries
pub struct GoalContractStage;

impl Default for GoalContractStage { fn default() -> Self { Self } }
impl GoalContractStage { pub fn new() -> Self { Self } }

impl BrainStage for GoalContractStage {
    fn name(&self) -> &str { "goal_contract" }
    fn frequency(&self) -> usize { 1 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._current_task.is_empty() {
            return Ok(StageDecision::Skip("no active task".into()));
        }

        if let Some(ref contract) = brain._goal_contract {
            if contract.completed || contract.all_phases_done() {
                return Ok(StageDecision::Skip("goal contract already completed".into()));
            }
            if contract.phases.is_empty() {
                return Ok(StageDecision::Skip("empty goal contract".into()));
            }
            if let Some(current) = contract.current_phase() {
                if current.verified {
                    let mut c = brain._goal_contract.take().unwrap();
                    c.advance_phase();
                    let phase_name = c.phases.get(c.current_phase)
                        .map(|p| p.id.clone())
                        .unwrap_or_else(|| "complete".into());
                    log::info!("[goal_contract] advancing to phase: {}", phase_name);
                    brain._goal_contract = Some(c);
                }
            }
        } else {
            let contract = GoalContract::decompose(&brain._current_task, brain._current_task_type);
            log::info!("[goal_contract] decomposed '{}' into {} phases: {:?}",
                brain._current_task, contract.phases.len(),
                contract.phases.iter().map(|p| p.id.as_str()).collect::<Vec<_>>());
            brain._goal_contract = Some(contract);
            brain._phase_evidence = VecDeque::with_capacity(32);
        }

        Ok(StageDecision::Continue)
    }
}

/// Stage: capture concrete evidence after work stages execute
pub struct EvidenceCaptureStage;

impl Default for EvidenceCaptureStage { fn default() -> Self { Self } }
impl EvidenceCaptureStage { pub fn new() -> Self { Self } }

impl BrainStage for EvidenceCaptureStage {
    fn name(&self) -> &str { "evidence_capture" }
    fn frequency(&self) -> usize { 1 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let contract = match brain._goal_contract.as_ref() {
            Some(c) if !c.phases.is_empty() => c,
            _ => return Ok(StageDecision::Continue),
        };

        let current_phase_id = contract.phases.get(contract.current_phase)
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "unknown".into());

        let evidence_id = format!("ev-{}-{}", current_phase_id, brain.iteration);

        let reward_delta = brain._reward;
        let edit_count = brain._micro_edits.len();
        let tool_count = brain.tool_call_count;
        let has_edits = edit_count > 0;

        let evidence = PhaseEvidence {
            id: evidence_id.clone(),
            phase_id: current_phase_id.clone(),
            stage_name: "evidence_capture".into(),
            evidence_type: EvidenceType::RewardDelta(reward_delta),
            description: format!("阶段 '{}' 迭代 {}: 奖励 {:.3}, 编辑 {} 个, 工具调用 {} 次",
                current_phase_id, brain.iteration, reward_delta, edit_count, tool_count),
            success: reward_delta >= 0.0,
            details: format!("reward={:.3} edits={} tools={}", reward_delta, edit_count, tool_count),
            iteration: brain.iteration,
        };

        brain._phase_evidence.push_back(evidence);

        if has_edits && reward_delta >= 0.0 {
            if let Some(ref mut contract) = brain._goal_contract {
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

impl Default for NarrowRecoveryStage { fn default() -> Self { Self } }
impl NarrowRecoveryStage { pub fn new() -> Self { Self } }

impl BrainStage for NarrowRecoveryStage {
    fn name(&self) -> &str { "narrow_recovery" }
    fn frequency(&self) -> usize { 3 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._reward >= -0.05 {
            return Ok(StageDecision::Continue);
        }
        if brain._stage_results.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let worst_stage = brain._stage_results.iter()
            .min_by(|a, b| a.efc.partial_cmp(&b.efc).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| s.stage_name.clone())
            .unwrap_or_else(|| "unknown".into());

        let recovery = analyze_failure(
            brain,
            &worst_stage,
            brain._reward,
            &brain._micro_edits,
        );

        log::warn!("[narrow_recovery] reward={:.3}, worst_stage={}, action={}",
            brain._reward, worst_stage, recovery.description());

        match recovery {
            RecoveryAction::NarrowEdit { .. } => {
                Ok(StageDecision::Skip(format!("narrow_recovery: {}", recovery.description())))
            }
            RecoveryAction::RetryStage => {
                Ok(StageDecision::Continue)
            }
            RecoveryAction::SkipPhase { phase_id: _, reason } => {
                if let Some(ref mut contract) = brain._goal_contract {
                    contract.advance_phase();
                }
                Ok(StageDecision::Skip(reason))
            }
            RecoveryAction::RollbackToCheckpoint { checkpoint_id } => {
                brain.checkpoint_manager.restore(
                    &mut brain.brain,
                    &mut brain.permission,
                    &mut brain.autonomy,
                    &mut brain._reward,
                    &checkpoint_id,
                ).ok();
                Ok(StageDecision::Skip(format!("rolled back to {}", checkpoint_id)))
            }
            RecoveryAction::AbortGoal { reason } => {
                brain._goal_contract = None;
                brain._phase_evidence.clear();
                Ok(StageDecision::Skip(format!("aborted: {}", reason)))
            }
        }
    }
}

/// Stage: final verification — map evidence against original goal contract
pub struct FinalVerificationStage;

impl Default for FinalVerificationStage { fn default() -> Self { Self } }
impl FinalVerificationStage { pub fn new() -> Self { Self } }

impl BrainStage for FinalVerificationStage {
    fn name(&self) -> &str { "final_verification" }
    fn frequency(&self) -> usize { 5 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let contract = match brain._goal_contract.as_ref() {
            Some(c) if c.all_phases_done() || brain._reward > 0.5 => c.clone(),
            _ => return Ok(StageDecision::Continue),
        };

        let report = contract.verification_report();
        let phase_desc = contract.progress_description();

        let evidence_count = brain._phase_evidence.len();
        let edit_count = brain._micro_edits.len();
        let tool_count = brain.tool_call_count;

        log::info!("[final_verification] {} | 证据: {}, 编辑: {}, 工具: {} | 标准: {}/{} 通过",
            phase_desc, evidence_count, edit_count, tool_count,
            report.met_criteria, report.total_criteria);

        if report.overall_success {
            log::info!("[final_verification] ✅ 目标完成: {}", contract.original_goal);
        } else {
            log::warn!("[final_verification] ⚠️ 目标未完成: {} | 未完成阶段: {:?}",
                contract.original_goal, report.unmet_phases);
        }

        if let Some(ref mut c) = brain._goal_contract {
            c.final_verification = Some(report);
            c.completed = true;
        }

        Ok(StageDecision::Continue)
    }
}

/// Stage: set _goal_complete flag when all phases are verified.
/// This signals the outer SEAL loop to stop iterating — /goal semantics.
pub struct GoalTerminatorStage;

impl Default for GoalTerminatorStage { fn default() -> Self { Self } }
impl GoalTerminatorStage { pub fn new() -> Self { Self } }

impl BrainStage for GoalTerminatorStage {
    fn name(&self) -> &str { "goal_terminator" }
    fn frequency(&self) -> usize { 1 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let contract = match brain._goal_contract.as_ref() {
            Some(c) if !c.completed => c,
            _ => return Ok(StageDecision::Continue),
        };

        if contract.all_phases_done() {
            let report = contract.verification_report();
            if report.overall_success {
                brain._goal_complete = true;
                log::info!("[goal_terminator] ✅ 目标完成，循环终止: {}", contract.original_goal);
                if let Some(ref mut c) = brain._goal_contract {
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

/// Proxy-Judge: single axis in a multi-dimensional verification verdict vector.
/// Each axis runs independently. If one fails, its own repair target is activated.
#[derive(Debug, Clone)]
pub struct VerificationAxis {
    pub name: String,
    pub command: Vec<String>,
    pub current_dir: Option<String>,
    pub severity: AxisSeverity,
    pub repair: RepairTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisSeverity { Critical, High, Medium, Low }

/// What to do when a specific verification axis fails
#[derive(Debug, Clone)]
pub enum RepairTarget {
    /// Re-run the same check (transient failure)
    Retry,
    /// Run a specific repair command
    AutoFix(Vec<String>),
    /// Record evidence but don't block
    LogOnly,
    /// Skip the failing phase
    SkipPhase,
    /// Full rollback
    Rollback,
}

/// Per-axis verdict after running a verification check
#[derive(Debug, Clone)]
pub struct AxisVerdict {
    pub axis_name: String,
    pub passed: bool,
    pub details: String,
    pub exit_code: Option<i32>,
    pub severity: AxisSeverity,
}

/// Aggregate verdict vector from all verification axes
#[derive(Debug, Clone)]
pub struct VerdictVector {
    pub axes: Vec<AxisVerdict>,
    pub passed: bool,
}

impl VerdictVector {
    pub fn new() -> Self {
        Self { axes: Vec::new(), passed: true }
    }

    pub fn push(&mut self, v: AxisVerdict) {
        if !v.passed { self.passed = false; }
        self.axes.push(v);
    }

    pub fn failed_axes(&self) -> Vec<&AxisVerdict> {
        self.axes.iter().filter(|a| !a.passed).collect()
    }

    pub fn summary(&self) -> String {
        let total = self.axes.len();
        let passed = self.axes.iter().filter(|a| a.passed).count();
        let failed = total - passed;
        if failed == 0 {
            format!("✅ {}/{} all passed", passed, total)
        } else {
            let names: Vec<&str> = self.axes.iter()
                .filter(|a| !a.passed).map(|a| a.axis_name.as_str()).collect();
            format!("⚠️ {}/{} passed, {} failed: {:?}", passed, total, failed, names)
        }
    }

    pub fn penalty(&self) -> f64 {
        let mut p = 0.0;
        for v in &self.axes {
            if !v.passed {
                match v.severity {
                    AxisSeverity::Critical => p += 0.10,
                    AxisSeverity::High => p += 0.05,
                    AxisSeverity::Medium => p += 0.02,
                    AxisSeverity::Low => p += 0.01,
                }
            }
        }
        p
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
            command: vec!["cargo".into(), "fmt".into(), "--check".into(), "--quiet".into()],
            current_dir: Some(workspace.into()),
            severity: AxisSeverity::Low,
            repair: RepairTarget::AutoFix(vec!["cargo".into(), "fmt".into()]),
        },
    ]
}

/// Run a single verification axis and return its verdict
pub fn run_verification_axis(axis: &VerificationAxis, workspace: &str) -> AxisVerdict {
    let _cmd = if axis.command.is_empty() { return AxisVerdict {
        axis_name: axis.name.clone(), passed: true, details: "no command".into(), exit_code: None, severity: axis.severity,
    }};

    let program = &axis.command[0];
    let args: Vec<&str> = axis.command[1..].iter().map(|s| s.as_str()).collect();
    let dir = axis.current_dir.as_deref().unwrap_or(workspace);

    match std::process::Command::new(program).args(&args).current_dir(dir).output() {
        Ok(output) => {
            let passed = output.status.success();
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if passed { "ok".into() } else {
                let lines: Vec<&str> = stderr.lines().chain(stdout.lines()).take(5).collect();
                lines.join("\n")
            };
            AxisVerdict {
                axis_name: axis.name.clone(), passed, details: detail,
                exit_code: output.status.code(), severity: axis.severity,
            }
        }
        Err(e) => AxisVerdict {
            axis_name: axis.name.clone(), passed: false,
            details: format!("cannot execute: {}", e), exit_code: None, severity: axis.severity,
        },
    }
}

/// Stage: run a vector of independent external checks (compile, test, lint, format).
/// Each axis is scored independently. Failed axes route to their own repair targets.
/// Writer never grades itself — this is the rater with multi-axis judgment.
pub struct ExternalVerifierStage {
    axes: Vec<VerificationAxis>,
}

impl Default for ExternalVerifierStage { fn default() -> Self {
    let workspace = std::env::var("NEOTRIX_WORKSPACE").unwrap_or_else(|_| ".".into());
    Self { axes: default_verification_axes(&workspace) }
}}
impl ExternalVerifierStage {
    pub fn new() -> Self { Self::default() }
    pub fn with_axes(axes: Vec<VerificationAxis>) -> Self { Self { axes } }
}

impl BrainStage for ExternalVerifierStage {
    fn name(&self) -> &str { "external_verifier" }
    fn frequency(&self) -> usize { 3 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._micro_edits.is_empty() {
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
                description: format!("axis '{}': {}", axis.name, if result.passed { "✅" } else { "❌" }),
                success: result.passed,
                details: result.details.clone(),
                iteration: brain.iteration,
            };
            brain._phase_evidence.push_back(evidence);

            if result.passed {
                log::info!("[verifier] {} ✅ {}", axis.name, result.details);
            } else {
                log::warn!("[verifier] {} ❌ {} (repair: {:?})", axis.name, result.details, axis.repair);
            }
        }

        let penalty = verdict.penalty();
        if penalty > 0.0 {
            brain._reward -= penalty;
            log::warn!("[verifier] penalty={:.3} (verdict: {})", penalty, verdict.summary());
        } else {
            log::info!("[verifier] {}", verdict.summary());
        }

        Ok(StageDecision::Continue)
    }
}

/// Write goal contract + evidence to `.neotrix/journal/` as persistent markdown
pub fn write_journal(brain: &SelfIteratingBrain) -> Result<(), std::io::Error> {
    let contract = match brain._goal_contract.as_ref() {
        Some(c) => c,
        None => return Ok(()),
    };

    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let journal_dir = home.join(".neotrix").join("journal");
    std::fs::create_dir_all(&journal_dir)?;

    let session_id = chrono::Local::now().format("goal-%Y%m%d-%H%M%S");
    let path = journal_dir.join(format!("{}.md", session_id));

    let mut lines = Vec::new();
    lines.push(format!("# Goal: {}", contract.original_goal));
    lines.push(format!("- Completed: {}", contract.completed));
    lines.push(format!("- Total phases: {}", contract.phases.len()));
    lines.push(String::new());

    for (i, phase) in contract.phases.iter().enumerate() {
        lines.push(format!("## Phase {}: {} ({})", i + 1, phase.id, phase.description));
        lines.push(format!("- Verified: {}", phase.verified));
        for criterion in &phase.done_criteria {
            lines.push(format!("  - [{}] {}", if phase.verified { "x" } else { " " }, criterion));
        }
        if let Some(ref report) = contract.final_verification {
            lines.push(format!("- Criteria met: {}/{}", report.met_criteria, report.total_criteria));
        }
        lines.push(String::new());
    }

    let evidence_count = brain._phase_evidence.len();
    lines.push(format!("## Evidence ({})", evidence_count));
    for ev in &brain._phase_evidence {
        lines.push(format!("- {} (iter {}): {} — {}",
            ev.id, ev.iteration, ev.description,
            if ev.success { "✅" } else { "❌" }));
    }
    lines.push(String::new());

    if let Some(ref report) = contract.final_verification {
        lines.push(format!("## Verification: {}/{} criteria met", report.met_criteria, report.total_criteria));
        if !report.unmet_phases.is_empty() {
            lines.push(format!("- Unmet phases: {}", report.unmet_phases.join(", ")));
        }
        for (criterion, reason) in &report.failed_criteria {
            lines.push(format!("  - ❌ {}: {}", criterion, reason));
        }
        lines.push(format!("- Overall: {}", if report.overall_success { "✅ PASS" } else { "❌ FAIL" }));
    }

    std::fs::write(&path, lines.join("\n"))?;
    log::info!("[journal] written to {:?}", path);

    // Auto-index into cross-session journal search
    let session_id_str = session_id.to_string();
    let timestamp_str = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    if let Ok(idx) = JournalIndex::open() {
        let evidence_count = brain._phase_evidence.len();
        let success = contract.final_verification.as_ref().map_or(false, |r| r.overall_success);
        if let Err(e) = idx.add_entry(&session_id_str, &contract.original_goal, &timestamp_str, evidence_count, success) {
            log::warn!("[journal] index entry failed: {}", e);
        } else {
            log::info!("[journal] indexed as '{}' (total: {})", session_id_str, idx.count().unwrap_or(0));
        }
    }

    Ok(())
}

/// Stage: recall semantically similar past journal entries at pipeline start.
/// This enables cross-session memory — the model doesn't start each goal from scratch.
pub struct SemanticRecallStage;

impl Default for SemanticRecallStage { fn default() -> Self { Self } }
impl SemanticRecallStage { pub fn new() -> Self { Self } }

impl BrainStage for SemanticRecallStage {
    fn name(&self) -> &str { "semantic_recall" }
    fn frequency(&self) -> usize { 1 }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._current_task.is_empty() {
            return Ok(StageDecision::Continue);
        }

        let idx = match JournalIndex::open() {
            Ok(idx) => idx,
            Err(_) => return Ok(StageDecision::Continue),
        };

        match idx.search(&brain._current_task, 3) {
            Ok(results) if !results.is_empty() => {
                log::info!("[recall] {} relevant past journal entries found", results.len());
                for (entry, score) in &results {
                    log::info!("[recall]   [{:.2}] {} ({})", score, entry.goal_text, entry.timestamp);
                    // Tag into current context via evidence log
                    let evidence = PhaseEvidence {
                        id: format!("recall-{}-{}", entry.id, brain.iteration),
                        phase_id: "recall".into(),
                        stage_name: "semantic_recall".into(),
                        evidence_type: EvidenceType::Custom("recall".into()),
                        description: format!("past goal '{}' (score={:.2}, success={})", entry.goal_text, score, entry.success),
                        success: true,
                        details: format!("recalled from journal: {}", entry.id),
                        iteration: brain.iteration,
                    };
                    brain._phase_evidence.push_back(evidence);
                }
            }
            Ok(_) => log::debug!("[recall] no relevant past journals found"),
            Err(e) => log::debug!("[recall] search failed: {}", e),
        }

        Ok(StageDecision::Continue)
    }
}

/// Public helper: set up the _goal_complete check in the outer loop
pub fn should_stop_seal_loop(brain: &SelfIteratingBrain) -> bool {
    brain._goal_complete || brain._goal_contract.as_ref().map_or(false, |c| c.completed)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_code_task() {
        let contract = GoalContract::decompose("实现用户登录功能", TaskType::CodeGeneration);
        assert_eq!(contract.phases.len(), 4);
        assert_eq!(contract.phases[0].id, "understand");
        assert_eq!(contract.phases[2].done_criteria[0], "新增测试覆盖关键路径");
    }

    #[test]
    fn test_decompose_debug_task() {
        let contract = GoalContract::decompose("修复内存泄漏", TaskType::Debugging);
        assert_eq!(contract.phases.len(), 3);
        assert_eq!(contract.phases[0].id, "reproduce");
        assert_eq!(contract.phases[2].id, "fix");
    }

    #[test]
    fn test_decompose_research_task() {
        let contract = GoalContract::decompose("研究Rust异步编程", TaskType::Research);
        assert_eq!(contract.phases.len(), 3);
        assert_eq!(contract.phases[1].id, "synthesize");
    }

    #[test]
    fn test_decompose_code_without_test() {
        let contract = GoalContract::decompose("重构模块", TaskType::CodeGeneration);
        assert_eq!(contract.phases.len(), 3);
    }

    #[test]
    fn test_code_with_test_flag() {
        let contract = GoalContract::decompose("添加单元测试", TaskType::CodeGeneration);
        assert_eq!(contract.phases.len(), 4);
    }

    #[test]
    fn test_advance_phase() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        assert_eq!(contract.current_phase, 0);
        contract.advance_phase();
        assert_eq!(contract.current_phase, 1);
        contract.advance_phase();
        assert_eq!(contract.current_phase, 2);
        contract.advance_phase();
        assert!(contract.completed);
    }

    #[test]
    fn test_all_phases_done() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        assert!(!contract.all_phases_done());
        for phase in &mut contract.phases {
            phase.verified = true;
        }
        assert!(contract.all_phases_done());
    }

    #[test]
    fn test_verification_report_all_pass() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        for phase in &mut contract.phases {
            phase.verified = true;
        }
        let report = contract.verification_report();
        assert!(report.overall_success);
        assert!(report.unmet_phases.is_empty());
    }

    #[test]
    fn test_verification_report_partial() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        contract.phases[0].verified = true;
        let report = contract.verification_report();
        assert!(!report.overall_success);
        assert_eq!(report.unmet_phases.len(), 2);
    }

    #[test]
    fn test_analyze_failure_no_edits() {
        let recovery = analyze_failure_no_brain("test_stage", -0.2, &[]);
        assert!(matches!(recovery, RecoveryAction::RetryStage));
    }

    #[test]
    fn test_analyze_failure_severe_drop() {
        let edits = vec![MicroEdit::AdjustDimension("ui".into(), -0.5)];
        let recovery = analyze_failure_no_brain("test_stage", -0.5, &edits);
        match recovery {
            RecoveryAction::NarrowEdit { dimension, .. } => assert_eq!(dimension, "ui"),
            _ => panic!("expected NarrowEdit"),
        }
    }

    #[test]
    fn test_evidence_capture() {
        let evidence = PhaseEvidence {
            id: "ev-test-1".into(),
            phase_id: "implement".into(),
            stage_name: "apply_edits".into(),
            evidence_type: EvidenceType::BuildPass,
            description: "构建通过".into(),
            success: true,
            details: "exit code 0".into(),
            iteration: 5,
        };
        assert_eq!(evidence.phase_id, "implement");
        assert!(evidence.success);
    }

    #[test]
    fn test_progress_description() {
        let contract = GoalContract::decompose("task", TaskType::General);
        let desc = contract.progress_description();
        assert!(desc.contains("0/3"));
        assert!(desc.contains("understand"));
    }

    #[test]
    fn test_recovery_action_descriptions() {
        assert_eq!(RecoveryAction::RetryStage.description(), "重试当前阶段");
        let narrow = RecoveryAction::NarrowEdit { dimension: "x".into(), description: "fix".into() };
        assert!(narrow.description().contains("x"));
        let skip = RecoveryAction::SkipPhase { phase_id: "test".into(), reason: "not needed".into() };
        assert!(skip.description().contains("test"));
        let abort = RecoveryAction::AbortGoal { reason: "invalid".into() };
        assert!(abort.description().contains("invalid"));
    }

    fn analyze_failure_no_brain(stage: &str, reward: f64, edits: &[MicroEdit]) -> RecoveryAction {
        if edits.is_empty() && reward < -0.1 {
            return RecoveryAction::RetryStage;
        }
        if reward < -0.3 {
            let dim = edits.last()
            .and_then(|e: &MicroEdit| e.dimension_name().map(String::from))
                .unwrap_or_else(|| "unknown".into());
            return RecoveryAction::NarrowEdit {
                dimension: dim,
                description: format!("阶段 '{}' 导致奖励下降 {:.2}", stage, reward),
            };
        }
        RecoveryAction::RetryStage
    }
}
