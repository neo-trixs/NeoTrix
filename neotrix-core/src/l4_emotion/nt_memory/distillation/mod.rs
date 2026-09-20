#![forbid(unsafe_code)]

//! Memory Distillation — merge similar memories, extract key facts, cross-session persistence.
//!
//! Implements R-P117 (ADD-only), R-P118 (promotion thresholds), R-MEM09 (consolidation).
//!
//! Sub-modules:
//! - `distiller`: merge similar memories via cosine similarity, produce `DistilledMemory`
//! - `persist`: file-based JSON session persistence
//! - `cross_session`: find related memories across sessions
//! - `compressor`: keep most accessed, most recent, most novel

pub mod compressor;
pub mod cross_session;
pub mod distiller;
pub mod persist;

pub use compressor::MemoryCompressor;
pub use cross_session::{CrossSessionBridge, RelatedMemory};
pub use distiller::{DistilledMemory, MemoryDistiller};
pub use persist::{SessionInfo, SessionPersistence};
