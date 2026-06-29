use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotivationAxes {
    pub competence: f64,
    pub autonomy: f64,
    pub relatedness: f64,
    pub learning_progress: f64,
    pub information_gain: f64,
    pub weights: HashMap<String, f64>,
    decay_rates: HashMap<String, f64>,
    competence_window: Vec<bool>,
}

impl MotivationAxes {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        let mut decay_rates = HashMap::new();
        for name in Self::axis_names() {
            weights.insert(name.to_string(), 1.0);
            decay_rates.insert(name.to_string(), 0.01);
        }
        Self {
            competence: 0.5,
            autonomy: 0.5,
            relatedness: 0.5,
            learning_progress: 0.5,
            information_gain: 0.5,
            weights,
            decay_rates,
            competence_window: Vec::with_capacity(20),
        }
    }

    pub fn update_competence(&mut self, success: bool) {
        self.competence_window.push(success);
        if self.competence_window.len() > 20 {
            self.competence_window.remove(0);
        }
        let wins = self.competence_window.iter().filter(|&&x| x).count() as f64;
        let total = self.competence_window.len() as f64;
        self.competence = if total > 0.0 { wins / total } else { 0.5 };
    }

    pub fn update_autonomy(&mut self, self_initiated: bool, total: u64) {
        let prev = self.autonomy;
        let raw = if total > 0 {
            if self_initiated {
                1.0
            } else {
                0.0
            }
        } else {
            0.5
        };
        let decay = self.decay_rates.get("autonomy").copied().unwrap_or(0.01);
        self.autonomy = prev * (1.0 - decay) + raw * 0.3;
        self.autonomy = self.autonomy.clamp(0.0, 1.0);
    }

    pub fn update_learning_progress(
        &mut self,
        prediction_error_before: f64,
        prediction_error_after: f64,
    ) {
        let raw = (prediction_error_before - prediction_error_after).clamp(0.0, 1.0);
        let decay = self
            .decay_rates
            .get("learning_progress")
            .copied()
            .unwrap_or(0.01);
        self.learning_progress = self.learning_progress * (1.0 - decay) + raw * 0.3;
        self.learning_progress = self.learning_progress.clamp(0.0, 1.0);
    }

    pub fn update_information_gain(&mut self, entropy_before: f64, entropy_after: f64) {
        let raw = (entropy_before - entropy_after).clamp(0.0, 1.0);
        let decay = self
            .decay_rates
            .get("information_gain")
            .copied()
            .unwrap_or(0.01);
        self.information_gain = self.information_gain * (1.0 - decay) + raw * 0.3;
        self.information_gain = self.information_gain.clamp(0.0, 1.0);
    }

    pub fn update_relatedness(&mut self, sentiment: f64) {
        let decay = self.decay_rates.get("relatedness").copied().unwrap_or(0.01);
        self.relatedness = self.relatedness * (1.0 - decay) + sentiment.abs() * 0.2;
        self.relatedness = self.relatedness.clamp(0.0, 1.0);
    }

    pub fn tick(&mut self) {
        self.competence *= 1.0 - self.decay_rates.get("competence").copied().unwrap_or(0.01);
        self.autonomy *= 1.0 - self.decay_rates.get("autonomy").copied().unwrap_or(0.01);
        self.relatedness *= 1.0 - self.decay_rates.get("relatedness").copied().unwrap_or(0.01);
        self.learning_progress *= 1.0
            - self
                .decay_rates
                .get("learning_progress")
                .copied()
                .unwrap_or(0.01);
        self.information_gain *= 1.0
            - self
                .decay_rates
                .get("information_gain")
                .copied()
                .unwrap_or(0.01);
    }

    pub fn composite_score(&self) -> f64 {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        for name in Self::axis_names() {
            let w = self.weights.get(name).copied().unwrap_or(1.0);
            let v = self.get(name);
            weighted_sum += w * v;
            total_weight += w;
        }
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    pub fn axis_names() -> Vec<&'static str> {
        vec![
            "competence",
            "autonomy",
            "relatedness",
            "learning_progress",
            "information_gain",
        ]
    }

    pub fn set_weight(&mut self, name: &str, weight: f64) {
        self.weights.insert(name.to_string(), weight);
    }

    pub fn set_decay(&mut self, name: &str, rate: f64) {
        self.decay_rates.insert(name.to_string(), rate);
    }

    pub fn get(&self, name: &str) -> f64 {
        match name {
            "competence" => self.competence,
            "autonomy" => self.autonomy,
            "relatedness" => self.relatedness,
            "learning_progress" => self.learning_progress,
            "information_gain" => self.information_gain,
            _ => 0.0,
        }
    }
}

