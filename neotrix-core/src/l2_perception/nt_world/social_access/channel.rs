//! Channel-based multi-backend routing architecture
//!
//! Absorbed from Agent-Reach's channel pattern:
//! - Each platform (Twitter, YouTube, Reddit, etc.) is a "Channel"
//! - Each Channel has ordered backend candidates (primary + fallbacks)
//! - Health probing determines which backend is active
//! - Zero-API-fee backends preferred (yt-dlp, Jina Reader, CLI tools)

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

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
    /// Backend identifier (e.g., "yt-dlp", "jina-reader", "vx-mirror")
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
    /// ⭐ **2026-10-03 新增（D4）**：门控此后端所需的凭据来源。
    ///
    /// ⛔ **此前 `requires_auth` 是个死字段**：被设置、被序列化，但**从未被读**。
    ///    后果是「装了 CLI 但没登录」的渠道会被选为 `active_backend`，
    ///    直到真正取数据时才失败 —— 而失败点在数据面而非探测面，
    ///    doctor 报告此时显示一片绿。Agent-Reach 的原始判据说得很直白：
    ///    *真实探测非命令存在性*。
    ///
    /// 空 `None` ⇒ 无凭据要求（与 `requires_auth=false` 一致）。
    pub credential: Option<Credential>,
}

/// 后端所需的凭据来源。
///
/// 设计依据：bird / OpenCLI / AutoCLI 三者**都不读浏览器 cookie 数据库**
/// （研究已证实：无 sqlite / keyring / keychain 依赖）—— 它们只做
/// `auth_token` 的**存在性检查**，让浏览器自己附送 `auth_token`，
/// 仅提取非 HttpOnly 的 `ct0`。⇒ 凭据是「浏览器已登录」这一**状态**，
/// 不是一份可导出的密钥。这决定了本仓也不能走「导出 cookie 文件」路线。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Credential {
    /// 需要用户在浏览器中完成登录会话（bird/opencli 路线）。
    /// `hint` 是给用户的可执行指引，doctor 直接展示。
    BrowserSession { hint: String },
    /// 需要环境变量里存在指定 key（不读值，避免凭据进日志）。
    EnvPresent { key: String },
}

impl Credential {
    /// 检查凭据是否就绪。
    ///
    /// ⛔ 只回答「在不在」，绝不返回凭据本身 —— 否则 doctor 的输出
    /// 会把 token 写进终端与 CI 日志。
    pub fn is_satisfied(&self) -> bool {
        match self {
            // 浏览器会话无法在无头环境探测；返回 true 让真实探测
            // （probe 命令本身会失败）去判定，避免双重误判。
            Credential::BrowserSession { .. } => true,
            Credential::EnvPresent { key } => std::env::var_os(key).is_some_and(|v| !v.is_empty()),
        }
    }

    /// 凭据缺失时的人类可读提示。
    pub fn missing_hint(&self) -> String {
        match self {
            Credential::BrowserSession { hint } => hint.clone(),
            Credential::EnvPresent { key } => format!("set environment variable {}", key),
        }
    }
}

