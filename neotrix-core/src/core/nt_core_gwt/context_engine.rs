#![forbid(unsafe_code)]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub current_tokens: usize,
    pub warning_threshold: f64,
}

impl ContextBudget {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            current_tokens: 0,
            warning_threshold: 0.8,
        }
    }

    pub fn remaining(&self) -> usize {
        self.max_tokens.saturating_sub(self.current_tokens)
    }

    pub fn is_over(&self) -> bool {
        self.current_tokens >= self.max_tokens
    }

    pub fn is_warning(&self) -> bool {
        self.current_tokens as f64 >= self.max_tokens as f64 * self.warning_threshold
    }

    pub fn add_tokens(&mut self, n: usize) {
        self.current_tokens = self.current_tokens.saturating_add(n).min(self.max_tokens);
    }

    pub fn reset(&mut self) {
        self.current_tokens = 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCompressionRule {
    pub pattern: String,
    pub replacement: String,
    pub trigger_count: u32,
    pub success_count: u32,
    pub is_active: bool,
}

impl ContextCompressionRule {
    pub fn new(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            trigger_count: 0,
            success_count: 0,
            is_active: true,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.trigger_count == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.trigger_count as f64
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompactionVerdict {
    Compact { reason: String },
    Suppress { reason: String },
    Continue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionRubric {
    pub subtask_threshold: u32,
    pub token_ratio_threshold: f64,
}

impl CompactionRubric {
    pub fn new(subtask_threshold: u32, token_ratio_threshold: f64) -> Self {
        Self {
            subtask_threshold,
            token_ratio_threshold,
        }
    }

    pub fn evaluate(
        &self,
        completed_subtasks: u32,
        current_tokens: usize,
        max_tokens: usize,
        is_stuck: bool,
    ) -> CompactionVerdict {
        let ratio = if max_tokens == 0 {
            1.0
        } else {
            current_tokens as f64 / max_tokens as f64
        };

        if is_stuck && ratio < 0.95 {
            return CompactionVerdict::Suppress {
                reason: "stuck in derivation, preserving context".to_string(),
            };
        }

        if completed_subtasks >= self.subtask_threshold {
            return CompactionVerdict::Compact {
                reason: format!("{} subtasks completed", completed_subtasks),
            };
        }

        if ratio > self.token_ratio_threshold && !is_stuck {
            return CompactionVerdict::Compact {
                reason: format!("token ratio {:.2} above threshold {:.2}", ratio, self.token_ratio_threshold),
            };
        }

        CompactionVerdict::Continue
    }
}

impl Default for CompactionRubric {
    fn default() -> Self {
        Self::new(3, 0.8)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total: u32,
    pub active: u32,
    pub avg_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePool {
    rules: Vec<ContextCompressionRule>,
}

impl RulePool {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: ContextCompressionRule) {
        self.rules.push(rule);
    }

    pub fn remove_rule(&mut self, pattern: &str) {
        self.rules.retain(|r| r.pattern != pattern);
    }

    pub fn apply(&mut self, text: &str) -> String {
        let mut result = text.to_string();
        for rule in self.rules.iter_mut() {
            if !rule.is_active {
                continue;
            }
            let before = result.len();
            result = result.replace(&rule.pattern, &rule.replacement);
            let after = result.len();
            rule.trigger_count += 1;
            if after < before {
                rule.success_count += 1;
            }
        }
        result
    }

    pub fn evolve(&mut self, batch: &[&str]) {
        let mut pattern_counts: Vec<(String, u32)> = Vec::new();
        for text in batch {
            let words: Vec<&str> = text.split_whitespace().collect();
            for window in words.windows(3) {
                let pattern = window.join(" ");
                if let Some(pos) = pattern_counts.iter().position(|(p, _)| *p == pattern) {
                    pattern_counts[pos].1 += 1;
                } else {
                    pattern_counts.push((pattern, 1));
                }
            }
        }

        let threshold = (batch.len() as u32).max(1) * 2;
        for (pattern, count) in &pattern_counts {
            if *count >= threshold && pattern.len() > 20 {
                let exists = self.rules.iter().any(|r| r.pattern == *pattern);
                if !exists {
                    self.add_rule(ContextCompressionRule::new(pattern.as_str(), "[...]"));
                }
            }
        }
    }

    pub fn stats(&self) -> PoolStats {
        let active = self.rules.iter().filter(|r| r.is_active).count() as u32;
        let avg = if self.rules.is_empty() {
            0.0
        } else {
            self.rules.iter().map(|r| r.success_rate()).sum::<f64>() / self.rules.len() as f64
        };
        PoolStats {
            total: self.rules.len() as u32,
            active,
            avg_success_rate: avg,
        }
    }
}

impl Default for RulePool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEngine {
    pub budget: ContextBudget,
    pub rubric: CompactionRubric,
    pub rule_pool: RulePool,
}

impl ContextEngine {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            budget: ContextBudget::new(max_tokens),
            rubric: CompactionRubric::default(),
            rule_pool: RulePool::new(),
        }
    }

    pub fn can_accept(&self, content: &str) -> bool {
        content.len() <= self.budget.remaining()
    }

    pub fn ingest(&mut self, content: &str) -> bool {
        if self.can_accept(content) {
            self.budget.add_tokens(content.len());
            true
        } else {
            false
        }
    }

    pub fn compress(&mut self, content: &str) -> String {
        self.rule_pool.apply(content)
    }

    pub fn compact(&self, completed_subtasks: u32, is_stuck: bool) -> CompactionVerdict {
        self.rubric.evaluate(
            completed_subtasks,
            self.budget.current_tokens,
            self.budget.max_tokens,
            is_stuck,
        )
    }

    pub fn learn_rules(&mut self, batch: &[&str]) {
        self.rule_pool.evolve(batch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_tracking() {
        let mut budget = ContextBudget::new(100);
        assert_eq!(budget.remaining(), 100);
        budget.add_tokens(60);
        assert_eq!(budget.remaining(), 40);
        assert!(!budget.is_warning());
        budget.add_tokens(25);
        assert!(budget.is_warning());
        assert!(!budget.is_over());
        budget.add_tokens(20);
        assert!(budget.is_over());
    }

    #[test]
    fn test_rubric_compact_on_threshold() {
        let rubric = CompactionRubric::new(5, 0.9);
        let verdict = rubric.evaluate(5, 100, 1000, false);
        assert!(matches!(verdict, CompactionVerdict::Compact { .. }));
    }

    #[test]
    fn test_rubric_suppress_on_stuck() {
        let rubric = CompactionRubric::new(5, 0.8);
        let verdict = rubric.evaluate(1, 500, 1000, true);
        assert!(matches!(verdict, CompactionVerdict::Suppress { .. }));
    }

    #[test]
    fn test_rubric_continue() {
        let rubric = CompactionRubric::new(10, 0.9);
        let verdict = rubric.evaluate(2, 300, 1000, false);
        assert_eq!(verdict, CompactionVerdict::Continue);
    }

    #[test]
    fn test_rule_pool_apply() {
        let mut pool = RulePool::new();
        pool.add_rule(ContextCompressionRule::new("very long repetitive text", "shorthand"));
        let result = pool.apply("this is a very long repetitive text here");
        assert!(result.contains("shorthand"));
        assert!(result.len() < 50);
    }

    #[test]
    fn test_context_engine_ingest_respects_budget() {
        let mut engine = ContextEngine::new(50);
        assert!(engine.ingest("hello world"));
        assert!(engine.can_accept("short"));
        assert!(!engine.ingest("this is a very long text that should exceed the available budget"));
    }

    #[test]
    fn test_context_engine_compact() {
        let mut engine = ContextEngine::new(1000);
        engine.budget.add_tokens(900);
        let verdict = engine.compact(4, false);
        if let CompactionVerdict::Compact { .. } = verdict {
            assert!(true);
        } else {
            assert!(false, "expected compact verdict");
        }
    }

    #[test]
    fn test_rule_pool_evolve() {
        let mut pool = RulePool::new();
        let batch = vec![
            "repeated error pattern in log output repeated error pattern",
            "repeated error pattern causing failure repeated error pattern",
            "repeated error pattern found in system log repeated error pattern",
            "repeated error pattern detected again repeated error pattern",
            "repeated error pattern with more data repeated error pattern",
            "repeated error pattern and extra repeated error pattern",
        ];
        pool.evolve(&batch);
        assert!(pool.stats().total > 0);
    }
}
