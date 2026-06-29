use super::sub_agent::{
    LeadAgentPlan, RecoveryStrategy, SubAgentCapability, SubTaskSpec, TaskDecomposition,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarnessKind {
    FanOut,
    Pipeline,
    Adversarial,
    ClassifyAndAct,
}

impl HarnessKind {
    pub fn name(&self) -> &'static str {
        match self {
            HarnessKind::FanOut => "fan-out",
            HarnessKind::Pipeline => "pipeline",
            HarnessKind::Adversarial => "adversarial",
            HarnessKind::ClassifyAndAct => "classify-and-act",
        }
    }
}

pub trait Harness: Send + Sync {
    fn kind(&self) -> HarnessKind;
    fn orchestrate(&self, goal: &str, keywords: &[String]) -> LeadAgentPlan;
}

// ─── FanOut Harness (default) ───────────────────────────────────────

pub struct FanOutHarness;

impl Harness for FanOutHarness {
    fn kind(&self) -> HarnessKind {
        HarnessKind::FanOut
    }

    fn orchestrate(&self, goal: &str, keywords: &[String]) -> LeadAgentPlan {
        let joined = keywords.join(" ");
        let mut sub_tasks = Vec::new();
        let mut deps = Vec::new();

        if joined.contains("refactor") || joined.contains("migrate") || joined.contains("rewrite") {
            sub_tasks.push(
                SubTaskSpec::new(
                    1,
                    "Analyze current codebase structure and dependencies",
                    SubAgentCapability::Researcher,
                )
                .with_artifacts(vec!["dependency_map".into()]),
            );
            sub_tasks.push(
                SubTaskSpec::new(
                    2,
                    &format!("Plan refactoring strategy for: {}", goal),
                    SubAgentCapability::Planner,
                )
                .with_artifacts(vec!["refactor_plan".into()]),
            );
            deps.push((1, 2));
            sub_tasks.push(
                SubTaskSpec::new(3, "Execute refactoring changes", SubAgentCapability::Coder)
                    .with_artifacts(vec!["changed_files".into()]),
            );
            deps.push((2, 3));
            sub_tasks.push(
                SubTaskSpec::new(
                    4,
                    "Write/update tests for refactored code",
                    SubAgentCapability::Tester,
                )
                .with_artifacts(vec!["test_files".into()]),
            );
            deps.push((3, 4));
            sub_tasks.push(
                SubTaskSpec::new(
                    5,
                    "Security audit of refactored code",
                    SubAgentCapability::SecurityAuditor,
                )
                .with_artifacts(vec!["audit_report".into()]),
            );
            deps.push((3, 5));
            sub_tasks.push(
                SubTaskSpec::new(
                    6,
                    "Update documentation for refactored code",
                    SubAgentCapability::Documenter,
                )
                .with_artifacts(vec!["docs".into()]),
            );
            deps.push((3, 6));
        } else if joined.contains("review") || joined.contains("audit") {
            sub_tasks.push(
                SubTaskSpec::new(
                    1,
                    "Static analysis and security review",
                    SubAgentCapability::SecurityAuditor,
                )
                .with_artifacts(vec!["security_report".into()]),
            );
            sub_tasks.push(
                SubTaskSpec::new(
                    2,
                    "Code quality and style review",
                    SubAgentCapability::Reviewer,
                )
                .with_artifacts(vec!["quality_report".into()]),
            );
            sub_tasks.push(
                SubTaskSpec::new(
                    3,
                    "Performance and architecture review",
                    SubAgentCapability::Reviewer,
                )
                .with_artifacts(vec!["perf_report".into()]),
            );
            sub_tasks.push(
                SubTaskSpec::new(4, "Test coverage analysis", SubAgentCapability::Tester)
                    .with_artifacts(vec!["coverage_report".into()]),
            );
        } else if joined.contains("feature")
            || joined.contains("implement")
            || joined.contains("build")
            || joined.contains("add")
        {
            sub_tasks.push(
                SubTaskSpec::new(
                    1,
                    "Research existing patterns and requirements",
                    SubAgentCapability::Researcher,
                )
                .with_artifacts(vec!["research_notes".into()]),
            );
            sub_tasks.push(
                SubTaskSpec::new(
                    2,
                    &format!("Design implementation plan for: {}", goal),
                    SubAgentCapability::Planner,
                )
                .with_artifacts(vec!["design_doc".into()]),
            );
            deps.push((1, 2));
            sub_tasks.push(
                SubTaskSpec::new(3, "Implement the feature", SubAgentCapability::Coder)
                    .with_artifacts(vec!["implementation".into()]),
            );
            deps.push((2, 3));
            sub_tasks.push(
                SubTaskSpec::new(
                    4,
                    "Write tests for the new feature",
                    SubAgentCapability::Tester,
                )
                .with_artifacts(vec!["tests".into()]),
            );
            deps.push((3, 4));
            sub_tasks.push(
                SubTaskSpec::new(
                    5,
                    "Document the new feature",
                    SubAgentCapability::Documenter,
                )
                .with_artifacts(vec!["docs".into()]),
            );
            deps.push((3, 5));
        } else {
            sub_tasks.push(SubTaskSpec::new(1, goal, SubAgentCapability::Coder));
        }

        LeadAgentPlan {
            goal: goal.to_string(),
            decomposition: TaskDecomposition {
                sub_tasks,
                dependency_graph: deps,
            },
            strategy: "fan-out".into(),
            created_at: format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        }
    }
}

