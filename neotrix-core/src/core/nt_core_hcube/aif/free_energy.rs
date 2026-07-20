#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeEnergyCalculator;

impl FreeEnergyCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_vfe(
        &self,
        belief: &[f64],
        observation: &[f64],
        likelihood: &[Vec<f64>],
    ) -> f64 {
        let uniform_prior = vec![1.0 / belief.len() as f64; belief.len()];
        let complexity = self.compute_complexity(belief, &uniform_prior);
        let accuracy = self.compute_accuracy(belief, observation, likelihood);
        (complexity + accuracy).max(-1e10).min(1e10)
    }

    pub fn compute_efe(
        &self,
        belief: &[f64],
        transition: &[Vec<f64>],
        prior: &[f64],
        horizon: usize,
    ) -> f64 {
        let mut total_efe = 0.0;
        let mut current_belief = belief.to_vec();

        for _step in 0..horizon {
            let mut next_belief = vec![0.0; current_belief.len()];
            for i in 0..current_belief.len() {
                for j in 0..current_belief.len() {
                    next_belief[j] += current_belief[i] * transition[i][j];
                }
            }
            current_belief = next_belief;

            let complexity = self.compute_complexity(&current_belief, prior);

            let mut inaccuracy = 0.0;
            for s in 0..current_belief.len() {
                if current_belief[s] > 0.0 {
                    let log_val = (current_belief[s] / (prior[s] + 1e-10) + 1e-10).ln();
                    inaccuracy += current_belief[s] * log_val;
                }
            }

            total_efe += complexity + inaccuracy;
        }

        total_efe.max(-1e10).min(1e10)
    }

    pub fn compute_complexity(&self, belief: &[f64], prior: &[f64]) -> f64 {
        self.compute_kl_divergence(belief, prior)
    }

    pub fn compute_accuracy(
        &self,
        belief: &[f64],
        observation: &[f64],
        likelihood: &[Vec<f64>],
    ) -> f64 {
        let num_obs = observation.len();
        let mut log_evidence = 0.0;

        for o in 0..num_obs {
            let mut predicted_obs = 0.0;
            for s in 0..belief.len() {
                predicted_obs += belief[s] * likelihood[s][o];
            }
            if predicted_obs > 0.0 && observation[o] > 0.0 {
                log_evidence += observation[o] * predicted_obs.ln();
            }
        }

        -log_evidence
    }

    pub fn compute_entropy(&self, distribution: &[f64]) -> f64 {
        let mut entropy = 0.0;
        for &p in distribution {
            if p > 0.0 {
                entropy -= p * (p + 1e-10).ln();
            }
        }
        entropy.max(0.0)
    }

    pub fn compute_kl_divergence(&self, p: &[f64], q: &[f64]) -> f64 {
        let mut kl = 0.0;
        for i in 0..p.len() {
            if p[i] > 0.0 {
                kl += p[i] * ((p[i] / (q[i] + 1e-10)) + 1e-10).ln();
            }
        }
        kl.max(0.0)
    }
}

impl Default for FreeEnergyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_vfe_basic() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![0.5, 0.5];
        let observation = vec![1.0, 0.0];
        let likelihood = vec![vec![0.9, 0.1], vec![0.2, 0.8]];
        let vfe = calc.compute_vfe(&belief, &observation, &likelihood);
        assert!(vfe.is_finite());
        assert!(vfe > 0.0);
    }

    #[test]
    fn test_compute_vfe_certain_belief() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![1.0, 0.0];
        let observation = vec![1.0, 0.0];
        let likelihood = vec![vec![1.0, 0.0], vec![0.5, 0.5]];
        let vfe = calc.compute_vfe(&belief, &observation, &likelihood);
        assert!(vfe.is_finite());
    }

    #[test]
    fn test_compute_efe_basic() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![0.5, 0.5];
        let transition = vec![vec![0.8, 0.2], vec![0.3, 0.7]];
        let prior = vec![0.5, 0.5];
        let efe = calc.compute_efe(&belief, &transition, &prior, 3);
        assert!(efe.is_finite());
    }

    #[test]
    fn test_compute_efe_deterministic() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![1.0, 0.0];
        let transition = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let prior = vec![0.5, 0.5];
        let efe = calc.compute_efe(&belief, &transition, &prior, 5);
        assert!(efe.is_finite());
    }

    #[test]
    fn test_entropy_uniform() {
        let calc = FreeEnergyCalculator::new();
        let dist = vec![0.25, 0.25, 0.25, 0.25];
        let entropy = calc.compute_entropy(&dist);
        let expected = 4.0 * 0.25 * (4.0f64).ln();
        assert!((entropy - expected).abs() < 1e-9);
    }

    #[test]
    fn test_entropy_deterministic() {
        let calc = FreeEnergyCalculator::new();
        let dist = vec![1.0, 0.0];
        let entropy = calc.compute_entropy(&dist);
        assert!((entropy).abs() < 1e-10);
    }

    #[test]
    fn test_kl_divergence_identical() {
        let calc = FreeEnergyCalculator::new();
        let p = vec![0.5, 0.5];
        let q = vec![0.5, 0.5];
        let kl = calc.compute_kl_divergence(&p, &q);
        assert!((kl).abs() < 1e-10);
    }

    #[test]
    fn test_kl_divergence_different() {
        let calc = FreeEnergyCalculator::new();
        let p = vec![1.0, 0.0];
        let q = vec![0.5, 0.5];
        let kl = calc.compute_kl_divergence(&p, &q);
        assert!(kl > 0.0);
    }

    #[test]
    fn test_accuracy_perfect() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![1.0, 0.0];
        let observation = vec![1.0, 0.0];
        let likelihood = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let acc = calc.compute_accuracy(&belief, &observation, &likelihood);
        assert!((acc).abs() < 1e-10);
    }

    #[test]
    fn test_complexity_zero_for_uniform() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![0.5, 0.5];
        let prior = vec![0.5, 0.5];
        let c = calc.compute_complexity(&belief, &prior);
        assert!((c).abs() < 1e-10);
    }

    #[test]
    fn test_edge_case_single_state() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![1.0];
        let observation = vec![1.0];
        let likelihood = vec![vec![1.0]];
        let vfe = calc.compute_vfe(&belief, &observation, &likelihood);
        assert!(vfe.is_finite());
        let entropy = calc.compute_entropy(&belief);
        assert!((entropy).abs() < 1e-10);
    }

    #[test]
    fn test_efe_horizon_zero() {
        let calc = FreeEnergyCalculator::new();
        let belief = vec![0.5, 0.5];
        let transition = vec![vec![0.8, 0.2], vec![0.3, 0.7]];
        let prior = vec![0.5, 0.5];
        let efe = calc.compute_efe(&belief, &transition, &prior, 0);
        assert!((efe).abs() < 1e-10);
    }

    #[test]
    fn test_kl_divergence_zero_probability() {
        let calc = FreeEnergyCalculator::new();
        let p = vec![0.0, 1.0];
        let q = vec![0.5, 0.5];
        let kl = calc.compute_kl_divergence(&p, &q);
        assert!(kl.is_finite());
        assert!(kl >= 0.0);
    }
}
