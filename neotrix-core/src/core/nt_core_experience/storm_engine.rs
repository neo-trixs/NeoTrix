use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Returns the 8 standard STORM perspectives (matches PerspectiveLens in reasoning_types.rs).
pub fn default_perspective_names() -> Vec<&'static str> {
    vec![
        "Practitioner", // Practitioner/Engineer — cares about implementation, feasibility
        "Architect",    // System Architect — cares about structure, scalability, integration
        "Skeptic",      // Skeptic/Critic — cares about risks, failure modes, limitations
        "Academic",     // Academic/Researcher — cares about evidence, rigor, related work
        "Economist",    // Economist/Business — cares about cost, ROI, incentives
        "Ethicist",     // Ethicist/Policy — cares about safety, bias, societal impact
        "User",         // User/End-User — cares about experience, usability, practical value
        "Historian",    // Historian — cares about past analogies, evolution, trajectory
    ]
}

/// Returns a map from perspective name to its default guiding prompt.
pub fn default_perspective_prompts() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("Practitioner", "Focus on practical implementation, feasibility, tooling, and real-world deployment challenges.");
    m.insert("Architect", "Focus on system design, architecture decisions, integration points, scalability, and maintainability.");
    m.insert(
        "Skeptic",
        "Focus on risks, failure modes, limitations, edge cases, and reasons this might not work.",
    );
    m.insert("Academic", "Focus on existing research, empirical evidence, theoretical foundations, and rigorous methodology.");
    m.insert(
        "Economist",
        "Focus on costs, benefits, incentives, market dynamics, resource allocation, and ROI.",
    );
    m.insert(
        "Ethicist",
        "Focus on ethical implications, bias, fairness, safety, transparency, and societal impact.",
    );
    m.insert("User", "Focus on end-user experience, usability, accessibility, practical value, and adoption barriers.");
    m.insert("Historian", "Focus on historical context, past analogies, evolution of similar ideas, and lessons learned.");
    m
}

// ── Phase 1: Perspective Definition ──

/// A research perspective with its assumptions, insights, blind spots, and questions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectiveDefinition {
    pub name: String,
    pub assumptions: Vec<String>,
    pub insights: Vec<String>,
    pub blind_spots: Vec<String>,
    pub questions: Vec<String>,
    /// VSA fingerprint (512-bit = 64 bytes) of this perspective for similarity matching.
    pub vsa_perspective: Option<Vec<u8>>,
}

// ── Phase 2: Conversation & Contradiction ──

/// A single turn in the simulated multi-perspective conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub perspective: String,
    pub question: String,
    pub answer: String,
    pub confidence: f64,
}

/// A detected contradiction between two perspectives on the same topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionEntry {
    pub topic: String,
    pub perspective_a: String,
    pub claim_a: String,
    pub perspective_b: String,
    pub claim_b: String,
    pub divergence: f64,
    pub resolution: Option<String>,
}

/// Maps agreements, contradictions, knowledge gaps, and reliability across perspectives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionMap {
    /// Topics where all perspectives agree.
    pub agreements: Vec<String>,
    /// Specific contradictions found.
    pub contradictions: Vec<ContradictionEntry>,
    /// Topics not adequately covered by any perspective.
    pub knowledge_gaps: Vec<String>,
    /// Reliability rating per topic area (topic, rating).
    pub reliability_ratings: Vec<(String, ReliabilityRating)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReliabilityRating {
    High,
    Medium,
    Low,
}

// ── Phase 3: Outline-Driven Report ──

/// A section of the STORM report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormReportSection {
    pub heading: String,
    pub content: String,
    pub contradictions: Vec<ContradictionEntry>,
}

/// Wikipedia-style research report with contradiction-aware structuring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormReport {
    pub title: String,
    pub introduction: String,
    pub sections: Vec<StormReportSection>,
    pub contradiction_map: ContradictionMap,
    pub recommendations: Vec<String>,
    pub reliability_assessment: String,
}

// ── Phase 4: Self-Critique ──

/// Peer-review style critique of the generated report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormCritique {
    pub overall_score: f64,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub biases: Vec<String>,
    pub oversimplifications: Vec<String>,
    pub missing_angles: Vec<String>,
    pub improvements: Vec<String>,
    pub final_version: String,
}

// ── Configuration ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormConfig {
    pub enabled: bool,
    pub perspectives_count: usize,
    pub questions_per_perspective: usize,
    pub conversation_rounds: usize,
    pub max_report_sections: usize,
    pub coherence_threshold: f64,
}

