//! Channel-based multi-backend routing architecture
//!
//! Absorbed from Agent-Reach's channel pattern:
//! - Each platform (Twitter, YouTube, Reddit, etc.) is a "Channel"
//! - Each Channel has ordered backend candidates (primary + fallbacks)
//! - Health probing determines which backend is active
//! - Zero-API-fee backends preferred (yt-dlp, Jina Reader, CLI tools)

use std::collections::HashMap;
use std::fmt;
use std::process::Command;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::traits::SocialPlatform;

// ─── Backend Status ──────────────────────────────────────────────────────

/// Backend health status after probing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendStatus {
    /// Backend is healthy and ready
    Ok,
    /// Backend has issues but may work (e.g., slow, degraded)
    Warn(String),
    /// Backend returned an error
    Error(String),
    /// Backend command/binary not found
    Missing,
    /// Probe timed out
    Timeout,
}

impl fmt::Display for BackendStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Warn(msg) => write!(f, "warn: {}", msg),
            Self::Error(msg) => write!(f, "error: {}", msg),
            Self::Missing => write!(f, "missing"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}

impl BackendStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Ok | Self::Warn(_))
    }
}

// ─── Probe Result ────────────────────────────────────────────────────────

/// Result of probing a backend command
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub status: BackendStatus,
    pub output: Option<String>,
    pub hint: Option<String>,
    pub latency_ms: u64,
}

impl ProbeResult {
    pub fn ok(output: String, latency_ms: u64) -> Self {
        Self {
            status: BackendStatus::Ok,
            output: Some(output),
            hint: None,
            latency_ms,
        }
    }

    pub fn warn(msg: String, latency_ms: u64) -> Self {
        Self {
            status: BackendStatus::Warn(msg.clone()),
            output: None,
            hint: Some(msg),
            latency_ms,
        }
    }

    pub fn error(msg: String, latency_ms: u64) -> Self {
        Self {
            status: BackendStatus::Error(msg.clone()),
            output: None,
            hint: Some(msg),
            latency_ms,
        }
    }

    pub fn missing(hint: String) -> Self {
        Self {
            status: BackendStatus::Missing,
            output: None,
            hint: Some(hint),
            latency_ms: 0,
        }
    }

    pub fn timeout() -> Self {
        Self {
            status: BackendStatus::Timeout,
            output: None,
            hint: Some("command timed out".into()),
            latency_ms: 0,
        }
    }
}

// ─── Backend ─────────────────────────────────────────────────────────────

/// A specific backend implementation for a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backend {
    /// Backend identifier (e.g., "yt-dlp", "jina-reader", "twitter-cli")
    pub name: String,
    /// Command to execute for health check
    pub probe_cmd: String,
    /// Arguments for health check probe
    pub probe_args: Vec<String>,
    /// Timeout for probe command
    pub probe_timeout: Duration,
    /// Whether this backend requires authentication
    pub requires_auth: bool,
    /// Cost tier: 0 = free, 1 = cheap, 2 = expensive
    pub cost_tier: u8,
    /// Priority weight for selection (higher = preferred)
    pub weight: u32,
}

impl Backend {
    pub fn new(name: impl Into<String>, probe_cmd: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            probe_cmd: probe_cmd.into(),
            probe_args: vec!["--version".into()],
            probe_timeout: Duration::from_secs(5),
            requires_auth: false,
            cost_tier: 0,
            weight: 100,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.probe_args = args;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.probe_timeout = timeout;
        self
    }

    pub fn with_auth(mut self, requires_auth: bool) -> Self {
        self.requires_auth = requires_auth;
        self
    }

    pub fn with_cost(mut self, tier: u8) -> Self {
        self.cost_tier = tier;
        self
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }
}

// ─── Channel ─────────────────────────────────────────────────────────────

