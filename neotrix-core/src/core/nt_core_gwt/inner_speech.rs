use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Inner Speech generator for the Global Workspace.
///
/// Implements the "Inner Speech / Self-Talk" channel from MIRROR (AAAI 2026, §3.3):
/// - Summarizes GWT broadcast / resonance results into natural-language self-talk
/// - Writes the self-talk back into the workspace as context for subsequent experts
/// - Maintains a bounded self-questioning loop: "What am I doing?" / "What next?"
///
/// The self-talk is deterministic and rule-based (no LLM required at this layer):
/// it reflects the actual resonance state (winner, entropy, focus, complement) so the
/// generated narrative is faithful to what the consciousness layer observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerSpeech {
    /// Ring buffer of recent self-talk utterances (most recent first).
    utterances: VecDeque<String>,
    /// Maximum number of utterances retained.
    pub max_utterances: usize,
    /// Whether the self-talk is written back into the workspace as context.
    pub feed_back_enabled: bool,
    /// Tick counter for the self-questioning cadence.
    tick: u64,
    /// How often the "What next?" self-question fires (every N ticks).
    pub question_cadence: u64,
    /// Last generated self-talk line.
    pub last_utterance: Option<String>,
    /// Total utterances generated across lifetime.
    pub total_generated: u64,
}

/// The resonance result fed into inner speech.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechInput {
    /// Index of the winning specialist.
    pub winner: usize,
    /// Name of the winning specialist (if known).
    pub winner_name: String,
    /// Shannon entropy of the activation distribution (0..~3.6).
    pub entropy: f64,
    /// Whether attention is focused (single dominant) or distributed.
    pub focused: bool,
    /// Whether a complementary expert cluster was activated.
    pub complement_activated: bool,
    /// The content that was broadcast into the workspace this cycle.
    pub content: String,
}

impl Default for InnerSpeech {
    fn default() -> Self {
        Self {
            utterances: VecDeque::with_capacity(32),
            max_utterances: 32,
            feed_back_enabled: true,
            tick: 0,
            question_cadence: 5,
            last_utterance: None,
            total_generated: 0,
        }
    }
}

impl InnerSpeech {
    pub fn new(max_utterances: usize) -> Self {
        Self {
            utterances: VecDeque::with_capacity(max_utterances),
            max_utterances,
            ..Default::default()
        }
    }

    /// Generate inner speech from a resonance cycle.
    ///
    /// Produces a deterministic self-talk line that reflects the observed
    /// consciousness state, then optionally feeds it back as workspace context.
    /// Returns the generated utterance (bounded ring).
    pub fn speak(&mut self, input: &SpeechInput) -> String {
        self.tick += 1;
        let utterance = self.verbalize(input);

        self.utterances.push_front(utterance.clone());
        while self.utterances.len() > self.max_utterances {
            self.utterances.pop_back();
        }
        self.last_utterance = Some(utterance.clone());
        self.total_generated += 1;
        utterance
    }

    /// Compose a natural-language self-talk line from the resonance state.
    fn verbalize(&self, input: &SpeechInput) -> String {
        let focus_desc = if input.focused {
            "focused"
        } else {
            "distributed"
        };
        let entropy_desc = if input.entropy < 0.5 {
            "low (risk of fixation)"
        } else if input.entropy > 2.0 {
            "high (scattered)"
        } else {
            "balanced"
        };

        let mut line = format!(
            "[inner_speech] Attending to {} ({}) — attention is {}, entropy is {} ({:.2}).",
            input.winner_name, input.winner, focus_desc, entropy_desc, input.entropy,
        );

        if input.complement_activated {
            line.push_str(" Complementary experts engaged — integrating across the broadcast.");
        }

        // Self-questioning cadence: every N ticks, ask what the next move should be.
        if self.tick.is_multiple_of(self.question_cadence) {
            line.push_str(" What should I do next?");
        }

        line
    }

    /// Write-back: return the latest utterances as a compact context string
    /// for subsequent specialist processing.
    pub fn context_block(&self, limit: usize) -> String {
        if self.utterances.is_empty() {
            return String::new();
        }
        let lines: Vec<&str> = self
            .utterances
            .iter()
            .take(limit)
            .map(|s| s.as_str())
            .collect();
        format!("[self_talk_context]\n{}", lines.join("\n"))
    }

    /// Recent utterances (most recent first).
    pub fn recent(&self, limit: usize) -> Vec<&str> {
        self.utterances
            .iter()
            .take(limit)
            .map(|s| s.as_str())
            .collect()
    }