impl Default for StormConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            perspectives_count: 8,
            questions_per_perspective: 4,
            conversation_rounds: 2,
            max_report_sections: 8,
            coherence_threshold: 0.55,
        }
    }
}

// ── Phase tracker ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StormPhase {
    Idle,
    PerspectiveDiscovery,
    ConversationSimulation,
    Synthesis,
    SelfCritique,
    Complete,
}

// ── Main Engine ──

/// STORM/Co-STORM research pipeline orchestrator.
///
/// Runs 4-phase multi-perspective research:
/// 1. PerspectiveDiscovery — generates 6-8 diverse viewpoints
/// 2. ConversationSimulation — simulates dialogue + builds ContradictionMap
/// 3. Synthesis — outline-driven Wikipedia-style report
/// 4. SelfCritique — peer-review polish loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormEngine {
    pub config: StormConfig,
    pub phase: StormPhase,
    pub topic: String,
    pub perspectives: Vec<PerspectiveDefinition>,
    pub conversation: Vec<ConversationTurn>,
    pub contradiction_map: Option<ContradictionMap>,
    pub report: Option<StormReport>,
    pub critique: Option<StormCritique>,
    pub cycle: u64,
}

impl StormEngine {
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            config: StormConfig::default(),
            phase: StormPhase::Idle,
            topic: topic.into(),
            perspectives: Vec::new(),
            conversation: Vec::new(),
            contradiction_map: None,
            report: None,
            critique: None,
            cycle: 0,
        }
    }

    pub fn reset(&mut self, new_topic: impl Into<String>) {
        self.phase = StormPhase::Idle;
        self.topic = new_topic.into();
        self.perspectives.clear();
        self.conversation.clear();
        self.contradiction_map = None;
        self.report = None;
        self.critique = None;
        self.cycle = 0;
    }

    pub fn stats(&self) -> String {
        format!(
            "STORM: phase={:?} topic=\"{}\" perspectives={} turns={} report={} critique={} cycle={}",
            self.phase,
            self.topic,
            self.perspectives.len(),
            self.conversation.len(),
            self.report.is_some() as u8,
            self.critique.is_some() as u8,
            self.cycle,
        )
    }

    // ── Phase transitions ──

    /// Advance to the next phase. Returns true if there is a next phase.
    pub fn advance_phase(&mut self) -> bool {
        self.phase = match self.phase {
            StormPhase::Idle => StormPhase::PerspectiveDiscovery,
            StormPhase::PerspectiveDiscovery => StormPhase::ConversationSimulation,
            StormPhase::ConversationSimulation => StormPhase::Synthesis,
            StormPhase::Synthesis => StormPhase::SelfCritique,
            StormPhase::SelfCritique => StormPhase::Complete,
            StormPhase::Complete => StormPhase::Complete,
        };
        self.phase != StormPhase::Complete
    }

    /// Start research on a new topic (Idle -> PerspectiveDiscovery).
    pub fn start_research(&mut self, topic: impl Into<String>) {
        self.reset(topic);
        self.phase = StormPhase::PerspectiveDiscovery;
    }
}

impl StormPhase {
    /// Return the next phase after this one.
    pub fn next(&self) -> StormPhase {
        match self {
            StormPhase::Idle => StormPhase::PerspectiveDiscovery,
            StormPhase::PerspectiveDiscovery => StormPhase::ConversationSimulation,
            StormPhase::ConversationSimulation => StormPhase::Synthesis,
            StormPhase::Synthesis => StormPhase::SelfCritique,
            StormPhase::SelfCritique => StormPhase::Complete,
            StormPhase::Complete => StormPhase::Complete,
        }
    }

    /// Human-readable label for tick reporting.
    pub fn label(&self) -> &'static str {
        match self {
            StormPhase::Idle => "idle",
            StormPhase::PerspectiveDiscovery => "perspective_discovery",
            StormPhase::ConversationSimulation => "conversation_simulation",
            StormPhase::Synthesis => "synthesis",
            StormPhase::SelfCritique => "self_critique",
            StormPhase::Complete => "complete",
        }
    }

    /// Return true if this phase can accept new data.
    pub fn accepting(&self) -> bool {
        matches!(
            self,
            StormPhase::PerspectiveDiscovery
                | StormPhase::ConversationSimulation
                | StormPhase::Synthesis
                | StormPhase::SelfCritique
        )
    }
}
