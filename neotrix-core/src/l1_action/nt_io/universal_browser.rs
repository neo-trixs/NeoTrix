//! Universal Browser — 统一浏览器能力骨架
//!
//! 融合所有浏览器相关能力到单一入口：
//! - `stealth_browser.rs` → 底层引擎 (anti-detection + fingerprint)
//! - `nt_media/auth.rs::CookieJar` → Cookie 持久化
//! - `social_access/auth.rs` → 登录流程 (OAuth2/Cookie/Manual)
//! - `channel.rs` → 多后端路由
//!
//! 设计原则:
//! 1. Single Source of Truth — 一个引擎，一套 Cookie
//! 2. Composable — social_access / trade / 任何模块都能用
//! 3. Channel-Aware — 与 channel.rs 路由集成
//! 4. Platform-Agnostic — 适用于 X/Twitter / 富通天下 / 任意平台

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 平台配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// 平台标识 (e.g., "twitter", "futong")
    pub id: String,
    /// 登录 URL
    pub login_url: String,
    /// 登录成功后跳转 URL (用于检测登录状态)
    pub success_url: Option<String>,
    /// Cookie 持久化路径
    pub cookie_path: Option<PathBuf>,
    /// 额外启动参数
    pub extra_args: Vec<String>,
    /// 请求超时
    pub timeout: Duration,
}

impl PlatformConfig {
    pub fn twitter() -> Self {
        Self {
            id: "twitter".into(),
            login_url: "https://x.com/login".into(),
            success_url: Some("https://x.com/home".into()),
            cookie_path: Some(dirs::home_dir().unwrap_or_default().join(".neotrix/cookies/twitter.json")),
            extra_args: vec![],
            timeout: Duration::from_secs(30),
        }
    }

    pub fn futong() -> Self {
        Self {
            id: "futong".into(),
            login_url: "https://www.futong.com.cn/login".into(),
            success_url: Some("https://www.futong.com.cn/dashboard".into()),
            cookie_path: Some(dirs::home_dir().unwrap_or_default().join(".neotrix/cookies/futong.json")),
            extra_args: vec![],
            timeout: Duration::from_secs(30),
        }
    }

    pub fn custom(id: &str, login_url: &str) -> Self {
        Self {
            id: id.into(),
            login_url: login_url.into(),
            success_url: None,
            cookie_path: Some(dirs::home_dir().unwrap_or_default().join(format!(".neotrix/cookies/{}.json", id))),
            extra_args: vec![],
            timeout: Duration::from_secs(30),
        }
    }
}

/// Cookie 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieEntry {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<u64>,
    #[serde(default)]
    pub secure: bool,
}

/// 登录凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredentials {
    /// 用户名/邮箱
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// Auth Token (Cookie-based)
    pub auth_token: Option<String>,
    /// CT0 Token (X/Twitter specific)
    pub ct0: Option<String>,
    /// 额外 cookies
    pub extra_cookies: HashMap<String, String>,
}

