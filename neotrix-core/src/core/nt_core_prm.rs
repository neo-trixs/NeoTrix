use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_gwt::module_def::SpecialistType;
use crate::core::nt_core_policy::E8Policy;

/// One step in a multi-agent reasoning trajectory.
#[derive(Debug, Clone)]
pub struct TrajectoryStep {
    /// Step index within the episode (0-based).
    pub step_idx: usize,
    /// Which specialist executed this step.
    pub specialist: SpecialistType,
    /// The E8 reasoning mode active during this step.
    pub e8_mode: ReasoningHexagram,
    /// Short description of the action taken.
    pub action: String,
    /// Input context passed to this step.
    pub input: String,
    /// Output/result produced by this step.
    pub output: String,
    /// Duration in milliseconds (if available).
    pub duration_ms: Option<u64>,
    /// Whether the step completed without error.
    pub success: bool,
    /// External reward signal (if any, e.g. test pass/fail).
    pub external_reward: Option<f64>,
}

/// A full multi-step reasoning episode (trajectory).
#[derive(Debug, Clone)]
pub struct AgentTrajectory {
    /// Unique identifier for this trajectory.
    pub trajectory_id: u64,
    /// The task description / prompt.
    pub task: String,
    /// All steps in execution order.
    pub steps: Vec<TrajectoryStep>,
    /// Final outcome reward (e.g. solution quality).
    pub outcome_reward: Option<f64>,
    /// Whether the episode completed successfully.
    pub completed: bool,
    /// Total wall-clock time in ms.
    pub total_duration_ms: Option<u64>,
}

impl AgentTrajectory {
    pub fn new(trajectory_id: u64, task: String) -> Self {
        Self {
            trajectory_id,
            task,
            steps: Vec::new(),
            outcome_reward: None,
            completed: false,
            total_duration_ms: None,
        }
    }

    pub fn push(&mut self, step: TrajectoryStep) {
        self.steps.push(step);
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// A criterion used to score a single step.
#[derive(Debug, Clone)]
pub struct ScoredCriterion {
    /// Criterion name (e.g. "correctness", "efficiency", "clarity").
    pub name: String,
    /// Score in [0.0, 1.0].
    pub score: f64,
    /// Optional justification.
    pub rationale: Option<String>,
}

/// Process reward score for a single step.
#[derive(Debug, Clone)]
pub struct ProcessScore {
    /// Step index this score applies to.
    pub step_idx: usize,
    /// Overall process reward in [0.0, 1.0].
    pub score: f64,
    /// Coach confidence in [0.0, 1.0].
    pub confidence: f64,
    /// Per-criterion breakdown.
    pub criteria: Vec<ScoredCriterion>,
    /// Semantic attribution tags (e.g. "correct_logic", "missing_edge_case").
    pub attribution_tags: Vec<String>,
}

impl ProcessScore {
    pub fn new(step_idx: usize) -> Self {
        Self {
            step_idx,
            score: 0.5,
            confidence: 0.0,
            criteria: Vec::new(),
            attribution_tags: Vec::new(),
        }
    }
}

/// Context passed to the Coach alongside each step.
#[derive(Debug, Clone)]
pub struct CoachContext {
    /// The full trajectory up to (and including) the current step.
    pub trajectory_so_far: Vec<TrajectoryStep>,
    /// Aggregated E8 transition data for the episode.
    pub transition_patterns: Vec<String>,
    /// Whether this is the final step of the trajectory.
    pub is_terminal: bool,
}

impl CoachContext {
    pub fn new(is_terminal: bool) -> Self {
        Self {
            trajectory_so_far: Vec::new(),
            transition_patterns: Vec::new(),
            is_terminal,
        }
    }
}

/// A Coach assigns process rewards by observing trajectory steps.
///
/// This is the core MAPPA abstraction: an LLM-as-judge (or analytic heuristic)
/// that scores each agent action and provides semantic attribution.
pub trait Coach: Send + Sync {
    /// Human-readable name for this coach (e.g. "llm-judge", "heuristic-v1").
    fn name(&self) -> &str;

    /// Score a single trajectory step in context.
    fn score_step(&self, step: &TrajectoryStep, context: &CoachContext) -> ProcessScore;

    /// Score an entire episode trajectory, returning per-step scores.
    fn score_episode(&self, trajectory: &AgentTrajectory) -> Vec<ProcessScore> {
        let terminal = CoachContext {
            trajectory_so_far: trajectory.steps.clone(),
            transition_patterns: Vec::new(),
            is_terminal: true,
        };
        trajectory
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                self.score_step(step, &CoachContext {
                    trajectory_so_far: trajectory.steps[..=i].to_vec(),
                    ..terminal.clone()
                })
            })
            .collect()
    }

