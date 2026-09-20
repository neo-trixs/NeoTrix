#![forbid(unsafe_code)]

//! # 3-Tier Memory Architecture (mem0-inspired)
//!
//! A unified memory system with three tiers, each optimized for a specific
//! access pattern:
//!
//! - **Tier 1 (Core)**: Always-in-context, 2-5K chars max. User identity,
//!   active goals, current project state. Loaded into every LLM turn.
//! - **Tier 2 (Archival)**: Vector-backed long-term storage. Retrieved
//!   on-demand via embedding similarity search or keyword fallback.
//! - **Tier 3 (Recall)**: Chronological conversation history. Keyword-indexed,
//!   recency-weighted. Searched only when Core + Archival are insufficient.
//!
//! ## Design Principles
//!
//! - **R-P116**: 3-Tier memory — always-in-context (T1), vector archival (T2),
//!   conversation recall (T3).
//! - **R-P122**: Localized maintenance — no global reorg, each tier manages
//!   its own eviction independently.
//! - **R-P11**: Config struct for each tier (CoreStoreConfig, ArchivalConfig,
//!   RecallConfig, RouterConfig).
//! - **R-P103**: Rust idioms — `#![forbid(unsafe_code)]`, proper error types,
//!   `#[cfg(test)]` modules.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use neotrix::l1_action::nt_memory::tiered_memory::{
//!     TieredMemoryHub, MemoryHub, RouterConfig, MemoryItem, MemoryTier, MemoryQuery,
//! };
//!
//! # async fn example() -> Result<(), String> {
//! let hub = TieredMemoryHub::in_memory(RouterConfig::default());
//!
//! // Write to appropriate tier (router decides)
//! let item = MemoryItem::new(
//!     "user-1".to_string(),
//!     "Alice is an ML engineer".to_string(),
//!     MemoryTier::Tier1Core,
//!     0.95,
//!     "session".to_string(),
//! );
//! hub.write(item).await?;
//!
//! // Retrieve (cascades through tiers)
//! let query = MemoryQuery {
//!     text: "Alice".to_string(),
//!     embedding: Vec::new(),
//!     tier_filter: vec![MemoryTier::Tier1Core, MemoryTier::Tier2Archival],
//!     max_results: 10,
//!     min_importance: 0.0,
//!     prefer_concise: true,
//! };
//! let results = hub.retrieve(&query).await?;
//!
//! // Get context snapshot for LLM injection
//! let context = hub.snapshot_for_context().await?;
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub mod tier_core;
pub mod tier_archival;
pub mod tier_recall;
pub mod router;

pub use traits::{
    HubStats, MemoryHub, MemoryItem, MemoryQuery, MemoryResult, MemoryTier, RetrievalStats,
};
pub use tier_core::{CoreStore, CoreStoreConfig, CoreStats};
pub use tier_archival::{ArchivalConfig, ArchivalStore, ArchivalStats, cosine_similarity};
pub use tier_recall::{RecallConfig, RecallStore, RecallStats};
pub use router::{RouterConfig, TieredMemoryHub};
