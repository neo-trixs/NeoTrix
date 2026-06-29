// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::VecDeque;

/// Boredom signal accumulator — curiosity modulation from prediction error
#[derive(Clone)]
pub struct BoredomAccumulator {
    pub boredom: f64,
    pub curiosity: f64,
    pub decay_rate: f64,
    pub novelty_threshold: f64,
    pub boredom_history: VecDeque<f64>,
    pub max_history: usize,
}

impl BoredomAccumulator {
    pub fn new() -> Self {
        BoredomAccumulator {
            boredom: 0.0,
            curiosity: 0.5,
            decay_rate: 0.05,
            novelty_threshold: 0.3,
            boredom_history: VecDeque::with_capacity(50),
            max_history: 1000,
        }
    }

    pub fn update(&mut self, prediction_error: f64, novelty: f64) {
        let pe = prediction_error.clamp(0.0, 1.0);

        if pe < self.novelty_threshold && novelty < 0.2 {
            self.boredom = (self.boredom + 0.05 * (1.0 - pe)).min(1.0);
        } else {
            self.boredom = (self.boredom - 0.1 * pe).max(0.0);
        }

        self.curiosity = (0.5 + self.boredom * 0.3 + novelty * 0.2).clamp(0.0, 1.0);
        self.boredom = (self.boredom * (1.0 - self.decay_rate)).clamp(0.0, 1.0);

        if self.boredom_history.len() >= self.max_history {
            self.boredom_history.pop_front();
        }
        self.boredom_history.push_back(self.boredom);
    }

    pub fn average_boredom(&self) -> f64 {
        let n = self.boredom_history.len();
        if n == 0 {
            return self.boredom;
        }
        self.boredom_history.iter().sum::<f64>() / n as f64
    }

    pub fn should_explore(&self) -> bool {
        self.boredom > 0.6 || self.curiosity > 0.7
    }

    pub fn reset(&mut self) {
        self.boredom = 0.0;
        self.curiosity = 0.5;
        self.boredom_history.clear();
    }

    pub fn report(&self) -> String {
        format!(
            "BoredomAccumulator: boredom={:.2}, curiosity={:.2}, avg_boredom={:.2}, explore={}",
            self.boredom,
            self.curiosity,
            self.average_boredom(),
            self.should_explore(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let b = BoredomAccumulator::new();
        assert!((b.boredom - 0.0).abs() < 0.01);
        assert!((b.curiosity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_low_novelty_increases_boredom() {
        let mut b = BoredomAccumulator::new();
        for _ in 0..20 {
            b.update(0.05, 0.05);
        }
        assert!(
            b.boredom > 0.3,
            "repeated low novelty should increase boredom"
        );
    }

    #[test]
    fn test_high_novelty_decreases_boredom() {
        let mut b = BoredomAccumulator::new();
        b.boredom = 0.8;
        b.update(0.9, 0.8);
        assert!(b.boredom < 0.8, "high novelty should decrease boredom");
    }

    #[test]
    fn test_should_explore_when_bored() {
        let mut b = BoredomAccumulator::new();
        b.boredom = 0.7;
        assert!(b.should_explore());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut b = BoredomAccumulator::new();
        b.boredom = 0.9;
        b.reset();
        assert!((b.boredom - 0.0).abs() < 0.01);
        assert!((b.curiosity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_report_contains_info() {
        let b = BoredomAccumulator::new();
        let r = b.report();
        assert!(r.contains("BoredomAccumulator"));
    }

    #[test]
    fn test_average_boredom() {
        let mut b = BoredomAccumulator::new();
        b.update(0.1, 0.1);
        b.update(0.1, 0.1);
        let avg = b.average_boredom();
        assert!(avg >= 0.0);
    }
}

/// VSA-aware exploration driven by boredom/curiosity
#[derive(Clone)]
pub struct ExplorationDriver {
    pub accumulator: BoredomAccumulator,
    pub known_vectors: Vec<Vec<f64>>,
    pub max_known: usize,
}

impl ExplorationDriver {
    pub fn new() -> Self {
        ExplorationDriver {
            accumulator: BoredomAccumulator::new(),
            known_vectors: Vec::with_capacity(64),
            max_known: 1000,
        }
    }

    /// Evaluate novelty as 1 - max cosine similarity to known vectors
    pub fn novelty(&self, vector: &[f64]) -> f64 {
        if self.known_vectors.is_empty() {
            return 1.0;
        }
        let max_sim = self.known_vectors.iter()
            .map(|k| cosine_similarity(vector, k))
            .fold(0.0f64, f64::max);
        1.0 - max_sim
    }

    /// Record exploration of a vector
    pub fn explore(&mut self, vector: Vec<f64>, prediction_error: f64, novelty_val: f64) {
        self.accumulator.update(prediction_error, novelty_val);
        if self.known_vectors.len() < self.max_known {
            self.known_vectors.push(vector);
        }
    }

    /// Select best candidate by combined novelty + curiosity
    pub fn select_target<'a>(&self, candidates: &[(&'a str, Vec<f64>)]) -> Option<&'a str> {
        if candidates.is_empty() {
            return None;
        }
        candidates.iter()
            .map(|(name, vec)| {
                let n = self.novelty(vec);
                let score = n * 0.5 + self.accumulator.curiosity * 0.3 + n * self.accumulator.boredom * 0.2;
                (name, score)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| *name)
    }

    pub fn known_count(&self) -> usize {
        self.known_vectors.len()
    }
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).take(len).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().take(len).map(|x| x * x).sum();
    let nb: f64 = b.iter().take(len).map(|x| x * x).sum();
    let denom = na.sqrt() * nb.sqrt();
    if denom < 1e-12 { 0.0 } else { dot / denom }
}

#[cfg(test)]
mod exploration_tests {
    use super::*;

    fn make_vec(v: f64) -> Vec<f64> {
        vec![v; 64]
    }

    #[test]
    fn test_novelty_empty_returns_high() {
        let d = ExplorationDriver::new();
        assert!((d.novelty(&make_vec(1.0)) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_novelty_identical_is_zero() {
        let mut d = ExplorationDriver::new();
        d.known_vectors.push(make_vec(1.0));
        assert!(d.novelty(&make_vec(1.0)) < 0.01);
    }

    #[test]
    fn test_novelty_orthogonal_is_high() {
        let mut d = ExplorationDriver::new();
        d.known_vectors.push(make_vec(1.0));
        let ortho: Vec<f64> = (0..64).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();
        assert!(d.novelty(&ortho) > 0.5);
    }

    #[test]
    fn test_explore_grows_known() {
        let mut d = ExplorationDriver::new();
        d.explore(make_vec(1.0), 0.1, 0.9);
        assert_eq!(d.known_count(), 1);
    }

    #[test]
    fn test_select_target_returns_best() {
        let mut d = ExplorationDriver::new();
        d.known_vectors.push(make_vec(1.0));
        let result = d.select_target(&[("a", make_vec(1.0)), ("b", make_vec(0.0))]);
        assert_eq!(result, Some("b"));
    }

    #[test]
    fn test_select_target_empty() {
        let d = ExplorationDriver::new();
        assert!(d.select_target(&[]).is_none());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = make_vec(1.0);
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_update_modifies_boredom() {
        let mut d = ExplorationDriver::new();
        d.explore(make_vec(1.0), 0.05, 0.05);
        d.explore(make_vec(1.0), 0.05, 0.05);
        assert!(d.accumulator.boredom > 0.0);
    }
}
