//! Multi-label classification and smart escalation (from classifier.dev).
//!
//! One Jev Noul question per label. Items below confidence threshold
//! are escalated to a more capable (reasoning) model.

use super::primitives::*;
use super::gate::*;

/// A label to classify.
#[derive(Debug, Clone)]
pub struct Label {
    /// Unique label identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// The question to ask Jev about this label.
    pub question: String,
}

/// Classification result for a single label.
#[derive(Debug, Clone)]
pub struct LabelResult {
    /// The label ID.
    pub label_id: String,
    /// Whether this label applies (noul > 0.5).
    pub applies: bool,
    /// Confidence in this judgment.
    pub confidence: f64,
    /// The raw Noul answer.
    pub answer: NoulAnswer,
}

/// Result of multi-label classification for one item.
#[derive(Debug, Clone)]
pub struct MultiLabelResult {
    /// Item identifier.
    pub item_id: String,
    /// Per-label results.
    pub labels: Vec<LabelResult>,
    /// Whether any label had low confidence (needs escalation).
    pub needs_escalation: bool,
    /// Highest confidence across all labels.
    pub max_confidence: f64,
    /// Average confidence across all labels.
    pub avg_confidence: f64,
}

/// Configuration for multi-label classification.
#[derive(Debug, Clone)]
pub struct MultiLabelConfig {
    /// Confidence threshold (default 0.7).
    pub threshold: f64,
    /// If true, labels below threshold are marked for escalation.
    pub escalate_below_threshold: bool,
}

impl Default for MultiLabelConfig {
    fn default() -> Self {
        Self {
            threshold: 0.7,
            escalate_below_threshold: true,
        }
    }
}

/// Build GateQuestions from labels (one Noul per label).
pub fn labels_to_questions(labels: &[Label]) -> Vec<GateQuestion> {
    labels
        .iter()
        .map(|label| GateQuestion {
            id: label.id.clone(),
            question_type: GateQuestionType::Noul {
                instructions: label.question.clone(),
            },
            priority: 1,
        })
        .collect()
}

/// Compute Noul confidence: max(p, 1-p).
fn noul_confidence(noul: f64) -> f64 {
    noul.max(1.0 - noul)
}

