use super::patterns::{match_patterns, IntentPattern};
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntentPhase {
    Scan,
    Reasoning,
    Resolved,
    Failed,
}

#[derive(Debug, Clone)]
pub struct IntentFrame {
    pub query: String,
    pub context: Vec<String>,
    pub gap_score: f64,
    pub probe_budget: u32,
    pub max_probes: u32,
    pub inferred_intent: Option<String>,
    pub confidence: f64,
    pub intent_vector: Option<Vec<u8>>,
    pub phase: IntentPhase,
}

#[derive(Debug, Clone)]
pub struct IntentResult {
    pub intent: IntentFrame,
    pub intent_vector: Vec<u8>,
    pub confidence: f64,
    pub probe_count: u32,
}

impl IntentFrame {
    pub fn new(query: &str, context: Vec<String>) -> Self {
        Self {
            query: query.to_string(),
            context,
            gap_score: 1.0,
            probe_budget: 3,
            max_probes: 3,
            inferred_intent: None,
            confidence: 0.0,
            intent_vector: None,
            phase: IntentPhase::Scan,
        }
    }

    pub fn with_max_probes(mut self, max: u32) -> Self {
        self.max_probes = max;
        self.probe_budget = max;
        self
    }

    pub fn scan_phase(&mut self) -> Vec<IntentPattern> {
        let patterns = match_patterns(&self.query);
        if patterns.is_empty() {
            self.gap_score = 1.0;
            self.phase = IntentPhase::Reasoning;
            return Vec::new();
        }
        let max_conf = patterns
            .iter()
            .map(|(_, c)| c)
            .fold(0.0f64, |a, &b| a.max(b));
        self.gap_score = (1.0 - max_conf).max(0.0);
        self.confidence = max_conf;
        if self.gap_score < 0.3 {
            if let Some((best_pat, _)) = patterns.first() {
                self.inferred_intent = Some(best_pat.label().to_string());
            }
            self.phase = IntentPhase::Resolved;
        } else {
            self.phase = IntentPhase::Reasoning;
        }
        let result: Vec<IntentPattern> = patterns.into_iter().map(|(p, _)| p).collect();
        result
    }

    pub fn reasoning_phase(&mut self, llm_result: Option<&str>) -> bool {
        if self.probe_budget == 0 {
            self.phase = IntentPhase::Failed;
            return false;
        }
        self.probe_budget -= 1;
        self.phase = IntentPhase::Reasoning;
        match llm_result {
            Some(intent) if !intent.trim().is_empty() => {
                let resolved = intent.trim().to_string();
                self.inferred_intent = Some(resolved);
                self.gap_score = 0.2;
                self.confidence = 0.8;
                self.phase = IntentPhase::Resolved;
                true
            }
            _ => {
                if self.probe_budget == 0 {
                    self.phase = IntentPhase::Failed;
                }
                false
            }
        }
    }

    pub fn resolve(&mut self, intent: &str, confidence: f64) {
        self.inferred_intent = Some(intent.to_string());
        self.confidence = confidence.clamp(0.0, 1.0);
        self.gap_score = (1.0 - self.confidence).max(0.0);
        self.phase = IntentPhase::Resolved;
        let aligner = CrossModalAligner::new(VSA_DIM, 42);
        self.intent_vector = Some(aligner.text_to_vsa(intent));
    }

    pub fn should_probe(&self) -> bool {
        self.gap_score > 0.3 && self.probe_budget > 0
    }

    pub fn intent_summary(&self) -> String {
        match &self.inferred_intent {
            Some(intent) => format!(
                "Intent: {} | confidence: {:.2} | gap: {:.2} | phase: {:?}",
                intent, self.confidence, self.gap_score, self.phase
            ),
            None => format!(
                "Intent: unresolved | gap: {:.2} | probes remaining: {} | phase: {:?}",
                self.gap_score, self.probe_budget, self.phase
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_frame_initial_state() {
        let frame = IntentFrame::new("hello", vec![]);
        assert_eq!(frame.phase, IntentPhase::Scan);
        assert!((frame.gap_score - 1.0).abs() < 1e-9);
        assert_eq!(frame.probe_budget, 3);
        assert!(frame.inferred_intent.is_none());
    }

    #[test]
    fn test_with_max_probes_overrides_default() {
        let frame = IntentFrame::new("test", vec![]).with_max_probes(5);
        assert_eq!(frame.max_probes, 5);
        assert_eq!(frame.probe_budget, 5);
    }

    #[test]
    fn test_scan_phase_matches_direct_command() {
        let mut frame = IntentFrame::new("write a sorting function", vec![]);
        let patterns = frame.scan_phase();
        assert!(!patterns.is_empty());
        assert_eq!(frame.phase, IntentPhase::Resolved);
        assert!(frame.gap_score < 0.3);
        assert!(frame.confidence > 0.0);
    }

    #[test]
    fn test_scan_phase_ambiguous_query_stays_in_reasoning() {
        let mut frame = IntentFrame::new("the sky is blue", vec![]);
        let patterns = frame.scan_phase();
        assert!(patterns.is_empty());
        assert_eq!(frame.phase, IntentPhase::Reasoning);
        assert!((frame.gap_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_reasoning_phase_with_result_resolves_intent() {
        let mut frame = IntentFrame::new("do something", vec![]);
        frame.scan_phase();
        if frame.phase == IntentPhase::Reasoning {
            let resolved = frame.reasoning_phase(Some("optimize the query pipeline"));
            assert!(resolved);
            assert_eq!(frame.phase, IntentPhase::Resolved);
            assert!(frame.inferred_intent.is_some());
        }
    }

    #[test]
    fn test_reasoning_phase_exhausts_probes() {
        let mut frame = IntentFrame::new("random text", vec![]).with_max_probes(2);
        frame.scan_phase();
        assert_eq!(frame.phase, IntentPhase::Reasoning);
        let r1 = frame.reasoning_phase(None);
        assert!(!r1);
        assert_eq!(frame.probe_budget, 1);
        let r2 = frame.reasoning_phase(None);
        assert!(!r2);
        assert_eq!(frame.probe_budget, 0);
        assert_eq!(frame.phase, IntentPhase::Failed);
    }

    #[test]
    fn test_resolve_generates_vsa_vector() {
        let mut frame = IntentFrame::new("do something", vec![]);
        frame.resolve("implement feature X", 0.9);
        assert_eq!(frame.phase, IntentPhase::Resolved);
        assert!(frame.intent_vector.is_some());
        assert_eq!(frame.intent_vector.as_ref().unwrap().len(), VSA_DIM);
    }

    #[test]
    fn test_should_probe_returns_true_when_ambiguous() {
        let frame = IntentFrame::new("hmm", vec![]);
        assert!(frame.should_probe());
    }

    #[test]
    fn test_should_probe_returns_false_when_exhausted() {
        let frame = IntentFrame::new("hmm", vec![]).with_max_probes(0);
        assert!(!frame.should_probe());
    }

    #[test]
    fn test_intent_summary_unresolved() {
        let frame = IntentFrame::new("hello", vec![]);
        let summary = frame.intent_summary();
        assert!(summary.contains("unresolved"));
    }

    #[test]
    fn test_intent_summary_resolved() {
        let mut frame = IntentFrame::new("hello", vec![]);
        frame.resolve("greeting", 0.95);
        let summary = frame.intent_summary();
        assert!(summary.contains("greeting"));
    }
}
