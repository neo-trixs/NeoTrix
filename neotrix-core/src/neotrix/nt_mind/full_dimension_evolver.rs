use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================
// Core Evolution Types
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionAxis {
    Skill,
    Context,
    Brain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRound {
    pub timestamp: u64,
    pub skill_delta: f64,
    pub context_delta: f64,
    pub brain_delta: f64,
    pub external_reward: Option<f64>,
    pub internal_reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTrace {
    pub tool_name: String,
    pub args: Vec<String>,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: u64,
}

// ============================================
// Skill Evolution
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProposal {
    pub name: String,
    pub confidence: f64,
    pub steps: Vec<String>,
    pub source_sequences: Vec<Vec<String>>,
}

pub struct ActionSequenceMiner {
    pub min_sequence_length: usize,
    pub min_frequency: usize,
}

impl ActionSequenceMiner {
    pub fn new(min_sequence_length: usize, min_frequency: usize) -> Self {
        Self {
            min_sequence_length,
            min_frequency,
        }
    }

    pub fn mine(&self, traces: &[ToolTrace]) -> Vec<SkillProposal> {
        let sequences = self.extract_sequences(traces);
        let clusters = self.cluster_sequences(&sequences);
        let total = sequences.len() as f64;

        clusters
            .into_iter()
            .map(|steps| {
                let count = sequences.iter().filter(|s| *s == &steps).count() as f64;
                let confidence = if total > 0.0 { count / total } else { 0.0 };
                let name = steps.join("_");
                SkillProposal {
                    name,
                    confidence,
                    steps: steps.clone(),
                    source_sequences: vec![steps],
                }
            })
            .collect()
    }

    fn extract_sequences(&self, traces: &[ToolTrace]) -> Vec<Vec<String>> {
        let tool_names: Vec<String> = traces
            .iter()
            .filter(|t| t.success)
            .map(|t| t.tool_name.clone())
            .collect();

        if tool_names.len() < self.min_sequence_length {
            return Vec::new();
        }

        tool_names
            .windows(self.min_sequence_length)
            .map(|w| w.to_vec())
            .collect()
    }

    fn cluster_sequences(&self, sequences: &[Vec<String>]) -> Vec<Vec<String>> {
        let mut freq: HashMap<Vec<String>, usize> = HashMap::new();
        for seq in sequences {
            *freq.entry(seq.clone()).or_insert(0) += 1;
        }

        freq.into_iter()
            .filter(|(_, count)| *count >= self.min_frequency)
            .map(|(seq, _)| seq)
            .collect()
    }
}

// ============================================
// Context Evolution
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLayer {
    pub level: u8,
    pub content: String,
    pub max_tokens: usize,
    pub priority: f64,
    pub access_count: u64,
}

pub struct LayerManager {
    pub layers: Vec<ContextLayer>,
    pub active_layers: Vec<usize>,
}

impl LayerManager {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            active_layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, layer: ContextLayer) {
        if self.layers.iter().any(|l| l.level == layer.level) {
            if let Some(existing) = self.layers.iter_mut().find(|l| l.level == layer.level) {
                *existing = layer;
            }
        } else {
            let idx = self.layers.len();
            self.layers.push(layer);
            self.active_layers.push(idx);
        }
    }

    pub fn progressive_load(&self, depth: u8) -> usize {
        self.layers
            .iter()
            .filter(|l| l.level <= depth)
            .map(|l| l.max_tokens)
            .sum()
    }

    pub fn promote_layer(&mut self, layer_idx: usize) {
        if let Some(layer) = self.layers.get_mut(layer_idx) {
            layer.priority = (layer.priority * 1.2).clamp(0.0, 1.0);
            layer.access_count += 1;
            if !self.active_layers.contains(&layer_idx) {
                self.active_layers.push(layer_idx);
            }
        }
    }

    pub fn demote_layer(&mut self, layer_idx: usize) {
        if let Some(layer) = self.layers.get_mut(layer_idx) {
            layer.priority *= 0.8;
            layer.access_count = layer.access_count.saturating_sub(1);
            self.active_layers.retain(|&i| i != layer_idx);
        }
    }

    pub fn active_token_budget(&self) -> usize {
        self.active_layers
            .iter()
            .filter_map(|&idx| self.layers.get(idx))
            .map(|l| l.max_tokens)
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceProfile {
    pub preferred_task_types: Vec<String>,
    pub context_depth_preference: f64,
    pub verbosity_preference: f64,
    pub safety_tolerance: f64,
    pub last_updated: u64,
}

pub struct PreferenceTracker {
    pub profile: PreferenceProfile,
    pub signal_history: Vec<(String, f64, u64)>,
}

impl PreferenceTracker {
    pub fn new() -> Self {
        Self {
            profile: PreferenceProfile {
                preferred_task_types: Vec::new(),
                context_depth_preference: 0.5,
                verbosity_preference: 0.5,
                safety_tolerance: 0.5,
                last_updated: 0,
            },
            signal_history: Vec::new(),
        }
    }

    pub fn record_signal(&mut self, signal_type: &str, value: f64) {
        let ts = Utc::now().timestamp() as u64;
        self.signal_history
            .push((signal_type.to_string(), value, ts));
    }

    pub fn update_profile(&mut self) {
        let mut depth_sum = 0.0f64;
        let mut depth_count = 0usize;
        let mut verb_sum = 0.0f64;
        let mut verb_count = 0usize;
        let mut safety_sum = 0.0f64;
        let mut safety_count = 0usize;
        let mut latest_ts = self.profile.last_updated;

        for (signal_type, value, ts) in &self.signal_history {
            if *ts > latest_ts {
                latest_ts = *ts;
            }
            match signal_type.as_str() {
                "context_depth" => {
                    depth_sum += *value;
                    depth_count += 1;
                }
                "verbosity" => {
                    verb_sum += *value;
                    verb_count += 1;
                }
                "safety" => {
                    safety_sum += *value;
                    safety_count += 1;
                }
                _ => {
                    if signal_type.starts_with("task_type:") {
                        let task = signal_type.trim_start_matches("task_type:");
                        if *value > 0.3
                            && !self
                                .profile
                                .preferred_task_types
                                .contains(&task.to_string())
                        {
                            self.profile.preferred_task_types.push(task.to_string());
                        }
                    }
                }
            }
        }

        if depth_count > 0 {
            self.profile.context_depth_preference = depth_sum / depth_count as f64;
        }
        if verb_count > 0 {
            self.profile.verbosity_preference = verb_sum / verb_count as f64;
        }
        if safety_count > 0 {
            self.profile.safety_tolerance = safety_sum / safety_count as f64;
        }
        self.profile.last_updated = latest_ts;
    }
}

// ============================================
// Brain Evolution
// ============================================

pub struct RewardCollector {
    pub external_rewards: Vec<(String, f64, u64)>,
    pub internal_rewards: Vec<(String, f64, u64)>,
    pub window_size: usize,
}

impl RewardCollector {
    pub fn new(window_size: usize) -> Self {
        Self {
            external_rewards: Vec::new(),
            internal_rewards: Vec::new(),
            window_size,
        }
    }

    fn trim_to_window(rewards: &mut Vec<(String, f64, u64)>, window_size: usize) {
        while rewards.len() > window_size {
            rewards.remove(0);
        }
    }

    pub fn record_external(&mut self, task_type: &str, reward: f64) {
        let ts = Utc::now().timestamp() as u64;
        self.external_rewards
            .push((task_type.to_string(), reward, ts));
        Self::trim_to_window(&mut self.external_rewards, self.window_size);
    }

    pub fn record_internal(&mut self, task_type: &str, reward: f64) {
        let ts = Utc::now().timestamp() as u64;
        self.internal_rewards
            .push((task_type.to_string(), reward, ts));
        Self::trim_to_window(&mut self.internal_rewards, self.window_size);
    }

    pub fn ema_by_task_type(&self, task_type: &str) -> f64 {
        let filtered: Vec<f64> = self
            .external_rewards
            .iter()
            .chain(self.internal_rewards.iter())
            .filter(|(tt, _, _)| tt == task_type)
            .map(|(_, r, _)| *r)
            .collect();

        if filtered.is_empty() {
            return 0.5;
        }

        let alpha = 2.0 / (self.window_size as f64 + 1.0);
        let mut ema = filtered[0];
        for &r in &filtered[1..] {
            ema = alpha * r + (1.0 - alpha) * ema;
        }
        ema
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trend {
    Improving,
    Declining,
    Stable,
}

pub struct PolicyUpdater {
    pub learning_rate: f64,
    pub regularization_weight: f64,
    pub reward_history: Vec<f64>,
}

impl PolicyUpdater {
    pub fn new(learning_rate: f64, regularization_weight: f64) -> Self {
        Self {
            learning_rate,
            regularization_weight,
            reward_history: Vec::new(),
        }
    }

    pub fn adjust(&mut self, recent_rewards: &[f64]) -> (f64, f64) {
        let trend = Self::detect_trend(recent_rewards);

        let avg = if recent_rewards.is_empty() {
            0.0
        } else {
            recent_rewards.iter().sum::<f64>() / recent_rewards.len() as f64
        };
        self.reward_history.push(avg);

        match trend {
            Trend::Improving => {
                self.learning_rate = (self.learning_rate * 1.2).min(0.1);
                self.regularization_weight = (self.regularization_weight * 0.95).max(0.01);
            }
            Trend::Declining => {
                self.learning_rate = (self.learning_rate * 0.8).max(0.001);
                self.regularization_weight = (self.regularization_weight * 1.05).min(1.0);
            }
            Trend::Stable => {}
        }

        (self.learning_rate, self.regularization_weight)
    }

    fn detect_trend(rewards: &[f64]) -> Trend {
        if rewards.len() < 2 {
            return Trend::Stable;
        }
        let n = rewards.len() as f64;
        let sum_x: f64 = (0..rewards.len()).map(|i| i as f64).sum();
        let sum_y: f64 = rewards.iter().copied().sum();
        let sum_xy: f64 = rewards.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_xx: f64 = (0..rewards.len()).map(|i| (i as f64) * (i as f64)).sum();

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator.abs() < 1e-10 {
            return Trend::Stable;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;

        if slope > 0.01 {
            Trend::Improving
        } else if slope < -0.01 {
            Trend::Declining
        } else {
            Trend::Stable
        }
    }
}

// ============================================
// Full-Dimension Evolution Orchestrator
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub proposals: Vec<SkillProposal>,
    pub skill_delta: f64,
    pub context_delta: f64,
    pub brain_delta: f64,
    pub new_lr: f64,
    pub new_reg: f64,
    pub round: EvolutionRound,
}

pub struct FullDimensionEvolver {
    pub miner: ActionSequenceMiner,
    pub layer_manager: LayerManager,
    pub preference_tracker: PreferenceTracker,
    pub reward_collector: RewardCollector,
    pub policy_updater: PolicyUpdater,
    pub rounds: Vec<EvolutionRound>,
    pub evolution_interval: u64,
    pub last_evolution: u64,
}

impl FullDimensionEvolver {
    pub fn new() -> Self {
        Self {
            miner: ActionSequenceMiner::new(2, 2),
            layer_manager: LayerManager::new(),
            preference_tracker: PreferenceTracker::new(),
            reward_collector: RewardCollector::new(100),
            policy_updater: PolicyUpdater::new(0.01, 0.1),
            rounds: Vec::new(),
            evolution_interval: 10,
            last_evolution: 0,
        }
    }

    pub fn should_evolve(&self, iteration: u64) -> bool {
        iteration >= self.last_evolution + self.evolution_interval
    }

    pub fn evolve_skill(&mut self, traces: &[ToolTrace]) -> Vec<SkillProposal> {
        self.miner.mine(traces)
    }

    pub fn evolve_context(&mut self) {
        self.preference_tracker.update_profile();
    }

    pub fn evolve_brain(&mut self, external_reward: Option<f64>) -> (f64, f64) {
        if let Some(reward) = external_reward {
            self.reward_collector.record_external("general", reward);
        }

        let mut combined: Vec<f64> = self
            .reward_collector
            .external_rewards
            .iter()
            .rev()
            .take(5)
            .map(|(_, r, _)| *r)
            .collect();

        combined.extend(
            self.reward_collector
                .internal_rewards
                .iter()
                .rev()
                .take(5)
                .map(|(_, r, _)| *r),
        );

        if combined.is_empty() {
            return (
                self.policy_updater.learning_rate,
                self.policy_updater.regularization_weight,
            );
        }

        self.policy_updater.adjust(&combined)
    }

    pub fn evolve(
        &mut self,
        traces: &[ToolTrace],
        external_reward: Option<f64>,
    ) -> EvolutionResult {
        let proposals = self.evolve_skill(traces);
        self.evolve_context();
        let (new_lr, new_reg) = self.evolve_brain(external_reward);

        let internal_reward = self.reward_collector.ema_by_task_type("general");

        let skill_delta = proposals.len() as f64 * 0.1;
        let context_delta: f64 = self
            .layer_manager
            .layers
            .iter()
            .map(|l| l.priority)
            .sum::<f64>()
            * 0.01;

        let old_lr = self.policy_updater.learning_rate;
        let old_reg = self.policy_updater.regularization_weight;

        self.policy_updater.learning_rate = new_lr;
        self.policy_updater.regularization_weight = new_reg;

        let brain_delta = (new_lr - old_lr).abs() + (new_reg - old_reg).abs();

        let round = EvolutionRound {
            timestamp: Utc::now().timestamp() as u64,
            skill_delta,
            context_delta,
            brain_delta,
            external_reward,
            internal_reward,
        };

        self.rounds.push(round.clone());
        self.last_evolution += 1;

        EvolutionResult {
            proposals,
            skill_delta,
            context_delta,
            brain_delta,
            new_lr,
            new_reg,
            round,
        }
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_empty_traces() {
        let miner = ActionSequenceMiner::new(2, 2);
        let traces: Vec<ToolTrace> = vec![];
        let proposals = miner.mine(&traces);
        assert!(proposals.is_empty());
    }

    #[test]
    fn test_miner_frequent_pattern() {
        let miner = ActionSequenceMiner::new(2, 2);
        let traces = vec![
            ToolTrace {
                tool_name: "search".into(),
                args: vec![],
                success: true,
                duration_ms: 10,
                timestamp: 1,
            },
            ToolTrace {
                tool_name: "read".into(),
                args: vec![],
                success: true,
                duration_ms: 10,
                timestamp: 2,
            },
            ToolTrace {
                tool_name: "search".into(),
                args: vec![],
                success: true,
                duration_ms: 10,
                timestamp: 3,
            },
            ToolTrace {
                tool_name: "read".into(),
                args: vec![],
                success: true,
                duration_ms: 10,
                timestamp: 4,
            },
        ];
        let proposals = miner.mine(&traces);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].steps, vec!["search", "read"]);
        assert!(proposals[0].confidence > 0.0);
    }

    #[test]
    fn test_layer_manager_progressive_load() {
        let mut lm = LayerManager::new();
        lm.add_layer(ContextLayer {
            level: 0,
            content: "system".into(),
            max_tokens: 1000,
            priority: 1.0,
            access_count: 0,
        });
        lm.add_layer(ContextLayer {
            level: 1,
            content: "task".into(),
            max_tokens: 2000,
            priority: 0.5,
            access_count: 0,
        });
        lm.add_layer(ContextLayer {
            level: 2,
            content: "archive".into(),
            max_tokens: 4000,
            priority: 0.1,
            access_count: 0,
        });

        assert_eq!(lm.progressive_load(0), 1000);
        assert_eq!(lm.progressive_load(1), 3000);
        assert_eq!(lm.progressive_load(2), 7000);
    }

    #[test]
    fn test_layer_manager_promote_demote() {
        let mut lm = LayerManager::new();
        lm.add_layer(ContextLayer {
            level: 0,
            content: "sys".into(),
            max_tokens: 100,
            priority: 0.5,
            access_count: 0,
        });

        lm.promote_layer(0);
        assert!(lm.layers[0].priority > 0.5);
        assert_eq!(lm.layers[0].access_count, 1);

        let prior = lm.layers[0].priority;
        lm.demote_layer(0);
        assert!(lm.layers[0].priority < prior);
    }

    #[test]
    fn test_preference_tracker_update() {
        let mut tracker = PreferenceTracker::new();
        tracker.record_signal("context_depth", 0.3);
        tracker.record_signal("context_depth", 0.7);
        tracker.record_signal("verbosity", 0.5);
        tracker.record_signal("safety", 0.9);

        tracker.update_profile();

        assert!((tracker.profile.context_depth_preference - 0.5).abs() < 0.01);
        assert!((tracker.profile.verbosity_preference - 0.5).abs() < 0.01);
        assert!((tracker.profile.safety_tolerance - 0.9).abs() < 0.01);
        assert!(tracker.profile.last_updated > 0);
    }

    #[test]
    fn test_reward_collector_ema() {
        let mut rc = RewardCollector::new(3);
        rc.record_external("code", 1.0);
        rc.record_external("code", 0.8);
        rc.record_external("code", 0.6);
        let ema = rc.ema_by_task_type("code");
        // alpha = 2/(3+1) = 0.5
        // EMA = 0.5*0.6 + 0.5*0.8 = 0.5*0.6 + 0.5*0.9
        // Step by step: 1.0 -> 0.5*0.8 + 0.5*1.0 = 0.4+0.5=0.9
        //                     -> 0.5*0.6 + 0.5*0.9 = 0.3+0.45=0.75
        assert!((ema - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_policy_updater_adjust_improving() {
        let mut updater = PolicyUpdater::new(0.01, 0.1);
        let rewards = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let (new_lr, _new_reg) = updater.adjust(&rewards);
        assert!(new_lr > 0.01);
    }

    #[test]
    fn test_policy_updater_adjust_declining() {
        let mut updater = PolicyUpdater::new(0.01, 0.1);
        let rewards = vec![1.0, 0.8, 0.6, 0.4, 0.2];
        let (new_lr, _new_reg) = updater.adjust(&rewards);
        assert!(new_lr < 0.01);
    }

    #[test]
    fn test_evolver_should_evolve() {
        let evolver = FullDimensionEvolver::new();
        assert!(evolver.should_evolve(10));
        assert!(!evolver.should_evolve(5));
        assert!(!evolver.should_evolve(9));
        assert!(evolver.should_evolve(100));
    }

    #[test]
    fn test_full_round() {
        let mut evolver = FullDimensionEvolver::new();
        let traces = vec![
            ToolTrace {
                tool_name: "search".into(),
                args: vec![],
                success: true,
                duration_ms: 10,
                timestamp: 1,
            },
            ToolTrace {
                tool_name: "read".into(),
                args: vec![],
                success: true,
                duration_ms: 20,
                timestamp: 2,
            },
        ];

        evolver.reward_collector.record_internal("general", 0.7);

        let result = evolver.evolve(&traces, Some(0.8));

        assert_eq!(evolver.rounds.len(), 1);
        assert!(result.skill_delta >= 0.0);
        assert!(result.context_delta >= 0.0);
        assert!(result.brain_delta >= 0.0);
        assert!(result.new_lr > 0.0);
        assert!(result.new_reg > 0.0);
        assert_eq!(result.round.external_reward, Some(0.8));
        assert!((result.round.internal_reward - 0.798).abs() < 0.01);
    }
}
