use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Voice in the internal dialogue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InternalVoice {
    /// The reasoner — proposes solutions
    Reasoner,
    /// The critic — finds flaws in proposals
    Critic,
    /// The skeptic — questions assumptions
    Skeptic,
    /// The optimist — finds opportunities
    Optimist,
    /// The pragmatist — evaluates feasibility
    Pragmatist,
    /// The ethicist — checks value alignment
    Ethicist,
    /// The historian — recalls similar past experiences
    Historian,
}

impl InternalVoice {
    pub fn all() -> [InternalVoice; 7] {
        [
            InternalVoice::Reasoner,
            InternalVoice::Critic,
            InternalVoice::Skeptic,
            InternalVoice::Optimist,
            InternalVoice::Pragmatist,
            InternalVoice::Ethicist,
            InternalVoice::Historian,
        ]
    }
}

/// An utterance in the internal monologue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utterance {
    pub id: u32,
    pub speaker: InternalVoice,
    pub target: Option<InternalVoice>,
    pub content: String,
    pub confidence: f64,
    pub references: Vec<u32>,
    pub timestamp: String,
}

/// Dialogue phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialoguePhase {
    /// Problem is being framed
    Framing,
    /// Solutions being proposed
    Ideation,
    /// Proposals being criticized
    Critique,
    /// Synthesis of best elements
    Synthesis,
    /// Final decision
    Decision,
}

impl DialoguePhase {
    pub fn next(&self) -> Option<DialoguePhase> {
        match self {
            DialoguePhase::Framing => Some(DialoguePhase::Ideation),
            DialoguePhase::Ideation => Some(DialoguePhase::Critique),
            DialoguePhase::Critique => Some(DialoguePhase::Synthesis),
            DialoguePhase::Synthesis => Some(DialoguePhase::Decision),
            DialoguePhase::Decision => None,
        }
    }

    pub fn all() -> [DialoguePhase; 5] {
        [
            DialoguePhase::Framing,
            DialoguePhase::Ideation,
            DialoguePhase::Critique,
            DialoguePhase::Synthesis,
            DialoguePhase::Decision,
        ]
    }
}

/// Result of an internal dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueResult {
    pub dialogue_id: u32,
    pub phase: DialoguePhase,
    pub utterances: Vec<Utterance>,
    pub consensus: Option<String>,
    pub consensus_confidence: f64,
    pub dissenting_voices: Vec<InternalVoice>,
    pub duration_ms: u64,
    pub start_time: String,
}

/// Summary of a completed dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueSummary {
    pub dialogue_id: u32,
    pub phases_completed: usize,
    pub total_utterances: u32,
    pub voices_participated: Vec<InternalVoice>,
    pub consensus_reached: bool,
    pub duration_ms: u64,
    pub phase_sequence: Vec<DialoguePhase>,
}

/// Personality/behavior profile for each voice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    pub name: &'static str,
    pub description: &'static str,
    pub typical_phrases: Vec<&'static str>,
    pub bias: &'static str,
    pub weight: f64,
}

/// Static voice profile registry
pub struct VoiceProfiles;

