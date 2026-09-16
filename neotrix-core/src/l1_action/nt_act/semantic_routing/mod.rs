//! Semantic Pattern Routing — NT-ACT
//!
//! Routes to the nearest skill based on behavior patterns.
//! Supports fan-in queues, policy traces, trust boundaries.
//!
//! Domain: NT-ACT (行动执行者)
//! Layer: L1 Action

pub mod pattern;
pub mod router;
pub mod catalog;

pub use pattern::BehaviorPattern;
pub use router::PatternRouter;
pub use catalog::PatternCatalog;

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Semantic similarity score
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SimilarityScore {
    pub value: f64,
    pub method: String,
}

/// Trust boundary level for skill execution
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustBoundary {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Policy trace for tracking routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTrace {
    pub skill_id: String,
    pub pattern_id: String,
    pub confidence: f64,
    pub trust_boundary: TrustBoundary,
    pub timestamp: u64,
    pub path: Vec<String>,
}

/// Fan-in queue for batching similar pattern requests
#[derive(Debug, Clone)]
pub struct FanInQueue {
    pub queue_id: String,
    pub pattern_type: String,
    pub pending_requests: Vec<PatternRequest>,
    pub max_batch_size: usize,
    pub timeout_ms: u64,
}

/// Individual pattern request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRequest {
    pub query: String,
    pub context: HashMap<String, String>,
    pub priority: u32,
    pub timestamp: u64,
}

/// Result of a pattern routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub skill_id: String,
    pub skill_name: String,
    pub score: SimilarityScore,
    pub policy_trace: PolicyTrace,
    pub references_loaded: Vec<String>,
    pub scripts_loaded: Vec<String>,
}

/// Default behavior pattern types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorPatternType {
    /// Sequential task execution
    Sequential,
    /// Parallel task execution
    Parallel,
    /// Iterative refinement
    Iterative,
    /// Hierarchical decomposition
    Hierarchical,
    /// Reactive event-driven
    Reactive,
    /// Goal-oriented planning
    GoalOriented,
    /// Knowledge retrieval
    KnowledgeRetrieval,
    /// Code generation
    CodeGeneration,
    /// Research and analysis
    Research,
    /// Creative generation
    Creative,
}

/// Fan-in queue statistics
#[derive(Debug, Clone, Default)]
pub struct FanInStats {
    pub total_queues: usize,
    pub total_pending: usize,
    pub processed_batches: usize,
    pub avg_batch_size: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_similarity_ordering() {
        let s1 = SimilarityScore { value: 0.8, method: "cosine".to_string() };
        let s2 = SimilarityScore { value: 0.5, method: "keyword".to_string() };
        assert!(s1 > s2);
    }

    #[test]
    fn test_trust_boundary_ordering() {
        assert!(TrustBoundary::Public < TrustBoundary::Internal);
        assert!(TrustBoundary::Internal < TrustBoundary::Confidential);
        assert!(TrustBoundary::Confidential < TrustBoundary::Restricted);
    }

    #[test]
    fn test_policy_trace() {
        let trace = PolicyTrace {
            skill_id: "nt-act-test".to_string(),
            pattern_id: "pattern-001".to_string(),
            confidence: 0.9,
            trust_boundary: TrustBoundary::Internal,
            timestamp: 1000,
            path: vec!["step1".to_string(), "step2".to_string()],
        };
        assert_eq!(trace.skill_id, "nt-act-test");
        assert_eq!(trace.confidence, 0.9);
    }

    #[test]
    fn test_fan_in_queue() {
        let queue = FanInQueue {
            queue_id: "q1".to_string(),
            pattern_type: "Research".to_string(),
            pending_requests: vec![
                PatternRequest {
                    query: "test query".to_string(),
                    context: HashMap::new(),
                    priority: 1,
                    timestamp: 1000,
                }
            ],
            max_batch_size: 10,
            timeout_ms: 5000,
        };
        assert_eq!(queue.pending_requests.len(), 1);
        assert_eq!(queue.max_batch_size, 10);
    }

    #[test]
    fn test_route_result() {
        let result = RouteResult {
            skill_id: "nt-act-test".to_string(),
            skill_name: "Test Skill".to_string(),
            score: SimilarityScore { value: 0.85, method: "cosine".to_string() },
            policy_trace: PolicyTrace {
                skill_id: "nt-act-test".to_string(),
                pattern_id: "p1".to_string(),
                confidence: 0.85,
                trust_boundary: TrustBoundary::Internal,
                timestamp: 1000,
                path: vec![],
            },
            references_loaded: vec![],
            scripts_loaded: vec![],
        };
        assert_eq!(result.score.value, 0.85);
    }
}