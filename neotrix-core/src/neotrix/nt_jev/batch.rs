//! Batch execution pattern (from classifier.dev).
//!
//! Classify many items in a single Jev request with token budgeting,
//! uncertain-item escalation, and per-item observability.

use super::primitives::*;
use super::gate::GateQuestion;
#[cfg(test)]
use super::gate::GateQuestionType;
use super::validation::validate_result_set;

/// A single item in a batch to classify.
#[derive(Debug, Clone)]
pub struct BatchItem {
    /// Unique identifier for this item.
    pub id: String,
    /// The state/context for this item.
    pub state: String,
    /// Questions to ask about this item.
    pub questions: Vec<GateQuestion>,
}

/// Result of processing a single item in a batch.
#[derive(Debug, Clone)]
pub struct BatchItemResult {
    /// The item ID.
    pub id: String,
    /// The decisions for this item.
    pub decisions: JevResultSet,
    /// Whether this item needs escalation (low confidence or validation error).
    pub needs_escalation: bool,
    /// The confidence of the primary answer.
    pub confidence: f64,
}

/// Configuration for batch processing.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of items per batch (default 100).
    pub max_batch_size: usize,
    /// Confidence threshold below which items are escalated (default 0.5).
    pub escalation_threshold: f64,
    /// Maximum tokens per batch (default 50000).
    ///
    /// TypeSafe Jev context contract: state + questions ≤ 64K tokens total,
    /// state + longest single question ≤ 32K. Default leaves headroom.
    pub max_tokens_per_batch: usize,
    /// Estimated tokens per question (default 50).
    pub tokens_per_question: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            escalation_threshold: 0.5,
            max_tokens_per_batch: 50_000,
            tokens_per_question: 50,
        }
    }
}

