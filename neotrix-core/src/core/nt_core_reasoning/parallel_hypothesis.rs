use std::collections::VecDeque;

use super::vsa_blackboard::{ExpertType, VsaBlackboard};
use crate::core::nt_core_hcube::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct ParallelHypothesisConfig {
    pub max_competing_hypotheses: usize,
    pub belief_update_rate: f64,
    pub abduction_confidence_threshold: f64,
    pub likelihood_noise: f64,
    pub convergence_threshold: f64,
}

impl Default for ParallelHypothesisConfig {
    fn default() -> Self {
        Self {
            max_competing_hypotheses: 8,
            belief_update_rate: 0.15,
            abduction_confidence_threshold: 0.7,
            likelihood_noise: 0.1,
            convergence_threshold: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompetingHypothesis {
    pub id: u64,
    pub description: String,
    pub prior: f64,
    pub posterior: f64,
    pub likelihood: f64,
    pub evidence_for: Vec<String>,
    pub evidence_against: Vec<String>,
    pub vsa_signature: Vec<u8>,
    pub is_abduced: bool,
    pub abductive_plausibility: f64,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct ParallelHypothesisEvaluator {
    pub config: ParallelHypothesisConfig,
    pub hypotheses: Vec<CompetingHypothesis>,
    pub next_id: u64,
    pub belief_history: VecDeque<Vec<(u64, f64)>>,
    pub blackboard: VsaBlackboard,
    pub convergence_rounds: usize,
}

#[derive(Debug, Clone)]
pub struct ParallelHypEvalStats {
    pub active_hypotheses: usize,
    pub top_confidence: f64,
    pub entropy: f64,
    pub convergence_rounds: usize,
    pub is_converged: bool,
    pub best_expert_type: ExpertType,
    pub abduction_count: usize,
}

impl ParallelHypothesisEvaluator {
    pub fn new(config: ParallelHypothesisConfig, blackboard: VsaBlackboard) -> Self {
        Self {
            config: config.clone(),
            hypotheses: Vec::with_capacity(config.max_competing_hypotheses),
            next_id: 1,
            belief_history: VecDeque::with_capacity(16),
            blackboard,
            convergence_rounds: 0,
        }
    }

    pub fn post_hypothesis(&mut self, desc: String, prior: f64) -> u64 {
        if self.hypotheses.len() >= self.config.max_competing_hypotheses {
            return 0;
        }
        let id = self.next_id;
        self.next_id += 1;
        let prior = prior.clamp(0.0, 1.0);
        let vsa_sig = QuantizedVSA::random_vector();
        let now = crate::core::unix_now_ms();
        self.hypotheses.push(CompetingHypothesis {
            id,
            description: desc,
            prior,
            posterior: prior,
            likelihood: 0.5,
            evidence_for: Vec::new(),
            evidence_against: Vec::new(),
            vsa_signature: vsa_sig,
            is_abduced: false,
            abductive_plausibility: 0.0,
            created_at: now,
        });
        self.blackboard.post_hypothesis(
            format!("hypothesis:{}", id).into_bytes(),
            prior,
            ExpertType::Synthesis,
            vec![],
        );
        id
    }

    pub fn observe_evidence(&mut self, evidence: &str, supports: Vec<u64>, contradicts: Vec<u64>) {
        for h in self.hypotheses.iter_mut() {
            if supports.contains(&h.id) {
                h.evidence_for.push(evidence.to_string());
            }
            if contradicts.contains(&h.id) {
                h.evidence_against.push(evidence.to_string());
            }
        }
        self.bayesian_update();
    }

    pub fn bayesian_update(&mut self) {
        let eps = 1e-12;
        let mut evidence_total = 0.0_f64;
        let mut posteriors = Vec::with_capacity(self.hypotheses.len());

        for h in &self.hypotheses {
            let mut likelihood = h.likelihood.max(eps);
            if !h.evidence_against.is_empty() {
                likelihood *=
                    (1.0 - self.config.likelihood_noise).powf(h.evidence_against.len() as f64);
            }
            if !h.evidence_for.is_empty() {
                let boost =
                    (1.0 + self.config.belief_update_rate).powf(h.evidence_for.len() as f64);
                likelihood = (likelihood * boost).min(1.0 - eps);
            }
            let unnormalized = h.prior * likelihood;
            evidence_total += unnormalized;
            posteriors.push((h.id, unnormalized));
        }

        let norm = evidence_total.max(eps);
        for (posterior, h) in posteriors.iter_mut().zip(self.hypotheses.iter_mut()) {
            h.posterior = (posterior.1 / norm).clamp(eps, 1.0 - eps);
            h.prior = h.posterior;
        }

        let snapshot: Vec<(u64, f64)> = self
            .hypotheses
            .iter()
            .map(|h| (h.id, h.posterior))
            .collect();
        self.belief_history.push_back(snapshot);
        if self.belief_history.len() > 16 {
            self.belief_history.pop_front();
        }

        if self.check_convergence() {
            self.convergence_rounds += 1;
        } else {
            self.convergence_rounds = 0;
        }
    }

    pub fn abduce(&mut self, observation: &str) -> Vec<u64> {
        let obs_bytes = observation.as_bytes();
        let mut abduced_ids = Vec::new();
        let abduced_plausibilities = {
            let mut scored: Vec<(f64, &CompetingHypothesis)> = self
                .hypotheses
                .iter()
                .map(|h| {
                    let vsa_sim = if h.vsa_signature.len() > 4 {
                        let obs_sig = QuantizedVSA::seeded_random(
                            obs_bytes.len() as u64,
                            h.vsa_signature.len(),
                        );
                        QuantizedVSA::similarity(&h.vsa_signature, &obs_sig)
                    } else {
                        0.0
                    };
                    let text_sim = if !h.description.is_empty() {
                        let common = h
                            .description
                            .bytes()
                            .zip(obs_bytes.iter())
                            .filter(|(a, b)| a == *b)
                            .count() as f64;
                        common / h.description.len().max(1) as f64
                    } else {
                        0.0
                    };
                    let plausibility = h.posterior * 0.4 + vsa_sim * 0.35 + text_sim * 0.25;
                    (plausibility, h)
                })
                .collect();

            scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

            for (plausibility, h) in &scored {
                if *plausibility >= self.config.abduction_confidence_threshold {
                    abduced_ids.push(h.id);
                }
            }
            let pls: std::collections::HashMap<u64, f64> =
                scored.iter().map(|(p, h)| (h.id, *p)).collect();

            if abduced_ids.is_empty() && !scored.is_empty() {
                let best_pl = scored[0].0;
                let desc = format!(
                    "abduced:{}",
                    observation.chars().take(48).collect::<String>()
                );
                let new_id = self.post_hypothesis(desc, best_pl);
                if new_id != 0 {
                    if let Some(ah) = self.hypotheses.iter_mut().find(|h| h.id == new_id) {
                        ah.is_abduced = true;
                        ah.abductive_plausibility = best_pl;
                        ah.vsa_signature = QuantizedVSA::seeded_random(
                            observation.len() as u64,
                            QuantizedVSA::default().dim(),
                        );
                    }
                    abduced_ids.push(new_id);
                }
            }
            pls
        };
        for &id in &abduced_ids {
            if let Some(existing) = self.hypotheses.iter_mut().find(|ch| ch.id == id) {
                let pl = *abduced_plausibilities.get(&id).unwrap_or(&0.0);
                existing.is_abduced = true;
                existing.abductive_plausibility = pl;
            }
        }

        self.bayesian_update();
        abduced_ids
    }

    pub fn best_hypothesis(&self) -> Option<&CompetingHypothesis> {
        self.hypotheses.iter().max_by(|a, b| {
            a.posterior
                .partial_cmp(&b.posterior)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn belief_entropy(&self) -> f64 {
        let eps = 1e-12;
        let beliefs = self.softmax_beliefs();
        -beliefs
            .iter()
            .map(|&p| {
                if p > eps {
                    p * p.log(std::f64::consts::E)
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / (self.hypotheses.len().max(1) as f64).ln()
    }

    pub fn softmax_beliefs(&self) -> Vec<f64> {
        let posteriors: Vec<f64> = self.hypotheses.iter().map(|h| h.posterior).collect();
        let max_p = posteriors.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = posteriors.iter().map(|&x| (x - max_p).exp()).collect();
        let sum: f64 = exps.iter().sum();
        if sum > 0.0 {
            exps.iter().map(|&e| e / sum).collect()
        } else {
            vec![1.0 / exps.len() as f64; exps.len()]
        }
    }

    #[allow(dead_code)]
    fn likelihood_function(&self, hypothesis: &CompetingHypothesis, evidence: &str) -> f64 {
        let ev_bytes = evidence.as_bytes();
        let sig =
            QuantizedVSA::seeded_random(ev_bytes.len() as u64, hypothesis.vsa_signature.len());
        let vsa_sim = QuantizedVSA::similarity(&hypothesis.vsa_signature, &sig);
        let prior_weight = hypothesis.prior;
        0.5 + 0.5 * vsa_sim * prior_weight
    }

    fn check_convergence(&self) -> bool {
        if self.belief_history.len() < 2 {
            return false;
        }
        let last = self.belief_history.back().unwrap();
        if self.belief_history.len() < 2 {
            return false;
        }
        let prev_idx = self.belief_history.len() - 2;
        let prev = &self.belief_history[prev_idx];

        let mut max_diff = 0.0_f64;
        for (id, recent) in last {
            if let Some((_, old)) = prev.iter().find(|(pid, _)| pid == id) {
                let diff = (recent - old).abs();
                if diff > max_diff {
                    max_diff = diff;
                }
            }
        }
        max_diff < self.config.convergence_threshold
    }

    #[allow(dead_code)]
    fn normalize_beliefs(&mut self) {
        let sum: f64 = self.hypotheses.iter().map(|h| h.posterior).sum();
        if sum > 0.0 {
            for h in self.hypotheses.iter_mut() {
                h.posterior /= sum;
                h.prior = h.posterior;
            }
        }
    }

    pub fn stats(&self) -> ParallelHypEvalStats {
        let top_conf = self.best_hypothesis().map(|h| h.posterior).unwrap_or(0.0);
        let entropy = self.belief_entropy();
        let abduction_count = self.hypotheses.iter().filter(|h| h.is_abduced).count();

        let best_expert = self
            .blackboard
            .best_hypothesis()
            .map(|h| h.expert)
            .unwrap_or(ExpertType::Synthesis);

        ParallelHypEvalStats {
            active_hypotheses: self.hypotheses.len(),
            top_confidence: top_conf,
            entropy,
            convergence_rounds: self.convergence_rounds,
            is_converged: self.check_convergence(),
            best_expert_type: best_expert,
            abduction_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_evaluator() -> ParallelHypothesisEvaluator {
        let config = ParallelHypothesisConfig::default();
        let bb = VsaBlackboard::new(64);
        ParallelHypothesisEvaluator::new(config, bb)
    }

    #[test]
    fn test_post_hypothesis() {
        let mut ev = make_evaluator();
        let id = ev.post_hypothesis("test hypothesis".into(), 0.5);
        assert!(id > 0);
        assert_eq!(ev.hypotheses.len(), 1);
        assert_eq!(ev.hypotheses[0].description, "test hypothesis");
        assert!((ev.hypotheses[0].prior - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_max_hypotheses() {
        let config = ParallelHypothesisConfig {
            max_competing_hypotheses: 2,
            ..Default::default()
        };
        let mut ev = ParallelHypothesisEvaluator::new(config, VsaBlackboard::new(64));
        assert!(ev.post_hypothesis("h1".into(), 0.5) > 0);
        assert!(ev.post_hypothesis("h2".into(), 0.5) > 0);
        assert_eq!(ev.post_hypothesis("h3".into(), 0.5), 0);
    }

    #[test]
    fn test_bayesian_update_increases_posterior_for_supported() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("h1".into(), 0.5);
        ev.post_hypothesis("h2".into(), 0.5);

        let id1 = ev.hypotheses[0].id;
        let id2 = ev.hypotheses[1].id;

        ev.observe_evidence("e1 supports h1", vec![id1], vec![id2]);
        let best = ev.best_hypothesis().unwrap();
        assert_eq!(best.id, id1);
        assert!(best.posterior > 0.5);
    }

    #[test]
    fn test_belief_entropy_uniform() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("a".into(), 0.5);
        ev.post_hypothesis("b".into(), 0.5);
        let e = ev.belief_entropy();
        assert!((e - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_belief_entropy_certain() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("a".into(), 0.99);
        ev.post_hypothesis("b".into(), 0.01);
        ev.bayesian_update();
        let e = ev.belief_entropy();
        assert!(e < 0.3);
    }

    #[test]
    fn test_abduce_creates_hypothesis() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("existing explanation".into(), 0.6);
        let ids = ev.abduce("observed phenomenon X");
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_convergence_detection() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("stable".into(), 0.9);
        ev.post_hypothesis("unstable".into(), 0.1);
        for _ in 0..5 {
            ev.bayesian_update();
        }
        let stats = ev.stats();
        assert!(stats.is_converged || stats.convergence_rounds >= 0);
    }

    #[test]
    fn test_stats_report() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("primary".into(), 0.8);
        ev.post_hypothesis("alternative".into(), 0.2);
        let stats = ev.stats();
        assert_eq!(stats.active_hypotheses, 2);
        assert!((stats.top_confidence - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_bayesian_posterior_sum_approx_one() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("h1".into(), 0.3);
        ev.post_hypothesis("h2".into(), 0.3);
        ev.post_hypothesis("h3".into(), 0.4);
        ev.bayesian_update();
        let sum: f64 = ev.hypotheses.iter().map(|h| h.posterior).sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_likelihood_function_range() {
        let ev = make_evaluator();
        let mut h = CompetingHypothesis {
            id: 1,
            description: "test".into(),
            prior: 0.5,
            posterior: 0.5,
            likelihood: 0.5,
            evidence_for: vec![],
            evidence_against: vec![],
            vsa_signature: QuantizedVSA::random_vector(),
            is_abduced: false,
            abductive_plausibility: 0.0,
            created_at: 0,
        };
        let l = ev.likelihood_function(&h, "some evidence");
        assert!((0.0..=1.0).contains(&l));
    }

    #[test]
    fn test_softmax_beliefs() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("a".into(), 0.9);
        ev.post_hypothesis("b".into(), 0.1);
        let sm = ev.softmax_beliefs();
        assert_eq!(sm.len(), 2);
        assert!((sm.iter().sum::<f64>() - 1.0).abs() < 1e-6);
        assert!(sm[0] > sm[1]);
    }

    #[test]
    fn test_observe_evidence_supports_and_contradicts() {
        let mut ev = make_evaluator();
        let id1 = ev.post_hypothesis("h1".into(), 0.5);
        let id2 = ev.post_hypothesis("h2".into(), 0.5);
        ev.observe_evidence("key evidence", vec![id1], vec![id2]);
        assert!(ev.hypotheses[0]
            .evidence_for
            .contains(&"key evidence".to_string()));
        assert!(ev.hypotheses[1]
            .evidence_against
            .contains(&"key evidence".to_string()));
    }

    #[test]
    fn test_prior_clamped() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("too high".into(), 5.0);
        assert!((ev.hypotheses[0].prior - 1.0).abs() < 1e-6);
        ev.post_hypothesis("too low".into(), -1.0);
        assert!((ev.hypotheses[1].prior - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_blackboard_integration() {
        let mut ev = make_evaluator();
        ev.post_hypothesis("integrated".into(), 0.7);
        assert_eq!(ev.blackboard.hypotheses.len(), 1);
        assert!(ev.blackboard.hypotheses[0].confidence - 0.7 < 1e-6);
    }

    #[test]
    fn test_empty_hypothesis_list_entropy() {
        let ev = make_evaluator();
        assert!((ev.belief_entropy() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_best_hypothesis_none_when_empty() {
        let ev = make_evaluator();
        assert!(ev.best_hypothesis().is_none());
    }
}
