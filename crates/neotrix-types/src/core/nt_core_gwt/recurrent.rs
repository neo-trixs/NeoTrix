/// Recurrent processing — cyclic attention in Global Workspace

#[derive(Debug, Clone)]
pub enum ConsciousnessState {
    Idle,
    Processing,
    Broadcasting,
}

#[derive(Debug, Clone)]
pub struct TickMetrics {
    pub cycle: u64,
    pub attention_score: f64,
    pub items_processed: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopExit {
    Continue,
    Broadcast,
    Halt,
}

#[derive(Debug, Clone)]
pub struct PanoramaCell {
    pub content: String,
    pub salience: f64,
}

#[derive(Debug, Clone)]
pub struct RecurrentCell {
    pub cell_type: String,
    pub activation: f64,
}

#[derive(Debug, Clone)]
pub struct CellDecision {
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct ConsciousnessLoop {
    pub cycle: u64,
    pub state: ConsciousnessState,
    pub salience_threshold: f64,
    pub broadcast_count: u64,
}

impl ConsciousnessLoop {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            state: ConsciousnessState::Idle,
            salience_threshold: 0.6,
            broadcast_count: 0,
        }
    }

    /// Run one consciousness cycle.
    /// Returns augmented context with consciousness insights.
    pub fn cycle(&mut self, context: &str, task_salience: f64, cells: &[PanoramaCell]) -> String {
        self.cycle += 1;
        self.state = ConsciousnessState::Processing;

        let total_salience: f64 = cells.iter().map(|c| c.salience).sum::<f64>() + task_salience;
        let n_items = cells.len();
        let decision = self.decide(total_salience, task_salience);

        match decision {
            LoopExit::Broadcast => {
                self.state = ConsciousnessState::Broadcasting;
                self.broadcast_count += 1;
                let insight = format!(
                    "[C{bc}] broadcast | salience={ts:.4} items={n} | {ctx:.80}",
                    bc = self.broadcast_count, ts = total_salience, n = n_items, ctx = context
                );
                self.state = ConsciousnessState::Idle;
                format!("{}\n# consciousness: {insight}", context)
            }
            LoopExit::Continue => {
                self.state = ConsciousnessState::Idle;
                let insight = format!(
                    "[C{cy}] continue | salience={ts:.4} items={n}",
                    cy = self.cycle, ts = total_salience, n = n_items
                );
                format!("{}\n# consciousness: {insight}", context)
            }
            LoopExit::Halt => context.to_string(),
        }
    }

    fn decide(&self, total_salience: f64, task_salience: f64) -> LoopExit {
        if total_salience > self.salience_threshold * 2.0 || task_salience > self.salience_threshold {
            LoopExit::Broadcast
        } else if total_salience > self.salience_threshold * 0.3 {
            LoopExit::Continue
        } else {
            LoopExit::Halt
        }
    }
}

impl Default for ConsciousnessLoop {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_cycle_halt_on_zero_salience() {
        let mut loop_ = ConsciousnessLoop::new();
        let result = loop_.cycle("test task", 0.0, &[]);
        assert_eq!(result, "test task", "halt should return context unchanged");
        assert_eq!(loop_.cycle, 1);
    }

    #[test]
    fn test_consciousness_cycle_continue_on_moderate_salience() {
        let mut loop_ = ConsciousnessLoop::new();
        let result = loop_.cycle("refactor module", 0.3, &[]);
        assert!(result.contains("continue"), "moderate salience should continue");
        assert!(result.contains("refactor module"), "original context should be preserved");
    }

    #[test]
    fn test_consciousness_cycle_broadcast_on_high_salience() {
        let mut loop_ = ConsciousnessLoop::new();
        let cells = vec![
            PanoramaCell { content: "critical bug in parser".into(), salience: 0.9 },
        ];
        let result = loop_.cycle("fix crash", 0.5, &cells);
        assert!(result.contains("broadcast"), "high salience should broadcast");
        assert_eq!(loop_.broadcast_count, 1);
    }

    #[test]
    fn test_consciousness_cycle_total_salience_triggers_broadcast() {
        let mut loop_ = ConsciousnessLoop::new();
        let cells = vec![
            PanoramaCell { content: "item1".into(), salience: 0.7 },
            PanoramaCell { content: "item2".into(), salience: 0.7 },
        ];
        let result = loop_.cycle("complex task", 0.1, &cells);
        // total_salience = 0.7+0.7+0.1 = 1.5 > 1.2 (threshold*2)
        assert!(result.contains("broadcast"), "total salience > 2*threshold should broadcast");
    }

    #[test]
    fn test_consciousness_cycle_increments_cycle_count() {
        let mut loop_ = ConsciousnessLoop::new();
        assert_eq!(loop_.cycle, 0);
        loop_.cycle("a", 0.0, &[]);
        assert_eq!(loop_.cycle, 1);
        loop_.cycle("b", 0.0, &[]);
        assert_eq!(loop_.cycle, 2);
    }

    #[test]
    fn test_consciousness_decide_thresholds() {
        let loop_ = ConsciousnessLoop::new();
        assert!(matches!(loop_.decide(0.1, 0.0), LoopExit::Halt), "very low salience should halt");
        assert!(matches!(loop_.decide(1.3, 0.0), LoopExit::Broadcast), "high total salience should broadcast");
        assert!(matches!(loop_.decide(0.0, 0.7), LoopExit::Broadcast), "high task salience should broadcast");
        // 0.3 > 0.3*0.6=0.18 but < 2*0.6=1.2 and < 0.6 → Continue
        assert!(matches!(loop_.decide(0.3, 0.0), LoopExit::Continue), "moderate salience should continue");
    }
}
