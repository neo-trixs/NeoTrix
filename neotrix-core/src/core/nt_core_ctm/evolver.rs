use super::chunk::{Chunk, ScoreWeights};
use std::collections::HashMap;

pub struct ProcessorEvolver {
    history: HashMap<String, (u64, u64, u64, Vec<f64>)>,
    adaptive_weights: HashMap<String, ScoreWeights>,
    window: usize,
    learning_rate: f64,
    stagnation_threshold: u64,
}

impl ProcessorEvolver {
    pub fn new(window: usize, learning_rate: f64, stagnation_threshold: u64) -> Self {
        Self {
            history: HashMap::new(),
            adaptive_weights: HashMap::new(),
            window,
            learning_rate,
            stagnation_threshold,
        }
    }

    fn ensure_processor(&mut self, processor: &str) {
        self.history
            .entry(processor.to_string())
            .or_insert((0, 0, 0, Vec::new()));
        self.adaptive_weights
            .entry(processor.to_string())
            .or_insert_with(ScoreWeights::default);
    }

    pub fn record_outcome(&mut self, processor: &str, score: f64, won: bool) {
        self.ensure_processor(processor);
        let key = processor.to_string();
        let entry = self.history.entry(key).or_insert((0, 0, 0, Vec::new()));
        entry.2 += 1;
        if won {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
        let scores = &mut entry.3;
        scores.push(score);
        if scores.len() > self.window {
            scores.remove(0);
        }
        self.evolve_weights(processor, won);
    }

    pub fn record_outcomes(&mut self, chunks: &[Chunk], winner_name: &str) {
        for chunk in chunks {
            let won = chunk.processor_name == winner_name;
            let score = chunk.weight(&ScoreWeights::default());
            self.record_outcome(&chunk.processor_name, score, won);
        }
    }

    pub fn adaptive_weights_for(&self, processor: &str) -> ScoreWeights {
        self.adaptive_weights
            .get(processor)
            .cloned()
            .unwrap_or_default()
    }

    pub fn win_rate(&self, processor: &str) -> f64 {
        self.history
            .get(processor)
            .map(|(wins, _, total, _)| {
                if *total == 0 {
                    0.0
                } else {
                    *wins as f64 / *total as f64
                }
            })
            .unwrap_or(0.0)
    }

    pub fn is_stagnating(&self, processor: &str) -> bool {
        self.history
            .get(processor)
            .map(|(wins, _, total, _)| {
                if *total == 0 {
                    return true;
                }
                let recent_wins_start = total.saturating_sub(self.stagnation_threshold);
                let recent_wins = *wins as u64 - recent_wins_start.min(*wins);
                recent_wins == 0 && *total >= self.stagnation_threshold
            })
            .unwrap_or(true)
    }

    pub fn stagnating_processors(&self) -> Vec<String> {
        self.history
            .keys()
            .filter(|p| self.is_stagnating(p))
            .cloned()
            .collect()
    }

    pub fn evolve_weights(&mut self, processor: &str, won: bool) {
        let lr = self.learning_rate;
        self.ensure_processor(processor);
        let weights = self
            .adaptive_weights
            .entry(processor.to_string())
            .or_insert_with(ScoreWeights::default);
        if won {
            weights.relevance = (weights.relevance + lr).min(3.0);
            weights.confidence = (weights.confidence + lr * 0.5).min(3.0);
        } else {
            weights.relevance = (weights.relevance - lr * 0.5).max(0.1);
            weights.surprise = (weights.surprise + lr * 0.3).min(3.0);
        }
    }

    pub fn stats(&self) -> EvolverStats {
        let total_inferences: u64 = self.history.values().map(|(_, _, t, _)| t).sum();
        let total_processors = self.history.len();
        let avg_win_rate = if total_processors == 0 {
            0.0
        } else {
            let sum: f64 = self.history.keys().map(|p| self.win_rate(p)).sum();
            sum / total_processors as f64
        };
        let stagnating_count = self.stagnating_processors().len();

        EvolverStats {
            total_inferences,
            avg_win_rate,
            stagnating_count,
            total_processors,
        }
    }
}

impl Default for ProcessorEvolver {
    fn default() -> Self {
        Self::new(50, 0.05, 20)
    }
}

pub struct EvolverStats {
    pub total_inferences: u64,
    pub avg_win_rate: f64,
    pub stagnating_count: usize,
    pub total_processors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk(name: &str, relevance: f64, confidence: f64, surprise: f64) -> Chunk {
        let mut c = Chunk::new(name, 0, vec![0u8; 16]);
        c.relevance = relevance;
        c.confidence = confidence;
        c.surprise = surprise;
        c
    }

    #[test]
    fn test_evolver_new() {
        let ev = ProcessorEvolver::new(10, 0.1, 5);
        assert_eq!(ev.window, 10);
        assert_eq!(ev.learning_rate, 0.1);
        assert_eq!(ev.stagnation_threshold, 5);
        assert!(ev.history.is_empty());
    }

