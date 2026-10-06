//! # EvolvingEvaluator — Self-Improving Evaluation
//!
//! Self-improving evaluation from RQGM.
//!
//! Note: `coordination::quality_control::evolving_evaluator::EvolvingEvaluator`
//! is a separate baseline-relative tracker, not this type. This is the
//! canonical threshold-gated evaluator wired to the JEV nightly loop.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::l5_cognition::nt_jev::eval::EvalReport;
use crate::l5_cognition::nt_jev::evolve::report_scores;

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
        // 锁投毒改为**恢复**而非 panic（2026-10-02），本文件 10 处**一致**处理。
// 判据：锁内是 `Vec<EvaluationCriteria>` / `HashMap` / `Vec<EvolutionEvent>`，
// 而 `EvaluationCriteria` 的 4 个字段（name/weight/threshold/description）**互相独立**，
// 且 `evolve_criteria` 用 `.clamp(min_weight, max_weight)` 写回
// ⇒ 「权重恒在区间内」这个不变量**与是否投毒无关**，元素不会撕裂。
// 同批改的必要性：`:80`/`:100` 在**已持有** `criteria`(+`scores`) 时再取第三把锁。
//    一旦那里 panic，是在**持锁状态**下 panic ⇒ 同时毒化多个锁
//    ⇒ 之后每个 `.unwrap()` 都 panic ⇒ **一次瞬时失败升级为该 evaluator 的永久拒绝服务**。
//    只修 `:168`（`remove_criterion`，实测 0 调用者）等于没修。
// ⛔ 若将来锁内引入互相约束的多字段结构，则应改回 panic（`into_inner()` 会把
//    「不一致」读成有效数据）。这条判据已写在 `session_replay` 的同批改动里。
let criteria = self.criteria.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        self.scores.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner).push(breakdown.clone());
        breakdown
    }

    pub fn evolve_criteria(&self) {
        let mut criteria = self.criteria.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let scores = self.scores.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
                self.evolution_log.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner).push(EvolutionEvent {
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
        let scores = self.scores.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        self.evolution_log.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    pub fn evaluation_count(&self) -> usize {
        self.scores.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner).len()
    }

    pub fn add_criterion(&self, name: String, weight: f64, threshold: f64, description: String) {
        let mut criteria = self.criteria.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        criteria.push(EvaluationCriteria { name, weight, threshold, description });
    }

    /// S1Bench-aligned criteria for JEV nightly eval (weights sum to 1.0).
    ///
    /// Names match `nt_jev::evolve::JEV_CRITERIA` exactly so
    /// [`EvolvingEvaluator::evaluate_jev_report`] joins without loss.
    /// Thresholds are starting points (jevassert-style); the evolu­tion loop
    /// moves weights/thresholds via [`EvolvingEvaluator::evolve_criteria`].
    pub fn jev_criteria() -> Vec<EvaluationCriteria> {
        vec![
            EvaluationCriteria { name: "accuracy".to_string(), weight: 0.35, threshold: 0.8, description: "S1Bench macro accuracy".to_string() },
            EvaluationCriteria { name: "calibration".to_string(), weight: 0.25, threshold: 0.85, description: "1 - ECE (max_ece 0.15)".to_string() },
            EvaluationCriteria { name: "coverage".to_string(), weight: 0.15, threshold: 0.6, description: "kept-coverage at 0.9 precision".to_string() },
            EvaluationCriteria { name: "efficiency".to_string(), weight: 0.15, threshold: 0.7, description: "latency-derived efficiency".to_string() },
            EvaluationCriteria { name: "robustness".to_string(), weight: 0.10, threshold: 0.6, description: "1 - Brier".to_string() },
        ]
    }

    /// One nightly step: score an [`EvalReport`] against JEV criteria.
    ///
    /// This is the L1→L4 landing point. Callers then invoke
    /// [`EvolvingEvaluator::evolve_criteria`] so sustained gains/losses move
    /// weights and append to the evolution log.
    pub fn evaluate_jev_report(&self, target: &str, report: &EvalReport) -> ScoreBreakdown {
        self.evaluate(target, &report_scores(report))
    }

    pub fn remove_criterion(&self, name: &str) -> bool {
        let mut criteria = self.criteria.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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

    #[test] fn test_jev_criteria_names_match_bridge() {
        use crate::l5_cognition::nt_jev::evolve::JEV_CRITERIA;
        let names: Vec<String> = EvolvingEvaluator::jev_criteria().iter().map(|c| c.name.clone()).collect();
        for k in JEV_CRITERIA {
            assert!(names.contains(&k.to_string()), "criterion {} missing", k);
        }
        let w: f64 = EvolvingEvaluator::jev_criteria().iter().map(|c| c.weight).sum();
        assert!((w - 1.0).abs() < 1e-12, "weights sum {}", w);
    }

    #[test] fn test_nightly_loop_two_cycles() {
        use crate::l5_cognition::nt_jev::eval::EvalReport;
        use crate::l5_cognition::nt_jev::evolve::JEV_CRITERIA;
        let e = EvolvingEvaluator::new(EvolvingEvaluator::jev_criteria(), 0.05);
        // Cycle 1: weak night.
        let weak = EvalReport { n: 50, accuracy: 0.6, brier: 0.3, ece: 0.25, coverage_at_p90: 0.4, mean_latency_ms: 200.0 };
        let s1 = e.evaluate_jev_report("nightly-1", &weak);
        assert!(s1.weighted_score >= 0.0 && s1.weighted_score <= 1.0);
        assert_eq!(s1.criteria_scores.len(), JEV_CRITERIA.len());
        e.evolve_criteria();
        // Cycle 2: strong night after calibration work.
        let strong = EvalReport { n: 50, accuracy: 0.92, brier: 0.08, ece: 0.05, coverage_at_p90: 0.8, mean_latency_ms: 30.0 };
        let s2 = e.evaluate_jev_report("nightly-2", &strong);
        assert!(s2.weighted_score > s1.weighted_score, "{} vs {}", s2.weighted_score, s1.weighted_score);
        e.evolve_criteria();
        assert_eq!(e.evaluation_count(), 2);
        assert!(!e.get_evolution_log().is_empty());
    }
}
