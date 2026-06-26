use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum DeadEndSignal {
    Looping,
    Oscillating,
    Diverging,
    Plateaued,
    Converging,
}

#[derive(Debug, Clone)]
pub struct TrajectoryPoint {
    pub vsa_vector: Vec<u8>,
    pub cycle: u64,
    pub confidence: f64,
    pub step_label: String,
}

impl TrajectoryPoint {
    pub fn new(vsa_vector: Vec<u8>, cycle: u64, confidence: f64, step_label: &str) -> Self {
        Self {
            vsa_vector,
            cycle,
            confidence,
            step_label: step_label.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeadEndDetectorConfig {
    pub window_size: usize,
    pub oscillation_threshold: f64,
    pub plateau_threshold: f64,
    pub convergence_threshold: f64,
    pub min_trajectory_length: usize,
}

impl Default for DeadEndDetectorConfig {
    fn default() -> Self {
        Self {
            window_size: 10,
            oscillation_threshold: 0.85,
            plateau_threshold: 0.95,
            convergence_threshold: 0.92,
            min_trajectory_length: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchRecommendation {
    Continue,
    Backtrack(u64),
    RandomRestart,
    MetaReplan(String),
}

#[derive(Debug, Clone)]
pub struct DeadEndDetector {
    config: DeadEndDetectorConfig,
    trajectory: VecDeque<TrajectoryPoint>,
}

impl DeadEndDetector {
    pub fn new(config: DeadEndDetectorConfig) -> Self {
        Self {
            config,
            trajectory: VecDeque::new(),
        }
    }

    pub fn record_point(&mut self, point: TrajectoryPoint) {
        self.trajectory.push_back(point);
        let max_history = self.config.window_size * 3;
        while self.trajectory.len() > max_history {
            self.trajectory.pop_front();
        }
    }

    pub fn detect_dead_end(&self) -> Option<DeadEndSignal> {
        if self.trajectory.len() < self.config.min_trajectory_length {
            return None;
        }

        if self.check_plateaued() {
            return Some(DeadEndSignal::Plateaued);
        }

        if self.check_looping() {
            return Some(DeadEndSignal::Looping);
        }

        if self.check_oscillating() {
            return Some(DeadEndSignal::Oscillating);
        }

        if self.check_diverging() {
            return Some(DeadEndSignal::Diverging);
        }

        if self.check_converging() {
            return Some(DeadEndSignal::Converging);
        }

        None
    }

    fn check_looping(&self) -> bool {
        let n = self.trajectory.len();
        let half = self.config.window_size / 2;
        if n < half * 2 {
            return false;
        }
        let pattern_start = n - half;
        for candidate_start in (0..=n - half * 2).rev() {
            let matches = (0..half)
                .filter(|&i| {
                    QuantizedVSA::similarity(
                        &self.trajectory[pattern_start + i].vsa_vector,
                        &self.trajectory[candidate_start + i].vsa_vector,
                    ) > 0.9
                })
                .count();
            if matches >= half - 1 {
                return true;
            }
        }
        false
    }

    fn check_oscillating(&self) -> bool {
        let n = self.trajectory.len();
        let window = self.config.window_size.min(n);
        if window < 4 {
            return false;
        }
        let start = n - window;
        let evens: Vec<&[u8]> = (start..n)
            .step_by(2)
            .map(|i| self.trajectory[i].vsa_vector.as_slice())
            .collect();
        let odds: Vec<&[u8]> = (start + 1..n)
            .step_by(2)
            .map(|i| self.trajectory[i].vsa_vector.as_slice())
            .collect();
        if evens.len() < 2 || odds.len() < 2 {
            return false;
        }
        let even_sim = QuantizedVSA::similarity(evens[0], evens[1]);
        let odd_sim = QuantizedVSA::similarity(odds[0], odds[1]);
        let cross_sim = QuantizedVSA::similarity(evens[0], odds[0]);
        even_sim > self.config.oscillation_threshold
            && odd_sim > self.config.oscillation_threshold
            && cross_sim < (1.0 - self.config.oscillation_threshold)
    }

    fn check_diverging(&self) -> bool {
        let n = self.trajectory.len();
        let window = self.config.window_size.min(n);
        if window < 3 {
            return false;
        }
        let start = n - window;
        let confs: Vec<f64> = (start..n).map(|i| self.trajectory[i].confidence).collect();
        confs.windows(2).all(|w| w[0] > w[1])
    }

    fn check_plateaued(&self) -> bool {
        let n = self.trajectory.len();
        let window = self.config.window_size.min(n);
        if window < 3 {
            return false;
        }
        let start = n - window;
        for i in start..n - 1 {
            let sim = QuantizedVSA::similarity(
                &self.trajectory[i].vsa_vector,
                &self.trajectory[i + 1].vsa_vector,
            );
            if sim < self.config.plateau_threshold {
                return false;
            }
        }
        true
    }

    fn check_converging(&self) -> bool {
        let n = self.trajectory.len();
        let check_len = self.config.min_trajectory_length.min(n);
        if check_len < 3 {
            return false;
        }
        let start = n - check_len;
        let last = &self.trajectory[n - 1].vsa_vector;
        (start..n - 1).all(|i| {
            QuantizedVSA::similarity(&self.trajectory[i].vsa_vector, last)
                > self.config.convergence_threshold
        })
    }

    pub fn remaining_paths(
        &self,
        trajectory: &[TrajectoryPoint],
        alternatives: &[Vec<u8>],
    ) -> usize {
        alternatives
            .iter()
            .filter(|alt| {
                !trajectory
                    .iter()
                    .any(|tp| QuantizedVSA::similarity(&tp.vsa_vector, alt) > 0.85)
            })
            .count()
    }

    pub fn suggest_switch(&self) -> SwitchRecommendation {
        match self.detect_dead_end() {
            None => SwitchRecommendation::Continue,
            Some(DeadEndSignal::Looping) => SwitchRecommendation::Backtrack(5),
            Some(DeadEndSignal::Oscillating) => SwitchRecommendation::RandomRestart,
            Some(DeadEndSignal::Diverging) => {
                SwitchRecommendation::MetaReplan("confidence collapse".to_string())
            }
            Some(DeadEndSignal::Plateaued) => SwitchRecommendation::Backtrack(3),
            Some(DeadEndSignal::Converging) => SwitchRecommendation::Continue,
        }
    }

    pub fn reset(&mut self) {
        self.trajectory.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DIM: usize = 256;

    fn make_vec(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, TEST_DIM)
    }

    fn make_point(seed: u64, cycle: u64, confidence: f64, label: &str) -> TrajectoryPoint {
        TrajectoryPoint::new(make_vec(seed), cycle, confidence, label)
    }

    #[test]
    fn test_empty_trajectory() {
        let config = DeadEndDetectorConfig::default();
        let detector = DeadEndDetector::new(config);
        assert!(detector.detect_dead_end().is_none());
    }

    #[test]
    fn test_short_trajectory() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..3 {
            detector.record_point(make_point(i, i as u64, 0.8, "step"));
        }
        assert!(detector.detect_dead_end().is_none());
    }

    #[test]
    fn test_looping_detection() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..15 {
            let seed = match i % 3 {
                0 => 1,
                1 => 2,
                _ => 3,
            };
            detector.record_point(make_point(seed, i as u64, 0.7, "loop"));
        }
        assert_eq!(detector.detect_dead_end(), Some(DeadEndSignal::Looping));
    }

    #[test]
    fn test_oscillating_detection() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..12 {
            let seed = if i % 2 == 0 { 10 } else { 20 };
            detector.record_point(make_point(seed, i as u64, 0.6, "osc"));
        }
        assert_eq!(detector.detect_dead_end(), Some(DeadEndSignal::Oscillating));
    }

    #[test]
    fn test_diverging_detection() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..12 {
            let confidence = 0.9 - (i as f64) * 0.1;
            detector.record_point(make_point(i as u64, i as u64, confidence, "diverge"));
        }
        assert_eq!(detector.detect_dead_end(), Some(DeadEndSignal::Diverging));
    }

    #[test]
    fn test_plateaued_detection() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..12 {
            detector.record_point(make_point(42, i as u64, 0.5, "plat"));
        }
        assert_eq!(detector.detect_dead_end(), Some(DeadEndSignal::Plateaued));
    }

    #[test]
    fn test_converging_detection() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..7 {
            detector.record_point(make_point(100 + i, i as u64, 0.6, "explore"));
        }
        for i in 0..5 {
            detector.record_point(make_point(42, 7 + i, 0.8, "converge"));
        }
        assert_eq!(detector.detect_dead_end(), Some(DeadEndSignal::Converging));
    }

