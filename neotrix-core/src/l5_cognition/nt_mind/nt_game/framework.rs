//! NT-GAME — Self-Evolving Consciousness Training Game Framework
//!
//! Core primitives: Actor, Episode, Rubric, Arena.
//!
//! Inspired by SPIRAL (self-play), MARSHAL (turn-level advantage),
//! legos (Actor/Episode/Rubric/Arena abstractions), and SIMPLE (game primitives).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════

pub type ActorId = u32;
pub type EpisodeId = u64;
pub type TurnIndex = usize;

/// Player roles in a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Player,
    Opponent,
    Designer,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Player => write!(f, "Player"),
            Role::Opponent => write!(f, "Opponent"),
            Role::Designer => write!(f, "Designer"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Action / Observation
// ═══════════════════════════════════════════════════════════════════

/// A game action — the agent's output for one turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// The action label (e.g., "Move", "Claim", "Transform").
    pub kind: String,
    /// JSON-serializable parameters.
    pub params: serde_json::Value,
    /// The actor who generated this action.
    pub actor_id: ActorId,
}

/// Text-native observation returned by an environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Text description of the current game state (LLM-TeamGym pattern).
    pub text: String,
    /// Legal actions available in this state.
    pub legal_actions: Vec<Action>,
    /// Optional: E8 hexagram encoding of the state (0-63).
    pub hexagram: Option<u8>,
    /// Optional: Phi value of the current state.
    pub phi: Option<f64>,
}

/// Result of stepping an environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// The observation after the action.
    pub observation: Observation,
    /// Immediate reward.
    pub reward: f64,
    /// Whether the episode is done.
    pub done: bool,
    /// Additional info (e.g., debug metrics).
    pub info: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════
// Actor
// ═══════════════════════════════════════════════════════════════════

/// An actor in a game episode — receives observations and produces actions.
///
/// From SPIRAL: both players share the same policy π_θ, with role conditioning
/// via system prompts. From legos: clean separation of actor from policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: ActorId,
    pub role: Role,
    pub system_prompt: String,
    /// Whether this actor shares policy with other actors (self-play).
    pub shared_policy: bool,
}

impl Actor {
    pub fn new(id: ActorId, role: Role, system_prompt: impl Into<String>) -> Self {
        Self {
            id,
            role,
            system_prompt: system_prompt.into(),
            shared_policy: true,
        }
    }

    /// Create a self-play pair (two actors sharing the same policy).
    pub fn self_play_pair(system_prompt: impl Into<String>) -> (Self, Self) {
        let prompt = system_prompt.into();
        let a = Self {
            id: 0,
            role: Role::Player,
            system_prompt: prompt.clone(),
            shared_policy: true,
        };
        let b = Self {
            id: 1,
            role: Role::Opponent,
            system_prompt: prompt,
            shared_policy: true,
        };
        (a, b)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Trajectory
// ═══════════════════════════════════════════════════════════════════

/// One step in a trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub turn: TurnIndex,
    pub actor_id: ActorId,
    pub observation: Observation,
    pub action: Action,
    pub reward: f64,
    /// Per-turn advantage estimate (filled during advantage computation).
    pub advantage: Option<f64>,
}

/// A complete trajectory from one episode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Trajectory {
    pub steps: Vec<TrajectoryStep>,
    pub total_reward: f64,
    pub length: usize,
}

