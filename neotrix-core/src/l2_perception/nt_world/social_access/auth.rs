//! AuthService — 统一认证服务
//!
//! 管理所有平台的 OAuth 2.0 认证流程。
//! 支持自动浏览器打开 + 本地回调服务器捕获 code 的完整自动登录流程。

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl, CsrfToken, RedirectUrl, Scope};
use oauth2::{EndpointNotSet, EndpointSet};
use oauth2::basic::{BasicClient, BasicTokenResponse};

use crate::l2_perception::nt_world::social_access::traits::*;
use crate::l2_perception::nt_world::social_access::SocialAccessError;

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 默认本地 OAuth 回调端口。
pub const DEFAULT_CALLBACK_PORT: u16 = 18180;

/// 认证服务
pub struct AuthService {
    sessions: HashMap<SocialPlatform, SessionEntry>,
    /// oauth2 v5 typestate：auth+token 端点已 set（其余未 set）。
    clients: HashMap<
        SocialPlatform,
        BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    >,
    callback_port: u16,
}

impl AuthService {
    /// 构造**不含任何客户端凭据**的认证服务。
    ///
    /// # ⛔ 为何默认不注册 OAuth client（2026-10-03 修复 D5）
    ///
    /// 原实现在此写死占位凭据：
    /// ```text
    /// BasicClient::new(ClientId::new("twitter_client_id".to_string()))
    ///     .set_client_secret(ClientSecret::new("twitter_client_secret".to_string()))
    /// ```
    /// 问题不是「未配置」，而是**看起来已配置**：
    ///
    /// 1. `get_x_auth_url()` 能产出语法完整的授权 URL，调用方无从分辨它是否可用
    ///    —— 本仓实测拿它换 token 得到 HTTP 400 `Missing required parameter [client_secret]`；
    /// 2. 用户会走完「打开浏览器 → 授权 → 回调」整条流程，**到最后一步才失败**，
    ///    而失败点在 token 交换，不在登录环节；
    /// 3. 占位串进了二进制，`strings` 即可看到「凭据已配置」的假象。
    ///
    /// ⇒ 改为：默认空 registry，OAuth client 由 [`Self::with_x_oauth`]
    /// 显式注入真实凭据。未配置时 `get_x_auth_url()` 直接返回可执行的错误，
    /// 而不是产出一个诱人的假 URL。
    ///
    /// cookie 路线（[`Self::login_x_cookies`]）不依赖任何 client 凭据，
    /// 因此**无需配置即可使用** —— 这也是 bird / opencli 的路线。
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            clients: HashMap::new(),
            callback_port: DEFAULT_CALLBACK_PORT,
        }
    }

    /// 注册真实 X OAuth2 client。
    ///
    /// 凭据由调用方提供 —— 不进源码、不进二进制、不进日志。
    pub fn with_x_oauth(mut self, client_id: &str, client_secret: &str) -> SocialAccessResult<Self> {
        let port = self.callback_port;
        let client = BasicClient::new(ClientId::new(client_id.to_string()))
            .set_client_secret(ClientSecret::new(client_secret.to_string()))
            .set_auth_uri(
                AuthUrl::new("https://api.x.com/2/oauth2/authorize".to_string())
                    .map_err(|e| Self::url_err(SocialPlatform::Twitter, e))?,
            )
            .set_token_uri(
                TokenUrl::new("https://api.x.com/2/oauth2/token".to_string())
                    .map_err(|e| Self::url_err(SocialPlatform::Twitter, e))?,
            )
            .set_redirect_uri(
                RedirectUrl::new(format!("http://localhost:{}/callback", port))
                    .map_err(|e| Self::url_err(SocialPlatform::Twitter, e))?,
            );
        self.clients.insert(SocialPlatform::Twitter, client);
        Ok(self)
    }

    /// 注册真实 Reddit OAuth2 client。
    pub fn with_reddit_oauth(mut self, client_id: &str) -> SocialAccessResult<Self> {
        let port = self.callback_port;
        let client = BasicClient::new(ClientId::new(client_id.to_string()))
            .set_auth_uri(
                AuthUrl::new("https://www.reddit.com/api/v1/authorize".to_string())
                    .map_err(|e| Self::url_err(SocialPlatform::Reddit, e))?,
            )
            .set_token_uri(
                TokenUrl::new("https://oauth.reddit.com/api/v1/access_token".to_string())
                    .map_err(|e| Self::url_err(SocialPlatform::Reddit, e))?,
            )
            .set_redirect_uri(
                RedirectUrl::new(format!("http://localhost:{}/reddit/callback", port))
                    .map_err(|e| Self::url_err(SocialPlatform::Reddit, e))?,
            );
        self.clients.insert(SocialPlatform::Reddit, client);
        Ok(self)
    }

    /// URL 解析失败的统一错误（取代生产路径上的 `unwrap`）。
    fn url_err(platform: SocialPlatform, e: impl std::fmt::Display) -> SocialAccessError {
        SocialAccessError::AuthFailed {
            platform,
            reason: format!("invalid endpoint URL: {}", e),
        }
    }

    /// 该平台是否已配置 OAuth client。
    pub fn has_oauth_client(&self, platform: &SocialPlatform) -> bool {
        self.clients.contains_key(platform)
    }

    /// 获取回调端口号
    pub fn callback_port(&self) -> u16 {
        self.callback_port
    }

    /// 自动登录 X/Twitter — 完整流程：
    /// 1. 生成授权 URL
    /// 2. 启动本地回调服务器
    /// 3. 打开浏览器
    /// 4. 等待用户授权并捕获 code
    /// 5. 交换 code 获取 access token
    pub fn login_x_auto(&mut self) -> Result<SessionEntry, SocialAccessError> {
        // ⛔ 先校验配置 —— 避免「生成假 URL → 走完浏览器流程 → 最后一步才失败」
        if !self.has_oauth_client(&SocialPlatform::Twitter) {
            return Err(Self::x_oauth_unconfigured());
        }

        let (auth_url, _csrf_token) = self.get_x_auth_url()?;

        // 启动本地回调服务器
        let port = self.callback_port;
        let (code_sender, code_receiver) = std::sync::mpsc::channel::<String>();

        // ⛔ 回调服务器的 bind 失败**必须回传**。
        //    原实现是 `.expect(..)` 直接 panic（违 AGENTS.md 硬规则）；
        //    改成返回 Result 后，若在此丢弃（`let _ =`），端口占用会变成
        //    **静默失败** —— 用户看到「点了授权但什么都没发生」，比 panic 更难排查。
        //    故用 oneshot channel 把 bind 结果送回主流程。
        let (bind_sender, bind_receiver) = std::sync::mpsc::channel::<Result<(), String>>();

        let server_handle = thread::spawn(move || {
            let bind_result = start_callback_server(port, code_sender);
            let _ = bind_sender.send(bind_result.as_ref().map(|_| ()).map_err(Clone::clone));
            bind_result
        });

        // 先确认回调端口真的绑上了，再去开浏览器。
        match bind_receiver.recv() {
            Ok(Err(e)) => {
                return Err(SocialAccessError::AuthFailed {
                    platform: SocialPlatform::Twitter,
                    reason: format!(
                        "could not start the OAuth callback listener: {}. \
                         Port {} is likely still in TIME_WAIT from a previous run.",
                        e, port
                    ),
                });
            }
            Err(_) => {
                return Err(SocialAccessError::AuthFailed {
                    platform: SocialPlatform::Twitter,
                    reason: "callback listener thread died before reporting readiness".into(),
                });
            }
            Ok(Ok(())) => {}
        }

        // 打开浏览器
        let _ = std::process::Command::new("/usr/bin/open")
            .arg(&auth_url)
            .spawn();

        // 等待回调 code
        let code = code_receiver.recv().map_err(|e| SocialAccessError::AuthFailed {
            platform: SocialPlatform::Twitter,
            reason: format!("等待授权回调超时: {}", e),
        })?;

        // 等待服务器线程结束
        let _ = server_handle.join();

        // 交换 code 获取 token
        let session = self.exchange_x_code(&code)?;
        Ok(session)
    }

    /// 获取 X/Twitter 授权 URL
    pub fn get_x_auth_url(&self) -> Result<(String, String), SocialAccessError> {
        // ⛔ 未配置真实凭据时**不产出**授权 URL —— 见 `AuthService::new` 的说明：
        //    假 URL 会让人走完整套浏览器流程才在 token 交换处失败。
        if !self.has_oauth_client(&SocialPlatform::Twitter) {
            return Err(Self::x_oauth_unconfigured());
        }
        let client = self.clients.get(&SocialPlatform::Twitter)
            .ok_or_else(|| SocialAccessError::Platform("X client not configured".into()))?;

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("tweet.read users.read".to_string()))
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().to_string()))
    }

    /// X OAuth 未配置时的可执行错误信息。
    fn x_oauth_unconfigured() -> SocialAccessError {
        SocialAccessError::AuthFailed {
            platform: SocialPlatform::Twitter,
            reason: "X OAuth client is not configured. Register an app at developer.x.com \
                     and pass its real client_id/client_secret to \
                     AuthService::with_x_oauth(). Alternatively use cookie auth: \
                     AuthService::login_x_cookies(auth_token, ct0)"
                .into(),
        }
    }

    /// 交换授权码获取 Access Token
    pub fn exchange_x_code(&mut self, code: &str) -> Result<SessionEntry, SocialAccessError> {
        let client = self.clients.get(&SocialPlatform::Twitter)
            .ok_or_else(|| SocialAccessError::Platform("X client not configured".into()))?;

        let token: BasicTokenResponse = client
            .exchange_code(oauth2::AuthorizationCode::new(code.to_string()))
            .request(&reqwest::blocking::Client::new())
            .map_err(|e| SocialAccessError::AuthFailed {
                platform: SocialPlatform::Twitter,
                reason: format!("token exchange failed: {}", e),
            })?;

        let access_token = oauth2::TokenResponse::access_token(&token).secret().to_string();
        let refresh_token = oauth2::TokenResponse::refresh_token(&token).map(|t| t.secret().to_string());

        let session = SessionEntry {
            platform: SocialPlatform::Twitter,
            credentials: Credentials {
                access_token: Some(access_token),
                refresh_token,
                expires_at: Some(now_ts() + 3600),
                oauth_state: Some("authenticated".to_string()),
            },
            state: AuthState::Authenticated,
        };

        self.sessions.insert(SocialPlatform::Twitter, session.clone());
        Ok(session)
    }

    /// 浏览器模拟登录 — 使用 UniversalBrowser 打开 Chrome 让用户手动登录，抓取 cookie
    #[cfg(feature = "stealth-net")]
    pub fn login_x_browser(&mut self) -> Result<SessionEntry, SocialAccessError> {
        use crate::l2_perception::nt_world::l1_facade::{UniversalBrowser, PlatformConfig};

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| SocialAccessError::Platform(format!("tokio runtime error: {}", e)))?;

        rt.block_on(async {
            let mut browser = UniversalBrowser::new();
            let config = PlatformConfig::twitter();

            browser.launch(config).await
                .map_err(|e| SocialAccessError::Platform(format!("browser launch error: {}", e)))?;

            // 打开 X 登录页，等待用户登录
            let result = browser.login_manual().await
                .map_err(|e| SocialAccessError::Platform(format!("login error: {}", e)))?;

            browser.close().await;

            if !result.success {
                return Err(SocialAccessError::AuthFailed {
                    platform: SocialPlatform::Twitter,
                    reason: result.error.unwrap_or("登录超时".into()),
                });
            }

            // 提取 auth_token 和 ct0
            let mut access_token = String::new();
            let mut ct0 = String::new();
            for cookie in &result.cookies {
                if cookie.name == "auth_token" {
                    access_token = cookie.value.clone();
                }
                if cookie.name == "ct0" {
                    ct0 = cookie.value.clone();
                }
            }

            let session = SessionEntry {
                platform: SocialPlatform::Twitter,
                credentials: Credentials {
                    access_token: Some(access_token),
                    refresh_token: None,
                    expires_at: None,
                    oauth_state: Some(ct0),
                },
                state: AuthState::Authenticated,
            };

            self.sessions.insert(SocialPlatform::Twitter, session.clone());
            Ok(session)
        })
    }

    /// 便捷方法：自动登录 X/Twitter
    #[cfg(feature = "stealth-net")]
    pub fn login_x(&mut self) -> Result<SessionEntry, SocialAccessError> {
        self.login_x_browser()
    }

    #[cfg(not(feature = "stealth-net"))]
    pub fn login_x(&mut self) -> Result<SessionEntry, SocialAccessError> {
        // 打开浏览器让用户登录
        let _ = std::process::Command::new("/usr/bin/open")
            .arg("https://x.com/login")
            .spawn();

        Err(SocialAccessError::AuthFailed {
            platform: SocialPlatform::Twitter,
            reason: "请在浏览器中登录 X/Twitter，然后使用 /social login x --token <auth_token> --ct0 <ct0> 提供 cookies".into(),
        })
    }

    /// Cookie-based login — 用户提供 `auth_token` 和 `ct0`。
    ///
    /// # 2026-10-03：补上原先缺失的输入校验
    ///
    /// 原实现接受任意字符串（含空串）并把 session 标记为
    /// `AuthState::Authenticated`。于是 `login_x_cookies("", "")`
    /// 会「登录成功」—— 后续每个请求都带着空 bearer 失败，
    /// 而失败点离根因很远。这是**假成功**，比直接报错坏得多。
    ///
    /// 长度下限取自同类项目的实测约束：bird/gobird 的 README 均记录
    /// `auth_token` 为 40 位十六进制、`ct0` 为 32–160 位字母数字。
    /// 此处只做**保守下限**校验（不强制十六进制/长度上限），
    /// 以免平台变更格式后把合法凭据误杀。
    pub fn login_x_cookies(&mut self, auth_token: &str, ct0: &str) -> Result<SessionEntry, SocialAccessError> {
        const MIN_AUTH_TOKEN: usize = 20;
        const MIN_CT0: usize = 16;

        let auth_token = auth_token.trim();
        let ct0 = ct0.trim();

        if auth_token.len() < MIN_AUTH_TOKEN {
            return Err(SocialAccessError::AuthFailed {
                platform: SocialPlatform::Twitter,
                reason: format!(
                    "auth_token is missing or too short ({} chars, need >= {}). \
                     Extract it from DevTools -> Application -> Cookies -> x.com -> auth_token",
                    auth_token.len(),
                    MIN_AUTH_TOKEN
                ),
            });
        }
        if ct0.len() < MIN_CT0 {
            return Err(SocialAccessError::AuthFailed {
                platform: SocialPlatform::Twitter,
                reason: format!(
                    "ct0 (CSRF token) is missing or too short ({} chars, need >= {}). \
                     Extract it the same way as auth_token",
                    ct0.len(),
                    MIN_CT0
                ),
            });
        }

        let session = SessionEntry {
            platform: SocialPlatform::Twitter,
            credentials: Credentials {
                access_token: Some(auth_token.to_string()),
                refresh_token: None,
                expires_at: None,
                oauth_state: Some(ct0.to_string()), // ct0 stored in oauth_state
            },
            state: AuthState::Authenticated,
        };

        self.sessions.insert(SocialPlatform::Twitter, session.clone());
        Ok(session)
    }

    pub fn login(
        &mut self,
        platform: SocialPlatform,
        _flow: AuthFlow,
        creds: Credentials,
    ) -> Result<SessionEntry, SocialAccessError> {
        let session = self._login_platform(platform.clone(), creds)?;
        self.sessions.insert(platform, session.clone());
        Ok(session)
    }

    pub fn refresh(&mut self, platform: SocialPlatform) -> Result<(), SocialAccessError> {
        let session = self.sessions.get_mut(&platform)
            .ok_or_else(|| SocialAccessError::AuthFailed {
                platform: platform.clone(),
                reason: "no session".into(),
            })?;
        if session.credentials.refresh_token.is_none() {
            return Err(SocialAccessError::AuthFailed {
                platform: platform.clone(),
                reason: "no refresh token".into(),
            });
        }
        Ok(())
    }

    pub fn logout(&mut self, platform: SocialPlatform) -> Result<(), SocialAccessError> {
        self.sessions.remove(&platform);
        Ok(())
    }

    pub fn get_session(&self, platform: SocialPlatform) -> Option<&SessionEntry> {
        self.sessions.get(&platform)
    }

    pub fn is_authenticated(&self, platform: SocialPlatform) -> bool {
        matches!(
            self.sessions.get(&platform).map(|s| &s.state),
            Some(AuthState::Authenticated)
        )
    }

    fn _login_platform(
        &self,
        platform: SocialPlatform,
        creds: Credentials,
    ) -> Result<SessionEntry, SocialAccessError> {
        self.clients.get(&platform)
            .ok_or_else(|| SocialAccessError::AuthFailed {
                platform: platform.clone(),
                reason: format!(
                    "no OAuth client registered for {:?}; \
                     call with_x_oauth()/with_reddit_oauth() first, \
                     or use the cookie flow",
                    platform
                ),
            })?;

        // ⛔ 原实现在此**不校验凭据内容**就把 session 标成 `Authenticated`
        //    并写死 `expires_at = now + 3600` 与 `oauth_state = "oauth_state"`。
        //    空 `access_token` 也能「登录成功」—— 假成功。
        //    改为：必须有 access_token 才算认证成功。
        let access_token = creds.access_token.filter(|t| !t.trim().is_empty()).ok_or_else(|| {
            SocialAccessError::AuthFailed {
                platform: platform.clone(),
                reason: "no access token supplied".into(),
            }
        })?;

        // ⛔ 不再伪造过期时间：只有真拿到 refresh token 时才设 expiry，
        //    否则留空表示「未知」优于「假装还有一小时」。
        let has_refresh = creds.refresh_token.is_some();

        Ok(SessionEntry {
            platform,
            credentials: Credentials {
                access_token: Some(access_token),
                refresh_token: creds.refresh_token,
                expires_at: if has_refresh { Some(now_ts() + 3600) } else { None },
                oauth_state: creds.oauth_state,
            },
            state: AuthState::Authenticated,
        })
    }

    pub fn stats(&self) -> HashMap<SocialPlatform, (usize, usize)> {
        let mut stats = HashMap::new();
        for (platform, session) in &self.sessions {
            let auth = if matches!(session.state, AuthState::Authenticated) { 1 } else { 0 };
            let guest = if matches!(session.state, AuthState::Guest) { 1 } else { 0 };
            stats.insert(platform.clone(), (auth, guest));
        }
        stats
    }
}

