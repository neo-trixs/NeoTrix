//! Backward compatibility redirect for orchestrator
//!
//! This module re-exports all `orchestrator_v2` types so that existing code
//! using `orchestrator_compat::TradeOrchestrator` continues to compile.
//! New code should use `orchestrator_v2` directly.
//!
//! ## Migration from v1 (`orchestrator`)
//!
//! | v1 import | v2 replacement |
//! |-----------|----------------|
//! | `orchestrator::TradeOrchestrator` | `orchestrator_v2::TradeOrchestrator` |
//! | `orchestrator::OrchTradeContext` | Use v2 `TradeTask` metadata |
//! | `orchestrator::TradePhase26` | Use v2 `WorkerType` + `TaskPriority` |
//! | `orchestrator::Quotation` | Use v2 task payload |
//!
//! ## Quick start
//!
//! ```rust,ignore
//! // Before (v1 — deprecated):
//! use nt_act_trade::orchestrator::TradeOrchestrator;
//! let mut orch = TradeOrchestrator::new();
//!
//! // After (v2 — preferred):
//! use nt_act_trade::orchestrator_v2::{TradeOrchestrator, OrchestratorConfig};
//! let orch = TradeOrchestrator::new(OrchestratorConfig::default());
//!
//! // Or via compat redirect (minimal change):
//! use nt_act_trade::orchestrator_compat::TradeOrchestrator;
//! ```

// Re-export all v2 types for backward compatibility
pub use super::orchestrator_v2::*;
