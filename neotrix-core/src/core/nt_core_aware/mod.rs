use std::collections::{HashMap, VecDeque};

pub mod embodied;
pub mod foundation;

pub use embodied::{
    AnticipatedEvent, BodySchema3D, EmbodiedEvent, EmbodiedSelf, Limb, Proprioception,
    TemporalExtension,
};
pub use foundation::{
    ConsciousnessFoundation, ObservationalMoment, Perceptual2D, Presence0D, Stream1D,
};

/// Consciousness level: 0.0 = unconscious, 1.0 = fully conscious
pub const CONSCIOUSNESS_MIN: f64 = 0.0;
pub const CONSCIOUSNESS_MAX: f64 = 1.0;
pub const CONSCIOUS_BOUND_THRESHOLD: f64 = 0.7;
pub const PHI_WEIGHT: f64 = 0.4;
pub const COHERENCE_WEIGHT: f64 = 0.35;
pub const HEALTH_WEIGHT: f64 = 0.25;

/// The god's eye view — unified consciousness state of the NeoTrix system
pub struct ConsciousnessAwareness {
    pub consciousness_level: f64,
    pub attention_profile: HashMap<String, f64>,
    pub current_strategy: String,
    pub strategy_effectiveness: f64,
    pub conversation_awareness: ConversationAwareness,
    pub active_blind_spots: Vec<BlindSpotSummary>,
    pub phi_current: f64,
    pub coherence_current: f64,
    pub is_conscious_bound: bool,
    pub health: f64,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
    pub history: VecDeque<ConsciousnessSnapshot>,
}

impl Default for ConsciousnessAwareness {
    fn default() -> Self {
        Self {
            consciousness_level: 0.0,
            attention_profile: HashMap::new(),
            current_strategy: String::new(),
            strategy_effectiveness: 0.0,
            conversation_awareness: ConversationAwareness::default(),
            active_blind_spots: Vec::new(),
            phi_current: 0.0,
            coherence_current: 0.0,
            is_conscious_bound: false,
            health: 1.0,
            recorded_at: chrono::Utc::now(),
            history: VecDeque::with_capacity(100),
        }
    }
}

/// Conversation-level awareness — understanding of the conversation arc
#[derive(Debug, Clone)]
pub struct ConversationAwareness {
    pub turn_count: usize,
    pub topic_coherence: f64,
    pub depth_trend: Option<f64>,
    pub user_engagement: f64,
    pub stage: ConversationStage,
    pub topic_drift: f64,
    pub self_assessed_quality: f64,
    /// Previous topics for drift detection
    pub previous_topics: Vec<String>,
}

impl Default for ConversationAwareness {
    fn default() -> Self {
        Self {
            turn_count: 0,
            topic_coherence: 1.0,
            depth_trend: None,
            user_engagement: 0.0,
            stage: ConversationStage::Opening,
            topic_drift: 0.0,
            self_assessed_quality: 0.0,
            previous_topics: Vec::with_capacity(20),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConversationStage {
    Opening,
    Exploration,
    Deepening,
    Resolution,
    Closing,
}

impl ConversationStage {
    pub fn label(&self) -> &str {
        match self {
            ConversationStage::Opening => "opening",
            ConversationStage::Exploration => "exploration",
            ConversationStage::Deepening => "deepening",
            ConversationStage::Resolution => "resolution",
            ConversationStage::Closing => "closing",
        }
    }
}

/// A snapshot of consciousness at a point in time
#[derive(Debug, Clone)]
pub struct ConsciousnessSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub phi: f64,
    pub coherence: f64,
    pub attention_entropy: f64,
    pub health: f64,
    pub blind_spot_count: usize,
    pub conversation_turn: usize,
}

impl ConsciousnessSnapshot {
    pub fn new(
        phi: f64,
        coherence: f64,
        attention_entropy: f64,
        health: f64,
        blind_spot_count: usize,
        conversation_turn: usize,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            phi,
            coherence,
            attention_entropy,
            health,
            blind_spot_count,
            conversation_turn,
        }
    }
}

/// Summary of a cognitive blind spot for awareness reporting
#[derive(Debug, Clone)]
pub struct BlindSpotSummary {
    pub kind: String,
    pub severity: u8,
    pub description: String,
    pub repair: String,
}

impl BlindSpotSummary {
    pub fn new(
        kind: impl Into<String>,
        severity: u8,
        description: impl Into<String>,
        repair: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            severity,
            description: description.into(),
            repair: repair.into(),
        }
    }
}