/// 启动本地回调服务器，监听授权回调中的 code。
///
/// ⛔ 原实现在此 `.expect("无法绑定本地回调端口")` —— 端口被占用时**整个进程 panic**。
///    这是生产路径上的 panic（违 AGENTS.md 硬规则），且发生概率不低：
///    18180 是固定端口，上一次 OAuth 流程崩溃后端口可能仍在 TIME_WAIT。
///    改为把错误送回调用方，由 `login_x_auto` 决定如何呈现。
fn start_callback_server(
    port: u16,
    code_sender: std::sync::mpsc::Sender<String>,
) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("bind 127.0.0.1:{} failed: {}", port, e))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_callback(stream, &code_sender);
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

/// 处理回调请求，提取 code 并发送
fn handle_callback(mut stream: TcpStream, code_sender: &std::sync::mpsc::Sender<String>) {
    let mut buffer = [0u8; 4096];
    let _ = stream.read(&mut buffer);

    let request = String::from_utf8_lossy(&buffer[..]);
    let code = extract_code_from_request(&request);

    if let Some(code) = code {
        let _ = code_sender.send(code);
    }

    // 发送 HTTP 200 响应
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>X 平台登录成功！您可以关闭此窗口。</h1></body></html>";
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// 从 HTTP 请求中提取 authorization code
fn extract_code_from_request(request: &str) -> Option<String> {
    // 查找 ?code= 或 &code= 参数
    for line in request.lines() {
        if line.contains("GET /callback") || line.contains("GET /callback?") {
            // 提取 query string
            if let Some(query_start) = line.find('?') {
                let query = &line[query_start + 1..];
                for param in query.split('&').flat_map(|p| p.split(' ')) {
                    if param.starts_with("code=") {
                        return Some(param[5..].to_string());
                    }
                }
            }
        }
        // 也检查 URL 中的 code 参数（带路径）
        if let Some(code_pos) = line.find("code=") {
            let after = &line[code_pos + 5..];
            let code_end = after.find(' ').or(after.find('\r')).or(after.find('\n'));
            if let Some(end) = code_end {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service_login() {
        let mut auth = AuthService::new();
        let session = SessionEntry::guest(SocialPlatform::Twitter);
        auth.sessions.insert(SocialPlatform::Twitter, session);
        assert!(!auth.is_authenticated(SocialPlatform::Twitter));
    }

    // ⛔ **D5 回归测试**：原测试断言「`get_x_auth_url()` 必定成功」。
//    那正是在给 bug 背书 —— 占位凭据下也能拿到一个语法完整、
//    但拿去换 token 必然 400 的 URL。
    //    新契约：**未注入真实凭据时必须报错，且不得产出 URL。**
    #[test]
    fn get_x_auth_url_errors_when_client_not_configured() {
        let auth = AuthService::new();
        assert!(!auth.has_oauth_client(&SocialPlatform::Twitter));
        let err = auth
            .get_x_auth_url()
            .expect_err("must not fabricate an auth URL without a real client");
        match err {
            SocialAccessError::AuthFailed { reason, .. } => {
                assert!(
                    reason.contains("not configured"),
                    "error must state the cause, got: {}",
                    reason
                );
                // 错误信息必须**可执行** —— 告诉用户下一步做什么
                assert!(
                    reason.contains("with_x_oauth"),
                    "error must name the remedy, got: {}",
                    reason
                );
            }
            other => panic!("expected AuthFailed, got {:?}", other),
        }
    }

    #[test]
    fn login_x_auto_fails_fast_without_client() {
        // ⛔ 关键：必须在**开浏览器之前**失败。
        //    旧行为会生成假 URL、拉起浏览器、走完授权，最后才在
        //    token 交换处失败 —— 用户白跑一圈。
        let mut auth = AuthService::new();
        assert!(auth.login_x_auto().is_err());
    }

    #[test]
    fn with_x_oauth_registers_client_and_yields_url() {
        let auth = AuthService::new()
            .with_x_oauth("test-client-id", "test-client-secret")
            .expect("registering a client with valid endpoints must succeed");
        assert!(auth.has_oauth_client(&SocialPlatform::Twitter));

        let (url, _state) = auth
            .get_x_auth_url()
            .expect("configured client must yield an auth URL");
        assert!(url.contains("api.x.com/2/oauth2/authorize"));
        assert!(url.contains("client_id=test-client-id"));
    }

    #[test]
    fn with_reddit_oauth_registers_client() {
        let auth = AuthService::new()
            .with_reddit_oauth("test-reddit-id")
            .expect("registering must succeed");
        assert!(auth.has_oauth_client(&SocialPlatform::Reddit));
    }

    // ── cookie 路线：输入校验（D7：消除假成功）─────────────────────

    #[test]
    fn login_x_cookies_rejects_empty_credentials() {
        let mut auth = AuthService::new();
        // ⛔ 旧实现：这两个都会「登录成功」—— 假成功
        assert!(auth.login_x_cookies("", "").is_err());
        assert!(auth.login_x_cookies("   ", "   ").is_err());
        assert!(auth.login_x_cookies("short", "alsoshort").is_err());
        // 一个够长一个不够长，也必须失败（不能只看总量）
        assert!(auth.login_x_cookies(&"a".repeat(40), "short").is_err());
        assert!(auth.login_x_cookies("short", &"b".repeat(40)).is_err());
        assert!(!auth.is_authenticated(SocialPlatform::Twitter));
    }

    #[test]
    fn login_x_cookies_accepts_realistic_credentials() {
        let mut auth = AuthService::new();
        let session = auth
            .login_x_cookies(&"a".repeat(40), &"b".repeat(32))
            .expect("realistic cookie lengths must be accepted");
        assert!(matches!(session.state, AuthState::Authenticated));
        assert_eq!(session.credentials.access_token.as_deref().map(str::len), Some(40));
        assert_eq!(session.credentials.oauth_state.as_deref().map(str::len), Some(32));
        assert!(auth.is_authenticated(SocialPlatform::Twitter));
    }

    #[test]
    fn login_x_cookies_trims_whitespace() {
        let mut auth = AuthService::new();
        let session = auth
            .login_x_cookies(&format!("  {}  ", "a".repeat(40)), &format!("{} ", "b".repeat(32)))
            .expect("surrounding whitespace must be tolerated");
        // ⛔ 若不 trim，token 会带上尾随空格并在请求头里变成非法值
        assert_eq!(session.credentials.access_token.as_deref().map(str::len), Some(40));
    }

    // ── `_login_platform`：空 token 不得判 Authenticated ───────────

    #[test]
    fn login_platform_rejects_empty_access_token() {
        let mut auth = AuthService::new()
            .with_x_oauth("cid", "csecret")
            .expect("client registration succeeds");
        // ⛔ 旧实现：空 token 也标 Authenticated 并伪造 expires_at
        let res = auth.login(SocialPlatform::Twitter, AuthFlow::OAuth2PKCE, Credentials::default());
        assert!(res.is_err(), "empty access token must not authenticate");
    }

    #[test]
    fn login_platform_requires_registered_client() {
        let mut auth = AuthService::new();
        let creds = Credentials {
            access_token: Some("t".into()),
            ..Default::default()
        };
        assert!(auth.login(SocialPlatform::Twitter, AuthFlow::OAuth2PKCE, creds).is_err());
    }

    #[test]
    fn login_platform_accepts_real_token_and_does_not_fake_expiry() {
        let mut auth = AuthService::new()
            .with_x_oauth("cid", "csecret")
            .expect("client registration succeeds");
        let creds = Credentials {
            access_token: Some("real-token".into()),
            refresh_token: None,
            expires_at: None,
            oauth_state: None,
        };
        let s = auth
            .login(SocialPlatform::Twitter, AuthFlow::OAuth2PKCE, creds)
            .expect("a real token must authenticate");
        assert!(matches!(s.state, AuthState::Authenticated));
        // ⛔ 无 refresh token 时不得伪造 expires_at
        assert_eq!(s.credentials.expires_at, None);
    }

    #[test]
    fn test_extract_code_from_request() {
        let request = "GET /callback?code=abc123def456 HTTP/1.1\r\nHost: localhost:18180\r\n";
        let code = extract_code_from_request(request);
        assert_eq!(code, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_callback_port() {
        let auth = AuthService::new();
        assert_eq!(auth.callback_port(), DEFAULT_CALLBACK_PORT);
    }
}
