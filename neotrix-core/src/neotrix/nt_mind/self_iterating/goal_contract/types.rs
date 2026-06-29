use crate::neotrix::nt_expert_routing::TaskType;

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
            TaskType::CodeGeneration
            | TaskType::CodeAnalysis
            | TaskType::Design
            | TaskType::UIDesign => Self::code_phases(task),
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
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "implement".into(),
                description: "实现功能或修改".into(),
                done_criteria: vec!["代码编译通过".into(), "新增代码遵循现有模式".into()],
                verified: false,
                evidence_ids: vec![],
            },
        ];
        if has_test {
            phases.push(GoalPhase {
                id: "test".into(),
                description: "编写并通过测试".into(),
                done_criteria: vec!["新增测试覆盖关键路径".into(), "cargo test 通过".into()],
                verified: false,
                evidence_ids: vec![],
            });
        }
        phases.push(GoalPhase {
            id: "verify".into(),
            description: "验证完整性".into(),
            done_criteria: vec!["无 lint 错误".into(), "无功能回退".into()],
            verified: false,
            evidence_ids: vec![],
        });
        phases
    }

    fn debug_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "reproduce".into(),
                description: "复现问题".into(),
                done_criteria: vec!["稳定复现的步骤".into(), "确认问题在重现环境中出现".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "diagnose".into(),
                description: "定位根因".into(),
                done_criteria: vec!["找到根因文件/行号".into(), "理解为什么触发".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "fix".into(),
                description: "修复并验证".into(),
                done_criteria: vec!["最小变更解决问题".into(), "原问题场景不再出现".into()],
                verified: false,
                evidence_ids: vec![],
            },
        ]
    }

    fn research_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "gather".into(),
                description: "收集资料".into(),
                done_criteria: vec!["找到 ≥3 个相关信息源".into(), "提取关键信息".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "synthesize".into(),
                description: "综合理解".into(),
                done_criteria: vec!["形成结构化理解".into(), "信息交叉验证".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "conclude".into(),
                description: "产出结论".into(),
                done_criteria: vec!["回答原始问题".into(), "指出不确定性".into()],
                verified: false,
                evidence_ids: vec![],
            },
        ]
    }

    fn engineering_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "plan".into(),
                description: "方案设计".into(),
                done_criteria: vec!["技术选型明确".into(), "步骤拆分到可执行粒度".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "build".into(),
                description: "构建实现".into(),
                done_criteria: vec!["各步骤完成".into(), "构建通过".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "review".into(),
                description: "审查验证".into(),
                done_criteria: vec!["满足所有需求".into(), "无回归风险".into()],
                verified: false,
                evidence_ids: vec![],
            },
        ]
    }

    fn reflection_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "observe".into(),
                description: "回顾事实".into(),
                done_criteria: vec!["列出客观事实".into(), "排除主观解释".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "analyze".into(),
                description: "分析模式".into(),
                done_criteria: vec!["识别出重复模式".into(), "找出因果链".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "apply".into(),
                description: "提炼经验".into(),
                done_criteria: vec!["产出可复用原则".into(), "明确下次改进点".into()],
                verified: false,
                evidence_ids: vec![],
            },
        ]
    }

    fn general_phases(_task: &str) -> Vec<GoalPhase> {
        vec![
            GoalPhase {
                id: "understand".into(),
                description: "理解任务".into(),
                done_criteria: vec!["任务边界清晰".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "execute".into(),
                description: "执行任务".into(),
                done_criteria: vec!["产出符合预期".into()],
                verified: false,
                evidence_ids: vec![],
            },
            GoalPhase {
                id: "validate".into(),
                description: "验证结果".into(),
                done_criteria: vec!["结果可接受".into()],
                verified: false,
                evidence_ids: vec![],
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
        format!(
            "阶段进度: {}/{} (当前: {})",
            done,
            total,
            self.phases
                .get(self.current_phase)
                .map_or("完成".into(), |p| p.id.clone())
        )
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
    NarrowEdit {
        dimension: String,
        description: String,
    },
    SkipPhase {
        phase_id: String,
        reason: String,
    },
    RollbackToCheckpoint {
        checkpoint_id: String,
    },
    AbortGoal {
        reason: String,
    },
}

impl RecoveryAction {
    pub fn description(&self) -> String {
        match self {
            RecoveryAction::RetryStage => "重试当前阶段".into(),
            RecoveryAction::NarrowEdit {
                dimension,
                description,
            } => format!("窄修复: {} ({})", dimension, description),
            RecoveryAction::SkipPhase { phase_id, reason } => {
                format!("跳过阶段 {}: {}", phase_id, reason)
            }
            RecoveryAction::RollbackToCheckpoint { checkpoint_id } => {
                format!("回滚到检查点 {}", checkpoint_id)
            }
            RecoveryAction::AbortGoal { reason } => format!("放弃目标: {}", reason),
        }
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
pub enum AxisSeverity {
    Critical,
    High,
    Medium,
    Low,
}

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
        Self {
            axes: Vec::new(),
            passed: true,
        }
    }

    pub fn push(&mut self, v: AxisVerdict) {
        if !v.passed {
            self.passed = false;
        }
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
            let names: Vec<&str> = self
                .axes
                .iter()
                .filter(|a| !a.passed)
                .map(|a| a.axis_name.as_str())
                .collect();
            format!(
                "⚠️ {}/{} passed, {} failed: {:?}",
                passed, total, failed, names
            )
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
