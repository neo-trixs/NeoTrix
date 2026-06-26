#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestCard {
    pub id: u64,
    pub title: String,
    pub steps: Vec<String>,
    pub expected_outcomes: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepOutcome {
    pub step_index: usize,
    pub actual_output: String,
    pub matched_expected: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestResult {
    pub card_id: u64,
    pub passed: bool,
    pub step_results: Vec<StepOutcome>,
    pub overall_evidence: String,
    pub integrity_check_passed: bool,
}

#[derive(Debug, Clone)]
pub struct UatGateStats {
    pub total_cards: usize,
    pub total_evaluations: usize,
    pub passed_evaluations: usize,
    pub pass_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UatGate {
    test_cards: Vec<TestCard>,
    results: Vec<TestResult>,
    next_id: u64,
}

impl UatGate {
    pub fn new() -> Self {
        Self {
            test_cards: Vec::new(),
            results: Vec::new(),
            next_id: 0,
        }
    }

    pub fn register_card(&mut self, title: &str, steps: &[&str], expected: &[&str]) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.test_cards.push(TestCard {
            id,
            title: title.to_string(),
            steps: steps.iter().map(|s| s.to_string()).collect(),
            expected_outcomes: expected.iter().map(|e| e.to_string()).collect(),
            timestamp: 0,
        });
        id
    }

    pub fn evaluate_card(&mut self, card_id: u64, actual_outputs: &[&str]) -> Option<TestResult> {
        let card = self.test_cards.iter().find(|c| c.id == card_id)?;
        let mut step_results: Vec<StepOutcome> = Vec::new();
        let mut all_passed = true;

        for (i, expected) in card.expected_outcomes.iter().enumerate() {
            let actual = actual_outputs.get(i).copied().unwrap_or("");
            let matched = actual.contains(expected.as_str()) || expected.contains(actual);
            if !matched {
                all_passed = false;
            }
            step_results.push(StepOutcome {
                step_index: i,
                actual_output: actual.to_string(),
                matched_expected: matched,
            });
        }

        let evidence = if all_passed {
            "all steps passed".to_string()
        } else {
            let failed: Vec<usize> = step_results
                .iter()
                .filter(|s| !s.matched_expected)
                .map(|s| s.step_index)
                .collect();
            format!("failed steps: {:?}", failed)
        };

        let result = TestResult {
            card_id,
            passed: all_passed,
            integrity_check_passed: all_passed,
            step_results,
            overall_evidence: evidence,
        };
        self.results.push(result.clone());
        Some(result)
    }

    pub fn pass_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 1.0;
        }
        let passed = self.results.iter().filter(|r| r.passed).count();
        passed as f64 / self.results.len() as f64
    }

    pub fn recent_results(&self, n: usize) -> Vec<&TestResult> {
        self.results.iter().rev().take(n).collect()
    }

    pub fn stats(&self) -> UatGateStats {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        UatGateStats {
            total_cards: self.test_cards.len(),
            total_evaluations: total,
            passed_evaluations: passed,
            pass_rate: if total == 0 {
                1.0
            } else {
                passed as f64 / total as f64
            },
        }
    }
}

impl Default for UatGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uat_gate_new() {
        let gate = UatGate::new();
        assert_eq!(gate.pass_rate(), 1.0);
        assert_eq!(gate.stats().total_cards, 0);
    }

    #[test]
    fn test_register_card() {
        let mut gate = UatGate::new();
        let id = gate.register_card(
            "test feature",
            &["step 1", "step 2"],
            &["output 1", "output 2"],
        );
        assert_eq!(id, 0);
        assert_eq!(gate.stats().total_cards, 1);
    }

    #[test]
    fn test_evaluate_all_pass() {
        let mut gate = UatGate::new();
        let id = gate.register_card("pass", &["s1", "s2"], &["expected", "expected"]);
        let result = gate.evaluate_card(id, &["expected", "expected"]).unwrap();
        assert!(result.passed);
        assert!(result.integrity_check_passed);
    }

    #[test]
    fn test_evaluate_some_fail() {
        let mut gate = UatGate::new();
        let id = gate.register_card("fail", &["s1", "s2"], &["expected_a", "expected_b"]);
        let result = gate.evaluate_card(id, &["wrong", "expected_b"]).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_evaluate_nonexistent_card() {
        let mut gate = UatGate::new();
        let result = gate.evaluate_card(999, &["x"]);
        assert!(result.is_none());
    }

    #[test]
    fn test_pass_rate() {
        let mut gate = UatGate::new();
        assert!((gate.pass_rate() - 1.0).abs() < 0.01);
        let id = gate.register_card("c1", &["s1"], &["ok"]);
        gate.evaluate_card(id, &["ok"]);
        assert!((gate.pass_rate() - 1.0).abs() < 0.01);
        gate.evaluate_card(id, &["fail"]);
        assert!((gate.pass_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_stats() {
        let mut gate = UatGate::new();
        let id = gate.register_card("c", &["s"], &["ok"]);
        gate.evaluate_card(id, &["ok"]);
        gate.evaluate_card(id, &["fail"]);
        let s = gate.stats();
        assert_eq!(s.total_cards, 1);
        assert_eq!(s.total_evaluations, 2);
        assert_eq!(s.passed_evaluations, 1);
    }

    #[test]
    fn test_recent_results() {
        let mut gate = UatGate::new();
        let id = gate.register_card("c1", &["s"], &["ok"]);
        gate.evaluate_card(id, &["ok"]);
        gate.evaluate_card(id, &["fail"]);
        let recent = gate.recent_results(1);
        assert_eq!(recent.len(), 1);
        assert!(!recent[0].passed);
    }
}