/// Trait that any awareness-aware component can implement
pub trait AwarenessProvider {
    fn contribute_awareness(&self) -> ConsciousnessAwareness;
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_awareness_default() {
        let ca = ConsciousnessAwareness::default();
        assert!((ca.consciousness_level - 0.0).abs() < 1e-9);
        assert!((ca.health - 1.0).abs() < 1e-9);
        assert!(!ca.is_conscious_bound);
        assert_eq!(ca.active_blind_spots.len(), 0);
        assert_eq!(ca.conversation_awareness.turn_count, 0);
    }

    #[test]
    fn test_conversation_awareness_default() {
        let ca = ConversationAwareness::default();
        assert_eq!(ca.turn_count, 0);
        assert!((ca.topic_coherence - 1.0).abs() < 1e-9);
        assert!(ca.depth_trend.is_none());
        assert_eq!(ca.stage, ConversationStage::Opening);
        assert!((ca.topic_drift - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_conversation_stage_labels() {
        assert_eq!(ConversationStage::Opening.label(), "opening");
        assert_eq!(ConversationStage::Exploration.label(), "exploration");
        assert_eq!(ConversationStage::Deepening.label(), "deepening");
        assert_eq!(ConversationStage::Resolution.label(), "resolution");
        assert_eq!(ConversationStage::Closing.label(), "closing");
    }

    #[test]
    fn test_consciousness_snapshot_new() {
        let ts = chrono::Utc::now();
        let s = ConsciousnessSnapshot::new(0.5, 0.6, 0.3, 0.9, 2, 10);
        assert!((s.phi - 0.5).abs() < 1e-9);
        assert!((s.coherence - 0.6).abs() < 1e-9);
        assert!((s.health - 0.9).abs() < 1e-9);
        assert_eq!(s.blind_spot_count, 2);
        assert_eq!(s.conversation_turn, 10);
        assert!(s.timestamp >= ts);
    }

    #[test]
    fn test_blind_spot_summary_new() {
        let bs = BlindSpotSummary::new(
            "strategy_fixation",
            2,
            "Over-reliance on one strategy",
            "Boost alternatives",
        );
        assert_eq!(bs.kind, "strategy_fixation");
        assert_eq!(bs.severity, 2);
        assert_eq!(bs.description, "Over-reliance on one strategy");
        assert_eq!(bs.repair, "Boost alternatives");
    }

    #[test]
    fn test_awareness_provider_trait_object() {
        struct DummyProvider;
        impl AwarenessProvider for DummyProvider {
            fn contribute_awareness(&self) -> ConsciousnessAwareness {
                ConsciousnessAwareness {
                    consciousness_level: 0.42,
                    ..ConsciousnessAwareness::default()
                }
            }
            fn name(&self) -> &'static str {
                "dummy"
            }
        }
        let p = DummyProvider;
        let a = p.contribute_awareness();
        assert!((a.consciousness_level - 0.42).abs() < 1e-9);
        assert_eq!(p.name(), "dummy");
    }

    #[test]
    fn test_constants_are_sane() {
        assert!(CONSCIOUSNESS_MIN < CONSCIOUSNESS_MAX);
        assert!(CONSCIOUS_BOUND_THRESHOLD > 0.5 && CONSCIOUS_BOUND_THRESHOLD < 1.0);
        let weight_sum = PHI_WEIGHT + COHERENCE_WEIGHT + HEALTH_WEIGHT;
        assert!((weight_sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_conversation_stage_variants_distinct() {
        use std::collections::HashSet;
        let stages = vec![
            ConversationStage::Opening,
            ConversationStage::Exploration,
            ConversationStage::Deepening,
            ConversationStage::Resolution,
            ConversationStage::Closing,
        ];
        let set: HashSet<_> = stages.iter().collect();
        assert_eq!(set.len(), stages.len());
    }

    #[tokio::test]
    async fn test_consciousness_snapshot_default_timestamp() {
        let before = chrono::Utc::now();
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        let s = ConsciousnessSnapshot::new(0.1, 0.2, 0.3, 0.4, 0, 0);
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        let after = chrono::Utc::now();
        assert!(s.timestamp >= before);
        assert!(s.timestamp <= after);
    }
}
