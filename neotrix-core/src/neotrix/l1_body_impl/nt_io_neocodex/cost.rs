// ── Cost Tracker (from Claude Code: budget enforcement) ──

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CostTracker {
    pub total_spent: f64,
    pub max_budget: f64,
    pub call_count: u64,
    pub history: VecDeque<(String, f64, u64)>,
}

impl CostTracker {
    pub fn new(max_budget: f64) -> Self {
        Self {
            total_spent: 0.0,
            max_budget,
            call_count: 0,
            history: VecDeque::new(),
        }
    }

    pub fn record(&mut self, tool: &str, cost: f64, tokens: u64) -> Result<(), String> {
        if self.total_spent + cost > self.max_budget {
            return Err(format!(
                "Budget exceeded: ${:.4} + ${:.4} > ${:.4} limit",
                self.total_spent, cost, self.max_budget
            ));
        }
        self.total_spent += cost;
        self.call_count += 1;
        self.history.push_back((tool.to_string(), cost, tokens));
        if self.history.len() > 1000 {
            self.history.pop_front();
        }
        Ok(())
    }

    pub fn remaining(&self) -> f64 {
        (self.max_budget - self.total_spent).max(0.0)
    }

    pub fn summary(&self) -> String {
        format!(
            "${:.4} spent / ${:.4} budget · {} calls",
            self.total_spent, self.max_budget, self.call_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker_budget() {
        let mut ct = CostTracker::new(1.0);
        assert!(ct.record("read", 0.5, 100).is_ok());
        assert!(ct.record("write", 0.6, 200).is_err());
        assert!(ct.remaining() < 0.51);
    }
}