/// Parse JevResultSet into per-label results.
pub fn parse_label_results(
    labels: &[Label],
    decisions: &JevResultSet,
    config: &MultiLabelConfig,
) -> MultiLabelResult {
    let item_id = labels
        .first()
        .map(|l| l.id.clone())
        .unwrap_or_default();

    let label_results: Vec<LabelResult> = labels
        .iter()
        .filter_map(|label| {
            decisions.get(&label.id).and_then(|decision| {
                if let JevDecision::Noul(noul) = decision {
                    let confidence = noul_confidence(noul.noul);
                    Some(LabelResult {
                        label_id: label.id.clone(),
                        applies: noul.noul > 0.5,
                        confidence,
                        answer: noul.clone(),
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    let max_confidence = label_results
        .iter()
        .map(|l| l.confidence)
        .fold(0.0f64, f64::max);
    let avg_confidence = if label_results.is_empty() {
        0.0
    } else {
        label_results.iter().map(|l| l.confidence).sum::<f64>() / label_results.len() as f64
    };
    let needs_escalation = config.escalate_below_threshold
        && label_results.iter().any(|l| l.confidence < config.threshold);

    MultiLabelResult {
        item_id,
        labels: label_results,
        needs_escalation,
        max_confidence,
        avg_confidence,
    }
}

/// Items that need escalation (low confidence).
pub fn filter_escalated(results: &[MultiLabelResult]) -> Vec<&MultiLabelResult> {
    results.iter().filter(|r| r.needs_escalation).collect()
}

/// Items that are confident (no escalation needed).
pub fn filter_confident(results: &[MultiLabelResult]) -> Vec<&MultiLabelResult> {
    results.iter().filter(|r| !r.needs_escalation).collect()
}

/// Summary of a multi-label batch run.
#[derive(Debug, Clone)]
pub struct MultiLabelSummary {
    /// Total items classified.
    pub total: usize,
    /// Items with all labels confident.
    pub confident: usize,
    /// Items needing escalation.
    pub escalated: usize,
    /// Average max-confidence across items.
    pub avg_max_confidence: f64,
    /// Per-label statistics: (label_id, avg_confidence, positive_count).
    pub label_stats: Vec<(String, f64, usize)>,
}

/// Compute summary from batch results.
pub fn multi_label_summary(results: Vec<MultiLabelResult>) -> MultiLabelSummary {
    let total = results.len();
    let escalated = results.iter().filter(|r| r.needs_escalation).count();
    let confident = total - escalated;
    let avg_max_confidence = if total > 0 {
        results.iter().map(|r| r.max_confidence).sum::<f64>() / total as f64
    } else {
        0.0
    };

    // Collect all unique label IDs
    let mut label_ids: Vec<String> = Vec::new();
    for result in &results {
        for label in &result.labels {
            if !label_ids.contains(&label.label_id) {
                label_ids.push(label.label_id.clone());
            }
        }
    }

    let label_stats: Vec<(String, f64, usize)> = label_ids
        .iter()
        .map(|id| {
            let label_results: Vec<&LabelResult> = results
                .iter()
                .flat_map(|r| r.labels.iter())
                .filter(|l| l.label_id == *id)
                .collect();
            let count = label_results.len();
            let avg_conf = if count > 0 {
                label_results.iter().map(|l| l.confidence).sum::<f64>() / count as f64
            } else {
                0.0
            };
            let positive = label_results.iter().filter(|l| l.applies).count();
            (id.clone(), avg_conf, positive)
        })
        .collect();

    MultiLabelSummary {
        total,
        confident,
        escalated,
        avg_max_confidence,
        label_stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_labels() -> Vec<Label> {
        vec![
            Label { id: "spam".to_string(), name: "Spam".to_string(), question: "Is this spam?".to_string() },
            Label { id: "offensive".to_string(), name: "Offensive".to_string(), question: "Is this offensive?".to_string() },
            Label { id: "relevant".to_string(), name: "Relevant".to_string(), question: "Is this relevant?".to_string() },
        ]
    }

    #[test]
    fn test_labels_to_questions() {
        let labels = make_labels();
        let questions = labels_to_questions(&labels);
        assert_eq!(questions.len(), 3);
        assert_eq!(questions[0].id, "spam");
        assert!(matches!(questions[0].question_type, GateQuestionType::Noul { .. }));
    }

    #[test]
    fn test_parse_label_results() {
        let labels = make_labels();
        let mut decisions = JevResultSet::new();
        decisions.insert("spam".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        }));
        decisions.insert("offensive".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.3,
            needs_review: true,
            reason: Some("unsure".to_string()),
            status: DecisionStatus::Review,
        }));
        decisions.insert("relevant".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.85,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        }));

        let config = MultiLabelConfig::default();
        let parsed = parse_label_results(&labels, &decisions, &config);
        assert_eq!(parsed.labels.len(), 3);
        // offensive: noul=0.3 → confidence = max(0.3, 0.7) = 0.7, and 0.7 < 0.7
        // is false, so no escalation at the default 0.7 threshold.
        // spam: noul=0.9, confidence = max(0.9, 0.1) = 0.9
        // relevant: noul=0.85, confidence = max(0.85, 0.15) = 0.85
        // So no escalation with threshold 0.7. Let's adjust test.
        assert!(!parsed.needs_escalation);
        assert!((parsed.max_confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_parse_label_results_low_confidence() {
        let labels = make_labels();
        let mut decisions = JevResultSet::new();
        decisions.insert("spam".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.55,
            needs_review: true,
            reason: None,
            status: DecisionStatus::Review,
        }));
        decisions.insert("offensive".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.5,
            needs_review: true,
            reason: None,
            status: DecisionStatus::Review,
        }));
        decisions.insert("relevant".to_string(), JevDecision::Noul(NoulAnswer {
            noul: 0.6,
            needs_review: true,
            reason: None,
            status: DecisionStatus::Review,
        }));

        let config = MultiLabelConfig::default();
        let parsed = parse_label_results(&labels, &decisions, &config);
        // All have confidence < 0.7 (spam=0.55, offensive=0.5, relevant=0.6)
        assert!(parsed.needs_escalation);
        assert!((parsed.max_confidence - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_filter_escalated() {
        let results = vec![
            MultiLabelResult { item_id: "a".to_string(), labels: vec![], needs_escalation: true, max_confidence: 0.3, avg_confidence: 0.3 },
            MultiLabelResult { item_id: "b".to_string(), labels: vec![], needs_escalation: false, max_confidence: 0.9, avg_confidence: 0.9 },
        ];
        let escalated = filter_escalated(&results);
        assert_eq!(escalated.len(), 1);
        assert_eq!(escalated[0].item_id, "a");
    }

    #[test]
    fn test_multi_label_summary() {
        let results = vec![
            MultiLabelResult { item_id: "a".to_string(), labels: vec![], needs_escalation: false, max_confidence: 0.9, avg_confidence: 0.8 },
            MultiLabelResult { item_id: "b".to_string(), labels: vec![], needs_escalation: true, max_confidence: 0.4, avg_confidence: 0.3 },
        ];
        let summary = multi_label_summary(results);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.confident, 1);
        assert_eq!(summary.escalated, 1);
    }

    #[test]
    fn test_multi_label_summary_empty() {
        let summary = multi_label_summary(vec![]);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.confident, 0);
        assert_eq!(summary.escalated, 0);
        assert!((summary.avg_max_confidence - 0.0).abs() < 0.01);
    }
}