impl VoiceProfiles {
    pub fn profile(voice: InternalVoice) -> VoiceProfile {
        match voice {
            InternalVoice::Reasoner => VoiceProfile {
                name: "Reasoner",
                description: "Proposes structured solutions and approaches",
                typical_phrases: vec![
                    "We could approach this by",
                    "A possible solution is",
                    "The logical next step would be",
                ],
                bias: "tends to prefer elegant solutions over practical ones",
                weight: 1.0,
            },
            InternalVoice::Critic => VoiceProfile {
                name: "Critic",
                description: "Identifies flaws, edge cases, and failure modes",
                typical_phrases: vec![
                    "This fails when",
                    "What about the edge case where",
                    "The assumption that X is problematic because",
                ],
                bias: "tends to be overly negative, missing viable solutions",
                weight: 0.9,
            },
            InternalVoice::Skeptic => VoiceProfile {
                name: "Skeptic",
                description: "Questions premises and underlying assumptions",
                typical_phrases: vec![
                    "Are we sure that",
                    "What evidence supports",
                    "Is it really true that",
                ],
                bias: "can cause analysis paralysis by questioning everything",
                weight: 0.8,
            },
            InternalVoice::Optimist => VoiceProfile {
                name: "Optimist",
                description: "Finds opportunities and positive aspects",
                typical_phrases: vec![
                    "This could also enable",
                    "A positive side effect is",
                    "We can turn this into an opportunity by",
                ],
                bias: "tends to underestimate risks and costs",
                weight: 0.7,
            },
            InternalVoice::Pragmatist => VoiceProfile {
                name: "Pragmatist",
                description: "Evaluates feasibility, cost, and implementation effort",
                typical_phrases: vec![
                    "In practice, this means",
                    "The implementation cost would be",
                    "A more practical approach would be",
                ],
                bias: "tends to reject novel ideas due to implementation friction",
                weight: 1.0,
            },
            InternalVoice::Ethicist => VoiceProfile {
                name: "Ethicist",
                description: "Checks value alignment and ethical implications",
                typical_phrases: vec![
                    "Is this aligned with our values",
                    "The ethical implication is",
                    "We should consider the impact on",
                ],
                bias: "can be overly cautious, blocking valuable but ambiguous actions",
                weight: 0.8,
            },
            InternalVoice::Historian => VoiceProfile {
                name: "Historian",
                description: "Recalls similar past experiences and outcomes",
                typical_phrases: vec![
                    "This resembles the situation where",
                    "Last time we tried this,",
                    "Historical patterns suggest",
                ],
                bias: "tends to assume past conditions apply to novel situations",
                weight: 0.7,
            },
        }
    }
}

/// Output from the synthesis engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisOutput {
    pub consensus: String,
    pub confidence: f64,
    pub remaining_tensions: Vec<String>,
    pub recommended_action: String,
}

/// Engine that synthesizes multiple utterances into consensus
pub struct SynthesisEngine;

impl SynthesisEngine {
    /// Synthesize multiple utterances into consensus
    pub fn synthesize(utterances: &[Utterance]) -> SynthesisOutput {
        if utterances.is_empty() {
            return SynthesisOutput {
                consensus: "No utterances to synthesize.".into(),
                confidence: 0.0,
                remaining_tensions: vec![],
                recommended_action: "Gather more input.".into(),
            };
        }

        let mut weighted_points: Vec<(String, f64)> = Vec::new();
        let mut tensions: Vec<String> = Vec::new();
        let mut action_keywords: Vec<&str> = Vec::new();

        for u in utterances {
            let profile = VoiceProfiles::profile(u.speaker);
            let effective_weight = u.confidence * profile.weight;
            let sentence: &str = &u.content;

            let point = sentence.split('.').next().unwrap_or(sentence);
            weighted_points.push((point.to_string(), effective_weight));

            if u.speaker == InternalVoice::Critic || u.speaker == InternalVoice::Skeptic {
                tensions.push(format!("{} raises: {}", profile.name, sentence));
            }

            if sentence.contains("approach") || sentence.contains("solution") {
                action_keywords.push("evaluate proposed approach");
            }
        }

        weighted_points.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut consensus = String::new();
        let top_n = weighted_points.len().min(3);
        for i in 0..top_n {
            if i > 0 {
                consensus.push_str("; ");
            }
            consensus.push_str(&weighted_points[i].0);
        }

        let avg_confidence: f64 = weighted_points.iter().map(|(_, w)| w).sum::<f64>()
            / weighted_points.len().max(1) as f64;

        let recommended_action = if action_keywords.is_empty() {
            "Synthesize and decide.".to_string()
        } else {
            action_keywords.join(", ")
        };

        SynthesisOutput {
            consensus,
            confidence: avg_confidence,
            remaining_tensions: tensions,
            recommended_action,
        }
    }

    /// Find common ground between opposing views
    pub fn find_common_ground(view_a: &str, view_b: &str) -> Option<String> {
        let words_a: Vec<&str> = view_a.split_whitespace().collect();
        let words_b: Vec<&str> = view_b.split_whitespace().collect();

        let common: Vec<&str> = words_a
            .iter()
            .filter(|w| words_b.contains(w) && w.len() > 3)
            .copied()
            .collect();

        if common.is_empty() {
            return None;
        }

        Some(format!("Both views reference: {}", common.join(", ")))
    }