impl Trajectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, step: TrajectoryStep) {
        self.total_reward += step.reward;
        self.length += 1;
        self.steps.push(step);
    }

    /// Get all steps for a specific actor.
    pub fn for_actor(&self, actor_id: ActorId) -> Vec<&TrajectoryStep> {
        self.steps
            .iter()
            .filter(|s| s.actor_id == actor_id)
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Rubric
// ═══════════════════════════════════════════════════════════════════

/// Reward function trait — evaluates trajectories to produce scores.
///
/// From legos: rubrics are registered to episodes and evaluated at the end.
pub trait RewardFn: Send + Sync {
    fn name(&self) -> &str;
    /// Score a trajectory. Returns a value in [0, 1].
    fn score(&self, trajectory: &Trajectory) -> f64;
}

/// A composite rubric combining multiple reward functions with weights.
pub struct Rubric {
    pub reward_fns: Vec<Box<dyn RewardFn>>,
    pub weights: Vec<f64>,
}

impl Rubric {
    pub fn new() -> Self {
        Self {
            reward_fns: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add(&mut self, rf: Box<dyn RewardFn>, weight: f64) {
        self.reward_fns.push(rf);
        self.weights.push(weight);
    }

    /// Evaluate all reward functions and compute weighted average.
    pub fn evaluate(&self, trajectory: &Trajectory) -> f64 {
        if self.reward_fns.is_empty() {
            return 0.0;
        }
        let total_weight: f64 = self.weights.iter().sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        let total: f64 = self
            .reward_fns
            .iter()
            .zip(self.weights.iter())
            .map(|(rf, w)| rf.score(trajectory) * w)
            .sum();
        total / total_weight
    }
}

impl Default for Rubric {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Built-in Reward Functions
// ═══════════════════════════════════════════════════════════════════

/// Simple win/loss reward: +1 for win, -1 for loss, 0 for draw.
pub struct WinLossReward;

impl RewardFn for WinLossReward {
    fn name(&self) -> &str {
        "win_loss"
    }
    fn score(&self, trajectory: &Trajectory) -> f64 {
        let last_step = trajectory.steps.last();
        match last_step.map(|s| s.reward) {
            Some(r) if r > 0.0 => 1.0,
            Some(r) if r < 0.0 => 0.0,
            _ => 0.5,
        }
    }
}

/// Length bonus: shorter episodes get higher scores (efficiency).
pub struct EfficiencyReward {
    pub max_turns: f64,
}

impl RewardFn for EfficiencyReward {
    fn name(&self) -> &str {
        "efficiency"
    }
    fn score(&self, trajectory: &Trajectory) -> f64 {
        let len = trajectory.length as f64;
        (1.0 - len / self.max_turns).clamp(0.0, 1.0)
    }
}

/// Resonance reward: bonus for states with high hexagram resonance.
pub struct ResonanceReward {
    pub resonance_counts: Vec<f64>,
}

impl RewardFn for ResonanceReward {
    fn name(&self) -> &str {
        "resonance"
    }
    fn score(&self, _trajectory: &Trajectory) -> f64 {
        if self.resonance_counts.is_empty() {
            return 0.0;
        }
        let avg = self.resonance_counts.iter().sum::<f64>() / self.resonance_counts.len() as f64;
        (avg / 6.0).clamp(0.0, 1.0)
    }
}

/// Phi integration reward: bonus for maintaining high Phi values.
pub struct PhiReward {
    pub phi_values: Vec<f64>,
}

impl RewardFn for PhiReward {
    fn name(&self) -> &str {
        "phi"
    }
    fn score(&self, _trajectory: &Trajectory) -> f64 {
        if self.phi_values.is_empty() {
            return 0.0;
        }
        let avg = self.phi_values.iter().sum::<f64>() / self.phi_values.len() as f64;
        avg.clamp(0.0, 1.0)
    }
}

/// Strategic depth reward: based on number of distinct hexagram states visited.
pub struct StrategicDepthReward;

impl RewardFn for StrategicDepthReward {
    fn name(&self) -> &str {
        "strategic_depth"
    }
    fn score(&self, trajectory: &Trajectory) -> f64 {
        let distinct: std::collections::HashSet<_> = trajectory
            .steps
            .iter()
            .filter_map(|s| s.observation.hexagram)
            .collect();
        let count = distinct.len() as f64;
        // Normalize by max possible (64 hexagrams)
        (count / 64.0).clamp(0.0, 1.0)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Episode
// ═══════════════════════════════════════════════════════════════════

/// Metadata about a completed or running episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMeta {
    pub id: EpisodeId,
    pub game_name: String,
    pub constellation_level: u8,
    pub seed: Option<u64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration: Option<Duration>,
}

/// A self-contained game episode — launches actors, collects trajectory, evaluates.
///
/// From legos: episodes are the core unit of interaction with a game environment.
/// From SIMPLE: episodes track metadata for reproducibility.
pub struct Episode {
    pub meta: EpisodeMeta,
    pub actors: Vec<Actor>,
    pub rubric: Rubric,
    pub trajectory: Trajectory,
    /// Maximum number of turns before forced termination.
    pub max_turns: usize,
}

impl Clone for Episode {
    fn clone(&self) -> Self {
        Self {
            meta: self.meta.clone(),
            actors: self.actors.clone(),
            rubric: Rubric::new(),
            trajectory: self.trajectory.clone(),
            max_turns: self.max_turns,
        }
    }
}

impl Episode {
    pub fn new(
        id: EpisodeId,
        game_name: impl Into<String>,
        actors: Vec<Actor>,
        rubric: Rubric,
        max_turns: usize,
        seed: Option<u64>,
    ) -> Self {
        Self {
            meta: EpisodeMeta {
                id,
                game_name: game_name.into(),
                constellation_level: 0,
                seed,
                started_at: None,
                finished_at: None,
                duration: None,
            },
            actors,
            rubric,
            trajectory: Trajectory::new(),
            max_turns,
        }
    }

    /// Record a turn into the trajectory.
    pub fn record_turn(&mut self, step: TrajectoryStep) {
        self.trajectory.push(step);
    }

    /// Check if the episode has exceeded its turn limit.
    pub fn is_turn_limit_reached(&self) -> bool {
        self.trajectory.length >= self.max_turns
    }

    /// Finalize the episode — compute final scores.
    pub fn finalize(&mut self) -> EpisodeResult {
        let score = self.rubric.evaluate(&self.trajectory);
        EpisodeResult {
            meta: self.meta.clone(),
            score,
            trajectory_reward: self.trajectory.total_reward,
            turns_played: self.trajectory.length,
        }
    }
}

/// Result of a completed episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeResult {
    pub meta: EpisodeMeta,
    pub score: f64,
    pub trajectory_reward: f64,
    pub turns_played: usize,
}

// ═══════════════════════════════════════════════════════════════════
// Arena
// ═══════════════════════════════════════════════════════════════════

/// Configuration for the Arena.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaConfig {
    /// Maximum parallel episodes.
    pub max_parallel: usize,
    /// Default maximum turns per episode.
    pub default_max_turns: usize,
    /// Default random seed.
    pub default_seed: Option<u64>,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            max_parallel: 1,
            default_max_turns: 50,
            default_seed: None,
        }
    }
}

/// Stateful container that launches and manages episodes.
///
/// From legos: the arena is the top-level entry point that holds
/// persistent state (artifact store) and launches episodes.
pub struct Arena {
    pub config: ArenaConfig,
    /// Global episode counter for ID generation.
    next_episode_id: EpisodeId,
    /// Results from completed episodes.
    results: Vec<EpisodeResult>,
}

impl Arena {
    pub fn new(config: ArenaConfig) -> Self {
        Self {
            config,
            next_episode_id: 0,
            results: Vec::new(),
        }
    }

    /// Launch a new episode with the given actors and rubric.
    pub fn launch(
        &mut self,
        game_name: impl Into<String>,
        actors: Vec<Actor>,
        rubric: Rubric,
        max_turns: Option<usize>,
        seed: Option<u64>,
    ) -> Episode {
        let id = self.next_episode_id;
        self.next_episode_id += 1;
        Episode::new(
            id,
            game_name,
            actors,
            rubric,
            max_turns.unwrap_or(self.config.default_max_turns),
            seed.or(self.config.default_seed),
        )
    }

    /// Record a completed episode result.
    pub fn record_result(&mut self, result: EpisodeResult) {
        self.results.push(result);
    }

    /// Get all completed episode results.
    pub fn results(&self) -> &[EpisodeResult] {
        &self.results
    }

    /// Compute aggregate statistics over completed episodes.
    pub fn stats(&self) -> ArenaStats {
        if self.results.is_empty() {
            return ArenaStats::default();
        }
        let scores: Vec<f64> = self.results.iter().map(|r| r.score).collect();
        let rewards: Vec<f64> = self.results.iter().map(|r| r.trajectory_reward).collect();
        let turns: Vec<f64> = self.results.iter().map(|r| r.turns_played as f64).collect();
        ArenaStats {
            total_episodes: self.results.len(),
            avg_score: scores.iter().sum::<f64>() / scores.len() as f64,
            avg_reward: rewards.iter().sum::<f64>() / rewards.len() as f64,
            avg_turns: turns.iter().sum::<f64>() / turns.len() as f64,
            best_score: scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            worst_score: scores.iter().cloned().fold(f64::INFINITY, f64::min),
        }
    }
}

/// Aggregate statistics over completed episodes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArenaStats {
    pub total_episodes: usize,
    pub avg_score: f64,
    pub avg_reward: f64,
    pub avg_turns: f64,
    pub best_score: f64,
    pub worst_score: f64,
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_self_play_pair() {
        let (a, b) = Actor::self_play_pair("You are a player.");
        assert_eq!(a.role, Role::Player);
        assert_eq!(b.role, Role::Opponent);
        assert!(a.shared_policy);
        assert!(b.shared_policy);
        assert_eq!(a.system_prompt, b.system_prompt);
    }

    #[test]
    fn test_trajectory() {
        let mut t = Trajectory::new();
        assert_eq!(t.length, 0);
        assert_eq!(t.total_reward, 0.0);
        t.push(TrajectoryStep {
            turn: 0,
            actor_id: 0,
            observation: Observation {
                text: "state".into(),
                legal_actions: vec![],
                hexagram: None,
                phi: None,
            },
            action: Action {
                kind: "move".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
            reward: 1.0,
            advantage: None,
        });
        assert_eq!(t.length, 1);
        assert!((t.total_reward - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rubric_evaluate() {
        let mut rubric = Rubric::new();
        rubric.add(Box::new(WinLossReward), 1.0);
        let mut traj = Trajectory::new();
        traj.push(TrajectoryStep {
            turn: 0,
            actor_id: 0,
            observation: Observation {
                text: "".into(),
                legal_actions: vec![],
                hexagram: None,
                phi: None,
            },
            action: Action {
                kind: "pass".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
            reward: 1.0,
            advantage: None,
        });
        let score = rubric.evaluate(&traj);
        assert!((score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_arena_launch_and_record() {
        let mut arena = Arena::new(ArenaConfig::default());
        let (a, b) = Actor::self_play_pair("test");
        let rubric = Rubric::new();
        let ep = arena.launch("test_game", vec![a, b], rubric, None, Some(42));
        assert_eq!(ep.meta.id, 0);
        assert_eq!(ep.meta.game_name, "test_game");
        assert_eq!(ep.meta.seed, Some(42));

        let result = EpisodeResult {
            meta: ep.meta.clone(),
            score: 0.8,
            trajectory_reward: 5.0,
            turns_played: 10,
        };
        arena.record_result(result);
        let stats = arena.stats();
        assert_eq!(stats.total_episodes, 1);
        assert!((stats.avg_score - 0.8).abs() < 1e-10);
    }
}
