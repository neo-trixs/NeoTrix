//! Unified media capabilities — single source of truth for all media operations.
//!
//! Consolidates:
//! - Content-Type detection (was scattered across 4 locations)
//! - URL routing (was duplicated in nt_io_download + nt_stream)
//! - Streaming pipeline (new: download→player end-to-end)
//! - Progress types (unified, replaces per-module definitions)
//!
//! Architecture:
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  nt_media                                                   │
//! │                                                             │
//! │  detect.rs  — MediaKind + 3-layer sniffing (magic/CT/ext)  │
//! │  router.rs  — UrlScheme + TransportType + MediaRoute        │
//! │  streaming.rs — StreamingPipeline (HTTP/Magnet/FIFO/Player) │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod detect;
pub mod router;
pub mod streaming;

// Re-export key types
pub use detect::MediaKind;
pub use router::{UrlScheme, TransportType, MediaRoute};
pub use streaming::{
    StreamingPipeline, PipelineConfig, PipelineHandle,
    PipelineProgress, PipelineStatus, PipelineError,
};