/// A channel represents a platform with ordered backend candidates
///
/// Inspired by Agent-Reach's Channel pattern:
/// - `backends` is an ordered list of candidates (primary first, fallbacks after)
/// - Health probing iterates backends in order, first "ok" wins
/// - Fallback to first "warn" if no "ok" found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Channel name (e.g., "twitter", "youtube", "web")
    pub name: String,
    /// Platform this channel serves
    pub platform: SocialPlatform,
    /// Ordered backend candidates (primary first)
    pub backends: Vec<Backend>,
    /// Currently active backend (set after health check)
    pub active_backend: Option<String>,
    /// Tier: 0 = zero-config, 1 = needs free key, 2 = needs setup
    pub tier: u8,
    /// Last probe timestamp
    pub last_probed: Option<u64>,
}

impl Channel {
    pub fn new(name: impl Into<String>, platform: SocialPlatform, backends: Vec<Backend>) -> Self {
        Self {
            name: name.into(),
            platform,
            backends,
            active_backend: None,
            tier: 0,
            last_probed: None,
        }
    }

    /// Get ordered backends, optionally filtered by config overrides
    pub fn ordered_backends(&self, config: &HashMap<String, String>) -> Vec<&Backend> {
        // Check for backend override in config
        let override_key = format!("{}_backend", self.name.to_lowercase());
        let override_env = format!("{}_BACKEND", self.name.to_uppercase());

        if let Some(override_name) = config.get(&override_key)
            .or_else(|| config.get(&override_env))
        {
            // Return only the specified backend
            if let Some(backend) = self.backends.iter().find(|b| &b.name == override_name) {
                return vec![backend];
            }
        }

        // Sort by weight (descending) and cost_tier (ascending)
        let mut sorted: Vec<&Backend> = self.backends.iter().collect();
        sorted.sort_by(|a, b| {
            b.weight.cmp(&a.weight)
                .then(a.cost_tier.cmp(&b.cost_tier))
        });
        sorted
    }

    /// Check if this channel can handle a URL
    pub fn can_handle(&self, url: &str) -> bool {
        match self.platform {
            SocialPlatform::Twitter => {
                url.contains("x.com") || url.contains("twitter.com")
            }
            SocialPlatform::Reddit => {
                url.contains("reddit.com") || url.contains("redd.it")
            }
            SocialPlatform::Instagram => {
                url.contains("instagram.com")
            }
            SocialPlatform::TikTok => {
                url.contains("tiktok.com") || url.contains("vm.tiktok.com")
            }
            SocialPlatform::Youtube => {
                url.contains("youtube.com") || url.contains("youtu.be")
            }
            SocialPlatform::Linkedin => {
                url.contains("linkedin.com")
            }
            SocialPlatform::Other(ref name) => url.contains(name),
        }
    }

    /// Probe all backends and set active_backend
    pub fn probe(&mut self, config: &HashMap<String, String>) -> Vec<(String, ProbeResult)> {
        let ordered = self.ordered_backends(config);
        let mut results = Vec::new();
        let mut first_warn: Option<String> = None;

        for backend in ordered {
            let result = probe_backend(backend);
            let status = result.status.clone();
            results.push((backend.name.clone(), result));

            match status {
                BackendStatus::Ok => {
                    self.active_backend = Some(backend.name.clone());
                    self.last_probed = Some(now_ts());
                    return results;
                }
                BackendStatus::Warn(_) => {
                    if first_warn.is_none() {
                        first_warn = Some(backend.name.clone());
                    }
                }
                _ => continue,
            }
        }

        // Fallback to first warn if no ok found
        if let Some(name) = first_warn {
            self.active_backend = Some(name);
        }

        self.last_probed = Some(now_ts());
        results
    }
}

// ─── Channel Registry ────────────────────────────────────────────────────

