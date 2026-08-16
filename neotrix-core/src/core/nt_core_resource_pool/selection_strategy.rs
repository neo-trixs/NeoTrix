/// Unified selection strategies shared across all pool types.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum PoolSelectionStrategy {
    /// Fastest / lowest latency
    #[default]
    Fastest,
    /// Weighted random by score
    WeightedRandom,
    /// Round-robin cycling
    RoundRobin,
    /// Adaptive / learned selection
    Adaptive,
    /// Priority-ordered (first matching wins)
    Priority,
}

impl PoolSelectionStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            PoolSelectionStrategy::Fastest => "fastest",
            PoolSelectionStrategy::WeightedRandom => "weighted_random",
            PoolSelectionStrategy::RoundRobin => "round_robin",
            PoolSelectionStrategy::Adaptive => "adaptive",
            PoolSelectionStrategy::Priority => "priority",
        }
    }
}
