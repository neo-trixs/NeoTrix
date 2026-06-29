//! BrowsingAgent — 自主浏览 agent，支持导航、表单填写、点击、登陆、知识吸收
//!
//! 包装 WebNavigator (CDP 浏览器) 提供 agent 可调用的高维操作。
//! 登陆支持自动填充 + 人工兜底。

use crate::neotrix::nt_act_social::web_navigator::{LoginCredentials, WebNavigator};
use crate::neotrix::nt_mind::credential_manager::CredentialManager;
use crate::neotrix::nt_mind::webapp_agent::WebAppRegistry;

/// 浏览器状态
pub enum BrowserStatus {
    Idle,
    Navigating { url: String },
    WaitingLogin { url: String, reason: String },
    Error(String),
}

/// BrowsingAgent — agent 可调用的自主浏览器操作
pub struct BrowsingAgent {
    nav: Option<WebNavigator>,
    current_session: Option<String>,
    current_url: String,
    status: BrowserStatus,
}

impl BrowsingAgent {
    pub fn new() -> Self {
        Self {
            nav: None,
            current_session: None,
            current_url: String::new(),
            status: BrowserStatus::Idle,
        }
    }

    /// 确保浏览器已启动, 返回 (nav, session) 元组
    fn take_browser(&mut self) -> Result<(WebNavigator, Option<String>), String> {
        let nav = self.nav.take().unwrap_or_else(|| {
            let mut n = WebNavigator::new();
            n.launch().ok();
            n
        });
        let session = self.current_session.take();
        Ok((nav, session))
    }

    fn put_browser(&mut self, nav: WebNavigator, session: Option<String>) {
        self.nav = Some(nav);
        self.current_session = session;
    }