    /// Update internal parameters based on trajectory + score feedback.
    fn learn(&mut self, _trajectory: &AgentTrajectory, _scores: &[ProcessScore]) {}
}

/// Collects raw reasoning steps into AgentTrajectories for coaching.
pub struct TrajectoryCollector {
    next_id: u64,
    active: Option<AgentTrajectory>,
    pub collected: Vec<AgentTrajectory>,
}

impl TrajectoryCollector {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            active: None,
            collected: Vec::new(),
        }
    }

    pub fn begin(&mut self, task: String) {
        let id = self.next_id;
        self.next_id += 1;
        self.active = Some(AgentTrajectory::new(id, task));
    }

    pub fn record_step(
        &mut self,
        specialist: SpecialistType,
        e8_mode: ReasoningHexagram,
        action: String,
        input: String,
        output: String,
        duration_ms: Option<u64>,
        success: bool,
        external_reward: Option<f64>,
    ) {
        if let Some(ref mut traj) = self.active {
            let step_idx = traj.steps.len();
            traj.push(TrajectoryStep {
                step_idx,
                specialist,
                e8_mode,
                action,
                input,
                output,
                duration_ms,
                success,
                external_reward,
            });
        }
    }

    pub fn finish(&mut self, outcome_reward: Option<f64>, completed: bool) -> Option<AgentTrajectory> {
        if let Some(mut traj) = self.active.take() {
            traj.outcome_reward = outcome_reward;
            traj.completed = completed;
            let clone = traj.clone();
            self.collected.push(traj);
            Some(clone)
        } else {
            None
        }
    }

    pub fn abort(&mut self) {
        self.active = None;
    }

    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub fn active_task(&self) -> Option<&str> {
        self.active.as_ref().map(|t| t.task.as_str())
    }

    pub fn clear_collected(&mut self) {
        self.collected.clear();
    }

    pub fn latest(&self) -> Option<&AgentTrajectory> {
        self.collected.last()
    }

    pub fn count(&self) -> usize {
        self.collected.len()
    }
}

/// Default heuristic coach — rule-based fallback when LLM-as-judge is unavailable.
///
/// Scores steps based on:
/// - Success/failure (base score)
/// - External reward signal (bonus)
/// - Step duration penalty (very fast or very slow steps)
/// - Trajectory position (later steps in successful episodes get bonus)
pub struct HeuristicCoach {
    pub name_label: String,
    pub success_base: f64,
    pub failure_penalty: f64,
    pub reward_bonus_weight: f64,
    pub duration_penalty_threshold_ms: u64,
}

impl Default for HeuristicCoach {
    fn default() -> Self {
        Self {
            name_label: "heuristic-v1".to_string(),
            success_base: 0.7,
            failure_penalty: 0.4,
            reward_bonus_weight: 0.2,
            duration_penalty_threshold_ms: 30_000,
        }
    }
}

impl HeuristicCoach {
    pub fn new(name: &str) -> Self {
        Self {
            name_label: name.to_string(),
            ..Default::default()
        }
    }
}

impl Coach for HeuristicCoach {
    fn name(&self) -> &str {
        &self.name_label
    }