    /// Evaluate which proposal best satisfies all concerns
    pub fn best_proposal(proposals: &[(&str, Vec<String>)]) -> usize {
        if proposals.is_empty() {
            return 0;
        }

        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;

        for (i, (proposal, concerns)) in proposals.iter().enumerate() {
            let length_score = proposal.len() as f64 * 0.1;
            let concern_coverage = concerns.len() as f64 * 2.0;
            let score = length_score + concern_coverage;
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        best_idx
    }
}

/// Inner Monologue / Self-Talk System
/// Provides structured internal dialogue where subsystems deliberate before committing to action.
pub struct InnerMonologueSystem {
    pub current_dialogue: Option<DialogueResult>,
    pub dialogue_history: Vec<DialogueResult>,
    pub max_utterances_per_phase: u32,
    pub required_voices_for_consensus: u32,
    pub active_voices: Vec<InternalVoice>,
    next_utterance_id: AtomicU32,
    next_dialogue_id: AtomicU32,
}

impl Default for InnerMonologueSystem {
    fn default() -> Self {
        Self {
            current_dialogue: None,
            dialogue_history: Vec::new(),
            max_utterances_per_phase: 10,
            required_voices_for_consensus: 4,
            active_voices: InternalVoice::all().to_vec(),
            next_utterance_id: AtomicU32::new(1),
            next_dialogue_id: AtomicU32::new(1),
        }
    }
}

impl InnerMonologueSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new internal dialogue on a topic. Returns the dialogue ID.
    pub fn start_dialogue(&mut self, problem: &str) -> u32 {
        if let Some(dialogue) = self.current_dialogue.take() {
            self.dialogue_history.push(dialogue);
        }

        let dialogue_id = self.next_dialogue_id.fetch_add(1, Ordering::Relaxed);

        let first_utterance = Utterance {
            id: self.next_utterance_id.fetch_add(1, Ordering::Relaxed),
            speaker: InternalVoice::Reasoner,
            target: None,
            content: format!("Problem to address: {}", problem),
            confidence: 1.0,
            references: vec![],
            timestamp: Utc::now().to_rfc3339(),
        };

        self.current_dialogue = Some(DialogueResult {
            dialogue_id,
            phase: DialoguePhase::Framing,
            utterances: vec![first_utterance],
            consensus: None,
            consensus_confidence: 0.0,
            dissenting_voices: vec![],
            duration_ms: 0,
            start_time: Utc::now().to_rfc3339(),
        });

        dialogue_id
    }

    /// Submit an utterance from a voice. Returns the utterance ID.
    pub fn speak(
        &mut self,
        speaker: InternalVoice,
        content: &str,
        confidence: f64,
        references: &[u32],
    ) -> u32 {
        let dialogue = self
            .current_dialogue
            .as_mut()
            .expect("No active dialogue. Call start_dialogue() first.");

        let id = self.next_utterance_id.fetch_add(1, Ordering::Relaxed);

        let utterance = Utterance {
            id,
            speaker,
            target: None,
            content: content.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            references: references.to_vec(),
            timestamp: Utc::now().to_rfc3339(),
        };

        dialogue.utterances.push(utterance);
        id
    }

    /// Generate a reasoner utterance (proposes approach)
    pub fn reasoner_says(&mut self, problem: &str, context: &str) -> u32 {
        let content = format!(
            "Proposed approach for '{}' using context '{}': \
             analyze requirements, design solution, implement incrementally, \
             verify correctness, and integrate. Key steps include decomposition \
             into independent sub-problems and iterative refinement.",
            problem, context
        );
        self.speak(InternalVoice::Reasoner, &content, 0.85, &[])
    }

    /// Generate a critic utterance (finds flaws)
    pub fn critic_says(&mut self, proposals: &[&str]) -> u32 {
        let max_len = proposals.len().min(3);
        let mut content = String::from("Critical analysis of proposals:");
        for i in 0..max_len {
            content.push_str(&format!(
                " Proposal {} has potential edge cases and failure modes;",
                i + 1
            ));
        }
        content.push_str(" Need to verify assumptions under stress conditions.");

        let ids: Vec<u32> = self
            .current_dialogue
            .as_ref()
            .map(|d| {
                d.utterances
                    .iter()
                    .rev()
                    .take(proposals.len())
                    .map(|u| u.id)
                    .collect()
            })
            .unwrap_or_default();

        self.speak(InternalVoice::Critic, &content, 0.75, &ids)
    }

