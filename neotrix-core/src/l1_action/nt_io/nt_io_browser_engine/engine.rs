//! engine — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use scraper::{Html, Selector};
#[cfg(feature = "stealth-net")]
use chromiumoxide::{Browser, BrowserConfig as CdpConfig, Page};
#[cfg(feature = "stealth-net")]
use futures::StreamExt;
use super::cookies::{AuthConfig, AuthSource, AuthState, CookieJar, now_ms};
use super::{MAX_HISTORY, MAX_NAV_STACK};
use super::error::BrowserError;
use super::fetch::{http_client_for, http_get, http_submit, multipart_submit, parse_page, parse_robots_disallows, pick_ua, render_snapshot_text, robots_denied};
use super::js::{js_select_option, js_set_checked, pick_form};
use super::policy::{AuditEvent, action_kind, domain_allowed, parse_retry_after_secs, ssrf_refused};
use super::session::{BrowserConfig, BrowserSession, BrowserStats, HistoryRetention, Tab};
use super::types::{BackendKind, BrowserAction, BrowserResult, PageSnapshot, ScrollDirection, verify_for};

impl BrowserEngine {
    pub(crate) async fn polite_wait(&self, url: &str) -> Result<(), BrowserError> {
        let parsed = url::Url::parse(url)
            .map_err(|e| BrowserError::ActionFailed(format!("bad url: {e}")))?;
        let host = parsed.host_str().unwrap_or("").to_string();
        if host.is_empty() {
            return Ok(());
        }
        // 策略门：SSRF 常闭 + allowlist 默认拒绝（ single choke point：所有后端导航必经）
        if ssrf_refused(&host) {
            return Err(BrowserError::SsrfRefused(host));
        }
        if let Some(list) = self.config.allowed_domains.as_ref() {
            if !domain_allowed(&host, list) {
                return Err(BrowserError::DomainDenied(host));
            }
        }
        // 429 冷却：周期内直接拒收，不再打请求
        {
            let polite = self.polite.read().await;
            if let Some(until) = polite.cooldowns.get(&host) {
                if Instant::now() < *until {
                    let left = until
                        .duration_since(Instant::now())
                        .as_millis() as u64;
                    return Err(BrowserError::CoolingDown(left));
                }
            }
        }
        if self.config.respect_robots {
            let rules = self.robots_for(&parsed).await;
            if robots_denied(&rules, parsed.path()) {
                return Err(BrowserError::ActionFailed(format!(
                    "robots.txt disallows {url}"
                )));
            }
        }
        let wait = {
            let polite = self.polite.read().await;
            match polite.last_fetch.get(&host) {
                Some(last) => {
                    Duration::from_millis(self.config.min_interval_ms)
                        .saturating_sub(last.elapsed())
                }
                None => Duration::ZERO,
            }
        };
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
        Ok(())
    }

    pub(crate) async fn record_fetch(&self, final_url: &str) {
        if let Ok(parsed) = url::Url::parse(final_url) {
            if let Some(host) = parsed.host_str() {
                let mut polite = self.polite.write().await;
                polite.last_fetch.insert(host.to_string(), Instant::now());
            }
        }
    }

