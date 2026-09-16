#[cfg(feature = "stealth-net")]
pub mod antidetect;
pub mod auth;
pub mod channel;
pub mod channel_adapter;
pub mod doctor;
pub mod extractors;
pub mod feed;
pub mod manager;
pub mod probe;
pub mod traits;

pub use traits::{
    AuthFlow, AuthState, Author, Credentials, EngagementMetrics, ExtractorItem, ExtractorResult,
    FeedItem, FeedType, HttpPool, MediaItem, SocialAccessError, SocialAccessResult,
    SocialPlatform, SocialPlatformAdapter, SocialPost, SessionEntry,
    TrendingTopic, UnifiedPost, build_headers,
};

pub use channel::{Backend, BackendStatus, Channel, ChannelRegistry, ProbeResult, default_channels};
pub use channel_adapter::{ChannelAdapter, ChannelAdapterRegistry};
pub use doctor::{DoctorReport, ChannelReport, BackendReport, run_doctor};
pub use probe::{probe_command, probe_backends, find_best_backend, command_exists, get_version};
