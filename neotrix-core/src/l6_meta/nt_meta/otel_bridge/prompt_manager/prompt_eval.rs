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

    /// ⚠️ 2026-09-30 修正：本辅助函数原签名是 `make_prompt(id, template)`，
    /// 并把首参传给 `PromptVersion::new` 的 **`id`** 位。
    /// 但 `register` 是按 **`name`** 建索引的（`self.prompts.entry(prompt.name…)`），
    /// 而测试又用 `eval.evaluate(&reg, "p1", …)` 按 **`"p1"`** 取
    /// ⇒ `"p1"` 落在 `id` 上、`name` 是 `"test_prompt"`，
    /// `get_latest("p1")` 返回 `None` ⇒ 所有用例都被判 fail。
    ///
    /// 这是**测试辅助函数的参数错位**，不是 `evaluate` 的实现缺陷 ——
    /// `evaluate` 的 `to_lowercase()` 两侧匹配逻辑本身是对的。
    /// 恢复本模块声明后，这 4 个测试立刻暴露了它（此前从不运行）。
    fn make_prompt(name: &str, template: &str) -> PromptVersion {
        PromptVersion::new(
            // id 与 name 保持一致，避免再次出现「按 name 查却把值放进 id」的错位。
            name.into(),
            name.into(),
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
