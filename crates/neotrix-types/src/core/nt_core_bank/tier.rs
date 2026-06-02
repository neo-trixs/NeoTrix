use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryTier {
    Working,
    Episodic,
    Semantic,
    Procedural,
}

impl MemoryTier {
    pub fn promote(&self) -> Option<Self> {
        match self {
            Self::Working => Some(Self::Episodic),
            Self::Episodic => Some(Self::Semantic),
            Self::Semantic => Some(Self::Procedural),
            Self::Procedural => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Procedural => "procedural",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifecycleAction {
    Promote(MemoryTier),
    Demote(MemoryTier),
    Retain,
    Archive,
    Evict,
}

/// Tier-system lifecycle policy — controls how memories flow between tiers.
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// Maximum age in seconds before auto-demotion (0 = never)
    pub tier_max_age: HashMap<MemoryTier, u64>,
    /// Minimum access count to stay in current tier
    pub tier_min_access: HashMap<MemoryTier, u32>,
    /// Decay rate per tick (0.0-1.0) for each tier
    pub tier_decay_rate: HashMap<MemoryTier, f64>,
    /// Promotion threshold: access count above which a memory is promoted
    pub promotion_access_threshold: u32,
    /// Demotion threshold: days since last access for demotion
    pub demotion_days_threshold: u64,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        use MemoryTier::*;
        let mut max_age = HashMap::new();
        max_age.insert(Working, 86400);
        max_age.insert(Episodic, 604800);
        max_age.insert(Semantic, 2592000);
        max_age.insert(Procedural, 0);

        let mut min_access = HashMap::new();
        min_access.insert(Working, 5);
        min_access.insert(Episodic, 3);
        min_access.insert(Semantic, 1);
        min_access.insert(Procedural, 0);

        let mut decay = HashMap::new();
        decay.insert(Working, 0.3);
        decay.insert(Episodic, 0.1);
        decay.insert(Semantic, 0.02);
        decay.insert(Procedural, 0.0);

        Self {
            tier_max_age: max_age,
            tier_min_access: min_access,
            tier_decay_rate: decay,
            promotion_access_threshold: 10,
            demotion_days_threshold: 30,
        }
    }
}

impl LifecycleConfig {
    pub fn evaluate(
        &self,
        current_tier: MemoryTier,
        access_count: u32,
        age_seconds: u64,
        last_access_days_ago: u64,
    ) -> LifecycleAction {
        let max_age = self.tier_max_age.get(&current_tier).copied().unwrap_or(0);
        if age_seconds > max_age && max_age > 0 {
            if last_access_days_ago > self.demotion_days_threshold {
                return LifecycleAction::Evict;
            }
            return self.demote(current_tier);
        }

        let min_acc = self.tier_min_access.get(&current_tier).copied().unwrap_or(0);
        if access_count < min_acc && min_acc > 0 {
            return self.demote(current_tier);
        }

        if access_count >= self.promotion_access_threshold {
            if let Some(promoted) = self.promote(current_tier) {
                return LifecycleAction::Promote(promoted);
            }
        }

        LifecycleAction::Retain
    }

    fn demote(&self, tier: MemoryTier) -> LifecycleAction {
        use MemoryTier::*;
        match tier {
            Working => LifecycleAction::Demote(Episodic),
            Episodic => LifecycleAction::Demote(Semantic),
            Semantic => LifecycleAction::Archive,
            Procedural => LifecycleAction::Retain,
        }
    }

    fn promote(&self, tier: MemoryTier) -> Option<MemoryTier> {
        use MemoryTier::*;
        match tier {
            Episodic => Some(Semantic),
            Semantic => Some(Procedural),
            _ => None,
        }
    }

    pub fn apply_decay(&self, tier: MemoryTier, importance: f64, ticks: u64) -> f64 {
        let rate = self.tier_decay_rate.get(&tier).copied().unwrap_or(0.0_f64);
        importance * (1.0_f64 - rate).powi(ticks as i32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLifecycle {
    pub importance: f64,
    pub confidence: f64,
    pub access_count: usize,
    pub created_at: i64,
    pub last_accessed: i64,
    pub ttl_seconds: Option<i64>,
}

impl MemoryLifecycle {
    pub fn new(importance: f64) -> Self {
        let now = Utc::now().timestamp();
        Self { importance, confidence: importance, access_count: 0, created_at: now, last_accessed: now, ttl_seconds: None }
    }

    pub fn with_ttl(importance: f64, ttl_seconds: i64) -> Self {
        let mut m = Self::new(importance);
        m.ttl_seconds = Some(ttl_seconds);
        m
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let now = Utc::now().timestamp();
            return now - self.created_at > ttl;
        }
        false
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promotion_threshold() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Episodic, 15, 100, 1);
        assert_eq!(action, LifecycleAction::Promote(MemoryTier::Semantic));
    }

    #[test]
    fn test_demotion_by_age() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Working, 1, 200000, 5);
        assert_eq!(action, LifecycleAction::Demote(MemoryTier::Episodic));
    }

    #[test]
    fn test_eviction() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Semantic, 0, 5000000, 90);
        assert_eq!(action, LifecycleAction::Evict);
    }

    #[test]
    fn test_decay() {
        let config = LifecycleConfig::default();
        let decayed = config.apply_decay(MemoryTier::Working, 1.0, 2);
        assert!((decayed - 0.49).abs() < 0.01);
    }

    #[test]
    fn test_no_promotion_from_working() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Working, 20, 0, 0);
        assert_ne!(action, LifecycleAction::Promote(MemoryTier::Episodic));
    }

    #[test]
    fn test_retain_normal() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Semantic, 2, 100, 5);
        assert_eq!(action, LifecycleAction::Retain);
    }

    #[test]
    fn test_procedural_never_demotes() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Procedural, 0, 999999, 999);
        assert_eq!(action, LifecycleAction::Retain);
    }

    #[test]
    fn test_demote_episodic_to_semantic() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Episodic, 0, 700000, 10);
        assert_eq!(action, LifecycleAction::Demote(MemoryTier::Semantic));
    }

    #[test]
    fn test_archive_from_semantic() {
        let config = LifecycleConfig::default();
        let action = config.evaluate(MemoryTier::Semantic, 0, 3000000, 5);
        assert_eq!(action, LifecycleAction::Archive);
    }
}