/// Registry of all channels
pub struct ChannelRegistry {
    channels: HashMap<String, Channel>,
    config: HashMap<String, String>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            config: HashMap::new(),
        }
    }

    pub fn register(&mut self, channel: Channel) {
        self.channels.insert(channel.name.clone(), channel);
    }

    pub fn get(&self, name: &str) -> Option<&Channel> {
        self.channels.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Channel> {
        self.channels.get_mut(name)
    }

    pub fn all_channels(&self) -> Vec<&Channel> {
        self.channels.values().collect()
    }

    pub fn set_config(&mut self, key: String, value: String) {
        self.config.insert(key, value);
    }

    /// Probe all channels and return results
    pub fn probe_all(&mut self) -> HashMap<String, Vec<(String, ProbeResult)>> {
        let mut results = HashMap::new();
        let config = self.config.clone();

        for (name, channel) in &mut self.channels {
            let channel_results = channel.probe(&config);
            results.insert(name.clone(), channel_results);
        }

        results
    }

    /// Find channel that can handle a URL
    pub fn find_for_url(&self, url: &str) -> Option<&Channel> {
        self.channels.values().find(|c| c.can_handle(url))
    }

    /// Get active backend for a channel
    pub fn active_backend(&self, channel_name: &str) -> Option<&str> {
        self.channels.get(channel_name)?
            .active_backend
            .as_deref()
    }
}

// ─── Built-in Channel Definitions ────────────────────────────────────────

/// Create the default channel registry with Agent-Reach inspired channels
pub fn default_channels() -> ChannelRegistry {
    let mut registry = ChannelRegistry::new();

    // Twitter/X — multi-backend: twitter-cli → OpenCLI → bird CLI
    registry.register(Channel::new(
        "twitter",
        SocialPlatform::Twitter,
        vec![
            Backend::new("twitter-cli", "twitter")
                .with_args(vec!["--help".into()])
                .with_auth(true)
                .with_weight(100),
            Backend::new("opencli", "opencli")
                .with_args(vec!["twitter".into(), "--help".into()])
                .with_auth(true)
                .with_weight(80),
            Backend::new("bird", "bird")
                .with_args(vec!["--version".into()])
                .with_auth(true)
                .with_weight(60),
        ],
    ));

    // YouTube — zero-config via yt-dlp
    registry.register(Channel::new(
        "youtube",
        SocialPlatform::Youtube,
        vec![
            Backend::new("yt-dlp", "yt-dlp")
                .with_args(vec!["--version".into()])
                .with_cost(0)
                .with_weight(100),
            Backend::new("jina-reader", "curl")
                .with_args(vec!["-s".into(), "https://r.jina.ai/".into()])
                .with_cost(0)
                .with_weight(50),
        ],
    ));

    // Reddit — OpenCLI or rdt-cli
    registry.register(Channel::new(
        "reddit",
        SocialPlatform::Reddit,
        vec![
            Backend::new("opencli", "opencli")
                .with_args(vec!["reddit".into(), "--help".into()])
                .with_auth(true)
                .with_weight(100),
            Backend::new("rdt-cli", "rdt")
                .with_args(vec!["--version".into()])
                .with_auth(true)
                .with_weight(80),
        ],
    ));

    // Instagram — OpenCLI
    registry.register(Channel::new(
        "instagram",
        SocialPlatform::Instagram,
        vec![
            Backend::new("opencli", "opencli")
                .with_args(vec!["instagram".into(), "--help".into()])
                .with_auth(true)
                .with_weight(100),
        ],
    ));

    // TikTok — yt-dlp or OpenCLI
    registry.register(Channel::new(
        "tiktok",
        SocialPlatform::TikTok,
        vec![
            Backend::new("yt-dlp", "yt-dlp")
                .with_args(vec!["--version".into()])
                .with_cost(0)
                .with_weight(100),
            Backend::new("opencli", "opencli")
                .with_args(vec!["tiktok".into(), "--help".into()])
                .with_auth(true)
                .with_weight(80),
        ],
    ));

    // LinkedIn — MCP or Jina
    registry.register(Channel::new(
        "linkedin",
        SocialPlatform::Linkedin,
        vec![
            Backend::new("jina-reader", "curl")
                .with_args(vec!["-s".into(), "https://r.jina.ai/".into()])
                .with_cost(0)
                .with_weight(100),
        ],
    ));

    // Web — Jina Reader (zero-config)
    registry.register(Channel::new(
        "web",
        SocialPlatform::Other("web".into()),
        vec![
            Backend::new("jina-reader", "curl")
                .with_args(vec!["-s".into(), "https://r.jina.ai/".into()])
                .with_cost(0)
                .with_weight(100),
        ],
    ));

    // GitHub — gh CLI
    registry.register(Channel::new(
        "github",
        SocialPlatform::Other("github".into()),
        vec![
            Backend::new("gh", "gh")
                .with_args(vec!["--version".into()])
                .with_cost(0)
                .with_weight(100),
        ],
    ));

    // Bilibili — bili-cli or OpenCLI
    registry.register(Channel::new(
        "bilibili",
        SocialPlatform::Other("bilibili".into()),
        vec![
            Backend::new("bili-cli", "bili")
                .with_args(vec!["--version".into()])
                .with_cost(0)
                .with_weight(100),
            Backend::new("opencli", "opencli")
                .with_args(vec!["bilibili".into(), "--help".into()])
                .with_cost(0)
                .with_weight(80),
        ],
    ));

    registry
}