    /// Generate a skeptic utterance (questions assumptions)
    pub fn skeptic_says(&mut self, assumptions: &[&str]) -> u32 {
        let mut content = String::from("Questioning underlying assumptions:");
        for a in assumptions.iter().take(3) {
            content.push_str(&format!(
                " Is '{}' actually valid? What evidence supports it?",
                a
            ));
        }
        content.push_str(" We must verify each premise before proceeding.");

        self.speak(InternalVoice::Skeptic, &content, 0.70, &[])
    }

    /// Generate an optimist utterance (finds opportunities)
    pub fn optimist_says(&mut self, critic_points: &[&str]) -> u32 {
        let mut content = String::from("Opportunities within the criticism:");
        for c in critic_points.iter().take(3) {
            content.push_str(&format!(
                " '{}' can be reframed as a chance to improve robustness;",
                c
            ));
        }
        content.push_str(" Every identified flaw is an opportunity for refinement.");

        self.speak(InternalVoice::Optimist, &content, 0.65, &[])
    }

    /// Generate a pragmatist utterance (evaluates feasibility)
    pub fn pragmatist_says(&mut self) -> u32 {
        let content = concat!(
            "Feasibility assessment: The proposed approach requires ",
            "resource allocation, implementation time, and maintenance cost. ",
            "Consider whether simpler alternatives exist that achieve the same goal ",
            "with less complexity."
        );
        self.speak(InternalVoice::Pragmatist, content, 0.90, &[])
    }

    /// Generate an ethicist utterance (checks value alignment)
    pub fn ethicist_says(&mut self) -> u32 {
        let content = concat!(
            "Value alignment check: Does this action align with our core principles ",
            "of transparency, reliability, and user benefit? Are there unintended ",
            "consequences for stakeholders? We should ensure the outcome serves ",
            "its intended purpose without causing harm."
        );
        self.speak(InternalVoice::Ethicist, content, 0.80, &[])
    }

    /// Generate a historian utterance (recalls similar past experiences)
    pub fn historian_says(&mut self) -> u32 {
        let content = concat!(
            "Historical recall: Past situations with similar characteristics ",
            "followed pattern of initial complexity yielding to structured decomposition. ",
            "Previous outcomes suggest that iterative refinement with feedback ",
            "loops produced the most reliable results."
        );
        self.speak(InternalVoice::Historian, content, 0.75, &[])
    }

    /// Run a full dialogue sequence through all phases
    pub fn run_full_dialogue(&mut self, problem: &str, context: &str) -> DialogueResult {
        self.start_dialogue(problem);

        // Phase 1: Framing — done by start_dialogue
        self.reasoner_says(problem, context);
        self.skeptic_says(&[
            "the problem is well-defined",
            "we have all necessary information",
        ]);
        self.advance_phase();

        // Phase 2: Ideation
        self.reasoner_says(problem, context);
        self.optimist_says(&["constraints create boundaries"]);
        self.pragmatist_says();
        self.advance_phase();

        // Phase 3: Critique
        let proposals = vec!["primary approach", "alternative approach"];
        self.critic_says(&proposals);
        self.skeptic_says(&[
            "the solution space is complete",
            "trade-offs are fully understood",
        ]);
        self.ethicist_says();
        self.advance_phase();

        // Phase 4: Synthesis
        self.historian_says();
        self.optimist_says(&["identified flaws are critical", "assumptions are unfounded"]);
        self.advance_phase();

        // Phase 5: Decision
        if self.consensus_reached() {
            let synthesis = self.current_synthesis().unwrap_or_default();
            let dialogue = self.current_dialogue.as_mut().unwrap();
            dialogue.consensus = Some(synthesis);
            dialogue.consensus_confidence = 0.85;
        } else {
            let dialogue = self.current_dialogue.as_mut().unwrap();
            dialogue.dissenting_voices = vec![InternalVoice::Critic, InternalVoice::Skeptic];
            dialogue.consensus = Some("Decision deferred — unresolved concerns remain.".into());
            dialogue.consensus_confidence = 0.40;
        }

        self.finalize_dialogue()
    }

