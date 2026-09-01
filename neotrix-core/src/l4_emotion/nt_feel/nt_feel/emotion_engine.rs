#![forbid(unsafe_code)]

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::nt_core_self::emotion_state::{
    EmotionConfig, EmotionDimension, EmotionEngine as CoreEmotionEngine, EmotionLabel,
    EmotionReport,
};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeelConfig {
    /// Core emotion engine config (alpha, decay_rate, max_history).
    pub core: EmotionConfig,
    /// Threshold above which emotions are dampened (0.0–1.0).
    pub dampening_threshold: f64,
    /// Multiplier applied to emotions exceeding the threshold (0.0–1.0).
    pub dampening_factor: f64,
    /// Weight of social trust in the combined attention signal (0.0–1.0).
    pub social_trust_weight: f64,
    /// Weight of empathy in the combined attention signal (0.0–1.0).
    pub social_empathy_weight: f64,
    /// Maximum snapshots kept in emotion history.
    pub max_snapshots: usize,
}

impl Default for FeelConfig {
    fn default() -> Self {
        Self {
            core: EmotionConfig::default(),
            dampening_threshold: 0.85,
            dampening_factor: 0.6,
            social_trust_weight: 0.3,
            social_empathy_weight: 0.2,
            max_snapshots: 500,
        }
    }
}

// ---------------------------------------------------------------------------
// Emotion snapshot (point-in-time record)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionSnapshot {
    pub id: String,
    pub label: EmotionLabel,
    pub report: EmotionReport,
    pub social: SocialState,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Social state (trust / empathy from conversation context)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialState {
    /// Accumulated trust toward the interlocutor (0.0–1.0).
    pub trust: f64,
    /// Empathy score derived from shared emotional cues (0.0–1.0).
    pub empathy: f64,
    /// Number of conversational turns contributing to social state.
    pub interaction_count: u64,
}

impl Default for SocialState {
    fn default() -> Self {
        Self {
            trust: 0.5,
            empathy: 0.0,
            interaction_count: 0,
        }
    }
}

impl SocialState {
    /// Update trust based on conversation cues.
    /// Positive cues raise trust; negative cues lower it. Both are EMA-smoothed.
    fn update_trust(&mut self, positive_cue: bool, negative_cue: bool) {
        let delta = if positive_cue {
            0.15
        } else if negative_cue {
            -0.15
        } else {
            0.0
        };
        self.trust = (self.trust + delta).max(0.0).min(1.0);
    }

    /// Update empathy based on emotional resonance (shared valence direction).
    fn update_empathy(&mut self, resonance: f64) {
        let resonance = resonance.max(0.0).min(1.0);
        self.empathy = (0.3 * resonance + 0.7 * self.empathy).max(0.0).min(1.0);
    }

    fn increment_interaction(&mut self) {
        self.interaction_count = self.interaction_count.saturating_add(1);
    }
}

// ---------------------------------------------------------------------------
// Attention signal (output to ConsciousnessTree)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionSignal {
    /// Current emotion label after regulation.
    pub label: EmotionLabel,
    /// Arousal level (0.0–1.0), higher = more urgent attention demand.
    pub arousal: f64,
    /// Valence (0.0–1.0), higher = more positive.
    pub valence: f64,
    /// Social trust (0.0–1.0).
    pub social_trust: f64,
    /// Social empathy (0.0–1.0).
    pub social_empathy: f64,
    /// Combined salience for attention routing: arousal × (1 + social_modulation).
    pub salience: f64,
}

// ---------------------------------------------------------------------------
// FeelEngine — unified emotion engine for NT-FEEL
// ---------------------------------------------------------------------------

/// Unified emotion engine combining core PAD-based emotion detection with
/// regulation (dampening extremes), social emotion tracking, snapshot history,
/// and attention routing for the ConsciousnessTree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeelEngine {
    core: CoreEmotionEngine,
    config: FeelConfig,
    social: SocialState,
    snapshots: VecDeque<EmotionSnapshot>,
}

impl Default for FeelEngine {
    fn default() -> Self {
        Self::new(FeelConfig::default())
    }
}

impl FeelEngine {
    pub fn new(config: FeelConfig) -> Self {
        let max_snapshots = config.max_snapshots;
        Self {
            core: CoreEmotionEngine::new(config.core.clone()),
            config,
            social: SocialState::default(),
            snapshots: VecDeque::with_capacity(max_snapshots),
        }
    }

