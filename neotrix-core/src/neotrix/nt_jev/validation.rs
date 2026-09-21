//! JEV Validation — probability mass validation and input checks

use std::collections::HashMap;
use super::primitives::*;

/// Validate probability distribution: total mass must be within tolerance of 1.0
///
/// Returns Ok(()) if valid, Err(message) if invalid.
pub fn validate_probabilities(probs: &HashMap<String, f64>, tolerance: f64) -> Result<(), String> {
    let total: f64 = probs.values().sum();
    let error = (total - 1.0).abs();
    if error > tolerance {
        return Err(format!(
            "probability mass error {:.4} > tolerance {:.4} (total={:.4})",
            error, tolerance, total
        ));
    }

    // Check for negative probabilities
    for (k, v) in probs {
        if *v < 0.0 {
            return Err(format!("negative probability for '{}': {}", k, v));
        }
    }

    Ok(())
}

/// Maximum options/levels per question — the hosted Jev cardinality limit.
///
/// Upstream supports up to 255 options per Choice (2-stage score-then-choose
/// at high cardinality). Local backends should truncate long before this
/// (jev-browser caps at 240 elements per step and says so in state).
pub const JEV_MAX_OPTIONS: usize = 255;

/// Validate option count against [`JEV_MAX_OPTIONS`].
pub fn validate_cardinality(count: usize) -> Result<(), String> {
    if count > JEV_MAX_OPTIONS {
        return Err(format!(
            "option count {} exceeds Jev limit {} — split or truncate",
            count, JEV_MAX_OPTIONS
        ));
    }
    Ok(())
}

/// Validate a NoulAnswer
pub fn validate_noul(answer: &NoulAnswer) -> Vec<String> {
    let mut errors = Vec::new();
    if answer.noul < 0.0 || answer.noul > 1.0 {
        errors.push(format!("noul value {} out of range [0, 1]", answer.noul));
    }
    errors
}

/// Validate a ChoiceAnswer
pub fn validate_choice(answer: &ChoiceAnswer) -> Vec<String> {
    let mut errors = Vec::new();

    if answer.choice.is_empty() {
        errors.push("choice is empty".into());
    }

    if answer.probabilities.is_empty() {
        errors.push("probabilities is empty".into());
    } else if let Err(e) = validate_probabilities(&answer.probabilities, 0.05) {
        errors.push(e);
    }

    if let Err(e) = validate_cardinality(answer.probabilities.len()) {
        errors.push(e);
    }

    if answer.confidence < 0.0 || answer.confidence > 1.0 {
        errors.push(format!("confidence {} out of range [0, 1]", answer.confidence));
    }

    if answer.margin < 0.0 || answer.margin > 1.0 {
        errors.push(format!("margin {} out of range [0, 1]", answer.margin));
    }

    errors
}

/// Validate a ScoreAnswer
pub fn validate_score(answer: &ScoreAnswer) -> Vec<String> {
    let mut errors = Vec::new();

    if answer.legend.is_empty() {
        errors.push("legend is empty".into());
    } else if answer.score < 0.0 || answer.score >= answer.legend.len() as f64 {
        errors.push(format!(
            "score {} out of range [0, {})",
            answer.score,
            answer.legend.len()
        ));
    }

    if !answer.probabilities.is_empty() {
        if let Err(e) = validate_probabilities(&answer.probabilities, 0.05) {
            errors.push(e);
        }
    }

    if let Err(e) = validate_cardinality(answer.legend.len()) {
        errors.push(e);
    }

    if answer.confidence < 0.0 || answer.confidence > 1.0 {
        errors.push(format!("confidence {} out of range [0, 1]", answer.confidence));
    }

    errors
}

/// Validate a JevDecision
pub fn validate_decision(decision: &JevDecision) -> Vec<String> {
    match decision {
        JevDecision::Noul(n) => validate_noul(n),
        JevDecision::Choice(c) => validate_choice(c),
        JevDecision::Score(s) => validate_score(s),
    }
}

/// Validate an entire result set
pub fn validate_result_set(decisions: &JevResultSet) -> HashMap<String, Vec<String>> {
    let mut all_errors = HashMap::new();
    for (id, decision) in decisions {
        let errors = validate_decision(decision);
        if !errors.is_empty() {
            all_errors.insert(id.clone(), errors);
        }
    }
    all_errors
}

