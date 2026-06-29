//! # Attention Schema — Salience Detection and Focus
//!
//! Implements the Attention Schema Theory:
//! the system models its own attention as a simplified control model.
//! Tracks what's salient, where focus is directed, and attention shifts.

#[derive(Debug, Clone)]
pub struct AttentionSchema {
    /// Current focus coordinates (E8 hex, GWT specialist, salience)
    pub focus_hex: u8,
    pub focus_specialist: usize,
    pub focus_salience: f64,
    pub attention_shifts: u64,
    dwell_cycles: u64,
    min_dwell: u64,
}

impl AttentionSchema {
    pub fn new() -> Self {
        Self {
            focus_hex: 0b101010,
            focus_specialist: 9,
            focus_salience: 0.5,
            attention_shifts: 0,
            dwell_cycles: 0,
            min_dwell: 3,
        }
    }

    /// Check if a new salient stimulus is worth attending to
    pub fn should_shift(&self, candidate_salience: f64) -> bool {
        candidate_salience > self.focus_salience + 0.15 && self.dwell_cycles >= self.min_dwell
    }

    /// Shift attention to a new focus
    pub fn shift_to(&mut self, hex: u8, specialist: usize, salience: f64) {
        self.focus_hex = hex;
        self.focus_specialist = specialist;
        self.focus_salience = salience;
        self.attention_shifts += 1;
        self.dwell_cycles = 0;
    }

    pub fn tick(&mut self) {
        self.dwell_cycles += 1;
        // Gradual salience decay
        self.focus_salience *= 0.995;
    }

    pub fn report(&self) -> String {
        format!(
            "attn:hex_{:06b}_spec_{}_sal_{:.3}_shifts_{}_dwell_{}",
            self.focus_hex, self.focus_specialist, self.focus_salience,
            self.attention_shifts, self.dwell_cycles
        )
    }
}

impl Default for AttentionSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Salience detector — evaluates stimuli for attention-worthiness
#[derive(Debug, Clone)]
pub struct SalienceDetector {
    /// Weights for salience factors
    pub novelty_weight: f64,
    pub urgency_weight: f64,
    pub relevance_weight: f64,
}

impl Default for SalienceDetector {
    fn default() -> Self {
        Self {
            novelty_weight: 0.3,
            urgency_weight: 0.4,
            relevance_weight: 0.3,
        }
    }
}

impl SalienceDetector {
    pub fn evaluate(&self, novelty: f64, urgency: f64, relevance: f64) -> f64 {
        (novelty * self.novelty_weight
            + urgency * self.urgency_weight
            + relevance * self.relevance_weight)
            .clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_schema_default() {
        let attn = AttentionSchema::new();
        assert_eq!(attn.focus_specialist, 9);
    }

    #[test]
    fn test_attention_shift() {
        let mut attn = AttentionSchema::new();
        attn.shift_to(0, 0, 0.9);
        assert_eq!(attn.focus_hex, 0);
        assert_eq!(attn.attention_shifts, 1);
    }

    #[test]
    fn test_dwell_prevents_rapid_shift() {
        let attn = AttentionSchema::new();
        assert!(!attn.should_shift(0.8)); // dwell < min_dwell
    }

    #[test]
    fn test_salience_detector() {
        let d = SalienceDetector::default();
        let score = d.evaluate(0.5, 0.8, 0.3);
        assert!(score > 0.4 && score < 0.6);
    }

    #[test]
    fn test_attention_tick_decay() {
        let mut attn = AttentionSchema::new();
        let before = attn.focus_salience;
        attn.tick();
        assert!(attn.focus_salience < before);
    }
}
