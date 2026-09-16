pub mod consolidation;
pub mod engine;
pub mod hebbian;

pub use consolidation::{ConsolidationConfig, ConsolidationResult, MemoryConsolidation};
pub use engine::{SleepConfig, SleepEngine, SleepResult, SleepStats};
pub use hebbian::HebbianUpdater;
