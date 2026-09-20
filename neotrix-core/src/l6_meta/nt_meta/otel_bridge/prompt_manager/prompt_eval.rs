#![deny(clippy::unwrap_used)]

use super::prompt_registry::PromptRegistry;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub input: String,
    pub expected_keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalResult {
    pub pass_rate: f64,
    pub total_cases: usize,
    pub failed_cases: usize,
}

#[derive(Debug, Default)]
pub struct PromptEval;

impl PromptEval {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        registry: &PromptRegistry,
        prompt_id: &str,
        test_cases: Vec<TestCase>,
    ) -> EvalResult {
        if test_cases.is_empty() {
            return EvalResult {
                pass_rate: 1.0,
                total_cases: 0,
                failed_cases: 0,
            };
        }

        let prompt = registry.get_latest(prompt_id);

        let failed = match prompt {
            Some(p) => test_cases
                .iter()
                .filter(|tc| {
                    let rendered = p.template.to_lowercase();
                    !tc.expected_keywords
                        .iter()
                        .any(|kw| rendered.contains(&kw.to_lowercase()))
                })
                .count(),
            None => test_cases.len(),
        };

        let total = test_cases.len();
        EvalResult {
            pass_rate: (total - failed) as f64 / total as f64,
            total_cases: total,
            failed_cases: failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l6_meta::nt_meta::otel_bridge::prompt_manager::prompt_version::PromptVersion;

    fn make_prompt(id: &str, template: &str) -> PromptVersion {
        PromptVersion::new(
            id.into(),
            "test_prompt".into(),
            1,
            template.into(),
            vec![],
            1000,
        )
    }

    #[test]
    fn test_evaluate_all_pass() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("p1", "The sky is blue and birds fly"));
        let eval = PromptEval::new();
        let cases = vec![
            TestCase {
                input: "color".into(),
                expected_keywords: vec!["blue".into()],
            },
            TestCase {
                input: "birds".into(),
                expected_keywords: vec!["fly".into()],
            },
        ];
        let result = eval.evaluate(&reg, "p1", cases);
        assert_eq!(result.pass_rate, 1.0);
        assert_eq!(result.failed_cases, 0);
    }

    #[test]
    fn test_evaluate_partial_pass() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("p1", "The sky is blue"));
        let eval = PromptEval::new();
        let cases = vec![
            TestCase {
                input: "a".into(),
                expected_keywords: vec!["blue".into()],
            },
            TestCase {
                input: "b".into(),
                expected_keywords: vec!["red".into()],
            },
        ];
        let result = eval.evaluate(&reg, "p1", cases);
        assert!((result.pass_rate - 0.5).abs() < f64::EPSILON);
        assert_eq!(result.failed_cases, 1);
    }

    #[test]
    fn test_evaluate_no_prompt() {
        let reg = PromptRegistry::new();
        let eval = PromptEval::new();
        let cases = vec![TestCase {
            input: "a".into(),
            expected_keywords: vec!["x".into()],
        }];
        let result = eval.evaluate(&reg, "nonexistent", cases);
        assert_eq!(result.pass_rate, 0.0);
        assert_eq!(result.failed_cases, 1);
    }

    #[test]
    fn test_evaluate_empty_cases() {
        let reg = PromptRegistry::new();
        let eval = PromptEval::new();
        let result = eval.evaluate(&reg, "p1", vec![]);
        assert_eq!(result.pass_rate, 1.0);
        assert_eq!(result.total_cases, 0);
    }

    #[test]
    fn test_evaluate_case_insensitive_keywords() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("p1", "The SKY is Blue"));
        let eval = PromptEval::new();
        let cases = vec![TestCase {
            input: "case".into(),
            expected_keywords: vec!["sky".into()],
        }];
        let result = eval.evaluate(&reg, "p1", cases);
        assert_eq!(result.pass_rate, 1.0);
    }

    #[test]
    fn test_evaluate_multiple_keywords_partial_match() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("p1", "hello world"));
        let eval = PromptEval::new();
        let cases = vec![TestCase {
            input: "multi".into(),
            expected_keywords: vec!["hello".into(), "xyz".into()],
        }];
        let result = eval.evaluate(&reg, "p1", cases);
        // "hello" matches, so case passes (any keyword match is enough)
        assert_eq!(result.pass_rate, 1.0);
    }

    #[test]
    fn test_evaluate_no_keyword_matches() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("p1", "the sky is blue"));
        let eval = PromptEval::new();
        let cases = vec![TestCase {
            input: "fail".into(),
            expected_keywords: vec!["red".into(), "green".into()],
        }];
        let result = eval.evaluate(&reg, "p1", cases);
        assert_eq!(result.pass_rate, 0.0);
        assert_eq!(result.failed_cases, 1);
    }

    #[test]
    fn test_eval_result_default() {
        let result = EvalResult {
            pass_rate: 0.75,
            total_cases: 4,
            failed_cases: 1,
        };
        let cloned = result.clone();
        assert_eq!(cloned.pass_rate, 0.75);
        assert_eq!(cloned.total_cases, 4);
        assert_eq!(cloned.failed_cases, 1);
    }

    #[test]
    fn test_test_case_clone() {
        let tc = TestCase {
            input: "test".into(),
            expected_keywords: vec!["a".into(), "b".into()],
        };
        let cloned = tc.clone();
        assert_eq!(cloned.input, "test");
        assert_eq!(cloned.expected_keywords, vec!["a", "b"]);
    }
}
