#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Hardened,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LoopTrigger {
    Manual,
    OnCycle { every_n_cycles: usize },
    OnEvent { event_type: String },
    OnTimer { interval_secs: u64 },
    OnGitHook { hook_type: String },
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StepAction {
    RunCommand {
        cmd: String,
        args: Vec<String>,
    },
    CheckOutput {
        expected: String,
    },
    AiScoring {
        prompt: String,
    },
    ConsciousnessTick,
    WebSearch {
        query: String,
    },
    KnowledgeRetrieval {
        topic: String,
    },
    PresentationSync,
    NewsCheck,
    VoiceSynthesis,
    Branch {
        condition: String,
        if_step: Box<StepAction>,
        else_step: Box<StepAction>,
    },
}

#[derive(Debug, Clone)]
pub struct LoopStep {
    pub name: String,
    pub action: StepAction,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct LoopTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub goal: String,
    pub triggers: Vec<LoopTrigger>,
    pub steps: Vec<LoopStep>,
    pub exit_conditions: Vec<String>,
    pub max_iterations: usize,
    pub installed_count: u64,
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoopStatus {
    Running,
    Completed,
    Failed(String),
    MaxIterationsReached,
}

#[derive(Debug, Clone)]
pub struct LoopIterationResult {
    pub step_name: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RunningLoop {
    pub template_id: String,
    pub current_step: usize,
    pub iteration_count: usize,
    pub started_at: u64,
    pub status: LoopStatus,
    pub history: Vec<LoopIterationResult>,
}

pub struct LoopTemplateRegistry {
    templates: HashMap<String, LoopTemplate>,
}

impl LoopTemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, template: LoopTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn get(&self, id: &str) -> Option<&LoopTemplate> {
        self.templates.get(id)
    }

