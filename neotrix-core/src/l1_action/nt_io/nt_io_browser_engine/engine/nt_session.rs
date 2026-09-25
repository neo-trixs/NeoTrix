//! nt_session — 会话管理：构造 / 创建 / 分发 / 认证 / 表单暂存 / Tab 簿记 / 统计.
//! 从 `nt_io_browser_engine/engine.rs` 纯搬移, 行为零变更.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use scraper::{Html, Selector};

use super::BrowserEngine;
use super::nt_politeness::Politeness;
use super::super::{MAX_HISTORY, MAX_NAV_STACK};
use super::super::cookies::{AuthConfig, AuthSource, AuthState, CookieJar, now_ms};
use super::super::error::BrowserError;
use super::super::fetch::{http_client_for, pick_ua};
use super::super::policy::{AuditEvent, action_kind};
use super::super::session::{BrowserConfig, BrowserSession, BrowserStats, HistoryRetention, Tab};
use super::super::types::{BackendKind, BrowserAction, BrowserResult, PageSnapshot};

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
        use super::super::super::nt_io_auth_store::AuthStore;
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
