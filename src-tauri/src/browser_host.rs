use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};
use thiserror::Error;

/// Errors from browser operations.
#[derive(Debug, Clone, Error)]
pub enum BrowserError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("browser window not open")]
    WindowNotFound,
    #[error("failed to create browser window: {0}")]
    WindowCreation(String),
    #[error("js eval error: {0}")]
    JsEval(String),
    #[error("navigation error: {0}")]
    Navigation(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("event emit error: {0}")]
    Emit(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub url: String,
    pub title: String,
    pub is_open: bool,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            url: "about:blank".into(),
            title: "Browser".into(),
            is_open: false,
        }
    }
}

/// 提取的页面内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: String,
    pub html: String,
    pub text: String,
}

impl PageContent {
    pub fn summary(&self, max_chars: usize) -> String {
        let text = if self.text.len() > max_chars {
            self.text.chars().take(max_chars).collect::<String>() + "..."
        } else {
            self.text.clone()
        };
        format!("# {}\n\n{}\n\n[source: {}]", self.title, text, self.url)
    }
}

/// 浏览器窗口管理器
pub struct BrowserHost;

impl BrowserHost {
    pub fn open_or_navigate(app: &AppHandle, url: &str) -> Result<BrowserState, BrowserError> {
        let parsed = url
            .parse()
            .map_err(|e| BrowserError::InvalidUrl(format!("{e}")))?;
        let window_id = "neotrix-browser";

        if let Some(window) = app.get_webview_window(window_id) {
            let _ = window.navigate(parsed);
            let _ = window.show();
            let _ = window.set_focus();
            return Ok(BrowserState {
                url: url.to_string(),
                title: "Loading...".into(),
                is_open: true,
            });
        }

        let window = WebviewWindowBuilder::new(app, window_id, tauri::WebviewUrl::External(parsed))
            .title("NeoTrix Browser")
            .inner_size(1024.0, 768.0)
            .min_inner_size(400.0, 300.0)
            .resizable(true)
            .fullscreen(false)
            .build()
            .map_err(|e| BrowserError::WindowCreation(format!("failed to create browser window: {}", e)))?;

        let close_handle = app.clone();
        // 窗口关闭由 Tauri 自动管理, 不需要轮询
        // 浏览器窗口关闭时, 前端通过 emit("browser_closed") 通知状态变更
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Err(e) = close_handle.emit("browser_closed", ()) {
                    tracing::warn!("Failed to emit browser_closed: {e}");
                }
            }
        });

        Ok(BrowserState {
            url: url.to_string(),
            title: "Loading...".into(),
            is_open: true,
        })
    }

    pub fn execute_js(app: &AppHandle, script: &str) -> Result<(), BrowserError> {
        let window = app
            .get_webview_window("neotrix-browser")
            .ok_or(BrowserError::WindowNotFound)?;
        window
            .eval(script)
            .map_err(|e| BrowserError::JsEval(format!("{e}")))
    }

    pub fn go_back(app: &AppHandle) -> Result<(), BrowserError> {
        Self::execute_js(app, "window.history.back()")
    }

    pub fn go_forward(app: &AppHandle) -> Result<(), BrowserError> {
        Self::execute_js(app, "window.history.forward()")
    }

    pub fn reload(app: &AppHandle) -> Result<(), BrowserError> {
        Self::execute_js(app, "location.reload()")
    }

    pub fn close(app: &AppHandle) -> Result<(), BrowserError> {
        if let Some(window) = app.get_webview_window("neotrix-browser") {
            window
                .close()
                .map_err(|e| BrowserError::Navigation(format!("{e}")))
        } else {
            Err(BrowserError::WindowNotFound)
        }
    }

    /// 服务端获取页面内容 (通过 HTTP, 绕过 eval 无法返回值的问题)
    pub fn fetch_page_content(url: &str) -> Result<PageContent, BrowserError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("NeoTrix/1.0")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| BrowserError::Network(format!("http client error: {e}")))?;

        let resp = client
            .get(url)
            .send()
            .map_err(|e| BrowserError::Network(format!("fetch error: {e}")))?;
        let final_url = resp.url().to_string();
        let html = resp
            .text()
            .map_err(|e| BrowserError::Network(format!("read error: {e}")))?;

        let title = extract_title(&html);
        let text = strip_html(&html);

        Ok(PageContent {
            url: final_url,
            title,
            html,
            text,
        })
    }
}

