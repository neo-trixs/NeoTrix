/// Counterfactual Futures Engine
/// Inspired by Milkyway (arXiv:2604.15719) "The World Leaks the Future"
///
/// Core mechanism: temporal contrast between earlier/later predictions on the
/// same unresolved question exposes omissions → internal feedback signal.
/// VSA-native: factor tracking harness with weight evolution, counterfactual
/// scenario branching via VSA diff simulation, confidence-aware calibration.
use std::collections::VecDeque;

const DEFAULT_VSA_DIM: usize = 256;
const MAX_FACTORS_PER_QUESTION: usize = 12;
const MAX_PREDICTIONS_HISTORY: usize = 20;
const MAX_QUESTIONS_DEFAULT: usize = 20;
const HARNESS_MAX_FACTORS: usize = 50;

#[derive(Debug, Clone)]
pub struct Factor {
    pub name: String,
    pub seed: u64,
    pub weight: f64,
    pub observed_impact: f64,
    pub impact_samples: u32,
}

#[derive(Debug, Clone)]
pub struct PredictionEntry {
    pub prediction: Vec<u8>,
    pub confidence: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Question {
    pub id: u64,
    pub description: String,
    pub factors: Vec<Factor>,
    pub prediction_history: VecDeque<PredictionEntry>,
    pub resolved: bool,
    pub resolution_outcome: Option<Vec<u8>>,
    pub resolution_accuracy: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CounterfactualScenario {
    pub hypothesis_name: String,
    pub predicted_outcome: Vec<u8>,
    pub plausibility: f64,
    pub divergence: f64,
}

#[derive(Debug, Clone)]
pub struct CounterfactualStats {
    pub step: u64,
    pub active_questions: usize,
    pub resolved_questions: usize,
    pub total_predictions: usize,
    pub internal_feedback_count: u64,
    pub scenario_count: u64,
    pub avg_resolution_accuracy: f64,
    pub harness_factors: usize,
    pub confidence_calibration: f64,
}

pub struct CounterfactualFuturesEngine {
    questions: Vec<Question>,
    harness_factors: Vec<Factor>,
    max_questions: usize,
    next_id: u64,
    step: u64,
    vsa_dim: usize,
    feedback_count: u64,
    scenario_generated: u64,
    total_resolution_error: f64,
    resolution_count: u32,
}

impl CounterfactualFuturesEngine {
    pub fn new(vsa_dim: usize, max_questions: usize) -> Self {
        Self {
            questions: Vec::new(),
            harness_factors: Vec::new(),
            max_questions,
            next_id: 1,
            step: 0,
            vsa_dim,
            feedback_count: 0,
            scenario_generated: 0,
            total_resolution_error: 0.0,
            resolution_count: 0,
        }
    }

    pub fn register_question(&mut self, description: &str, factor_seeds: &[(String, u64)]) -> u64 {
        if self.questions.len() >= self.max_questions {
            if let Some(pos) = self.questions.iter().position(|q| !q.resolved) {
                self.questions.remove(pos);
            } else {
                return 0;
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let factors = factor_seeds
            .iter()
            .take(MAX_FACTORS_PER_QUESTION)
            .map(|(name, seed)| {
                let _vector = seeded_random(*seed, self.vsa_dim);
                Factor {
                    name: name.clone(),
                    seed: *seed,
                    weight: 1.0,
                    observed_impact: 0.0,
                    impact_samples: 0,
                }
            })
            .collect();
        self.questions.push(Question {
            id,
            description: description.to_string(),
            factors,
            prediction_history: VecDeque::with_capacity(MAX_PREDICTIONS_HISTORY),
            resolved: false,
            resolution_outcome: None,
            resolution_accuracy: None,
        });
        id
    }

    pub fn make_prediction(&mut self, question_id: u64, prediction: &[u8], confidence: f64) {
        let Some(q) = self.questions.iter_mut().find(|q| q.id == question_id) else {
            return;
        };
        if q.resolved {
            return;
        }
        let confidence = confidence.clamp(0.0, 1.0);
        if q.prediction_history.len() >= MAX_PREDICTIONS_HISTORY {
            q.prediction_history.pop_front();
        }
        q.prediction_history.push_back(PredictionEntry {
            prediction: prediction.to_vec(),
            confidence,
            timestamp: self.step,
        });
    }

    fn question(&self, id: u64) -> Option<&Question> {
        self.questions.iter().find(|q| q.id == id)
    }

    pub fn internal_feedback(&mut self, question_id: u64) -> Vec<String> {
        let mut gaps = Vec::new();
        let q = match self.question(question_id) {
            Some(q) => q,
            None => return gaps,
        };
        if q.prediction_history.len() < 2 {
            return gaps;
        }
        let latest = q
            .prediction_history
            .back()
            .expect("prediction_history.len() >= 2 per guard");
        if let Some(first) = q.prediction_history.front() {
            let divergence = 1.0 - hamming_sim(&latest.prediction, &first.prediction);
            if divergence > 0.3 {
                gaps.push(format!(
                    "prediction divergence {:.2}: factors may have shifted since first prediction",
                    divergence
                ));
            }
        }
        for i in 1..q.prediction_history.len() {
            let prev = &q.prediction_history[i - 1];
            let curr = &q.prediction_history[i];
            let step_div = 1.0 - hamming_sim(&prev.prediction, &curr.prediction);
            if step_div > 0.2 {
                gaps.push(format!(
                    "step divergence at t={}: {:.2} — possible new factor introduced",
                    curr.timestamp, step_div
                ));
            }
        }
        if q.prediction_history.len() >= 3 {
            let recent: Vec<f64> = q
                .prediction_history
                .iter()
                .rev()
                .take(3)
                .map(|p| p.confidence)
                .collect();
            if recent[0] < recent[1] * 0.7 && recent[1] < recent[2] * 0.7 {
                gaps.push(
                    "confidence dropping over last 3 predictions — uncertainty increasing"
                        .to_string(),
                );
            }
        }
        if !gaps.is_empty() {
            self.feedback_count += 1;
        }
        gaps
    }

    pub fn generate_counterfactuals(
        &mut self,
        question_id: u64,
        hypotheses: &[(String, u64)],
    ) -> Vec<CounterfactualScenario> {
        let count_before = self.scenario_generated;
        let mut scenarios = Vec::new();
        let (factors, baseline) = {
            let q = match self.question(question_id) {
                Some(q) => q,
                None => return scenarios,
            };
            if q.prediction_history.is_empty() || q.resolved {
                return scenarios;
            }
            let latest = q
                .prediction_history
                .back()
                .expect("prediction_history is non-empty per guard");
            (q.factors.clone(), latest.prediction.clone())
        };
        for (hyp_name, hyp_seed) in hypotheses {
            let hyp_vec = seeded_random(*hyp_seed, self.vsa_dim);
            let counterfactual = xor_bind(&baseline, &hyp_vec);
            let plausibility =
                estimate_plausibility(&factors, &hyp_vec, &counterfactual, &baseline);
            let divergence = 1.0 - hamming_sim(&counterfactual, &baseline);
            scenarios.push(CounterfactualScenario {
                hypothesis_name: hyp_name.clone(),
                predicted_outcome: counterfactual,
                plausibility,
                divergence,
            });
        }
        self.scenario_generated = count_before + scenarios.len() as u64;
        scenarios
    }

    pub fn resolve(&mut self, question_id: u64, actual_outcome: &[u8]) -> f64 {
        let Some(q) = self
            .questions
            .iter_mut()
            .find(|q| q.id == question_id && !q.resolved)
        else {
            return 0.0;
        };
        q.resolved = true;
        q.resolution_outcome = Some(actual_outcome.to_vec());
        let Some(latest) = q.prediction_history.back() else {
            return 0.0;
        };
        let accuracy = hamming_sim(&latest.prediction, actual_outcome);
        q.resolution_accuracy = Some(accuracy);
        self.total_resolution_error += 1.0 - accuracy;
        self.resolution_count += 1;
        for factor in &mut q.factors {
            let prev = factor.observed_impact * factor.impact_samples as f64;
            factor.observed_impact = (prev + accuracy) / (factor.impact_samples + 1) as f64;
            factor.impact_samples += 1;
        }
        accuracy
    }

    pub fn update_harness(&mut self) {
        let resolved: Vec<Question> = self
            .questions
            .iter()
            .filter(|q| q.resolved && q.resolution_accuracy.is_some())
            .cloned()
            .collect();
        for q in &resolved {
            let accuracy = q.resolution_accuracy.unwrap_or(1.0);
            for factor in &q.factors {
                if let Some(hf) = self
                    .harness_factors
                    .iter_mut()
                    .find(|f| f.seed == factor.seed)
                {
                    hf.weight = hf.weight * 0.9 + accuracy * factor.observed_impact * 0.1;
                    let prev_avg = hf.observed_impact * (hf.impact_samples - 1).min(1) as f64;
                    hf.observed_impact = if hf.impact_samples > 0 {
                        (prev_avg + factor.observed_impact) / (hf.impact_samples + 1) as f64
                    } else {
                        factor.observed_impact
                    };
                    hf.impact_samples += 1;
                } else if self.harness_factors.len() < HARNESS_MAX_FACTORS {
                    let _vector = seeded_random(factor.seed, self.vsa_dim);
                    self.harness_factors.push(Factor {
                        name: factor.name.clone(),
                        seed: factor.seed,
                        weight: 1.0,
                        observed_impact: accuracy * factor.observed_impact,
                        impact_samples: 1,
                    });
                }
            }
        }
        self.harness_factors.retain(|f| f.weight > 0.1);
    }

    pub fn tick(&mut self) {
        self.step += 1;
    }

    pub fn synthesize_prediction(&self, question_id: u64) -> Option<Vec<u8>> {
        let q = self.question(question_id)?;
        if q.factors.is_empty() {
            return None;
        }
        let total: f64 = q.factors.iter().map(|f| f.weight).sum();
        if total <= 0.0 {
            return None;
        }
        let factor_vectors: Vec<Vec<u8>> = q
            .factors
            .iter()
            .map(|f| seeded_random(f.seed, self.vsa_dim))
            .collect();
        let refs: Vec<&[u8]> = factor_vectors.iter().map(|v| v.as_slice()).collect();
        Some(majority_bundle_to_dim(&refs, self.vsa_dim))
    }

    pub fn stats(&self) -> CounterfactualStats {
        let active = self.questions.iter().filter(|q| !q.resolved).count();
        let resolved = self.questions.iter().filter(|q| q.resolved).count();
        let total_preds: usize = self
            .questions
            .iter()
            .map(|q| q.prediction_history.len())
            .sum();
        let avg_acc = if self.resolution_count > 0 {
            1.0 - self.total_resolution_error / self.resolution_count as f64
        } else {
            0.0
        };
        let mut conf_err_sum = 0.0;
        let mut conf_count = 0;
        for q in &self.questions {
            if let Some(acc) = q.resolution_accuracy {
                if let Some(last) = q.prediction_history.back() {
                    conf_err_sum += (last.confidence - acc).abs();
                    conf_count += 1;
                }
            }
        }
        let calib = if conf_count > 0 {
            (1.0 - conf_err_sum / conf_count as f64).max(0.0)
        } else {
            0.0
        };
        CounterfactualStats {
            step: self.step,
            active_questions: active,
            resolved_questions: resolved,
            total_predictions: total_preds,
            internal_feedback_count: self.feedback_count,
            scenario_count: self.scenario_generated,
            avg_resolution_accuracy: avg_acc,
            harness_factors: self.harness_factors.len(),
            confidence_calibration: calib,
        }
    }

    pub fn unresolved_count(&self) -> usize {
        self.questions.iter().filter(|q| !q.resolved).count()
    }

    pub fn resolved_count(&self) -> usize {
        self.questions.iter().filter(|q| q.resolved).count()
    }

    pub fn question_ids(&self) -> Vec<u64> {
        self.questions.iter().map(|q| q.id).collect()
    }

    pub fn unresolved_ids(&self) -> Vec<u64> {
        self.questions
            .iter()
            .filter(|q| !q.resolved)
            .map(|q| q.id)
            .collect()
    }
}

impl Default for CounterfactualFuturesEngine {
    fn default() -> Self {
        Self::new(DEFAULT_VSA_DIM, MAX_QUESTIONS_DEFAULT)
    }
}

// --- VSA helpers (deterministic, no rand dependency) ---

fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn seeded_random(seed: u64, dim: usize) -> Vec<u8> {
    let mut state = seed;
    (0..dim)
        .map(|_| (xorshift(&mut state) & 0xFF) as u8)
        .collect()
}

fn hamming_sim(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dist: u32 = a
        .iter()
        .zip(b.iter())
        .take(len)
        .map(|(x, y)| (x ^ y).count_ones())
        .sum();
    let max_dist = (len as u32) * 8;
    1.0 - dist as f64 / max_dist as f64
}

fn xor_bind(a: &[u8], b: &[u8]) -> Vec<u8> {
    let len = a.len().min(b.len());
    a.iter()
        .zip(b.iter())
        .take(len)
        .map(|(x, y)| x ^ y)
        .collect()
}

fn majority_bundle_to_dim(vectors: &[&[u8]], dim: usize) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![0; dim];
    }
    let n = vectors.len();
    let mut result = Vec::with_capacity(dim);
    for i in 0..dim {
        let ones = vectors
            .iter()
            .filter(|v| v.get(i).map_or(false, |&x| x > 128))
            .count();
        result.push(if ones > n / 2 { 1 } else { 0 });
    }
    result
}

fn estimate_plausibility(
    factors: &[Factor],
    hyp_vec: &[u8],
    _counterfactual: &[u8],
    baseline: &[u8],
) -> f64 {
    let max_sim = factors
        .iter()
        .map(|f| {
            let fv = seeded_random(f.seed, hyp_vec.len());
            hamming_sim(hyp_vec, &fv)
        })
        .fold(0.0f64, f64::max);
    let baseline_dev = 1.0 - hamming_sim(hyp_vec, baseline);
    (max_sim * 0.6 + baseline_dev.clamp(0.0, 0.5) * 0.4).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> CounterfactualFuturesEngine {
        CounterfactualFuturesEngine::new(64, 10)
    }

    fn v(seed: u8, dim: usize) -> Vec<u8> {
        seeded_random(seed as u64, dim)
    }

    #[test]
    fn test_new_engine() {
        let e = make_engine();
        let s = e.stats();
        assert_eq!(s.active_questions, 0);
        assert_eq!(s.step, 0);
    }

    #[test]
    fn test_register_question() {
        let mut e = make_engine();
        let factors = vec![("temp".to_string(), 1), ("pressure".to_string(), 2)];
        let id = e.register_question("weather tomorrow", &factors);
        assert!(id > 0);
        assert_eq!(e.unresolved_count(), 1);
    }

    #[test]
    fn test_make_prediction() {
        let mut e = make_engine();
        let id = e.register_question("test q", &[("f1".to_string(), 10)]);
        e.make_prediction(id, &v(1, 64), 0.8);
        let s = e.stats();
        assert_eq!(s.total_predictions, 1);
    }

    #[test]
    fn test_prediction_on_resolved_question_noop() {
        let mut e = make_engine();
        let id = e.register_question("test", &[("f".to_string(), 5)]);
        e.make_prediction(id, &v(1, 64), 0.8);
        e.resolve(id, &v(1, 64));
        e.make_prediction(id, &v(2, 64), 0.9);
        let s = e.stats();
        assert_eq!(s.total_predictions, 1);
    }

    #[test]
    fn test_internal_feedback_no_gaps_when_stable() {
        let mut e = make_engine();
        let id = e.register_question("stable", &[("f1".to_string(), 10)]);
        let v1 = v(42, 64);
        let v2 = v(42, 64);
        e.make_prediction(id, &v1, 0.9);
        e.make_prediction(id, &v2, 0.9);
        let gaps = e.internal_feedback(id);
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_internal_feedback_detects_divergence() {
        let mut e = make_engine();
        let id = e.register_question("divergent", &[("f1".to_string(), 10)]);
        e.make_prediction(id, &v(1, 64), 0.9);
        e.make_prediction(id, &v(99, 64), 0.6);
        let gaps = e.internal_feedback(id);
        assert!(!gaps.is_empty(), "should detect divergence");
        assert!(gaps.iter().any(|g| g.contains("divergence")));
    }

    #[test]
    fn test_internal_feedback_detects_confidence_drop() {
        let mut e = make_engine();
        let id = e.register_question("conf_drop", &[("f1".to_string(), 10)]);
        let v1 = v(10, 64);
        e.make_prediction(id, &v1, 0.9);
        e.make_prediction(id, &v1, 0.6);
        e.make_prediction(id, &v1, 0.4);
        let gaps = e.internal_feedback(id);
        assert!(gaps.iter().any(|g| g.contains("confidence")));
    }

    #[test]
    fn test_generate_counterfactuals() {
        let mut e = make_engine();
        let id = e.register_question("cf", &[("base".to_string(), 10)]);
        e.make_prediction(id, &v(1, 64), 0.8);
        let hyps = vec![
            ("what_if_rain".to_string(), 20),
            ("what_if_sun".to_string(), 30),
        ];
        let scens = e.generate_counterfactuals(id, &hyps);
        assert_eq!(scens.len(), 2);
        assert!(e.stats().scenario_count > 0);
    }

    #[test]
    fn test_resolve_and_accuracy() {
        let mut e = make_engine();
        let id = e.register_question("resolve_test", &[("f1".to_string(), 10)]);
        let outcome = v(42, 64);
        e.make_prediction(id, &outcome, 0.9);
        let acc = e.resolve(id, &outcome);
        assert!((acc - 1.0).abs() < 0.01, "perfect match: {}", acc);
    }

    #[test]
    fn test_update_harness_transfers_knowledge() {
        let mut e = make_engine();
        let id1 = e.register_question("q1", &[("temp".to_string(), 1)]);
        e.make_prediction(id1, &v(5, 64), 0.8);
        e.resolve(id1, &v(5, 64));
        e.update_harness();
        assert!(e.stats().harness_factors > 0, "harness should have factors");
    }

    #[test]
    fn test_synthesize_prediction() {
        let mut e = make_engine();
        let id = e.register_question("synth", &[("a".to_string(), 1), ("b".to_string(), 2)]);
        let syn = e.synthesize_prediction(id);
        assert!(syn.is_some());
        assert_eq!(syn.unwrap().len(), 64);
    }

    #[test]
    fn test_synthesize_prediction_no_factors() {
        let mut e = make_engine();
        let id = e.register_question("empty", &[]);
        assert!(e.synthesize_prediction(id).is_none());
    }

    #[test]
    fn test_question_eviction_when_full() {
        let mut e = CounterfactualFuturesEngine::new(64, 2);
        let _id1 = e.register_question("q1", &[("f".to_string(), 1)]);
        let _id2 = e.register_question("q2", &[("f".to_string(), 2)]);
        let id3 = e.register_question("q3", &[("f".to_string(), 3)]);
        assert!(id3 > 0, "should evict oldest unresolved");
        assert_eq!(e.questions.len(), 2);
    }

    #[test]
    fn test_resolve_no_prediction() {
        let mut e = make_engine();
        let id = e.register_question("no_pred", &[("f".to_string(), 1)]);
        let acc = e.resolve(id, &v(1, 64));
        assert!((acc - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_tick_increments_step() {
        let mut e = make_engine();
        assert_eq!(e.step, 0);
        e.tick();
        assert_eq!(e.step, 1);
    }

    #[test]
    fn test_stats_after_full_cycle() {
        let mut e = make_engine();
        let id = e.register_question("full", &[("f".to_string(), 1)]);
        e.make_prediction(id, &v(1, 64), 0.8);
        e.make_prediction(id, &v(1, 64), 0.7);
        e.internal_feedback(id);
        let hyps = vec![("h".to_string(), 99)];
        e.generate_counterfactuals(id, &hyps);
        e.resolve(id, &v(1, 64));
        e.update_harness();
        e.tick();
        let s = e.stats();
        assert!(s.active_questions == 0 || s.active_questions == 0);
        assert_eq!(s.resolved_questions, 1);
        assert!(s.avg_resolution_accuracy > 0.9);
    }
}
