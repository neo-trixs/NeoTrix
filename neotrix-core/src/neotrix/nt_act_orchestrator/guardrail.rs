use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardrailResult {
    Pass,
    Fail(String),
    Warn(String),
}

pub trait TaskGuardrail: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn validate_input(&self, task_description: &str, input: &str) -> GuardrailResult;
    fn validate_output(&self, task_description: &str, expected: Option<&str>, actual: &str) -> GuardrailResult;
}

pub struct FnGuardrail {
    name: &'static str,
    input_check: Option<Arc<dyn Fn(&str, &str) -> GuardrailResult + Send + Sync>>,
    output_check: Option<Arc<dyn Fn(&str, Option<&str>, &str) -> GuardrailResult + Send + Sync>>,
}

impl FnGuardrail {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            input_check: None,
            output_check: None,
        }
    }

    pub fn with_input_check(mut self, f: Arc<dyn Fn(&str, &str) -> GuardrailResult + Send + Sync>) -> Self {
        self.input_check = Some(f);
        self
    }

    pub fn with_output_check(mut self, f: Arc<dyn Fn(&str, Option<&str>, &str) -> GuardrailResult + Send + Sync>) -> Self {
        self.output_check = Some(f);
        self
    }
}

impl TaskGuardrail for FnGuardrail {
    fn name(&self) -> &'static str {
        self.name
    }

    fn validate_input(&self, task: &str, input: &str) -> GuardrailResult {
        match &self.input_check {
            Some(f) => f(task, input),
            None => GuardrailResult::Pass,
        }
    }

    fn validate_output(&self, task: &str, expected: Option<&str>, actual: &str) -> GuardrailResult {
        match &self.output_check {
            Some(f) => f(task, expected, actual),
            None => GuardrailResult::Pass,
        }
    }
}

impl std::fmt::Debug for FnGuardrail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnGuardrail")
            .field("name", &self.name)
            .finish()
    }
}

#[derive(Debug)]
pub struct GuardrailChain {
    guardrails: Vec<Box<dyn TaskGuardrail>>,
}

impl GuardrailChain {
    pub fn new() -> Self {
        Self { guardrails: Vec::new() }
    }

    pub fn add(&mut self, guardrail: Box<dyn TaskGuardrail>) {
        self.guardrails.push(guardrail);
    }

    pub fn validate_input(&self, task: &str, input: &str) -> Vec<(&str, GuardrailResult)> {
        self.guardrails.iter().map(|g| {
            (g.name(), g.validate_input(task, input))
        }).collect()
    }

    pub fn validate_output(&self, task: &str, expected: Option<&str>, actual: &str) -> Vec<(&str, GuardrailResult)> {
        self.guardrails.iter().map(|g| {
            (g.name(), g.validate_output(task, expected, actual))
        }).collect()
    }

    pub fn has_failures<'a>(&self, results: &[(&'a str, GuardrailResult)]) -> Vec<&'a str> {
        results.iter()
            .filter_map(|(name, r)| match r {
                GuardrailResult::Fail(_) => Some(*name),
                _ => None,
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.guardrails.is_empty()
    }

    pub fn len(&self) -> usize {
        self.guardrails.len()
    }
}

pub mod builtins {
    use super::*;

    pub fn non_empty_output() -> FnGuardrail {
        FnGuardrail::new("non_empty_output")
            .with_output_check(Arc::new(|_task, _expected, actual| {
                if actual.trim().is_empty() {
                    GuardrailResult::Fail("Output is empty".to_string())
                } else {
                    GuardrailResult::Pass
                }
            }))
    }

    pub fn max_input_length(max_chars: usize) -> FnGuardrail {
        FnGuardrail::new("max_input_length")
            .with_input_check(Arc::new(move |_task, input| {
                if input.len() > max_chars {
                    GuardrailResult::Fail(format!(
                        "Input exceeds max length ({} > {})", input.len(), max_chars
                    ))
                } else {
                    GuardrailResult::Pass
                }
            }))
    }

    pub fn contains_keywords(keywords: Vec<String>) -> FnGuardrail {
        let kws = keywords.clone();
        FnGuardrail::new("contains_keywords")
            .with_output_check(Arc::new(move |_task, _expected, actual| {
                let missing: Vec<&str> = kws.iter()
                    .filter(|k| !actual.contains(k.as_str()))
                    .map(|s| s.as_str())
                    .collect();
                if missing.is_empty() {
                    GuardrailResult::Pass
                } else {
                    GuardrailResult::Warn(format!("Missing keywords: {:?}", missing))
                }
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::builtins::*;

    #[test]
    fn test_non_empty_output_pass() {
        let guardrail = non_empty_output();
        let result = guardrail.validate_output("test", None, "some output");
        assert!(matches!(result, GuardrailResult::Pass));
    }

    #[test]
    fn test_non_empty_output_fail() {
        let guardrail = non_empty_output();
        let result = guardrail.validate_output("test", None, "  ");
        assert!(matches!(result, GuardrailResult::Fail(_)));
    }

    #[test]
    fn test_max_input_length() {
        let guardrail = max_input_length(10);
        let pass = guardrail.validate_input("test", "short");
        assert!(matches!(pass, GuardrailResult::Pass));

        let fail = guardrail.validate_input("test", "this is way too long");
        assert!(matches!(fail, GuardrailResult::Fail(_)));
    }

    #[test]
    fn test_contains_keywords() {
        let guardrail = contains_keywords(vec!["TODO".to_string(), "FIXME".to_string()]);
        let pass = guardrail.validate_output("test", None, "TODO: refactor this, FIXME: bug here");
        assert!(matches!(pass, GuardrailResult::Pass));

        let warn = guardrail.validate_output("test", None, "nothing");
        assert!(matches!(warn, GuardrailResult::Warn(_)));
    }

    #[test]
    fn test_guardrail_chain() {
        let mut chain = GuardrailChain::new();
        chain.add(Box::new(non_empty_output()));
        chain.add(Box::new(max_input_length(100)));

        let results = chain.validate_output("test", None, "valid output");
        let failures = chain.has_failures(&results);
        assert!(failures.is_empty());
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_guardrail_chain_failures() {
        let mut chain = GuardrailChain::new();
        chain.add(Box::new(non_empty_output()));

        let results = chain.validate_output("test", None, "  ");
        let failures = chain.has_failures(&results);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0], "non_empty_output");
    }

    #[test]
    fn test_fn_guardrail_input_only() {
        let g = FnGuardrail::new("input_only")
            .with_input_check(Arc::new(|_, _| GuardrailResult::Pass));
        assert!(matches!(g.validate_input("t", "x"), GuardrailResult::Pass));
        assert!(matches!(g.validate_output("t", None, "x"), GuardrailResult::Pass));
    }

    #[test]
    fn test_empty_chain() {
        let chain = GuardrailChain::new();
        assert!(chain.is_empty());
    }
}
