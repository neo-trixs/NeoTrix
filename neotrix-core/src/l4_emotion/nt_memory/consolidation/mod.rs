#![forbid(unsafe_code)]

//! Memory Consolidation — short-term to long-term promotion, working memory, and TTL cache.
//!
//! Implements R-P117, R-P118, R-MEM09 rules for memory lifecycle management.

pub mod consolidator;
pub mod working;
pub mod cache;

pub use consolidator::{ConsolidatedMemory, MemoryConsolidator, PromotionType};
pub use working::{WorkingMemory, WorkingMemoryConfig};
pub use cache::{CacheStats, MemoryCache};
