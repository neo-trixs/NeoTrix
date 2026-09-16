//! # EvolvingEvaluator — Self-Improving Evaluation
//!
//! Self-improving evaluation from RQGM.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCriteria {
    pub name: String,
    pub weight: f64,
    pub threshold: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub criteria_scores: HashMap<String, f64>,
    pub weighted_score: f64,
    pub raw_score: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub criteria_name: String,
    pub old_weight: f64,
    pub new_weight: f64,
    pub reason: String,
    pub timestamp: u64,
}

pub struct EvolvingEvaluator {
    criteria: Arc<Mutex<Vec<EvaluationCriteria>>>,
    scores: Arc<Mutex<Vec<ScoreBreakdown>>>,
    evolution_log: Arc<Mutex<Vec<EvolutionEvent>>>,
    learning_rate: f64,
    min_weight: f64,
    max_weight: f64,
}

impl EvolvingEvaluator {
    pub fn new(initial_criteria: Vec<EvaluationCriteria>, learning_rate: f64) -> Self {
        Self {
            criteria: Arc::new(Mutex::new(initial_criteria)),
            scores: Arc::new(Mutex::new(Vec::new())),
            evolution_log: Arc::new(Mutex::new(Vec::new())),
            learning_rate,
            min_weight: 0.01,
            max_weight: 1.0,
        }
    }

    pub fn evaluate(&self, _target: &str, actual: &HashMap<String, f64>) -> ScoreBreakdown {
        let criteria = self.criteria.lock().unwrap();
        let mut criteria_scores = HashMap::new();
        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;
        for crit in criteria.iter() {
            let actual_value = actual.get(&crit.name).copied().unwrap_or(0.0);
            let score = self.compute_criterion_score(actual_value, crit.threshold);
            criteria_scores.insert(crit.name.clone(), score);
            weighted_score += score * crit.weight;
            total_weight += crit.weight;
        }
        let raw_score = if !criteria_scores.is_empty() {
            criteria_scores.values().sum::<f64>() / criteria_scores.len() as f64
        } else { 0.0 };
        let weighted_score = if total_weight > 0.0 { weighted_score / total_weight } else { 0.0 };
        let confidence = self.compute_confidence(&criteria_scores, &criteria);
        let breakdown = ScoreBreakdown { criteria_scores, weighted_score, raw_score, confidence };
        self.scores.lock().unwrap().push(breakdown.clone());
        breakdown
    }

    pub fn evolve_criteria(&self) {
        let mut criteria = self.criteria.lock().unwrap();
        let scores = self.scores.lock().unwrap();
        if scores.is_empty() { return; }
        let recent_scores: Vec<&ScoreBreakdown> = scores.iter().rev().take(10).collect();
        for crit in criteria.iter_mut() {
            let avg_performance: f64 = recent_scores.iter().filter_map(|s| s.criteria_scores.get(&crit.name).copied()).sum::<f64>() / recent_scores.len() as f64;
            let adjustment = if avg_performance > crit.threshold {
                self.learning_rate * (avg_performance - crit.threshold)
            } else {
                -self.learning_rate * (crit.threshold - avg_performance)
            };
            let old_weight = crit.weight;
            crit.weight = (crit.weight + adjustment).clamp(self.min_weight, self.max_weight);
            crit.threshold = (crit.threshold + adjustment * 0.5).clamp(0.0, 1.0);
            if (crit.weight - old_weight).abs() > 0.001 {
                self.evolution_log.lock().unwrap().push(EvolutionEvent {
                    criteria_name: crit.name.clone(),
                    old_weight,
                    new_weight: crit.weight,
                    reason: format!("Performance-based adjustment (avg: {:.3})", avg_performance),
                    timestamp: get_timestamp(),
                });
            }
        }
    }

    pub fn get_score(&self) -> f64 {
        let scores = self.scores.lock().unwrap();
        scores.last().map(|s| s.weighted_score).unwrap_or(0.0)
    }

    fn compute_criterion_score(&self, actual: f64, threshold: f64) -> f64 {
        if threshold <= 0.0 { return if actual > 0.0 { 1.0 } else { 0.0 }; }
        let ratio = actual / threshold;
        ratio.min(1.0)
    }

    fn compute_confidence(&self, scores: &HashMap<String, f64>, criteria: &[EvaluationCriteria]) -> f64 {
        if scores.is_empty() || criteria.is_empty() { return 0.0; }
        let mean: f64 = scores.values().sum::<f64>() / scores.len() as f64;
        let variance: f64 = scores.values().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
        1.0 / (1.0 + variance)
    }

    pub fn get_evolution_log(&self) -> Vec<EvolutionEvent> {
        self.evolution_log.lock().unwrap().clone()
    }

    pub fn evaluation_count(&self) -> usize {
        self.scores.lock().unwrap().len()
    }

    pub fn add_criterion(&self, name: String, weight: f64, threshold: f64, description: String) {
        let mut criteria = self.criteria.lock().unwrap();
        criteria.push(EvaluationCriteria { name, weight, threshold, description });
    }

    pub fn remove_criterion(&self, name: &str) -> bool {
        let mut criteria = self.criteria.lock().unwrap();
        let len_before = criteria.len();
        criteria.retain(|c| c.name != name);
        criteria.len() < len_before
    }
}

impl Default for EvolvingEvaluator {
    fn default() -> Self {
        Self::new(vec![
            EvaluationCriteria { name: "accuracy".to_string(), weight: 0.4, threshold: 0.8, description: "Accuracy of outputs".to_string() },
            EvaluationCriteria { name: "efficiency".to_string(), weight: 0.3, threshold: 0.7, description: "Resource efficiency".to_string() },
            EvaluationCriteria { name: "robustness".to_string(), weight: 0.3, threshold: 0.6, description: "Robustness under uncertainty".to_string() },
        ], 0.05)
    }
}

fn get_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test] fn test_evaluate() { let e = EvolvingEvaluator::default(); let mut actual = HashMap::new(); actual.insert("accuracy".to_string(), 0.9); actual.insert("efficiency".to_string(), 0.8); actual.insert("robustness".to_string(), 0.7); let score = e.evaluate("target_1", &actual); assert!(score.weighted_score >= 0.0 && score.weighted_score <= 1.0); }
    #[test] fn test_evolve_criteria() { let e = EvolvingEvaluator::default(); let mut actual = HashMap::new(); actual.insert("accuracy".to_string(), 0.95); actual.insert("efficiency".to_string(), 0.9); actual.insert("robustness".to_string(), 0.85); e.evaluate("t1", &actual); e.evolve_criteria(); assert!(e.get_score() >= 0.0); }
    #[test] fn test_get_score() { let e = EvolvingEvaluator::default(); assert_eq!(e.get_score(), 0.0); }
}
