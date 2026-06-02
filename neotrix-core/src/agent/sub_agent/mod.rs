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

pub mod types;
pub mod pool;
pub mod execution;
#[cfg(test)]
pub mod tests;

pub use types::{SubAgentConfig, SubAgentStatus, SubAgentResult, SubAgentHandle, SubAgentEvent};
pub use pool::SubAgentPool;
pub use execution::SubAgentVariant;
#[cfg(feature = "sandbox")]
pub use execution::SandboxAgent;