    #[test]
    fn test_no_dead_end() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..15 {
            detector.record_point(make_point((i as u64).wrapping_mul(37) + 7, i as u64, 0.5, "rand"));
        }
        assert_eq!(detector.detect_dead_end(), None);
    }

    #[test]
    fn test_remaining_paths_all_unexplored() {
        let config = DeadEndDetectorConfig::default();
        let detector = DeadEndDetector::new(config);
        let trajectory = vec![make_point(1, 0, 0.5, "a")];
        let alternatives = vec![make_vec(10), make_vec(20), make_vec(30)];
        assert_eq!(detector.remaining_paths(&trajectory, &alternatives), 3);
    }

    #[test]
    fn test_remaining_paths_some_visited() {
        let config = DeadEndDetectorConfig::default();
        let detector = DeadEndDetector::new(config);
        let trajectory = vec![make_point(10, 0, 0.5, "a")];
        let alternatives = vec![make_vec(10), make_vec(20), make_vec(30)];
        assert_eq!(detector.remaining_paths(&trajectory, &alternatives), 2);
    }

    #[test]
    fn test_suggest_switch_continue() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..8 {
            detector.record_point(make_point((i as u64).wrapping_mul(37), i as u64, 0.5, "step"));
        }
        assert_eq!(detector.suggest_switch(), SwitchRecommendation::Continue);
    }

    #[test]
    fn test_suggest_switch_backtrack_on_looping() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..15 {
            let seed = match i % 3 {
                0 => 1,
                1 => 2,
                _ => 3,
            };
            detector.record_point(make_point(seed, i as u64, 0.7, "loop"));
        }
        assert_eq!(
            detector.suggest_switch(),
            SwitchRecommendation::Backtrack(5)
        );
    }

    #[test]
    fn test_suggest_switch_random_restart_on_oscillating() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..12 {
            let seed = if i % 2 == 0 { 10 } else { 20 };
            detector.record_point(make_point(seed, i as u64, 0.6, "osc"));
        }
        assert_eq!(
            detector.suggest_switch(),
            SwitchRecommendation::RandomRestart
        );
    }

    #[test]
    fn test_suggest_switch_on_diverging() {
        let mut config = DeadEndDetectorConfig::default();
        config.min_trajectory_length = 3;
        let mut detector = DeadEndDetector::new(config);
        for i in 0..12 {
            let confidence = 0.9 - (i as f64) * 0.1;
            detector.record_point(make_point(i as u64, i as u64, confidence, "diverge"));
        }
        assert_eq!(
            detector.suggest_switch(),
            SwitchRecommendation::MetaReplan("confidence collapse".to_string())
        );
    }

    #[test]
    fn test_reset_clears_trajectory() {
        let config = DeadEndDetectorConfig::default();
        let mut detector = DeadEndDetector::new(config);
        for i in 0..12 {
            detector.record_point(make_point(42, i as u64, 0.5, "plat"));
        }
        assert!(detector.detect_dead_end().is_some());
        detector.reset();
        assert!(detector.detect_dead_end().is_none());
    }
}