/// 从 HTML 提取 <title>
fn extract_title(html: &str) -> String {
    for line in html.lines() {
        if let Some(start) = line.find("<title") {
            if let Some(rel_start) = line[start..].find('>') {
                let content_start = start + rel_start;
                let after_tag = &line[content_start + 1..];
                if let Some(end) = after_find("</title>", after_tag) {
                    return after_tag[..end].trim().to_string();
                }
            }
        }
    }
    String::new()
}

fn after_find(pat: &str, s: &str) -> Option<usize> {
    s.find(pat)
}

/// 剥离 HTML 标签, 返回纯文本
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut skip_chars = 0usize;
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if skip_chars > 0 {
            skip_chars -= 1;
            i += 1;
            continue;
        }
        let c = chars[i];
        if !in_tag && c == '<' {
            let rest: String = chars[i..].iter().collect();
            // 开标签检测 (仅在普通内容中)
            if !in_script && !in_style {
                if rest.starts_with("<script") || rest.starts_with("<SCRIPT") {
                    in_script = true;
                    in_tag = true;
                    i += 1;
                    continue;
                }
                if rest.starts_with("<style") || rest.starts_with("<STYLE") {
                    in_style = true;
                    in_tag = true;
                    i += 1;
                    continue;
                }
                in_tag = true;
                i += 1;
                continue;
            }
            // script/style 内容中的闭合标签检测
            if rest.starts_with("</script") || rest.starts_with("</SCRIPT") {
                in_script = false;
                in_tag = true;
                i += 1;
                continue;
            }
            if rest.starts_with("</style") || rest.starts_with("</STYLE") {
                in_style = false;
                in_tag = true;
                i += 1;
                continue;
            }
            // script/style 内容中的其他标签，直接跳过
            in_tag = true;
            i += 1;
            continue;
        }
        if in_tag && c == '>' {
            in_tag = false;
            // 添加空格代替标签
            if !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
            i += 1;
            continue;
        }
        if !in_tag && !in_script && !in_style {
            if c.is_whitespace() {
                if !out.ends_with(' ') && !out.is_empty() {
                    out.push(' ');
                }
            } else {
                // 解码常见 HTML 实体
                match c {
                    '&' => {
                        let rest: String = chars[i..].iter().collect();
                        if rest.starts_with("&amp;") {
                            out.push('&');
                            skip_chars = 4;
                        } else if rest.starts_with("&lt;") {
                            out.push('<');
                            skip_chars = 3;
                        } else if rest.starts_with("&gt;") {
                            out.push('>');
                            skip_chars = 3;
                        } else if rest.starts_with("&quot;") {
                            out.push('"');
                            skip_chars = 5;
                        } else if rest.starts_with("&#39;") || rest.starts_with("&#x27;") {
                            out.push('\'');
                            skip_chars = rest.starts_with("&#39;") as usize * 4
                                + rest.starts_with("&#x27;") as usize * 5;
                        } else if rest.starts_with("&nbsp;") {
                            out.push(' ');
                            skip_chars = 5;
                        } else {
                            out.push('&');
                        }
                    }
                    _ => out.push(c),
                }
            }
        }
        i += 1;
    }
    out.trim().to_string()
}

pub struct BrowserHostState(pub Mutex<BrowserHost>);

// ══════════════════════════════════════════════════════════════
// AuthBridge — 桌面桥鉴权事件（P0-3，对标 LingeeBridge）
//
// 401 处理链：后端 emit("token-expired") → 前端静默续签 →
// 续签失败则后端 emit("require-login")，前端回退登录页。
// 事件命名沿用 BrowserHost 的 emit 先例（browser_closed）。
// ══════════════════════════════════════════════════════════════

/// 桥接下发的桌面配置子集。
///
/// 对标 tenant bootstrap 一次下发思想：版本/语言/附件上限/菜单与特性开关一次给齐，
/// 前端启动时经 `auth_bridge_get_config` 拉取并缓存，不再逐项询问。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthBridgeConfig {
    /// 桌面端版本号
    pub version: String,
    /// 默认语言（BCP-47，如 zh-CN）
    pub locale: String,
    /// 附件数量上限
    pub max_attachment_count: u32,
    /// 管理菜单是否可见
    pub menus_visible: bool,
    /// 已启用的特性开关
    pub features: Vec<String>,
    /// 附件可信上传目录（按 OS 区分，首个为默认写入目录）
    pub trusted_upload_dirs: Vec<String>,
}