    fn score_step(&self, step: &TrajectoryStep, context: &CoachContext) -> ProcessScore {
        let mut score = if step.success { self.success_base } else { self.failure_penalty };

        if let Some(ext_r) = step.external_reward {
            score = score + self.reward_bonus_weight * ext_r.max(0.0);
        }

        if let Some(dur) = step.duration_ms {
            if dur > self.duration_penalty_threshold_ms {
                score *= 0.9;
            }
        }

        if context.is_terminal && step.success {
            score = (score + 0.1).min(1.0);
        }

        let mut criteria = Vec::new();
        criteria.push(ScoredCriterion {
            name: "completion".to_string(),
            score: if step.success { 1.0 } else { 0.0 },
            rationale: Some(if step.success { "step completed" } else { "step failed" }.to_string()),
        });

        if let Some(ext_r) = step.external_reward {
            criteria.push(ScoredCriterion {
                name: "external_reward".to_string(),
                score: ext_r.max(0.0).min(1.0),
                rationale: Some(format!("external reward signal: {:.2}", ext_r)),
            });
        }

        let mut tags = Vec::new();
        if step.success {
            tags.push("step_ok".to_string());
        } else {
            tags.push("step_fail".to_string());
        }
        if let Some(dur) = step.duration_ms {
            if dur > self.duration_penalty_threshold_ms {
                tags.push("slow_step".to_string());
            }
        }

        ProcessScore {
            step_idx: step.step_idx,
            score: score.max(0.0).min(1.0),
            confidence: 0.5,
            criteria,
            attribution_tags: tags,
        }
    }
}

/// Lightweight online learner that wraps Coach + Policy + TrajectoryCollector.
///
/// This is the CPU-trainable PRM integration point:
/// 1. Collect trajectories via `TrajectoryCollector`
/// 2. Score them with a `Coach`
/// 3. Learn from scores via `E8Policy::learn_from_scores`
pub struct ProcessRewardLearner {
    pub policy: E8Policy,
    pub coach: Box<dyn Coach>,
    pub collector: TrajectoryCollector,
    pub learning_count: u64,
    pub score_history: Vec<f64>,
}

impl ProcessRewardLearner {
    pub fn new(policy: E8Policy, coach: Box<dyn Coach>) -> Self {
        Self {
            policy,
            coach,
            collector: TrajectoryCollector::new(),
            learning_count: 0,
            score_history: Vec::new(),
        }
    }

