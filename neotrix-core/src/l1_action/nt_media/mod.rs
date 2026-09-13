//! Unified media capabilities — single source of truth for all media operations.
//!
//! Consolidates:
//! - Content-Type detection (was scattered across 4 locations)
//! - URL routing (was duplicated in nt_io_download + nt_stream)
//! - Streaming pipeline (new: download→player end-to-end)
//! - Progress types (unified, replaces per-module definitions)

pub mod audio_decode;
pub mod auth;
pub mod detect;
pub mod download_progress;
pub mod hls;
pub mod persistence;
pub mod playback;
pub mod router;
pub mod streaming;
pub mod thumbnail;
pub mod yt_extract;

pub use auth::{AuthConfig, AuthStrategy, CookieEntry, CookieJar};
pub use detect::MediaKind;
pub use download_progress::{format_bytes, DownloadProgress, ProgressConfig};
pub use persistence::{
    check_disk_space, ChunkDownloadStatus, ChunkState, compute_sha256_streaming, DownloadStatus,
    ResumeValidation, SidecarState,
};
pub use router::{MediaRoute, TransportType, UrlScheme};
pub use streaming::{
    PipelineConfig, PipelineError, PipelineHandle, PipelineProgress, PipelineStatus, RetryPolicy,
    StallDetector, StreamingPipeline,
};
