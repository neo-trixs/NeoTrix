//! session — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::collections::HashMap;

use super::cookies::{AuthState, CookieJar};
use super::types::{BackendKind, PageSnapshot};

/// Browser session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserSession {
    /// Session ID
    pub id: String,
    /// Current URL
    pub current_url: String,
    /// Page title
    pub title: Option<String>,
    /// Cookies（兼容旧字段：jar 的扁平视图）
    pub cookies: HashMap<String, String>,
    /// Local storage
    pub local_storage: HashMap<String, String>,
    /// Session storage
    pub session_storage: HashMap<String, String>,
    /// Open tabs
    pub tabs: Vec<Tab>,
    /// Current tab index
    pub current_tab: usize,
    /// Created at
    pub created_at: String,
    /// 创建毫秒戳（TTL 计算用，可序列化）
    pub created_ms: u64,
    /// 已执行动作计数（预算用）
    pub actions_taken: u64,
    /// 后端种类（会话级锁定，防混用污染）
    pub backend: BackendKind,
    /// 会话级 UA（创建时锁定：逐请求轮换是 bot 强信号，P0）
    pub user_agent: String,
    /// 会话认证（None = 匿名；401/403 触发重载重试，过期提前拒收）
    pub auth: Option<AuthState>,
    /// 当前页面快照
    pub snapshot: Option<PageSnapshot>,
    /// 自研 CookieJar（真凭据只活在这里）
    pub jar: CookieJar,
    /// 待提交表单值（input name → value）
    pub pending_fills: HashMap<String, String>,
    /// 待上传文件（input name → 本地路径表；提交时 multipart）
    pub pending_uploads: HashMap<String, Vec<String>>,
    /// 导航历史（后退/前进栈）
    pub history_back: Vec<String>,
    pub history_fwd: Vec<String>,
}

/// Browser tab
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tab {
    /// Tab index
    pub index: usize,
    /// Current URL
    pub url: String,
    /// Page title
    pub title: Option<String>,
    /// Whether tab is active
    pub active: bool,
}

/// Browser configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserConfig {
    /// User agent string (`None` = 每次请求从自研池轮换)
    pub user_agent: Option<String>,
    /// Viewport width
    pub viewport_width: u32,
    /// Viewport height
    pub viewport_height: u32,
    /// Whether to enable JavaScript（仅 ChromeHeadless 真生效）
    pub javascript_enabled: bool,
    /// Whether to accept cookies
    pub accept_cookies: bool,
    /// Proxy settings
    pub proxy: Option<String>,
    /// Timeout in milliseconds（全局动作上限，R-P38）
    pub timeout_ms: u64,
    /// Whether to take screenshots on actions
    pub auto_screenshot: bool,
    /// 后端选择（默认 Mock，零网络）
    pub backend: BackendKind,
    /// 同一 host 最小抓取间隔（礼貌限速，默认 1000ms）
    pub min_interval_ms: u64,
    /// 是否遵守 robots.txt（默认 true；抓取失败视为放行并缓存空规则）
    pub respect_robots: bool,
    /// 域名 allowlist（默认 None = 全放行；Some(list) = 默认拒绝，未列名直接拒收）
    pub allowed_domains: Option<Vec<String>>,
    /// 单会话最大动作数（默认 None；超限后新动作直接拒收，防 runaway loop）
    pub max_actions_per_session: Option<u64>,
    /// 会话 TTL 秒（默认 None；创建超期后新动作直接拒收）
    pub session_ttl_secs: Option<u64>,
    /// 历史保留策略（默认 Full；MetadataOnly 只留长度，防 secrets 进日志）
    pub history_retention: HistoryRetention,
    /// Cdp 复用 profile 目录（默认 None = chromiumoxide-runner 临时 profile）。
    /// Some(dir) 时 CDP 以该 profile 启动，继承其中登录态（Cookie/会话）；
    /// 须先退出占用该目录的 Chrome（profile 文件锁），否则启动失败如实报错。
    /// 例：NT_BROWSE_PROFILE=$HOME/Library/Application\ Support/Google/Chrome
    pub profile_dir: Option<String>,
}

/// 历史保留策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum HistoryRetention {
    /// 全量输出（调试用；可能含页面敏感信息）
    #[default]
    Full,
    /// 仅元数据（output 截断 200 字符；审计用 AuditEvent）
    MetadataOnly,
}


impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            user_agent: None,
            viewport_width: 1280,
            viewport_height: 720,
            javascript_enabled: true,
            accept_cookies: true,
            proxy: None,
            timeout_ms: 30000,
            auto_screenshot: false,
            backend: BackendKind::Mock,
            min_interval_ms: 1000,
            respect_robots: true,
            allowed_domains: None,
            max_actions_per_session: None,
            session_ttl_secs: None,
            history_retention: HistoryRetention::Full,
            profile_dir: None,
        }
    }
}

/// Browser statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserStats {
    pub active_sessions: usize,
    pub total_actions: usize,
    pub successful_actions: usize,
    pub failed_actions: usize,
    pub avg_action_duration_ms: f64,
}

// ============================================================================
// DOM 提取（纯函数，可单测）
// ============================================================================
