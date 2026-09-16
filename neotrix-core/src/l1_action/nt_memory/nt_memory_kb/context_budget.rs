//! Context Budget Manager — 上下文预算管理
//! 确保推理上下文严格有界，不受任务时长影响

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContextBudget {
    max_tokens: usize,
    used_tokens: usize,
    allocations: HashMap<String, usize>, // module → allocated tokens
    priority_weights: HashMap<String, f64>,
}

impl ContextBudget {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            used_tokens: 0,
            allocations: HashMap::new(),
            priority_weights: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, module: &str, tokens: usize) -> bool {
        if self.used_tokens + tokens > self.max_tokens {
            return false;
        }
        *self.allocations.entry(module.to_string()).or_insert(0) += tokens;
        self.used_tokens += tokens;
        true
    }

    pub fn release(&mut self, module: &str) {
        if let Some(tokens) = self.allocations.remove(module) {
            self.used_tokens = self.used_tokens.saturating_sub(tokens);
        }
    }

    pub fn remaining(&self) -> usize {
        self.max_tokens.saturating_sub(self.used_tokens)
    }

    pub fn utilization(&self) -> f64 {
        self.used_tokens as f64 / self.max_tokens as f64
    }

    pub fn set_priority(&mut self, module: &str, weight: f64) {
        self.priority_weights.insert(module.to_string(), weight);
    }

    pub fn fits(&self, tokens: usize) -> bool {
        self.used_tokens + tokens <= self.max_tokens
    }
}

impl Default for ContextBudget {
    fn default() -> Self { Self::new(8192) } // Default 8K tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_basic() {
        let mut budget = ContextBudget::new(1000);
        assert!(budget.allocate("search", 300));
        assert!(budget.allocate("reason", 400));
        assert!(!budget.allocate("overflow", 400)); // would exceed
        assert_eq!(budget.remaining(), 300);
        
        budget.release("search");
        assert_eq!(budget.remaining(), 600);
        assert!(budget.fits(600));
    }
}
