//! # System 1 — Fast, Intuitive, Associative Reasoning
//!
//! Quick pattern matching and intuition, no deliberate computation.
//! Operates directly on VSA vectors via similarity and associative recall.

use crate::memory::hypercube::HyperCube;

#[derive(Debug, Clone)]
pub struct System1 {
    /// Associative memory (pattern → response)
    associations: Vec<(Vec<f64>, Vec<f64>)>,
    max_associations: usize,
}

impl System1 {
    pub fn new() -> Self {
        Self {
            associations: Vec::new(),
            max_associations: 1024,
        }
    }

    /// Associate an input pattern with a response
    pub fn learn(&mut self, input: Vec<f64>, output: Vec<f64>) {
        if self.associations.len() >= self.max_associations {
            self.associations.remove(0);
        }
        self.associations.push((input, output));
    }

    /// Fast associative recall — return the most similar learned response
    pub fn recall(&self, input: &[f64]) -> Option<&[f64]> {
        self.associations
            .iter()
            .max_by(|(a, _), (b, _)| {
                let sim_a = HyperCube::similarity(a, input);
                let sim_b = HyperCube::similarity(b, input);
                sim_a.partial_cmp(&sim_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, output)| output.as_slice())
    }

    /// Confidence in the recall (highest similarity)
    pub fn confidence(&self, input: &[f64]) -> f64 {
        self.associations
            .iter()
            .map(|(a, _)| HyperCube::similarity(a, input))
            .fold(0.0_f64, |a: f64, b: f64| a.max(b))
    }

    pub fn len(&self) -> usize {
        self.associations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.associations.is_empty()
    }
}

impl Default for System1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::hypercube::HyperCube;

    #[test]
    fn test_system1_empty_recall() {
        let s1 = System1::new();
        let input = vec![1.0, 0.0];
        assert!(s1.recall(&input).is_none());
    }

    #[test]
    fn test_system1_learn_and_recall() {
        let mut s1 = System1::new();
        let hc = HyperCube::new(8);
        let input = hc.seeded_vector(1);
        let output = hc.seeded_vector(2);
        s1.learn(input.clone(), output.clone());
        let recalled = s1.recall(&input).unwrap();
        assert!((HyperCube::similarity(recalled, &output) - 1.0).abs() < 1e-9);
    }
}