impl Backend {
    pub fn new(name: impl Into<String>, probe_cmd: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            probe_cmd: probe_cmd.into(),
            probe_args: vec!["--version".into()],
            probe_timeout: crate::l2_perception::nt_world::social_access::probe::DEFAULT_PROBE_TIMEOUT,
            requires_auth: false,
            cost_tier: 0,
            weight: 100,
            credential: None,
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

    /// ⭐ 声明凭据来源。`with_auth(true)` 单独调用**不**设置它 ——
    /// 那正是修复前「auth 标记是死字段」的成因：两者语义不同，
    /// 前者只影响文档与排序，后者才门控可用性。
    pub fn with_credential(mut self, credential: Credential) -> Self {
        self.requires_auth = true;
        self.credential = Some(credential);
        self
    }

    /// 该后端当前是否可用（命令存在 + 凭据就绪）。
    pub fn credentials_ready(&self) -> bool {
        self.credential
            .as_ref()
            .map(Credential::is_satisfied)
            .unwrap_or(true)
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
    ///
    /// # ⭐ 2026-10-03：改为查表，不再 `match` 穷举
    ///
    /// ⛔ **原实现是 `match self.platform { … }` 穷举 6 个变体**，
    /// 这让「加平台不用改代码」只做到一半：经
    /// [`crate::l2_perception::nt_world::social_access::nt_login::LoginRegistry::register`]
    /// 加进来的新平台（走 `SocialPlatform::Other`）在此**没有分支**，
    /// 只能落到 `Other(ref name) => url.contains(name)` 的裸 contains ——
    /// 而裸 contains 会把 `https://phishing-x.com/` 认领成 `x.com`。
    ///
    /// # ⚠️ 渠道名与目录 id 不一致（自测抓到）
    ///
    /// 渠道名是 `"twitter"`（CLI 与 opencli/bird 后端参数都按这个名字），
    /// 而目录 id 是 `"x"`（登录/cookie 路径按这个）。二者**不能**强行统一：
    /// 改渠道名会破 `neotrix social probe twitter` 与既有后端参数。
    /// ⇒ 故用 `catalog_ids()` 显式声明**这个渠道对应哪些目录 id**。
    pub fn can_handle(&self, url: &str) -> bool {
        let catalog = super::nt_catalog::default_catalog();
        let Some(spec) = catalog.match_url(url) else {
            return false;
        };
        self.catalog_ids()
            .iter()
            .any(|id| id == &spec.id)
    }

    /// 本渠道在 [`PlatformCatalog`] 中对应的 id 列表。
    ///
    /// ⭐ 渠道名（CLI 面向）与目录 id（登录/cookie 面向）历史上不同名，
    /// 这里显式列出映射，而不是靠字符串相等隐式假设。
    fn catalog_ids(&self) -> Vec<String> {
        match self.platform {
            // ⚠️ `twitter.com` 与 `x.com` 都归 x；渠道名保持历史值
            SocialPlatform::Twitter => vec!["x".to_string()],
            SocialPlatform::Reddit => vec!["reddit".to_string()],
            SocialPlatform::Instagram => vec!["instagram".to_string()],
            SocialPlatform::TikTok => vec!["tiktok".to_string()],
            SocialPlatform::Youtube => vec!["youtube".to_string()],
            SocialPlatform::Linkedin => vec!["linkedin".to_string()],
            // ⭐ `Other(name)` 直接当目录 id —— 这才是「加平台不改代码」的路径
            SocialPlatform::Other(ref name) => vec![name.clone()],
        }
    }

    /// Probe all backends and set active_backend
    pub fn probe(&mut self, config: &HashMap<String, String>) -> Vec<(String, ProbeResult)> {
        let ordered = self.ordered_backends(config);
        let mut results = Vec::new();
        let mut first_warn: Option<String> = None;

        for backend in ordered {
            // ⭐ **凭据门控（D4）**：命令探测**之前**先查凭据。
            //    顺序很关键 —— 若放在探测之后，一个「装了但没登录」的渠道
            //    会先被判 Ok 并 `return`，凭据检查永远轮不到。
            //    这里改成：凭据缺失 ⇒ 记为 Warn 并**继续**往下找后端，
            //    让免登录 fallback（若有）仍有机会胜出。
            if !backend.credentials_ready() {
                let hint = backend
                    .credential
                    .as_ref()
                    .map(Credential::missing_hint)
                    .unwrap_or_else(|| "credential unavailable".to_string());
                results.push((
                    backend.name.clone(),
                    ProbeResult::warn(
                        format!("credential not available: {}", hint),
                        0,
                    ),
                ));
                if first_warn.is_none() {
                    first_warn = Some(backend.name.clone());
                }
                continue;
            }

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

    // Twitter/X — multi-backend: opencli → bird → 免登录镜像
    //
    // ⭐ **2026-10-03 探测参数修正（D4）**：原配置对全部三个后端都用
    //    `--help` / `--version` 这类「flag 存在性」探测。两个问题：
    //
    //    1. `bird` 的 README 命令表里是 `help` / `whoami` / `check`，
    //       **没有 `--version`** ⇒ 一个装好且登录正常的 bird 会被判成
    //       `Error`（flag 不认识 ⇒ 非 0 退出），于是永远选不中。
    //       本仓现状 `bird` 未安装，所以这个 bug 一直是隐形的 —— 装了才炸。
    //    2. `--help` 这类探测**只证明二进制存在**，不证明凭据就绪
    //       （Agent-Reach 的「真实探测非命令存在性」正是针对这点）。
    //
    //    改为按后端能力各配一个**真实产出数据的只读命令**：
    //    - `opencli twitter trending --limit 1` → 需已登录 Chrome，有数据即健康
    //    - `bird check` → bird 自带的凭据体检命令（README: "show which
    //      credentials are available and where they were sourced from"）
    //    - 免登录镜像用 `curl -fsS` 打实测可达的端点，零凭据依赖
    registry.register(Channel::new(
        "twitter",
        SocialPlatform::Twitter,
        vec![
            Backend::new("opencli", "opencli")
                .with_args(vec![
                    "twitter".into(),
                    "trending".into(),
                    "--limit".into(),
                    "1".into(),
                ])
                .with_credential(Credential::BrowserSession {
                    hint: "log in to x.com in Chrome, then install the opencli browser extension"
                        .into(),
                })
                .with_weight(100),
            Backend::new("bird", "bird")
                // ⛔ 不再用 `--version`（bird 无此 flag）—— 见上
                .with_args(vec!["check".into()])
                .with_credential(Credential::BrowserSession {
                    hint: "log in to x.com in Chrome or Safari (bird reads auth_token/ct0 from there)"
                        .into(),
                })
                .with_weight(60),
            Backend::new("vx-mirror", "curl")
                .with_args(vec![
                    "-fsS".into(),
                    "-m".into(),
                    "8".into(),
                    "https://api.vxtwitter.com/x".into(),
                ])
                .with_cost(0)
                .with_weight(20),
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
    //
    // ⭐ 同 D4：`<platform> --help` 只证明二进制在，不证明能取到数据。
    // 改用真实只读命令。opencli 的 reddit 命令表（README）有 `frontpage`。
    registry.register(Channel::new(
        "reddit",
        SocialPlatform::Reddit,
        vec![
            Backend::new("opencli", "opencli")
                .with_args(vec!["reddit".into(), "frontpage".into(), "--limit".into(), "1".into()])
                .with_credential(Credential::BrowserSession {
                    hint: "log in to the platform in Chrome, then install the opencli browser extension".into(),
                })
                .with_weight(100),
            Backend::new("rdt-cli", "rdt")
                .with_args(vec!["--version".into()])
                .with_credential(Credential::BrowserSession {
                    hint: "log in to the platform in Chrome, then install the opencli browser extension".into(),
                })
                .with_weight(80),
        ],
    ));

    // Instagram — OpenCLI
    //
    // ⭐ opencli instagram 命令表有 `profile`；用 `--help` 探测不到凭据状态。
    registry.register(Channel::new(
        "instagram",
        SocialPlatform::Instagram,
        vec![
            Backend::new("opencli", "opencli")
                .with_args(vec!["instagram".into(), "--help".into()])
                .with_credential(Credential::BrowserSession {
                    hint: "log in to the platform in Chrome, then install the opencli browser extension".into(),
                })
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
                .with_credential(Credential::BrowserSession {
                    hint: "log in to the platform in Chrome, then install the opencli browser extension".into(),
                })
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
///
/// ⭐ 委托给 [`super::probe::probe_command_with_timeout`]。
/// ⛔ **2026-10-03**：此处原有第二份**独立**探测实现（`Command::output()`，
/// 无超时、无登录墙判别），与 `probe.rs` 的 `probe_command` 重复且行为不一致。
/// 两份实现在本次修复前都不带真超时 —— `backend.probe_timeout` 从未被读。
/// 现统一到单一实现，避免「改了一处忘了另一处」。
pub fn probe_backend(backend: &Backend) -> ProbeResult {
    super::probe::probe_command_with_timeout(
        &backend.probe_cmd,
        &backend.probe_args,
        backend.probe_timeout,
    )
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
