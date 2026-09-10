use std::collections::{HashMap, VecDeque};

/// Consciousness level: 0.0 = unconscious, 1.0 = fully conscious
#[derive(Debug, Clone)]
pub struct ConsciousnessConfig {
    pub min_level: f64,
    pub max_level: f64,
    pub bound_threshold: f64,
    pub phi_weight: f64,
    pub coherence_weight: f64,
    pub health_weight: f64,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            min_level: 0.0,
            max_level: 1.0,
            bound_threshold: 0.7,
            phi_weight: 0.4,
            coherence_weight: 0.35,
            health_weight: 0.25,
        }
    }
}

impl ConsciousnessConfig {
    pub fn consciousness_level(&self, phi: f64, coherence: f64, health: f64) -> f64 {
        (self.phi_weight * phi + self.coherence_weight * coherence + self.health_weight * health)
            .max(self.min_level)
            .min(self.max_level)
    }
}

pub const DEFAULT_CONSCIOUSNESS_CONFIG: ConsciousnessConfig = ConsciousnessConfig {
    min_level: 0.0,
    max_level: 1.0,
    bound_threshold: 0.7,
    phi_weight: 0.4,
    coherence_weight: 0.35,
    health_weight: 0.25,
};

pub const CONSCIOUSNESS_MIN: f64 = DEFAULT_CONSCIOUSNESS_CONFIG.min_level;
pub const CONSCIOUSNESS_MAX: f64 = DEFAULT_CONSCIOUSNESS_CONFIG.max_level;
pub const CONSCIOUS_BOUND_THRESHOLD: f64 = DEFAULT_CONSCIOUSNESS_CONFIG.bound_threshold;
pub const PHI_WEIGHT: f64 = DEFAULT_CONSCIOUSNESS_CONFIG.phi_weight;
pub const COHERENCE_WEIGHT: f64 = DEFAULT_CONSCIOUSNESS_CONFIG.coherence_weight;
pub const HEALTH_WEIGHT: f64 = DEFAULT_CONSCIOUSNESS_CONFIG.health_weight;

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

impl ConsciousnessAwareness {
    pub fn new(phi: f64, coherence: f64, health: f64) -> Self {
        let config = DEFAULT_CONSCIOUSNESS_CONFIG;
        let level = config.consciousness_level(phi, coherence, health);
        Self {
            consciousness_level: level,
            phi_current: phi,
            coherence_current: coherence,
            health,
            is_conscious_bound: coherence > CONSCIOUS_BOUND_THRESHOLD
                && phi > CONSCIOUS_BOUND_THRESHOLD,
            ..Self::default()
        }
    }

    pub fn record_snapshot(&mut self) {
        let snapshot = ConsciousnessSnapshot::new(
            self.phi_current,
            self.coherence_current,
            self.attention_entropy(),
            self.health,
            self.active_blind_spots.len(),
            self.conversation_awareness.turn_count,
        );
        self.history.push_back(snapshot);
        if self.history.len() > 100 {
            self.history.pop_front();
        }
        self.recorded_at = chrono::Utc::now();
    }

    pub fn update_metrics(&mut self, phi: f64, coherence: f64, health: f64) {
        let config = DEFAULT_CONSCIOUSNESS_CONFIG;
        self.phi_current = phi;
        self.coherence_current = coherence;
        self.health = health;
        self.consciousness_level = config.consciousness_level(phi, coherence, health);
        self.is_conscious_bound =
            coherence > CONSCIOUS_BOUND_THRESHOLD && phi > CONSCIOUS_BOUND_THRESHOLD;
    }

    pub fn set_attention(&mut self, key: impl Into<String>, weight: f64) {
        self.attention_profile
            .insert(key.into(), weight.clamp(0.0, 1.0));
    }

    pub fn attention_entropy(&self) -> f64 {
        if self.attention_profile.is_empty() {
            return 0.0;
        }
        let total: f64 = self.attention_profile.values().sum();
        if total <= 0.0 {
            return 0.0;
        }
        self.attention_profile
            .values()
            .filter(|&&v| v > 0.0)
            .map(|&v| {
                let p = v / total;
                -p * p.log2()
            })
            .sum()
    }

    pub fn top_attention(&self, n: usize) -> Vec<(&str, f64)> {
        let mut pairs: Vec<_> = self
            .attention_profile
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs.into_iter().take(n).collect()
    }

