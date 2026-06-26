use std::collections::VecDeque;

/// MS AGT-inspired dynamic trust scoring engine for NeoTrix governance.
/// Score range: 0–1000 (default 500 = neutral).
/// Behavioral tiers: Normal (≥600), Elevated (≥400), High (≥200), Critical (<200).
/// Each tier gates different operations: Normal=full, Elevated=review-required, High=restricted, Critical=blocked.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BehavioralTier {
    Normal = 3,
    Elevated = 2,
    High = 1,
    Critical = 0,
}

impl BehavioralTier {
    pub fn from_score(score: u16) -> Self {
        match score {
            600..=1000 => BehavioralTier::Normal,
            400..=599 => BehavioralTier::Elevated,
            200..=399 => BehavioralTier::High,
            _ => BehavioralTier::Critical,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            BehavioralTier::Normal => "normal",
            BehavioralTier::Elevated => "elevated",
            BehavioralTier::High => "high",
            BehavioralTier::Critical => "critical",
        }
    }

    /// Whether operations at this tier require human review
    pub fn requires_review(&self) -> bool {
        matches!(self, BehavioralTier::Elevated | BehavioralTier::High)
    }

    /// Whether operations at this tier are blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self, BehavioralTier::Critical)
    }
}

#[derive(Debug, Clone)]
pub struct TrustEvidence {
    pub source: String,
    pub weight: f64,
    pub score_delta: i32,
    pub timestamp: u64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct TrustScoringConfig {
    pub initial_score: u16,
    pub decay_per_tick: i32,
    pub regression_mean: u16,
    pub max_evidence_history: usize,
    pub positive_cap: u16,
    pub negative_floor: u16,
}

impl Default for TrustScoringConfig {
    fn default() -> Self {
        Self {
            initial_score: 500,
            decay_per_tick: 1,
            regression_mean: 500,
            max_evidence_history: 200,
            positive_cap: 1000,
            negative_floor: 0,
        }
    }
}

pub struct TrustScoringEngine {
    pub score: u16,
    pub tier: BehavioralTier,
    pub evidence: VecDeque<TrustEvidence>,
    pub config: TrustScoringConfig,
    pub total_ticks: u64,
    pub escalated: bool,
}

impl TrustScoringEngine {
    pub fn new() -> Self {
        Self::with_config(TrustScoringConfig::default())
    }

    pub fn with_config(config: TrustScoringConfig) -> Self {
        let score = config.initial_score;
        Self {
            tier: BehavioralTier::from_score(score),
            score,
            evidence: VecDeque::with_capacity(config.max_evidence_history),
            config,
            total_ticks: 0,
            escalated: false,
        }
    }

    /// Record positive evidence (successful action, compliance, etc.)
    pub fn record_positive(&mut self, source: &str, weight: f64, description: &str) {
        let delta = (weight * 50.0) as i32;
        let new_score = (self.score as i32 + delta).min(self.config.positive_cap as i32) as u16;
        self.apply_score(new_score, source, delta, weight, description);
    }

    /// Record negative evidence (violation, failure, suspicious behavior, etc.)
    pub fn record_negative(&mut self, source: &str, weight: f64, description: &str) {
        let delta = -(weight * 80.0) as i32;
        let new_score = (self.score as i32 + delta).max(self.config.negative_floor as i32) as u16;
        self.apply_score(new_score, source, delta, weight, description);
    }

    fn apply_score(
        &mut self,
        new_score: u16,
        source: &str,
        delta: i32,
        weight: f64,
        description: &str,
    ) {
        let old_tier = self.tier;
        self.score = new_score;
        self.tier = BehavioralTier::from_score(self.score);
        if self.evidence.len() >= self.config.max_evidence_history {
            self.evidence.pop_front();
        }
        self.evidence.push_back(TrustEvidence {
            source: source.to_string(),
            weight,
            score_delta: delta,
            timestamp: self.total_ticks,
            description: description.to_string(),
        });
        if self.tier != old_tier {
            self.escalated = self.tier < old_tier;
        }
    }