// ─── Utilities ───────────────────────────────────────────────────────────

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Probe a single backend command
pub fn probe_backend(backend: &Backend) -> ProbeResult {
    let start = Instant::now();

    let output = Command::new(&backend.probe_cmd)
        .args(&backend.probe_args)
        .output();

    let latency = start.elapsed().as_millis() as u64;

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                ProbeResult::ok(stdout.trim().to_string(), latency)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if stderr.contains("not found") || stderr.contains("No such file") {
                    ProbeResult::missing(format!("{}: {}", backend.name, stderr))
                } else {
                    ProbeResult::error(stderr, latency)
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                ProbeResult::missing(format!("{}: command not found", backend.probe_cmd))
            } else {
                ProbeResult::error(e.to_string(), latency)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_can_handle() {
        let twitter = Channel::new("twitter", SocialPlatform::Twitter, vec![]);
        assert!(twitter.can_handle("https://x.com/user/status/123"));
        assert!(twitter.can_handle("https://twitter.com/user/status/123"));
        assert!(!twitter.can_handle("https://reddit.com/r/rust"));

        let youtube = Channel::new("youtube", SocialPlatform::Youtube, vec![]);
        assert!(youtube.can_handle("https://youtube.com/watch?v=abc"));
        assert!(youtube.can_handle("https://youtu.be/abc"));
        assert!(!youtube.can_handle("https://twitter.com/user"));
    }

    #[test]
    fn test_backend_status_is_healthy() {
        assert!(BackendStatus::Ok.is_healthy());
        assert!(BackendStatus::Warn("slow".into()).is_healthy());
        assert!(!BackendStatus::Error("failed".into()).is_healthy());
        assert!(!BackendStatus::Missing.is_healthy());
        assert!(!BackendStatus::Timeout.is_healthy());
    }

    #[test]
    fn test_probe_result_ok() {
        let result = ProbeResult::ok("1.0.0".into(), 50);
        assert_eq!(result.status, BackendStatus::Ok);
        assert_eq!(result.output.as_deref(), Some("1.0.0"));
        assert_eq!(result.latency_ms, 50);
    }

    #[test]
    fn test_registry_find_for_url() {
        let registry = default_channels();
        assert!(registry.find_for_url("https://x.com/user").is_some());
        assert!(registry.find_for_url("https://youtube.com/watch").is_some());
        assert!(registry.find_for_url("https://reddit.com/r/rust").is_some());
        assert!(registry.find_for_url("https://example.com").is_none());
    }

    #[test]
    fn test_probe_backend_missing() {
        let backend = Backend::new("nonexistent", "nonexistent_cmd_xyz");
        let result = probe_backend(&backend);
        assert_eq!(result.status, BackendStatus::Missing);
    }
}
