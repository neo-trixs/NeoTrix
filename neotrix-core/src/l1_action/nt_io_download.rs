//! NeoTrix download engine — thin wrapper delegating to streaming.rs.
//!
//! All download logic now lives in `nt_media::streaming` (R-P42).
//! This module re-exports the canonical types and provides a stable
//! public API surface for callers that were using `nt_io_download` directly.

pub use crate::l1_action::nt_media::streaming::{
    AggregateProgress, DownloadConfig, DownloadEngine, DownloadProgressData, DownloadStatus,
    DownloadTask, TaskHandle,
};

/// Alias for callers that expect `DownloadProgress` name.
pub type DownloadProgress = DownloadProgressData;

// Re-export sub-modules that callers may reference.
pub use crate::l1_action::nt_media::router::{is_huggingface_url, UrlScheme};