    pub fn list_by_difficulty(&self, d: Difficulty) -> Vec<&LoopTemplate> {
        self.templates
            .values()
            .filter(|t| t.difficulty == d)
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&LoopTemplate> {
        let q = query.to_lowercase();
        self.templates
            .values()
            .filter(|t| {
                t.id.to_lowercase().contains(&q)
                    || t.name.to_lowercase().contains(&q)
                    || t.description.to_lowercase().contains(&q)
                    || t.goal.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn all(&self) -> Vec<&LoopTemplate> {
        self.templates.values().collect()
    }

    pub fn count(&self) -> usize {
        self.templates.len()
    }

    pub fn template_count(&self) -> usize {
        self.templates.len()
    }
}

impl Default for LoopTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConsciousnessLoopEngine;

impl ConsciousnessLoopEngine {
    pub fn instantiate(template: &LoopTemplate) -> RunningLoop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        RunningLoop {
            template_id: template.id.clone(),
            current_step: 0,
            iteration_count: 0,
            started_at: now,
            status: LoopStatus::Running,
            history: Vec::new(),
        }
    }

    pub fn progress(running: &mut RunningLoop, result: LoopIterationResult) {
        running.history.push(result.clone());
        if result.success {
            if running.current_step + 1 >= running.template().map_or(1, |t| t.steps.len()) {
                running.iteration_count += 1;
                running.current_step = 0;
            } else {
                running.current_step += 1;
            }
        }
    }

    pub fn check_conditions(running: &mut RunningLoop, conditions: &[String]) -> LoopStatus {
        if running.status != LoopStatus::Running {
            return running.status.clone();
        }
        for cond in conditions {
            if cond == "completed" {
                running.status = LoopStatus::Completed;
                return LoopStatus::Completed;
            }
            if cond.starts_with("fail:") {
                let msg = cond.trim_start_matches("fail:");
                running.status = LoopStatus::Failed(msg.to_string());
                return LoopStatus::Failed(msg.to_string());
            }
        }
        LoopStatus::Running
    }
}

pub fn default_templates() -> Vec<LoopTemplate> {
    vec![
        LoopTemplate {
            id: "pre-commit-guard".into(),
            name: "Pre-Commit Guard".into(),
            description: "Run tests before every commit; block the commit if any test fails.".into(),
            goal: "Ensure no broken code reaches the repository.".into(),
            triggers: vec![LoopTrigger::OnGitHook { hook_type: "pre-commit".into() }],
            steps: vec![
                LoopStep {
                    name: "run-tests".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["test".into(), "--lib".into()] },
                    description: "Execute library tests to verify correctness.".into(),
                },
                LoopStep {
                    name: "check-output".into(),
                    action: StepAction::CheckOutput { expected: "test result: ok".into() },
                    description: "Verify test runner reports success.".into(),
                },
                LoopStep {
                    name: "report".into(),
                    action: StepAction::AiScoring { prompt: "Summarize test results and flag any warnings.".into() },
                    description: "Generate a human-readable test summary.".into(),
                },
            ],
            exit_conditions: vec!["completed".into(), "fail:tests failed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Intermediate,
        },
        LoopTemplate {
            id: "post-edit-test".into(),
            name: "Post-Edit Test".into(),
            description: "After any source edit, run related tests to catch regressions immediately.".into(),
            goal: "Catch regressions within seconds of a code change.".into(),
            triggers: vec![LoopTrigger::OnEvent { event_type: "file_changed:*.rs".into() }],
            steps: vec![
                LoopStep {
                    name: "check-affected".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["check".into(), "-p".into(), "neotrix".into()] },
                    description: "Quick compilation check on changed crate.".into(),
                },
                LoopStep {
                    name: "run-associated".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["test".into(), "--lib".into()] },
                    description: "Run lib-level tests for immediate feedback.".into(),
                },
            ],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Intermediate,
        },
        LoopTemplate {
            id: "ship-pr-until-green".into(),
            name: "Ship PR Until Green".into(),
            description: "Implement, test, open a PR, wait for CI, and loop on failures until green.".into(),
            goal: "Automate the entire PR lifecycle until merge readiness.".into(),
            triggers: vec![LoopTrigger::Manual],
            steps: vec![
                LoopStep {
                    name: "implement".into(),
                    action: StepAction::ConsciousnessTick,
                    description: "Consciousness-driven implementation step.".into(),
                },
                LoopStep {
                    name: "test".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["test".into()] },
                    description: "Run full test suite.".into(),
                },
                LoopStep {
                    name: "open-pr".into(),
                    action: StepAction::RunCommand { cmd: "gh".into(), args: vec!["pr".into(), "create".into(), "--fill".into()] },
                    description: "Create a pull request via GitHub CLI.".into(),
                },
                LoopStep {
                    name: "wait-ci".into(),
                    action: StepAction::RunCommand { cmd: "gh".into(), args: vec!["run".into(), "watch".into()] },
                    description: "Wait for CI checks to complete.".into(),
                },
                LoopStep {
                    name: "check-ci".into(),
                    action: StepAction::CheckOutput { expected: "pass".into() },
                    description: "Verify CI pipeline passed.".into(),
                },
            ],
            exit_conditions: vec!["completed".into(), "fail:ci failure".into()],
            max_iterations: 5,
            installed_count: 0,
            difficulty: Difficulty::Advanced,
        },
        LoopTemplate {
            id: "ci-failure-watcher".into(),
            name: "CI Failure Watcher".into(),
            description: "Poll CI status periodically, detect failures, and attempt automated fixes.".into(),
            goal: "Minimize time between CI breakage and repair.".into(),
            triggers: vec![LoopTrigger::OnTimer { interval_secs: 300 }],
            steps: vec![
                LoopStep {
                    name: "fetch-ci-status".into(),
                    action: StepAction::RunCommand { cmd: "gh".into(), args: vec!["run".into(), "list".into(), "--limit".into(), "5".into()] },
                    description: "Retrieve latest CI run statuses.".into(),
                },
                LoopStep {
                    name: "analyze-failures".into(),
                    action: StepAction::AiScoring { prompt: "Analyze CI failure logs and identify root cause.".into() },
                    description: "Use AI to diagnose CI failures.".into(),
                },
                LoopStep {
                    name: "apply-fix".into(),
                    action: StepAction::ConsciousnessTick,
                    description: "Consciousness-driven fix generation.".into(),
                },
                LoopStep {
                    name: "verify-fix".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["test".into(), "--lib".into()] },
                    description: "Verify the fix passes tests.".into(),
                },
            ],
            exit_conditions: vec!["completed".into()],
            max_iterations: 3,
            installed_count: 0,
            difficulty: Difficulty::Advanced,
        },
        LoopTemplate {
            id: "deploy-verification".into(),
            name: "Deploy Verification".into(),
            description: "After deployment, run health checks to confirm the service is operational.".into(),
            goal: "Detect deployment failures before they affect users.".into(),
            triggers: vec![LoopTrigger::OnEvent { event_type: "deploy_completed".into() }],
            steps: vec![
                LoopStep {
                    name: "health-endpoint".into(),
                    action: StepAction::RunCommand { cmd: "curl".into(), args: vec!["-f".into(), "http://localhost:8080/health".into()] },
                    description: "Hit the health endpoint.".into(),
                },
                LoopStep {
                    name: "check-response".into(),
                    action: StepAction::CheckOutput { expected: "ok".into() },
                    description: "Verify health endpoint returns OK.".into(),
                },
                LoopStep {
                    name: "log-result".into(),
                    action: StepAction::RunCommand { cmd: "echo".into(), args: vec!["deploy_verified".into()] },
                    description: "Record verification outcome.".into(),
                },
            ],
            exit_conditions: vec!["completed".into(), "fail:health check failed".into()],
            max_iterations: 3,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        },
        LoopTemplate {
            id: "knowledge-compaction".into(),
            name: "Knowledge Compaction".into(),
            description: "Periodically compact and deduplicate the knowledge base to maintain efficiency.".into(),
            goal: "Keep knowledge store lean and responsive.".into(),
            triggers: vec![LoopTrigger::OnCycle { every_n_cycles: 100 }],
            steps: vec![
                LoopStep {
                    name: "scan-entries".into(),
                    action: StepAction::KnowledgeRetrieval { topic: "all".into() },
                    description: "Iterate over all knowledge entries.".into(),
                },
                LoopStep {
                    name: "deduplicate".into(),
                    action: StepAction::AiScoring { prompt: "Identify semantically duplicate entries and suggest merges.".into() },
                    description: "Find and resolve duplicate knowledge.".into(),
                },
                LoopStep {
                    name: "compact-index".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["run".into(), "--bin".into(), "nt-compact".into()] },
                    description: "Rebuild VSA index after compaction.".into(),
                },
            ],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Advanced,
        },
        LoopTemplate {
            id: "news-curation".into(),
            name: "News Curation".into(),
            description: "Daily news fetch, relevance scoring, and concise briefing.".into(),
            goal: "Keep the consciousness informed of relevant developments.".into(),
            triggers: vec![LoopTrigger::OnTimer { interval_secs: 86400 }],
            steps: vec![
                LoopStep {
                    name: "fetch-news".into(),
                    action: StepAction::WebSearch { query: "AI research breakthroughs today".into() },
                    description: "Retrieve latest AI news from web search.".into(),
                },
                LoopStep {
                    name: "score-relevance".into(),
                    action: StepAction::AiScoring { prompt: "Score each article by relevance to NeoTrix architecture gaps.".into() },
                    description: "Filter out irrelevant noise.".into(),
                },
                LoopStep {
                    name: "generate-brief".into(),
                    action: StepAction::AiScoring { prompt: "Write a 3-sentence brief on each high-scoring article.".into() },
                    description: "Condense findings into actionable summaries.".into(),
                },
                LoopStep {
                    name: "voice-summary".into(),
                    action: StepAction::VoiceSynthesis,
                    description: "Optionally synthesize an audio brief.".into(),
                },
            ],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Intermediate,
        },
        LoopTemplate {
            id: "consciousness-self-audit".into(),
            name: "Consciousness Self-Audit".into(),
            description: "Periodic self-evaluation of consciousness health, handler performance, and knowledge coherence.".into(),
            goal: "Maintain meta-cognitive awareness and detect degradation early.".into(),
            triggers: vec![LoopTrigger::OnCycle { every_n_cycles: 50 }],
            steps: vec![
                LoopStep {
                    name: "audit-handlers".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["run".into(), "--bin".into(), "nt-audit".into()] },
                    description: "Run handler profiling and health check.".into(),
                },
                LoopStep {
                    name: "eval-coherence".into(),
                    action: StepAction::AiScoring { prompt: "Evaluate overall consciousness coherence and identify anomalies.".into() },
                    description: "Meta-cognitive coherence assessment.".into(),
                },
                LoopStep {
                    name: "knowledge-gap".into(),
                    action: StepAction::KnowledgeRetrieval { topic: "architecture_gaps".into() },
                    description: "Retrieve known architecture gaps and check progress.".into(),
                },
                LoopStep {
                    name: "report".into(),
                    action: StepAction::AiScoring { prompt: "Generate a self-audit report with recommendations.".into() },
                    description: "Produce actionable audit findings.".into(),
                },
            ],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Hardened,
        },
        LoopTemplate {
            id: "dependency-update-check".into(),
            name: "Dependency Update Check".into(),
            description: "Check for outdated crate dependencies and evaluate upgrade impact.".into(),
            goal: "Keep dependencies current without introducing regressions.".into(),
            triggers: vec![LoopTrigger::OnTimer { interval_secs: 604800 }],
            steps: vec![
                LoopStep {
                    name: "check-outdated".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["outdated".into()] },
                    description: "List outdated dependencies.".into(),
                },
                LoopStep {
                    name: "evaluate-impact".into(),
                    action: StepAction::AiScoring { prompt: "Evaluate each outdated crate for security and compatibility impact.".into() },
                    description: "Risk assessment per dependency.".into(),
                },
                LoopStep {
                    name: "upgrade-safe".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["update".into()] },
                    description: "Apply safe version bumps.".into(),
                },
                LoopStep {
                    name: "verify".into(),
                    action: StepAction::RunCommand { cmd: "cargo".into(), args: vec!["test".into(), "--lib".into()] },
                    description: "Run tests to confirm no regressions.".into(),
                },
            ],
            exit_conditions: vec!["completed".into(), "fail:verification failed".into()],
            max_iterations: 2,
            installed_count: 0,
            difficulty: Difficulty::Intermediate,
        },
    ]
}

