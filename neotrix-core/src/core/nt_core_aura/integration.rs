use super::intent_frame::IntentFrame;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

pub trait IntentAware {
    fn infer_intent(&mut self, input: &str) -> IntentFrame;

    fn current_intent(&self) -> Option<&IntentFrame>;

    fn intent_alignment(&self, output: &[u8]) -> f64 {
        match self.current_intent() {
            Some(frame) => match frame.intent_vector.as_ref() {
                Some(iv) => QuantizedVSA::similarity(iv, output),
                None => 0.0,
            },
            None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    struct MockIntentAware {
        frame: Option<IntentFrame>,
    }

    impl MockIntentAware {
        fn new() -> Self {
            Self { frame: None }
        }
    }

    impl IntentAware for MockIntentAware {
        fn infer_intent(&mut self, input: &str) -> IntentFrame {
            let mut frame = IntentFrame::new(input, vec![]);
            frame.scan_phase();
            if frame.phase == crate::core::nt_core_aura::intent_frame::IntentPhase::Resolved {
                if let Some(ref intent) = frame.inferred_intent.clone() {
                    frame.resolve(intent, frame.confidence);
                }
            }
            self.frame = Some(frame.clone());
            frame
        }

        fn current_intent(&self) -> Option<&IntentFrame> {
            self.frame.as_ref()
        }
    }

    #[test]
    fn test_infer_intent_direct_command() {
        let mut aware = MockIntentAware::new();
        let result = aware.infer_intent("write a test");
        assert!(result.intent_vector.is_some());
        assert_eq!(result.intent_vector.as_ref().unwrap().len(), VSA_DIM);
    }

    #[test]
    fn test_infer_intent_ambiguous_reasoning() {
        let mut aware = MockIntentAware::new();
        let result = aware.infer_intent("something unclear here");
        assert_eq!(
            result.phase,
            crate::core::nt_core_aura::intent_frame::IntentPhase::Reasoning
        );
    }

    #[test]
    fn test_current_intent_returns_some_after_infer() {
        let mut aware = MockIntentAware::new();
        aware.infer_intent("hello");
        assert!(aware.current_intent().is_some());
    }

    #[test]
    fn test_current_intent_returns_none_initially() {
        let aware = MockIntentAware::new();
        assert!(aware.current_intent().is_none());
    }

    #[test]
    fn test_intent_alignment_self_similarity_is_one() {
        let mut aware = MockIntentAware::new();
        let frame = aware.infer_intent("write code");
        if let Some(ref vec) = frame.intent_vector {
            let alignment = aware.intent_alignment(vec);
            assert!((alignment - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_intent_alignment_zero_when_no_intent() {
        let aware = MockIntentAware::new();
        let output = vec![1u8; VSA_DIM];
        let alignment = aware.intent_alignment(&output);
        assert!((alignment - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_intent_alignment_different_vector_lower() {
        let mut aware = MockIntentAware::new();
        aware.infer_intent("write code");
        let opposite = vec![0u8; VSA_DIM]; // all zeros, likely dissimilar
        let alignment = aware.intent_alignment(&opposite);
        assert!(alignment < 1.0);
    }
}