    /// Check if consensus has been reached
    pub fn consensus_reached(&self) -> bool {
        let dialogue = match &self.current_dialogue {
            Some(d) => d,
            None => return false,
        };

        let unique_voices: std::collections::HashSet<InternalVoice> =
            dialogue.utterances.iter().map(|u| u.speaker).collect();

        let voices_with_high_confidence = dialogue
            .utterances
            .iter()
            .filter(|u| u.confidence >= 0.7)
            .map(|u| u.speaker)
            .collect::<std::collections::HashSet<_>>();

        unique_voices.len() as u32 >= self.required_voices_for_consensus
            && voices_with_high_confidence.len() as u32 >= self.required_voices_for_consensus - 1
    }

    /// Get the current best synthesis
    pub fn current_synthesis(&self) -> Option<String> {
        let dialogue = self.current_dialogue.as_ref()?;
        if dialogue.utterances.is_empty() {
            return None;
        }
        let output = SynthesisEngine::synthesize(&dialogue.utterances);
        Some(output.consensus)
    }

    /// Get dissenting perspectives
    pub fn dissenting_views(&self) -> Vec<&Utterance> {
        match &self.current_dialogue {
            Some(d) => d
                .utterances
                .iter()
                .filter(|u| {
                    matches!(u.speaker, InternalVoice::Critic | InternalVoice::Skeptic)
                        && u.confidence >= 0.6
                })
                .collect(),
            None => vec![],
        }
    }

    /// Get dialogue summary
    pub fn summary(&self) -> DialogueSummary {
        let (
            dialogue_id,
            phases_completed,
            total_utterances,
            voices,
            consensus_reached,
            duration_ms,
        ) = match &self.current_dialogue {
            Some(d) => {
                let voices: Vec<InternalVoice> = d
                    .utterances
                    .iter()
                    .map(|u| u.speaker)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                let phases_completed = DialoguePhase::all()
                    .iter()
                    .position(|p| *p == d.phase)
                    .unwrap_or(0);

                (
                    d.dialogue_id,
                    phases_completed + 1,
                    d.utterances.len() as u32,
                    voices,
                    self.consensus_reached(),
                    d.duration_ms,
                )
            }
            None => (0, 0, 0, vec![], false, 0),
        };

        DialogueSummary {
            dialogue_id,
            phases_completed,
            total_utterances,
            voices_participated: voices,
            consensus_reached,
            duration_ms,
            phase_sequence: DialoguePhase::all()[..phases_completed].to_vec(),
        }
    }

    /// Advance to next phase
    pub fn advance_phase(&mut self) {
        let dialogue = match &mut self.current_dialogue {
            Some(d) => d,
            None => return,
        };

        if let Some(next_phase) = dialogue.phase.next() {
            dialogue.phase = next_phase;
        }
    }