/// Split a batch of items into chunks that fit within token budget.
pub fn split_batch(items: &[BatchItem], config: &BatchConfig) -> Vec<Vec<BatchItem>> {
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();
    let mut current_tokens = 0;

    for item in items {
        let item_tokens: usize = item.questions.len() * config.tokens_per_question;
        if !current_chunk.is_empty()
            && (current_chunk.len() >= config.max_batch_size
                || current_tokens + item_tokens > config.max_tokens_per_batch)
        {
            chunks.push(std::mem::take(&mut current_chunk));
            current_tokens = 0;
        }
        current_tokens += item_tokens;
        current_chunk.push(item.clone());
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Compute the confidence of a result set (highest individual confidence).
pub fn batch_confidence(decisions: &JevResultSet) -> f64 {
    decisions
        .values()
        .map(|d| d.confidence())
        .fold(0.0f64, f64::max)
}

/// Process a batch item: validate, check confidence, decide if escalation needed.
pub fn process_batch_item(
    item: &BatchItem,
    decisions: JevResultSet,
    config: &BatchConfig,
) -> BatchItemResult {
    let validation_errors = validate_result_set(&decisions);

    if validation_errors.values().any(|e| !e.is_empty()) {
        return BatchItemResult {
            id: item.id.clone(),
            decisions,
            needs_escalation: true,
            confidence: 0.0,
        };
    }

    let confidence = batch_confidence(&decisions);
    let needs_escalation = confidence < config.escalation_threshold
        || decisions.values().any(|d| d.needs_review());

    BatchItemResult {
        id: item.id.clone(),
        decisions,
        needs_escalation,
        confidence,
    }
}

/// Summary statistics for a batch run.
#[derive(Debug, Clone)]
pub struct BatchSummary {
    /// Total items processed.
    pub total: usize,
    /// Items that needed escalation.
    pub escalated: usize,
    /// Items that were confident.
    pub confident: usize,
    /// Average confidence across all items.
    pub avg_confidence: f64,
    /// Per-item results.
    pub results: Vec<BatchItemResult>,
}

/// Compute batch summary from processed items.
pub fn batch_summary(results: Vec<BatchItemResult>) -> BatchSummary {
    let total = results.len();
    let escalated = results.iter().filter(|r| r.needs_escalation).count();
    let confident = total - escalated;
    let avg_confidence = if total > 0 {
        results.iter().map(|r| r.confidence).sum::<f64>() / total as f64
    } else {
        0.0
    };

    BatchSummary {
        total,
        escalated,
        confident,
        avg_confidence,
        results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_question(id: &str) -> GateQuestion {
        GateQuestion {
            id: id.to_string(),
            question_type: GateQuestionType::Noul {
                instructions: "Is this OK?".to_string(),
            },
            priority: 0,
        }
    }

    fn make_item(id: &str) -> BatchItem {
        BatchItem {
            id: id.to_string(),
            state: "test state".to_string(),
            questions: vec![make_question("q")],
        }
    }

    fn noul_decision(noul: f64, needs_review: bool) -> JevDecision {
        JevDecision::Noul(NoulAnswer {
            noul,
            needs_review,
            reason: if needs_review {
                Some("low confidence".to_string())
            } else {
                None
            },
            status: if needs_review {
                DecisionStatus::Review
            } else {
                DecisionStatus::Selected
            },
        })
    }

    #[test]
    fn test_split_batch_by_size() {
        let items: Vec<BatchItem> = (0..10).map(|i| make_item(&format!("item_{i}"))).collect();
        let config = BatchConfig {
            max_batch_size: 3,
            ..Default::default()
        };
        let chunks = split_batch(&items, &config);
        assert_eq!(chunks.len(), 4); // 3+3+3+1
        assert_eq!(chunks[0].len(), 3);
        assert_eq!(chunks[3].len(), 1);
    }

    #[test]
    fn test_split_batch_by_tokens() {
        let items: Vec<BatchItem> = (0..5)
            .map(|i| {
                let mut item = make_item(&format!("item_{i}"));
                // 100 questions * 50 tokens = 5000 tokens each
                item.questions = (0..100)
                    .map(|j| make_question(&format!("q{j}")))
                    .collect();
                item
            })
            .collect();
        let config = BatchConfig {
            max_batch_size: 100,
            max_tokens_per_batch: 10_000,
            tokens_per_question: 50,
            ..Default::default()
        };
        let chunks = split_batch(&items, &config);
        // 5000 tokens per item, 10000 budget -> 2 items per chunk
        assert_eq!(chunks.len(), 3); // 2+2+1
    }

    #[test]
    fn test_process_batch_item_confident() {
        let item = make_item("test");
        let mut decisions = JevResultSet::new();
        decisions.insert(
            "q".to_string(),
            noul_decision(0.9, false),
        );
        let config = BatchConfig::default();
        let processed = process_batch_item(&item, decisions, &config);
        assert!(!processed.needs_escalation);
        assert!(processed.confidence > 0.0);
    }

    #[test]
    fn test_process_batch_item_low_confidence() {
        let item = make_item("test");
        let mut decisions = JevResultSet::new();
        decisions.insert(
            "q".to_string(),
            noul_decision(0.3, true),
        );
        let config = BatchConfig {
            escalation_threshold: 0.7,
            ..Default::default()
        };
        let processed = process_batch_item(&item, decisions, &config);
        assert!(processed.needs_escalation);
    }

    #[test]
    fn test_batch_summary() {
        let results = vec![
            BatchItemResult {
                id: "a".to_string(),
                decisions: JevResultSet::new(),
                needs_escalation: false,
                confidence: 0.9,
            },
            BatchItemResult {
                id: "b".to_string(),
                decisions: JevResultSet::new(),
                needs_escalation: true,
                confidence: 0.3,
            },
        ];
        let summary = batch_summary(results);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.escalated, 1);
        assert_eq!(summary.confident, 1);
        assert!((summary.avg_confidence - 0.6).abs() < 0.01);
    }
}