    /// Decay score toward regression mean every tick
    pub fn tick(&mut self) {
        self.total_ticks += 1;
        let mean = self.config.regression_mean as i32;
        let current = self.score as i32;
        if current > mean {
            let decay = self.config.decay_per_tick.min(current - mean);
            self.score = (current - decay) as u16;
        } else if current < mean {
            let decay = self.config.decay_per_tick.min(mean - current);
            self.score = (current + decay) as u16;
        }
        self.tier = BehavioralTier::from_score(self.score);
    }

    /// Check if a specific operation is permitted at current tier
    pub fn permit_operation(&self, operation: &str) -> (bool, String) {
        match (self.tier, operation) {
            (BehavioralTier::Critical, _) => (
                false,
                "blocked: critical tier — all operations suspended".into(),
            ),
            (BehavioralTier::High, "self_modify") | (BehavioralTier::High, "identity_evolve") => {
                (true, "restricted: high tier — requires review".into())
            }
            (BehavioralTier::Elevated, "self_modify")
            | (BehavioralTier::Elevated, "identity_evolve") => {
                (true, "restricted: elevated tier — requires review".into())
            }
            (BehavioralTier::Normal, _) => (true, "permitted: normal tier".into()),
            _ => (true, "permitted".into()),
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "trust: score={}/1000 tier={} evidence_count={} ticks={} escalated={}",
            self.score,
            self.tier.label(),
            self.evidence.len(),
            self.total_ticks,
            self.escalated,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tier_is_normal() {
        let engine = TrustScoringEngine::new();
        assert_eq!(engine.tier, BehavioralTier::Normal);
        assert_eq!(engine.score, 500);
    }

    #[test]
    fn test_negative_evidence_lowers_tier() {
        let mut engine = TrustScoringEngine::new();
        engine.record_negative("test", 5.0, "severe violation");
        assert_eq!(engine.tier, BehavioralTier::Elevated);
        assert!(engine.score < 500);
    }

    #[test]
    fn test_positive_evidence_raises_score() {
        let mut engine = TrustScoringEngine::new();
        engine.score = 300;
        engine.record_positive("test", 5.0, "successful compliance");
        assert!(engine.score > 300);
    }

    #[test]
    fn test_tick_decay_toward_mean() {
        let mut engine = TrustScoringEngine::new();
        engine.score = 800;
        engine.tick();
        assert!(engine.score < 800);
        engine.score = 200;
        engine.tick();
        assert!(engine.score > 200);
    }

    #[test]
    fn test_critical_tier_blocks_all() {
        let mut engine = TrustScoringEngine::new();
        engine.score = 100;
        engine.tier = BehavioralTier::from_score(100);
        let (permitted, _) = engine.permit_operation("self_modify");
        assert!(!permitted);
        let (permitted, _) = engine.permit_operation("read");
        assert!(!permitted);
    }

    #[test]
    fn test_normal_tier_permits_all() {
        let engine = TrustScoringEngine::new();
        let (permitted, _) = engine.permit_operation("self_modify");
        assert!(permitted);
    }

    #[test]
    fn test_summary_format() {
        let engine = TrustScoringEngine::new();
        let s = engine.summary();
        assert!(s.contains("trust:"));
        assert!(s.contains("score="));
    }

    #[test]
    fn test_tier_from_score_boundaries() {
        assert_eq!(BehavioralTier::from_score(1000), BehavioralTier::Normal);
        assert_eq!(BehavioralTier::from_score(600), BehavioralTier::Normal);
        assert_eq!(BehavioralTier::from_score(599), BehavioralTier::Elevated);
        assert_eq!(BehavioralTier::from_score(400), BehavioralTier::Elevated);
        assert_eq!(BehavioralTier::from_score(399), BehavioralTier::High);
        assert_eq!(BehavioralTier::from_score(200), BehavioralTier::High);
        assert_eq!(BehavioralTier::from_score(199), BehavioralTier::Critical);
        assert_eq!(BehavioralTier::from_score(0), BehavioralTier::Critical);
    }
}
