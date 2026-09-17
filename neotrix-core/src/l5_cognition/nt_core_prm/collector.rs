use super::*;
use serde::{Deserialize, Serialize};
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_traits::SpecialistType;
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

    pub fn finish(
        &mut self,
        outcome_reward: Option<f64>,
        completed: bool,
    ) -> Option<AgentTrajectory> {
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
        base_scores
            .into_iter()
            .map(|mut ps| {
                ps.score = (ps.score / lata_factor).max(0.0).min(1.0);
                ps
            })
            .collect()
    }

    fn score_step(&self, step: &TrajectoryStep, context: &CoachContext) -> ProcessScore {
        let mut score = if step.success {
            self.success_base
        } else {
            self.failure_penalty
        };

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
            rationale: Some(
                if step.success {
                    "step completed"
                } else {
                    "step failed"
                }
                .to_string(),
            ),
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
                    self.success_base = (self.success_base
                        + lr * (normalized_reward - self.success_base))
                        .max(0.0)
                        .min(1.0);
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