/// Dry-run validation — check inputs without running the model
pub fn dry_run_validate(
    state: &str,
    questions: &[(String, String)], // (id, question_type)
) -> Vec<String> {
    let mut errors = Vec::new();

    if state.trim().is_empty() {
        errors.push("state is empty".into());
    }

    if questions.is_empty() {
        errors.push("no questions provided".into());
    }

    for (id, _qt) in questions {
        if id.trim().is_empty() {
            errors.push("question has empty ID".into());
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_probabilities_ok() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.6);
        probs.insert("b".to_string(), 0.4);
        assert!(validate_probabilities(&probs, 0.05).is_ok());
    }

    #[test]
    fn test_validate_probabilities_error() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.8);
        probs.insert("b".to_string(), 0.3);
        assert!(validate_probabilities(&probs, 0.05).is_err());
    }

    #[test]
    fn test_validate_probabilities_negative() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), -0.1);
        probs.insert("b".to_string(), 1.1);
        assert!(validate_probabilities(&probs, 0.2).is_err());
    }

    #[test]
    fn test_validate_noul_ok() {
        let n = NoulAnswer { noul: 0.7, needs_review: false, reason: None, status: DecisionStatus::Selected };
        assert!(validate_noul(&n).is_empty());
    }

    #[test]
    fn test_validate_noul_out_of_range() {
        let n = NoulAnswer { noul: 1.5, needs_review: false, reason: None, status: DecisionStatus::Selected };
        assert!(!validate_noul(&n).is_empty());
    }

    #[test]
    fn test_validate_choice_ok() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.7);
        probs.insert("b".to_string(), 0.3);
        let c = ChoiceAnswer {
            choice: "a".into(),
            probabilities: probs,
            confidence: 0.6,
            margin: 0.4,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        assert!(validate_choice(&c).is_empty());
    }

    #[test]
    fn test_validate_choice_empty() {
        let c = ChoiceAnswer {
            choice: "".into(),
            probabilities: HashMap::new(),
            confidence: 0.0,
            margin: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        assert!(!validate_choice(&c).is_empty());
    }

    #[test]
    fn test_validate_score_ok() {
        let mut probs = HashMap::new();
        probs.insert("0".to_string(), 0.8);
        probs.insert("1".to_string(), 0.2);
        let s = ScoreAnswer {
            score: 0.0,
            probabilities: probs,
            confidence: 0.6,
            legend: vec!["low".into(), "high".into()],
            needs_review: false,
            reason: None,
            status: DecisionStatus::Scored,
        };
        assert!(validate_score(&s).is_empty());
    }

    #[test]
    fn test_validate_score_out_of_range() {
        let s = ScoreAnswer {
            score: 5.0,
            probabilities: HashMap::new(),
            confidence: 0.0,
            legend: vec!["low".into(), "high".into()],
            needs_review: false,
            reason: None,
            status: DecisionStatus::Scored,
        };
        assert!(!validate_score(&s).is_empty());
    }

    #[test]
    fn test_dry_run_validate() {
        let errors = dry_run_validate("test state", &[("q1".into(), "noul".into())]);
        assert!(errors.is_empty());

        let errors = dry_run_validate("", &[("q1".into(), "noul".into())]);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_cardinality_ok() {
        assert!(validate_cardinality(0).is_ok());
        assert!(validate_cardinality(255).is_ok());
    }

    #[test]
    fn test_validate_cardinality_over_limit() {
        let err = validate_cardinality(256).unwrap_err();
        assert!(err.contains("255"), "got: {}", err);
    }

    #[test]
    fn test_validate_choice_cardinality() {
        // 256 options each with tiny mass — mass check may pass, count must fail.
        let mut probs = HashMap::new();
        for i in 0..256 {
            probs.insert(format!("opt{i}"), 1.0 / 256.0);
        }
        let c = ChoiceAnswer {
            choice: "opt0".into(),
            probabilities: probs,
            confidence: 0.0,
            margin: 0.0,
            needs_review: true,
            reason: None,
            status: DecisionStatus::Review,
        };
        let errors = validate_choice(&c);
        assert!(
            errors.iter().any(|e| e.contains("255")),
            "expected cardinality error, got: {:?}",
            errors
        );
    }
}