    // -- Core observation (delegated to EmotionEngine) ------------------------

    /// Observe an emotion dimension update.
    pub fn observe(&mut self, dim: EmotionDimension, value: f64, trigger: impl Into<String>) {
        let regulated = self.regulate(dim, value);
        self.core.observe(dim, regulated, trigger);
    }

    /// OCC appraisal observation.
    pub fn observe_appraisal(
        &mut self,
        novelty: f64,
        goal_conduciveness: f64,
        coping: f64,
        trigger: impl Into<String>,
    ) {
        self.core.observe_appraisal(novelty, goal_conduciveness, coping, trigger);
    }

    /// Detect emotion from text (keyword-based, mirrors DigitalHumanEmotion).
    pub fn detect_from_text(&mut self, text: &str) -> EmotionLabel {
        let lower = text.to_lowercase();
        if lower.contains("happy") || lower.contains("great") || lower.contains("thank")
            || lower.contains("开心") || lower.contains("高兴") || lower.contains("太好了")
        {
            self.observe(EmotionDimension::Joy, 0.8, "text_joy");
            self.social.update_trust(true, false);
        }
        if lower.contains("sad") || lower.contains("sorry") || lower.contains("bad")
            || lower.contains("难过") || lower.contains("伤心") || lower.contains("失望")
        {
            self.observe(EmotionDimension::Frustration, 0.7, "text_sadness");
        }
        if lower.contains("angry") || lower.contains("mad") || lower.contains("furious")
            || lower.contains("生气") || lower.contains("愤怒")
        {
            self.observe(EmotionDimension::Frustration, 0.9, "text_anger");
            self.observe(EmotionDimension::Urgency, 0.8, "text_anger");
            self.social.update_trust(false, true);
        }
        if lower.contains("wow") || lower.contains("amazing") || lower.contains("unexpected")
            || lower.contains("哇") || lower.contains("惊喜")
        {
            self.observe(EmotionDimension::Curiosity, 0.8, "text_surprise");
        }
        self.social.increment_interaction();
        let resonance = self.compute_resonance();
        self.social.update_empathy(resonance);
        self.current_label()
    }

    // -- Regulation -----------------------------------------------------------

    /// Dampen extreme emotions: if value exceeds the threshold, reduce it by
    /// the dampening factor. Low-arousal emotions pass through unchanged.
    fn regulate(&self, _dim: EmotionDimension, value: f64) -> f64 {
        let v = value.max(0.0).min(1.0);
        if v > self.config.dampening_threshold {
            let excess = v - self.config.dampening_threshold;
            self.config.dampening_threshold + excess * self.config.dampening_factor
        } else {
            v
        }
    }

    /// Compute emotional resonance with the interlocutor (valence alignment).
    fn compute_resonance(&self) -> f64 {
        let report = self.core.report();
        let v = report.valence;
        (1.0 - (v - 0.5).abs() * 2.0).max(0.0)
    }

    // -- Tick / decay ---------------------------------------------------------

    /// Advance time: decay all emotion dimensions toward baseline.
    pub fn tick(&mut self) {
        self.core.tick();
    }

    // -- Current state --------------------------------------------------------

    pub fn current_label(&self) -> EmotionLabel {
        self.core.report().emotion_label
    }

    pub fn report(&self) -> EmotionReport {
        self.core.report()
    }

    pub fn confidence_score(&self) -> f64 {
        self.core.report().confidence_score
    }

    pub fn social_state(&self) -> &SocialState {
        &self.social
    }

    pub fn config(&self) -> &FeelConfig {
        &self.config
    }

    // -- Snapshots -----------------------------------------------------------

    /// Take a snapshot of the current emotion state and append to history.
    pub fn snapshot(&mut self) -> EmotionSnapshot {
        let report = self.core.report();
        let snap = EmotionSnapshot {
            id: Uuid::new_v4().to_string(),
            label: report.emotion_label,
            report,
            social: self.social.clone(),
            timestamp: 0,
        };
        if self.snapshots.len() >= self.config.max_snapshots {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snap);
        self.snapshots.back().cloned().unwrap()
    }

