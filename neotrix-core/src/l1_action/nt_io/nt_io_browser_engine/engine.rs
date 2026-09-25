//! engine — BrowserEngine 门面：struct 定义 + 子模块重导出，行为零变更.
//! 拆分：nt_politeness / nt_http / nt_cdp / nt_session / nt_state（从 engine.rs 纯搬移）.

use std::collections::HashMap;
use std::sync::Arc;

use reqwest::Client;
#[cfg(feature = "stealth-net")]
use chromiumoxide::{Browser, Page};

use super::policy::AuditEvent;
use super::session::{BrowserConfig, BrowserSession};
use super::types::{BrowserAction, BrowserResult};
use nt_politeness::Politeness;

#[path = "engine/nt_politeness.rs"]
pub mod nt_politeness;
#[path = "engine/nt_http.rs"]
pub mod nt_http;
#[path = "engine/nt_cdp.rs"]
pub mod nt_cdp;
#[path = "engine/nt_session.rs"]
pub mod nt_session;
#[path = "engine/nt_state.rs"]
pub mod nt_state;

// ============================================================================
// Browser Engine
// ============================================================================

/// Built-in browser engine for agent web tasks（自研最小无头内核）
pub struct BrowserEngine {
    /// Active sessions
    pub(crate) sessions: Arc<tokio::sync::RwLock<HashMap<String, BrowserSession>>>,
    /// Configuration
    config: BrowserConfig,
    /// Action history
    history: Arc<tokio::sync::RwLock<Vec<(String, BrowserAction, BrowserResult)>>>,
    /// 审计事件（无正文，只留元数据）
    audit: Arc<tokio::sync::RwLock<Vec<AuditEvent>>>,
    /// 引擎级共享 HTTP 客户端（连接池复用，P1；代理在构建期注入）
    client: Client,
    /// 礼貌爬取状态（站点限速 + robots 缓存，P1）
    polite: Arc<tokio::sync::RwLock<Politeness>>,
    /// CDP 浏览器单例（懒启动，需 stealth-net 特性；Browser 自身非 Clone，外包 Arc）
    #[cfg(feature = "stealth-net")]
    cdp_browser: Arc<tokio::sync::Mutex<Option<Arc<Browser>>>>,
    /// 会话 → CDP 页面（Page 不可序列化，独立于 BrowserSession 存放）
    #[cfg(feature = "stealth-net")]
    cdp_pages: Arc<tokio::sync::RwLock<HashMap<String, Page>>>,
}