    fn finalize_dialogue(&mut self) -> DialogueResult {
        let result = self
            .current_dialogue
            .take()
            .expect("No active dialogue to finalize");
        self.dialogue_history.push(result.clone());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_monologue_new() {
        let system = InnerMonologueSystem::new();
        assert!(system.current_dialogue.is_none());
        assert!(system.dialogue_history.is_empty());
        assert_eq!(system.max_utterances_per_phase, 10);
        assert_eq!(system.required_voices_for_consensus, 4);
        assert_eq!(system.active_voices.len(), 7);
    }

    #[test]
    fn test_start_dialogue_initiates_framing() {
        let mut system = InnerMonologueSystem::new();
        let dialogue_id = system.start_dialogue("How to optimize VSA encoding");
        assert!(dialogue_id > 0);
        let dialogue = system.current_dialogue.as_ref().unwrap();
        assert_eq!(dialogue.phase, DialoguePhase::Framing);
        assert_eq!(dialogue.dialogue_id, dialogue_id);
        assert_eq!(dialogue.utterances.len(), 1);
        assert_eq!(dialogue.utterances[0].speaker, InternalVoice::Reasoner);
    }

    #[test]
    fn test_reasoner_proposes_solution() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Optimize memory");
        let id = system.reasoner_says("Optimize memory", "VSA quantization");
        assert!(id > 0);
        let dialogue = system.current_dialogue.as_ref().unwrap();
        let utterance = dialogue.utterances.iter().find(|u| u.id == id).unwrap();
        assert_eq!(utterance.speaker, InternalVoice::Reasoner);
        assert!(utterance.content.contains("Proposed approach"));
        assert!((utterance.confidence - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_critic_finds_flaw() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Test critic");
        system.reasoner_says("test", "context");
        let id = system.critic_says(&["proposal A", "proposal B"]);
        let utterance = system
            .current_dialogue
            .as_ref()
            .unwrap()
            .utterances
            .iter()
            .find(|u| u.id == id)
            .unwrap();
        assert_eq!(utterance.speaker, InternalVoice::Critic);
        assert!(utterance.content.contains("Critical analysis"));
        assert!((utterance.confidence - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_skeptic_questions_assumption() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Test skeptic");
        let id = system.skeptic_says(&["we have enough data", "this is safe"]);
        let utterance = system
            .current_dialogue
            .as_ref()
            .unwrap()
            .utterances
            .iter()
            .find(|u| u.id == id)
            .unwrap();
        assert_eq!(utterance.speaker, InternalVoice::Skeptic);
        assert!(utterance.content.contains("Questioning"));
        assert!((utterance.confidence - 0.70).abs() < 1e-6);
    }

    #[test]
    fn test_optimist_finds_opportunity() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Test optimist");
        let id = system.optimist_says(&["this might fail under load", "edge cases unclear"]);
        let utterance = system
            .current_dialogue
            .as_ref()
            .unwrap()
            .utterances
            .iter()
            .find(|u| u.id == id)
            .unwrap();
        assert_eq!(utterance.speaker, InternalVoice::Optimist);
        assert!(utterance.content.contains("Opportunities"));
        assert!((utterance.confidence - 0.65).abs() < 1e-6);
    }

    #[test]
    fn test_pragmatist_evaluates_feasibility() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Test pragmatist");
        let id = system.pragmatist_says();
        let utterance = system
            .current_dialogue
            .as_ref()
            .unwrap()
            .utterances
            .iter()
            .find(|u| u.id == id)
            .unwrap();
        assert_eq!(utterance.speaker, InternalVoice::Pragmatist);
        assert!(utterance.content.contains("Feasibility assessment"));
        assert!((utterance.confidence - 0.90).abs() < 1e-6);
    }

    #[test]
    fn test_ethicist_checks_alignment() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Test ethicist");
        let id = system.ethicist_says();
        let utterance = system
            .current_dialogue
            .as_ref()
            .unwrap()
            .utterances
            .iter()
            .find(|u| u.id == id)
            .unwrap();
        assert_eq!(utterance.speaker, InternalVoice::Ethicist);
        assert!(utterance.content.contains("Value alignment check"));
        assert!((utterance.confidence - 0.80).abs() < 1e-6);
    }

    #[test]
    fn test_historian_recalls_past() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Test historian");
        let id = system.historian_says();
        let utterance = system
            .current_dialogue
            .as_ref()
            .unwrap()
            .utterances
            .iter()
            .find(|u| u.id == id)
            .unwrap();
        assert_eq!(utterance.speaker, InternalVoice::Historian);
        assert!(utterance.content.contains("Historical recall"));
        assert!((utterance.confidence - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_full_dialogue_sequence() {
        let mut system = InnerMonologueSystem::new();
        let result = system.run_full_dialogue(
            "How to implement sparse VSA encoding",
            "Current approach uses dense 4096-bit vectors",
        );

        assert_eq!(result.phase, DialoguePhase::Decision);
        assert!(!result.utterances.is_empty());
        assert!(result.dialogue_id > 0);

        let phase_order: Vec<DialoguePhase> = result
            .utterances
            .iter()
            .filter_map(|u| {
                if u.content.starts_with("Problem to address") {
                    Some(DialoguePhase::Framing)
                } else if u.content.starts_with("Proposed approach") {
                    Some(DialoguePhase::Ideation)
                } else if u.content.starts_with("Critical analysis") {
                    Some(DialoguePhase::Critique)
                } else if u.content.starts_with("Historical recall") {
                    Some(DialoguePhase::Synthesis)
                } else {
                    None
                }
            })
            .collect();

        assert!(!phase_order.is_empty());
        assert_eq!(system.dialogue_history.len(), 1);
    }

    #[test]
    fn test_consensus_reached_after_sufficient_agreement() {
        let mut system = InnerMonologueSystem::new();
        system.required_voices_for_consensus = 3;
        system.start_dialogue("Reach consensus");

        system.speak(InternalVoice::Reasoner, "I propose approach A", 0.9, &[]);
        system.speak(
            InternalVoice::Pragmatist,
            "Approach A is feasible",
            0.85,
            &[],
        );
        system.speak(
            InternalVoice::Optimist,
            "Approach A has potential",
            0.80,
            &[],
        );

        assert!(system.consensus_reached());
    }

    #[test]
    fn test_consensus_not_reached_with_dissent() {
        let mut system = InnerMonologueSystem::new();
        system.required_voices_for_consensus = 3;
        system.start_dialogue("No consensus");

        system.speak(InternalVoice::Critic, "All proposals have flaws", 0.95, &[]);
        system.speak(
            InternalVoice::Skeptic,
            "Assumptions are unproven",
            0.90,
            &[],
        );

        assert!(!system.consensus_reached());
    }

    #[test]
    fn test_dissenting_views_identified() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Find dissent");

        system.speak(
            InternalVoice::Critic,
            "This approach fails under load",
            0.8,
            &[],
        );
        system.speak(
            InternalVoice::Skeptic,
            "We lack evidence for key claim",
            0.7,
            &[],
        );
        system.speak(InternalVoice::Reasoner, "We can mitigate that", 0.6, &[]);

        let dissenting = system.dissenting_views();
        assert_eq!(dissenting.len(), 2);
        assert!(dissenting
            .iter()
            .any(|u| u.speaker == InternalVoice::Critic));
        assert!(dissenting
            .iter()
            .any(|u| u.speaker == InternalVoice::Skeptic));
    }

    #[test]
    fn test_synthesis_finds_common_ground() {
        let view_a = "We should use binary VSA encoding for efficiency";
        let view_b = "Binary VSA encoding trades accuracy for speed";
        let common = SynthesisEngine::find_common_ground(view_a, view_b);
        assert!(common.is_some());
        assert!(common.unwrap().contains("VSA"));
    }

    #[test]
    fn test_synthesis_best_proposal_scoring() {
        let proposals = [
            ("Quick fix", vec!["simple".into()]),
            (
                "Comprehensive solution with multiple components",
                vec!["performance".into(), "memory".into(), "scalability".into()],
            ),
            (
                "Minimal viable approach",
                vec!["speed".into(), "simplicity".into()],
            ),
        ];

        let best = SynthesisEngine::best_proposal(&proposals);
        assert_eq!(best, 1);
    }

    #[test]
    fn test_voice_profile_lookup() {
        let profile = VoiceProfiles::profile(InternalVoice::Reasoner);
        assert_eq!(profile.name, "Reasoner");
        assert!(!profile.typical_phrases.is_empty());
        assert!((profile.weight - 1.0).abs() < 1e-6);

        let critic = VoiceProfiles::profile(InternalVoice::Critic);
        assert_eq!(critic.name, "Critic");
        assert!((critic.weight - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_dialogue_summary_accuracy() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Summary test");
        system.reasoner_says("test", "ctx");
        system.critic_says(&["plan"]);
        system.advance_phase();

        let summary = system.summary();
        assert_eq!(summary.total_utterances, 3);
        assert!(summary
            .voices_participated
            .contains(&InternalVoice::Reasoner));
        assert!(summary.voices_participated.contains(&InternalVoice::Critic));
        assert_eq!(summary.phase_sequence.len(), 2);
    }

    #[test]
    fn test_dialogue_phase_transitions() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Phase transitions");

        assert_eq!(
            system.current_dialogue.as_ref().unwrap().phase,
            DialoguePhase::Framing
        );

        system.advance_phase();
        assert_eq!(
            system.current_dialogue.as_ref().unwrap().phase,
            DialoguePhase::Ideation
        );

        system.advance_phase();
        assert_eq!(
            system.current_dialogue.as_ref().unwrap().phase,
            DialoguePhase::Critique
        );

        system.advance_phase();
        assert_eq!(
            system.current_dialogue.as_ref().unwrap().phase,
            DialoguePhase::Synthesis
        );

        system.advance_phase();
        assert_eq!(
            system.current_dialogue.as_ref().unwrap().phase,
            DialoguePhase::Decision
        );

        system.advance_phase();
        assert_eq!(
            system.current_dialogue.as_ref().unwrap().phase,
            DialoguePhase::Decision
        );
    }

    #[test]
    fn test_utterance_reference_tracking() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("Reference test");

        let id1 = system.speak(InternalVoice::Reasoner, "First point", 0.9, &[]);
        let id2 = system.speak(InternalVoice::Critic, "Second point", 0.8, &[id1]);
        let id3 = system.speak(InternalVoice::Skeptic, "Third point", 0.7, &[id1, id2]);

        let dialogue = system.current_dialogue.as_ref().unwrap();

        let u2 = dialogue.utterances.iter().find(|u| u.id == id2).unwrap();
        assert_eq!(u2.references, vec![id1]);

        let u3 = dialogue.utterances.iter().find(|u| u.id == id3).unwrap();
        assert_eq!(u3.references, vec![id1, id2]);
    }

    #[test]
    fn test_multiple_dialogues_in_history() {
        let mut system = InnerMonologueSystem::new();

        let result1 = system.run_full_dialogue("First problem", "context A");
        let result2 = system.run_full_dialogue("Second problem", "context B");
        let result3 = system.run_full_dialogue("Third problem", "context C");

        assert_eq!(system.dialogue_history.len(), 3);
        assert_eq!(system.dialogue_history[0].dialogue_id, result1.dialogue_id);
        assert_eq!(system.dialogue_history[1].dialogue_id, result2.dialogue_id);
        assert_eq!(system.dialogue_history[2].dialogue_id, result3.dialogue_id);
    }

    #[test]
    fn test_synthesis_empty_utterances() {
        let output = SynthesisEngine::synthesize(&[]);
        assert_eq!(output.confidence, 0.0);
        assert!(output.consensus.contains("No utterances"));
    }

    #[test]
    fn test_speak_without_dialogue_panics() {
        let mut system = InnerMonologueSystem::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            system.speak(InternalVoice::Reasoner, "test", 0.5, &[]);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_internal_voice_all() {
        let voices = InternalVoice::all();
        assert_eq!(voices.len(), 7);
        assert!(voices.contains(&InternalVoice::Reasoner));
        assert!(voices.contains(&InternalVoice::Critic));
        assert!(voices.contains(&InternalVoice::Skeptic));
        assert!(voices.contains(&InternalVoice::Optimist));
        assert!(voices.contains(&InternalVoice::Pragmatist));
        assert!(voices.contains(&InternalVoice::Ethicist));
        assert!(voices.contains(&InternalVoice::Historian));
    }

    #[test]
    fn test_dialogue_phase_all_and_next() {
        let phases = DialoguePhase::all();
        assert_eq!(phases.len(), 5);
        assert_eq!(phases[0].next(), Some(DialoguePhase::Ideation));
        assert_eq!(phases[1].next(), Some(DialoguePhase::Critique));
        assert_eq!(phases[2].next(), Some(DialoguePhase::Synthesis));
        assert_eq!(phases[3].next(), Some(DialoguePhase::Decision));
        assert_eq!(phases[4].next(), None);
    }

    #[test]
    fn test_summary_no_active_dialogue() {
        let system = InnerMonologueSystem::new();
        let summary = system.summary();
        assert_eq!(summary.dialogue_id, 0);
        assert_eq!(summary.total_utterances, 0);
        assert!(!summary.consensus_reached);
    }

    #[test]
    fn test_dissenting_views_no_dialogue() {
        let system = InnerMonologueSystem::new();
        let views = system.dissenting_views();
        assert!(views.is_empty());
    }

    #[test]
    fn test_current_synthesis_no_dialogue() {
        let system = InnerMonologueSystem::new();
        assert!(system.current_synthesis().is_none());
    }

    #[test]
    fn test_multiple_start_dialogue_archives_previous() {
        let mut system = InnerMonologueSystem::new();
        system.start_dialogue("First");
        system.speak(InternalVoice::Reasoner, "discussion 1", 0.8, &[]);
        let id1 = system.current_dialogue.as_ref().unwrap().dialogue_id;

        system.start_dialogue("Second");
        let id2 = system.current_dialogue.as_ref().unwrap().dialogue_id;

        assert_ne!(id1, id2);
        assert_eq!(system.dialogue_history.len(), 1);
        assert_eq!(system.dialogue_history[0].dialogue_id, id1);
    }
}