    /// 429 落盘：记域名冷却（调用方返回 RateLimited，不自动重试，由 Agent 决策）
    pub(crate) async fn record_rate_limited(&self, url: &str, retry_after_secs: u64) {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                let mut polite = self.polite.write().await;
                polite.cooldowns.insert(
                    host.to_string(),
                    Instant::now() + Duration::from_secs(retry_after_secs),
                );
            }
        }
    }

    pub(crate) async fn robots_for(&self, url: &url::Url) -> Vec<String> {
        let host = url.host_str().unwrap_or("").to_string();
        {
            let polite = self.polite.read().await;
            if let Some(entry) = polite.robots.get(&host) {
                if entry.fetched_at.elapsed() < ROBOTS_TTL {
                    return entry.rules.clone();
                }
            }
        }
        // 未命中：直抓 /robots.txt（不走礼貌递归，10s 上限，失败放行）
        let robots_url = format!("{}://{}/robots.txt", url.scheme(), host);
        let resp = self
            .client
            .get(&robots_url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .ok()
            .filter(|r| r.status().is_success());
        let mut rules = Vec::new();
        if let Some(r) = resp {
            if let Ok(body) = r.text().await {
                rules = parse_robots_disallows(&body);
            }
        }
        {
            let mut polite = self.polite.write().await;
            polite.robots.insert(
                host,
                RobotsEntry {
                    rules: rules.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }
        rules
    }
}

// ============================================================================
// 后端：Chrome headless（真渲染，无 profile）
// ============================================================================

pub(crate) fn chrome_path() -> String {
    if cfg!(target_os = "macos") {
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string()
    } else if cfg!(target_os = "windows") {
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string()
    } else {
        "google-chrome".to_string()
    }
}

/// headless 渲染（禁用 --user-data-dir：见模块头 R-P38 注释）
pub(crate) fn chrome_render(
    url: &str,
    budget_ms: u64,
    screenshot_path: Option<&std::path::Path>,
    width: u32,
    height: u32,
) -> Result<String, BrowserError> {
    let mut cmd = std::process::Command::new(chrome_path());
    cmd.arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--no-first-run")
        .arg("--disable-background-networking")
        .arg("--disable-sync")
        .arg("--mute-audio")
        .arg("--disable-features=ChromeWhatsNewUI")
        .arg("--disable-component-update")
        .arg("--disable-client-side-phishing-detection")
        // 2026-09-22：mock 钥匙串，headless 永不弹系统密码框（见模块头）
        .arg("--use-mock-keychain")
        .arg("--timeout=30000")
        .arg(format!("--virtual-time-budget={budget_ms}"))
        .arg("--dump-dom");
    if let Some(path) = screenshot_path {
        cmd.arg(format!("--screenshot={}", path.display()));
        cmd.arg(format!("--window-size={width},{height}"));
    }
    cmd.arg(url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut child = cmd
        .spawn()
        .map_err(|e| BrowserError::BackendUnavailable(format!("chrome spawn: {e}")))?;
    let deadline =
        std::time::Instant::now() + Duration::from_millis(budget_ms + 3000);
    loop {
        match child
            .try_wait()
            .map_err(|e| BrowserError::ActionFailed(format!("chrome wait: {e}")))?
        {
            Some(status) => {
                let mut stdout = String::new();
                use std::io::Read;
                let _ = child
                    .stdout
                    .take()
                    .and_then(|mut o| o.read_to_string(&mut stdout).ok());
                if !status.success() {
                    return Err(BrowserError::ActionFailed(format!(
                        "chrome exit {}",
                        status.code().unwrap_or(-1)
                    )));
                }
                if stdout.len() < 80 {
                    return Err(BrowserError::ActionFailed("empty page".to_string()));
                }
                return Ok(stdout);
            }
            None => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(BrowserError::Timeout);
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

/// headless 打印 PDF（禁用 --user-data-dir：同 R-P38 注释）
pub(crate) fn chrome_render_pdf(url: &str, dest: &std::path::Path, budget_ms: u64) -> Result<(), BrowserError> {
    let mut child = std::process::Command::new(chrome_path())
        .arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--no-first-run")
        .arg("--mute-audio")
        // 2026-09-22：mock 钥匙串，headless 永不弹系统密码框（见模块头）
        .arg("--use-mock-keychain")
        .arg("--no-pdf-header-footer")
        .arg(format!("--print-to-pdf={}", dest.display()))
        .arg(format!("--virtual-time-budget={budget_ms}"))
        .arg("--timeout=30000")
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| BrowserError::BackendUnavailable(format!("chrome spawn: {e}")))?;
    let deadline =
        std::time::Instant::now() + Duration::from_millis(budget_ms + 3000);
    loop {
        match child
            .try_wait()
            .map_err(|e| BrowserError::ActionFailed(format!("chrome wait: {e}")))?
        {
            Some(status) => {
                if !status.success() {
                    return Err(BrowserError::ActionFailed(format!(
                        "chrome exit {}",
                        status.code().unwrap_or(-1)
                    )));
                }
                if !dest.exists() {
                    return Err(BrowserError::ActionFailed(
                        "pdf not produced".to_string(),
                    ));
                }
                return Ok(());
            }
            None => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(BrowserError::Timeout);
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

/// 状态文件格式版本（Load 校验，不一致直接报错不猜）
pub(crate) const STATE_FORMAT_VERSION: u32 = 1;

/// 持久化信封（token 脱敏后存）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedSession {
    version: u32,
    saved_at_ms: u64,
    session: BrowserSession,
}

/// 下载上限 50MB
pub(crate) const MAX_DOWNLOAD_BYTES: usize = 50 * 1024 * 1024;

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

/// 礼貌爬取状态
#[derive(Debug, Default)]
pub(crate) struct Politeness {
    last_fetch: HashMap<String, Instant>,
    robots: HashMap<String, RobotsEntry>,
    /// 域名冷却截止（429 触发）
    cooldowns: HashMap<String, Instant>,
}

#[derive(Debug, Clone)]
pub(crate) struct RobotsEntry {
    rules: Vec<String>,
    fetched_at: Instant,
}

pub(crate) const ROBOTS_TTL: Duration = Duration::from_secs(600);

impl BrowserEngine {
    /// 严格构造：代理 URL 非法等直接报错
    pub fn try_new(config: BrowserConfig) -> Result<Self, BrowserError> {
        let client = http_client_for(&config)?;
        Ok(Self {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            config,
            history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            audit: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            client,
            polite: Arc::new(tokio::sync::RwLock::new(Politeness::default())),
            #[cfg(feature = "stealth-net")]
            cdp_browser: Arc::new(tokio::sync::Mutex::new(None)),
            #[cfg(feature = "stealth-net")]
            cdp_pages: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    /// Create a new browser engine（代理非法时回退直连；严格校验请用 try_new）
    pub fn new(config: BrowserConfig) -> Self {
        match Self::try_new(config.clone()) {
            Ok(engine) => engine,
            Err(_) => {
                let fallback = BrowserConfig {
                    proxy: None,
                    ..config
                };
                // 回退路径的 build 必然成功（无代理 + 默认 redirect）
                Self::try_new(fallback).unwrap_or_else(|_| Self {
                    sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
                    config: BrowserConfig::default(),
                    history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
                    audit: Arc::new(tokio::sync::RwLock::new(Vec::new())),
                    client: Client::new(),
                    polite: Arc::new(tokio::sync::RwLock::new(Politeness::default())),
                    #[cfg(feature = "stealth-net")]
                    cdp_browser: Arc::new(tokio::sync::Mutex::new(None)),
                    #[cfg(feature = "stealth-net")]
                    cdp_pages: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
                })
            }
        }
    }

    /// Create with default config (Mock 后端：零网络)
    pub fn with_defaults() -> Self {
        Self::new(BrowserConfig::default())
    }

    /// Create a new browser session
    pub async fn create_session(&self) -> Result<String, BrowserError> {
        let session_id = format!("browser-{}", uuid::Uuid::new_v4());
        let session = BrowserSession {
            id: session_id.clone(),
            current_url: "about:blank".to_string(),
            title: None,
            cookies: HashMap::new(),
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
            tabs: vec![Tab {
                index: 0,
                url: "about:blank".to_string(),
                title: None,
                active: true,
            }],
            current_tab: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            created_ms: now_ms(),
            actions_taken: 0,
            backend: self.config.backend,
            // UA 会话级锁定：同会话始终同一指纹（P0）
            user_agent: pick_ua(self.config.user_agent.as_deref()),
            auth: None,
            snapshot: None,
            jar: CookieJar::new(),
            pending_fills: HashMap::new(),
            pending_uploads: HashMap::new(),
            history_back: Vec::new(),
            history_fwd: Vec::new(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    pub(crate) fn op_timeout(&self) -> Duration {
        Duration::from_millis(self.config.timeout_ms)
    }

    /// Execute a browser action（外层超时 = 内层 ×2 + 5s，R-P38）
    pub async fn execute(
        &self,
        session_id: &str,
        action: BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        let start = std::time::Instant::now();
        let outer = self
            .op_timeout()
            .saturating_mul(2)
            .saturating_add(Duration::from_secs(5));

        // 错误直接传播（调用方可区分 SessionNotFound/Timeout/动作失败）；
        // 动作级失败由各后端以 success=false 的 BrowserResult 表达并记入历史。
        let mut result = tokio::time::timeout(outer, self.dispatch(session_id, &action))
            .await
            .map_err(|_| BrowserError::Timeout)??;
        result.duration_ms = start.elapsed().as_millis() as u64;

        // 动作计数（预算用；会话没了就跳过）
        {
            let mut sessions = self.sessions.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                s.actions_taken = s.actions_taken.saturating_add(1);
            }
        }

        // Store in history（保留策略 + 审计事件，无正文）
        {
            let mut history = self.history.write().await;
            let mut stored = result.clone();
            if matches!(
                self.config.history_retention,
                HistoryRetention::MetadataOnly
            ) {
                stored.output = stored.output.chars().take(200).collect();
            }
            history.push((session_id.to_string(), action.clone(), stored));
            let len = history.len();
            if len > MAX_HISTORY {
                history.drain(0..len - MAX_HISTORY);
            }
        }
        {
            let kind = action_kind(&action).to_string();
            let error_kind = result.error.as_ref().map(|e| {
                e.chars().take(120).collect::<String>()
            });
            let mut audit = self.audit.write().await;
            audit.push(AuditEvent {
                ts_ms: now_ms(),
                session_id: session_id.to_string(),
                action_kind: kind,
                url: result.current_url.clone(),
                success: result.success,
                duration_ms: result.duration_ms,
                error_kind,
            });
            let len = audit.len();
            if len > MAX_HISTORY {
                audit.drain(0..len - MAX_HISTORY);
            }
        }

        Ok(result)
    }

    /// 审计事件（只读快照）
    pub async fn audit_log(&self) -> Vec<AuditEvent> {
        self.audit.read().await.clone()
    }

    pub(crate) async fn dispatch(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        // 会话存在性 + 后端读取（短锁）
        let backend = {
            let sessions = self.sessions.read().await;
            sessions
                .get(session_id)
                .map(|s| s.backend)
                .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?
        };
        // 过期预检（已知过期点已过：直接拒收，省一次无效请求）
        self.check_auth_fresh(session_id).await?;
        // 预算与 TTL（runaway loop 熔断）
        {
            let sessions = self.sessions.read().await;
            let s = sessions
                .get(session_id)
                .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
            if let Some(max) = self.config.max_actions_per_session {
                if s.actions_taken >= max {
                    return Err(BrowserError::BudgetExhausted(format!(
                        "{}/{} actions",
                        s.actions_taken, max
                    )));
                }
            }
            if let Some(ttl) = self.config.session_ttl_secs {
                if now_ms().saturating_sub(s.created_ms) > ttl.saturating_mul(1000) {
                    return Err(BrowserError::SessionTtlExpired);
                }
            }
        }
        // Tab 簿记与后端无关：三后端共用同一套真实现
        match action {
            BrowserAction::NewTab { url } => return self.new_tab(session_id, url.clone()).await,
            BrowserAction::SwitchTab { index } => {
                return self.switch_tab(session_id, *index).await
            }
            BrowserAction::CloseTab => return self.close_tab(session_id).await,
            // 状态与传输：引擎级实现，与后端无关
            BrowserAction::SaveState { path } => {
                return self.save_state(session_id, path.clone()).await
            }
            BrowserAction::LoadState { path } => {
                return self.load_state(path.clone()).await
            }
            BrowserAction::Download { url, path } => {
                return self.download(session_id, url.clone(), path.clone()).await
            }
            BrowserAction::Upload { selector, files } => {
                return self
                    .record_uploads(session_id, selector.clone(), files.clone())
                    .await
            }
            _ => {}
        }
        match backend {
            BackendKind::Mock => self.dispatch_mock(session_id, action).await,
            BackendKind::Http => self.dispatch_http(session_id, action).await,
            BackendKind::ChromeHeadless => self.dispatch_chrome(session_id, action).await,
            BackendKind::Cdp => self.dispatch_cdp(session_id, action).await,
        }
    }

    // -- Mock 后端：纯内存占位（保持旧单测 hermetic） ---------------------------

    pub(crate) async fn dispatch_mock(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        match action {
            BrowserAction::Navigate { url } => {
                let mut sessions = self.sessions.write().await;
                let session = sessions
                    .get_mut(session_id)
                    .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
                session.current_url = url.clone();
                if let Some(tab) = session.tabs.get_mut(session.current_tab) {
                    tab.url = url.clone();
                }
                Ok(BrowserResult::ok(
                    format!("Navigated to {url}"),
                    url.clone(),
                    None,
                    0,
                ))
            }
            BrowserAction::Click { selector } => Ok(BrowserResult::ok(
                format!("Clicked {selector}"),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::Type { selector, text } => Ok(BrowserResult::ok(
                format!("Typed '{text}' into {selector}"),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::Screenshot => Ok(BrowserResult::ok(
                "Screenshot saved to /tmp/screenshot.png".to_string(),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::GetContent => Ok(BrowserResult::ok(
                "<html><body>Page content</body></html>".to_string(),
                "about:blank".to_string(),
                Some("Page Title".to_string()),
                0,
            )),
            BrowserAction::SelectOption { selector, values } => Ok(BrowserResult::ok(
                format!("Selected {values:?} in {selector}"),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::Check { selector } => Ok(BrowserResult::ok(
                format!("Checked {selector}"),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::Uncheck { selector } => Ok(BrowserResult::ok(
                format!("Unchecked {selector}"),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::Press { selector, key } => Ok(BrowserResult::ok(
                format!("Pressed '{key}' on {selector}"),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::GetText { selector } => Ok(BrowserResult::ok(
                format!("Text of {selector}"),
                "about:blank".to_string(),
                None,
                0,
            )),
            BrowserAction::PrintPdf => Ok(BrowserResult::ok(
                "PDF saved to /tmp/page.pdf".to_string(),
                "about:blank".to_string(),
                None,
                0,
            )),
            _ => Ok(BrowserResult::ok(
                "Action executed".to_string(),
                "about:blank".to_string(),
                None,
                0,
            )),
        }
    }

    // -- 会话快照落盘 ----------------------------------------------------------

    pub(crate) async fn commit_snapshot(
        &self,
        session_id: &str,
        snap: PageSnapshot,
        push_history: bool,
    ) -> Result<(), BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        if push_history && session.current_url != snap.url {
            session.history_back.push(session.current_url.clone());
            if session.history_back.len() > MAX_NAV_STACK {
                session.history_back.remove(0);
            }
            session.history_fwd.clear();
        }
        session.current_url = snap.url.clone();
        session.title = snap.title.clone();
        if let Some(tab) = session.tabs.get_mut(session.current_tab) {
            tab.url = snap.url.clone();
            tab.title = snap.title.clone();
        }
        session.snapshot = Some(snap);
        Ok(())
    }

    pub(crate) fn blocking_timeout(&self) -> Duration {
        self.op_timeout()
    }

    // -- HTTP 后端 --------------------------------------------------------------

    pub(crate) async fn http_fetch_snapshot(
        &self,
        url: &str,
        jar: &CookieJar,
        ua: &str,
        auth: Option<(String, String)>,
    ) -> Result<(PageSnapshot, Vec<String>), BrowserError> {
        self.polite_wait(url).await?;
        let timeout = self.blocking_timeout();
        let fetched = match http_get(&self.client, url, jar, ua, auth, timeout).await {
            Err(BrowserError::RateLimited(secs)) => {
                self.record_rate_limited(url, secs).await;
                return Err(BrowserError::RateLimited(secs));
            }
            other => other?,
        };
        let applied_host = url::Url::parse(&fetched.final_url)
            .map(|u| u.host_str().unwrap_or("").to_string())
            .unwrap_or_default();
        let (title, text, links, forms, raw_html) =
            parse_page(&fetched.final_url, &fetched.body)?;
        Ok((
            PageSnapshot {
                url: fetched.final_url,
                title,
                text,
                links,
                forms,
                raw_html,
                http_status: Some(fetched.status),
            },
            // 返回 set-cookie 由调用方存入会话 jar（附带响应 host）
            fetched
                .set_cookies
                .into_iter()
                .map(|c| format!("{applied_host}\u{1f}{c}"))
                .collect(),
        ))
    }

    pub(crate) async fn apply_cookies(&self, session_id: &str, packed: Vec<String>) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            for item in packed {
                if let Some((host, raw)) = item.split_once('\u{1f}') {
                    session
                        .jar
                        .store_from_headers(host, std::iter::once(raw));
                }
            }
            // 同步扁平视图（兼容旧字段）
            session.cookies.clear();
            for (domain, bucket) in session
                .jar
                .entries
                .iter()
            {
                for c in bucket {
                    session
                        .cookies
                        .insert(format!("{}:{}", domain, c.name), c.value.clone());
                }
            }
        }
    }

    /// 401/403 自愈：热加载 token → 重试一次 → 仍败 → AuthExpired（带精确指引）
    pub(crate) async fn authed_navigate(
        &self,
        session_id: &str,
        url: &str,
    ) -> Result<(PageSnapshot, Vec<String>), BrowserError> {
        let (jar, ua) = self.session_http_ctx(session_id).await?;
        let auth = self.session_auth_header(session_id).await?;
        match self
            .http_fetch_snapshot(url, &jar, &ua, auth.clone())
            .await
        {
            Err(BrowserError::AuthRejected(_)) => {
                let auth2 = self.session_auth_header(session_id).await?;
                match self
                    .http_fetch_snapshot(url, &jar, &ua, auth2)
                    .await
                {
                    Err(BrowserError::AuthRejected(_)) => {
                        Err(BrowserError::AuthExpired {
                            hint: self.auth_hint(session_id).await,
                        })
                    }
                    other => other,
                }
            }
            other => other,
        }
    }

    pub(crate) async fn auth_hint(&self, session_id: &str) -> String {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .and_then(|s| s.auth.as_ref().map(|a| a.config.hint()))
            .unwrap_or_else(|| "token 被拒：请检查认证配置后重试".to_string())
    }

    /// HTTP 导航（独立函数：Click/Reload/回退直接复用，避免 async 递归）
    pub(crate) async fn navigate_http(
        &self,
        session_id: &str,
        url: &str,
    ) -> Result<BrowserResult, BrowserError> {
        let (snap, packed) = self.authed_navigate(session_id, url).await?;
        self.apply_cookies(session_id, packed).await;
        let out = render_snapshot_text(&snap);
        let verify = verify_for(url, &snap);
        let (curl, title) = (snap.url.clone(), snap.title.clone());
        self.commit_snapshot(session_id, snap, true).await?;
        self.record_fetch(&curl).await;
        Ok(BrowserResult::ok_verified(out, curl, title, 0, verify))
    }

    pub(crate) async fn dispatch_http(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        match action {
            BrowserAction::Navigate { url } => self.navigate_http(session_id, url).await,
            BrowserAction::GetContent => {
                let (url, snap) = self.session_snapshot(session_id).await?;
                match snap {
                    Some(s) => Ok(BrowserResult::ok(
                        render_snapshot_text(&s),
                        url,
                        s.title.clone(),
                        0,
                    )),
                    None => Err(BrowserError::ActionFailed(
                        "no page loaded; Navigate first".to_string(),
                    )),
                }
            }
            BrowserAction::Click { selector } => {
                let raw = self.session_raw_html(session_id).await?;
                let doc = Html::parse_document(&raw);
                // 仅支持有 href 的 <a>（诚实面：submit 请走 SubmitForm）
                let css = format!("a[href]{selector}");
                let target = Selector::parse(&css)
                    .ok()
                    .and_then(|sel| doc.select(&sel).next())
                    .and_then(|el| el.value().attr("href").map(str::to_string))
                    .or_else(|| {
                        Selector::parse(selector).ok().and_then(|sel| {
                            doc.select(&sel).next().and_then(|el| {
                                if el.value().name() == "a" {
                                    el.value().attr("href").map(str::to_string)
                                } else {
                                    None
                                }
                            })
                        })
                    });
                match target {
                    Some(href) => {
                        let base = self.session_url(session_id).await?;
                        let abs = url::Url::parse(&base)
                            .and_then(|b| b.join(&href))
                            .map(|u| u.to_string())
                            .unwrap_or(href);
                        return self.navigate_http(session_id, &abs).await;
                    }
                    None => Err(BrowserError::ActionFailed(format!(
                        "click: no navigable link matches '{selector}'; submit 控件请用 SubmitForm"
                    ))),
                }
            }
            BrowserAction::Type { selector, text } => {
                self.record_fill(session_id, selector.clone(), text.clone())
                    .await
            }
            BrowserAction::FillField { selector, value } => {
                self.record_fill(session_id, selector.clone(), value.clone())
                    .await
            }
            BrowserAction::SelectOption { selector, values } => {
                // 下拉单选取首值记录（多选提交时由服务端解释）
                let value = values.first().cloned().unwrap_or_default();
                self.record_fill(session_id, selector.clone(), value)
                    .await
            }
            BrowserAction::Check { selector } => {
                let (key, default) =
                    self.resolve_fill_key(session_id, selector).await?;
                let value = if default.is_empty() {
                    "on".to_string()
                } else {
                    default
                };
                self.record_fill(session_id, key, value).await
            }
            BrowserAction::Uncheck { selector } => {
                self.drop_fill(session_id, selector).await
            }
            BrowserAction::Press { selector, key } => {
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("press recorded: '{key}' on '{selector}'（Http 无焦点概念）"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::GetText { selector } => {
                let raw = self.session_raw_html(session_id).await?;
                let doc = Html::parse_document(&raw);
                let sel = Selector::parse(selector).map_err(|_| {
                    BrowserError::ActionFailed(format!("bad selector '{selector}'"))
                })?;
                let text = doc
                    .select(&sel)
                    .map(|el| {
                        el.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    })
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(text, url, title, 0))
            }
            BrowserAction::PrintPdf => Err(BrowserError::ActionFailed(
                "PrintPdf 需要 ChromeHeadless/Cdp 后端（Http 无渲染）".to_string(),
            )),
            BrowserAction::SubmitForm { selector } => {
                self.http_submit_form(session_id, selector.clone()).await
            }
            BrowserAction::Reload => {
                let url = self.session_url(session_id).await?;
                return self.navigate_http(session_id, &url).await;
            }
            BrowserAction::GoBack => {
                let prev = self.pop_history(session_id, true).await?;
                return self.navigate_http(session_id, &prev).await;
            }
            BrowserAction::GoForward => {
                let next = self.pop_history(session_id, false).await?;
                return self.navigate_http(session_id, &next).await;
            }
            BrowserAction::WaitForElement {
                selector,
                timeout_ms,
            } => self.http_wait(session_id, selector, *timeout_ms).await,
            BrowserAction::Scroll { direction, amount } => {
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!(
                        "scroll recorded ({direction:?}, {:?}); 静态抓取无可视滚动",
                        amount.unwrap_or(300.0)
                    ),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Screenshot => Err(BrowserError::ActionFailed(
                "screenshot 需要 ChromeHeadless 后端（Http 无渲染）".to_string(),
            )),
            BrowserAction::ExecuteJs { .. } => Err(BrowserError::ActionFailed(
                "ExecuteJs 需要 CDP 通道（在途）；Http 后端不支持".to_string(),
            )),
            // Tab 簿记（通常由 dispatch 预拦截；保留分支以完备匹配）
            BrowserAction::NewTab { url } => self.new_tab(session_id, url.clone()).await,
            BrowserAction::SwitchTab { index } => {
                self.switch_tab(session_id, *index).await
            }
            BrowserAction::CloseTab => self.close_tab(session_id).await,
            // 状态与传输由引擎层预拦截，不应到达后端分发
            BrowserAction::SaveState { .. }
            | BrowserAction::LoadState { .. }
            | BrowserAction::Download { .. }
            | BrowserAction::Upload { .. } => Err(BrowserError::ActionFailed(
                "unreachable: handled by engine dispatch".to_string(),
            )),
        }
    }

    pub(crate) async fn http_submit_form(
        &self,
        session_id: &str,
        selector: Option<String>,
    ) -> Result<BrowserResult, BrowserError> {
        let (snap, fills, uploads) = {
            let sessions = self.sessions.read().await;
            let s = sessions
                .get(session_id)
                .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
            (
                s.snapshot.clone().ok_or_else(|| {
                    BrowserError::ActionFailed("no page loaded; Navigate first".to_string())
                })?,
                s.pending_fills.clone(),
                s.pending_uploads.clone(),
            )
        };
        let form = pick_form(&snap, selector.as_deref())?;
        let mut params: Vec<(String, String)> = Vec::new();
        // 文件字段走 multipart（其余跳过文本组装）
        let mut upload_parts: Vec<(String, Vec<String>)> = Vec::new();
        for f in &form.fields {
            if f.kind == "file" {
                if let Some(paths) = uploads.get(&f.name) {
                    if !paths.is_empty() {
                        upload_parts.push((f.name.clone(), paths.clone()));
                    }
                }
                continue;
            }
            if f.kind == "submit"
                || f.kind == "button"
                || f.kind == "image"
                || f.kind == "reset"
            {
                continue;
            }
            let v = fills.get(&f.name).cloned().unwrap_or(f.value.clone());
            // 未勾选的 checkbox/radio 值为空则跳过
            if (f.kind == "checkbox" || f.kind == "radio") && v.is_empty() {
                continue;
            }
            params.push((f.name.clone(), v));
        }
        let (jar, ua) = self.session_http_ctx(session_id).await?;
        let timeout = self.blocking_timeout();
        let action = form.action.clone();
        let method = form.method.clone();
        self.polite_wait(&action).await?;
        let auth = self.session_auth_header(session_id).await?;
        let submit_once = |auth: Option<(String, String)>| {
            let client = self.client.clone();
            let jar = jar.clone();
            let ua = ua.clone();
            let action = action.clone();
            let method = method.clone();
            let params = params.clone();
            let upload_parts = upload_parts.clone();
            async move {
                if upload_parts.is_empty() {
                    let page = http_submit(
                        &client, &action, &method, &params, &jar, &ua, auth, timeout,
                    )
                    .await?;
                    let h = url::Url::parse(&page.final_url)
                        .map(|u| u.host_str().unwrap_or("").to_string())
                        .unwrap_or_default();
                    return Ok::<_, BrowserError>((page, h));
                }
                let page = multipart_submit(
                    &client, &action, &params, &upload_parts, &jar, &ua, auth, timeout,
                )
                .await?;
                let h = url::Url::parse(&page.final_url)
                    .map(|u| u.host_str().unwrap_or("").to_string())
                    .unwrap_or_default();
                Ok::<_, BrowserError>((page, h))
            }
        };
        let (fetched, host) = match submit_once(auth).await {
            Err(BrowserError::AuthRejected(_)) => {
                let auth2 = self.session_auth_header(session_id).await?;
                match submit_once(auth2).await {
                    Err(BrowserError::AuthRejected(_)) => {
                        return Err(BrowserError::AuthExpired {
                            hint: self.auth_hint(session_id).await,
                        });
                    }
                    other => other?,
                }
            }
            other => other?,
        };
        self.apply_cookies(
            session_id,
            fetched
                .set_cookies
                .into_iter()
                .map(|c| format!("{host}\u{1f}{c}"))
                .collect(),
        )
        .await;
        let (title, text, links, forms, raw_html) =
            parse_page(&fetched.final_url, &fetched.body)?;
        let status = fetched.status;
        let snap = PageSnapshot {
            url: fetched.final_url.clone(),
            title: title.clone(),
            text,
            links,
            forms,
            raw_html,
            http_status: Some(status),
        };
        let out = render_snapshot_text(&snap);
        let verify = verify_for(&action, &snap);
        // 提交成功后清空待填值与待传文件
        {
            let mut sessions = self.sessions.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                s.pending_fills.clear();
                s.pending_uploads.clear();
            }
        }
        self.commit_snapshot(session_id, snap, true).await?;
        self.record_fetch(&fetched.final_url).await;
        Ok(BrowserResult::ok_verified(
            out,
            fetched.final_url,
            title,
            0,
            verify,
        ))
    }

    pub(crate) async fn http_wait(
        &self,
        session_id: &str,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<BrowserResult, BrowserError> {
        let deadline =
            std::time::Instant::now() + Duration::from_millis(timeout_ms.min(120_000));
        loop {
            {
                let sessions = self.sessions.read().await;
                let s = sessions
                    .get(session_id)
                    .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
                if let Some(snap) = &s.snapshot {
                    let doc = Html::parse_document(&snap.raw_html);
                    if Selector::parse(selector)
                        .ok()
                        .map(|sel| doc.select(&sel).next().is_some())
                        .unwrap_or(false)
                    {
                        let (url, title) = (s.current_url.clone(), s.title.clone());
                        return Ok(BrowserResult::ok(
                            format!("element '{selector}' present"),
                            url,
                            title,
                            0,
                        ));
                    }
                }
            }
            if std::time::Instant::now() >= deadline {
                let (url, title) = self.session_url_title(session_id).await?;
                return Ok(BrowserResult::fail(
                    format!("wait timed out: '{selector}'"),
                    url,
                    title,
                    timeout_ms,
                ));
            }
            // 重抓一次再判定
            let url = self.session_url(session_id).await?;
            if url != "about:blank" {
                let (jar, ua) = self.session_http_ctx(session_id).await?;
                let auth = self.session_auth_header(session_id).await.unwrap_or(None);
                if let Ok((snap, packed)) = self.http_fetch_snapshot(&url, &jar, &ua, auth).await {
                    self.apply_cookies(session_id, packed).await;
                    self.commit_snapshot(session_id, snap, false).await?;
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // -- Chrome 后端 ------------------------------------------------------------

    pub(crate) async fn dispatch_chrome(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        match action {
            BrowserAction::Navigate { url } => {
                self.polite_wait(url).await?;
                let budget = self.config.timeout_ms.min(20_000);
                let url_owned = url.clone();
                let joined = tokio::task::spawn_blocking(move || {
                    chrome_render(&url_owned, budget, None, 1280, 720)
                })
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("render task: {e}")))?;
                let html = joined?;
                let (title, text, links, forms, raw_html) = parse_page(url, &html)?;
                let snap = PageSnapshot {
                    url: url.clone(),
                    title: title.clone(),
                    text,
                    links,
                    forms,
                    raw_html,
                    http_status: None,
                };
                let out = render_snapshot_text(&snap);
                self.commit_snapshot(session_id, snap, true).await?;
                self.record_fetch(url).await;
                Ok(BrowserResult::ok(out, url.clone(), title, 0))
            }
            BrowserAction::Screenshot => {
                let (url, title) = self.session_url_title(session_id).await?;
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.png",
                    uuid::Uuid::new_v4()
                ));
                let target = if url == "about:blank" {
                    "about:blank".to_string()
                } else {
                    url.clone()
                };
                let budget = self.config.timeout_ms.min(20_000);
                let (w, h) = (self.config.viewport_width, self.config.viewport_height);
                let dest = path.clone();
                let joined = tokio::task::spawn_blocking(move || {
                    chrome_render(&target, budget, Some(&dest), w, h)
                })
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("screenshot task: {e}")))?;
                joined?;
                Ok(BrowserResult::ok(
                    format!("screenshot: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            // 渲染页的链接/表单沿用 HTTP 语义回放（Cookie 不互通，见模块头）
            BrowserAction::Click { .. } => {
                self.dispatch_http(session_id, action).await.map(|mut r| {
                    r.output = format!("[chrome-rendered] {}", r.output);
                    r
                })
            }
            BrowserAction::GetContent => {
                self.dispatch_http(session_id, action).await
            }
            BrowserAction::Type { selector, text } => {
                self.record_fill(session_id, selector.clone(), text.clone())
                    .await
            }
            BrowserAction::FillField { selector, value } => {
                self.record_fill(session_id, selector.clone(), value.clone())
                    .await
            }
            BrowserAction::SelectOption { .. }
            | BrowserAction::Check { .. }
            | BrowserAction::Uncheck { .. }
            | BrowserAction::Press { .. }
            | BrowserAction::GetText { .. } => {
                self.dispatch_http(session_id, action).await.map(|mut r| {
                    r.output = format!("[chrome-rendered] {}", r.output);
                    r
                })
            }
            BrowserAction::PrintPdf => {
                let (url, title) = self.session_url_title(session_id).await?;
                if url == "about:blank" {
                    return Err(BrowserError::ActionFailed(
                        "no page loaded; Navigate first".to_string(),
                    ));
                }
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.pdf",
                    uuid::Uuid::new_v4()
                ));
                let url_owned = url.clone();
                let dest = path.clone();
                let budget = self.config.timeout_ms.min(20_000);
                tokio::task::spawn_blocking(move || {
                    chrome_render_pdf(&url_owned, &dest, budget)
                })
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("pdf task: {e}")))?
                ?;
                Ok(BrowserResult::ok(
                    format!("pdf: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::SubmitForm { selector } => {
                self.http_submit_form(session_id, selector.clone()).await
            }
            BrowserAction::ExecuteJs { .. } => Err(BrowserError::ActionFailed(
                "ExecuteJs 需要 CDP 通道（在途）；dump-dom 管道不支持".to_string(),
            )),
            other => self.dispatch_http(session_id, other).await,
        }
    }

    // -- CDP 后端（真操控，需 stealth-net 特性） -------------------------------

    /// CDP 分发（无特性时诚实报错）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn dispatch_cdp(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        match action {
            BrowserAction::Navigate { url } => self.cdp_navigate(session_id, url).await,
            BrowserAction::GetContent => {
                let page = self.cdp_page(session_id).await?;
                let html = page
                    .content()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp content: {e}")))?;
                let url = page.url().await.ok().flatten().unwrap_or_default();
                let base = if url.is_empty() {
                    self.session_url(session_id).await?
                } else {
                    url
                };
                let (title, text, links, forms, raw_html) = parse_page(&base, &html)?;
                let snap = PageSnapshot {
                    url: base.clone(),
                    title: title.clone(),
                    text,
                    links,
                    forms,
                    raw_html,
            http_status: None,
                };
                let out = render_snapshot_text(&snap);
                self.commit_snapshot(session_id, snap, true).await?;
                self.record_fetch(&base).await;
                Ok(BrowserResult::ok(out, base, title, 0))
            }
            BrowserAction::Click { selector } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp click '{selector}': {e}"))
                })?;
                el.click()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp click: {e}")))?;
                tokio::time::sleep(Duration::from_millis(800)).await;
                return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
            }
            BrowserAction::Type { selector, text } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp type '{selector}': {e}"))
                })?;
                el.click()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp focus: {e}")))?;
                el.type_str(text.clone())
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp type: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("typed into '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::FillField { selector, value } => {
                return Box::pin(self
                    .dispatch_cdp(
                        session_id,
                        &BrowserAction::Type {
                            selector: selector.clone(),
                            text: value.clone(),
                        },
                    ))
                    .await;
            }
            BrowserAction::SubmitForm { selector } => {
                let (_, snap_opt) = self.session_snapshot(session_id).await?;
                let snap = snap_opt.ok_or_else(|| {
                    BrowserError::ActionFailed(
                        "no page loaded; Navigate first".to_string(),
                    )
                })?;
                // 用快照表单序号定位 document.forms[idx] 真提交
                let idx = match selector.as_deref() {
                    None => 0,
                    Some(s) if s.starts_with('#') => {
                        s.strip_prefix('#').unwrap_or("").parse().map_err(|_| {
                            BrowserError::ActionFailed(format!("bad form selector '{s}'"))
                        })?
                    }
                    Some(_) => 0,
                };
                if pick_form(&snap, Some(&format!("#{idx}"))).is_err() {
                    return Err(BrowserError::ActionFailed(format!("no form #{idx}")));
                }
                let page = self.cdp_page(session_id).await?;
                page.evaluate(format!(
                    "(()=>{{const f=document.forms[{idx}];if(!f)return 'no-form';f.submit();return 'ok';}})()"
                ))
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("cdp submit: {e}")))?;
                tokio::time::sleep(Duration::from_millis(800)).await;
                return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
            }
            BrowserAction::SelectOption { selector, values } => {
                let page = self.cdp_page(session_id).await?;
                let out = page
                    .evaluate(js_select_option(selector, values))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp select: {e}")))?;
                let text = out
                    .value()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .unwrap_or_else(|| "undefined".to_string());
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(text, url, title, 0))
            }
            BrowserAction::Check { selector } => {
                let page = self.cdp_page(session_id).await?;
                page.evaluate(js_set_checked(selector, true))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp check: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("checked '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Uncheck { selector } => {
                let page = self.cdp_page(session_id).await?;
                page.evaluate(js_set_checked(selector, false))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp uncheck: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("unchecked '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Press { selector, key } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp press '{selector}': {e}"))
                })?;
                el.press_key(key.clone())
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp press: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("pressed '{key}' on '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::GetText { selector } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp text '{selector}': {e}"))
                })?;
                let text = el
                    .inner_text()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp text: {e}")))?
                    .unwrap_or_default();
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(text, url, title, 0))
            }
            BrowserAction::PrintPdf => {
                let page = self.cdp_page(session_id).await?;
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.pdf",
                    uuid::Uuid::new_v4()
                ));
                page.save_pdf(Default::default(), &path)
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp pdf: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("pdf: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::ExecuteJs { script } => {
                let page = self.cdp_page(session_id).await?;
                let result = page
                    .evaluate(script.clone())
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp eval: {e}")))?;
                let out = result
                    .value()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .unwrap_or_else(|| "undefined".to_string());
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(out, url, title, 0))
            }
            BrowserAction::Screenshot => {
                let page = self.cdp_page(session_id).await?;
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.png",
                    uuid::Uuid::new_v4()
                ));
                page.save_screenshot(
                    chromiumoxide::page::ScreenshotParams::builder()
                        .full_page(true)
                        .build(),
                    &path,
                )
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("cdp shot: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("screenshot: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Scroll { direction, amount } => {
                let (dx, dy) = match direction {
                    ScrollDirection::Up => (0, -(amount.unwrap_or(300.0) as i64)),
                    ScrollDirection::Down => (0, amount.unwrap_or(300.0) as i64),
                    ScrollDirection::Left => (-(amount.unwrap_or(300.0) as i64), 0),
                    ScrollDirection::Right => (amount.unwrap_or(300.0) as i64, 0),
                };
                let page = self.cdp_page(session_id).await?;
                page.evaluate(format!("window.scrollBy({dx},{dy})"))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp scroll: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("scrolled {direction:?}"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::WaitForElement {
                selector,
                timeout_ms,
            } => {
                let page = self.cdp_page(session_id).await?;
                let deadline = std::time::Instant::now()
                    + Duration::from_millis((*timeout_ms).min(120_000));
                loop {
                    if page
                        .find_element(selector.clone())
                        .await
                        .is_ok()
                    {
                        let (url, title) = self.session_url_title(session_id).await?;
                        return Ok(BrowserResult::ok(
                            format!("element '{selector}' present"),
                            url,
                            title,
                            0,
                        ));
                    }
                    if std::time::Instant::now() >= deadline {
                        let (url, title) = self.session_url_title(session_id).await?;
                        return Ok(BrowserResult::fail(
                            format!("wait timed out: '{selector}'"),
                            url,
                            title,
                            *timeout_ms,
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
            BrowserAction::Reload => {
                let page = self.cdp_page(session_id).await?;
                page.reload()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp reload: {e}")))?;
                return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
            }
            BrowserAction::GoBack => {
                return self.cdp_history(session_id, "back").await;
            }
            BrowserAction::GoForward => {
                return self.cdp_history(session_id, "forward").await;
            }
            BrowserAction::NewTab { .. }
            | BrowserAction::SwitchTab { .. }
            | BrowserAction::CloseTab => {
                Err(BrowserError::ActionFailed(
                    "tab ops 由引擎层预拦截，不应到达 CDP 分发".to_string(),
                ))
            }
            // 状态与传输由引擎层预拦截，不应到达后端分发
            BrowserAction::SaveState { .. }
            | BrowserAction::LoadState { .. }
            | BrowserAction::Download { .. }
            | BrowserAction::Upload { .. } => Err(BrowserError::ActionFailed(
                "unreachable: handled by engine dispatch".to_string(),
            )),
        }
    }

    #[cfg(not(feature = "stealth-net"))]
    pub(crate) async fn dispatch_cdp(
        &self,
        _session_id: &str,
        _action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        Err(BrowserError::BackendUnavailable(
            "Cdp 后端需要 stealth-net 特性".to_string(),
        ))
    }

    /// CDP 导航（含 Cookie 回灌到自研 jar）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_navigate(
        &self,
        session_id: &str,
        url: &str,
    ) -> Result<BrowserResult, BrowserError> {
        self.polite_wait(url).await?;
        let page = self.cdp_page(session_id).await?;
        page.goto(url.to_string())
            .await
            .map_err(|e| BrowserError::NavigationFailed(format!("cdp goto {url}: {e}")))?;
        let html = page
            .content()
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp content: {e}")))?;
        let current = page.url().await.ok().flatten().unwrap_or_default();
        let base = if current.is_empty() {
            url.to_string()
        } else {
            current
        };
        // Cookie 双向同步：CDP → 自研 jar
        if let Ok(cookies) = page.get_cookies().await {
            let packed: Vec<String> = cookies
                .iter()
                .map(|c| {
                    format!(
                        "{}\u{1f}{}={}",
                        c.domain.trim_start_matches('.'),
                        c.name,
                        c.value
                    )
                })
                .collect();
            self.apply_cookies(session_id, packed).await;
        }
        let (title, text, links, forms, raw_html) = parse_page(&base, &html)?;
        let snap = PageSnapshot {
            url: base.clone(),
            title: title.clone(),
            text,
            links,
            forms,
            raw_html,
            http_status: None,
        };
        let out = render_snapshot_text(&snap);
        self.commit_snapshot(session_id, snap, true).await?;
        self.record_fetch(&base).await;
        Ok(BrowserResult::ok(out, base, title, 0))
    }

    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_history(
        &self,
        session_id: &str,
        dir: &str,
    ) -> Result<BrowserResult, BrowserError> {
        let page = self.cdp_page(session_id).await?;
        page.evaluate(format!("history.{dir}()"))
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp history: {e}")))?;
        tokio::time::sleep(Duration::from_millis(800)).await;
        return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
    }

    /// CDP 浏览器单例（懒启动 + handler 泵 + 会话 UA 注入 stealth）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_browser(&self) -> Result<Arc<Browser>, BrowserError> {
        {
            let guard = self.cdp_browser.lock().await;
            if let Some(browser) = guard.as_ref() {
                return Ok(browser.clone());
            }
        }
        // B 方案：附着用户活体 Chrome（对方以 --remote-debugging-port 启动，登录态完整保持）。
        // NT_BROWSE_CDP_URL=http://127.0.0.1:9333（http 形自动取 /json/version 换 ws）。
        // profile 副本带不过钥匙串登录态，launch 路线只做未登录页；要登录态必须走本分支。
        if let Some(endpoint) = std::env::var("NT_BROWSE_CDP_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
        {
            let endpoint = endpoint.trim().to_string();
            let (browser, mut handler) = Browser::connect(endpoint.clone()).await.map_err(|e| {
                BrowserError::BackendUnavailable(format!("cdp connect {endpoint}: {e}"))
            })?;
            tokio::spawn(async move {
                while let Some(event) = handler.next().await {
                    if let Err(e) = event {
                        log::warn!("[nt_browser] cdp handler: {e:?}");
                    }
                }
            });
            let browser = Arc::new(browser);
            *self.cdp_browser.lock().await = Some(browser.clone());
            log::info!("[nt_browser] cdp attached to {endpoint}");
            return Ok(browser);
        }
        let mut builder = CdpConfig::builder()
            .chrome_executable(chrome_path())
            .no_sandbox()
            .window_size(self.config.viewport_width, self.config.viewport_height)
            .request_timeout(self.op_timeout())
            .launch_timeout(Duration::from_secs(60))
            .disable_default_args();
        for arg in [
            "--disable-dev-shm-usage",
            "--disable-blink-features=AutomationControlled",
            "--no-first-run",
            "--disable-background-networking",
            "--disable-sync",
            "--mute-audio",
            "--disable-component-update",
            "--disable-client-side-phishing-detection",
            // 2026-09-22：mock 钥匙串，CDP 浏览器永不弹系统密码框
            "--use-mock-keychain",
        ] {
            builder = builder.arg(arg);
        }
        if let Some(proxy) = self.config.proxy.as_deref() {
            let proxy = proxy.trim();
            if !proxy.is_empty() {
                builder = builder.arg(format!("--proxy-server={proxy}"));
            }
        }
        // profile 复用：继承已登录会话（须先退出占用的 Chrome，见字段注释）
        if let Some(profile) = self.config.profile_dir.as_deref() {
            let profile = profile.trim();
            if !profile.is_empty() {
                builder = builder.user_data_dir(profile);
            }
        }
        let cfg = builder
            .build()
            .map_err(|e| BrowserError::BackendUnavailable(format!("cdp config: {e}")))?;
        let (browser, mut handler) = Browser::launch(cfg)
            .await
            .map_err(|e| BrowserError::BackendUnavailable(format!("cdp launch: {e}")))?;
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    log::warn!("[nt_browser] cdp handler: {e:?}");
                }
            }
        });
        let browser = Arc::new(browser);
        *self.cdp_browser.lock().await = Some(browser.clone());
        Ok(browser)
    }

    /// 会话 CDP 页面（懒创建 + stealth + 会话 UA）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_page(&self, session_id: &str) -> Result<Page, BrowserError> {
        {
            let sessions = self.sessions.read().await;
            sessions
                .get(session_id)
                .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        }
        {
            let pages = self.cdp_pages.read().await;
            if let Some(page) = pages.get(session_id) {
                return Ok(page.clone());
            }
        }
        let browser = self.cdp_browser().await?;
        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp new_page: {e}")))?;
        let ua = self
            .sessions
            .read()
            .await
            .get(session_id)
            .map(|s| s.user_agent.clone())
            .unwrap_or_default();
        page.enable_stealth_mode_with_agent(&ua)
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp stealth: {e}")))?;
        self.cdp_pages
            .write()
            .await
            .insert(session_id.to_string(), page.clone());
        Ok(page)
    }

    // -- 会话小件 ---------------------------------------------------------------

    /// 会话绑定认证（立即加载一次；文件源之后每次请求前热加载）
    pub async fn set_session_auth(
        &self,
        session_id: &str,
        config: AuthConfig,
    ) -> Result<(), BrowserError> {
        let state = AuthState::load(&config)?;
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        session.auth = Some(state);
        Ok(())
    }

    /// 解绑认证
    pub async fn clear_session_auth(
        &self,
        session_id: &str,
    ) -> Result<(), BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        session.auth = None;
        Ok(())
    }

    /// 按站点名绑定认证（统一纳管：读 auth.toml 对应条目）
    pub async fn set_session_auth_by_site(
        &self,
        session_id: &str,
        site: &str,
    ) -> Result<(), BrowserError> {
        use super::super::nt_io_auth_store::AuthStore;
        let store = AuthStore::load();
        let entry = store.get(site).ok_or_else(|| {
            BrowserError::ActionFailed(format!(
                "auth.toml 无站点 [{site}]（{}），可用：{}",
                store.path().display(),
                store.site_names().join(",")
            ))
        })?;
        let source = match entry.token_file.as_deref() {
            Some(p) => AuthSource::File(AuthStore::expand_tilde(p)),
            None => AuthSource::Literal(entry.token_literal.clone().unwrap_or_default()),
        };
        let config = AuthConfig {
            source,
            header_name: entry
                .header_name
                .clone()
                .unwrap_or_else(|| "Authorization".to_string()),
            scheme: entry.scheme.clone().unwrap_or_else(|| "Bearer".to_string()),
            expires_at_ms: entry.expires_at_ms,
            reseed_hint: entry.reseed_hint.clone().unwrap_or_default(),
        };
        self.set_session_auth(session_id, config).await
    }

    /// 当前认证头（header 名, header 值）；文件源先热加载
    pub(crate) async fn session_auth_header(
        &self,
        session_id: &str,
    ) -> Result<Option<(String, String)>, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        match session.auth.as_mut() {
            None => Ok(None),
            Some(state) => {
                // 热加载失败（如文件被删）→ 视为过期，给出精确指引而非裸错
                if state.refresh().is_err() {
                    return Err(BrowserError::AuthExpired {
                        hint: state.config.hint(),
                    });
                }
                Ok(Some((
                    state.config.header_name.clone(),
                    state.header_value(),
                )))
            }
        }
    }

    /// 过期预检：已知过期点已过则直接拒收（省一次无效请求）
    pub(crate) async fn check_auth_fresh(
        &self,
        session_id: &str,
    ) -> Result<(), BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        match session.auth.as_ref() {
            Some(state) if state.expired() => Err(BrowserError::AuthExpired {
                hint: state.config.hint(),
            }),
            _ => Ok(()),
        }
    }

    /// 会话 HTTP 上下文：(jar 快照, 锁定的 UA)
    pub(crate) async fn session_http_ctx(
        &self,
        session_id: &str,
    ) -> Result<(CookieJar, String), BrowserError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|s| (s.jar.clone(), s.user_agent.clone()))
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))
    }

    pub(crate) async fn session_url(&self, session_id: &str) -> Result<String, BrowserError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|s| s.current_url.clone())
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))
    }

    pub(crate) async fn session_url_title(
        &self,
        session_id: &str,
    ) -> Result<(String, Option<String>), BrowserError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|s| (s.current_url.clone(), s.title.clone()))
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))
    }

    pub(crate) async fn session_snapshot(
        &self,
        session_id: &str,
    ) -> Result<(String, Option<PageSnapshot>), BrowserError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|s| (s.current_url.clone(), s.snapshot.clone()))
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))
    }

    pub(crate) async fn session_raw_html(&self, session_id: &str) -> Result<String, BrowserError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .and_then(|s| s.snapshot.as_ref().map(|snap| snap.raw_html.clone()))
            .ok_or_else(|| {
                BrowserError::ActionFailed("no page loaded; Navigate first".to_string())
            })
    }

    pub(crate) async fn record_fill(
        &self,
        session_id: &str,
        selector: String,
        value: String,
    ) -> Result<BrowserResult, BrowserError> {
        let (key, _) = self
            .resolve_fill_key(session_id, &selector)
            .await?;
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        session.pending_fills.insert(key.clone(), value);
        let (url, title) = (session.current_url.clone(), session.title.clone());
        Ok(BrowserResult::ok(
            format!("fill recorded: {key}"),
            url,
            title,
            0,
        ))
    }

    /// 选择器 → (input name, 表单默认值)：快照中解析不到则回退选择器原文
    pub(crate) async fn resolve_fill_key(
        &self,
        session_id: &str,
        selector: &str,
    ) -> Result<(String, String), BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        let mut key = selector.to_string();
        let mut default = String::new();
        if let Some(snap) = &session.snapshot {
            let doc = Html::parse_document(&snap.raw_html);
            if let Ok(sel) = Selector::parse(selector) {
                if let Some(el) = doc.select(&sel).next() {
                    if let Some(name) = el.value().attr("name") {
                        key = name.to_string();
                        default = el
                            .value()
                            .attr("value")
                            .unwrap_or("")
                            .to_string();
                    }
                }
            }
            if default.is_empty() {
                for form in &snap.forms {
                    if let Some(f) = form.fields.iter().find(|f| f.name == key) {
                        if !f.value.is_empty() {
                            default = f.value.clone();
                        }
                        break;
                    }
                }
            }
        }
        Ok((key, default))
    }

    /// 删除待填值（Uncheck 用）
    pub(crate) async fn drop_fill(
        &self,
        session_id: &str,
        selector: &str,
    ) -> Result<BrowserResult, BrowserError> {
        let (key, _) = self.resolve_fill_key(session_id, selector).await?;
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        session.pending_fills.remove(&key);
        let (url, title) = (session.current_url.clone(), session.title.clone());
        Ok(BrowserResult::ok(
            format!("fill removed: {key}"),
            url,
            title,
            0,
        ))
    }

    pub(crate) async fn pop_history(
        &self,
        session_id: &str,
        back: bool,
    ) -> Result<String, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        let stack = if back {
            &mut session.history_back
        } else {
            &mut session.history_fwd
        };
        match stack.pop() {
            Some(url) => {
                let other = if back {
                    &mut session.history_fwd
                } else {
                    &mut session.history_back
                };
                other.push(session.current_url.clone());
                Ok(url)
            }
            None => Err(BrowserError::ActionFailed(
                "history empty".to_string(),
            )),
        }
    }

    pub(crate) async fn new_tab(
        &self,
        session_id: &str,
        url: Option<String>,
    ) -> Result<BrowserResult, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        let index = session.tabs.len();
        for t in session.tabs.iter_mut() {
            t.active = false;
        }
        session.tabs.push(Tab {
            index,
            url: url.clone().unwrap_or_else(|| "about:blank".to_string()),
            title: None,
            active: true,
        });
        session.current_tab = index;
        let (curl, title) = (session.current_url.clone(), session.title.clone());
        Ok(BrowserResult::ok(
            format!("new tab {index}"),
            curl,
            title,
            0,
        ))
    }

    pub(crate) async fn switch_tab(
        &self,
        session_id: &str,
        index: usize,
    ) -> Result<BrowserResult, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        if index >= session.tabs.len() {
            return Err(BrowserError::ActionFailed(format!(
                "no tab {index}"
            )));
        }
        for t in session.tabs.iter_mut() {
            t.active = t.index == index;
        }
        session.current_tab = index;
        let (curl, title) = (session.current_url.clone(), session.title.clone());
        Ok(BrowserResult::ok(
            format!("switched to tab {index}"),
            curl,
            title,
            0,
        ))
    }

    pub(crate) async fn close_tab(&self, session_id: &str) -> Result<BrowserResult, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        if session.tabs.len() <= 1 {
            return Err(BrowserError::ActionFailed(
                "cannot close the last tab".to_string(),
            ));
        }
        session.tabs.remove(session.current_tab);
        for (i, t) in session.tabs.iter_mut().enumerate() {
            t.index = i;
        }
        session.current_tab = 0;
        if let Some(tab) = session.tabs.get_mut(0) {
            tab.active = true;
        }
        let (curl, title) = (session.current_url.clone(), session.title.clone());
        Ok(BrowserResult::ok("tab closed".to_string(), curl, title, 0))
    }

    // -- 状态持久化与传输（引擎级，与后端无关） ----------------------------------

    /// Save session to JSON file（内存 token 默认脱敏置空）
    pub(crate) async fn save_state(
        &self,
        session_id: &str,
        path: String,
    ) -> Result<BrowserResult, BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        let mut stored = session.clone();
        let mut note = String::new();
        if let Some(auth) = stored.auth.as_mut() {
            if matches!(auth.config.source, AuthSource::Literal(_))
                && !auth.cached_token.is_empty()
            {
                // 内存 token 不落盘：缓存与来源一并清空（加载后重新 set）
                auth.cached_token.clear();
                auth.config.source = AuthSource::Literal(String::new());
                note = "（内存 token 已脱敏，加载后需重新 set_session_auth）".to_string();
            }
        }
        let saved = SavedSession {
            version: STATE_FORMAT_VERSION,
            saved_at_ms: now_ms(),
            session: stored,
        };
        let json = serde_json::to_string_pretty(&saved).map_err(|e| {
            BrowserError::ActionFailed(format!("serialize state: {e}"))
        })?;
        tokio::fs::write(&path, json).await.map_err(|e| {
            BrowserError::ActionFailed(format!("write state {path}: {e}"))
        })?;
        let (curl, title) = (session.current_url.clone(), session.title.clone());
        Ok(BrowserResult::ok(
            format!("state saved: {path}{note}"),
            curl,
            title,
            0,
        ))
    }

    /// Load session from JSON file（新 ID，老会话不覆盖；版本不一致直接报错）
    pub(crate) async fn load_state(&self, path: String) -> Result<BrowserResult, BrowserError> {
        let raw = tokio::fs::read_to_string(&path).await.map_err(|e| {
            BrowserError::ActionFailed(format!("read state {path}: {e}"))
        })?;
        let saved: SavedSession = serde_json::from_str(&raw).map_err(|e| {
            BrowserError::ActionFailed(format!("parse state {path}: {e}"))
        })?;
        if saved.version != STATE_FORMAT_VERSION {
            return Err(BrowserError::ActionFailed(format!(
                "state version {} != engine {}",
                saved.version, STATE_FORMAT_VERSION
            )));
        }
        let mut session = saved.session;
        let new_id = format!("browser-{}", uuid::Uuid::new_v4());
        session.id.clone_from(&new_id);
        let (curl, title) = (session.current_url.clone(), session.title.clone());
        let mut note = String::new();
        if let Some(auth) = session.auth.as_ref() {
            match &auth.config.source {
                AuthSource::File(_) => {
                    note = "（文件 token 将在下次请求时热加载）".to_string();
                }
                AuthSource::Literal(_) if auth.cached_token.is_empty() => {
                    note =
                        "（内存 token 未持久化，请重新 set_session_auth）".to_string();
                }
                _ => {}
            }
        }
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(new_id.clone(), session);
        }
        Ok(BrowserResult::ok(
            format!("state loaded: {path} -> {new_id}{note}"),
            curl,
            title,
            0,
        ))
    }

    /// Download URL to file（Http 客户端直下，50MB 上限，会话认证自动携带）
    pub(crate) async fn download(
        &self,
        session_id: &str,
        url: String,
        path: String,
    ) -> Result<BrowserResult, BrowserError> {
        {
            let sessions = self.sessions.read().await;
            sessions
                .get(session_id)
                .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        }
        if let Some(parent) = std::path::Path::new(&path).parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(BrowserError::ActionFailed(format!(
                    "parent dir missing: {}",
                    parent.display()
                )));
            }
        }
        self.polite_wait(&url).await?;
        let (jar, ua) = self.session_http_ctx(session_id).await?;
        let auth = self.session_auth_header(session_id).await?;
        let timeout = self.blocking_timeout();
        let get_once = |auth: Option<(String, String)>| {
            let client = self.client.clone();
            let jar = jar.clone();
            let ua = ua.clone();
            let url = url.clone();
            async move {
                let mut req = client
                    .get(&url)
                    .timeout(timeout)
                    .header(reqwest::header::USER_AGENT, &ua);
                if let Some((name, value)) = auth {
                    req = req.header(name, value);
                }
                if let Ok(parsed) = url::Url::parse(&url) {
                    if let Some(cookie) = jar
                        .header_for(parsed.scheme(), parsed.host_str().unwrap_or(""))
                    {
                        req = req.header(reqwest::header::COOKIE, cookie);
                    }
                }
                let resp = req.send().await.map_err(|e| {
                    BrowserError::ActionFailed(format!("download {url}: {e}"))
                })?;
                let status = resp.status();
                if status == reqwest::StatusCode::UNAUTHORIZED
                    || status == reqwest::StatusCode::FORBIDDEN
                {
                    return Err(BrowserError::AuthRejected(status.as_u16()));
                }
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    let secs = parse_retry_after_secs(resp.headers());
                    return Err(BrowserError::RateLimited(secs));
                }
                if !status.is_success() {
                    return Err(BrowserError::ActionFailed(format!(
                        "download {url}: http {status}"
                    )));
                }
                if let Some(len) = resp.content_length() {
                    if len > MAX_DOWNLOAD_BYTES as u64 {
                        return Err(BrowserError::ActionFailed(format!(
                            "download too large: {len}B > 50MB"
                        )));
                    }
                }
                let ctype = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                let bytes = resp.bytes().await.map_err(|e| {
                    BrowserError::ActionFailed(format!("download body: {e}"))
                })?;
                if bytes.len() > MAX_DOWNLOAD_BYTES {
                    return Err(BrowserError::ActionFailed(format!(
                        "download too large: {}B > 50MB",
                        bytes.len()
                    )));
                }
                Ok::<_, BrowserError>((bytes, ctype))
            }
        };
        let (bytes, ctype) = match get_once(auth).await {
            Err(BrowserError::AuthRejected(_)) => {
                let auth2 = self.session_auth_header(session_id).await?;
                match get_once(auth2).await {
                    Err(BrowserError::AuthRejected(_)) => {
                        return Err(BrowserError::AuthExpired {
                            hint: self.auth_hint(session_id).await,
                        });
                    }
                    other => other?,
                }
            }
            other => other?,
        };
        tokio::fs::write(&path, &bytes).await.map_err(|e| {
            BrowserError::ActionFailed(format!("write download {path}: {e}"))
        })?;
        self.record_fetch(&url).await;
        let (curl, title) = self.session_url_title(session_id).await?;
        Ok(BrowserResult::ok(
            format!(
                "downloaded {}B ({}) -> {path}",
                bytes.len(),
                ctype.unwrap_or_else(|| "unknown".to_string())
            ),
            curl,
            title,
            0,
        ))
    }

    /// Record file(s) for upload input（即时校验存在性；提交时 multipart）
    pub(crate) async fn record_uploads(
        &self,
        session_id: &str,
        selector: String,
        files: Vec<String>,
    ) -> Result<BrowserResult, BrowserError> {
        if files.is_empty() {
            return Err(BrowserError::ActionFailed(
                "upload 需要至少一个文件".to_string(),
            ));
        }
        for f in &files {
            let meta = tokio::fs::metadata(f).await.map_err(|_| {
                BrowserError::ActionFailed(format!("upload 文件不存在: {f}"))
            })?;
            if !meta.is_file() {
                return Err(BrowserError::ActionFailed(format!(
                    "upload 不是文件: {f}"
                )));
            }
        }
        let (key, _) = self.resolve_fill_key(session_id, &selector).await?;
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        session.pending_uploads.insert(key.clone(), files.clone());
        let (url, title) = (session.current_url.clone(), session.title.clone());
        Ok(BrowserResult::ok(
            format!("upload recorded: {key} <- {} file(s)", files.len()),
            url,
            title,
            0,
        ))
    }

    /// Close a session（同时丢弃其 CDP 页面）
    pub async fn close_session(&self, session_id: &str) -> Result<(), BrowserError> {
        #[cfg(feature = "stealth-net")]
        {
            self.cdp_pages.write().await.remove(session_id);
        }
        let mut sessions = self.sessions.write().await;
        if sessions.remove(session_id).is_some() {
            Ok(())
        } else {
            Err(BrowserError::SessionNotFound(session_id.to_string()))
        }
    }

    /// 关闭 CDP 浏览器单例（会话级页面一并丢弃）
    #[cfg(feature = "stealth-net")]
    pub async fn shutdown_cdp(&self) {
        self.cdp_pages.write().await.clear();
        *self.cdp_browser.lock().await = None;
    }

    /// Get statistics
    pub async fn stats(&self) -> BrowserStats {
        let sessions = self.sessions.read().await;
        let history = self.history.read().await;

        let total = history.len();
        let successful = history.iter().filter(|(_, _, r)| r.success).count();
        let failed = total - successful;

        let avg_duration = if total > 0 {
            history.iter().map(|(_, _, r)| r.duration_ms as f64).sum::<f64>() / total as f64
        } else {
            0.0
        };

        BrowserStats {
            active_sessions: sessions.len(),
            total_actions: total,
            successful_actions: successful,
            failed_actions: failed,
            avg_action_duration_ms: avg_duration,
        }
    }
}
