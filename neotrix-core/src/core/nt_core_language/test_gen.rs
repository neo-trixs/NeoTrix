use super::value::NeValue;

#[derive(Debug, Clone)]
pub struct TestGenConfig {
    pub max_examples: usize,
    pub include_negative: bool,
    pub fuzz_iterations: u64,
}

impl Default for TestGenConfig {
    fn default() -> Self {
        TestGenConfig {
            max_examples: 10,
            include_negative: true,
            fuzz_iterations: 50,
        }
    }
}

/// Format a NeValue as Ne source code
pub fn ne_value_to_source(v: &NeValue) -> String {
    match v {
        NeValue::Nil => "nil".to_string(),
        NeValue::Int(n) => n.to_string(),
        NeValue::Float(f) => f.to_string(),
        NeValue::Bool(b) => b.to_string(),
        NeValue::Str(s) => format!("\"{}\"", s),
        NeValue::Vsa(_) => "#vsa".to_string(),
        NeValue::List(items) => {
            let inner: Vec<String> = items.iter().map(ne_value_to_source).collect();
            format!("(list {})", inner.join(" "))
        }
        NeValue::Lambda(_, _) => "#lambda".to_string(),
        NeValue::Primitive(_) => "#primitive".to_string(),
        NeValue::Exports(_) => "#exports".to_string(),
        NeValue::TestResult { .. } => "#test-result".to_string(),
    }
}

/// Generate a Ne probabilistic test program from function examples.
/// Produces assert-based test code for a given function name with example inputs+outputs.
pub fn generate_probabilistic_test(
    func_name: &str,
    examples: &[(Vec<NeValue>, NeValue)],
    config: &TestGenConfig,
) -> String {
    let mut code = String::new();
    code.push_str(&format!(
        "; Auto-generated probabilistic test for {}\n",
        func_name
    ));
    code.push_str("(begin\n");
    let limit = config.max_examples.min(examples.len());
    for (i, (inputs, output)) in examples.iter().enumerate().take(limit) {
        let input_strs: Vec<String> = inputs.iter().map(ne_value_to_source).collect();
        let output_str = ne_value_to_source(output);
        code.push_str(&format!(
            "  (assert (== ({} {}) {}) \"example_{}: {}({}) == {}\")\n",
            func_name,
            input_strs.join(" "),
            output_str,
            i,
            func_name,
            input_strs.join(" "),
            output_str,
        ));
    }
    code.push_str("  true\n)");
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ne_value_to_source() {
        assert_eq!(ne_value_to_source(&NeValue::Nil), "nil");
        assert_eq!(ne_value_to_source(&NeValue::Int(42)), "42");
        assert_eq!(ne_value_to_source(&NeValue::Float(3.14)), "3.14");
        assert_eq!(ne_value_to_source(&NeValue::Bool(true)), "true");
        assert_eq!(ne_value_to_source(&NeValue::Bool(false)), "false");
        assert_eq!(
            ne_value_to_source(&NeValue::Str("hello".into())),
            "\"hello\""
        );
        assert_eq!(ne_value_to_source(&NeValue::Vsa(vec![1, 2, 3])), "#vsa");
        assert_eq!(
            ne_value_to_source(&NeValue::Primitive("add".into())),
            "#primitive"
        );
        assert_eq!(
            ne_value_to_source(&NeValue::Lambda(vec![], vec![])),
            "#lambda"
        );
        assert_eq!(
            ne_value_to_source(&NeValue::TestResult {
                passed: 1,
                failed: 0,
                total: 1,
                assert_count: 5,
                coverage: 3
            }),
            "#test-result"
        );
    }

    #[test]
    fn test_generate_probabilistic_test() {
        let examples = vec![
            (vec![NeValue::Int(2), NeValue::Int(3)], NeValue::Int(5)),
            (vec![NeValue::Int(0), NeValue::Int(0)], NeValue::Int(0)),
        ];
        let config = TestGenConfig::default();
        let code = generate_probabilistic_test("add", &examples, &config);
        assert!(code.contains("add"));
        assert!(code.contains("example_0"));
        assert!(code.contains("example_1"));
        assert!(code.contains("assert"));
    }

    #[test]
    fn test_test_gen_config_defaults() {
        let cfg = TestGenConfig::default();
        assert_eq!(cfg.max_examples, 10);
        assert_eq!(cfg.include_negative, true);
        assert_eq!(cfg.fuzz_iterations, 50);
    }
}
