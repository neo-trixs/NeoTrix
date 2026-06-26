//! Parallel 模块 - 多Agent任务并行处理
//!
//! 纯矩阵运算驱动的并行执行器

pub mod coordinator;
pub mod executor;
pub mod hands;
#[cfg(test)]
pub mod tests;
pub mod types;

// Re-export 主要类型
pub use coordinator::MultiAgentCoordinator;
pub use executor::{ExecMode, OptimalTaskAllocator, ParallelExecutor};
pub use hands::{Hand, HandType, HandsController};
pub use types::{
    Agent, AgentId, AgentMessage, AgentPerformance, AgentPool, AllocationStrategy, Task, TaskId,
    TaskState, TodoTask,
};
