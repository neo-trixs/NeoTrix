use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    fn test_memory_tier_promote_working() {
        assert_eq!(MemoryTier::Working.promote(), Some(MemoryTier::Episodic));
    }

    #[test]
    fn test_memory_tier_promote_episodic() {
        assert_eq!(MemoryTier::Episodic.promote(), Some(MemoryTier::Semantic));
    }

    #[test]
    fn test_memory_tier_promote_semantic() {
        assert_eq!(MemoryTier::Semantic.promote(), Some(MemoryTier::Procedural));
    }

    #[test]
    fn test_memory_tier_promote_procedural() {
        assert_eq!(MemoryTier::Procedural.promote(), None);
    }

    #[test]
    fn test_memory_tier_as_str() {
        assert_eq!(MemoryTier::Working.as_str(), "working");
        assert_eq!(MemoryTier::Episodic.as_str(), "episodic");
        assert_eq!(MemoryTier::Semantic.as_str(), "semantic");
        assert_eq!(MemoryTier::Procedural.as_str(), "procedural");
    }

    #[test]
    fn test_memory_lifecycle_new() {
        let lc = MemoryLifecycle::new(0.8);
        assert!((lc.importance - 0.8).abs() < 1e-9);
        assert_eq!(lc.access_count, 0);
    }

    #[test]
    fn test_memory_lifecycle_with_ttl() {
        let lc = MemoryLifecycle::with_ttl(0.5, 3600);
        assert_eq!(lc.ttl_seconds, Some(3600));
    }

    #[test]
    fn test_memory_lifecycle_with_confidence() {
        let lc = MemoryLifecycle::new(0.6).with_confidence(0.9);
        assert!((lc.confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_memory_lifecycle_touch() {
        let mut lc = MemoryLifecycle::new(0.5);
        lc.touch();
        assert_eq!(lc.access_count, 1);
    }

    #[test]
    fn test_memory_lifecycle_not_expired_without_ttl() {
        let lc = MemoryLifecycle::new(0.5);
        assert!(!lc.is_expired());
    }

    #[test]
    fn test_memory_tier_partial_eq() {
        assert_eq!(MemoryTier::Working, MemoryTier::Working);
        assert_ne!(MemoryTier::Working, MemoryTier::Semantic);
    }
}
