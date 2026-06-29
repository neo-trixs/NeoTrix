use std::sync::atomic::{AtomicU64, Ordering};

/// Gas cost constants for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasOp {
    /// Basic VSA operation (bind, bundle, permute, etc.)
    VsaOp = 10,
    /// Simple handler dispatch
    HandlerCall = 50,
    /// Graph query (knowledge retrieval)
    GraphQuery = 100,
    /// Hypergraph traversal
    HypergraphTraversal = 200,
    /// VSA similarity search (NTSSEG)
    VsaSearch = 300,
    /// LLM inference (external API)
    LlmInference = 1000,
    /// Self-modification (safety gate + edit)
    SelfModify = 5000,
    /// State serialization/checkpoint
    Checkpoint = 150,
    /// Agent bus message send
    AgentMessage = 30,
    /// Sub-consciousness spawn
    SpawnSubConsciousness = 500,
}

/// Gas meter for a single execution context
#[derive(Debug, Clone)]
pub struct GasMeter {
    /// Gas limit for this execution
    pub limit: u64,
    /// Gas consumed so far
    pub consumed: u64,
    /// Whether to panic/fail on exceeding
    pub hard_limit: bool,
}

impl GasMeter {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            consumed: 0,
            hard_limit: true,
        }
    }

    /// Soft limit - logs warning but doesn't fail
    pub fn new_soft(limit: u64) -> Self {
        Self {
            limit,
            consumed: 0,
            hard_limit: false,
        }
    }

    /// Consume gas for an operation. Returns Ok(remaining) or Err("out of gas").
    pub fn consume(&mut self, op: GasOp) -> Result<u64, String> {
        let cost = op as u64;
        self.consumed += cost;
        if self.consumed > self.limit {
            if self.hard_limit {
                return Err(format!(
                    "OUT_OF_GAS: consumed {} > limit {} (op={:?})",
                    self.consumed, self.limit, op
                ));
            } else {
                log::warn!(
                    "[gas] soft limit exceeded: {} > {}",
                    self.consumed,
                    self.limit
                );
            }
        }
        Ok(self.limit.saturating_sub(self.consumed))
    }

    /// Remaining gas
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Utilization ratio 0.0-1.0
    pub fn utilization(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        self.consumed as f64 / self.limit as f64
    }
}

/// Global gas budget tracker for the entire system
#[derive(Debug)]
pub struct GlobalGasBudget {
    /// Per-cycle gas budget
    pub per_cycle_budget: u64,
    /// Gas consumed this cycle
    cycle_consumed: AtomicU64,
    /// Total gas consumed all time
    total_consumed: AtomicU64,
}

impl GlobalGasBudget {
    pub fn new(per_cycle: u64) -> Self {
        Self {
            per_cycle_budget: per_cycle,
            cycle_consumed: AtomicU64::new(0),
            total_consumed: AtomicU64::new(0),
        }
    }

    /// Default: 100,000 gas per cycle
    pub fn default() -> Self {
        Self::new(100_000)
    }

    /// Try to allocate gas from the global budget
    pub fn allocate(&self, amount: u64) -> Result<u64, String> {
        let current = self.cycle_consumed.fetch_add(amount, Ordering::SeqCst);
        let new_total = current + amount;
        if new_total > self.per_cycle_budget {
            // Rollback
            self.cycle_consumed.fetch_sub(amount, Ordering::SeqCst);
            return Err(format!(
                "GLOBAL_OUT_OF_GAS: cycle budget {} exhausted (attempted {})",
                self.per_cycle_budget, new_total
            ));
        }
        self.total_consumed.fetch_add(amount, Ordering::SeqCst);
        Ok(self.per_cycle_budget - new_total)
    }

    /// Reset cycle counter (call at start of each cycle)
    pub fn reset_cycle(&self) {
        self.cycle_consumed.store(0, Ordering::SeqCst);
    }

    /// Utilization of current cycle
    pub fn cycle_utilization(&self) -> f64 {
        self.cycle_consumed.load(Ordering::SeqCst) as f64 / self.per_cycle_budget as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_consume_and_remaining() {
        let mut meter = GasMeter::new(100);
        let remaining = meter.consume(GasOp::VsaOp).unwrap();
        assert_eq!(remaining, 90);
        assert_eq!(meter.remaining(), 90);
        assert!((meter.utilization() - 0.10).abs() < 0.01);
    }

    #[test]
    fn test_hard_limit_triggers_error() {
        let mut meter = GasMeter::new(30);
        assert!(meter.consume(GasOp::VsaOp).is_ok()); // 10 consumed, 20 remaining
        assert!(meter.consume(GasOp::VsaOp).is_ok()); // 20 consumed, 10 remaining
        assert!(meter.consume(GasOp::VsaOp).is_ok()); // 30 consumed, 0 remaining
                                                      // Next op exceeds limit
        let result = meter.consume(GasOp::VsaOp);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("OUT_OF_GAS"));
    }

    #[test]
    fn test_global_budget_allocation_and_exhaustion() {
        let budget = GlobalGasBudget::new(100);
        // Allocate 60 → ok
        assert!(budget.allocate(60).is_ok());
        assert!((budget.cycle_utilization() - 0.60).abs() < 0.01);
        // Allocate 40 → ok (60+40=100)
        assert!(budget.allocate(40).is_ok());
        // Allocate 1 → fails (100+1 > 100)
        let result = budget.allocate(1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("GLOBAL_OUT_OF_GAS"));
        // Reset
        budget.reset_cycle();
        assert!((budget.cycle_utilization() - 0.0).abs() < 0.01);
        // After reset, allocate 50 → ok
        assert!(budget.allocate(50).is_ok());
    }

    #[test]
    fn test_soft_limit_does_not_error() {
        let mut meter = GasMeter::new_soft(20);
        // 10 + 10 = 20, at limit
        assert!(meter.consume(GasOp::VsaOp).is_ok());
        assert!(meter.consume(GasOp::VsaOp).is_ok());
        // Exceed soft limit — should log warning but not error
        let result = meter.consume(GasOp::VsaOp);
        assert!(result.is_ok());
        assert_eq!(meter.consumed, 30);
        assert_eq!(meter.remaining(), 0);
    }

    #[test]
    fn test_gas_op_costs() {
        assert_eq!(GasOp::VsaOp as u64, 10);
        assert_eq!(GasOp::HandlerCall as u64, 50);
        assert_eq!(GasOp::GraphQuery as u64, 100);
        assert_eq!(GasOp::HypergraphTraversal as u64, 200);
        assert_eq!(GasOp::VsaSearch as u64, 300);
        assert_eq!(GasOp::LlmInference as u64, 1000);
        assert_eq!(GasOp::SelfModify as u64, 5000);
        assert_eq!(GasOp::Checkpoint as u64, 200);
        assert_eq!(GasOp::AgentMessage as u64, 30);
        assert_eq!(GasOp::SpawnSubConsciousness as u64, 500);
    }
}