/// 浏览器操作结果
#[derive(Debug, Clone)]
pub struct BrowserResult {
    pub success: bool,
    pub content: Option<String>,
    pub cookies: Vec<CookieEntry>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Cookie 存储 — 从 nt_media/auth.rs 合并
// ═══════════════════════════════════════════════════════════════════════════

/// 线程安全的 Cookie 存储
#[derive(Debug, Clone)]
pub struct CookieStore {
    inner: Arc<RwLock<HashMap<String, Vec<CookieEntry>>>>,
    file_path: Option<PathBuf>,
}

impl CookieStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            file_path: None,
        }
    }

    pub fn with_file(path: PathBuf) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            file_path: Some(path),
        }
    }

    /// 从文件加载 cookies
    pub async fn load(&self) -> Result<(), String> {
        let path = match &self.file_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        if !path.exists() {
            return Ok(());
        }

        let data = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| format!("Failed to read cookie file: {}", e))?;

        let store: HashMap<String, Vec<CookieEntry>> = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse cookie file: {}", e))?;

        *self.inner.write().await = store;
        Ok(())
    }

    /// 保存 cookies 到文件
    pub async fn save(&self) -> Result<(), String> {
        let path = match &self.file_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let data = self.inner.read().await;
        let json = serde_json::to_string_pretty(&*data)
            .map_err(|e| format!("Failed to serialize cookies: {}", e))?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create cookie dir: {}", e))?;
        }

        tokio::fs::write(&path, json)
            .await
            .map_err(|e| format!("Failed to write cookie file: {}", e))?;

        Ok(())
    }

    /// 设置 cookie
    pub async fn set(&self, domain: &str, cookie: CookieEntry) {
        let mut data = self.inner.write().await;
        data.entry(domain.to_string())
            .or_insert_with(Vec::new)
            .push(cookie);
    }

    /// 获取域名的所有 cookies
    pub async fn get_all(&self, domain: &str) -> Vec<CookieEntry> {
        let data = self.inner.read().await;
        data.get(domain).cloned().unwrap_or_default()
    }

    /// 获取指定 cookie
    pub async fn get(&self, domain: &str, name: &str) -> Option<CookieEntry> {
        let data = self.inner.read().await;
        data.get(domain)?
            .iter()
            .find(|c| c.name == name)
            .cloned()
    }

    /// 清除域名的所有 cookies
    pub async fn clear(&self, domain: &str) {
        let mut data = self.inner.write().await;
        data.remove(domain);
    }

    /// 从 chromiumoxide cookies 导入
    pub async fn import_chromium_cookies(&self, domain: &str, cookies: Vec<chromiumoxide::cdp::browser_protocol::network::Cookie>) {
        let mut data = self.inner.write().await;
        let entries: Vec<CookieEntry> = cookies.into_iter().map(|c| {
            CookieEntry {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
                expires: if c.expires > 0.0 { Some(c.expires as u64) } else { None },
                secure: c.secure,
            }
        }).collect();
        data.insert(domain.to_string(), entries);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Universal Browser — 统一浏览器引擎
// ═══════════════════════════════════════════════════════════════════════════

pub struct UniversalBrowser {
    /// 底层 stealth 浏览器
    browser: Arc<RwLock<Option<chromiumoxide::Browser>>>,
    /// Cookie 存储
    cookie_store: CookieStore,
    /// 当前平台配置
    config: Arc<RwLock<Option<PlatformConfig>>>,
    /// 代理
    proxy: Option<String>,
}

impl UniversalBrowser {
    pub fn new() -> Self {
        Self {
            browser: Arc::new(RwLock::new(None)),
            cookie_store: CookieStore::new(),
            config: Arc::new(RwLock::new(None)),
            proxy: None,
        }
    }

    pub fn with_proxy(mut self, proxy: &str) -> Self {
        self.proxy = Some(proxy.to_string());
        self
    }

    pub fn with_cookie_store(mut self, store: CookieStore) -> Self {
        self.cookie_store = store;
        self
    }

    /// 启动浏览器
    pub async fn launch(&mut self, config: PlatformConfig) -> Result<(), String> {
        use chromiumoxide::browser::{Browser as CBrowser, BrowserConfig};

        let mut cfg_builder = BrowserConfig::builder()
            .no_sandbox()
            .window_size(1920, 1080);

        // 禁用默认参数，使用自定义的 stealth 参数
        cfg_builder = cfg_builder.disable_default_args();

        // 添加 stealth 参数
        let stealth_args = [
            "--disable-blink-features=AutomationControlled",
            "--disable-dev-shm-usage",
            "--disable-gpu",
            "--no-first-run",
            "--disable-background-networking",
        ];
        for arg in &stealth_args {
            cfg_builder = cfg_builder.arg(*arg);
        }

        // 添加平台额外参数
        for arg in &config.extra_args {
            cfg_builder = cfg_builder.arg(arg.as_str());
        }

        // 代理
        if let Some(ref proxy) = self.proxy {
            cfg_builder = cfg_builder.arg(format!("--proxy-server={}", proxy));
        }

        let (browser, mut handler) = CBrowser::launch(
            cfg_builder.build().map_err(|e| format!("Browser config: {}", e))?
        )
        .await
        .map_err(|e| format!("Launch browser: {}", e))?;

        // 处理 CDP 事件
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    log::error!("[universal-browser] CDP handler error: {:?}", e);
                }
            }
        });

        // 启用 stealth 模式
        let page = browser.new_page("about:blank")
            .await
            .map_err(|e| format!("Create page: {}", e))?;
        page.enable_stealth_mode()
            .await
            .map_err(|e| format!("Enable stealth: {}", e))?;
        let _ = page.close().await;

        // 加载已有 cookies
        if let Some(ref path) = config.cookie_path {
            self.cookie_store = CookieStore::with_file(path.clone());
            let _ = self.cookie_store.load().await;
        }

        *self.browser.write().await = Some(browser);
        *self.config.write().await = Some(config);
        Ok(())
    }

    /// 创建新页面
    async fn new_page(&self) -> Result<chromiumoxide::Page, String> {
        let guard = self.browser.read().await;
        let browser = guard.as_ref().ok_or("Browser not launched")?;
        let page = browser.new_page("about:blank")
            .await
            .map_err(|e| format!("New page: {}", e))?;
        page.enable_stealth_mode()
            .await
            .map_err(|e| format!("Enable stealth: {}", e))?;
        Ok(page)
    }

    /// Cookie 登录 — 自动检测 + 注入
    pub async fn login_with_cookies(&self, credentials: &LoginCredentials) -> Result<BrowserResult, String> {
        let config = self.config.read().await;
        let config = config.as_ref().ok_or("Browser not configured")?;
        let login_url = config.login_url.clone();
        let success_url = config.success_url.clone();
        let start = SystemTime::now();

        let page = self.new_page().await?;

        // 注入已有的 cookies
        if let Some(ref auth_token) = credentials.auth_token {
            let cookie_js = format!(
                r#"document.cookie = "auth_token={}; path=/; domain=.x.com";"#,
                auth_token
            );
            page.evaluate(&*cookie_js).await.map_err(|e| format!("Inject cookie: {}", e))?;
        }
        if let Some(ref ct0) = credentials.ct0 {
            let cookie_js = format!(
                r#"document.cookie = "ct0={}; path=/; domain=.x.com";"#,
                ct0
            );
            page.evaluate(&*cookie_js).await.map_err(|e| format!("Inject cookie: {}", e))?;
        }

        // 导航到登录页
        page.goto(&login_url)
            .await
            .map_err(|e| format!("Navigate: {}", e))?;

        // 等待页面加载
        tokio::time::sleep(Duration::from_secs(3)).await;

        // 检测是否已登录
        let current_url = page.url().await.unwrap_or(None).unwrap_or_default();
        let is_logged_in = if let Some(ref success) = success_url {
            current_url.contains(success) || current_url != login_url
        } else {
            current_url != login_url
        };

        if is_logged_in {
            // 导出 cookies
            let cookies = page.get_cookies().await.unwrap_or_default();
            let domain = config.id.clone();
            self.cookie_store.import_chromium_cookies(&domain, cookies.clone()).await;
            let _ = self.cookie_store.save().await;

            let cookie_entries: Vec<CookieEntry> = cookies.into_iter().map(|c| {
                CookieEntry {
                    name: c.name,
                    value: c.value,
                    domain: c.domain,
                    path: c.path,
                    expires: if c.expires > 0.0 { Some(c.expires as u64) } else { None },
                    secure: c.secure,
                }
            }).collect();

            let _ = page.close().await;
            let duration = start.elapsed().unwrap_or_default().as_millis() as u64;

            Ok(BrowserResult {
                success: true,
                content: None,
                cookies: cookie_entries,
                error: None,
                duration_ms: duration,
            })
        } else {
            let _ = page.close().await;
            let duration = start.elapsed().unwrap_or_default().as_millis() as u64;

            Ok(BrowserResult {
                success: false,
                content: None,
                cookies: vec![],
                error: Some("Login failed - still on login page".into()),
                duration_ms: duration,
            })
        }
    }

    /// 手动登录 — 打开浏览器让用户操作
    pub async fn login_manual(&self) -> Result<BrowserResult, String> {
        let config = self.config.read().await;
        let config = config.as_ref().ok_or("Browser not configured")?;
        let login_url = config.login_url.clone();
        let success_url = config.success_url.clone();
        let timeout = config.timeout;
        let start = SystemTime::now();

        let page = self.new_page().await?;

        // 导航到登录页
        page.goto(&login_url)
            .await
            .map_err(|e| format!("Navigate: {}", e))?;

        // 等待用户登录 (轮询检测)
        let mut authenticated = false;
        let deadline = start + timeout;
        while SystemTime::now() < deadline {
            tokio::time::sleep(Duration::from_secs(2)).await;

            let cookies = page.get_cookies().await.unwrap_or_default();
            let has_auth = cookies.iter().any(|c| c.name == "auth_token" && !c.value.is_empty());
            if has_auth {
                authenticated = true;
                break;
            }

            // 也检查 URL 变化
            let current_url = page.url().await.unwrap_or(None).unwrap_or_default();
            if let Some(ref success) = success_url {
                if current_url.contains(success) {
                    authenticated = true;
                    break;
                }
            }
        }

        if authenticated {
            let cookies = page.get_cookies().await.unwrap_or_default();
            let domain = config.id.clone();
            self.cookie_store.import_chromium_cookies(&domain, cookies.clone()).await;
            let _ = self.cookie_store.save().await;

            let cookie_entries: Vec<CookieEntry> = cookies.into_iter().map(|c| {
                CookieEntry {
                    name: c.name,
                    value: c.value,
                    domain: c.domain,
                    path: c.path,
                    expires: if c.expires > 0.0 { Some(c.expires as u64) } else { None },
                    secure: c.secure,
                }
            }).collect();

            let _ = page.close().await;
            let duration = start.elapsed().unwrap_or_default().as_millis() as u64;

            Ok(BrowserResult {
                success: true,
                content: None,
                cookies: cookie_entries,
                error: None,
                duration_ms: duration,
            })
        } else {
            let _ = page.close().await;
            let duration = start.elapsed().unwrap_or_default().as_millis() as u64;

            Ok(BrowserResult {
                success: false,
                content: None,
                cookies: vec![],
                error: Some("Login timeout".into()),
                duration_ms: duration,
            })
        }
    }

    /// 抓取页面内容
    pub async fn fetch(&self, url: &str) -> Result<BrowserResult, String> {
        let start = SystemTime::now();
        let page = self.new_page().await?;

        page.goto(url)
            .await
            .map_err(|e| format!("Navigate: {}", e))?;

        tokio::time::sleep(Duration::from_secs(2)).await;

        let content = page.content()
            .await
            .map_err(|e| format!("Get content: {}", e))?;

        let cookies = page.get_cookies().await.unwrap_or_default();
        let _ = page.close().await;

        let duration = start.elapsed().unwrap_or_default().as_millis() as u64;

        Ok(BrowserResult {
            success: true,
            content: Some(content),
            cookies: cookies.into_iter().map(|c| CookieEntry {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
                expires: if c.expires > 0.0 { Some(c.expires as u64) } else { None },
                secure: c.secure,
            }).collect(),
            error: None,
            duration_ms: duration,
        })
    }

    /// 执行 JavaScript 并返回页面内容
    pub async fn eval(&self, url: &str, js: &str) -> Result<BrowserResult, String> {
        let start = SystemTime::now();
        let page = self.new_page().await?;

        page.goto(url)
            .await
            .map_err(|e| format!("Navigate: {}", e))?;

        tokio::time::sleep(Duration::from_secs(2)).await;

        // 执行 JS (结果通过页面内容获取)
        page.evaluate(js)
            .await
            .map_err(|e| format!("Eval JS: {}", e))?;

        // 等待 JS 执行
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 获取执行后的页面内容
        let content = page.content()
            .await
            .map_err(|e| format!("Get content: {}", e))?;

        let _ = page.close().await;
        let duration = start.elapsed().unwrap_or_default().as_millis() as u64;

        Ok(BrowserResult {
            success: true,
            content: Some(content),
            cookies: vec![],
            error: None,
            duration_ms: duration,
        })
    }

    /// 关闭浏览器
    pub async fn close(&self) {
        let mut guard = self.browser.write().await;
        if let Some(mut browser) = guard.take() {
            let _ = browser.close().await;
        }
    }

    /// 获取 cookie 存储引用
    pub fn cookie_store(&self) -> &CookieStore {
        &self.cookie_store
    }
}

