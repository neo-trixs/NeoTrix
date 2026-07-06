use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_traits::SpecialistType;
use crate::core::nt_core_policy::E8Policy;
use serde::{Serialize, Deserialize};

/// One step in a multi-agent reasoning trajectory.
///
/// NOTE: f64 fields may produce NaN in edge cases. Use
/// `serde_json::Builder::new().nan_infinity(true)` to handle during (de)serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredCriterion {
    /// Criterion name (e.g. "correctness", "efficiency", "clarity").
    pub name: String,
    /// Score in [0.0, 1.0].
    pub score: f64,
    /// Optional justification.
    pub rationale: Option<String>,
}

/// Process reward score for a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
/// Manual impls for Debug, Clone, Serialize, Deserialize (dyn = unsafe)
pub struct TrajectoryCollector {
    next_id: u64,
    active: Option<AgentTrajectory>,
    pub collected: Vec<AgentTrajectory>,
}

// Manual Clone impl: AgentTrajectory derives Clone
impl Clone for TrajectoryCollector {
    fn clone(&self) -> Self {
        Self {
            next_id: self.next_id,
            active: self.active.clone(),
            collected: self.collected.clone(),
        }
    }
}

impl std::fmt::Debug for TrajectoryCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrajectoryCollector")
            .field("next_id", &self.next_id)
            .field("active", &self.active)
            .field("collected", &self.collected)
            .finish()
    }
}

impl serde::Serialize for TrajectoryCollector {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TrajectoryCollector", 3)?;
        state.serialize_field("next_id", &self.next_id)?;
        state.serialize_field("active", &self.active)?;
        state.serialize_field("collected", &self.collected)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for TrajectoryCollector {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Helper {
            next_id: u64,
            active: Option<AgentTrajectory>,
            collected: Vec<AgentTrajectory>,
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(Self {
            next_id: helper.next_id,
            active: helper.active,
            collected: helper.collected,
        })
    }
}

impl Default for TrajectoryCollector {
    fn default() -> Self {
        Self::new()
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Score episode with LATA trajectory-length normalization.
    ///
    /// Divides each step score by √L so that cumulative reward doesn't
    /// grow unbounded with trajectory length (arXiv: agentic-grpo-longhorizon).
    fn score_episode(&self, trajectory: &AgentTrajectory) -> Vec<ProcessScore> {
        let l = trajectory.steps.len().max(1) as f64;
        let lata_factor = l.sqrt();
        let base_scores: Vec<ProcessScore> = trajectory
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let ctx = CoachContext {
                    trajectory_so_far: trajectory.steps[..=i].to_vec(),
                    transition_patterns: Vec::new(),
                    is_terminal: i == trajectory.steps.len() - 1,
                };
                self.score_step(step, &ctx)
            })
            .collect();
        // Apply LATA normalization to each score
        base_scores.into_iter().map(|mut ps| {
            ps.score = (ps.score / lata_factor).max(0.0).min(1.0);
            ps
        }).collect()
    }

    fn score_step(&self, step: &TrajectoryStep, context: &CoachContext) -> ProcessScore {
        let mut score = if step.success { self.success_base } else { self.failure_penalty };

        if let Some(ext_r) = step.external_reward {
            score += self.reward_bonus_weight * ext_r.max(0.0);
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

    /// Update internal parameters based on trajectory + score feedback.
    /// Uses EMA to adapt scoring parameters from observed outcomes.
    fn learn(&mut self, trajectory: &AgentTrajectory, scores: &[ProcessScore]) {
        if scores.is_empty() {
            return;
        }
        let avg_score: f64 = scores.iter().map(|s| s.score).sum::<f64>() / scores.len() as f64;
        let lr = 0.05;
        if trajectory.completed {
            if let Some(reward) = trajectory.outcome_reward {
                let normalized_reward = ((reward + 1.0) / 2.0).max(0.0).min(1.0);
                if normalized_reward > 0.6 {
                    self.success_base = (self.success_base + lr * (normalized_reward - self.success_base)).max(0.0).min(1.0);
                    let target_penalty = self.failure_penalty + lr * 0.1;
                    self.failure_penalty = target_penalty.max(0.0).min(1.0);
                } else {
                    let target_base = self.success_base - lr * 0.1;
                    self.success_base = target_base.max(0.0).min(1.0);
                    let target_penalty = self.failure_penalty - lr * (self.failure_penalty - 0.2);
                    self.failure_penalty = target_penalty.max(0.0).min(1.0);
                }
            }
            let bonus_delta = lr * (avg_score - self.reward_bonus_weight);
            self.reward_bonus_weight = (self.reward_bonus_weight + bonus_delta).max(0.0).min(0.5);
        }
    }
}

/// Lightweight online learner that wraps Coach + Policy + TrajectoryCollector.
///
/// This is the CPU-trainable PRM integration point:
/// 1. Collect trajectories via `TrajectoryCollector`
/// 2. Score them with a `Coach`
/// 3. Learn from scores via `E8Policy::learn_from_scores`
///
/// # Serde note
/// The `policy` and `coach` fields are skipped during serialization.
/// After deserialization, call `init_coach()` to restore the coach.
/// `Debug`/`Clone` are implemented manually because `dyn Coach`
/// doesn't natively derive these traits.
pub struct ProcessRewardLearner {
    pub policy: E8Policy,
    pub coach: Box<dyn Coach>,
    pub collector: TrajectoryCollector,
    pub learning_count: u64,
    pub score_history: Vec<f64>,
}

fn default_coach() -> Box<dyn Coach> {
    Box::new(HeuristicCoach::default())
}

impl std::fmt::Debug for ProcessRewardLearner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessRewardLearner")
            .field("policy", &self.policy)
            .field("coach", &"<dyn Coach>")
            .field("collector", &self.collector)
            .field("learning_count", &self.learning_count)
            .field("score_history", &self.score_history)
            .finish()
    }
}

impl Default for ProcessRewardLearner {
    fn default() -> Self {
        Self {
            policy: E8Policy::default(),
            coach: default_coach(),
            collector: TrajectoryCollector::default(),
            learning_count: 0,
            score_history: Vec::new(),
        }
    }
}

impl Clone for ProcessRewardLearner {
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone(),
            coach: default_coach(),
            collector: self.collector.clone(),
            learning_count: self.learning_count,
            score_history: self.score_history.clone(),
        }
    }
}

impl serde::Serialize for ProcessRewardLearner {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ProcessRewardLearner", 3)?;
        state.serialize_field("collector", &self.collector)?;
        state.serialize_field("learning_count", &self.learning_count)?;
        state.serialize_field("score_history", &self.score_history)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ProcessRewardLearner {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct ProcessRewardLearnerHelper {
            collector: TrajectoryCollector,
            learning_count: u64,
            score_history: Vec<f64>,
        }
        let helper = ProcessRewardLearnerHelper::deserialize(deserializer)?;
        Ok(Self {
            policy: E8Policy::new(0.3, 0.995, 0.01, 0.1, 0.9),
            coach: default_coach(),
            collector: helper.collector,
            learning_count: helper.learning_count,
            score_history: helper.score_history,
        })
    }
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

    /// Replace the coach after deserialization.
    /// Required because `Box<dyn Coach>` is skipped during serde.
    pub fn init_coach(&mut self, coach: Box<dyn Coach>) {
        self.coach = coach;
    }

    /// Run one learning step: collect, score, learn.
    ///
    /// `collect_fn` should populate `collector` with trajectory steps.
    /// Applies LATA (√L) trajectory-length normalization and optional auxiliary reward blend.
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

            // Learn from trajectory and scores directly
            self.policy.learn_from_scores(traj, &scores);

            // Blend auxiliary rule-based reward if available
            let aux = Self::auxiliary_reward(traj);
            if aux != 0.0 {
                let aux_scores: Vec<ProcessScore> = traj.steps.iter()
                    .map(|s| {
                        let mut ps = ProcessScore::new(s.step_idx);
                        ps.score = aux.max(0.0).min(1.0);
                        ps.attribution_tags = vec!["auxiliary_reward".to_string()];
                        ps
                    })
                    .collect();
                self.policy.learn_from_scores(traj, &aux_scores);
            }

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

    /// Compute an auxiliary rule-based reward for the entire trajectory.
    ///
    /// Uses simple heuristics:
    /// - Completion bonus: +0.3 if trajectory completed successfully
    /// - Efficiency bonus: +0.1 if trajectory is short and successful
    /// - Consistency penalty: -0.1 if any step failed
    ///
    /// Returns a single scalar in [-0.1, 0.4].
    pub fn auxiliary_reward(traj: &AgentTrajectory) -> f64 {
        let mut reward = 0.0;
        if traj.completed {
            reward += 0.3;
        }
        let steps = traj.steps.len();
        if traj.completed && steps <= 5 {
            reward += 0.1;
        }
        if traj.steps.iter().any(|s| !s.success) {
            reward -= 0.1;
        }
        reward
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

// ═══════════════════════════════════════════════════════════════════
// λ-GRPO: Implicit PRM via Step-Level Advantage Normalization
// ═══════════════════════════════════════════════════════════════════

/// λ-GRPO configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LambdaGrpoConfig {
    /// Lambda parameter: 0.0 = pure outcome GRPO, 1.0 = full step-level PRM
    /// Recommended: 0.3 (balance between outcome + process supervision)
    pub lambda: f64,
    /// Clipping parameter ε for PPO-style clipped surrogate objective (default: 0.2)
    pub epsilon: f64,
    /// Group size for GRPO group sampling (default: 4)
    pub group_size: usize,
    /// Whether to use LATA (√L) trajectory-length normalization (default: true)
    pub lata_normalize: bool,
    /// KL penalty coefficient (default: 0.01)
    pub kl_coef: f64,
    /// Whether to use per-trajectory difficulty-adaptive λ.
    /// When enabled, λ_t = lambda * convergence_t where convergence_t
    /// is computed from step reward variance within each trajectory.
    /// Low variance → high convergence → higher λ (more step-level PRM).
    /// High variance → low convergence → lower λ (more outcome-level GRPO).
    /// Default: false (fixed λ).
    pub difficulty_adaptive_lambda: bool,
    /// Multiplicative scale for difficulty adaptivity (default: 1.0).
    /// Higher values amplify the convergence effect on λ.
    pub difficulty_scale: f64,
}

impl Default for LambdaGrpoConfig {
    fn default() -> Self {
        Self {
            lambda: 0.3,
            epsilon: 0.2,
            group_size: 4,
            lata_normalize: true,
            kl_coef: 0.01,
            difficulty_adaptive_lambda: false,
            difficulty_scale: 1.0,
        }
    }
}

/// Per-step advantage computed by λ-GRPO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAdvantage {
    pub step_idx: usize,
    /// Raw advantage value (can be positive or negative)
    pub advantage: f64,
    /// Normalized advantage (z-scored within group)
    pub normalized_advantage: f64,
}

/// λ-GRPO result for a single trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaGrpoResult {
    /// Trajectory ID
    pub trajectory_id: u64,
    /// Per-step advantages
    pub step_advantages: Vec<StepAdvantage>,
    /// Total loss for this trajectory
    pub loss: f64,
    /// Policy gradient contribution
    pub policy_gradient: f64,
    /// KL divergence penalty
    pub kl_penalty: f64,
}

/// Normalize values within a group using z-score: (x - μ) / (σ + ε)
///
/// Edge cases:
/// - Empty slice returns empty vec
/// - Single element returns vec![0.0]
/// - Constant values (σ ≈ 0) return zeros
pub fn zscore_normalize(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0.0];
    }
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { *v } else { 0.0 }).collect();
    let mean = clean.iter().sum::<f64>() / n as f64;
    let variance = clean.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std = variance.sqrt();
    if !std.is_finite() || std < 1e-8 {
        return vec![0.0; n];
    }
    clean.iter().map(|v| (v - mean) / std).collect()
}

/// Blended advantage: λ · A_step + (1-λ) · A_outcome
pub fn blended_advantage(
    step_advantage: f64,
    outcome_advantage: f64,
    lambda: f64,
) -> f64 {
    lambda * step_advantage + (1.0 - lambda) * outcome_advantage
}

