use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Data structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposedInstruction {
    pub id: String,
    pub description: String,
    pub priority: u8,
    pub dependencies: Vec<String>,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub content: String,
    pub instruction_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub artifacts: Vec<Artifact>,
    pub success: bool,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critique {
    pub id: String,
    pub artifact_id: String,
    pub rating: u8,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopDecision {
    pub should_continue: bool,
    pub max_iterations: u8,
    pub next_instruction: Option<String>,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialSnapshot {
    pub trial_id: String,
    pub iterations: Vec<IterationSnapshot>,
    pub final_score: f64,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationSnapshot {
    pub instruction: String,
    pub critique_score: u8,
    pub artifact_summary: String,
}

// ---------------------------------------------------------------------------
// Agent structs
// ---------------------------------------------------------------------------

pub struct MetaAgent {
    pub name: String,
    pub decomposition_count: u64,
}

pub struct TargetAgent {
    pub name: String,
    pub execution_count: u64,
}

pub struct FeedbackAgent {
    pub name: String,
    pub review_count: u64,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SiaConfig {
    pub max_iterations: u8,
    pub min_rating_to_pass: u8,
    pub feedback_depth: u8,
    pub require_critique: bool,
}

impl Default for SiaConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            min_rating_to_pass: 7,
            feedback_depth: 3,
            require_critique: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

pub struct SiaLoop {
    pub meta_agent: MetaAgent,
    pub target_agent: TargetAgent,
    pub feedback_agent: FeedbackAgent,
    pub max_loop_iterations: u8,
    pub trial_history: Vec<TrialSnapshot>,
    pub config: SiaConfig,
}

impl SiaLoop {
    pub fn new(config: SiaConfig) -> Self {
        Self {
            meta_agent: MetaAgent {
                name: "SIA-Meta".into(),
                decomposition_count: 0,
            },
            target_agent: TargetAgent {
                name: "SIA-Target".into(),
                execution_count: 0,
            },
            feedback_agent: FeedbackAgent {
                name: "SIA-Feedback".into(),
                review_count: 0,
            },
            max_loop_iterations: config.max_iterations,
            trial_history: Vec::new(),
            config,
        }
    }

    pub fn run_trial(&mut self, task: &str) -> TrialSnapshot {
        let start = now_ms();
        let trial_id = Uuid::new_v4().to_string();
        let instructions = self.meta_agent.decompose_task(task);
        let mut iterations = Vec::new();
        let mut current_instruction = instructions
            .first()
            .map(|i| i.description.clone())
            .unwrap_or_else(|| task.to_string());
        let mut overall_score = 0.0_f64;

        for _ in 0..self.config.max_iterations {
            let iter = self.execute_single_iteration(&current_instruction);
            overall_score = iter.critique_score as f64;
            iterations.push(iter);

            let instruction_for_eval = iterations.last().map(|i| i.instruction.as_str()).unwrap_or("");
            let artifact = Artifact {
                id: Uuid::new_v4().to_string(),
                content: instruction_for_eval.to_string(),
                instruction_id: String::new(),
                timestamp: now_ms() as i64 / 1000,
            };
            let critique = Critique {
                id: Uuid::new_v4().to_string(),
                artifact_id: artifact.id.clone(),
                rating: iterations.last().map(|i| i.critique_score).unwrap_or(0),
                issues: Vec::new(),
                suggestions: Vec::new(),
                summary: format!("Rating: {}", iterations.last().map(|i| i.critique_score).unwrap_or(0)),
            };
            let decision = self.meta_agent.evaluate_iteration(&artifact, &critique);

            if !decision.should_continue || overall_score >= self.config.min_rating_to_pass as f64 {
                break;
            }
            if let Some(next) = decision.next_instruction {
                current_instruction = next;
            }
        }

        let total_duration = now_ms() - start;
        let snapshot = TrialSnapshot {
            trial_id,
            iterations,
            final_score: overall_score,
            total_duration_ms: total_duration,
        };
        self.trial_history.push(snapshot.clone());
        snapshot
    }

    pub fn run_trial_with_guard(
        &mut self,
        task: &str,
        guard: Option<&mut super::guard::GuardConfig>,
    ) -> TrialSnapshot {
        if let Some(ref g) = guard {
            if g.enabled {
                let initial = 1.0;
                if let super::guard::GuardResult::Violation { .. } = super::guard::check_guard_violation(g, initial, initial) {
                    return TrialSnapshot {
                        trial_id: Uuid::new_v4().to_string(),
                        iterations: Vec::new(),
                        final_score: 0.0,
                        total_duration_ms: 0,
                    };
                }
            }
        }
        self.run_trial(task)
    }

    pub fn execute_single_iteration(&mut self, instruction: &str) -> IterationSnapshot {
        let start = now_ms();

        let decomposed = self.meta_agent.decompose_task(instruction);
        let primary = decomposed
            .first()
            .cloned()
            .unwrap_or(DecomposedInstruction {
                id: Uuid::new_v4().to_string(),
                description: instruction.to_string(),
                priority: 5,
                dependencies: Vec::new(),
                expected_outcome: "completed".into(),
            });

        let report = self.target_agent.execute_instruction(&primary);
        let artifact = report
            .artifacts
            .first()
            .cloned()
            .unwrap_or(Artifact {
                id: Uuid::new_v4().to_string(),
                content: instruction.to_string(),
                instruction_id: primary.id.clone(),
                timestamp: (now_ms() / 1000) as i64,
            });

        let critique = self.feedback_agent.review_artifact(&artifact);
        let summary = if report.artifacts.len() > 1 {
            format!(
                "{} artifacts produced, rating {}",
                report.artifacts.len(),
                critique.rating
            )
        } else {
            let preview: String = artifact.content.chars().take(80).collect();
            format!("`{}` — rating {}", preview, critique.rating)
        };

        let _duration = now_ms() - start;
        IterationSnapshot {
            instruction: instruction.to_string(),
            critique_score: critique.rating,
            artifact_summary: summary,
        }
    }

    pub fn trial_history_summary(&self) -> String {
        if self.trial_history.is_empty() {
            return "No trials completed.".to_string();
        }
        let mut lines = Vec::new();
        for (i, t) in self.trial_history.iter().enumerate() {
            let iter_count = t.iterations.len();
            lines.push(format!(
                "Trial #{} | id={} | iterations={} | final_score={:.1} | duration={}ms",
                i + 1,
                &t.trial_id[..8],
                iter_count,
                t.final_score,
                t.total_duration_ms,
            ));
        }
        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Agent method impls
// ---------------------------------------------------------------------------

impl MetaAgent {
    pub fn decompose_task(&mut self, task: &str) -> Vec<DecomposedInstruction> {
        self.decomposition_count += 1;
        task.split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .enumerate()
            .map(|(i, sentence)| DecomposedInstruction {
                id: format!("inst-{}", i + 1),
                description: sentence.to_string(),
                priority: (10 - i as u8).min(10),
                dependencies: if i > 0 {
                    vec![format!("inst-{}", i)]
                } else {
                    Vec::new()
                },
                expected_outcome: format!("Complete step {}", i + 1),
            })
            .collect()
    }

    pub fn evaluate_iteration(
        &self,
        _artifact: &Artifact,
        critique: &Critique,
    ) -> LoopDecision {
        let rating = critique.rating;
        let should_continue = rating < 7;
        let next_instruction = if should_continue {
            if let Some(s) = critique.suggestions.first() {
                Some(format!("Improve: {}", s))
            } else {
                Some("Revisit and improve the previous attempt.".to_string())
            }
        } else {
            None
        };
        LoopDecision {
            should_continue,
            max_iterations: 10,
            next_instruction,
            overall_score: rating as f64,
        }
    }
}

impl TargetAgent {
    pub fn execute_instruction(&mut self, instruction: &DecomposedInstruction) -> ExecutionReport {
        self.execution_count += 1;
        let ts = (now_ms() / 1000) as i64;
        let artifact = Artifact {
            id: Uuid::new_v4().to_string(),
            content: instruction.description.clone(),
            instruction_id: instruction.id.clone(),
            timestamp: ts,
        };
        ExecutionReport {
            artifacts: vec![artifact],
            success: true,
            errors: Vec::new(),
            duration_ms: 10,
        }
    }
}

impl FeedbackAgent {
    pub fn review_artifact(&mut self, artifact: &Artifact) -> Critique {
        self.review_count += 1;
        let content = &artifact.content;
        let len = content.len();
        let keyword_count = [
            "implement", "create", "build", "design", "optimize",
            "refactor", "test", "deploy", "analyze", "integrate",
        ]
        .iter()
        .filter(|kw| content.to_lowercase().contains(*kw))
        .count();

        let rating = if len > 200 && keyword_count >= 3 {
            9
        } else if len > 100 && keyword_count >= 2 {
            7
        } else if len > 50 && keyword_count >= 1 {
            5
        } else if len > 20 {
            3
        } else {
            1
        };

        let mut issues = Vec::new();
        if len < 50 {
            issues.push("Content too short; lacks depth".to_string());
        }
        if keyword_count < 2 {
            issues.push("Few actionable keywords detected".to_string());
        }

        let mut suggestions = Vec::new();
        if len < 100 {
            suggestions.push("Expand the instruction with more detail and context".to_string());
        }
        if keyword_count < 3 {
            suggestions.push("Add specific action verbs (implement, create, build, etc.)".to_string());
        }

        Critique {
            id: Uuid::new_v4().to_string(),
            artifact_id: artifact.id.clone(),
            rating,
            issues,
            suggestions,
            summary: format!("Rating {}/10 — {} keywords across {} chars", rating, keyword_count, len),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sia_loop_new() {
        let config = SiaConfig::default();
        let loop_ = SiaLoop::new(config);
        assert_eq!(loop_.meta_agent.name, "SIA-Meta");
        assert_eq!(loop_.target_agent.name, "SIA-Target");
        assert_eq!(loop_.feedback_agent.name, "SIA-Feedback");
        assert_eq!(loop_.config.max_iterations, 5);
        assert!(loop_.trial_history.is_empty());
        assert_eq!(loop_.meta_agent.decomposition_count, 0);
        assert_eq!(loop_.target_agent.execution_count, 0);
        assert_eq!(loop_.feedback_agent.review_count, 0);
    }

    #[test]
    fn test_meta_agent_decompose_basic() {
        let mut meta = MetaAgent {
            name: "test".into(),
            decomposition_count: 0,
        };
        let instructions = meta.decompose_task("Build the API. Add authentication. Write tests.");
        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0].description, "Build the API");
        assert_eq!(instructions[1].description, "Add authentication");
        assert_eq!(instructions[2].description, "Write tests");
        assert_eq!(instructions[0].dependencies.len(), 0);
        assert_eq!(instructions[1].dependencies[0], "inst-1");
        assert_eq!(meta.decomposition_count, 1);
    }

    #[test]
    fn test_target_execute_creates_artifact() {
        let mut target = TargetAgent {
            name: "test-target".into(),
            execution_count: 0,
        };
        let instruction = DecomposedInstruction {
            id: "test-1".into(),
            description: "Write unit tests for the SIA loop".into(),
            priority: 8,
            dependencies: Vec::new(),
            expected_outcome: "All tests pass".into(),
        };
        let report = target.execute_instruction(&instruction);
        assert!(report.success);
        assert_eq!(report.artifacts.len(), 1);
        assert_eq!(report.artifacts[0].content, "Write unit tests for the SIA loop");
        assert_eq!(report.artifacts[0].instruction_id, "test-1");
        assert_eq!(target.execution_count, 1);
    }

    #[test]
    fn test_feedback_agent_rates_content() {
        let mut fb = FeedbackAgent {
            name: "test-fb".into(),
            review_count: 0,
        };
        let long_artifact = Artifact {
            id: "a1".into(),
            content: "Implement the core loop with MetaAgent decomposition and TargetAgent execution and FeedbackAgent review. Build the full pipeline and integrate all three agents.".into(),
            instruction_id: "i1".into(),
            timestamp: 0,
        };
        let critique = fb.review_artifact(&long_artifact);
        assert!(critique.rating >= 7);
        assert!(critique.summary.contains("Rating"));
        assert_eq!(fb.review_count, 1);
    }

    #[test]
    fn test_meta_agent_evaluate_stops_on_high_rating() {
        let meta = MetaAgent {
            name: "test".into(),
            decomposition_count: 0,
        };
        let artifact = Artifact {
            id: "a1".into(),
            content: "done".into(),
            instruction_id: "i1".into(),
            timestamp: 0,
        };
        let critique = Critique {
            id: "c1".into(),
            artifact_id: "a1".into(),
            rating: 9,
            issues: Vec::new(),
            suggestions: Vec::new(),
            summary: "Great".into(),
        };
        let decision = meta.evaluate_iteration(&artifact, &critique);
        assert!(!decision.should_continue);
        assert!(decision.next_instruction.is_none());
        assert!((decision.overall_score - 9.0).abs() < 1e-9);
    }

    #[test]
    fn test_meta_agent_evaluate_continues_on_low_rating() {
        let meta = MetaAgent {
            name: "test".into(),
            decomposition_count: 0,
        };
        let artifact = Artifact {
            id: "a1".into(),
            content: "bad".into(),
            instruction_id: "i1".into(),
            timestamp: 0,
        };
        let critique = Critique {
            id: "c1".into(),
            artifact_id: "a1".into(),
            rating: 3,
            issues: vec!["Too short".into()],
            suggestions: vec!["Add more detail".into()],
            summary: "Needs work".into(),
        };
        let decision = meta.evaluate_iteration(&artifact, &critique);
        assert!(decision.should_continue);
        assert!(decision.next_instruction.is_some());
    }

    #[test]
    fn test_run_trial_completes() {
        let config = SiaConfig {
            max_iterations: 3,
            min_rating_to_pass: 7,
            feedback_depth: 2,
            require_critique: true,
        };
        let mut loop_ = SiaLoop::new(config);
        let snapshot = loop_.run_trial("Build a web API. Add authentication middleware.");
        assert!(!snapshot.trial_id.is_empty());
        assert!(snapshot.iterations.len() <= 3);
        assert!(snapshot.iterations.len() >= 1);
        assert_eq!(loop_.trial_history.len(), 1);
    }

    #[test]
    fn test_trial_history_summary() {
        let mut loop_ = SiaLoop::new(SiaConfig::default());
        let empty_summary = loop_.trial_history_summary();
        assert_eq!(empty_summary, "No trials completed.");

        loop_.run_trial("Build feature A. Test feature A.");
        let summary = loop_.trial_history_summary();
        assert!(summary.contains("Trial #1"));
        assert!(summary.contains("iterations="));
        assert!(summary.contains("duration="));
    }

    #[test]
    fn test_guard_violation_returns_early() {
        let config = SiaConfig::default();
        let mut loop_ = SiaLoop::new(config);
        let mut guard = crate::neotrix::nt_act_spear::guard::GuardConfig {
            enabled: true,
            metric_name: "accuracy".into(),
            floor: 1.5,
            relative_floor: false,
            penalty_on_violation: 0.5,
        };
        let snapshot = loop_.run_trial_with_guard("test task", Some(&mut guard));
        assert!(snapshot.iterations.is_empty());
        assert!((snapshot.final_score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_decomposed_instruction_serde_roundtrip() {
        let inst = DecomposedInstruction {
            id: "inst-1".into(),
            description: "Build the core".into(),
            priority: 9,
            dependencies: vec![],
            expected_outcome: "Core built".into(),
        };
        let json = serde_json::to_string(&inst).expect("value should be ok in test");
        let deserialized: DecomposedInstruction = serde_json::from_str(&json).expect("value should be ok in test");
        assert_eq!(deserialized.id, "inst-1");
        assert_eq!(deserialized.description, "Build the core");
        assert_eq!(deserialized.priority, 9);
    }

    #[test]
    fn test_feedback_rates_short_content_low() {
        let mut fb = FeedbackAgent {
            name: "fb".into(),
            review_count: 0,
        };
        let short = Artifact {
            id: "a1".into(),
            content: "do it".into(),
            instruction_id: "i1".into(),
            timestamp: 0,
        };
        let critique = fb.review_artifact(&short);
        assert!(critique.rating <= 3);
        assert!(critique.issues.iter().any(|i| i.contains("short")));
    }

    #[test]
    fn test_execute_single_iteration_increments_counts() {
        let mut loop_ = SiaLoop::new(SiaConfig::default());
        let d0 = loop_.meta_agent.decomposition_count;
        let e0 = loop_.target_agent.execution_count;
        let r0 = loop_.feedback_agent.review_count;

        let snapshot = loop_.execute_single_iteration("Refactor the database layer for performance.");

        assert!(loop_.meta_agent.decomposition_count > d0);
        assert!(loop_.target_agent.execution_count > e0);
        assert!(loop_.feedback_agent.review_count > r0);
        assert!(snapshot.critique_score > 0);
        assert!(!snapshot.artifact_summary.is_empty());
    }
}