impl Default for UniversalBrowser {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. 平台注册表 — 与 channel.rs 集成
// ═══════════════════════════════════════════════════════════════════════════

/// 平台注册表 — 管理所有平台的浏览器配置
pub struct PlatformRegistry {
    platforms: HashMap<String, PlatformConfig>,
}

impl PlatformRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            platforms: HashMap::new(),
        };

        // 预注册常用平台
        registry.register(PlatformConfig::twitter());
        registry.register(PlatformConfig::futong());

        registry
    }

    pub fn register(&mut self, config: PlatformConfig) {
        self.platforms.insert(config.id.clone(), config);
    }

    pub fn get(&self, id: &str) -> Option<&PlatformConfig> {
        self.platforms.get(id)
    }

    pub fn all_ids(&self) -> Vec<&str> {
        self.platforms.keys().map(|s| s.as_str()).collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. 测试
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_config_twitter() {
        let config = PlatformConfig::twitter();
        assert_eq!(config.id, "twitter");
        assert!(config.login_url.contains("x.com"));
    }

    #[test]
    fn test_platform_config_futong() {
        let config = PlatformConfig::futong();
        assert_eq!(config.id, "futong");
        assert!(config.login_url.contains("futong"));
    }

    #[test]
    fn test_platform_config_custom() {
        let config = PlatformConfig::custom("test", "https://example.com/login");
        assert_eq!(config.id, "test");
        assert_eq!(config.login_url, "https://example.com/login");
    }

    #[test]
    fn test_cookie_store_new() {
        let store = CookieStore::new();
        assert!(store.file_path.is_none());
    }

    #[test]
    fn test_platform_registry() {
        let registry = PlatformRegistry::new();
        assert!(registry.get("twitter").is_some());
        assert!(registry.get("futong").is_some());
        assert!(registry.get("nonexistent").is_none());
    }
}
