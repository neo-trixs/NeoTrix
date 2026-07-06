use super::intent_frame::IntentFrame;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct IntentBuffer {
    buffer: VecDeque<IntentFrame>,
    max_size: usize,
}

impl IntentBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size.max(1)),
            max_size: max_size.max(1),
        }
    }

    pub fn push(&mut self, frame: IntentFrame) {
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(frame);
    }

    pub fn recent(&self, n: usize) -> Vec<&IntentFrame> {
        self.buffer.iter().rev().take(n).collect()
    }

    pub fn current_intent(&self) -> Option<&IntentFrame> {
        self.buffer.back()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn intent_shift_score(&self) -> f64 {
        if self.buffer.len() < 2 {
            return 0.0;
        }
        let current = match self.buffer.back().and_then(|f| f.intent_vector.as_ref()) {
            Some(v) => v,
            None => return 0.0,
        };
        let previous = match self
            .buffer
            .iter()
            .rev()
            .nth(1)
            .and_then(|f| f.intent_vector.as_ref())
        {
            Some(v) => v,
            None => return 0.0,
        };
        1.0 - crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::similarity(current, previous)
    }

    pub fn most_common_intent(&self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for frame in &self.buffer {
            if let Some(ref intent) = frame.inferred_intent {
                *counts.entry(intent.clone()).or_insert(0) += 1;
            }
        }
        counts.into_iter().max_by_key(|&(_, c)| c).map(|(k, _)| k)
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for IntentBuffer {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_frame(intent: &str, vector: Vec<u8>) -> IntentFrame {
        IntentFrame {
            query: intent.to_string(),
            context: vec![],
            gap_score: 0.1,
            probe_budget: 3,
            max_probes: 3,
            inferred_intent: Some(intent.to_string()),
            confidence: 0.9,
            intent_vector: Some(vector),
            phase: crate::core::nt_core_aura::intent_frame::IntentPhase::Resolved,
        }
    }

    fn fake_vector(value: u8) -> Vec<u8> {
        vec![value; crate::core::nt_core_hcube::vsa_quantized::VSA_DIM]
    }

    #[test]
    fn test_push_and_current_intent() {
        let mut buf = IntentBuffer::default();
        assert!(buf.is_empty());
        let frame = sample_frame("write code", fake_vector(1));
        buf.push(frame);
        assert_eq!(buf.len(), 1);
        assert!(buf.current_intent().is_some());
    }

    #[test]
    fn test_buffer_overflow_evicts_oldest() {
        let mut buf = IntentBuffer::new(3);
        for i in 0..5 {
            buf.push(sample_frame(&format!("intent {}", i), fake_vector(i as u8)));
        }
        assert_eq!(buf.len(), 3);
        let recent = buf.recent(10);
        assert_eq!(recent.len(), 3);
        assert!(recent[0].query.contains("intent 4"));
        assert!(recent[2].query.contains("intent 2"));
    }

    #[test]
    fn test_empty_buffer_intent_shift_score_zero() {
        let buf: IntentBuffer = IntentBuffer::default();
        assert!((buf.intent_shift_score() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_single_frame_intent_shift_score_zero() {
        let mut buf = IntentBuffer::default();
        buf.push(sample_frame("hello", fake_vector(1)));
        assert!((buf.intent_shift_score() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_intent_shift_score_detects_change() {
        let mut buf = IntentBuffer::new(5);
        let v1 = fake_vector(0);
        let v2 = fake_vector(1);
        buf.push(sample_frame("first", v1));
        buf.push(sample_frame("second", v2));
        let score = buf.intent_shift_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_most_common_intent() {
        let mut buf = IntentBuffer::new(10);
        buf.push(sample_frame("write", fake_vector(1)));
        buf.push(sample_frame("write", fake_vector(1)));
        buf.push(sample_frame("read", fake_vector(0)));
        let common = buf.most_common_intent();
        assert_eq!(common, Some("write".to_string()));
    }

    #[test]
    fn test_most_common_intent_empty() {
        let buf: IntentBuffer = IntentBuffer::default();
        assert_eq!(buf.most_common_intent(), None);
    }

    #[test]
    fn test_recent_returns_newest_first() {
        let mut buf = IntentBuffer::new(10);
        buf.push(sample_frame("first", fake_vector(0)));
        buf.push(sample_frame("second", fake_vector(1)));
        buf.push(sample_frame("third", fake_vector(2)));
        let recent = buf.recent(2);
        assert_eq!(recent.len(), 2);
        assert!(recent[0].query.contains("third"));
        assert!(recent[1].query.contains("second"));
    }

    #[test]
    fn test_clear_empties_buffer() {
        let mut buf = IntentBuffer::new(5);
        buf.push(sample_frame("test", fake_vector(0)));
        buf.push(sample_frame("test2", fake_vector(1)));
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }
}
