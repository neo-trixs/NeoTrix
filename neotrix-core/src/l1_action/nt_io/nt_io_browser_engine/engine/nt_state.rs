//! nt_state — 持久化/落盘：会话存取 + 下载 + 上传暂存.
//! 从 `nt_io_browser_engine/engine.rs` 纯搬移, 行为零变更.

use super::BrowserEngine;
use super::super::cookies::{AuthSource, now_ms};
use super::super::error::BrowserError;
use super::super::policy::parse_retry_after_secs;
use super::super::session::BrowserSession;
use super::super::types::BrowserResult;

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

impl BrowserEngine {
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
}