// ─── Pipeline Harness (sequential stages) ───────────────────────────

pub struct PipelineHarness;

impl Harness for PipelineHarness {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Pipeline
    }

    fn orchestrate(&self, goal: &str, _keywords: &[String]) -> LeadAgentPlan {
        let mut deps = Vec::new();
        let sub_tasks = vec![
            SubTaskSpec::new(
                1,
                &format!("Stage 1 — Research and gather context for: {}", goal),
                SubAgentCapability::Researcher,
            )
            .with_artifacts(vec!["context".into()]),
            SubTaskSpec::new(
                2,
                &format!("Stage 2 — Design solution for: {}", goal),
                SubAgentCapability::Planner,
            )
            .with_artifacts(vec!["design".into()])
            .with_recovery(RecoveryStrategy::Escalate {
                retry_limit: 2,
                target: SubAgentCapability::Planner,
            }),
            SubTaskSpec::new(
                3,
                &format!("Stage 3 — Implement solution for: {}", goal),
                SubAgentCapability::Coder,
            )
            .with_artifacts(vec!["implementation".into()]),
            SubTaskSpec::new(
                4,
                &format!("Stage 4 — Test implementation for: {}", goal),
                SubAgentCapability::Tester,
            )
            .with_artifacts(vec!["tests".into()]),
            SubTaskSpec::new(
                5,
                &format!("Stage 5 — Document and finalize: {}", goal),
                SubAgentCapability::Documenter,
            )
            .with_artifacts(vec!["docs".into()]),
        ];
        deps.push((1, 2));
        deps.push((2, 3));
        deps.push((3, 4));
        deps.push((4, 5));

        LeadAgentPlan {
            goal: goal.to_string(),
            decomposition: TaskDecomposition {
                sub_tasks,
                dependency_graph: deps,
            },
            strategy: "pipeline".into(),
            created_at: format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        }
    }
}

// ─── Adversarial Harness (dual verification) ────────────────────────

pub struct AdversarialHarness;

impl Harness for AdversarialHarness {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Adversarial
    }

    fn orchestrate(&self, goal: &str, _keywords: &[String]) -> LeadAgentPlan {
        let mut deps = Vec::new();
        let sub_tasks = vec![
            SubTaskSpec::new(1, &format!("Primary: {}", goal), SubAgentCapability::Coder)
                .with_artifacts(vec!["primary_output".into()]),
            SubTaskSpec::new(
                2,
                &format!("Adversarial verification of: {}", goal),
                SubAgentCapability::SecurityAuditor,
            )
            .with_artifacts(vec!["verification_report".into()])
            .with_recovery(RecoveryStrategy::Retry { retry_limit: 3 }),
            SubTaskSpec::new(
                3,
                &format!("Independent cross-check of: {}", goal),
                SubAgentCapability::Reviewer,
            )
            .with_artifacts(vec!["cross_check".into()])
            .with_recovery(RecoveryStrategy::Retry { retry_limit: 3 }),
            SubTaskSpec::new(
                4,
                &format!("Synthesize results for: {}", goal),
                SubAgentCapability::Integrator,
            )
            .with_artifacts(vec!["final_report".into()]),
        ];
        deps.push((1, 2));
        deps.push((1, 3));
        deps.push((2, 4));
        deps.push((3, 4));

        LeadAgentPlan {
            goal: goal.to_string(),
            decomposition: TaskDecomposition {
                sub_tasks,
                dependency_graph: deps,
            },
            strategy: "adversarial".into(),
            created_at: format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        }
    }
}

// ─── Classify and Act Harness ───────────────────────────────────────

pub struct ClassifyAndActHarness;

impl Harness for ClassifyAndActHarness {
    fn kind(&self) -> HarnessKind {
        HarnessKind::ClassifyAndAct
    }

