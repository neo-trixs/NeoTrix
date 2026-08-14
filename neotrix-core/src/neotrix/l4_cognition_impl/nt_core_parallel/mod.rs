//! Parallel 模块 - 多Agent任务并行处理
//!
//! 纯矩阵运算驱动的并行执行器

pub mod types;
pub mod executor;
pub mod coordinator;
pub mod hands;
pub mod contract;
pub mod isolation;
pub mod diversity;
#[cfg(test)]
pub mod tests;

// Re-export 主要类型
pub use types::{TaskId, AgentId, TaskState, Task, Agent, AgentPool, TodoTask, AllocationStrategy, AgentMessage, AgentPerformance};
pub use executor::{ExecMode, ParallelExecutor, OptimalTaskAllocator};
pub use coordinator::MultiAgentCoordinator;
pub use hands::{HandType, Hand, HandsController};
pub use contract::{TaskContract, TaskContractWarden, ContractState, ContractStats};
pub use isolation::{IntentIsolator, AtomicDecomposer, IsolatedContext, DecompositionPlan, AtomicUnit, OutputContract, OutputFormat, TaskKind};
pub use diversity::{Candidate, DppSelector};