    /// Return the last `n` snapshots (most recent first).
    pub fn recent_snapshots(&self, n: usize) -> Vec<&EmotionSnapshot> {
        self.snapshots.iter().rev().take(n).collect()
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    // -- Attention routing (ConsciousnessTree integration) --------------------

    /// Produce an attention signal that the ConsciousnessTree consumes to
    /// modulate attention allocation. High arousal + high social trust =
    /// higher salience.
    pub fn attention_signal(&self) -> AttentionSignal {
        let report = self.core.report();
        let arousal = report.arousal;
        let social_mod = self.config.social_trust_weight * self.social.trust
            + self.config.social_empathy_weight * self.social.empathy;
        let salience = arousal * (1.0 + social_mod);
        AttentionSignal {
            label: report.emotion_label,
            arousal,
            valence: report.valence,
            social_trust: self.social.trust,
            social_empathy: self.social.empathy,
            salience: salience.max(0.0).min(2.0),
        }
    }

    // -- JSON serialization ---------------------------------------------------

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feel_engine_default_config() {
        let engine = FeelEngine::default();
        assert_eq!(engine.current_label(), EmotionLabel::Neutral);
        assert_eq!(engine.social_state().trust, 0.5);
        assert_eq!(engine.social_state().empathy, 0.0);
    }

    #[test]
    fn test_observe_updates_state() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Joy, 0.8, "test_joy");
        let report = engine.report();
        assert!(report.joy > 0.5, "joy should rise above baseline");
    }

    #[test]
    fn test_regulation_dampens_extreme() {
        let mut engine = FeelEngine::default();
        // Feed extreme frustration that exceeds threshold
        engine.observe(EmotionDimension::Frustration, 0.99, "extreme");
        let report = engine.report();
        // After regulation, frustration should be below raw 0.99
        assert!(
            report.frustration < 0.95,
            "extreme emotion should be dampened, got {}",
            report.frustration
        );
    }

    #[test]
    fn test_regulation_passes_through_below_threshold() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Joy, 0.5, "mild");
        let report = engine.report();
        assert!(
            report.joy > 0.45 && report.joy < 0.65,
            "sub-threshold emotion should pass through, got {}",
            report.joy
        );
    }

    #[test]
    fn test_detect_from_text_joy() {
        let mut engine = FeelEngine::default();
        let label = engine.detect_from_text("太开心了，谢谢！");
        assert_eq!(label, EmotionLabel::Joy);
    }

    #[test]
    fn test_detect_from_text_anger() {
        let mut engine = FeelEngine::default();
        let label = engine.detect_from_text("I am so angry!");
        assert_eq!(label, EmotionLabel::Anger);
    }

    #[test]
    fn test_detect_from_text_neutral() {
        let mut engine = FeelEngine::default();
        let label = engine.detect_from_text("ordinary text nothing special");
        assert_eq!(label, EmotionLabel::Neutral);
    }

    #[test]
    fn test_social_trust_positive_cue() {
        let mut engine = FeelEngine::default();
        engine.detect_from_text("thank you very much");
        assert!(
            engine.social_state().trust > 0.5,
            "trust should rise on positive cue"
        );
    }

    #[test]
    fn test_social_trust_negative_cue() {
        let mut engine = FeelEngine::default();
        engine.detect_from_text("I am furious with you");
        assert!(
            engine.social_state().trust < 0.5,
            "trust should drop on negative cue"
        );
    }

    #[test]
    fn test_social_empathy_builds() {
        let mut engine = FeelEngine::default();
        // Multiple emotional exchanges should build empathy
        engine.detect_from_text("I am happy today");
        engine.detect_from_text("thank you so much");
        assert!(
            engine.social_state().empathy > 0.0,
            "empathy should accumulate"
        );
    }

    #[test]
    fn test_social_interaction_count() {
        let mut engine = FeelEngine::default();
        engine.detect_from_text("hello");
        engine.detect_from_text("thanks");
        engine.detect_from_text("wow");
        assert_eq!(engine.social_state().interaction_count, 3);
    }

    #[test]
    fn test_snapshot_stored() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Curiosity, 0.7, "curious");
        let snap = engine.snapshot();
        assert_eq!(snap.label, engine.current_label());
        assert_eq!(engine.snapshot_count(), 1);
    }

    #[test]
    fn test_recent_snapshots_order() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Joy, 0.6, "first");
        let _ = engine.snapshot();
        engine.observe(EmotionDimension::Frustration, 0.6, "second");
        let _ = engine.snapshot();
        let recent = engine.recent_snapshots(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].report.observation_count, 2);
    }

    #[test]
    fn test_snapshot_max_capacity() {
        let config = FeelConfig {
            max_snapshots: 3,
            ..Default::default()
        };
        let mut engine = FeelEngine::new(config);
        for i in 0..5 {
            engine.observe(EmotionDimension::Joy, 0.5, format!("obs_{i}"));
            let _ = engine.snapshot();
        }
        assert_eq!(engine.snapshot_count(), 3);
    }

    #[test]
    fn test_attention_signal_arousal() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Frustration, 0.9, "frustrated");
        engine.observe(EmotionDimension::Urgency, 0.8, "urgent");
        let signal = engine.attention_signal();
        assert!(
            signal.arousal > 0.5,
            "high frustration+urgency should produce high arousal"
        );
        assert!(signal.salience > 0.0);
    }

    #[test]
    fn test_attention_signal_social_modulation() {
        let mut engine = FeelEngine::default();
        // Build trust
        engine.detect_from_text("thank you very much");
        engine.detect_from_text("太开心了");
        let signal = engine.attention_signal();
        assert!(
            signal.social_trust > 0.5,
            "trust should modulate salience"
        );
        // Salience should be amplified by social trust
        let base_arousal = signal.arousal;
        let social_mod = 0.3 * signal.social_trust + 0.2 * signal.social_empathy;
        let expected_salience = base_arousal * (1.0 + social_mod);
        assert!(
            (signal.salience - expected_salience).abs() < 0.01,
            "salience should reflect social modulation"
        );
    }

    #[test]
    fn test_tick_decays() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Frustration, 0.9, "spike");
        let before = engine.report().frustration;
        engine.tick();
        let after = engine.report().frustration;
        assert!(
            after < before,
            "tick should decay emotion, before={} after={}",
            before,
            after
        );
    }

    #[test]
    fn test_confidence_score_range() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Confidence, 0.9, "confident");
        let score = engine.confidence_score();
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_appraisal_observation() {
        let mut engine = FeelEngine::default();
        engine.observe_appraisal(0.9, 0.9, 0.9, "novel_and_coping");
        let report = engine.report();
        assert!(
            report.curiosity > 0.5,
            "high novelty should raise curiosity"
        );
        assert!(
            report.confidence > 0.5,
            "high coping should raise confidence"
        );
    }

    #[test]
    fn test_json_roundtrip() {
        let mut engine = FeelEngine::default();
        engine.observe(EmotionDimension::Joy, 0.7, "json_test");
        engine.detect_from_text("开心");
        let json = engine.to_json().unwrap();
        let restored = FeelEngine::from_json(&json).unwrap();
        assert_eq!(restored.current_label(), engine.current_label());
        assert_eq!(
            restored.social_state().interaction_count,
            engine.social_state().interaction_count
        );
    }

    #[test]
    fn test_attention_signal_salience_capped() {
        let config = FeelConfig {
            social_trust_weight: 1.0,
            social_empathy_weight: 1.0,
            ..Default::default()
        };
        let mut engine = FeelEngine::new(config);
        // Max arousal + max social → salience should still be ≤ 2.0
        engine.observe(EmotionDimension::Frustration, 1.0, "max");
        engine.observe(EmotionDimension::Urgency, 1.0, "max");
        engine.social.trust = 1.0;
        engine.social.empathy = 1.0;
        let signal = engine.attention_signal();
        assert!(
            signal.salience <= 2.0,
            "salience should be capped at 2.0, got {}",
            signal.salience
        );
    }

    #[test]
    fn test_regulate_zero_passthrough() {
        let engine = FeelEngine::default();
        assert_eq!(engine.regulate(EmotionDimension::Joy, 0.0), 0.0);
    }

    #[test]
    fn test_regulate_clamp_above_one() {
        let engine = FeelEngine::default();
        let result = engine.regulate(EmotionDimension::Joy, 1.5);
        assert!(result <= 1.0);
    }

    #[test]
    fn test_regulate_clamp_below_zero() {
        let engine = FeelEngine::default();
        let result = engine.regulate(EmotionDimension::Joy, -0.5);
        assert!(result >= 0.0);
    }
}