impl Default for AuthBridgeConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            locale: "zh-CN".into(),
            max_attachment_count: 20,
            menus_visible: false,
            features: Vec::new(),
            trusted_upload_dirs: vec![default_trusted_upload_dir().to_string_lossy().to_string()],
        }
    }
}

impl AuthBridgeConfig {
    /// 可信上传目录：首选配置首项，缺省回退按 OS 默认。
    pub fn get_trusted_upload_dir(&self) -> PathBuf {
        if let Some(first) = self.trusted_upload_dirs.first() {
            if !first.is_empty() {
                return PathBuf::from(first);
            }
        }
        default_trusted_upload_dir()
    }
}

/// 按 OS 区分的可信上传目录（T31 蓝图附件可信路径）。
///
/// - darwin/macos → `~/Library/Application Support/NeoTrix/uploads`
/// - win32/windows → `%APPDATA%/NeoTrix/uploads`
/// - 其他 → `temp_dir()/NeoTrix/uploads`
fn default_trusted_upload_dir() -> PathBuf {
    match std::env::consts::OS {
        "macos" | "darwin" => match std::env::var("HOME") {
            Ok(home) if !home.is_empty() => {
                PathBuf::from(home).join("Library/Application Support/NeoTrix/uploads")
            }
            _ => std::env::temp_dir().join("NeoTrix/uploads"),
        },
        "windows" | "win32" => match std::env::var("APPDATA") {
            Ok(appdata) if !appdata.is_empty() => PathBuf::from(appdata).join("NeoTrix/uploads"),
            _ => std::env::temp_dir().join("NeoTrix/uploads"),
        },
        _ => std::env::temp_dir().join("NeoTrix/uploads"),
    }
}

/// 桥鉴权事件表（T31）：四事件收敛，前端 switch 兜底用 snake_case 序列化名。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeEvent {
    TokenExpired,
    RequireLogin,
    AuthLogout,
    SessionExpired,
}

impl BridgeEvent {
    /// 实际 emit 通道名（沿用 P0-3 kebab-case 先例）。
    pub fn event_name(self) -> &'static str {
        match self {
            Self::TokenExpired => "token-expired",
            Self::RequireLogin => "require-login",
            Self::AuthLogout => "auth-logout",
            Self::SessionExpired => "session-expired",
        }
    }
}

/// token 过期事件负载。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExpiredPayload {
    /// 过期原因（token-expired / unauthorized / refresh-failed）
    pub reason: String,
    /// 事件毫秒时间戳
    pub at_ms: u64,
}

impl TokenExpiredPayload {
    /// 以当前时间为戳构造负载。
    pub fn now(reason: impl Into<String>) -> Self {
        let at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or_default();
        Self {
            reason: reason.into(),
            at_ms,
        }
    }
}

/// 鉴权桥：配置下发与 token 事件回灌。
pub struct AuthBridge;

impl AuthBridge {
    /// 下发桌面桥配置（前端启动时调用一次）。
    pub fn get_config() -> AuthBridgeConfig {
        AuthBridgeConfig::default()
    }

    /// 回灌 token 过期：前端收到后先静默续签，续签失败再等 `require-login`。
    pub fn notify_token_expired(app: &AppHandle, reason: &str) -> Result<(), BrowserError> {
        app.emit(
            BridgeEvent::TokenExpired.event_name(),
            TokenExpiredPayload::now(reason),
        )
        .map_err(|e| BrowserError::Emit(format!("{e}")))?;
        Ok(())
    }

    /// 登出广播：前端清本地会话态。
    pub fn notify_logout(app: &AppHandle) -> Result<(), BrowserError> {
        app.emit(BridgeEvent::AuthLogout.event_name(), ())
            .map_err(|e| BrowserError::Emit(format!("{e}")))?;
        Ok(())
    }

    /// 回退登录：静默续签已失败，前端跳转登录页。
    pub fn require_login(app: &AppHandle, reason: &str) -> Result<(), BrowserError> {
        app.emit(
            BridgeEvent::RequireLogin.event_name(),
            TokenExpiredPayload::now(reason),
        )
        .map_err(|e| BrowserError::Emit(format!("{e}")))?;
        Ok(())
    }

    /// 会话级过期（区别于 token 级）：服务端会话已失效，前端清态并回登录页。
    pub fn notify_session_expired(app: &AppHandle, reason: &str) -> Result<(), BrowserError> {
        app.emit(
            BridgeEvent::SessionExpired.event_name(),
            TokenExpiredPayload::now(reason),
        )
        .map_err(|e| BrowserError::Emit(format!("{e}")))?;
        Ok(())
    }
}