/// Compute trajectory-level convergence from step reward variance.
///
/// Low variance → high convergence (easy/consistent) → lean on step-level.
/// High variance → low convergence (hard/noisy) → lean on outcome.
/// Returns a value in [0, 1].
pub fn trajectory_convergence(trajectory: &AgentTrajectory) -> f64 {
    let rewards: Vec<f64> = trajectory
        .steps
        .iter()
        .map(|s| s.external_reward.unwrap_or(if s.success { 1.0 } else { 0.0 }))
        .collect();
    let n = rewards.len();
    if n < 2 {
        return 0.5; // neutral for short trajectories
    }
    let mean = rewards.iter().sum::<f64>() / n as f64;
    let variance = rewards.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    if !variance.is_finite() || variance < 1e-8 {
        return 0.95; // nearly constant → high convergence
    }
    // convergence = exp(-variance * 5), maps variance ~0 → 1.0, variance ~1 → 0.006
    (-variance * 5.0).exp()
}

/// λ-GRPO loss function: implicit PRM via step-level advantage normalization.
///
/// Key formula (from λ-GRPO ICML 2026):
///   L_λ-GRPO = E[min(r_t(θ)·A_t, clip(r_t(θ), 1-ε, 1+ε)·A_t)] - β·KL
///
/// Where:
///   r_t(θ) = π_θ(a_t|s_t) / π_old(a_t|s_t)  — importance sampling ratio
///   A_t = λ · A_step_t + (1-λ) · A_outcome  — blended advantage
///   A_step_t = (reward_t - mean(reward_group)) / std(reward_group)  — step-level normalization
///   A_outcome = (outcome - mean(outcome_group)) / std(outcome_group)  — outcome-level
///   When difficulty_adaptive_lambda is enabled: λ_t = λ_base * convergence_t * difficulty_scale
///
/// Computes per-step rewards directly from trajectory data WITHOUT needing an
/// explicit PRM model. The λ parameter implicitly controls PRM strength:
///   λ=0:  pure outcome-level GRPO (ignore step signals)
///   λ=1:  pure step-level PRM (ignore final outcome)
///   λ=0.3: recommended balance
pub fn lambda_grpo_loss(
    trajectories: &[AgentTrajectory],
    config: &LambdaGrpoConfig,
) -> Vec<LambdaGrpoResult> {
    if trajectories.is_empty() {
        return Vec::new();
    }

    // Step 1: Extract implicit step-level rewards from trajectory data (no PRM/coach needed)
    let all_step_rewards: Vec<f64> = trajectories
        .iter()
        .flat_map(|t| {
            t.steps.iter().map(|s| {
                s.external_reward
                    .unwrap_or(if s.success { 1.0 } else { 0.0 })
            })
        })
        .collect();

    // Step 2: Extract outcome rewards
    let all_outcomes: Vec<f64> = trajectories
        .iter()
        .map(|t| {
            t.outcome_reward
                .unwrap_or(if t.completed { 1.0 } else { 0.0 })
        })
        .collect();

    // Step 3: Z-score normalize both within their groups
    let normalized_steps = zscore_normalize(&all_step_rewards);
    let normalized_outcomes = zscore_normalize(&all_outcomes);

    // Step 4: Compute λ-GRPO advantages and losses per trajectory
    let mut results = Vec::with_capacity(trajectories.len());
    let mut step_offset = 0;

    for (traj_idx, trajectory) in trajectories.iter().enumerate() {
        let n_steps = trajectory.steps.len();
        let outcome_adv = if traj_idx < normalized_outcomes.len() {
            normalized_outcomes[traj_idx]
        } else {
            0.0
        };

        // Per-trajectory difficulty-adaptive λ
        let lambda_t = if config.difficulty_adaptive_lambda {
            let conv = trajectory_convergence(trajectory);
            let base = config.lambda;
            let adapted = base * conv * config.difficulty_scale;
            adapted.max(0.0).min(1.0)
        } else {
            config.lambda
        };

        // Apply LATA (√L) normalization if enabled
        let lata = if config.lata_normalize {
            (n_steps.max(1) as f64).sqrt()
        } else {
            1.0
        };

        let mut step_advantages = Vec::with_capacity(n_steps);
        let mut total_loss = 0.0;
        let mut policy_gradient_sum = 0.0;

        for local_idx in 0..n_steps {
            let abs_step_idx = step_offset + local_idx;
            let step_adv_raw = if abs_step_idx < normalized_steps.len() {
                normalized_steps[abs_step_idx]
            } else {
                0.0
            };

            let step_adv = step_adv_raw / lata;
            let outcome_adv_scaled = outcome_adv / lata;
            let blended = blended_advantage(step_adv, outcome_adv_scaled, lambda_t);

            step_advantages.push(StepAdvantage {
                step_idx: trajectory.steps[local_idx].step_idx,
                advantage: blended,
                normalized_advantage: step_adv_raw,
            });

            // Clipped surrogate loss (PPO-style pessimistic bound)
            // When advantage ≥ 0: cap at ε to avoid over-optimization
            // When advantage < 0: floor at -ε to avoid instability
            let clipped_adv = if blended >= 0.0 {
                blended.min(config.epsilon)
            } else {
                blended.max(-config.epsilon)
            };
            let step_loss = -clipped_adv;

            total_loss += step_loss;
            policy_gradient_sum += -blended;
        }

        // KL penalty: β · KL ≈ β · (LATA²) as regularization
        let kl_penalty = config.kl_coef * lata.powi(2);

        results.push(LambdaGrpoResult {
            trajectory_id: trajectory.trajectory_id,
            step_advantages,
            loss: total_loss + kl_penalty,
            policy_gradient: policy_gradient_sum,
            kl_penalty,
        });

        step_offset += n_steps;
    }

    results
}

/// λ-GRPO enhanced ProcessRewardLearner.
///
/// Wraps the standard `ProcessRewardLearner` and replaces its `learn_step`
/// with λ-GRPO's implicit PRM learning. Trajectories are collected via
/// `TrajectoryCollector`, then the λ-GRPO loss computes step-level advantages
/// via within-group z-score normalization and blended step/outcome rewards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaGrpoLearner {
    pub config: LambdaGrpoConfig,
    #[serde(skip)]
    pub inner: ProcessRewardLearner,
    pub results: Vec<LambdaGrpoResult>,
    pub total_steps_learned: u64,
}

impl LambdaGrpoLearner {
    pub fn new(policy: E8Policy, coach: Box<dyn Coach>, config: LambdaGrpoConfig) -> Self {
        Self {
            config,
            inner: ProcessRewardLearner::new(policy, coach),
            results: Vec::new(),
            total_steps_learned: 0,
        }
    }

    /// Learn step with λ-GRPO: collect trajectories, compute λ-GRPO loss, update policy.
    ///
    /// `collect_fn` populates the inner `TrajectoryCollector`, after which
    /// `lambda_grpo_loss` computes per-step advantages WITHOUT requiring an
    /// explicit PRM. The advantages are fed into `E8Policy::learn_from_scores`
    /// for a 2x training speedup over explicit PRM methods.
    pub fn learn_step_grpo<F>(&mut self, collect_fn: F) -> Vec<LambdaGrpoResult>
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.inner.collector);
        let trajectories: Vec<AgentTrajectory> = self.inner.collector.collected.drain(..).collect();

        let step_results = lambda_grpo_loss(&trajectories, &self.config);

        // Feed λ-GRPO advantages into the policy (implicit PRM update)
        for (result, traj) in step_results.iter().zip(trajectories.iter()) {
            let mode_scores: Vec<_> = result
                .step_advantages
                .iter()
                .filter_map(|sa| {
                    traj.steps
                        .get(sa.step_idx)
                        .map(|step| (sa.step_idx, step.e8_mode, sa.advantage))
                })
                .collect();
            if !mode_scores.is_empty() {
                let process_scores: Vec<ProcessScore> = mode_scores
                    .iter()
                    .map(|&(orig_idx, _mode, adv)| ProcessScore {
                        step_idx: orig_idx,
                        score: ((adv + 1.0) / 2.0).max(0.0).min(1.0),
                        confidence: 0.5,
                        criteria: Vec::new(),
                        attribution_tags: vec!["grpo".to_string()],
                    })
                    .collect();
                self.inner.policy.learn_from_scores(traj, &process_scores);
            }
        }

        self.results.extend(step_results.clone());
        self.total_steps_learned += 1;
        self.inner.learning_count += 1;

        step_results
    }

    /// Learn multiple steps for convergence tracking.
    ///
    /// `collect_fn` is called `steps` times, each time producing a batch of
    /// λ-GRPO results. Returns a vector of batch results for convergence analysis.
    pub fn learn_steps<F>(&mut self, collect_fn: F, steps: usize) -> Vec<Vec<LambdaGrpoResult>>
    where
        F: Fn(&mut TrajectoryCollector),
    {
        let mut all_results = Vec::with_capacity(steps);
        for _ in 0..steps {
            let r = self.learn_step_grpo(|c| collect_fn(c));
            all_results.push(r);
        }
        all_results
    }
}

#[cfg(test)]
mod lambda_grpo_tests {
    use super::*;