    #[test]
    fn test_record_outcome_increases_wins() {
        let mut ev = ProcessorEvolver::new(10, 0.05, 5);
        ev.record_outcome("spatial", 0.8, true);
        ev.record_outcome("spatial", 0.6, true);
        ev.record_outcome("spatial", 0.3, false);

        let (wins, losses, total, _) = ev.history.get("spatial").unwrap();
        assert_eq!(*wins, 2);
        assert_eq!(*losses, 1);
        assert_eq!(*total, 3);
    }

    #[test]
    fn test_win_rate() {
        let mut ev = ProcessorEvolver::new(10, 0.05, 5);
        assert!((ev.win_rate("unknown") - 0.0).abs() < 1e-6);

        ev.record_outcome("p1", 1.0, true);
        ev.record_outcome("p1", 1.0, true);
        ev.record_outcome("p1", 0.0, false);
        assert!((ev.win_rate("p1") - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_stagnation_detection() {
        let mut ev = ProcessorEvolver::new(50, 0.05, 3);
        assert!(ev.is_stagnating("ghost"));
        for _ in 0..3 {
            ev.record_outcome("stuck", 0.2, false);
        }
        assert!(ev.is_stagnating("stuck"));
        ev.record_outcome("stuck", 0.9, true);
        assert!(!ev.is_stagnating("stuck"));
    }

    #[test]
    fn test_stagnating_processors() {
        let mut ev = ProcessorEvolver::new(50, 0.05, 3);
        ev.record_outcome("good", 1.0, true);
        ev.record_outcome("good", 1.0, true);
        ev.record_outcome("good", 1.0, true);
        for _ in 0..3 {
            ev.record_outcome("bad", 0.1, false);
        }
        let stagnating = ev.stagnating_processors();
        assert!(stagnating.contains(&"bad".to_string()));
        assert!(!stagnating.contains(&"good".to_string()));
    }

    #[test]
    fn test_evolve_weights_after_win() {
        let mut ev = ProcessorEvolver::new(10, 0.1, 5);
        let w = ev.adaptive_weights_for("p");
        assert!((w.relevance - 1.0).abs() < 1e-6);
        assert!((w.confidence - 1.0).abs() < 1e-6);

        ev.record_outcome("p", 0.9, true);
        let w = ev.adaptive_weights_for("p");
        assert!((w.relevance - 1.1).abs() < 1e-6);
        assert!((w.confidence - 1.05).abs() < 1e-6);
    }

    #[test]
    fn test_evolve_weights_after_loss() {
        let mut ev = ProcessorEvolver::new(10, 0.1, 5);
        ev.record_outcome("p", 0.2, false);
        let w = ev.adaptive_weights_for("p");
        assert!((w.relevance - 0.95).abs() < 1e-6);
        assert!((w.surprise - 0.23).abs() < 1e-6);
    }

    #[test]
    fn test_record_outcomes_batch() {
        let mut ev = ProcessorEvolver::new(10, 0.05, 5);
        let chunks = vec![
            make_chunk("a", 0.9, 0.8, 0.1),
            make_chunk("b", 0.5, 0.5, 0.3),
            make_chunk("c", 0.3, 0.4, 0.5),
        ];
        ev.record_outcomes(&chunks, "a");
        let (wins_a, losses_a, total_a, _) = ev.history.get("a").unwrap();
        assert_eq!(*wins_a, 1);
        assert_eq!(*losses_a, 0);
        assert_eq!(*total_a, 1);

        let (wins_b, losses_b, _, _) = ev.history.get("b").unwrap();
        assert_eq!(*wins_b, 0);
        assert_eq!(*losses_b, 1);
    }

    #[test]
    fn test_evolver_stats() {
        let mut ev = ProcessorEvolver::new(10, 0.05, 3);
        ev.record_outcome("p1", 1.0, true);
        ev.record_outcome("p1", 1.0, true);
        ev.record_outcome("p2", 0.0, false);
        ev.record_outcome("p2", 0.0, false);
        ev.record_outcome("p2", 0.0, false);
        let s = ev.stats();
        assert_eq!(s.total_inferences, 5);
        assert_eq!(s.total_processors, 2);
        assert!((s.avg_win_rate - 0.5).abs() < 1e-6);
        assert_eq!(s.stagnating_count, 1);
    }

    #[test]
    fn test_weights_clamp_min() {
        let mut ev = ProcessorEvolver::new(10, 2.0, 5);
        for _ in 0..10 {
            ev.record_outcome("p", 0.1, false);
        }
        let w = ev.adaptive_weights_for("p");
        assert!(w.relevance >= 0.1);
        assert!(w.surprise >= 0.1);
    }

    #[test]
    fn test_weights_clamp_max() {
        let mut ev = ProcessorEvolver::new(10, 2.0, 5);
        for _ in 0..10 {
            ev.record_outcome("p", 1.0, true);
        }
        let w = ev.adaptive_weights_for("p");
        assert!(w.relevance <= 3.0);
        assert!(w.confidence <= 3.0);
    }
}
