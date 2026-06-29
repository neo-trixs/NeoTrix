use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

const MAX_SWITCH_HISTORY: usize = 100;
const HIGH_NOVELTY_THRESHOLD: f64 = 0.7;
const LOW_COG_LOAD_THRESHOLD: f64 = 0.3;
const HIGH_COG_LOAD_THRESHOLD: f64 = 0.7;
const CONFIDENCE_ADJUST_RATE: f64 = 0.05;
const MODE_FLIP_INERTIA: f64 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncoderMode {
    Learn,
    Cog,
}

#[derive(Debug, Clone)]
pub struct AdaptiveVsaEncoder {
    mode: EncoderMode,
    learn_confidence: f64,
    cog_confidence: f64,
    switch_history: VecDeque<(u64, EncoderMode, f64)>,
    max_switch_history: usize,
    aligner: CrossModalAligner,
}

impl AdaptiveVsaEncoder {
    pub fn new() -> Self {
        Self {
            mode: EncoderMode::Learn,
            learn_confidence: 0.0,
            cog_confidence: 0.0,
            switch_history: VecDeque::with_capacity(MAX_SWITCH_HISTORY),
            max_switch_history: MAX_SWITCH_HISTORY,
            aligner: CrossModalAligner::new(VSA_DIM, 42),
        }
    }

    pub fn encode_adaptive(
        &mut self,
        text: &str,
        task_type: &str,
        cognitive_load: f64,
        novelty_score: f64,
    ) -> (Vec<u8>, EncoderMode) {
        let previous = self.mode;
        let repeated = matches!(
            task_type,
            "recall" | "retrieval" | "summarize" | "translate" | "paraphrase"
        );

        let target = if novelty_score > HIGH_NOVELTY_THRESHOLD {
            EncoderMode::Learn
        } else if cognitive_load < LOW_COG_LOAD_THRESHOLD {
            EncoderMode::Learn
        } else if repeated || cognitive_load > HIGH_COG_LOAD_THRESHOLD {
            EncoderMode::Cog
        } else {
            previous
        };

        let selected = match (previous, target) {
            (a, b) if a == b => a,
            _ => {
                let gap = self.learn_confidence - self.cog_confidence;
                if gap.abs() > MODE_FLIP_INERTIA {
                    target
                } else {
                    previous
                }
            }
        };

        self.mode = selected;

        if previous != self.mode {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.switch_history
                .push_back((now, self.mode, novelty_score));
            if self.switch_history.len() > self.max_switch_history {
                self.switch_history.pop_front();
            }
        }

        let vector = match self.mode {
            EncoderMode::Learn => self.aligner.text_to_vsa(text),
            EncoderMode::Cog => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                text.hash(&mut hasher);
                let seed = hasher.finish();
                QuantizedVSA::seeded_random(seed.wrapping_add(1), VSA_DIM)
            }
        };

        (vector, self.mode)
    }

    pub fn force_mode(&mut self, mode: EncoderMode) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if self.mode != mode {
            self.switch_history.push_back((now, mode, 0.5));
            if self.switch_history.len() > self.max_switch_history {
                self.switch_history.pop_front();
            }
        }
        self.mode = mode;
    }

    pub fn record_outcome(&mut self, success: bool) {
        let delta = if success {
            CONFIDENCE_ADJUST_RATE
        } else {
            -CONFIDENCE_ADJUST_RATE
        };
        match self.mode {
            EncoderMode::Learn => {
                self.learn_confidence = (self.learn_confidence + delta).clamp(0.0, 1.0);
            }
            EncoderMode::Cog => {
                self.cog_confidence = (self.cog_confidence + delta).clamp(0.0, 1.0);
            }
        }
    }

    pub fn switch_count(&self) -> usize {
        self.switch_history.len()
    }

    pub fn stats(&self) -> (f64, f64, usize, EncoderMode) {
        (
            self.learn_confidence,
            self.cog_confidence,
            self.switch_history.len(),
            self.mode,
        )
    }
}

impl Default for AdaptiveVsaEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoder() -> AdaptiveVsaEncoder {
        AdaptiveVsaEncoder::new()
    }

    #[test]
    fn test_default_mode_is_learn() {
        let e = encoder();
        assert_eq!(e.mode, EncoderMode::Learn);
    }

    #[test]
    fn test_high_novelty_triggers_learn() {
        let mut e = encoder();
        e.force_mode(EncoderMode::Cog);
        let (vec, mode) = e.encode_adaptive("novel pattern detected", "reasoning", 0.5, 0.85);
        assert_eq!(mode, EncoderMode::Learn);
        assert!(vec.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_high_cognitive_load_triggers_cog() {
        let mut e = encoder();
        let (vec, mode) = e.encode_adaptive("routine summarization task", "summarize", 0.8, 0.1);
        assert_eq!(mode, EncoderMode::Cog);
        assert!(vec.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_force_mode_override() {
        let mut e = encoder();
        assert_eq!(e.mode, EncoderMode::Learn);
        e.force_mode(EncoderMode::Cog);
        assert_eq!(e.mode, EncoderMode::Cog);
        e.force_mode(EncoderMode::Learn);
        assert_eq!(e.mode, EncoderMode::Learn);
    }

    #[test]
    fn test_record_outcome_updates_confidence() {
        let mut e = encoder();
        assert_eq!(e.learn_confidence, 0.0);
        assert_eq!(e.cog_confidence, 0.0);
        e.record_outcome(true);
        assert!((e.learn_confidence - 0.05).abs() < 1e-10);
        e.force_mode(EncoderMode::Cog);
        e.record_outcome(false);
        assert!((e.cog_confidence + 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_switch_count_tracks_switches() {
        let mut e = encoder();
        assert_eq!(e.switch_count(), 0);
        e.force_mode(EncoderMode::Cog);
        assert_eq!(e.switch_count(), 1);
        e.force_mode(EncoderMode::Learn);
        assert_eq!(e.switch_count(), 2);
        e.force_mode(EncoderMode::Learn);
        assert_eq!(e.switch_count(), 2);
    }

    #[test]
    fn test_stats_report() {
        let mut e = encoder();
        e.force_mode(EncoderMode::Cog);
        e.record_outcome(true);
        e.record_outcome(true);
        e.force_mode(EncoderMode::Learn);
        let (learn, cog, count, mode) = e.stats();
        assert_eq!(learn, 0.0);
        assert!((cog - 0.10).abs() < 1e-10);
        assert_eq!(count, 2);
        assert_eq!(mode, EncoderMode::Learn);
    }

    #[test]
    fn test_low_cog_load_triggers_learn() {
        let mut e = encoder();
        e.force_mode(EncoderMode::Cog);
        let (_, mode) = e.encode_adaptive("exploring with low load", "recall", 0.2, 0.5);
        assert_eq!(mode, EncoderMode::Learn);
    }

    #[test]
    fn test_confidence_clamping() {
        let mut e = encoder();
        for _ in 0..100 {
            e.record_outcome(true);
        }
        assert!((e.learn_confidence - 1.0).abs() < 1e-10);
        for _ in 0..100 {
            e.record_outcome(false);
        }
        assert!((e.learn_confidence - 0.0).abs() < 1e-10);
    }
}
