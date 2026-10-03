#[cfg(feature = "stealth-net")]
pub mod antidetect;
pub mod auth;
pub mod channel;
pub mod channel_adapter;
pub mod doctor;
pub mod extractors;
pub mod feed;
pub mod manager;
pub mod nt_login;
pub mod nt_payload_guard;
pub mod nt_selector_contract;
pub mod nt_x_browser;
pub mod probe;
pub mod traits;

pub use traits::{
    AuthFlow, AuthState, Author, Credentials, EngagementMetrics, ExtractorItem, ExtractorResult,
    FeedItem, FeedType, HttpPool, MediaItem, SocialAccessError, SocialAccessResult,
    SocialPlatform, SocialPlatformAdapter, SocialPost, SessionEntry,
    TrendingTopic, UnifiedPost, build_headers,
};

pub use channel::{Backend, BackendStatus, Channel, ChannelRegistry, Credential, ProbeResult, default_channels};
pub use channel_adapter::{ChannelAdapter, ChannelAdapterRegistry};
pub use feed::{FeedService, PlatformAdapterRegistry, PredictedAction, PredictedActions, Prediction, RankOutcome, UniversalRecommender};
pub use manager::{FeedResponse, SocialAccessManager, SessionPool};
pub use doctor::{DoctorReport, ChannelReport, BackendReport, run_doctor};
pub use nt_login::{
    LoginRegistry, LoginTarget, Observation, ProbeOutcome, SuccessProbe, default_registry as default_login_registry,
    evaluate as evaluate_login,
};
pub use nt_selector_contract::{PageShape, X_SELECTOR_CONTRACT};
pub use nt_x_browser::{XBrowserRetriever, XQuery};
pub use nt_payload_guard::{PayloadVerdict, classify as classify_payload};
pub use probe::{
    DEFAULT_PROBE_TIMEOUT, RunOutcome, command_exists, find_best_backend, get_version,
    probe_backends, probe_command, probe_command_with_timeout, run_with_timeout,
};