    /// Run one learning step: collect, score, learn.
    ///
    /// `collect_fn` should populate `collector` with trajectory steps.
    pub fn learn_step<F>(&mut self, collect_fn: F)
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.collector);

        let trajectories: Vec<AgentTrajectory> = self.collector.collected.drain(..).collect();

        for traj in &trajectories {
            let scores = self.coach.score_episode(traj);
            let avg_score = scores.iter().map(|s| s.score).sum::<f64>() / scores.len().max(1) as f64;
            self.score_history.push(avg_score);

            self.policy.learn_from_scores(traj, &scores);

            for step in &traj.steps {
                if let Some(ext_r) = step.external_reward {
                    let _outcome = crate::core::nt_core_policy::E8Outcome {
                        task: traj.task.clone(),
                        mode: step.e8_mode,
                        reward: ext_r,
                        iteration: self.learning_count,
                    };
                }
            }
        }

        self.learning_count += 1;
    }

    /// Average score from recent learning steps.
    pub fn avg_recent_score(&self, window: usize) -> f64 {
        let window = window.min(self.score_history.len());
        if window == 0 {
            return 0.0;
        }
        self.score_history.iter().rev().take(window).sum::<f64>() / window as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(idx: usize, success: bool) -> TrajectoryStep {
        TrajectoryStep {
            step_idx: idx,
            specialist: SpecialistType::Planner,
            e8_mode: ReasoningHexagram(idx as u8),
            action: "test".into(),
            input: "".into(),
            output: "".into(),
            duration_ms: None,
            success,
            external_reward: None,
        }
    }

    #[test]
    fn test_trajectory_step_defaults() {
        let step = make_step(0, true);
        assert_eq!(step.step_idx, 0);
        assert!(step.success);
    }

    #[test]
    fn test_trajectory_collector_begin_finish() {
        let mut tc = TrajectoryCollector::new();
        assert!(!tc.is_active());
        tc.begin("test task".into());
        assert!(tc.is_active());
        assert_eq!(tc.active_task(), Some("test task"));

        tc.record_step(SpecialistType::Planner, ReasoningHexagram(0),
            "plan".into(), "input".into(), "output".into(), None, true, None);
        tc.record_step(SpecialistType::CodeAnalyzer, ReasoningHexagram(1),
            "code".into(), "input2".into(), "output2".into(), None, true, Some(0.8));

        let traj = tc.finish(Some(1.0), true);
        assert!(traj.is_some());
        let traj = traj.unwrap();
        assert_eq!(traj.steps.len(), 2);
        assert_eq!(traj.task, "test task");
        assert!(traj.completed);
    }

    #[test]
    fn test_heuristic_coach_scores_success_step() {
        let coach = HeuristicCoach::default();
        let step = make_step(0, true);
        let ctx = CoachContext::new(false);
        let score = coach.score_step(&step, &ctx);
        assert_eq!(score.score, coach.success_base);
        assert!(score.attribution_tags.contains(&"step_ok".to_string()));
    }

    #[test]
    fn test_heuristic_coach_scores_failure_step() {
        let coach = HeuristicCoach::default();
        let step = make_step(0, false);
        let ctx = CoachContext::new(false);
        let score = coach.score_step(&step, &ctx);
        assert_eq!(score.score, coach.failure_penalty);
        assert!(score.attribution_tags.contains(&"step_fail".to_string()));
    }

    #[test]
    fn test_heuristic_coach_terminal_bonus() {
        let coach = HeuristicCoach::default();
        let step = make_step(0, true);
        let ctx = CoachContext::new(true);
        let score = coach.score_step(&step, &ctx);
        assert!(score.score > coach.success_base);
    }

    #[test]
    fn test_collector_abort() {
        let mut tc = TrajectoryCollector::new();
        tc.begin("abortable".into());
        tc.record_step(SpecialistType::Planner, ReasoningHexagram(0),
            "action".into(), "in".into(), "out".into(), None, true, None);
        tc.abort();
        assert!(!tc.is_active());
        assert!(tc.latest().is_none());
    }

    #[test]
    fn test_heuristic_coach_external_reward_bonus() {
        let coach = HeuristicCoach::default();
        let step = TrajectoryStep {
            step_idx: 0,
            specialist: SpecialistType::MetaCognitionAnalyst,
            e8_mode: ReasoningHexagram(0),
            action: "verify".into(),
            input: "code".into(),
            output: "pass".into(),
            duration_ms: None,
            success: true,
            external_reward: Some(0.5),
        };
        let ctx = CoachContext::new(false);
        let score = coach.score_step(&step, &ctx);
        assert!(score.score > coach.success_base);
        assert!(score.criteria.iter().any(|c| c.name == "external_reward"));
    }

    #[test]
    fn test_process_reward_learner_end_to_end() {
        let policy = crate::core::nt_core_policy::E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner = ProcessRewardLearner::new(policy, coach);

        learner.learn_step(|collector| {
            collector.begin("test task".into());
            collector.record_step(SpecialistType::Planner, ReasoningHexagram(0),
                "plan".into(), "".into(), "plan_output".into(), None, true, None);
            collector.record_step(SpecialistType::CodeAnalyzer, ReasoningHexagram(1),
                "code".into(), "plan_output".into(), "code_output".into(), None, true, None);
            collector.record_step(SpecialistType::MetaCognitionAnalyst, ReasoningHexagram(2),
                "verify".into(), "code_output".into(), "verified".into(), None, true, Some(1.0));
            collector.finish(Some(1.0), true);
        });

        assert_eq!(learner.learning_count, 1);
        assert!(!learner.score_history.is_empty());
        assert!(learner.avg_recent_score(1) > 0.0);
        // Policy values should have been updated by the learning step
        let total_value: f64 = learner.policy.mode_values.iter().sum();
        assert!(total_value > 0.0, "policy should have learned positive values");
    }

    #[test]
    fn test_trajectory_collector_multiple_collected() {
        let mut tc = TrajectoryCollector::new();
        tc.begin("task1".into());
        tc.record_step(SpecialistType::Planner, ReasoningHexagram(0),
            "plan".into(), "".into(), "out1".into(), None, true, None);
        tc.finish(Some(1.0), true);

        tc.begin("task2".into());
        tc.record_step(SpecialistType::CodeAnalyzer, ReasoningHexagram(1),
            "code".into(), "".into(), "out2".into(), None, true, None);
        tc.finish(Some(0.0), false);

        assert_eq!(tc.count(), 2);
        let latest = tc.latest().unwrap();
        assert_eq!(latest.task, "task2");
        assert!(!latest.completed);
    }
}
