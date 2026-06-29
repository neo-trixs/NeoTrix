use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::sub_agent::{
    LeadAgentPlan, RecoveryStrategy, SubAgentCapability, SubTaskSpec, TaskDecomposition,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewOption {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub approach: String,
    pub estimated_complexity: f64,
    pub estimated_risk: f64,
    pub estimated_tokens: u64,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub plan: LeadAgentPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub goal: String,
    pub options: Vec<PreviewOption>,
    pub recommended: usize,
    pub generated_at: String,
    pub generation_duration_ms: u64,
}

pub struct PreviewEngine;

impl PreviewEngine {
    pub fn generate_options(goal: &str) -> PreviewResult {
        let start = Instant::now();
        let keywords = goal.to_lowercase();

        let mut options = Vec::new();
        if keywords.contains("refactor") || keywords.contains("migrate") {
            options.push(Self::option_incremental(goal));
            options.push(Self::option_full_rewrite(goal));
            options.push(Self::option_strangler(goal));
        } else if keywords.contains("feature") || keywords.contains("implement") {
            options.push(Self::option_minimal(goal));
            options.push(Self::option_robust(goal));
            options.push(Self::option_maximal(goal));
        } else if keywords.contains("review") || keywords.contains("audit") {
            options.push(Self::option_quick_review(goal));
            options.push(Self::option_deep_review(goal));
        } else {
            options.push(Self::option_fast(goal));
            options.push(Self::option_thorough(goal));
        }

        let recommended = 0;
        let elapsed = start.elapsed().as_millis() as u64;

        PreviewResult {
            goal: goal.to_string(),
            options,
            recommended,
            generated_at: format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
            generation_duration_ms: elapsed,
        }
    }

    fn option_incremental(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 0,
            title: "Incremental Refactor".into(),
            description: "Refactor step by step, one module at a time, with tests at each stage"
                .into(),
            approach: "incremental".into(),
            estimated_complexity: 3.0,
            estimated_risk: 2.0,
            estimated_tokens: 50000,
            pros: vec![
                "Low risk".into(),
                "Continuous integration safety".into(),
                "Easy to review".into(),
            ],
            cons: vec!["Takes longer".into(), "Temporary code churn".into()],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Analyze current structure and dependencies",
                        SubAgentCapability::Researcher,
                        vec![],
                    ),
                    (
                        2,
                        "Plan incremental refactor stages",
                        SubAgentCapability::Planner,
                        vec![1],
                    ),
                    (
                        3,
                        "Execute stage 1 refactor",
                        SubAgentCapability::Coder,
                        vec![2],
                    ),
                    (4, "Test stage 1", SubAgentCapability::Tester, vec![3]),
                    (
                        5,
                        "Execute stage 2 refactor",
                        SubAgentCapability::Coder,
                        vec![4],
                    ),
                    (6, "Test stage 2", SubAgentCapability::Tester, vec![5]),
                    (
                        7,
                        "Security audit",
                        SubAgentCapability::SecurityAuditor,
                        vec![6],
                    ),
                    (8, "Update docs", SubAgentCapability::Documenter, vec![6]),
                ],
            ),
        }
    }

    fn option_full_rewrite(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 1,
            title: "Full Rewrite".into(),
            description: "Rewrite the entire module from scratch using modern patterns".into(),
            approach: "rewrite".into(),
            estimated_complexity: 5.0,
            estimated_risk: 5.0,
            estimated_tokens: 200000,
            pros: vec![
                "Clean architecture".into(),
                "Best practices throughout".into(),
                "No legacy debt".into(),
            ],
            cons: vec![
                "Highest risk".into(),
                "Longest duration".into(),
                "Feature parity challenge".into(),
            ],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Analyze existing interface contracts",
                        SubAgentCapability::Researcher,
                        vec![],
                    ),
                    (
                        2,
                        "Design new architecture",
                        SubAgentCapability::Planner,
                        vec![1],
                    ),
                    (
                        3,
                        "Scaffold new module structure",
                        SubAgentCapability::Coder,
                        vec![2],
                    ),
                    (
                        4,
                        "Implement core logic",
                        SubAgentCapability::Coder,
                        vec![3],
                    ),
                    (
                        5,
                        "Write comprehensive tests",
                        SubAgentCapability::Tester,
                        vec![4],
                    ),
                    (
                        6,
                        "Integration testing with existing system",
                        SubAgentCapability::Tester,
                        vec![5],
                    ),
                    (
                        7,
                        "Security review",
                        SubAgentCapability::SecurityAuditor,
                        vec![6],
                    ),
                    (8, "Documentation", SubAgentCapability::Documenter, vec![6]),
                ],
            ),
        }
    }

    fn option_strangler(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 2,
            title: "Strangler Fig Pattern".into(),
            description: "Build new alongside old, route traffic gradually".into(),
            approach: "strangler".into(),
            estimated_complexity: 4.0,
            estimated_risk: 3.0,
            estimated_tokens: 120000,
            pros: vec![
                "Gradual migration".into(),
                "Rollback safe".into(),
                "Can co-exist with old system".into(),
            ],
            cons: vec![
                "More code to maintain temporarily".into(),
                "Complex routing".into(),
            ],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Identify interfaces to strangler",
                        SubAgentCapability::Researcher,
                        vec![],
                    ),
                    (
                        2,
                        "Design facade layer",
                        SubAgentCapability::Planner,
                        vec![1],
                    ),
                    (
                        3,
                        "Build new implementation",
                        SubAgentCapability::Coder,
                        vec![2],
                    ),
                    (4, "Build routing layer", SubAgentCapability::Coder, vec![3]),
                    (
                        5,
                        "Gradual traffic migration",
                        SubAgentCapability::Integrator,
                        vec![4],
                    ),
                    (6, "Verify parity", SubAgentCapability::Tester, vec![5]),
                    (
                        7,
                        "Remove old implementation",
                        SubAgentCapability::Coder,
                        vec![6],
                    ),
                ],
            ),
        }
    }

    fn option_minimal(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 0,
            title: "Minimal Implementation".into(),
            description: "Fastest path to working feature with minimal changes".into(),
            approach: "minimal".into(),
            estimated_complexity: 2.0,
            estimated_risk: 2.0,
            estimated_tokens: 30000,
            pros: vec![
                "Fast delivery".into(),
                "Minimal code changes".into(),
                "Easy to review".into(),
            ],
            cons: vec!["Less flexible".into(), "May need rework later".into()],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Research existing patterns",
                        SubAgentCapability::Researcher,
                        vec![],
                    ),
                    (
                        2,
                        "Implement minimal version",
                        SubAgentCapability::Coder,
                        vec![1],
                    ),
                    (3, "Basic tests", SubAgentCapability::Tester, vec![2]),
                ],
            ),
        }
    }

    fn option_robust(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 1,
            title: "Robust Implementation".into(),
            description: "Well-architected implementation with tests and error handling".into(),
            approach: "robust".into(),
            estimated_complexity: 3.0,
            estimated_risk: 3.0,
            estimated_tokens: 80000,
            pros: vec![
                "Good architecture".into(),
                "Comprehensive error handling".into(),
                "Well tested".into(),
            ],
            cons: vec!["Takes longer".into()],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Research patterns",
                        SubAgentCapability::Researcher,
                        vec![],
                    ),
                    (
                        2,
                        "Design implementation plan",
                        SubAgentCapability::Planner,
                        vec![1],
                    ),
                    (
                        3,
                        "Implement with error handling",
                        SubAgentCapability::Coder,
                        vec![2],
                    ),
                    (
                        4,
                        "Write comprehensive tests",
                        SubAgentCapability::Tester,
                        vec![3],
                    ),
                    (5, "Document", SubAgentCapability::Documenter, vec![3]),
                ],
            ),
        }
    }

    fn option_maximal(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 2,
            title: "Production-Grade Implementation".into(),
            description:
                "Full production-ready implementation with monitoring, observability, and docs"
                    .into(),
            approach: "production".into(),
            estimated_complexity: 5.0,
            estimated_risk: 4.0,
            estimated_tokens: 150000,
            pros: vec![
                "Production ready".into(),
                "Full observability".into(),
                "Comprehensive docs".into(),
            ],
            cons: vec!["Longest timeline".into(), "Most code".into()],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Research and benchmark",
                        SubAgentCapability::Researcher,
                        vec![],
                    ),
                    (
                        2,
                        "Design architecture",
                        SubAgentCapability::Planner,
                        vec![1],
                    ),
                    (3, "Implement core", SubAgentCapability::Coder, vec![2]),
                    (
                        4,
                        "Add observability",
                        SubAgentCapability::InfraOps,
                        vec![3],
                    ),
                    (
                        5,
                        "Implement monitoring",
                        SubAgentCapability::InfraOps,
                        vec![3],
                    ),
                    (6, "Write tests", SubAgentCapability::Tester, vec![3]),
                    (
                        7,
                        "Security audit",
                        SubAgentCapability::SecurityAuditor,
                        vec![3],
                    ),
                    (8, "Documentation", SubAgentCapability::Documenter, vec![3]),
                ],
            ),
        }
    }

    fn option_quick_review(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 0,
            title: "Quick Security Review".into(),
            description: "Focused security and correctness review".into(),
            approach: "quick".into(),
            estimated_complexity: 2.0,
            estimated_risk: 1.0,
            estimated_tokens: 20000,
            pros: vec!["Fast results".into(), "Security focused".into()],
            cons: vec!["Less thorough".into()],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Security audit",
                        SubAgentCapability::SecurityAuditor,
                        vec![],
                    ),
                    (
                        2,
                        "Code quality review",
                        SubAgentCapability::Reviewer,
                        vec![],
                    ),
                ],
            ),
        }
    }

    fn option_deep_review(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 1,
            title: "Deep Multi-Dimensional Review".into(),
            description:
                "Comprehensive review across security, performance, architecture, and testing"
                    .into(),
            approach: "deep".into(),
            estimated_complexity: 3.0,
            estimated_risk: 2.0,
            estimated_tokens: 60000,
            pros: vec!["Most thorough".into(), "Covers all dimensions".into()],
            cons: vec!["Takes longer".into()],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Security audit",
                        SubAgentCapability::SecurityAuditor,
                        vec![],
                    ),
                    (
                        2,
                        "Performance review",
                        SubAgentCapability::Reviewer,
                        vec![],
                    ),
                    (
                        3,
                        "Architecture review",
                        SubAgentCapability::Reviewer,
                        vec![],
                    ),
                    (
                        4,
                        "Test coverage analysis",
                        SubAgentCapability::Tester,
                        vec![],
                    ),
                    (
                        5,
                        "Generate consolidated report",
                        SubAgentCapability::Documenter,
                        vec![1, 2, 3, 4],
                    ),
                ],
            ),
        }
    }

    fn option_fast(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 0,
            title: "Fast Execution".into(),
            description: "Quick implementation with minimal ceremony".into(),
            approach: "fast".into(),
            estimated_complexity: 1.0,
            estimated_risk: 3.0,
            estimated_tokens: 15000,
            pros: vec!["Fastest path".into()],
            cons: vec!["Higher risk".into(), "Less thorough".into()],
            plan: Self::make_plan(goal, vec![(1, goal, SubAgentCapability::Coder, vec![])]),
        }
    }

    fn option_thorough(goal: &str) -> PreviewOption {
        PreviewOption {
            id: 1,
            title: "Thorough Execution".into(),
            description: "Full process with plan, implementation, tests, and documentation".into(),
            approach: "thorough".into(),
            estimated_complexity: 3.0,
            estimated_risk: 2.0,
            estimated_tokens: 60000,
            pros: vec![
                "Comprehensive".into(),
                "Well tested".into(),
                "Documented".into(),
            ],
            cons: vec!["Slower".into()],
            plan: Self::make_plan(
                goal,
                vec![
                    (
                        1,
                        "Research requirements",
                        SubAgentCapability::Researcher,
                        vec![],
                    ),
                    (2, "Plan approach", SubAgentCapability::Planner, vec![1]),
                    (3, "Execute", SubAgentCapability::Coder, vec![2]),
                    (4, "Test", SubAgentCapability::Tester, vec![3]),
                    (5, "Document", SubAgentCapability::Documenter, vec![3]),
                ],
            ),
        }
    }

    fn make_plan(
        goal: &str,
        tasks: Vec<(usize, &str, SubAgentCapability, Vec<usize>)>,
    ) -> LeadAgentPlan {
        let mut deps = Vec::new();
        let sub_tasks: Vec<SubTaskSpec> = tasks
            .into_iter()
            .map(|(id, desc, cap, dep_ids)| {
                for d in dep_ids {
                    deps.push((d, id));
                }
                SubTaskSpec {
                    id,
                    description: desc.to_string(),
                    capability: cap,
                    constraints: vec![],
                    expected_artifacts: vec![],
                    recovery: RecoveryStrategy::default(),
                }
            })
            .collect();

        LeadAgentPlan {
            goal: goal.to_string(),
            decomposition: TaskDecomposition {
                sub_tasks,
                dependency_graph: deps,
            },
            strategy: "preview".into(),
            created_at: format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        }
    }
}