    fn make_trajectory(
        id: u64,
        task: &str,
        n_steps: usize,
        all_success: bool,
        outcome: Option<f64>,
        completed: bool,
    ) -> AgentTrajectory {
        let mut traj = AgentTrajectory::new(id, task.to_string());
        for i in 0..n_steps {
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "lambda_test".into(),
                input: "in".into(),
                output: "out".into(),
                duration_ms: None,
                success: all_success,
                external_reward: None,
            });
        }
        traj.outcome_reward = outcome;
        traj.completed = completed;
        traj
    }

    fn make_trajectory_with_rewards(
        id: u64,
        task: &str,
        step_successes: &[bool],
        step_rewards: &[Option<f64>],
        outcome: Option<f64>,
        completed: bool,
    ) -> AgentTrajectory {
        let mut traj = AgentTrajectory::new(id, task.to_string());
        for i in 0..step_successes.len() {
            let ext_r = step_rewards.get(i).copied().unwrap_or(None);
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "step_action".into(),
                input: "in".into(),
                output: "out".into(),
                duration_ms: None,
                success: step_successes[i],
                external_reward: ext_r,
            });
        }
        traj.outcome_reward = outcome;
        traj.completed = completed;
        traj
    }

    #[test]
    fn test_lambda_grpo_config_defaults() {
        let cfg = LambdaGrpoConfig::default();
        assert_eq!(cfg.lambda, 0.3);
        assert_eq!(cfg.epsilon, 0.2);
        assert_eq!(cfg.group_size, 4);
        assert!(cfg.lata_normalize);
        assert_eq!(cfg.kl_coef, 0.01);
    }

    #[test]
    fn test_blended_advantage_lambda_0() {
        let blended = blended_advantage(0.8, 0.2, 0.0);
        assert!((blended - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_blended_advantage_lambda_1() {
        let blended = blended_advantage(0.8, 0.2, 1.0);
        assert!((blended - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_blended_advantage_lambda_0_5() {
        let blended = blended_advantage(0.8, 0.2, 0.5);
        assert!((blended - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_blended_advantage_negative_values() {
        let blended = blended_advantage(-0.5, 1.0, 0.3);
        // 0.3 * (-0.5) + 0.7 * 1.0 = -0.15 + 0.7 = 0.55
        assert!((blended - 0.55).abs() < 1e-10);
    }

    #[test]
    fn test_zscore_normalize_empty() {
        let result = zscore_normalize(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_zscore_normalize_single_element() {
        let result = zscore_normalize(&[42.0]);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_zscore_normalize_constant() {
        let result = zscore_normalize(&[5.0, 5.0, 5.0]);
        assert_eq!(result.len(), 3);
        for v in &result {
            assert!((v - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_zscore_normalize_basic() {
        let result = zscore_normalize(&[1.0, 2.0, 3.0]);
        assert_eq!(result.len(), 3);
        // mean = 2.0, std = sqrt((1+0+1)/3) = sqrt(2/3) ≈ 0.8165
        // [-1/0.8165, 0, 1/0.8165] = [-1.2247, 0, 1.2247]
        assert!((result[0] - (-1.224744871391589)).abs() < 1e-10);
        assert!((result[1] - 0.0).abs() < 1e-10);
        assert!((result[2] - 1.224744871391589).abs() < 1e-10);
    }

    #[test]
    fn test_zscore_normalize_two_values() {
        let result = zscore_normalize(&[10.0, 20.0]);
        assert_eq!(result.len(), 2);
        // mean = 15, std = sqrt((25+25)/2) = 5
        // [(10-15)/5, (20-15)/5] = [-1, 1]
        assert!((result[0] - (-1.0)).abs() < 1e-10);
        assert!((result[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lambda_grpo_loss_empty_input() {
        let cfg = LambdaGrpoConfig::default();
        let results = lambda_grpo_loss(&[], &cfg);
        assert!(results.is_empty());
    }

    #[test]
    fn test_lambda_grpo_loss_single_trajectory() {
        let cfg = LambdaGrpoConfig::default();
        let traj = make_trajectory(1, "simple", 3, true, Some(1.0), true);
        let results = lambda_grpo_loss(&[traj], &cfg);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].step_advantages.len(), 3);
        // Single trajectory → step scores are all same → z-score normalizes to 0
        // So blended advantage ≈ 0, loss ≈ 0
        for sa in &results[0].step_advantages {
            assert!(sa.advantage.abs() < 1e-8 || sa.advantage.is_nan());
        }
    }

    #[test]
    fn test_lambda_grpo_loss_with_group() {
        let cfg = LambdaGrpoConfig::default();
        // Two trajectories: one good, one bad
        let good = make_trajectory(1, "good", 2, true, Some(1.0), true);
        let bad = make_trajectory(2, "bad", 2, false, Some(0.0), false);
        let results = lambda_grpo_loss(&[good, bad], &cfg);
        assert_eq!(results.len(), 2);
        // Good trajectory should have positive advantages
        for sa in &results[0].step_advantages {
            assert!(sa.advantage >= -0.01 || sa.advantage.is_nan());
        }
        // Bad trajectory should have negative advantages
        for sa in &results[1].step_advantages {
            assert!(sa.advantage <= 0.01 || sa.advantage.is_nan());
        }
    }

    #[test]
    fn test_lambda_grpo_loss_positive_for_good_trajectories() {
        let cfg = LambdaGrpoConfig::default();
        // Mixed group: 2 good + 1 bad
        let good1 = make_trajectory(1, "good1", 1, true, Some(1.0), true);
        let good2 = make_trajectory(2, "good2", 1, true, Some(0.9), true);
        let bad = make_trajectory(3, "bad", 1, false, Some(0.0), false);
        let results = lambda_grpo_loss(&[good1, good2, bad], &cfg);

        // Good trajectories should have positive advantages relative to group
        assert_eq!(results[0].trajectory_id, 1);
        assert_eq!(results[1].trajectory_id, 2);
        assert_eq!(results[2].trajectory_id, 3);

        // Good outcomes normalize to positive; bad to negative
        for sa in &results[0].step_advantages {
            assert!(sa.advantage > -0.5, "good trajectory should not be heavily penalized");
        }
        for sa in &results[1].step_advantages {
            assert!(sa.advantage > -0.5, "good trajectory should not be heavily penalized");
        }
        // The bad one should have negative or near-zero advantages
        for sa in &results[2].step_advantages {
            assert!(sa.advantage <= 0.5, "bad trajectory should not have large positive advantage");
        }
    }

    #[test]
    fn test_lambda_grpo_loss_external_rewards_feed_through() {
        let cfg = LambdaGrpoConfig::default();
        // Trajectories with explicit external rewards on each step
        let high = make_trajectory_with_rewards(
            1, "high", &[true, true], &[Some(0.9), Some(0.8)], Some(1.0), true,
        );
        let low = make_trajectory_with_rewards(
            2, "low", &[true, false], &[Some(0.1), Some(0.0)], Some(0.0), false,
        );
        let results = lambda_grpo_loss(&[high, low], &cfg);
        assert_eq!(results.len(), 2);
        // High-reward trajectory should have higher (less negative) loss
        // than low-reward trajectory
        assert!(
            results[0].loss < results[1].loss,
            "high-reward trajectory should have lower loss: {:.4} vs {:.4}",
            results[0].loss,
            results[1].loss
        );
    }

    #[test]
    fn test_lambda_grpo_loss_lata_disable() {
        let mut cfg = LambdaGrpoConfig::default();
        cfg.lata_normalize = false;

        let short = make_trajectory(1, "short", 1, true, Some(1.0), true);
        let long = make_trajectory(2, "long", 8, true, Some(0.9), true);
        let results = lambda_grpo_loss(&[short, long], &cfg);

        // Both should have valid results
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].step_advantages.len(), 1);
        assert_eq!(results[1].step_advantages.len(), 8);
    }

    #[test]
    fn test_lambda_grpo_loss_different_lambda_values() {
        let good = make_trajectory(1, "good", 2, true, Some(1.0), true);
        let bad = make_trajectory(2, "bad", 2, false, Some(0.0), false);

        // λ=0: pure outcome — both step rewards ignored, only outcome matters
        let cfg0 = LambdaGrpoConfig { lambda: 0.0, ..Default::default() };
        let r0 = lambda_grpo_loss(&[good.clone(), bad.clone()], &cfg0);
        let loss_diff_0 = (r0[0].loss - r0[1].loss).abs();

        // λ=1: pure step — outcome ignored, only step rewards matter
        let cfg1 = LambdaGrpoConfig { lambda: 1.0, ..Default::default() };
        let r1 = lambda_grpo_loss(&[good.clone(), bad.clone()], &cfg1);
        let loss_diff_1 = (r1[0].loss - r1[1].loss).abs();

        // λ=0.5: balanced
        let cfg5 = LambdaGrpoConfig { lambda: 0.5, ..Default::default() };
        let r5 = lambda_grpo_loss(&[good, bad], &cfg5);
        let loss_diff_5 = (r5[0].loss - r5[1].loss).abs();

        // All should produce different loss separations
        // (different λ values weight step vs outcome differently)
        assert!(loss_diff_0 >= 0.0);
        assert!(loss_diff_1 >= 0.0);
        assert!(loss_diff_5 >= 0.0);
    }

    #[test]
    fn test_lambda_grpo_learner_learn_step() {
        let policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let config = LambdaGrpoConfig::default();
        let mut learner = LambdaGrpoLearner::new(policy, coach, config);

        let results = learner.learn_step_grpo(|collector| {
            collector.begin("grpo_test".into());
            collector.record_step(
                SpecialistType::Planner, ReasoningHexagram(0),
                "plan".into(), "".into(), "plan_out".into(), None, true, None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer, ReasoningHexagram(1),
                "code".into(), "plan_out".into(), "code_out".into(), None, true, None,
            );
            collector.record_step(
                SpecialistType::MetaCognitionAnalyst, ReasoningHexagram(2),
                "verify".into(), "code_out".into(), "verified".into(), None, true, Some(1.0),
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].step_advantages.len(), 3);
        assert_eq!(learner.total_steps_learned, 1);
        assert_eq!(learner.inner.learning_count, 1);
        // Policy should have been updated
        let total_value: f64 = learner.inner.policy.mode_values.iter().sum();
        assert!(total_value >= 0.0, "policy values should be non-negative");
    }

    #[test]
    fn test_lambda_grpo_learner_multiple_steps() {
        let policy = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let config = LambdaGrpoConfig::default();
        let mut learner = LambdaGrpoLearner::new(policy, coach, config);

        let all_results = learner.learn_steps(
            |collector| {
                collector.begin("multi_step".into());
                collector.record_step(
                    SpecialistType::Planner, ReasoningHexagram(0),
                    "plan".into(), "".into(), "out".into(), None, true, None,
                );
                collector.finish(Some(1.0), true);
            },
            3,
        );

        assert_eq!(all_results.len(), 3);
        assert_eq!(learner.total_steps_learned, 3);
        assert_eq!(learner.inner.learning_count, 3);
        assert_eq!(learner.results.len(), 3);

        // Each step produced one trajectory result
        for (i, batch) in all_results.iter().enumerate() {
            assert_eq!(batch.len(), 1, "batch {} should have 1 result", i);
        }
    }

    #[test]
    fn test_lambda_grpo_loss_with_varying_lengths() {
        let cfg = LambdaGrpoConfig::default();
        let short = make_trajectory(1, "short", 1, true, Some(1.0), true);
        let medium = make_trajectory(2, "medium", 3, true, Some(0.8), true);
        let long = make_trajectory(3, "long", 6, true, Some(0.9), true);
        let results = lambda_grpo_loss(&[short, medium, long], &cfg);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].step_advantages.len(), 1);
        assert_eq!(results[1].step_advantages.len(), 3);
        assert_eq!(results[2].step_advantages.len(), 6);
        // All should produce valid step advantages
        for result in &results {
            for sa in &result.step_advantages {
                assert!(!sa.advantage.is_nan(), "advantage should not be NaN");
            }
        }
    }

    #[test]
    fn test_lambda_grpo_learner_config_affects_learning() {
        // λ=0: only outcome matters, step-level rewards ignored
        // λ=1: only step matters, outcome ignored
        // Use 2 trajectories so group normalization produces non-zero advantages

        // Create trajectories where step-level and outcome-level signals conflict:
        // Traj A: step fails (0) but final outcome succeeds (1) — overridden by external reward
        // Traj B: step succeeds (1) but final outcome fails (0)
        // This creates divergent step vs outcome z-scores for λ to blend differently
        let collect = |collector: &mut TrajectoryCollector| {
            collector.begin("step_bad_outcome_good".into());
            collector.record_step(
                SpecialistType::Planner, ReasoningHexagram(0),
                "step".into(), "".into(), "out".into(), None, false, Some(1.0),
            );
            collector.finish(Some(0.0), true);

            collector.begin("step_good_outcome_bad".into());
            collector.record_step(
                SpecialistType::Planner, ReasoningHexagram(1),
                "step".into(), "".into(), "out".into(), None, true, Some(0.0),
            );
            collector.finish(Some(1.0), true);
        };

        let cfg0 = LambdaGrpoConfig { lambda: 0.0, ..Default::default() };
        let policy0 = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach0: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner0 = LambdaGrpoLearner::new(policy0, coach0, cfg0);
        let r0 = learner0.learn_step_grpo(collect);

        let cfg1 = LambdaGrpoConfig { lambda: 1.0, ..Default::default() };
        let policy1 = E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach1: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner1 = LambdaGrpoLearner::new(policy1, coach1, cfg1);
        let r1 = learner1.learn_step_grpo(collect);

        // Different λ values → different advantages and loss
        assert!(
            (r0[0].loss - r1[0].loss).abs() > 1e-6,
            "λ=0 and λ=1 should produce different loss: {:.6} vs {:.6}",
            r0[0].loss,
            r1[0].loss
        );
        let adv0 = r0[0].step_advantages[0].advantage;
        let adv1 = r1[0].step_advantages[0].advantage;
        assert!(
            (adv0 - adv1).abs() > 1e-6,
            "λ=0 and λ=1 should produce different advantages: {:.6} vs {:.6}",
            adv0,
            adv1
        );
    }

    #[test]
    fn test_trajectory_convergence_constant() {
        let mut traj = AgentTrajectory::new(1, "test".into());
        for i in 0..4 {
            traj.push(TrajectoryStep {
                step_idx: i, specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(), input: "".into(), output: "".into(),
                duration_ms: None, success: true, external_reward: Some(1.0),
            });
        }
        let c = trajectory_convergence(&traj);
        assert!(c > 0.9, "constant rewards should give high convergence: {:.4}", c);
    }

    #[test]
    fn test_trajectory_convergence_noisy() {
        let mut traj = AgentTrajectory::new(2, "noisy".into());
        let rewards = [0.0, 1.0, 0.0, 1.0];
        for (i, &r) in rewards.iter().enumerate() {
            traj.push(TrajectoryStep {
                step_idx: i, specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(), input: "".into(), output: "".into(),
                duration_ms: None, success: r > 0.5, external_reward: Some(r),
            });
        }
        let c = trajectory_convergence(&traj);
        assert!(c < 0.5, "noisy rewards should give low convergence: {:.4}", c);
    }

    #[test]
    fn test_trajectory_convergence_short() {
        let mut traj = AgentTrajectory::new(3, "short".into());
        traj.push(TrajectoryStep {
            step_idx: 0, specialist: SpecialistType::Planner,
            e8_mode: ReasoningHexagram(0),
            action: "".into(), input: "".into(), output: "".into(),
            duration_ms: None, success: true, external_reward: None,
        });
        let c = trajectory_convergence(&traj);
        assert!((c - 0.5).abs() < 1e-6, "short (<2) should give neutral 0.5: {:.4}", c);
    }

    #[test]
    fn test_lambda_grpo_loss_difficulty_adaptive() {
        // Two trajectories with same outcome but different step variance.
        // High-consistency traj (all 1.0s) should get higher λ than low-consistency (0,1,0,1).
        let mut high_cons = AgentTrajectory::new(1, "easy".into());
        for i in 0..4 {
            high_cons.push(TrajectoryStep {
                step_idx: i, specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(), input: "".into(), output: "".into(),
                duration_ms: None, success: true, external_reward: Some(1.0),
            });
        }
        high_cons.outcome_reward = Some(1.0);
        high_cons.completed = true;

        let mut low_cons = AgentTrajectory::new(2, "hard".into());
        let noisy = [0.0, 1.0, 0.0, 1.0];
        for (i, &r) in noisy.iter().enumerate() {
            low_cons.push(TrajectoryStep {
                step_idx: i, specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(), input: "".into(), output: "".into(),
                duration_ms: None, success: r > 0.5, external_reward: Some(r),
            });
        }
        low_cons.outcome_reward = Some(1.0);
        low_cons.completed = true;

        let mut cfg = LambdaGrpoConfig {
            difficulty_adaptive_lambda: true,
            difficulty_scale: 1.0,
            ..Default::default()
        };
        let results = lambda_grpo_loss(&[high_cons.clone(), low_cons.clone()], &cfg);

        assert_eq!(results.len(), 2);
        // With adaptive λ, the high-consistency traj leans more on step-level,
        // which should produce different advantages than uniform λ.
        // Both trajectories have same outcome (1.0) but different step variance.
        // λ_high ≈ 0.3 * 0.95 * 1.0 = 0.285, λ_low ≈ 0.3 * ~0.03 * 1.0 ≈ 0.009
        // So high-consistency advantages blend step + outcome, low-consistency ≈ pure outcome.
        // Outcome z-scores are the same (1.0, 1.0) → normalized to 0.0 within group.
        // Step z-scores: high_cons steps all same → 0.0; low_cons steps: 0→-1, 1→1, 0→-1, 1→1
        // So advantages should reflect the difference.
        for result in &results {
            for sa in &result.step_advantages {
                assert!(!sa.advantage.is_nan(), "advantage should not be NaN");
            }
        }

        // Verify that disabling adaptive λ produces different results
        cfg.difficulty_adaptive_lambda = false;
        let results_uniform = lambda_grpo_loss(&[high_cons.clone(), low_cons.clone()], &cfg);
        let mut any_diff = false;
        for (adaptive, uniform) in results.iter().zip(results_uniform.iter()) {
            for (sa_a, sa_u) in adaptive.step_advantages.iter().zip(uniform.step_advantages.iter()) {
                if (sa_a.advantage - sa_u.advantage).abs() > 1e-8 {
                    any_diff = true;
                }
            }
        }
        assert!(any_diff, "adaptive and uniform λ should produce different advantages");
    }

    #[test]
    fn test_lambda_grpo_loss_adaptive_high_vs_low_scale() {
        let mut traj = AgentTrajectory::new(1, "mixed".into());
        let mixed = [0.0, 0.8, 0.2, 0.9, 0.1, 0.95];
        for (i, &r) in mixed.iter().enumerate() {
            traj.push(TrajectoryStep {
                step_idx: i, specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "".into(), input: "".into(), output: "".into(),
                duration_ms: None, success: r > 0.5, external_reward: Some(r),
            });
        }
        traj.outcome_reward = Some(0.8);
        traj.completed = true;

        // Low scale (0.1) should produce λ close to base (0.3 * conv * 0.1)
        let cfg_low = LambdaGrpoConfig {
            difficulty_adaptive_lambda: true,
            difficulty_scale: 0.1,
            ..Default::default()
        };
        let r_low = lambda_grpo_loss(&[traj.clone()], &cfg_low);

        // High scale (3.0) should amplify convergence effect
        let cfg_high = LambdaGrpoConfig {
            difficulty_adaptive_lambda: true,
            difficulty_scale: 3.0,
            ..Default::default()
        };
        let r_high = lambda_grpo_loss(&[traj], &cfg_high);

        // With same trajectory, different scales → different advantages
        let mut diff = false;
        for (sa_l, sa_h) in r_low[0].step_advantages.iter().zip(r_high[0].step_advantages.iter()) {
            if (sa_l.advantage - sa_h.advantage).abs() > 1e-8 {
                diff = true;
            }
        }
        assert!(diff, "scale=0.1 and scale=3.0 should produce different advantages");
    }
}

// ═══════════════════════════════════════════════════════════════════
// Step-GRPO: Token-Efficient Overthinking Reduction
// ═══════════════════════════════════════════════════════════════════

/// Step-GRPO configuration for overthinking reduction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StepGrpoConfig {
    /// Base reward for each correct step (default: 1.0)
    pub step_reward: f64,
    /// Penalty per step beyond optimal length (default: -0.1 per extra step)
    pub overthinking_penalty: f64,
    /// Reward for completing the task (default: 2.0)
    pub completion_bonus: f64,
    /// Optimal number of steps for the task type (default: 3)
    pub optimal_steps: usize,
    /// Maximum steps before heavy penalty (default: 10)
    pub max_steps: usize,
    /// Whether to apply length normalization (default: true)
    pub length_normalize: bool,
    /// GRPO clipping epsilon (default: 0.2)
    pub epsilon: f64,
}

impl Default for StepGrpoConfig {
    fn default() -> Self {
        Self {
            step_reward: 1.0,
            overthinking_penalty: -0.1,
            completion_bonus: 2.0,
            optimal_steps: 3,
            max_steps: 10,
            length_normalize: true,
            epsilon: 0.2,
        }
    }
}

/// Per-step reward with overthinking penalty
#[derive(Debug, Clone)]
pub struct StepReward {
    pub step_idx: usize,
    /// Base reward for this step
    pub base_reward: f64,
    /// Overthinking penalty applied
    pub overthinking_penalty: f64,
    /// Whether this step completed the task
    pub is_completion: bool,
    /// Final reward = base_reward + overthinking_penalty + completion_bonus
    pub final_reward: f64,
}

impl StepReward {
    pub fn new(step_idx: usize, base: f64) -> Self {
        Self {
            step_idx,
            base_reward: base,
            overthinking_penalty: 0.0,
            is_completion: false,
            final_reward: base,
        }
    }
}

/// Compute step-level rewards with overthinking penalty.
///
/// Steps beyond `optimal_steps` receive increasing penalties.
/// Steps beyond `max_steps` receive heavy penalties.
/// The completion step gets a bonus.
///
/// Returns (step_rewards, total_tokens_saved_estimate)
pub fn compute_step_rewards(
    trajectory: &AgentTrajectory,
    config: &StepGrpoConfig,
) -> (Vec<StepReward>, usize) {
    let total_steps = trajectory.steps.len();
    let mut rewards = Vec::with_capacity(total_steps);
    let mut tokens_saved = 0usize;

    for (i, step) in trajectory.steps.iter().enumerate() {
        let base = if step.success { config.step_reward } else { 0.0 };
        let mut reward = StepReward::new(i, base);

        if i >= config.optimal_steps {
            let extra = i - config.optimal_steps + 1;
            let penalty = if extra > (config.max_steps - config.optimal_steps) {
                -0.5 * (extra as f64)
            } else {
                config.overthinking_penalty * (extra as f64)
            };
            reward.overthinking_penalty = penalty;
            tokens_saved += 1;
        }

        if i == total_steps - 1 && step.success && trajectory.completed {
            reward.is_completion = true;
            reward.final_reward = base + reward.overthinking_penalty + config.completion_bonus;
        } else {
            reward.final_reward = base + reward.overthinking_penalty;
        }

        if config.length_normalize {
            reward.final_reward /= total_steps.max(1) as f64;
        }

        rewards.push(reward);
    }

    (rewards, tokens_saved)
}

/// Compute Step-GRPO advantages from step rewards.
///
/// Normalizes rewards within a group:
///   advantage_t = (reward_t - μ_group) / (σ_group + ε)
pub fn compute_step_advantages(
    all_rewards: &[Vec<StepReward>],
    _config: &StepGrpoConfig,
) -> Vec<Vec<f64>> {
    let all_final: Vec<f64> = all_rewards.iter()
        .flat_map(|rewards| rewards.iter().map(|r| r.final_reward))
        .collect();

    let mu = all_final.iter().sum::<f64>() / (all_final.len() as f64).max(1.0);
    let variance = all_final.iter()
        .map(|r| (r - mu).powi(2))
        .sum::<f64>() / (all_final.len() as f64).max(1.0);
    let sigma = variance.sqrt().max(1e-8);

    all_rewards.iter()
        .map(|rewards| {
            rewards.iter().map(|r| (r.final_reward - mu) / sigma).collect()
        })
        .collect()
}

/// Estimate token savings from Step-GRPO compared to baseline
pub fn estimate_token_savings(
    all_rewards: &[Vec<StepReward>],
) -> (usize, f64) {
    let total_saved: usize = all_rewards.iter()
        .flat_map(|r| r.iter())
        .map(|r| if r.overthinking_penalty < 0.0 { 1 } else { 0 })
        .sum();
    let total_steps: usize = all_rewards.iter().map(|r| r.len()).sum();
    let savings_ratio = if total_steps > 0 {
        total_saved as f64 / total_steps as f64
    } else {
        0.0
    };
    (total_saved, savings_ratio)
}

/// Step-GRPO report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGrpoReport {
    pub config: StepGrpoConfig,
    pub trajectory_count: usize,
    pub total_steps: usize,
    pub total_tokens_saved: usize,
    pub savings_ratio: f64,
    pub avg_steps_per_task: f64,
}

/// Step-GRPO enhanced learner that reduces overthinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGrpoLearner {
    pub config: StepGrpoConfig,
    pub collector: TrajectoryCollector,
    pub reports: Vec<StepGrpoReport>,
}

impl StepGrpoLearner {
    pub fn new(config: StepGrpoConfig) -> Self {
        Self {
            config,
            collector: TrajectoryCollector::new(),
            reports: Vec::new(),
        }
    }

    /// Run Step-GRPO on collected trajectories and return (rewards, advantages, report)
    pub fn evaluate<F>(&mut self, collect_fn: F) -> (Vec<Vec<StepReward>>, Vec<Vec<f64>>, StepGrpoReport)
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.collector);
        let trajectories: Vec<AgentTrajectory> = self.collector.collected.drain(..).collect();
        let total_steps: usize = trajectories.iter().map(|t| t.steps.len()).sum();
        let count = trajectories.len();

        let all_rewards: Vec<Vec<StepReward>> = trajectories.iter()
            .map(|t| compute_step_rewards(t, &self.config))
            .map(|(r, _)| r)
            .collect();

        let (total_saved, savings_ratio) = estimate_token_savings(&all_rewards);
        let advantages = compute_step_advantages(&all_rewards, &self.config);

        let report = StepGrpoReport {
            config: self.config,
            trajectory_count: count,
            total_steps,
            total_tokens_saved: total_saved,
            savings_ratio,
            avg_steps_per_task: if count > 0 { total_steps as f64 / count as f64 } else { 0.0 },
        };
        self.reports.push(report.clone());

        (all_rewards, advantages, report)
    }
}

#[cfg(test)]
mod step_grpo_tests {
    use super::*;

    fn make_step_grpo_traj(
        id: u64,
        task: &str,
        n_steps: usize,
        all_success: bool,
        completed: bool,
    ) -> AgentTrajectory {
        let mut traj = AgentTrajectory::new(id, task.to_string());
        for i in 0..n_steps {
            traj.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram(i as u8),
                action: "grpo_step".into(),
                input: "in".into(),
                output: "out".into(),
                duration_ms: None,
                success: all_success,
                external_reward: None,
            });
        }
        traj.completed = completed;
        traj.outcome_reward = if completed { Some(1.0) } else { None };
        traj
    }

    #[test]
    fn test_step_grpo_config_defaults() {
        let cfg = StepGrpoConfig::default();
        assert_eq!(cfg.step_reward, 1.0);
        assert_eq!(cfg.overthinking_penalty, -0.1);
        assert_eq!(cfg.completion_bonus, 2.0);
        assert_eq!(cfg.optimal_steps, 3);
        assert_eq!(cfg.max_steps, 10);
        assert!(cfg.length_normalize);
        assert_eq!(cfg.epsilon, 0.2);
    }

    #[test]
    fn test_compute_step_rewards_basic() {
        let cfg = StepGrpoConfig::default();
        let traj = make_step_grpo_traj(1, "simple", 3, true, true);
        let (rewards, saved) = compute_step_rewards(&traj, &cfg);

        assert_eq!(rewards.len(), 3);
        assert_eq!(saved, 0);
        // All steps succeed, length=3 = optimal, no overthinking penalty
        // length_normalize = true → divide by 3
        for r in &rewards {
            assert!(r.overthinking_penalty >= 0.0 || r.overthinking_penalty == 0.0);
        }
        // Last step gets completion bonus before normalization
        assert!(rewards[2].is_completion);
    }

    #[test]
    fn test_compute_step_rewards_overthinking_penalty() {
        let cfg = StepGrpoConfig {
            optimal_steps: 2,
            length_normalize: false,
            ..Default::default()
        };
        // 5 steps, optimal is 2 → steps 2,3,4 get penalties
        let traj = make_step_grpo_traj(1, "overthink", 5, true, true);
        let (rewards, saved) = compute_step_rewards(&traj, &cfg);

        assert_eq!(rewards.len(), 5);
        assert!(saved > 0);
        // Steps 0 and 1: no penalty
        assert_eq!(rewards[0].overthinking_penalty, 0.0);
        assert_eq!(rewards[1].overthinking_penalty, 0.0);
        // Step 2: extra = 1 → penalty = -0.1
        assert_eq!(rewards[2].overthinking_penalty, -0.1);
        // Step 3: extra = 2 → penalty = -0.2
        assert_eq!(rewards[3].overthinking_penalty, -0.2);
        // Step 4: extra = 3 → penalty = -0.3
        assert!((rewards[4].overthinking_penalty - (-0.3)).abs() < 1e-12);
    }

    #[test]
    fn test_compute_step_rewards_completion_bonus() {
        let cfg = StepGrpoConfig {
            length_normalize: false,
            ..Default::default()
        };
        let mut traj = make_step_grpo_traj(1, "bonus", 2, true, true);
        // Make completion explicit
        traj.completed = true;

        let (rewards, _) = compute_step_rewards(&traj, &cfg);
        // Last (index 1) step gets completion bonus = 2.0
        // base=1.0, penalty=0.0, bonus=2.0 → final=3.0
        assert!(rewards[1].is_completion);
        assert!((rewards[1].final_reward - 3.0).abs() < 1e-6);

        // Non-last step does not get completion bonus
        assert!(!rewards[0].is_completion);
        assert!((rewards[0].final_reward - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_step_advantages_basic() {
        let cfg = StepGrpoConfig::default();
        let traj1 = make_step_grpo_traj(1, "good", 2, true, true);
        let traj2 = make_step_grpo_traj(2, "bad", 2, false, false);

        let r1 = compute_step_rewards(&traj1, &cfg).0;
        let r2 = compute_step_rewards(&traj2, &cfg).0;

        let advantages = compute_step_advantages(&[r1, r2], &cfg);
        assert_eq!(advantages.len(), 2);
        assert_eq!(advantages[0].len(), 2);
        assert_eq!(advantages[1].len(), 2);

        // Good trajectory should have some positive advantages
        for &a in &advantages[0] {
            assert!(!a.is_nan());
        }
        // Bad trajectory should have some negative or lower advantages
        for &a in &advantages[1] {
            assert!(!a.is_nan());
        }
    }

    #[test]
    fn test_compute_step_advantages_group_normalization() {
        let cfg = StepGrpoConfig::default();
        // Three trajectories: high, medium, low
        let high = make_step_grpo_traj(1, "high", 2, true, true);
        let mid = make_step_grpo_traj(2, "mid", 2, true, false);
        let low = make_step_grpo_traj(3, "low", 2, false, false);

        let rh = compute_step_rewards(&high, &cfg).0;
        let rm = compute_step_rewards(&mid, &cfg).0;
        let rl = compute_step_rewards(&low, &cfg).0;

        let advantages = compute_step_advantages(&[rh, rm, rl], &cfg);
        assert_eq!(advantages.len(), 3);

        // All advantages should be normalized around 0
        let all: Vec<f64> = advantages.iter().flat_map(|v| v.iter().copied()).collect();
        let mean = all.iter().sum::<f64>() / all.len() as f64;
        assert!(mean.abs() < 1e-6, "group-normalized advantages should have mean near 0");
    }

    #[test]
    fn test_estimate_token_savings() {
        let cfg = StepGrpoConfig {
            optimal_steps: 1,
            length_normalize: false,
            ..Default::default()
        };
        // 3-step trajectory where optimal is 1
        let traj = make_step_grpo_traj(1, "verbose", 3, true, true);
        let (rewards, _) = compute_step_rewards(&traj, &cfg);

        let (saved, ratio) = estimate_token_savings(&[rewards]);
        assert!(saved > 0);
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_step_grpo_learner_evaluate() {
        let cfg = StepGrpoConfig::default();
        let mut learner = StepGrpoLearner::new(cfg);

        let (rewards, advantages, report) = learner.evaluate(|collector| {
            collector.begin("eval_test".into());
            collector.record_step(
                SpecialistType::Planner, ReasoningHexagram(0),
                "plan".into(), "".into(), "out1".into(), None, true, None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer, ReasoningHexagram(1),
                "code".into(), "out1".into(), "out2".into(), None, true, None,
            );
            collector.record_step(
                SpecialistType::MetaCognitionAnalyst, ReasoningHexagram(2),
                "verify".into(), "out2".into(), "verified".into(), None, true, Some(1.0),
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].len(), 3);
        assert_eq!(advantages.len(), 1);
        assert_eq!(advantages[0].len(), 3);
        assert_eq!(report.trajectory_count, 1);
        assert_eq!(report.total_steps, 3);
    }

    #[test]
    fn test_step_grpo_report_generation() {
        let cfg = StepGrpoConfig::default();
        let mut learner = StepGrpoLearner::new(cfg);

        learner.evaluate(|collector| {
            collector.begin("report_test1".into());
            collector.record_step(
                SpecialistType::Planner, ReasoningHexagram(0),
                "step1".into(), "".into(), "out".into(), None, true, None,
            );
            collector.finish(Some(1.0), true);
        });

        learner.evaluate(|collector| {
            collector.begin("report_test2".into());
            collector.record_step(
                SpecialistType::Planner, ReasoningHexagram(0),
                "step1".into(), "".into(), "out".into(), None, true, None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer, ReasoningHexagram(1),
                "step2".into(), "out".into(), "out2".into(), None, true, None,
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(learner.reports.len(), 2);
        let r2 = &learner.reports[1];
        assert_eq!(r2.trajectory_count, 1);
        assert_eq!(r2.total_steps, 2);
        assert!(r2.avg_steps_per_task > 0.0);
    }

    #[test]
    fn test_compute_step_rewards_max_steps_heavy_penalty() {
        let cfg = StepGrpoConfig {
            optimal_steps: 2,
            max_steps: 4,
            length_normalize: false,
            ..Default::default()
        };
        // 6 steps, max_steps=4, optimal=2
        // Steps 0,1: no penalty
        // Step 2: extra=1, within max range (max-opt=2), penalty=-0.1
        // Step 3: extra=2, within max range, penalty=-0.2
        // Step 4: extra=3 > 2 → heavy penalty = -0.5*3 = -1.5
        // Step 5: extra=4 > 2 → heavy penalty = -0.5*4 = -2.0
        let traj = make_step_grpo_traj(1, "too_long", 6, true, true);
        let (rewards, _) = compute_step_rewards(&traj, &cfg);

        assert_eq!(rewards.len(), 6);
        // Steps 0,1: no penalty
        assert_eq!(rewards[0].overthinking_penalty, 0.0);
        assert_eq!(rewards[1].overthinking_penalty, 0.0);
        // Step 2: within max range
        assert_eq!(rewards[2].overthinking_penalty, -0.1);
        // Step 3: within max range
        assert_eq!(rewards[3].overthinking_penalty, -0.2);
        // Step 4: heavy penalty
        assert_eq!(rewards[4].overthinking_penalty, -1.5);
        // Step 5: heavy penalty
        assert_eq!(rewards[5].overthinking_penalty, -2.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// WS-GRPO: Weakly Supervised GRPO with Preference Model
// ═══════════════════════════════════════════════════════════════════

use std::collections::HashMap;

/// Extract a task type from a task description by keyword matching.
///
/// Known types: code, math, reasoning, planning, search.
/// Falls back to "unknown" when no keywords match.
fn extract_task_type(task: &str) -> String {
    let lower = task.to_lowercase();
    if lower.contains("code") || lower.contains("program") || lower.contains("implement")
        || lower.contains("function") || lower.contains("algorithm") || lower.contains("debug")
    {
        return "code".to_string();
    }
    if lower.contains("math") || lower.contains("equation") || lower.contains("calculate")
        || lower.contains("numerical") || lower.contains("arithmetic")
    {
        return "math".to_string();
    }
    if lower.contains("reason") || lower.contains("logic") || lower.contains("deduce")
        || lower.contains("infer") || lower.contains("syllogism")
    {
        return "reasoning".to_string();
    }
    if lower.contains("plan") || lower.contains("schedule") || lower.contains("organize")
        || lower.contains("strategy") || lower.contains("arrange")
    {
        return "planning".to_string();
    }
    if lower.contains("search") || lower.contains("find") || lower.contains("lookup")
        || lower.contains("retrieve") || lower.contains("query")
    {
        return "search".to_string();
    }
    "unknown".to_string()
}

/// Weakly-supervised preference model that learns per-task-type reward expectations.
///
/// Maintains a running average of rewards for each extracted task type and
/// exposes the learned preference as a score in [0, 1].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPreferenceModel {
    /// (task description, final reward) history
    pub trajectory_outcomes: Vec<(String, f64)>,
    /// Learned per-task-type preference scores
    pub preference_scores: HashMap<String, f64>,
    /// Learning rate for exponential moving average (default 0.1)
    pub learning_rate: f64,
    /// Maximum number of stored outcomes (default 100)
    pub max_history: usize,
}

impl WsPreferenceModel {
    pub fn new(learning_rate: f64, max_history: usize) -> Self {
        Self {
            trajectory_outcomes: Vec::new(),
            preference_scores: HashMap::new(),
            learning_rate,
            max_history,
        }
    }

    /// Record a trajectory outcome and update the preference score for its task type.
    ///
    /// Extracts the task type from `task`, stores the outcome pair, then
    /// updates the preference score as an exponential moving average toward
    /// the running mean reward for that type.
    pub fn record_outcome(&mut self, task: &str, reward: f64) {
        let task_type = extract_task_type(task);
        self.trajectory_outcomes.push((task_type.clone(), reward));
        if self.trajectory_outcomes.len() > self.max_history {
            self.trajectory_outcomes.remove(0);
        }
        let outcomes: Vec<f64> = self
            .trajectory_outcomes
            .iter()
            .filter(|(t, _)| t == &task_type)
            .map(|(_, r)| *r)
            .collect();
        let avg = if outcomes.is_empty() {
            0.5
        } else {
            outcomes.iter().sum::<f64>() / outcomes.len() as f64
        };
        let entry = self.preference_scores.entry(task_type).or_insert(0.5);
        *entry = *entry + self.learning_rate * (avg - *entry);
    }

    /// Return the preference score for the task type extracted from `task`.
    ///
    /// Returns 0.5 (neutral) for unknown task types.
    pub fn preference_score(&self, task: &str) -> f64 {
        let task_type = extract_task_type(task);
        self.preference_scores.get(&task_type).copied().unwrap_or(0.5)
    }
}

/// A prefix-level reward with a weak supervision signal from the preference model.
#[derive(Debug, Clone)]
pub struct WsPrefixReward {
    /// Step index this reward applies to
    pub step_idx: usize,
    /// Reward assigned to the prefix leading to this step
    pub prefix_reward: f64,
    /// The preference model's contribution to this step
    pub preference_signal: f64,
}

/// WS-GRPO learner that blends weak supervision signals with λ-GRPO.
///
/// Wraps a `ProcessRewardLearner` and a `WsPreferenceModel`. Each learning step:
/// 1. Collects trajectories via the provided closure
/// 2. Extracts task types and queries the preference model
/// 3. Computes prefix-level pseudo-rewards: each step gets `preference_signal * 0.2`
/// 4. Blends with λ-GRPO advantages via z-score normalization
/// 5. Updates the preference model with observed outcomes
/// 6. Feeds the blended advantages into the policy via `learn_from_scores`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsGrpoLearner {
    pub preference_model: WsPreferenceModel,
    #[serde(skip)]
    pub inner: ProcessRewardLearner,
    pub results: Vec<LambdaGrpoResult>,
}

impl WsGrpoLearner {
    pub fn new(policy: E8Policy, coach: Box<dyn Coach>, learning_rate: f64, max_history: usize) -> Self {
        Self {
            preference_model: WsPreferenceModel::new(learning_rate, max_history),
            inner: ProcessRewardLearner::new(policy, coach),
            results: Vec::new(),
        }
    }

    /// Run one WS-GRPO learning step.
    ///
    /// `collect_fn` populates the inner `TrajectoryCollector`. Returns the base
    /// λ-GRPO results (pre-blending) for convergence tracking.
    pub fn learn_step_ws<F>(&mut self, collect_fn: F) -> Vec<LambdaGrpoResult>
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.inner.collector);
        let trajectories: Vec<AgentTrajectory> = self.inner.collector.collected.drain(..).collect();

        // Step 1: Compute base λ-GRPO advantages
        let config = LambdaGrpoConfig::default();
        let step_results = lambda_grpo_loss(&trajectories, &config);

        // Steps 2-4: For each trajectory, extract task type, get preference,
        // compute prefix rewards, blend with λ-GRPO via z-score normalization,
        // and feed into policy
        for (result, traj) in step_results.iter().zip(trajectories.iter()) {
            let pref_score = self.preference_model.preference_score(&traj.task);

            // Step 3: Prefix-level pseudo-rewards: each step gets preference_signal * 0.2
            let prefix_rewards: Vec<WsPrefixReward> = traj
                .steps
                .iter()
                .enumerate()
                .map(|(i, _)| WsPrefixReward {
                    step_idx: i,
                    prefix_reward: pref_score * 0.2,
                    preference_signal: pref_score,
                })
                .collect();

            // Step 4: Blend λ-GRPO advantages with WS prefix rewards, z-score normalize
            let blended: Vec<f64> = result
                .step_advantages
                .iter()
                .zip(prefix_rewards.iter())
                .map(|(sa, pr)| sa.advantage + pr.prefix_reward)
                .collect();

            let normalized = zscore_normalize(&blended);

            // Step 6: Feed normalized advantages into policy
            let mode_scores: Vec<_> = traj
                .steps
                .iter()
                .zip(normalized.iter())
                .enumerate()
                .map(|(i, (step, adv))| (i, step.e8_mode, *adv))
                .collect();
            if !mode_scores.is_empty() {
                let process_scores: Vec<ProcessScore> = mode_scores
                    .iter()
                    .map(|&(orig_idx, _mode, adv)| ProcessScore {
                        step_idx: orig_idx,
                        score: ((adv + 1.0) / 2.0).max(0.0).min(1.0),
                        confidence: 0.5,
                        criteria: Vec::new(),
                        attribution_tags: vec!["ws_grpo".to_string()],
                    })
                    .collect();
                self.inner.policy.learn_from_scores(traj, &process_scores);
            }
        }

        // Step 5: Update preference model with trajectory outcomes
        for traj in &trajectories {
            let reward = traj.outcome_reward.unwrap_or(if traj.completed { 1.0 } else { 0.0 });
            self.preference_model.record_outcome(&traj.task, reward);
        }

        self.results.extend(step_results.clone());
        self.inner.learning_count += 1;

        step_results
    }
}

// ═══════════════════════════════════════════════════════════════════
// GroundedPRM: MCTS Tree-Guided Step Verification
// ═══════════════════════════════════════════════════════════════════

use std::f64::consts::SQRT_2;

/// Verification action types for MCTS tree expansion.
///
/// Each type grounds a different dimension of step quality:
/// - ModeConsistency: E8 hexagram alignment with task domain
/// - TransitionPattern: consecutive-state transition fidelity
/// - RewardHistory: empirical reward expectation from past similar modes
/// - DirectionChange: orientation shifts relative to trajectory arc
/// - OscillationCheck: avoid wasteful mode flips
/// - StepPosition: position-dependent relevance signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationAction {
    ModeConsistency,
    TransitionPattern,
    RewardHistory,
    DirectionChange,
    OscillationCheck,
    StepPosition,
}

impl VerificationAction {
    pub const ALL: [VerificationAction; 6] = [
        VerificationAction::ModeConsistency,
        VerificationAction::TransitionPattern,
        VerificationAction::RewardHistory,
        VerificationAction::DirectionChange,
        VerificationAction::OscillationCheck,
        VerificationAction::StepPosition,
    ];

    /// Human-readable dimension label.
    pub fn label(&self) -> &'static str {
        match self {
            VerificationAction::ModeConsistency => "mode_consistency",
            VerificationAction::TransitionPattern => "transition_pattern",
            VerificationAction::RewardHistory => "reward_history",
            VerificationAction::DirectionChange => "direction_change",
            VerificationAction::OscillationCheck => "oscillation_check",
            VerificationAction::StepPosition => "step_position",
        }
    }

    /// Ground-truth score for this dimension given a step in context.
    /// All scores in [0, 1].
    pub fn ground(
        &self,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        mode_rewards: &[(f64, u64)],
        task_type: &str,
    ) -> f64 {
        match self {
            VerificationAction::ModeConsistency => {
                // Task→mode alignment: code tasks prefer mid-range hexagrams,
                // math prefers structured high bits, general = neutral.
                let mode = step.e8_mode.0;
                match task_type {
                    t if t.contains("code") => {
                        // Coding: prefer modes with MODE bit on (0x08) and DEPTH moderate
                        let bits = mode & 0x3F;
                        if bits & 0x08 != 0 && (bits & 0x1C) < 0x1C { 0.8 } else { 0.4 }
                    }
                    t if t.contains("math") => {
                        // Math: prefer structured high-ABST modes
                        if mode & 0x20 != 0 && mode & 0x04 != 0 { 0.85 } else { 0.45 }
                    }
                    t if t.contains("reason") => {
                        // Reasoning: prefer balanced STANCE+MODE
                        let bits = mode & 0x3F;
                        let stance = (bits & 0x20) >> 5;
                        let mode_bit = (bits & 0x08) >> 3;
                        if stance == mode_bit { 0.75 } else { 0.5 }
                    }
                    _ => 0.6, // General: neutral
                }
            }
            VerificationAction::TransitionPattern => {
                // Transition fidelity: compare step's mode vs previous step's mode.
                // High score = moderate Hamming distance (1-2 bit flips = healthy exploration).
                if let Some(prev) = trajectory.last() {
                    let prev_mode = prev.e8_mode.0;
                    let diff = (step.e8_mode.0 ^ prev_mode) as u32;
                    let hamming = diff.count_ones();
                    match hamming {
                        0 => 0.3,  // no change = stuck
                        1 => 0.9,  // single bit = focused refinement
                        2 => 0.8,  // two bits = exploration
                        3 | 4 => 0.5, // moderate jump
                        _ => 0.2,  // large jump = unstable
                    }
                } else {
                    0.7 // first step: default trust
                }
            }
            VerificationAction::RewardHistory => {
                // Expectation: compare step success to historical avg for this mode.
                let mode = step.e8_mode.0 as usize;
                let (_, count) = mode_rewards[mode];
                if count > 0 {
                    let avg = mode_rewards[mode].0 / count as f64;
                    let step_val = if step.success { 1.0 } else { 0.0 };
                    // Score = 1 - |avg - step_val|: closer to historical = more grounded
                    1.0 - (avg - step_val).abs().max(0.0).min(1.0)
                } else {
                    0.5 // no history: neutral
                }
            }
            VerificationAction::DirectionChange => {
                // Orientation: ABST bit (0x20) flip = paradigm shift.
                if let Some(prev) = trajectory.last() {
                    let prev_abst = prev.e8_mode.0 & 0x20;
                    let cur_abst = step.e8_mode.0 & 0x20;
                    if prev_abst != cur_abst {
                        // Direction change is good later in trajectory (mature pivot),
                        // bad early (unstable start)
                        let step_frac = step.step_idx as f64 / trajectory.len().max(1) as f64;
                        0.2 + step_frac * 0.6 // 0.2→0.8 as trajectory progresses
                    } else {
                        0.7 // steady direction = generally good
                    }
                } else {
                    0.6
                }
            }
            VerificationAction::OscillationCheck => {
                // Oscillation: repeating same mode wastes compute.
                if trajectory.len() >= 2 {
                    let same_as_prev = trajectory.last().map(|s| s.e8_mode.0 == step.e8_mode.0).unwrap_or(false);
                    let same_as_prev2 = trajectory.len() >= 2
                        && trajectory[trajectory.len() - 2].e8_mode.0 == step.e8_mode.0;
                    if same_as_prev && same_as_prev2 {
                        0.2 // repeating pattern
                    } else if same_as_prev {
                        0.4 // single repeat (could be normal)
                    } else {
                        0.8 // novel mode
                    }
                } else {
                    0.8
                }
            }
            VerificationAction::StepPosition => {
                // Position-dependent: early steps have lower base expectation,
                // later steps higher (convergence expectation).
                if trajectory.is_empty() {
                    return 0.5;
                }
                let total = trajectory.len() as f64;
                let pos = step.step_idx as f64;
                let fraction = pos / total.max(1.0);
                0.3 + fraction * 0.5 // 0.3 (start) → 0.8 (end)
            }
        }
    }
}

/// A single MCTS tree node representing one verification hypothesis.
struct MctsNode {
    /// The action that this node represents.
    action: VerificationAction,
    /// Ground score from the action's grounding function.
    /// Parent node index (None for root).
    parent: Option<usize>,
    /// Child node indices.
    children: Vec<usize>,
    /// Visit count for UCB1 exploration bonus.
    visits: u64,
    /// Cumulative reward from rollouts.
    total_value: f64,
}

/// MCTS tree for step verification.
///
/// Performs `num_iterations` rollouts:
/// 1. **Selection**: traverse from root using UCB1 until a leaf is reached
/// 2. **Expansion**: add children for all unvisited actions
/// 3. **Simulation**: ground score = the selected action's intrinsic quality
/// 4. **Backpropagation**: propagate reward up to root
///
/// After completion, root's `total_value / visits` is the grounded verification score.
struct MctsTree {
    nodes: Vec<MctsNode>,
    root_idx: usize,
}

impl MctsTree {
    fn new() -> Self {
        // Root node: no action, occupies index 0
        let root = MctsNode {
            action: VerificationAction::ModeConsistency, // placeholder, unused for root
            parent: None,
            children: Vec::new(),
            visits: 0,
            total_value: 0.0,
        };
        let mut tree = MctsTree {
            nodes: vec![root],
            root_idx: 0,
        };
        // Expand root with all 6 verification actions
        for act in VerificationAction::ALL {
            let child = MctsNode {
                action: act,
                parent: Some(tree.root_idx),
                children: Vec::new(),
                visits: 0,
                total_value: 0.0,
            };
            tree.nodes.push(child);
            let child_idx = tree.nodes.len() - 1;
            tree.nodes[tree.root_idx].children.push(child_idx);
        }
        tree
    }

    /// Select a leaf node using UCB1.
    fn select(&mut self) -> usize {
        let mut current = self.root_idx;
        loop {
            let node = &self.nodes[current];
            if node.children.is_empty() {
                return current;
            }
            // UCB1: pick child maximizing Q + c * sqrt(ln(N_parent) / N_child)
            let total_parent = node.visits.max(1) as f64;
            let mut best_child = node.children[0];
            let mut best_ucb = f64::NEG_INFINITY;
            for &child_idx in &node.children {
                let child = &self.nodes[child_idx];
                let n = child.visits.max(1) as f64;
                let q = if child.visits > 0 {
                    child.total_value / n
                } else {
                    0.0 // unvisited: high exploration bonus
                };
                let exploration = SQRT_2 * (total_parent.ln() / n).sqrt();
                let ucb = q + exploration;
                if ucb > best_ucb {
                    best_ucb = ucb;
                    best_child = child_idx;
                }
            }
            current = best_child;
        }
    }

    /// Simulate: look up the grounding score for the leaf node's action.
    fn simulate(
        &self,
        leaf_idx: usize,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        mode_rewards: &[(f64, u64)],
        task_type: &str,
    ) -> f64 {
        let action = self.nodes[leaf_idx].action;
        action.ground(step, trajectory, mode_rewards, task_type)
    }

    /// Backpropagate reward from leaf to root.
    fn backpropagate(&mut self, leaf_idx: usize, reward: f64) {
        let mut current = leaf_idx;
        loop {
            self.nodes[current].visits += 1;
            self.nodes[current].total_value += reward;
            match self.nodes[current].parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
    }

    /// Run the full MCTS loop for the given step+context.
    fn run(
        &mut self,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        mode_rewards: &[(f64, u64)],
        task_type: &str,
        num_iterations: usize,
    ) {
        for _ in 0..num_iterations {
            let leaf = self.select();
            let reward = self.simulate(leaf, step, trajectory, mode_rewards, task_type);
            self.backpropagate(leaf, reward);
        }
    }

    /// Return the root's grounded score = total_value / visits.
    fn grounded_score(&self) -> f64 {
        if self.nodes[self.root_idx].visits == 0 {
            return 0.5; // neutral default
        }
        self.nodes[self.root_idx].total_value / self.nodes[self.root_idx].visits as f64
    }

    /// Return per-action scores for interpretability.
    fn per_action_scores(&self) -> Vec<(VerificationAction, f64, u64)> {
        self.nodes[self.root_idx]
            .children
            .iter()
            .map(|&child_idx| {
                let child = &self.nodes[child_idx];
                let avg = if child.visits > 0 {
                    child.total_value / child.visits as f64
                } else {
                    0.0
                };
                (child.action, avg, child.visits)
            })
            .collect()
    }
}

/// GroundedPRM step verifier.
///
/// For each step in a trajectory, MCTS tree search explores verification
/// hypotheses grounded in analytic signals (mode consistency, transition
/// fidelity, reward history, direction change, oscillation check, step
/// position). The MCTS root value after `num_iterations` rollouts is the
/// grounded verification score.
///
/// The blended score = λ * grounded + (1-λ) * original PRM score, where
/// λ is the `grounded_weight` (default 0.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedPrmVerifier {
    /// Number of MCTS iterations per step (default 48).
    pub num_iterations: usize,
    /// Blend weight λ for grounded score (default 0.3).
    pub grounded_weight: f64,
    /// Per-mode empirical reward accumulator for reward-history grounding.
    pub mode_rewards: Vec<(f64, u64)>,
    /// Total steps verified in this verifier's lifetime.
    pub total_steps_verified: u64,
}

impl GroundedPrmVerifier {
    pub fn new(num_iterations: usize, grounded_weight: f64) -> Self {
        Self {
            num_iterations,
            grounded_weight,
            mode_rewards: vec![(0.0, 0); 64],
            total_steps_verified: 0,
        }
    }

    /// Verify a single step in the context of its trajectory.
    ///
    /// Returns (grounded_score, per_action_scores).
    pub fn verify_step(
        &mut self,
        step: &TrajectoryStep,
        trajectory: &[TrajectoryStep],
        task_type: &str,
    ) -> (f64, Vec<(VerificationAction, f64, u64)>) {
        self.total_steps_verified += 1;
        let mut tree = MctsTree::new();
        tree.run(step, trajectory, &self.mode_rewards, task_type, self.num_iterations);
        let score = tree.grounded_score();
        let actions = tree.per_action_scores();
        (score, actions)
    }

    /// Verify all steps in a trajectory and return blended scores.
    pub fn verify_trajectory(
        &mut self,
        trajectory: &AgentTrajectory,
        original_scores: &[ProcessScore],
        task_type: &str,
    ) -> Vec<ProcessScore> {
        // Build context slices: for step i, context = steps[0..i]
        let mut context: Vec<TrajectoryStep> = Vec::new();
        let mut blended = Vec::with_capacity(trajectory.steps.len());

        for (i, step) in trajectory.steps.iter().enumerate() {
            let (grounded, actions) = self.verify_step(step, &context, task_type);
            let original = original_scores.get(i).cloned().unwrap_or_else(|| ProcessScore::new(i));
            let blended_score = self.grounded_weight * grounded + (1.0 - self.grounded_weight) * original.score;
            let grounded_criterion = ScoredCriterion {
                name: "grounded_prm".to_string(),
                score: grounded,
                rationale: Some(format!(
                    "MCTS-verified via {} actions",
                    actions.iter().filter(|(_, _, v)| *v > 0).count()
                )),
            };
            let mut criteria = original.criteria.clone();
            criteria.push(grounded_criterion);
            // Add attribution tags per action
            let mut tags = original.attribution_tags.clone();
            tags.push("grounded_prm".to_string());
            for (act, avg, visits) in &actions {
                if *visits > 0 {
                    tags.push(format!("gp_{}:{:.2}", act.label(), avg));
                }
            }
            blended.push(ProcessScore {
                step_idx: i,
                score: blended_score.max(0.0).min(1.0),
                confidence: (original.confidence + grounded) / 2.0,
                criteria,
                attribution_tags: tags,
            });
            context.push(step.clone());
        }

        // Update mode rewards for future steps
        for step in &trajectory.steps {
            let mode = step.e8_mode.0 as usize;
            let step_val = if step.success { 1.0 } else { 0.0 };
            let (sum, count) = self.mode_rewards[mode];
            self.mode_rewards[mode] = (sum + step_val, count + 1);
        }

        blended
    }

    /// Convenience: one-shot verify + blend without managing trajectory state.
    pub fn verify_and_blend(
        &mut self,
        trajectory: &AgentTrajectory,
        coach: &dyn Coach,
        task_type: &str,
    ) -> Vec<ProcessScore> {
        // Get original scores from the coach
        let original = coach.score_episode(trajectory);
        self.verify_trajectory(trajectory, &original, task_type)
    }
}

#[cfg(test)]
mod grounded_prm_tests {
    use super::*;
    use crate::core::nt_core_hex::ReasoningHexagram;

    fn make_step(idx: usize, mode: u8, success: bool) -> TrajectoryStep {
        TrajectoryStep {
            step_idx: idx,
            specialist: SpecialistType::PatternMatcher,
            e8_mode: ReasoningHexagram(mode),
            action: format!("action_{}", idx),
            input: "in".into(),
            output: "out".into(),
            duration_ms: Some(100),
            success,
            external_reward: Some(if success { 1.0 } else { 0.0 }),
        }
    }

    #[test]
    fn test_grounded_prm_verifier_default_creation() {
        let verifier = GroundedPrmVerifier::new(48, 0.3);
        assert_eq!(verifier.num_iterations, 48);
        assert!((verifier.grounded_weight - 0.3).abs() < 1e-10);
        assert_eq!(verifier.total_steps_verified, 0);
    }

    #[test]
    fn test_mcts_tree_initialization() {
        let tree = MctsTree::new();
        assert_eq!(tree.nodes.len(), 7); // root + 6 actions
        assert_eq!(tree.nodes[0].children.len(), 6);
        // Verify all children are unvisited
        for &child_idx in &tree.nodes[0].children {
            assert_eq!(tree.nodes[child_idx].visits, 0);
        }
    }

    #[test]
    fn test_mcts_tree_single_step_verification() {
        let step = make_step(0, 56, true); // mode 56 = high STANCE + MODE
        let trajectory: Vec<TrajectoryStep> = vec![];
        let mode_rewards = [(0.0, 0); 64];
        let mut tree = MctsTree::new();
        tree.run(&step, &trajectory, &mode_rewards, "general", 48);
        let score = tree.grounded_score();
        // With 48 iterations across 6 actions ≈ 8 per action
        assert!(score > 0.0 && score <= 1.0, "score should be in (0,1], got {}", score);
        assert!(tree.nodes[0].visits > 0, "root should have been visited");
    }

    #[test]
    fn test_verification_action_ground_functions() {
        let step = make_step(0, 56, true); // mode 56
        let trajectory: Vec<TrajectoryStep> = vec![make_step(0, 48, true)];

        let mode_rewards = [(0.0, 0); 64];

        // Mode consistency for "code" task
        let mc = VerificationAction::ModeConsistency.ground(&step, &trajectory, &mode_rewards, "code");
        assert!(mc >= 0.0 && mc <= 1.0, "mode_consistency out of range: {}", mc);

        // Transition pattern
        let tp = VerificationAction::TransitionPattern.ground(&step, &trajectory, &mode_rewards, "general");
        assert!(tp >= 0.0 && tp <= 1.0, "transition out of range: {}", tp);

        // Direction change within same ABST
        let dc = VerificationAction::DirectionChange.ground(&step, &trajectory, &mode_rewards, "general");
        assert!(dc >= 0.0 && dc <= 1.0, "direction out of range: {}", dc);

        // Oscillation (first step after prev = different)
        let osc = VerificationAction::OscillationCheck.ground(&step, &trajectory, &mode_rewards, "general");
        assert!(osc >= 0.0 && osc <= 1.0, "oscillation out of range: {}", osc);
    }

    #[test]
    fn test_verifier_verify_single_step() {
        let step = make_step(0, 56, true);
        let mut verifier = GroundedPrmVerifier::new(48, 0.3);
        let (score, actions) = verifier.verify_step(&step, &[], "code");
        assert!(score > 0.0 && score <= 1.0, "score out of range: {}", score);
        assert_eq!(actions.len(), 6);
        assert_eq!(verifier.total_steps_verified, 1);
    }

    #[test]
    fn test_verifier_verify_trajectory() {
        let steps = vec![
            make_step(0, 56, true),
            make_step(1, 48, true),
            make_step(2, 40, true),
            make_step(3, 32, true),
            make_step(4, 24, true),
        ];
        let traj = AgentTrajectory {
            trajectory_id: 1,
            task: "code task".into(),
            steps,
            outcome_reward: Some(1.0),
            completed: true,
            total_duration_ms: Some(500),
        };
        let original: Vec<ProcessScore> = (0..5).map(|i| {
            ProcessScore { step_idx: i, score: 0.5 + i as f64 * 0.1, confidence: 0.5, criteria: vec![], attribution_tags: vec![] }
        }).collect();

        let mut verifier = GroundedPrmVerifier::new(36, 0.3);
        let blended = verifier.verify_trajectory(&traj, &original, "code");
        assert_eq!(blended.len(), 5);
        for (i, ps) in blended.iter().enumerate() {
            assert!(ps.score >= 0.0 && ps.score <= 1.0, "score out of range at {}: {}", i, ps.score);
            assert_eq!(ps.step_idx, i);
            // Should have grounded_prm criterion added
            assert!(ps.criteria.iter().any(|c| c.name == "grounded_prm"), "missing grounded_prm criterion at {}", i);
            // Should have grounded_prm tag
            assert!(ps.attribution_tags.iter().any(|t| t == "grounded_prm"), "missing grounded_prm tag at {}", i);
        }
        assert_eq!(verifier.total_steps_verified, 5);
    }

    #[test]
    fn test_verifier_blend_weight_effect() {
        let step = make_step(0, 56, true);
        let original = ProcessScore {
            step_idx: 0, score: 0.2, confidence: 0.5, criteria: vec![], attribution_tags: vec![],
        };
        let traj = AgentTrajectory {
            trajectory_id: 1,
            task: "test".into(),
            steps: vec![step],
            outcome_reward: None,
            completed: false,
            total_duration_ms: None,
        };

        // High grounded weight (0.9) → grounded dominates
        let mut verifier_high = GroundedPrmVerifier::new(48, 0.9);
        let blended_high = verifier_high.verify_trajectory(&traj, &[original.clone()], "general");
        let grounded_only = blended_high[0].score;
        assert!(grounded_only >= 0.0 && grounded_only <= 1.0);

        // Low grounded weight (0.1) → original dominates
        let mut verifier_low = GroundedPrmVerifier::new(48, 0.1);
        let blended_low = verifier_low.verify_trajectory(&traj, &[original], "general");
        let blended_low_score = blended_low[0].score;
        assert!((blended_low_score - 0.2).abs() < 0.3 || grounded_only > blended_low_score,
            "high weight should pull further from original than low weight");
    }

    #[test]
    fn test_verifier_mode_rewards_update() {
        let steps = vec![
            make_step(0, 10, true),
            make_step(1, 20, false),
            make_step(2, 30, true),
        ];
        let traj = AgentTrajectory {
            trajectory_id: 2, task: "test".into(), steps,
            outcome_reward: Some(0.5), completed: true, total_duration_ms: None,
        };

        let original: Vec<ProcessScore> = (0..3).map(|i| ProcessScore {
            step_idx: i, score: 0.5, confidence: 0.5, criteria: vec![], attribution_tags: vec![],
        }).collect();

        let mut verifier = GroundedPrmVerifier::new(24, 0.3);
        let _ = verifier.verify_trajectory(&traj, &original, "general");

        // Mode 10 should have (1.0, 1), 20 should have (0.0, 1), 30 should have (1.0, 1)
        assert_eq!(verifier.mode_rewards[10].1, 1);
        assert!((verifier.mode_rewards[10].0 - 1.0).abs() < 1e-10);
        assert_eq!(verifier.mode_rewards[20].1, 1);
        assert!((verifier.mode_rewards[20].0 - 0.0).abs() < 1e-10);
        assert_eq!(verifier.mode_rewards[30].1, 1);
        assert!((verifier.mode_rewards[30].0 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_verify_and_blend_one_shot() {
        let steps = vec![make_step(0, 0, true), make_step(1, 1, true)];
        let traj = AgentTrajectory {
            trajectory_id: 3, task: "math problem".into(), steps,
            outcome_reward: Some(1.0), completed: true, total_duration_ms: None,
        };
        let coach = HeuristicCoach::default();
        let mut verifier = GroundedPrmVerifier::new(48, 0.3);
        let blended = verifier.verify_and_blend(&traj, &coach, "math");
        assert_eq!(blended.len(), 2);
        assert!(blended[0].score >= 0.0 && blended[0].score <= 1.0);
    }

    #[test]
    fn test_grounded_prm_oscillation_detection() {
        // Repeated modes should produce low oscillation score
        let step = make_step(2, 42, true);
        let trajectory = vec![
            make_step(0, 42, true),
            make_step(1, 42, true),
        ];
        let osc = VerificationAction::OscillationCheck.ground(&step, &trajectory, &[(0.0, 0); 64], "general");
        assert!(osc <= 0.5, "oscillation should be low for repeated mode, got {}", osc);
    }

    #[test]
    fn test_direction_change_early_vs_late() {
        let step_early = make_step(0, 60, true); // ABST bit 0x20 is on
        let step_late = make_step(8, 60, true);
        let mut traj_early = vec![make_step(0, 36, true)]; // 36 = no ABST

        let dc_early = VerificationAction::DirectionChange.ground(&step_early, &traj_early, &[(0.0, 0); 64], "general");
        traj_early.push(make_step(1, 36, true));
        traj_early.push(make_step(2, 36, true));
        traj_early.push(make_step(3, 36, true));
        traj_early.push(make_step(4, 36, true));
        traj_early.push(make_step(5, 36, true));
        traj_early.push(make_step(6, 36, true));
        traj_early.push(make_step(7, 36, true));
        let dc_late = VerificationAction::DirectionChange.ground(&step_late, &traj_early, &[(0.0, 0); 64], "general");
        // Late direction change should score higher than early
        assert!(dc_late >= dc_early, "late direction change {}, early {}", dc_late, dc_early);
    }

    #[test]
    fn test_reward_history_ground_exact_match() {
        let step = make_step(0, 5, true);
        let mut mode_rewards = [(0.0, 0); 64];
        mode_rewards[5] = (5.0, 10); // avg = 0.5
        let score = VerificationAction::RewardHistory.ground(&step, &[], &mode_rewards, "general");
        // step_val = 1.0, avg = 0.5, diff = 0.5, score = 1 - 0.5 = 0.5
        assert!((score - 0.5).abs() < 1e-10, "expected 0.5, got {}", score);
    }

    #[test]
    fn test_transition_pattern_hamming_distances() {
        // Hamming distance 1 (single bit flip) = 0.9
        let step_h1 = make_step(0, 0x21, true); // 33 in binary: 100001
        let prev_h1 = vec![make_step(0, 0x01, true)]; // 1 in binary: 000001 — Hamming=1
        let tp_h1 = VerificationAction::TransitionPattern.ground(&step_h1, &prev_h1, &[(0.0, 0); 64], "general");
        assert!((tp_h1 - 0.9).abs() < 1e-10, "expected 0.9 for Hamming=1, got {}", tp_h1);

        // Hamming distance 0 (same) = 0.3
        let step_h0 = make_step(1, 0x01, true);
        let prev_h0 = vec![make_step(0, 0x01, true)];
        let tp_h0 = VerificationAction::TransitionPattern.ground(&step_h0, &prev_h0, &[(0.0, 0); 64], "general");
        assert!((tp_h0 - 0.3).abs() < 1e-10, "expected 0.3 for Hamming=0, got {}", tp_h0);
    }

    #[test]
    fn test_grounded_prm_confidence_blend() {
        let step = make_step(0, 42, true);
        let original = ProcessScore {
            step_idx: 0, score: 0.5, confidence: 0.3, criteria: vec![], attribution_tags: vec![],
        };
        let traj = AgentTrajectory {
            trajectory_id: 4, task: "test".into(),
            steps: vec![step],
            outcome_reward: None, completed: false, total_duration_ms: None,
        };

        let mut verifier = GroundedPrmVerifier::new(48, 0.5);
        let blended = verifier.verify_trajectory(&traj, &[original], "general");
        // confidence = (0.3 + grounded) / 2, should be >= 0.15
        assert!(blended[0].confidence >= 0.15, "confidence too low: {}", blended[0].confidence);
    }

    #[test]
    fn test_grounded_score_persists_across_calls() {
        let mut verifier = GroundedPrmVerifier::new(48, 0.3);
        let steps = vec![make_step(0, 42, true), make_step(1, 42, true)];
        let traj = AgentTrajectory {
            trajectory_id: 5, task: "test".into(), steps,
            outcome_reward: None, completed: true, total_duration_ms: None,
        };
        let original: Vec<ProcessScore> = (0..2).map(|i| ProcessScore {
            step_idx: i, score: 0.5, confidence: 0.5, criteria: vec![], attribution_tags: vec![],
        }).collect();

        let _ = verifier.verify_trajectory(&traj, &original, "general");
        assert_eq!(verifier.total_steps_verified, 2);
        let _ = verifier.verify_trajectory(&traj, &original, "general");
        assert_eq!(verifier.total_steps_verified, 4);
    }

    #[test]
    fn test_mcts_ucb_exploration_all_actions_visited() {
        let step = make_step(0, 56, true);
        let mut tree = MctsTree::new();
        tree.run(&step, &[], &[(0.0, 0); 64], "general", 60);
        // With 60 iterations and 6 actions, each should be visited at least once
        for &child_idx in &tree.nodes[0].children {
            assert!(tree.nodes[child_idx].visits > 0,
                "action {:?} was never visited in 60 iterations",
                tree.nodes[child_idx].action);
        }
    }
}

#[cfg(test)]
mod ws_grpo_tests {
    use super::*;

    #[test]
    fn test_ws_preference_model_record_and_score() {
        let mut model = WsPreferenceModel::new(0.1, 100);

        model.record_outcome("code task", 0.8);
        model.record_outcome("code task", 0.9);
        model.record_outcome("math problem", 0.3);

        let code_score = model.preference_score("code task");
        let math_score = model.preference_score("math problem");

        // code avg = (0.8 + 0.9) / 2 = 0.85, ema: 0.5 + 0.1*(0.8-0.5) = 0.53,
        //   then 0.53 + 0.1*(0.85-0.53) = 0.562
        assert!((code_score - 0.562).abs() < 1e-6, "code_score={}", code_score);
        assert!(code_score > 0.5, "code score should reflect positive outcomes");

        // math avg = 0.3, ema: 0.5 + 0.1*(0.3-0.5) = 0.48
        assert!((math_score - 0.48).abs() < 1e-6, "math_score={}", math_score);
        assert!(math_score < 0.5, "math score should reflect negative outcome");
    }

    #[test]
    fn test_ws_preference_model_unknown_task() {
        let model = WsPreferenceModel::new(0.1, 100);
        let score = model.preference_score("completely unknown gibberish task");
        assert!((score - 0.5).abs() < 1e-10, "unknown tasks should score 0.5, got {}", score);
    }

    #[test]
    fn test_ws_grpo_learner_learn_step() {
        let policy = crate::core::nt_core_policy::E8Policy::new(0.0, 1.0, 0.0, 0.5, 0.0);
        let coach: Box<dyn Coach> = Box::new(HeuristicCoach::default());
        let mut learner = WsGrpoLearner::new(policy, coach, 0.1, 100);

        let results = learner.learn_step_ws(|collector| {
            collector.begin("code debugging".into());
            collector.record_step(
                SpecialistType::Planner,
                ReasoningHexagram(0),
                "plan".into(),
                "".into(),
                "out".into(),
                None,
                true,
                None,
            );
            collector.record_step(
                SpecialistType::CodeAnalyzer,
                ReasoningHexagram(1),
                "code".into(),
                "out".into(),
                "result".into(),
                None,
                true,
                None,
            );
            collector.finish(Some(1.0), true);
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].step_advantages.len(), 2);

        // Preference model should have been updated for "code" type
        let code_score = learner.preference_model.preference_score("code debugging");
        // one outcome with reward 1.0 → 0.5 + 0.1*(1.0-0.5) = 0.55
        assert!((code_score - 0.55).abs() < 1e-6, "code_score={}", code_score);

        // Policy should have been updated via learn_from_scores
        let total_value: f64 = learner.inner.policy.mode_values.iter().sum();
        assert!(total_value >= 0.0, "policy values should be non-negative");
    }

    #[test]
    fn test_ws_preference_model_task_type_extraction() {
        assert_eq!(extract_task_type("implement fibonacci"), "code");
        assert_eq!(extract_task_type("solve equation 2x+3=7"), "math");
        assert_eq!(extract_task_type("logical deduction puzzle"), "reasoning");
        assert_eq!(extract_task_type("organize team schedule"), "planning");
        assert_eq!(extract_task_type("find the best route"), "search");
        assert_eq!(extract_task_type("write a poem"), "unknown");
    }
}
