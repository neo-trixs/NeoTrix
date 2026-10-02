#![forbid(unsafe_code)]

//! ADD-only writes & temporal fact management (R-P117, R-P120, R-P122).
//!
//! Core design:
//! - **Every write is an ADD** (R-P117): entries are appended, never mutated.
//!   "Updates" close the old entry's temporal window and append a successor.
//! - **Temporal fact windows** (R-P120): every fact carries `valid_from`/`valid_to`
//!   timestamps. Queries support "now" (valid_to is None or > now) and "at T"
//!   (valid_from <= T < valid_to).
//! - **Localized maintenance** (R-P122): supersession is the only write path;
//!   cleanup is a read-side concern, never a write-side delete.

pub mod memory_entry;
pub mod temporal_query;
pub mod temporal_store;

pub use memory_entry::AddOnlyMemoryEntry;
pub use temporal_query::{QueryResult, TemporalQuery};
pub use temporal_store::{StoreError, TemporalStore};
