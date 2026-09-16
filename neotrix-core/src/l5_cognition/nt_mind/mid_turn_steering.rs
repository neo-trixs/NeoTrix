//! # MidTurnSteering — Real-Time Course Correction
//!
//! Real-time course correction during agent reasoning.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceMetrics {
    pub cosine_similarity: f64,
    pub semantic_distance: f64,
    pub logical_consistency: f64,
    pub overall_divergence: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SteeringAction {
    Continue,
    Nudge { adjustment_factor: f64 },
    Redirect { new_focus: String },
    Restart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringResult {
    pub action: SteeringAction,
    pub divergence_score: f64,
    pub corrected_trajectory: Vec<String>,
    pub confidence: f64,
}

pub struct MidTurnSteering {
    divergence_threshold: f64,
    max_corrections: usize,
    correction_count: Arc<Mutex<usize>>,
    trajectory_history: Arc<Mutex<Vec<String>>>,
}

impl MidTurnSteering {
    pub fn new(divergence_threshold: f64, max_corrections: usize) -> Self {
        Self {
            divergence_threshold,
            max_corrections,
            correction_count: Arc::new(Mutex::new(0)),
            trajectory_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn steer(&self, current_reasoning: &[String], target_outcome: &str) -> SteeringResult {
        let divergence = self.assess_divergence(current_reasoning, target_outcome);
        let action = if divergence.overall_divergence > self.divergence_threshold {
            self.determine_action(&divergence, current_reasoning)
        } else {
            SteeringAction::Continue
        };
        let corrected_trajectory = match &action {
            SteeringAction::Continue => current_reasoning.to_vec(),
            SteeringAction::Nudge { adjustment_factor } => self.apply_nudge(current_reasoning, *adjustment_factor),
            SteeringAction::Redirect { new_focus } => self.apply_redirect(current_reasoning, new_focus),
            SteeringAction::Restart => vec![target_outcome.to_string()],
        };
        self.record_correction(&action);
        SteeringResult {
            action,
            divergence_score: divergence.overall_divergence,
            corrected_trajectory,
            confidence: 1.0 - divergence.overall_divergence,
        }
    }

    pub fn assess_divergence(&self, current_reasoning: &[String], target_outcome: &str) -> DivergenceMetrics {
        let cosine_similarity = self.compute_cosine_similarity(current_reasoning, target_outcome);
        let semantic_distance = self.compute_semantic_distance(current_reasoning, target_outcome);
        let logical_consistency = self.compute_logical_consistency(current_reasoning);
        let overall_divergence = (1.0 - cosine_similarity) * 0.4 + semantic_distance * 0.35 + (1.0 - logical_consistency) * 0.25;
        DivergenceMetrics {
            cosine_similarity,
            semantic_distance,
            logical_consistency,
            overall_divergence: overall_divergence.min(1.0).max(0.0),
            timestamp: get_timestamp(),
        }
    }

    pub fn correct_trajectory(&self, current_reasoning: &[String], target_outcome: &str) -> Vec<String> {
        let divergence = self.assess_divergence(current_reasoning, target_outcome);
        if divergence.overall_divergence <= self.divergence_threshold {
            return current_reasoning.to_vec();
        }
        let mut corrected = Vec::new();
        corrected.push(target_outcome.to_string());
        let steering = self.steer(current_reasoning, target_outcome);
        for step in &steering.corrected_trajectory {
            if !corrected.contains(step) { corrected.push(step.clone()); }
        }
        corrected
    }

    fn compute_cosine_similarity(&self, reasoning: &[String], target: &str) -> f64 {
        if reasoning.is_empty() { return 0.0; }
        let overlap = reasoning.iter().filter(|s| s.contains(target)).count();
        (overlap as f64) / (reasoning.len() as f64).max(1.0)
    }

    fn compute_semantic_distance(&self, reasoning: &[String], target: &str) -> f64 {
        if reasoning.is_empty() { return 1.0; }
        let match_count = reasoning.iter().filter(|s| s.contains(target)).count();
        1.0 - (match_count as f64) / (reasoning.len() as f64).max(1.0)
    }

    fn compute_logical_consistency(&self, reasoning: &[String]) -> f64 {
        if reasoning.is_empty() { return 0.5; }
        let consistency = if reasoning.len() > 1 {
            reasoning.windows(2).filter(|w| w[0].len() > 0).count() as f64 / reasoning.len() as f64
        } else { 0.5 };
        consistency.min(1.0)
    }

    fn determine_action(&self, divergence: &DivergenceMetrics, reasoning: &[String]) -> SteeringAction {
        if divergence.overall_divergence > 0.8 && reasoning.len() > 3 {
            SteeringAction::Restart
        } else if divergence.overall_divergence > 0.6 {
            SteeringAction::Redirect { new_focus: "core_objective".to_string() }
        } else if divergence.overall_divergence > self.divergence_threshold {
            SteeringAction::Nudge { adjustment_factor: divergence.overall_divergence }
        } else {
            SteeringAction::Continue
        }
    }

    fn apply_nudge(&self, reasoning: &[String], factor: f64) -> Vec<String> {
        reasoning.iter().enumerate().map(|(i, step)| {
            if i % 2 == 0 { format!("{} [adjusted:{}]", step, factor) } else { step.clone() }
        }).collect()
    }

    fn apply_redirect(&self, reasoning: &[String], new_focus: &str) -> Vec<String> {
        let mut result = vec![new_focus.to_string()];
        result.extend(reasoning.iter().cloned());
        result
    }

    fn record_correction(&self, _action: &SteeringAction) {
        let mut count = self.correction_count.lock().unwrap();
        if *count < self.max_corrections { *count += 1; }
    }

    pub fn correction_count(&self) -> usize { *self.correction_count.lock().unwrap() }
    pub fn reset(&self) { *self.correction_count.lock().unwrap() = 0; self.trajectory_history.lock().unwrap().clear(); }
}

impl Default for MidTurnSteering {
    fn default() -> Self { Self::new(0.5, 10) }
}

fn get_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_steer() { let steering = MidTurnSteering::new(0.5, 10); let result = steering.steer(&["step1".to_string(), "step2".to_string()], "goal"); assert!(result.confidence >= 0.0 && result.confidence <= 1.0); }
    #[test] fn test_assess_divergence() { let steering = MidTurnSteering::new(0.5, 10); let metrics = steering.assess_divergence(&["step1".to_string()], "goal"); assert!(metrics.overall_divergence >= 0.0 && metrics.overall_divergence <= 1.0); }
    #[test] fn test_correct_trajectory() { let steering = MidTurnSteering::new(0.5, 10); let corrected = steering.correct_trajectory(&["step1".to_string()], "goal"); assert!(!corrected.is_empty()); }
}
