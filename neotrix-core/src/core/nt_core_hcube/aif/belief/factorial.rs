#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub trait POMDPFactor {
    fn num_states(&self) -> usize;
    fn transition(&self, from: usize, to: usize) -> f64;
    fn likelihood(&self, state: usize, obs: usize) -> f64;
}

pub struct FactorialPOMDP {
    pub factors: Vec<Box<dyn POMDPFactor>>,
}

impl FactorialPOMDP {
    pub fn new(factors: Vec<Box<dyn POMDPFactor>>) -> Self {
        FactorialPOMDP { factors }
    }

    pub fn update(&self, beliefs: &[Vec<f64>], observations: &[usize]) -> Vec<Vec<f64>> {
        let mut updated = Vec::with_capacity(self.factors.len());
        for (i, factor) in self.factors.iter().enumerate() {
            let obs = if i < observations.len() {
                observations[i]
            } else {
                0
            };
            let default_belief;
            let belief = if i < beliefs.len() {
                &beliefs[i]
            } else {
                let n = factor.num_states();
                let uniform = 1.0 / n as f64;
                default_belief = vec![uniform; n];
                &default_belief
            };

            let n = factor.num_states();
            let mut posterior = vec![0.0; n];
            let mut evidence = 0.0;

            for s in 0..n {
                posterior[s] = factor.likelihood(s, obs) * belief[s];
                evidence += posterior[s];
            }

            if evidence > 0.0 {
                for p in &mut posterior {
                    *p /= evidence;
                }
            } else {
                let uniform = 1.0 / n as f64;
                posterior.fill(uniform);
            }

            updated.push(posterior);
        }
        updated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorGraphBeliefPropagation {
    pub max_iterations: usize,
    pub tolerance: f64,
}

impl FactorGraphBeliefPropagation {
    pub fn new(max_iterations: usize, tolerance: f64) -> Self {
        FactorGraphBeliefPropagation {
            max_iterations,
            tolerance,
        }
    }

    pub fn run(
        &self,
        beliefs: &[Vec<f64>],
        max_iters: usize,
        tol: f64,
    ) -> Vec<Vec<f64>> {
        if beliefs.is_empty() {
            return beliefs.to_vec();
        }

        let n_factors = beliefs.len();
        let mut current = beliefs.to_vec();

        let actual_max_iters = max_iters.min(self.max_iterations);

        for _iter in 0..actual_max_iters {
            let mut messages = vec![vec![0.0; current[0].len()]; n_factors];

            for i in 0..n_factors {
                let mut msg_sum = 0.0;
                for j in 0..current[i].len() {
                    let mut product = 1.0;
                    for k in 0..n_factors {
                        if k != i {
                            product *= current[k][j];
                        }
                    }
                    messages[i][j] = current[i][j] * product;
                    msg_sum += messages[i][j];
                }
                if msg_sum > 0.0 {
                    for j in 0..messages[i].len() {
                        messages[i][j] /= msg_sum;
                    }
                } else {
                    let uniform = 1.0 / messages[i].len() as f64;
                    for j in 0..messages[i].len() {
                        messages[i][j] = uniform;
                    }
                }
            }

            let mut max_change = 0.0;
            for i in 0..n_factors {
                for j in 0..current[i].len() {
                    let change = (messages[i][j] - current[i][j]).abs();
                    if change > max_change {
                        max_change = change;
                    }
                }
            }

            current = messages;

            if max_change < tol {
                break;
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFactor {
        n: usize,
    }

    impl POMDPFactor for TestFactor {
        fn num_states(&self) -> usize {
            self.n
        }

        fn transition(&self, from: usize, to: usize) -> f64 {
            if from == to { 0.9 } else { 0.1 / (self.n as f64 - 1.0) }
        }

        fn likelihood(&self, state: usize, obs: usize) -> f64 {
            if state == obs { 0.9 } else { 0.1 / (self.n as f64 - 1.0) }
        }
    }

    #[test]
    fn test_factorial_pomdp_update() {
        let factors: Vec<Box<dyn POMDPFactor>> = vec![
            Box::new(TestFactor { n: 2 }),
            Box::new(TestFactor { n: 2 }),
        ];
        let pomdp = FactorialPOMDP::new(factors);
        let beliefs = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let updated = pomdp.update(&beliefs, &[0, 1]);
        assert_eq!(updated.len(), 2);
        for b in &updated {
            let sum: f64 = b.iter().sum();
            assert!((sum - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_factor_graph_belief_propagation_convergence() {
        let bp = FactorGraphBeliefPropagation::new(100, 1e-6);
        let beliefs = vec![vec![0.6, 0.4], vec![0.3, 0.7]];
        let result = bp.run(&beliefs, 100, 1e-6);
        assert_eq!(result.len(), 2);
        for b in &result {
            let sum: f64 = b.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_factor_graph_belief_propagation_empty() {
        let bp = FactorGraphBeliefPropagation::new(100, 1e-6);
        let beliefs: Vec<Vec<f64>> = vec![];
        let result = bp.run(&beliefs, 100, 1e-6);
        assert!(result.is_empty());
    }

    #[test]
    fn test_factorial_pomdp_evidence_zero() {
        let factors: Vec<Box<dyn POMDPFactor>> = vec![
            Box::new(TestFactor { n: 2 }),
            Box::new(TestFactor { n: 2 }),
        ];
        let pomdp = FactorialPOMDP::new(factors);
        let bad_beliefs = vec![vec![0.0, 0.0], vec![0.0, 1.0]];
        let updated = pomdp.update(&bad_beliefs, &[0, 1]);
        assert_eq!(updated.len(), 2);
        for b in &updated {
            let sum: f64 = b.iter().sum();
            assert!((sum - 1.0).abs() < 1e-10);
        }
    }
}
