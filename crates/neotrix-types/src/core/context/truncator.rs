use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TruncationStrategy {
    OldestFirst(usize),
    SummarizeMiddle(usize),
    ImportanceBased(f64),
}

impl TruncationStrategy {
    pub fn apply(&self, messages: &[String]) -> Vec<String> {
        match self {
            TruncationStrategy::OldestFirst(keep) => {
                if messages.len() <= *keep {
                    messages.to_vec()
                } else {
                    messages[messages.len() - *keep..].to_vec()
                }
            }
            TruncationStrategy::SummarizeMiddle(window) => {
                if messages.len() <= *window + 2 {
                    return messages.to_vec();
                }
                let mut result = Vec::new();
                let head = *window.min(&messages.len().saturating_sub(2));
                result.extend_from_slice(&messages[..head]);
                result.push(format!("[.. truncated {} messages ..]", messages.len() - head - *window));
                let tail_start = messages.len().saturating_sub(*window);
                if tail_start > head {
                    result.extend_from_slice(&messages[tail_start..]);
                }
                result
            }
            TruncationStrategy::ImportanceBased(threshold) => {
                messages
                    .iter()
                    .enumerate()
                    .filter(|(i, msg)| {
                        let score = importance_score(msg, *i, messages.len());
                        score >= *threshold
                    })
                    .map(|(_, msg)| msg.clone())
                    .collect()
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            TruncationStrategy::OldestFirst(_) => "oldest_first",
            TruncationStrategy::SummarizeMiddle(_) => "summarize_middle",
            TruncationStrategy::ImportanceBased(_) => "importance_based",
        }
    }
}

fn importance_score(msg: &str, index: usize, total: usize) -> f64 {
    let length_factor = (msg.len() as f64).ln_1p().min(1.0);
    let recency_factor = if total > 1 {
        index as f64 / (total - 1) as f64
    } else {
        1.0
    };
    let question_indicator = if msg.contains('?') { 0.2 } else { 0.0 };
    let code_indicator = if msg.contains("```") || msg.contains("fn ") || msg.contains("impl ") { 0.3 } else { 0.0 };
    0.2 * length_factor + 0.4 * recency_factor + 0.2 * question_indicator + 0.2 * code_indicator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oldest_first_keeps_all_when_under_limit() {
        let msgs = vec!["a".into(), "b".into(), "c".into()];
        let result = TruncationStrategy::OldestFirst(5).apply(&msgs);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_oldest_first_truncates() {
        let msgs: Vec<String> = (0..10).map(|i| format!("msg-{}", i)).collect();
        let result = TruncationStrategy::OldestFirst(3).apply(&msgs);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "msg-7");
        assert_eq!(result[2], "msg-9");
    }

    #[test]
    fn test_summarize_middle_small() {
        let msgs: Vec<String> = (0..3).map(|i| format!("msg-{}", i)).collect();
        let result = TruncationStrategy::SummarizeMiddle(5).apply(&msgs);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_summarize_middle_truncates() {
        let msgs: Vec<String> = (0..20).map(|i| format!("msg-{}", i)).collect();
        let result = TruncationStrategy::SummarizeMiddle(3).apply(&msgs);
        assert!(result.len() < 20);
        assert!(result.iter().any(|m| m.contains("[.. truncated")));
        assert_eq!(result[0], "msg-0");
    }

    #[test]
    fn test_importance_based_filters() {
        let msgs = vec![
            "short".into(),
            "what is the answer? this is a question that needs answering".into(),
            "fn compute(x: i32) -> i32 { x + 1 }".into(),
            "ok".into(),
        ];
        let result = TruncationStrategy::ImportanceBased(0.5).apply(&msgs);
        assert!(result.len() < msgs.len());
    }

    #[test]
    fn test_strategy_names() {
        assert_eq!(TruncationStrategy::OldestFirst(5).name(), "oldest_first");
        assert_eq!(TruncationStrategy::SummarizeMiddle(5).name(), "summarize_middle");
        assert_eq!(TruncationStrategy::ImportanceBased(0.5).name(), "importance_based");
    }

    #[test]
    fn test_importance_score_bounds() {
        let score = importance_score("test", 0, 1);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_importance_score_code_indicator() {
        let code_score = importance_score("fn main() { }", 0, 2);
        let plain_score = importance_score("hello", 0, 2);
        assert!(code_score > plain_score);
    }

    #[test]
    fn test_importance_based_returns_all_for_low_threshold() {
        let msgs: Vec<String> = (0..5).map(|i| format!("msg-{}", i)).collect();
        let result = TruncationStrategy::ImportanceBased(0.0).apply(&msgs);
        assert_eq!(result.len(), 5);
    }
}