/// Tauri 命令：前端启动时拉取桥配置。
#[tauri::command]
pub fn auth_bridge_get_config() -> AuthBridgeConfig {
    AuthBridge::get_config()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Hello World</title></head><body></body></html>";
        assert_eq!(extract_title(html), "Hello World");
    }

    #[test]
    fn test_extract_title_empty() {
        let html = "<html><head></head><body></body></html>";
        assert_eq!(extract_title(html), "");
    }

    #[test]
    fn test_strip_html_simple() {
        let html = "<p>Hello <b>World</b></p>";
        let text = strip_html(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains('<'));
        assert!(!text.contains('>'));
    }

    #[test]
    fn test_strip_html_removes_script() {
        let html =
            "<html><head><script>alert('xss');</script></head><body><p>Hello</p></body></html>";
        let text = strip_html(html);
        assert!(text.contains("Hello"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("xss"));
    }

    #[test]
    fn test_strip_html_removes_style() {
        let html = "<html><head><style>body { color: red; }</style></head><body><p>Hello</p></body></html>";
        let text = strip_html(html);
        assert!(text.contains("Hello"));
        assert!(!text.contains("color"));
    }

    #[test]
    fn test_strip_html_entities() {
        let html = "<p>AT&amp;T &lt;test&gt;</p>";
        let text = strip_html(html);
        assert!(text.contains("AT&T"));
        assert!(text.contains("<test>"));
    }

    #[test]
    fn test_page_content_summary() {
        let pc = PageContent {
            url: "https://example.com".into(),
            title: "Test".into(),
            html: String::new(),
            text: "Hello World".into(),
        };
        let s = pc.summary(100);
        assert!(s.contains("Test"));
        assert!(s.contains("Hello World"));
        assert!(s.contains("example.com"));
    }

    #[test]
    fn test_page_content_summary_truncate() {
        let pc = PageContent {
            url: "https://example.com".into(),
            title: "Test".into(),
            html: String::new(),
            text: "A".repeat(1000),
        };
        let s = pc.summary(50);
        assert!(s.len() < 200); // 标题 + url 开销之外的内容被截断
    }

    #[test]
    fn test_auth_bridge_config_default() {
        let cfg = AuthBridge::get_config();
        assert_eq!(cfg.locale, "zh-CN");
        assert_eq!(cfg.max_attachment_count, 20);
        assert!(!cfg.menus_visible);
        assert!(!cfg.version.is_empty());
        // 序列化往返（前端收到的 JSON 形状）
        let v = serde_json::to_value(&cfg).unwrap_or_default();
        assert_eq!(v["locale"], "zh-CN");
        assert_eq!(v["max_attachment_count"], 20);
    }

    #[test]
    fn test_token_expired_payload() {
        let p = TokenExpiredPayload::now("token-expired");
        assert_eq!(p.reason, "token-expired");
        let v = serde_json::to_value(&p).unwrap_or_default();
        assert_eq!(v["reason"], "token-expired");
        assert!(v["at_ms"].as_u64().is_some());
    }

    #[test]
    fn test_bridge_event_serialization_snapshot() {
        // serde snake_case 快照：前端 switch 兜底依赖此形状
        let cases = [
            (BridgeEvent::TokenExpired, "token_expired", "token-expired"),
            (BridgeEvent::RequireLogin, "require_login", "require-login"),
            (BridgeEvent::AuthLogout, "auth_logout", "auth-logout"),
            (
                BridgeEvent::SessionExpired,
                "session_expired",
                "session-expired",
            ),
        ];
        for (ev, serde_name, emit_name) in cases {
            let v = serde_json::to_value(ev).unwrap_or_default();
            assert_eq!(v.as_str().unwrap_or_default(), serde_name);
            assert_eq!(ev.event_name(), emit_name);
        }
    }

    #[test]
    fn test_trusted_upload_dir_non_empty() {
        let cfg = AuthBridgeConfig::default();
        assert!(!cfg.trusted_upload_dirs.is_empty());
        let dir = cfg.get_trusted_upload_dir();
        assert!(!dir.as_os_str().is_empty());
        // 空配置回退 OS 默认仍非空
        let empty = AuthBridgeConfig {
            trusted_upload_dirs: Vec::new(),
            ..AuthBridgeConfig::default()
        };
        assert!(!empty.get_trusted_upload_dir().as_os_str().is_empty());
    }
}