    /// 导航到 URL 并返回页面文本
    pub async fn navigate(&mut self, url: &str) -> Result<String, String> {
        self.status = BrowserStatus::Navigating {
            url: url.to_string(),
        };
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };
        nav.navigate(&sess, url)?;
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        let text = nav.get_page_text(&sess)?;
        self.current_url = url.to_string();
        self.put_browser(nav, Some(sess));
        self.status = BrowserStatus::Idle;
        Ok(text)
    }

    /// 填充表单字段
    pub fn fill(&mut self, selector: &str, value: &str) -> Result<(), String> {
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };
        let result = nav.fill(&sess, selector, value);
        self.put_browser(nav, Some(sess));
        result
    }

    /// 点击元素
    pub fn click(&mut self, selector: &str) -> Result<(), String> {
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };
        let result = nav.click(&sess, selector);
        self.put_browser(nav, Some(sess));
        result
    }

    /// 提取页面/元素文本
    pub fn extract(&mut self, selector: Option<&str>) -> Result<String, String> {
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };
        let result = match selector {
            Some(sel) => {
                let js = format!(
                    r#"(() => {{ const el = document.querySelector('{}'); return el?.textContent?.trim() || ''; }})()"#,
                    sel.replace('\'', "\\'")
                );
                let val = nav.evaluate_js(&sess, &js)?;
                Ok(val.as_str().unwrap_or("").to_string())
            }
            None => nav.get_page_text(&sess),
        };
        self.put_browser(nav, Some(sess));
        result
    }

    /// 截图 (base64)
    pub fn screenshot(&mut self) -> Result<String, String> {
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };
        let result = nav.screenshot(&sess);
        self.put_browser(nav, Some(sess));
        result
    }

    /// 获取当前 URL
    pub fn current_url(&self) -> &str {
        &self.current_url
    }

    /// 获取当前状态
    pub fn status(&self) -> &BrowserStatus {
        &self.status
    }

    /// 尝试自动登陆，若失败则设置 WaitingLogin 状态让前端介入
    pub async fn try_login(
        &mut self,
        url: &str,
        cred_mgr: &mut CredentialManager,
    ) -> Result<String, String> {
        let host = extract_domain(url);
        let entries = cred_mgr.find(&host);

        if entries.is_empty() {
            self.status = BrowserStatus::WaitingLogin {
                url: url.to_string(),
                reason: format!(
                    "No stored credentials for {}. User intervention needed.",
                    host
                ),
            };
            return Err(format!("LOGIN_REQUIRED:{}:{}", host, url));
        }

        let entry = &entries[0];
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };

        nav.navigate(&sess, url)?;
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let credentials = LoginCredentials {
            username_field: "input[name='email'], input[name='username'], input[type='email'], input[type='text']".into(),
            password_field: "input[type='password']".into(),
            username: entry.username.clone(),
            password: entry.password.clone(),
            submit_selector: Some("button[type='submit']".into()),
            wait_for_url: None,
        };

        match nav.login_flow(&sess, url, &credentials) {
            Ok(page) => {
                self.current_url = url.to_string();
                self.status = BrowserStatus::Idle;
                let _ = nav.save_cookies(&sess, &host);
                self.put_browser(nav, Some(sess));
                Ok(format!(
                    "Successfully logged into {} as {}. Page title: {}",
                    host, entry.username, page.title
                ))
            }
            Err(e) => {
                self.status = BrowserStatus::WaitingLogin {
                    url: url.to_string(),
                    reason: format!(
                        "Auto-login failed for {}: {}. User intervention needed.",
                        host, e
                    ),
                };
                self.put_browser(nav, Some(sess));
                Err(format!("LOGIN_REQUIRED:{}:{}:{}", host, url, e))
            }
        }
    }

    /// 人工提供凭据后继续登录
    pub async fn login_with_credentials(
        &mut self,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<String, String> {
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };

        nav.navigate(&sess, url)?;
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let credentials = LoginCredentials {
            username_field: "input[name='email'], input[name='username'], input[type='email'], input[type='text']".into(),
            password_field: "input[type='password']".into(),
            username: username.into(),
            password: password.into(),
            submit_selector: Some("button[type='submit']".into()),
            wait_for_url: None,
        };

        match nav.login_flow(&sess, url, &credentials) {
            Ok(page) => {
                self.current_url = url.to_string();
                self.status = BrowserStatus::Idle;
                let host = extract_domain(url);
                let _ = nav.save_cookies(&sess, &host);
                self.put_browser(nav, Some(sess));
                Ok(format!("Login successful. Page: {}", page.title))
            }
            Err(e) => {
                self.status = BrowserStatus::Error(format!("Login failed: {}", e));
                self.put_browser(nav, Some(sess));
                Err(e)
            }
        }
    }

    /// 吸收页面内容到 WebAppRegistry
    pub fn ingest_current_page(&mut self, registry: &mut WebAppRegistry) -> Result<String, String> {
        let (nav, session) = self.take_browser()?;
        let sess = match session {
            Some(s) => s,
            None => {
                let s = nav.new_page()?;
                self.current_session = Some(s.clone());
                s
            }
        };
        let extract = match nav.extract_page(&sess) {
            Ok(e) => e,
            Err(e) => {
                self.put_browser(nav, Some(sess));
                return Err(e);
            }
        };
        let html = extract.html.clone().unwrap_or_default();
        let knowledge = registry.ingest_from_browser(&self.current_url, &extract.title, &html);
        self.put_browser(nav, Some(sess));
        Ok(format!(
            "Ingested {} chars from '{}' (summary: {})",
            extract.text.len(),
            extract.title,
            knowledge.summary,
        ))
    }

    /// 关闭浏览器
    pub fn close(&mut self) {
        if let Some(mut nav) = self.nav.take() {
            nav.close();
        }
        self.current_session = None;
        self.current_url.clear();
        self.status = BrowserStatus::Idle;
    }
}

impl Drop for BrowsingAgent {
    fn drop(&mut self) {
        self.close();
    }
}

fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or(url)
        .split(':')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/page"), "example.com");
        assert_eq!(
            extract_domain("http://sub.example.com:8080/path"),
            "sub.example.com"
        );
        assert_eq!(extract_domain("example.com"), "example.com");
    }

    #[test]
    fn test_initial_state() {
        let agent = BrowsingAgent::new();
        assert!(matches!(agent.status(), BrowserStatus::Idle));
    }
}