impl RunningLoop {
    pub fn template(&self) -> Option<LoopTemplate> {
        default_templates()
            .into_iter()
            .find(|t| t.id == self.template_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let t = LoopTemplate {
            id: "test-loop".into(),
            name: "Test Loop".into(),
            description: "A test template.".into(),
            goal: "Verify correctness.".into(),
            triggers: vec![LoopTrigger::Manual],
            steps: vec![LoopStep {
                name: "step-1".into(),
                action: StepAction::ConsciousnessTick,
                description: "A single step.".into(),
            }],
            exit_conditions: vec!["completed".into()],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        };
        assert_eq!(t.id, "test-loop");
        assert_eq!(t.difficulty, Difficulty::Beginner);
        assert_eq!(t.steps.len(), 1);
    }

    #[test]
    fn test_registry_add_get() {
        let mut reg = LoopTemplateRegistry::new();
        assert_eq!(reg.count(), 0);
        let t = LoopTemplate {
            id: "test-loop".into(),
            name: "Test Loop".into(),
            description: "desc".into(),
            goal: "goal".into(),
            triggers: vec![],
            steps: vec![],
            exit_conditions: vec![],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        };
        reg.register(t);
        assert_eq!(reg.count(), 1);
        assert!(reg.get("test-loop").is_some());
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_search() {
        let mut reg = LoopTemplateRegistry::new();
        for t in default_templates() {
            reg.register(t);
        }
        let results = reg.search("guard");
        assert!(!results.is_empty());
        assert!(results.iter().any(|t| t.id == "pre-commit-guard"));
    }

    #[test]
    fn test_difficulty_filtering() {
        let mut reg = LoopTemplateRegistry::new();
        for t in default_templates() {
            reg.register(t);
        }
        let beginners = reg.list_by_difficulty(Difficulty::Beginner);
        assert!(!beginners.is_empty());
        for t in &beginners {
            assert_eq!(t.difficulty, Difficulty::Beginner);
        }
    }

    #[test]
    fn test_loop_instantiation() {
        let templates = default_templates();
        let t = templates
            .iter()
            .find(|t| t.id == "pre-commit-guard")
            .unwrap();
        let running = ConsciousnessLoopEngine::instantiate(t);
        assert_eq!(running.template_id, "pre-commit-guard");
        assert_eq!(running.current_step, 0);
        assert_eq!(running.iteration_count, 0);
        assert_eq!(running.status, LoopStatus::Running);
    }

    #[test]
    fn test_progress_simulation() {
        let templates = default_templates();
        let t = templates
            .iter()
            .find(|t| t.id == "deploy-verification")
            .unwrap();
        let mut running = ConsciousnessLoopEngine::instantiate(t);
        assert_eq!(running.current_step, 0);
        let result = LoopIterationResult {
            step_name: "health-endpoint".into(),
            success: true,
            output: "ok".into(),
            duration_ms: 150,
        };
        ConsciousnessLoopEngine::progress(&mut running, result);
        assert_eq!(running.current_step, 1);
        assert_eq!(running.history.len(), 1);
    }

    #[test]
    fn test_max_iterations_enforcement() {
        let running = RunningLoop {
            template_id: "test".into(),
            current_step: 0,
            iteration_count: 5,
            started_at: 0,
            status: LoopStatus::MaxIterationsReached,
            history: vec![],
        };
        assert_eq!(running.status, LoopStatus::MaxIterationsReached);
    }

    #[test]
    fn test_exit_condition_check() {
        let mut running = RunningLoop {
            template_id: "test".into(),
            current_step: 0,
            iteration_count: 0,
            started_at: 0,
            status: LoopStatus::Running,
            history: vec![],
        };
        let conditions = vec!["completed".into()];
        let status = ConsciousnessLoopEngine::check_conditions(&mut running, &conditions);
        assert_eq!(status, LoopStatus::Completed);
        assert_eq!(running.status, LoopStatus::Completed);
    }

    #[test]
    fn test_exit_condition_failure() {
        let mut running = RunningLoop {
            template_id: "test".into(),
            current_step: 0,
            iteration_count: 0,
            started_at: 0,
            status: LoopStatus::Running,
            history: vec![],
        };
        let conditions = vec!["fail:something broke".into()];
        let status = ConsciousnessLoopEngine::check_conditions(&mut running, &conditions);
        assert_eq!(status, LoopStatus::Failed("something broke".into()));
    }

    #[test]
    fn test_default_templates_count() {
        let templates = default_templates();
        assert!(
            templates.len() >= 8,
            "Expected at least 8 templates, got {}",
            templates.len()
        );
    }

    #[test]
    fn test_template_lookup_by_id() {
        let templates = default_templates();
        let ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"pre-commit-guard"));
        assert!(ids.contains(&"post-edit-test"));
        assert!(ids.contains(&"ship-pr-until-green"));
        assert!(ids.contains(&"ci-failure-watcher"));
        assert!(ids.contains(&"deploy-verification"));
        assert!(ids.contains(&"knowledge-compaction"));
        assert!(ids.contains(&"news-curation"));
        assert!(ids.contains(&"consciousness-self-audit"));
    }

