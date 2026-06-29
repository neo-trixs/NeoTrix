/// Storm breaker — cognitive load monitor for repetitive reasoning suppression.
use std::collections::VecDeque;

use super::cognitive_load::ThinkingMode;

#[derive(Debug)]
pub struct StormBreaker {
    pub recent_inferences: VecDeque<String>,
    pub max_history: usize,
    pub suppression_count: u32,
}

impl StormBreaker {
    pub fn new() -> Self {
        StormBreaker {
            recent_inferences: VecDeque::with_capacity(20),
            max_history: 20,
            suppression_count: 0,
        }
    }

    pub fn check(&mut self, inference: &str) -> bool {
        self.recent_inferences.push_back(inference.to_string());
        if self.recent_inferences.len() > self.max_history {
            self.recent_inferences.pop_front();
        }
        // If last 3 inferences are identical → suppress
        if self.recent_inferences.len() >= 3 {
            let len = self.recent_inferences.len();
            if self.recent_inferences[len - 1] == self.recent_inferences[len - 2]
                && self.recent_inferences[len - 2] == self.recent_inferences[len - 3]
            {
                self.suppression_count += 1;
                return true; // Suppress (storm detected)
            }
        }
        false
    }

    pub fn next_mode(&self, current: ThinkingMode) -> ThinkingMode {
        // Fast → Balanced → Deep alternate
        match current {
            ThinkingMode::Fast => ThinkingMode::Balanced,
            ThinkingMode::Balanced => ThinkingMode::Deep,
            ThinkingMode::Deep => ThinkingMode::Fast,
        }
    }
}

impl Default for StormBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_storm_breaker() {
        let sb = StormBreaker::new();
        assert_eq!(sb.max_history, 20);
        assert_eq!(sb.suppression_count, 0);
        assert!(sb.recent_inferences.is_empty());
    }

    #[test]
    fn test_no_storm_with_unique_inferences() {
        let mut sb = StormBreaker::new();
        assert!(!sb.check("hello"));
        assert!(!sb.check("world"));
        assert!(!sb.check("foo"));
        // 3 unique → no storm
        assert_eq!(sb.suppression_count, 0);
    }

    #[test]
    fn test_storm_detected() {
        let mut sb = StormBreaker::new();
        assert!(!sb.check("repeat"));
        assert!(!sb.check("repeat"));
        assert!(sb.check("repeat")); // 3rd identical → storm
        assert_eq!(sb.suppression_count, 1);
    }

    #[test]
    fn test_storm_clears_after_insertion() {
        let mut sb = StormBreaker::new();
        sb.check("a");
        sb.check("b");
        sb.check("c");
        assert!(!sb.check("a")); // not 3 identical at end
    }

    #[test]
    fn test_next_mode_cycles() {
        assert_eq!(
            StormBreaker::new().next_mode(ThinkingMode::Fast),
            ThinkingMode::Balanced
        );
        assert_eq!(
            StormBreaker::new().next_mode(ThinkingMode::Balanced),
            ThinkingMode::Deep
        );
        assert_eq!(
            StormBreaker::new().next_mode(ThinkingMode::Deep),
            ThinkingMode::Fast
        );
    }

    #[test]
    fn test_suppression_count_accumulates() {
        let mut sb = StormBreaker::new();
        sb.check("x");
        sb.check("y");
        sb.check("z");
        // Clear fifo: now push "same" 3 times
        sb.recent_inferences.clear();
        sb.check("same");
        sb.check("same");
        sb.check("same");
        assert_eq!(sb.suppression_count, 1);
        sb.check("same"); // 4th consecutive
        assert_eq!(sb.suppression_count, 2);
    }
}
