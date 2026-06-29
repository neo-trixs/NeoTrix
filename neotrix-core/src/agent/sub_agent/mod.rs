//! Concurrent sub-agent execution system.
//!
//! Inspired by DeepSeek-TUI's sub-agent pattern. Manages a pool of
//! concurrent sub-agents with bounded result retrieval, timeout handling,
//! and structured event emission.
//!
//! # Architecture
//!
//! ```text
//! SubAgentPool
//!   ├── launch(prompt, capabilities) → Uuid
//!   ├── wait_for(id) → SubAgentResult
//!   ├── handle_read_slice(id, start, end) → Vec<String>
//!   ├── cancel(id) / cancel_all()
//!   └── shutdown()
//! ```

pub mod async_delegate;
pub mod execution;
pub mod pool;
#[cfg(test)]
pub mod tests;
pub mod types;

#[cfg(feature = "sandbox")]
pub use execution::SandboxAgent;
pub use execution::SubAgentVariant;
pub use pool::SubAgentPool;
pub use types::{SubAgentConfig, SubAgentEvent, SubAgentHandle, SubAgentResult, SubAgentStatus};