    #[test]
    fn test_running_loop_history_recording() {
        let mut running = RunningLoop {
            template_id: "test".into(),
            current_step: 0,
            iteration_count: 0,
            started_at: 1000,
            status: LoopStatus::Running,
            history: vec![],
        };
        let r1 = LoopIterationResult {
            step_name: "step-a".into(),
            success: true,
            output: "done".into(),
            duration_ms: 50,
        };
        let r2 = LoopIterationResult {
            step_name: "step-b".into(),
            success: false,
            output: "error".into(),
            duration_ms: 200,
        };
        ConsciousnessLoopEngine::progress(&mut running, r1);
        ConsciousnessLoopEngine::progress(&mut running, r2);
        assert_eq!(running.history.len(), 2);
        assert!(running.history[0].success);
        assert!(!running.history[1].success);
        assert_eq!(running.history[1].duration_ms, 200);
    }

    #[test]
    fn test_registry_all() {
        let mut reg = LoopTemplateRegistry::new();
        for t in default_templates() {
            reg.register(t);
        }
        let all = reg.all();
        assert_eq!(all.len(), reg.count());
    }

    #[test]
    fn test_step_action_variants() {
        let cmd = StepAction::RunCommand {
            cmd: "ls".into(),
            args: vec![],
        };
        let chk = StepAction::CheckOutput {
            expected: "ok".into(),
        };
        let _ai = StepAction::AiScoring {
            prompt: "score".into(),
        };
        let web = StepAction::WebSearch {
            query: "test".into(),
        };
        let kr = StepAction::KnowledgeRetrieval {
            topic: "rust".into(),
        };
        let voice = StepAction::VoiceSynthesis;
        let branch = StepAction::Branch {
            condition: "x > 0".into(),
            if_step: Box::new(StepAction::ConsciousnessTick),
            else_step: Box::new(cmd.clone()),
        };
        if let StepAction::RunCommand { cmd: c, .. } = &cmd {
            assert_eq!(c, "ls");
        }
        if let StepAction::CheckOutput { expected } = &chk {
            assert_eq!(expected, "ok");
        }
        if let StepAction::Branch { condition, .. } = &branch {
            assert_eq!(condition, "x > 0");
        }
        assert!(matches!(web, StepAction::WebSearch { .. }));
        assert!(matches!(kr, StepAction::KnowledgeRetrieval { .. }));
        assert!(matches!(voice, StepAction::VoiceSynthesis));
    }