impl Default for MotivationAxes {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityScheduler {
    pub base_curiosity: f64,
    pub novelty_bonus: f64,
    pub epsilon: f64,
    pub total_explore: u64,
    pub total_exploit: u64,
    pub epsilon_decay: f64,
}

impl CuriosityScheduler {
    pub fn new() -> Self {
        Self {
            base_curiosity: 0.3,
            novelty_bonus: 0.0,
            epsilon: 0.3,
            total_explore: 0,
            total_exploit: 0,
            epsilon_decay: 0.001,
        }
    }

    pub fn compute_curiosity(&self, axes: &MotivationAxes, max_similarity_to_memory: f64) -> f64 {
        let lp_weight = axes
            .weights
            .get("learning_progress")
            .copied()
            .unwrap_or(1.0);
        let ig_weight = axes.weights.get("information_gain").copied().unwrap_or(1.0);
        let lp = lp_weight * axes.learning_progress;
        let ig = ig_weight * axes.information_gain;
        let novelty = self.novelty_bonus * (1.0 - max_similarity_to_memory);
        lp + ig + novelty
    }

    pub fn schedule_action(&mut self) -> bool {
        let explore = rand::random::<f64>() < self.epsilon;
        if explore {
            self.total_explore += 1;
        } else {
            self.total_exploit += 1;
        }
        explore
    }

    pub fn decay_epsilon(&mut self) {
        self.epsilon = (self.epsilon * (1.0 - self.epsilon_decay)).max(0.01);
    }

