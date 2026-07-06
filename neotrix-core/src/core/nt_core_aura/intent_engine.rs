use super::intent_buffer::IntentBuffer;
use super::intent_frame::{IntentFrame, IntentPhase};

#[derive(Debug, Clone)]
pub struct IntentEngine {
    buffer: IntentBuffer,
    current_frame: Option<IntentFrame>,
    total_intents: u64,
    ambiguous_count: u64,
    ambiguity_rate: f64,
}

impl IntentEngine {
    pub fn new() -> Self {
        Self {
            buffer: IntentBuffer::default(),
            current_frame: None,
            total_intents: 0,
            ambiguous_count: 0,
            ambiguity_rate: 0.0,
        }
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer = IntentBuffer::new(size);
        self
    }

    pub fn process_input(&mut self, input: &str, context: Vec<String>) -> IntentFrame {
        let mut frame = IntentFrame::new(input, context);
        frame.scan_phase();

        if frame.phase == IntentPhase::Reasoning {
            frame.reasoning_phase(None);
        }

        if frame.phase == IntentPhase::Resolved && frame.intent_vector.is_none() {
            let intent = frame
                .inferred_intent
                .clone()
                .unwrap_or_else(|| "resolved".to_string());
            let conf = frame.confidence.max(0.5);
            frame.resolve(&intent, conf);
        }

        self.total_intents += 1;
        if frame.gap_score > 0.5 {
            self.ambiguous_count += 1;
        }
        self.ambiguity_rate = if self.total_intents > 0 {
            self.ambiguous_count as f64 / self.total_intents as f64
        } else {
            0.0
        };

        let result = frame.clone();
        self.buffer.push(frame);
        self.current_frame = Some(result.clone());
        result
    }

    pub fn current_intent_vector(&self) -> Option<Vec<u8>> {
        self.current_frame
            .as_ref()
            .and_then(|f| f.intent_vector.clone())
    }

    pub fn intent_confidence(&self) -> f64 {
        self.current_frame
            .as_ref()
            .map(|f| f.confidence)
            .unwrap_or(0.0)
    }

    pub fn intent_history(&self, n: usize) -> Vec<&IntentFrame> {
        self.buffer.recent(n)
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.current_frame = None;
        self.total_intents = 0;
        self.ambiguous_count = 0;
        self.ambiguity_rate = 0.0;
    }

    pub fn buffer(&self) -> &IntentBuffer {
        &self.buffer
    }

    pub fn ambiguity_rate(&self) -> f64 {
        self.ambiguity_rate
    }

    pub fn total_intents(&self) -> u64 {
        self.total_intents
    }
}

impl Default for IntentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    #[test]
    fn test_process_input_resolves_direct_command() {
        let mut engine = IntentEngine::new();
        let result = engine.process_input("write a parser", vec![]);
        assert_eq!(result.phase, IntentPhase::Resolved);
        assert!(result.confidence > 0.0);
        assert!(result.intent_vector.is_some());
    }

    #[test]
    fn test_process_input_ambiguous_stays_in_reasoning() {
        let mut engine = IntentEngine::new();
        let result = engine.process_input("the sky is blue", vec![]);
        assert_eq!(result.phase, IntentPhase::Reasoning);
    }

    #[test]
    fn test_current_intent_vector_returns_some_after_process() {
        let mut engine = IntentEngine::new();
        engine.process_input("build the module", vec![]);
        let vector = engine.current_intent_vector();
        assert!(vector.is_some());
        assert_eq!(vector.unwrap().len(), VSA_DIM);
    }

    #[test]
    fn test_intent_confidence_returns_value() {
        let mut engine = IntentEngine::new();
        engine.process_input("fix the bug", vec![]);
        let conf = engine.intent_confidence();
        assert!(conf > 0.0);
    }

    #[test]
    fn test_intent_history_returns_recent_frames() {
        let mut engine = IntentEngine::new();
        engine.process_input("first command", vec![]);
        engine.process_input("second command", vec![]);
        let history = engine.intent_history(5);
        assert_eq!(history.len(), 2);
        assert!(history[0].query.contains("second"));
    }

    #[test]
    fn test_reset_clears_all_state() {
        let mut engine = IntentEngine::new();
        engine.process_input("hello", vec![]);
        engine.reset();
        assert!(engine.current_intent_vector().is_none());
        assert!((engine.intent_confidence() - 0.0).abs() < 1e-9);
        assert_eq!(engine.total_intents(), 0);
        assert!((engine.ambiguity_rate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_ambiguity_rate_increases_with_ambiguous_inputs() {
        let mut engine = IntentEngine::new();
        engine.process_input("build something", vec![]);
        assert!(engine.ambiguity_rate() < 0.5);
        engine.process_input("hmm what", vec![]);
        engine.process_input("random noise", vec![]);
        assert!(engine.ambiguity_rate() > 0.0);
    }

    #[test]
    fn test_with_buffer_size_uses_custom_size() {
        let engine = IntentEngine::new().with_buffer_size(3);
        let buf = engine.buffer();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_process_input_with_context() {
        let mut engine = IntentEngine::new();
        let ctx = vec!["previous discussion".to_string()];
        let result = engine.process_input("explain this", ctx);
        assert!(!result.context.is_empty());
    }
}