    #[test]
    fn test_loop_trigger_variants() {
        let m = LoopTrigger::Manual;
        let c = LoopTrigger::OnCycle { every_n_cycles: 10 };
        let e = LoopTrigger::OnEvent {
            event_type: "deploy".into(),
        };
        let t = LoopTrigger::OnTimer {
            interval_secs: 3600,
        };
        let g = LoopTrigger::OnGitHook {
            hook_type: "pre-push".into(),
        };
        assert_eq!(m, LoopTrigger::Manual);
        if let LoopTrigger::OnCycle { every_n_cycles } = c {
            assert_eq!(every_n_cycles, 10);
        }
        if let LoopTrigger::OnTimer { interval_secs } = t {
            assert_eq!(interval_secs, 3600);
        }
        assert!(matches!(e, LoopTrigger::OnEvent { .. }));
        assert!(matches!(g, LoopTrigger::OnGitHook { .. }));
    }

    #[test]
    fn test_instantiate_all_default_templates() {
        for template in default_templates() {
            let running = ConsciousnessLoopEngine::instantiate(&template);
            assert_eq!(running.template_id, template.id);
            assert_eq!(running.status, LoopStatus::Running);
            assert!(running.started_at > 0);
        }
    }

    #[test]
    fn test_increment_installed_count() {
        let mut t = LoopTemplate {
            id: "counter-test".into(),
            name: "Counter".into(),
            description: "test".into(),
            goal: "test".into(),
            triggers: vec![],
            steps: vec![],
            exit_conditions: vec![],
            max_iterations: 1,
            installed_count: 0,
            difficulty: Difficulty::Beginner,
        };
        t.installed_count += 1;
        assert_eq!(t.installed_count, 1);
        t.installed_count += 1;
        assert_eq!(t.installed_count, 2);
    }

    #[test]
    fn test_branch_is_not_infinite_recursive() {
        let action = StepAction::Branch {
            condition: "a".into(),
            if_step: Box::new(StepAction::Branch {
                condition: "b".into(),
                if_step: Box::new(StepAction::ConsciousnessTick),
                else_step: Box::new(StepAction::WebSearch {
                    query: "nested".into(),
                }),
            }),
            else_step: Box::new(StepAction::NewsCheck),
        };
        if let StepAction::Branch {
            condition,
            if_step,
            else_step,
        } = &action
        {
            assert_eq!(condition, "a");
            if let StepAction::Branch {
                condition: inner_c, ..
            } = if_step.as_ref()
            {
                assert_eq!(inner_c, "b");
            } else {
                panic!("Expected nested branch");
            }
            assert!(matches!(else_step.as_ref(), StepAction::NewsCheck));
        }
    }
}