    fn orchestrate(&self, goal: &str, _keywords: &[String]) -> LeadAgentPlan {
        let mut deps = Vec::new();
        let sub_tasks = vec![
            SubTaskSpec::new(
                1,
                "Classify task type and determine execution strategy",
                SubAgentCapability::Planner,
            )
            .with_artifacts(vec!["classification".into()]),
            SubTaskSpec::new(
                2,
                &format!("Execute primary work for: {}", goal),
                SubAgentCapability::Coder,
            )
            .with_artifacts(vec!["primary_output".into()]),
            SubTaskSpec::new(
                3,
                &format!("Quality assurance for: {}", goal),
                SubAgentCapability::Tester,
            )
            .with_artifacts(vec!["qa_report".into()]),
        ];
        deps.push((1, 2));
        deps.push((1, 3));

        LeadAgentPlan {
            goal: goal.to_string(),
            decomposition: TaskDecomposition {
                sub_tasks,
                dependency_graph: deps,
            },
            strategy: "classify-and-act".into(),
            created_at: format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        }
    }
}

// ─── Harness registry ───────────────────────────────────────────────

/// Static fallback used when the registry is missing FanOutHarness.
/// This prevents a production panic on an invariant violation.
static FALLBACK_HARNESS: FanOutHarness = FanOutHarness;

pub struct HarnessRegistry {
    harnesses: HashMap<HarnessKind, Box<dyn Harness>>,
}

impl HarnessRegistry {
    pub fn new() -> Self {
        let mut harnesses: HashMap<HarnessKind, Box<dyn Harness>> = HashMap::new();
        harnesses.insert(HarnessKind::FanOut, Box::new(FanOutHarness));
        harnesses.insert(HarnessKind::Pipeline, Box::new(PipelineHarness));
        harnesses.insert(HarnessKind::Adversarial, Box::new(AdversarialHarness));
        harnesses.insert(HarnessKind::ClassifyAndAct, Box::new(ClassifyAndActHarness));
        Self { harnesses }
    }

    pub fn get(&self, kind: HarnessKind) -> Option<&dyn Harness> {
        self.harnesses.get(&kind).map(|b| b.as_ref())
    }

    pub fn select(&self, keywords: &[String], effort: &str) -> &dyn Harness {
        let kind = if effort == "deep" {
            // deep effort prefers adversarial for non-trivial tasks
            let joined = keywords.join(" ");
            if joined.len() > 20 {
                HarnessKind::Adversarial
            } else {
                HarnessKind::FanOut
            }
        } else {
            let joined = keywords.join(" ");
            if joined.contains("verify") || joined.contains("audit") || joined.contains("validate")
            {
                HarnessKind::Adversarial
            } else if joined.contains("pipeline")
                || joined.contains("stage")
                || joined.contains("sequential")
            {
                HarnessKind::Pipeline
            } else if joined.contains("classify")
                || joined.contains("route")
                || joined.contains("categorize")
            {
                HarnessKind::ClassifyAndAct
            } else {
                HarnessKind::FanOut
            }
        };
        // FanOut is always registered (inserted in new()), so this branch
        // should never fire in production. Using a static fallback to avoid panic.
        self.get(kind)
            .or_else(|| self.get(HarnessKind::FanOut))
            .unwrap_or_else(|| {
                log::error!(
                    "FanOutHarness not registered - critical invariant violated, using fallback"
                );
                &FALLBACK_HARNESS as &dyn Harness
            })
    }

    pub fn list_kinds(&self) -> Vec<HarnessKind> {
        self.harnesses.keys().copied().collect()
    }
}

impl Default for HarnessRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Harness Resume (snapshot/restore) ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSnapshot {
    pub token: String,
    pub harness_kind: HarnessKind,
    pub plan: LeadAgentPlan,
    pub completed_task_ids: Vec<usize>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeToken {
    pub snapshots: Vec<PlanSnapshot>,
    pub restored_at: String,
}

impl HarnessRegistry {
    pub fn snapshot_all(&self, completed_tasks: &HashMap<usize, bool>) -> Vec<PlanSnapshot> {
        self.harnesses
            .iter()
            .map(|(kind, _harness)| PlanSnapshot {
                token: format!(
                    "snap-{}-{}",
                    kind.name(),
                    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
                ),
                harness_kind: *kind,
                plan: LeadAgentPlan {
                    goal: format!("snapshot:{}", kind.name()),
                    decomposition: TaskDecomposition {
                        sub_tasks: vec![],
                        dependency_graph: vec![],
                    },
                    strategy: kind.name().into(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
                completed_task_ids: completed_tasks
                    .iter()
                    .filter(|(_, &v)| v)
                    .map(|(k, _)| *k)
                    .collect(),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .collect()
    }

    pub fn restore(snapshots: Vec<PlanSnapshot>) -> ResumeToken {
        ResumeToken {
            restored_at: chrono::Utc::now().to_rfc3339(),
            snapshots,
        }
    }

    pub fn resume_from<'a>(&self, token: &'a ResumeToken) -> Vec<&'a PlanSnapshot> {
        token
            .snapshots
            .iter()
            .filter(|s| self.get(s.harness_kind).is_some())
            .collect()
    }
}
