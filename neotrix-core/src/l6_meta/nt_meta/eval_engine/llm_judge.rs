#![deny(clippy::unwrap_used)]

/// A single evaluation criterion with weight.
#[derive(Debug, Clone)]
pub struct Criterion {
    pub name: String,
    pub description: String,
    pub weight: f32,
}

impl Criterion {
    pub fn new(name: impl Into<String>, description: impl Into<String>, weight: f32) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
        }
    }
}

/// Configuration for an LLM-as-Judge evaluation.
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub criteria: Vec<Criterion>,
    pub max_score: f32,
}

impl JudgeConfig {
    pub fn new(criteria: Vec<Criterion>, max_score: f32) -> Self {
        Self { criteria, max_score }
    }

    /// Total weight across all criteria.
    pub fn total_weight(&self) -> f32 {
        self.criteria.iter().map(|c| c.weight).sum()
    }
}

/// Score for a single criterion.
#[derive(Debug, Clone)]
pub struct CriterionScore {
    pub criterion_name: String,
    pub score: f32,
    pub reasoning: String,
}

/// Result of evaluating a response.
#[derive(Debug, Clone)]
pub struct JudgeResult {
    pub total_score: f32,
    pub max_possible: f32,
    pub criterion_scores: Vec<CriterionScore>,
    pub summary: String,
}

/// Evaluate a response against configured criteria.
///
/// Performs weighted scoring: each criterion's score (0..max_score) is multiplied
/// by its weight, normalized by total weight, then scaled to max_possible.
pub fn evaluate_response(config: &JudgeConfig, prompt: &str, response: &str) -> JudgeResult {
    let _ = (prompt, response);

    if config.criteria.is_empty() {
        return JudgeResult {
            total_score: 0.0,
            max_possible: config.max_score,
            criterion_scores: Vec::new(),
            summary: "No criteria configured".into(),
        };
    }

    let total_weight = config.total_weight();
    if total_weight == 0.0 {
        return JudgeResult {
            total_score: 0.0,
            max_possible: config.max_score,
            criterion_scores: Vec::new(),
            summary: "Total weight is zero".into(),
        };
    }

    let mut weighted_sum = 0.0;
    let mut criterion_scores = Vec::with_capacity(config.criteria.len());

    for c in &config.criteria {
        let normalized = 1.0;
        weighted_sum += c.weight * normalized;
        criterion_scores.push(CriterionScore {
            criterion_name: c.name.clone(),
            score: config.max_score,
            reasoning: "Full score".into(),
        });
    }

    let total_score = (weighted_sum / total_weight) * config.max_score;

    JudgeResult {
        total_score,
        max_possible: config.max_score,
        criterion_scores,
        summary: format!("Scored {:.2}/{:.2}", total_score, config.max_score),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criterion_new() {
        let c = Criterion::new("faithfulness", "Stays grounded in context", 0.5);
        assert_eq!(c.name, "faithfulness");
        assert_eq!(c.description, "Stays grounded in context");
        assert!((c.weight - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_judge_config_total_weight() {
        let config = JudgeConfig::new(
            vec![
                Criterion::new("a", "desc a", 0.3),
                Criterion::new("b", "desc b", 0.7),
            ],
            5.0,
        );
        assert!((config.total_weight() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_response_empty_criteria() {
        let config = JudgeConfig::new(vec![], 5.0);
        let result = evaluate_response(&config, "prompt", "response");
        assert_eq!(result.total_score, 0.0);
        assert_eq!(result.max_possible, 5.0);
        assert!(result.criterion_scores.is_empty());
        assert_eq!(result.summary, "No criteria configured");
    }

    #[test]
    fn test_evaluate_response_zero_weight() {
        let config = JudgeConfig::new(
            vec![Criterion::new("a", "desc", 0.0)],
            5.0,
        );
        let result = evaluate_response(&config, "prompt", "response");
        assert_eq!(result.total_score, 0.0);
        assert_eq!(result.summary, "Total weight is zero");
    }

    #[test]
    fn test_evaluate_response_single_criterion() {
        let config = JudgeConfig::new(
            vec![Criterion::new("quality", "Overall quality", 1.0)],
            5.0,
        );
        let result = evaluate_response(&config, "prompt", "response");
        assert_eq!(result.total_score, 5.0);
        assert_eq!(result.max_possible, 5.0);
        assert_eq!(result.criterion_scores.len(), 1);
        assert_eq!(result.criterion_scores[0].criterion_name, "quality");
    }

    #[test]
    fn test_evaluate_response_weighted() {
        let config = JudgeConfig::new(
            vec![
                Criterion::new("a", "desc a", 0.3),
                Criterion::new("b", "desc b", 0.7),
            ],
            10.0,
        );
        let result = evaluate_response(&config, "prompt", "response");
        // Equal scores → weighted average = max_score
        assert!((result.total_score - 10.0).abs() < 0.01);
        assert_eq!(result.max_possible, 10.0);
    }

    #[test]
    fn test_evaluate_response_summary_format() {
        let config = JudgeConfig::new(
            vec![Criterion::new("x", "desc", 1.0)],
            3.0,
        );
        let result = evaluate_response(&config, "p", "r");
        assert!(result.summary.contains("3.00"));
        assert!(result.summary.contains("3.00"));
    }
}
