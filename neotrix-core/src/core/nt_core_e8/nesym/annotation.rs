#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FidelityLevel {
    Exact,
    Approximate(f64),
    Abstracted,
    Learned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityAnnotation {
    pub level: FidelityLevel,
    pub confidence: f64,
    pub source: String,
    pub timestamp: u64,
}

impl FidelityAnnotation {
    pub fn new(level: FidelityLevel, confidence: f64, source: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            level,
            confidence: confidence.max(0.0).min(1.0),
            source,
            timestamp,
        }
    }

    pub fn is_reliable(&self) -> bool {
        matches!(self.level, FidelityLevel::Exact) || self.confidence > 0.7
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFidelityAnnotation {
    pub annotations: Vec<FidelityAnnotation>,
}

impl MultiFidelityAnnotation {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
        }
    }

    pub fn add(&mut self, annotation: FidelityAnnotation) {
        self.annotations.push(annotation);
    }

    pub fn aggregate_confidence(&self) -> f64 {
        if self.annotations.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.annotations.iter().map(|a| a.confidence).sum();
        sum / self.annotations.len() as f64
    }

    pub fn best_annotation(&self) -> Option<&FidelityAnnotation> {
        self.annotations
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn by_level(&self, level: FidelityLevel) -> Vec<&FidelityAnnotation> {
        self.annotations.iter().filter(|a| a.level == level).collect()
    }

    pub fn count(&self) -> usize {
        self.annotations.len()
    }
}

impl Default for MultiFidelityAnnotation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fidelity_annotation_new() {
        let ann = FidelityAnnotation::new(FidelityLevel::Exact, 1.0, "test".to_string());
        assert_eq!(ann.level, FidelityLevel::Exact);
        assert!(ann.timestamp > 0);
    }

    #[test]
    fn test_reliable_exact() {
        let ann = FidelityAnnotation::new(FidelityLevel::Exact, 0.5, "source".to_string());
        assert!(ann.is_reliable());
    }

    #[test]
    fn test_reliable_high_confidence() {
        let ann = FidelityAnnotation::new(FidelityLevel::Approximate(0.9), 0.95, "source".to_string());
        assert!(ann.is_reliable());
    }

    #[test]
    fn test_not_reliable() {
        let ann = FidelityAnnotation::new(FidelityLevel::Learned, 0.5, "source".to_string());
        assert!(!ann.is_reliable());
    }

    #[test]
    fn test_multi_fidelity_add_and_count() {
        let mut multi = MultiFidelityAnnotation::new();
        assert_eq!(multi.count(), 0);
        multi.add(FidelityAnnotation::new(FidelityLevel::Exact, 1.0, "src1".to_string()));
        multi.add(FidelityAnnotation::new(FidelityLevel::Learned, 0.6, "src2".to_string()));
        assert_eq!(multi.count(), 2);
    }

    #[test]
    fn test_aggregate_confidence() {
        let mut multi = MultiFidelityAnnotation::new();
        multi.add(FidelityAnnotation::new(FidelityLevel::Exact, 1.0, "a".to_string()));
        multi.add(FidelityAnnotation::new(FidelityLevel::Approximate(0.8), 0.5, "b".to_string()));
        let avg = multi.aggregate_confidence();
        assert!((avg - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_best_annotation() {
        let mut multi = MultiFidelityAnnotation::new();
        multi.add(FidelityAnnotation::new(FidelityLevel::Approximate(0.8), 0.5, "low".to_string()));
        multi.add(FidelityAnnotation::new(FidelityLevel::Exact, 1.0, "high".to_string()));
        let best = multi.best_annotation();
        assert!(best.is_some());
        assert_eq!(best.unwrap().source, "high");
    }

    #[test]
    fn test_by_level() {
        let mut multi = MultiFidelityAnnotation::new();
        multi.add(FidelityAnnotation::new(FidelityLevel::Exact, 1.0, "src1".to_string()));
        multi.add(FidelityAnnotation::new(FidelityLevel::Learned, 0.6, "src2".to_string()));
        multi.add(FidelityAnnotation::new(FidelityLevel::Exact, 0.9, "src3".to_string()));
        let exact = multi.by_level(FidelityLevel::Exact);
        assert_eq!(exact.len(), 2);
        let learned = multi.by_level(FidelityLevel::Learned);
        assert_eq!(learned.len(), 1);
    }

    #[test]
    fn test_empty_aggregate() {
        let multi = MultiFidelityAnnotation::new();
        assert!((multi.aggregate_confidence() - 0.0).abs() < 1e-9);
    }
}