    pub fn effective_dims(&self) -> usize {
        self.attention_profile
            .values()
            .filter(|&&v| v > 1e-6)
            .count()
    }

    pub fn is_dormant(&self) -> bool {
        self.consciousness_level < 0.1 && self.health < 0.3
    }

    pub fn merge_attention(&mut self, other: &HashMap<String, f64>) {
        for (k, v) in other {
            let entry = self.attention_profile.entry(k.clone()).or_insert(0.0);
            *entry = (*entry + v) / 2.0;
        }
    }

    pub fn summarise(&self) -> String {
        format!(
            "Consciousness {:.3} | Phi {:.4} | Coh {:.3} | Health {:.3} | BlindSpots {} | Stage {}",
            self.consciousness_level,
            self.phi_current,
            self.coherence_current,
            self.health,
            self.active_blind_spots.len(),
            self.conversation_awareness.stage.label(),
        )
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

impl ConversationAwareness {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_turn(&mut self, topic: &str, response_len: usize, complexity: f64) {
        let prev_topic = self.previous_topics.last().cloned();
        let prev_depth = self.depth_trend;

        let (drift, coherence) = if let Some(ref prev) = prev_topic {
            let overlap = Self::keyword_overlap(prev, topic);
            (1.0 - overlap, overlap)
        } else {
            (0.0, 1.0)
        };

        let engagement = (response_len as f64 / 5000.0).min(1.0);
        let depth_delta = match prev_depth {
            Some(prev) => complexity - prev,
            None => 0.0,
        };
        let quality = 0.4 * coherence + 0.3 * engagement + 0.3 * (1.0 - drift);

        self.turn_count += 1;
        self.topic_drift = drift;
        self.topic_coherence = coherence;
        self.user_engagement = engagement;
        self.depth_trend = Some(depth_delta);
        self.self_assessed_quality = quality;
        self.stage = ConversationStage::estimate(self.turn_count, drift);
        self.previous_topics.push(topic.to_string());
        if self.previous_topics.len() > 20 {
            self.previous_topics.remove(0);
        }
    }

    pub fn is_opening(&self) -> bool {
        matches!(self.stage, ConversationStage::Opening)
    }

    pub fn is_closing(&self) -> bool {
        matches!(self.stage, ConversationStage::Closing)
    }

    pub fn engagement_score(&self) -> f64 {
        0.4 * self.topic_coherence + 0.3 * self.user_engagement + 0.3 * self.self_assessed_quality
    }

    fn keyword_overlap(a: &str, b: &str) -> f64 {
        let words_a: std::collections::HashSet<&str> =
            a.split_whitespace().filter(|w| w.len() > 3).collect();
        let words_b: std::collections::HashSet<&str> =
            b.split_whitespace().filter(|w| w.len() > 3).collect();
        if words_a.is_empty() || words_b.is_empty() {
            return 0.5;
        }
        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();
        intersection as f64 / union as f64
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

    pub fn estimate(turn_count: usize, topic_drift: f64) -> Self {
        match turn_count {
            0..=2 => ConversationStage::Opening,
            3..=6 if topic_drift < 0.7 => ConversationStage::Exploration,
            7..=15 if topic_drift < 0.4 => ConversationStage::Deepening,
            _ if topic_drift < 0.25 => ConversationStage::Resolution,
            _ => ConversationStage::Closing,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ConversationStage::Closing)
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

    #[test]
    fn test_consciousness_snapshot_default_timestamp() {
        let before = chrono::Utc::now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let s = ConsciousnessSnapshot::new(0.1, 0.2, 0.3, 0.4, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let after = chrono::Utc::now();
        assert!(s.timestamp >= before);
        assert!(s.timestamp <= after);
    }

    #[test]
    fn test_consciousness_awareness_new() {
        let ca = ConsciousnessAwareness::new(0.8, 0.7, 0.9);
        assert!((ca.phi_current - 0.8).abs() < 1e-9);
        assert!((ca.coherence_current - 0.7).abs() < 1e-9);
        assert!((ca.health - 0.9).abs() < 1e-9);
        assert!(ca.consciousness_level > 0.0);
        assert!(ca.is_conscious_bound);
    }

    #[test]
    fn test_consciousness_awareness_new_below_bound() {
        let ca = ConsciousnessAwareness::new(0.3, 0.2, 0.5);
        assert!(!ca.is_conscious_bound);
    }

    #[test]
    fn test_consciousness_awareness_update_metrics() {
        let mut ca = ConsciousnessAwareness::default();
        ca.update_metrics(0.6, 0.5, 0.7);
        assert!((ca.phi_current - 0.6).abs() < 1e-9);
        assert!((ca.consciousness_level - 0.575).abs() < 1e-9);
    }

    #[test]
    fn test_consciousness_awareness_set_and_top_attention() {
        let mut ca = ConsciousnessAwareness::default();
        ca.set_attention("reasoning", 0.9);
        ca.set_attention("perception", 0.5);
        ca.set_attention("memory", 0.7);
        let top = ca.top_attention(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "reasoning");
        assert!(top[0].1 >= top[1].1);
    }

    #[test]
    fn test_consciousness_awareness_attention_entropy_empty() {
        let ca = ConsciousnessAwareness::default();
        assert!((ca.attention_entropy()).abs() < 1e-9);
    }

    #[test]
    fn test_consciousness_awareness_effective_dims() {
        let mut ca = ConsciousnessAwareness::default();
        ca.set_attention("a", 0.5);
        ca.set_attention("b", 0.0001);
        assert_eq!(ca.effective_dims(), 1);
    }

    #[test]
    fn test_consciousness_awareness_is_dormant() {
        let mut ca = ConsciousnessAwareness::default();
        assert!(ca.is_dormant());
        ca.consciousness_level = 0.5;
        assert!(!ca.is_dormant());
    }

    #[test]
    fn test_consciousness_awareness_merge_attention() {
        let mut ca = ConsciousnessAwareness::default();
        ca.set_attention("x", 0.8);
        let mut other = HashMap::new();
        other.insert("x".to_string(), 0.4);
        other.insert("y".to_string(), 0.6);
        ca.merge_attention(&other);
        assert!((ca.attention_profile["x"] - 0.6).abs() < 1e-9);
        assert!((ca.attention_profile["y"] - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_consciousness_awareness_record_snapshot() {
        let mut ca = ConsciousnessAwareness::new(0.5, 0.6, 0.8);
        ca.record_snapshot();
        assert_eq!(ca.history.len(), 1);
        let s = &ca.history[0];
        assert!((s.phi - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_consciousness_awareness_summarise() {
        let ca = ConsciousnessAwareness::new(0.6, 0.5, 0.7);
        let s = ca.summarise();
        assert!(s.contains("Consciousness"));
        assert!(s.contains("Phi"));
    }

    #[test]
    fn test_conversation_awareness_new() {
        let ca = ConversationAwareness::new();
        assert_eq!(ca.turn_count, 0);
        assert_eq!(ca.stage, ConversationStage::Opening);
    }

    #[test]
    fn test_conversation_awareness_record_turn() {
        let mut ca = ConversationAwareness::new();
        ca.record_turn("hello world first", 200, 0.3);
        assert_eq!(ca.turn_count, 1);
        assert_eq!(ca.stage, ConversationStage::Opening);

        ca.record_turn("hello world exploring deeper concepts", 800, 0.6);
        assert_eq!(ca.turn_count, 2);
        assert!(!ca.previous_topics.is_empty());
    }

    #[test]
    fn test_conversation_awareness_engagement_score() {
        let mut ca = ConversationAwareness::new();
        ca.topic_coherence = 0.8;
        ca.user_engagement = 0.6;
        ca.self_assessed_quality = 0.7;
        let score = ca.engagement_score();
        let expected = 0.4 * 0.8 + 0.3 * 0.6 + 0.3 * 0.7;
        assert!((score - expected).abs() < 1e-9);
    }

    #[test]
    fn test_conversation_stage_estimate() {
        assert_eq!(
            ConversationStage::estimate(1, 0.0),
            ConversationStage::Opening
        );
        assert_eq!(
            ConversationStage::estimate(5, 0.3),
            ConversationStage::Exploration
        );
        assert_eq!(
            ConversationStage::estimate(10, 0.2),
            ConversationStage::Deepening
        );
        assert_eq!(
            ConversationStage::estimate(20, 0.1),
            ConversationStage::Resolution
        );
        assert_eq!(
            ConversationStage::estimate(20, 0.5),
            ConversationStage::Closing
        );
    }

    #[test]
    fn test_conversation_stage_is_terminal() {
        assert!(!ConversationStage::Opening.is_terminal());
        assert!(ConversationStage::Closing.is_terminal());
    }
}
