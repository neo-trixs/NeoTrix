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
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::http_client;

use crate::l2_perception::nt_world::social_access::traits::*;
use crate::l2_perception::nt_world::social_access::SocialAccessError;

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 认证服务
pub struct AuthService {
    sessions: HashMap<SocialPlatform, SessionEntry>,
    clients: HashMap<SocialPlatform, BasicClient>,
    callback_port: u16,
}

impl AuthService {
    pub fn new() -> Self {
        let callback_port = 18180; // 本地回调端口

        let mut clients = HashMap::new();

        // X/Twitter OAuth2 PKCE client
        let twitter_client = BasicClient::new(
            ClientId::new("twitter_client_id".to_string()),
            Some(ClientSecret::new("twitter_client_secret".to_string())),
            AuthUrl::new("https://api.x.com/2/oauth2/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://api.x.com/2/oauth2/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}/callback", callback_port)).unwrap());

        clients.insert(SocialPlatform::Twitter, twitter_client);

        // Reddit OAuth2 client
        let reddit_client = BasicClient::new(
            ClientId::new("reddit_client_id".to_string()),
            None,
            AuthUrl::new("https://www.reddit.com/api/v1/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://oauth.reddit.com/api/v1/access_token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}/reddit/callback", callback_port)).unwrap());

        clients.insert(SocialPlatform::Reddit, reddit_client);

        Self { sessions: HashMap::new(), clients, callback_port }
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
        let (auth_url, _csrf_token) = self.get_x_auth_url()?;

        // 启动本地回调服务器
        let port = self.callback_port;
        let (code_sender, code_receiver) = std::sync::mpsc::channel::<String>();

        let server_handle = thread::spawn(move || {
            start_callback_server(port, code_sender);
        });

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
        let client = self.clients.get(&SocialPlatform::Twitter)
            .ok_or_else(|| SocialAccessError::Platform("X client not configured".into()))?;

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("tweet.read users.read".to_string()))
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().to_string()))
    }

    /// 交换授权码获取 Access Token
    pub fn exchange_x_code(&mut self, code: &str) -> Result<SessionEntry, SocialAccessError> {
        let client = self.clients.get(&SocialPlatform::Twitter)
            .ok_or_else(|| SocialAccessError::Platform("X client not configured".into()))?;

        let token: BasicTokenResponse = client
            .exchange_code(oauth2::AuthorizationCode::new(code.to_string()))
            .request(http_client)
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
        use crate::l1_action::nt_io::universal_browser::{UniversalBrowser, PlatformConfig};

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

    /// Cookie-based login — 用户提供 auth_token 和 ct0
    pub fn login_x_cookies(&mut self, auth_token: &str, ct0: &str) -> Result<SessionEntry, SocialAccessError> {
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
        let _client = self.clients.get(&platform)
            .ok_or_else(|| SocialAccessError::Platform("Client not configured".into()))?;

        Ok(SessionEntry {
            platform,
            credentials: Credentials {
                access_token: creds.access_token,
                refresh_token: creds.refresh_token,
                expires_at: Some(now_ts() + 3600),
                oauth_state: Some("oauth_state".to_string()),
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

/// 启动本地回调服务器，监听授权回调中的 code
fn start_callback_server(port: u16, code_sender: std::sync::mpsc::Sender<String>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .expect("无法绑定本地回调端口");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_callback(stream, &code_sender);
            }
            Err(_) => continue,
        }
    }
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

    #[test]
    fn test_get_x_auth_url() {
        let auth = AuthService::new();
        let (url, _state) = auth.get_x_auth_url().unwrap();
        println!("AUTH_URL={}", url);
        assert!(url.contains("api.x.com/2/oauth2/authorize"));
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
        assert_eq!(auth.callback_port(), 18180);
    }
}