    pub fn exploration_rate(&self) -> f64 {
        let total = self.total_explore + self.total_exploit;
        if total > 0 {
            self.total_explore as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn set_exploration_mode(&mut self, mode: &str) {
        self.epsilon = match mode {
            "greedy" => 0.01,
            "exploratory" => 0.5,
            "balanced" => 0.2,
            _ => self.epsilon,
        };
    }
}

impl Default for CuriosityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardArbiter {
    pub intrinsic_weight: f64,
    pub extrinsic_weight: f64,
    pub combined_drive: f64,
    pub last_decision: String,
}

impl RewardArbiter {
    pub fn new() -> Self {
        Self {
            intrinsic_weight: 0.5,
            extrinsic_weight: 0.5,
            combined_drive: 0.0,
            last_decision: String::new(),
        }
    }

    pub fn arbitrate(&mut self, intrinsic: f64, extrinsic: f64, context: &str) -> f64 {
        let (iw, ew) = match context {
            "explicit_instruction" => (0.2, 0.8),
            "idle" => (1.0, 0.0),
            "high_load" => (0.3, 0.3),
            _ => (0.5, 0.5),
        };
        self.intrinsic_weight = iw;
        self.extrinsic_weight = ew;
        self.combined_drive = intrinsic * iw + extrinsic * ew;
        self.last_decision = context.to_string();
        self.combined_drive
    }

    pub fn set_weights(&mut self, intrinsic: f64, extrinsic: f64) {
        self.intrinsic_weight = intrinsic.clamp(0.0, 1.0);
        self.extrinsic_weight = extrinsic.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.intrinsic_weight = 0.5;
        self.extrinsic_weight = 0.5;
        self.combined_drive = 0.0;
        self.last_decision = String::new();
    }
}

impl Default for RewardArbiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrinsicMotivationEngine {
    pub axes: MotivationAxes,
    pub curiosity: CuriosityScheduler,
    pub arbiter: RewardArbiter,
    pub cycle_count: u64,
}

impl IntrinsicMotivationEngine {
    pub fn new() -> Self {
        Self {
            axes: MotivationAxes::new(),
            curiosity: CuriosityScheduler::new(),
            arbiter: RewardArbiter::new(),
            cycle_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.cycle_count += 1;
        self.axes.tick();
        self.curiosity.decay_epsilon();
    }

    pub fn explore_or_exploit(&mut self, max_similarity: f64) -> bool {
        let _curiosity = self.curiosity_score(max_similarity);
        self.curiosity.schedule_action()
    }

    pub fn process_outcome(
        &mut self,
        success: bool,
        self_initiated: bool,
        total_actions: u64,
        pred_error_before: f64,
        pred_error_after: f64,
        entropy_before: f64,
        entropy_after: f64,
        sentiment: f64,
    ) {
        self.axes.update_competence(success);
        self.axes.update_autonomy(self_initiated, total_actions);
        self.axes
            .update_learning_progress(pred_error_before, pred_error_after);
        self.axes
            .update_information_gain(entropy_before, entropy_after);
        self.axes.update_relatedness(sentiment);
    }

    pub fn current_drive(&mut self, context: &str, extrinsic_reward: f64) -> f64 {
        self.arbiter
            .arbitrate(self.axes.composite_score(), extrinsic_reward, context)
    }

    pub fn curiosity_score(&self, max_similarity: f64) -> f64 {
        self.curiosity.compute_curiosity(&self.axes, max_similarity)
    }

    pub fn exploration_rate(&self) -> f64 {
        self.curiosity.exploration_rate()
    }

    pub fn motivation_report(&self) -> HashMap<String, f64> {
        let mut report = HashMap::new();
        report.insert("competence".to_string(), self.axes.competence);
        report.insert("autonomy".to_string(), self.axes.autonomy);
        report.insert("relatedness".to_string(), self.axes.relatedness);
        report.insert("learning_progress".to_string(), self.axes.learning_progress);
        report.insert("information_gain".to_string(), self.axes.information_gain);
        report.insert("composite".to_string(), self.axes.composite_score());
        report.insert(
            "curiosity".to_string(),
            self.curiosity.compute_curiosity(&self.axes, 0.5),
        );
        report.insert("drive".to_string(), self.arbiter.combined_drive);
        report
    }
}

impl Default for IntrinsicMotivationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motivation_axes_new() {
        let axes = MotivationAxes::new();
        assert!((axes.competence - 0.5).abs() < 1e-6);
        assert!((axes.autonomy - 0.5).abs() < 1e-6);
        assert!((axes.relatedness - 0.5).abs() < 1e-6);
        assert!((axes.learning_progress - 0.5).abs() < 1e-6);
        assert!((axes.information_gain - 0.5).abs() < 1e-6);
        for name in MotivationAxes::axis_names() {
            assert!((axes.weights[name] - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_competence_update_win() {
        let mut axes = MotivationAxes::new();
        axes.competence_window = vec![false; 10];
        axes.competence = 0.0;
        for _ in 0..20 {
            axes.update_competence(true);
        }
        assert!((axes.competence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_competence_update_loss() {
        let mut axes = MotivationAxes::new();
        axes.competence_window = vec![true; 10];
        axes.competence = 1.0;
        for _ in 0..20 {
            axes.update_competence(false);
        }
        assert!((axes.competence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_autonomy_update() {
        let mut axes = MotivationAxes::new();
        axes.autonomy = 0.0;
        axes.update_autonomy(true, 1);
        assert!(axes.autonomy > 0.0);
    }

    #[test]
    fn test_learning_progress_update() {
        let mut axes = MotivationAxes::new();
        axes.learning_progress = 0.0;
        axes.update_learning_progress(0.8, 0.2);
        assert!(axes.learning_progress > 0.0);
    }

    #[test]
    fn test_information_gain_update() {
        let mut axes = MotivationAxes::new();
        axes.information_gain = 0.0;
        axes.update_information_gain(0.9, 0.3);
        assert!(axes.information_gain > 0.0);
    }

    #[test]
    fn test_axes_tick_decay() {
        let mut axes = MotivationAxes::new();
        axes.competence = 1.0;
        axes.autonomy = 1.0;
        axes.relatedness = 1.0;
        axes.learning_progress = 1.0;
        axes.information_gain = 1.0;
        axes.tick();
        assert!(axes.competence < 1.0);
        assert!(axes.autonomy < 1.0);
        assert!(axes.relatedness < 1.0);
        assert!(axes.learning_progress < 1.0);
        assert!(axes.information_gain < 1.0);
    }

    #[test]
    fn test_composite_score() {
        let mut axes = MotivationAxes::new();
        axes.competence = 1.0;
        axes.autonomy = 0.0;
        axes.relatedness = 0.0;
        axes.learning_progress = 0.0;
        axes.information_gain = 0.0;
        let score = axes.composite_score();
        assert!((score - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_curiosity_scheduler_new() {
        let c = CuriosityScheduler::new();
        assert!((c.base_curiosity - 0.3).abs() < 1e-6);
        assert!((c.epsilon - 0.3).abs() < 1e-6);
        assert!((c.epsilon_decay - 0.001).abs() < 1e-6);
        assert_eq!(c.total_explore, 0);
        assert_eq!(c.total_exploit, 0);
    }

    #[test]
    fn test_curiosity_schedule_action() {
        let mut c = CuriosityScheduler::new();
        c.epsilon = 1.0;
        let result = c.schedule_action();
        assert!(result);
        assert_eq!(c.total_explore, 1);
        assert_eq!(c.total_exploit, 0);
    }

    #[test]
    fn test_curiosity_compute() {
        let axes = MotivationAxes::new();
        let c = CuriosityScheduler::new();
        let score = c.compute_curiosity(&axes, 0.5);
        assert!(score >= 0.0);
    }

    #[test]
    fn test_reward_arbitrate_idle() {
        let mut arb = RewardArbiter::new();
        let drive = arb.arbitrate(0.8, 0.2, "idle");
        assert!((drive - 0.8).abs() < 1e-6);
        assert!((arb.intrinsic_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reward_arbitrate_instruction() {
        let mut arb = RewardArbiter::new();
        let drive = arb.arbitrate(0.5, 1.0, "explicit_instruction");
        assert!((drive - 0.9).abs() < 1e-6);
        assert!((arb.extrinsic_weight - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_engine_new() {
        let eng = IntrinsicMotivationEngine::new();
        assert_eq!(eng.cycle_count, 0);
        assert!((eng.axes.competence - 0.5).abs() < 1e-6);
        assert!((eng.curiosity.epsilon - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_engine_tick() {
        let mut eng = IntrinsicMotivationEngine::new();
        eng.axes.competence = 1.0;
        eng.tick();
        assert_eq!(eng.cycle_count, 1);
        assert!(eng.axes.competence < 1.0);
        assert!(eng.curiosity.epsilon < 0.3);
    }

    #[test]
    fn test_engine_process_outcome() {
        let mut eng = IntrinsicMotivationEngine::new();
        eng.process_outcome(true, true, 1, 0.8, 0.2, 0.9, 0.3, 0.7);
        assert!(eng.axes.competence > 0.0);
        assert!(eng.axes.autonomy > 0.0);
        assert!(eng.axes.learning_progress > 0.0);
        assert!(eng.axes.information_gain > 0.0);
        assert!(eng.axes.relatedness > 0.0);
    }

    #[test]
    fn test_engine_drive() {
        let mut eng = IntrinsicMotivationEngine::new();
        let drive = eng.current_drive("explicit_instruction", 1.0);
        assert!(drive > 0.0);
    }

    #[test]
    fn test_motivation_report() {
        let mut eng = IntrinsicMotivationEngine::new();
        eng.current_drive("idle", 0.0);
        let report = eng.motivation_report();
        assert!(report.contains_key("competence"));
        assert!(report.contains_key("autonomy"));
        assert!(report.contains_key("relatedness"));
        assert!(report.contains_key("learning_progress"));
        assert!(report.contains_key("information_gain"));
        assert!(report.contains_key("composite"));
        assert!(report.contains_key("curiosity"));
        assert!(report.contains_key("drive"));
    }

    #[test]
    fn test_axex_set_weight() {
        let mut axes = MotivationAxes::new();
        axes.set_weight("competence", 2.0);
        assert!((axes.weights["competence"] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_axex_set_decay() {
        let mut axes = MotivationAxes::new();
        axes.set_decay("competence", 0.1);
        axes.competence = 1.0;
        axes.tick();
        assert!((axes.competence - 0.9) < 0.01);
    }

    #[test]
    fn test_curiosity_exploration_rate() {
        let mut c = CuriosityScheduler::new();
        assert!((c.exploration_rate() - 0.0).abs() < 1e-6);
        c.total_explore = 3;
        c.total_exploit = 1;
        assert!((c.exploration_rate() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_curiosity_set_exploration_mode() {
        let mut c = CuriosityScheduler::new();
        c.set_exploration_mode("greedy");
        assert!((c.epsilon - 0.01).abs() < 1e-6);
        c.set_exploration_mode("exploratory");
        assert!((c.epsilon - 0.5).abs() < 1e-6);
        c.set_exploration_mode("balanced");
        assert!((c.epsilon - 0.2).abs() < 1e-6);
    }
}