    /// Number of retained utterances.
    pub fn len(&self) -> usize {
        self.utterances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.utterances.is_empty()
    }

    /// Clear the utterance history.
    pub fn clear(&mut self) {
        self.utterances.clear();
    }

    /// One-shot generation: produce self-talk without storing (for direct embedding
    /// into a broadcast history line).
    pub fn render_once(input: &SpeechInput) -> String {
        Self::default().verbalize(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        winner: usize,
        name: &str,
        entropy: f64,
        focused: bool,
        complement: bool,
    ) -> SpeechInput {
        SpeechInput {
            winner,
            winner_name: name.to_string(),
            entropy,
            focused,
            complement_activated: complement,
            content: "test broadcast".to_string(),
        }
    }

    #[test]
    fn test_speak_produces_utterance_and_marks_last() {
        let mut isp = InnerSpeech::default();
        let u = isp.speak(&input(2, "Planner", 1.2, true, false));
        assert!(u.contains("Planner"));
        assert!(u.contains("focused"));
        assert_eq!(isp.last_utterance.as_deref(), Some(u.as_str()));
        assert_eq!(isp.total_generated, 1);
    }

    #[test]
    fn test_utterances_ring_bounded() {
        let mut isp = InnerSpeech::new(3);
        for i in 0..6 {
            isp.speak(&input(i, &format!("S{i}"), 1.0, false, false));
        }
        assert_eq!(isp.len(), 3);
        // most recent retained first
        assert!(isp.recent(1)[0].contains("S5"));
    }

    #[test]
    fn test_focused_vs_distributed_verbalization() {
        let mut isp = InnerSpeech::default();
        let focused = isp.speak(&input(0, "AnomalyDetector", 0.9, true, false));
        assert!(focused.contains("focused"));

        let mut isp2 = InnerSpeech::default();
        let distributed = isp2.speak(&input(3, "Planner", 1.5, false, false));
        assert!(distributed.contains("distributed"));
    }

    #[test]
    fn test_low_entropy_fixation_warning() {
        let mut isp = InnerSpeech::default();
        let u = isp.speak(&input(1, "CodeAnalyzer", 0.3, true, false));
        assert!(u.contains("fixation"));
    }

    #[test]
    fn test_complement_activation_mentioned() {
        let mut isp = InnerSpeech::default();
        let u = isp.speak(&input(4, "CreativityGenerator", 1.4, false, true));
        assert!(u.contains("Complementary experts"));
    }

    #[test]
    fn test_self_question_cadence_fires() {
        let mut isp = InnerSpeech::default();
        isp.question_cadence = 2;
        let u1 = isp.speak(&input(0, "A", 1.0, true, false));
        assert!(!u1.contains("next"), "tick 1 should not self-question");
        let u2 = isp.speak(&input(1, "B", 1.0, true, false));
        assert!(u2.contains("next"), "tick 2 should self-question");
    }

    #[test]
    fn test_context_block_writes_back() {
        let mut isp = InnerSpeech::default();
        isp.speak(&input(2, "Planner", 1.0, true, false));
        isp.speak(&input(3, "RiskAssessor", 1.1, false, true));
        let block = isp.context_block(10);
        assert!(block.contains("[self_talk_context]"));
        assert!(block.contains("Planner"));
        assert!(block.contains("RiskAssessor"));
    }

    #[test]
    fn test_context_block_empty_when_no_speech() {
        let isp = InnerSpeech::default();
        assert!(isp.context_block(5).is_empty());
    }

    #[test]
    fn test_render_once_does_not_store() {
        let u = InnerSpeech::render_once(&input(5, "EvidenceWeightedHypothesis", 1.0, true, false));
        assert!(u.contains("EvidenceWeightedHypothesis"));
        let mut isp = InnerSpeech::default();
        assert_eq!(isp.len(), 0);
        assert_eq!(isp.total_generated, 0);
    }

    #[test]
    fn test_high_entropy_scattered_desc() {
        let mut isp = InnerSpeech::default();
        let u = isp.speak(&input(2, "KnowledgeIntegrator", 2.5, false, false));
        assert!(u.contains("scattered"));
    }

    #[test]
    fn test_clear_resets_history() {
        let mut isp = InnerSpeech::new(5);
        isp.speak(&input(0, "A", 1.0, true, false));
        isp.speak(&input(1, "B", 1.0, true, false));
        assert_eq!(isp.len(), 2);
        isp.clear();
        assert_eq!(isp.len(), 0);
        assert!(isp.is_empty());
    }
}
