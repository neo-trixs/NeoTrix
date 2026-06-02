//! Parallel 模块 - 多Agent任务并行处理
//!
//! 纯矩阵运算驱动的并行执行器

pub mod types;
pub mod executor;
pub mod coordinator;
pub mod hands;
#[cfg(test)]
pub mod tests;

// Re-export 主要类型
pub use types::{TaskId, AgentId, TaskState, Task, Agent, AgentPool, TodoTask, AllocationStrategy, AgentMessage, AgentPerformance};
pub use executor::{ExecMode, ParallelExecutor, OptimalTaskAllocator};
pub use coordinator::MultiAgentCoordinator;
pub use hands::{HandType, Hand, HandsController};
