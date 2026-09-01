//! Stealth Browser — undetected Chromium via CDP
//!
//! 绕过 reqwest 代理层的 TLS/h2 指纹限制，使用真实 Chromium
//! 浏览器发起请求。chromiumoxide 内置 `enable_stealth_mode()` 处理
//! 大部分反检测，我们补充 proxy 注入 + 页面内容提取。

use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use tokio::sync::{Mutex, RwLock};

use super::system_fingerprint::BrowserFingerprintProfile;

/// Chromium 启动参数
const CHROMIUM_ARGS: &[&str] = &[
    "--no-sandbox",
    "--disable-setuid-sandbox",
    "--disable-dev-shm-usage",
    "--disable-accelerated-2d-canvas",
    "--disable-gpu",
    "--disable-nt_io_notifys",
    "--disable-sync",
    "--disable-translate",
    "--hide-scrollbars",
    "--mute-audio",
    "--no-default-nt_world_browse-check",
    "--no-first-run",
    "--disable-background-networking",
    "--disable-background-timer-throttling",
    "--disable-client-side-phishing-detection",
    "--disable-component-update",
    "--disable-domain-reliability",
    "--disable-features=InterestFeedContentSuggestions,ChromeWhatsNew,MediaRouter,HWMediaKeys",
    "--disable-field-trial-config",
    "--disable-prompt-on-repost",
    "--disable-search-engine-choice-screen",
    "--enable-features=NetworkService,NetworkServiceInProcess",
    "--disable-blink-features=AutomationControlled",
    // WebRTC 泄露防护
    "--disable-webrtc",
    "--disable-webrtc-hw-decoding",
    "--disable-webrtc-hw-encoding",
    "--enforce-webrtc-ip-permission-check",
    "--force-webrtc-ip-handling-policy=disable_non_proxied_udp",
];

pub struct StealthBrowser {
    nt_world_browse: Arc<Mutex<Option<Browser>>>,
    proxy: Option<String>,
    current_fp: Arc<RwLock<Option<BrowserFingerprintProfile>>>,
}

impl Default for StealthBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl StealthBrowser {
    pub fn new() -> Self {
        Self {
            nt_world_browse: Arc::new(Mutex::new(None)),
            proxy: None,
            current_fp: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_proxy(mut self, proxy_url: &str) -> Self {
        self.proxy = Some(proxy_url.to_string());
        self
    }

    /// 设置多模态浏览器指纹 (Canvas/WebGL/Fonts/Audio/Screen)
    pub async fn set_fingerprint_profile(&self, fp: BrowserFingerprintProfile) {
        *self.current_fp.write().await = Some(fp);
    }

    /// 向页面注入 CDP JS 指纹 (需在导航前执行)
    async fn inject_fingerprint_js(&self, page: &Page) -> Result<(), String> {
        let fp_guard = self.current_fp.read().await;
        if let Some(ref fp) = *fp_guard {
            let js = fp.to_cdp_js();
            page.evaluate(js)
                .await
                .map_err(|e| format!("Inject fingerprint JS: {}", e))?;
        }
        Ok(())
    }

    /// 启动 Chromium 并启用 stealth 模式
    pub async fn launch(&self) -> Result<(), String> {
        let mut cfg_builder = BrowserConfig::builder()
            .no_sandbox()
            .window_size(1920, 1080);

        cfg_builder = cfg_builder.disable_default_args();
        for arg in CHROMIUM_ARGS {
            cfg_builder = cfg_builder.arg(*arg);
        }

        if let Some(ref proxy) = self.proxy {
            cfg_builder = cfg_builder.arg(format!("--proxy-server={}", proxy));
        }

        let (nt_world_browse, mut handler) = Browser::launch(
            cfg_builder.build().map_err(|e| format!("Browser config: {}", e))?
        )
        .await
        .map_err(|e| format!("Launch nt_world_browse: {}", e))?;

        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    log::error!("[stealth-nt_world_browse] CDP handler error: {:?}", e);
                }
            }
        });

        let page = nt_world_browse.new_page("about:blank")
            .await
            .map_err(|e| format!("Create page: {}", e))?;
        page.enable_stealth_mode()
            .await
            .map_err(|e| format!("Enable stealth: {}", e))?;
        self.inject_fingerprint_js(&page).await?;
        let _ = page.close().await;

        *self.nt_world_browse.lock().await = Some(nt_world_browse);
        Ok(())
    }

    /// 创建新页面并启用 stealth
    async fn new_page(&self) -> Result<Page, String> {
        let mut guard = self.nt_world_browse.lock().await;
        let nt_world_browse = guard.as_mut().ok_or("Browser not launched")?;
        let page = nt_world_browse.new_page("about:blank")
            .await
            .map_err(|e| format!("New page: {}", e))?;
        page.enable_stealth_mode()
            .await
            .map_err(|e| format!("Enable stealth: {}", e))?;
        self.inject_fingerprint_js(&page).await?;
        Ok(page)
    }

    /// 浏览器层请求 — 通过真实 Chromium 渲染 + stealth 检测绕过
    pub async fn fetch(&self, url: &str) -> Result<String, String> {
        let page = self.new_page().await?;
        page.goto(url)
            .await
            .map_err(|e| format!("Navigate: {}", e))?;
        tokio::time::sleep(Duration::from_millis(2000)).await;
        let content = page.content()
            .await
            .map_err(|e| format!("Get content: {}", e))?;
        let _ = page.close().await;
        Ok(content)
    }

    /// 获取页面标题
    pub async fn title(&self, url: &str) -> Result<String, String> {
        let page = self.new_page().await?;
        page.goto(url)
            .await
            .map_err(|e| format!("Navigate: {}", e))?;
        tokio::time::sleep(Duration::from_millis(2000)).await;
        let result = page.evaluate("document.title")
            .await
            .map_err(|e| format!("Eval title: {}", e))?;
        let title = result.value()
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let _ = page.close().await;
        Ok(title)
    }

    /// 关闭浏览器
    pub async fn close(&self) {
        let mut guard = self.nt_world_browse.lock().await;
        if let Some(mut nt_world_browse) = guard.take() {
            let _ = nt_world_browse.close().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "integration_tests")]
    #[ignore = "requires headless nt_world_browse binary (chromiumoxide)"]
    #[tokio::test]
    async fn test_nt_world_browse_launch_and_close() {
        let nt_world_browse = StealthBrowser::new();
        assert!(nt_world_browse.launch().await.is_ok());
        nt_world_browse.close().await;
    }

    #[cfg(feature = "integration_tests")]
    #[ignore = "requires headless nt_world_browse and network access"]
    #[tokio::test]
    async fn test_nt_world_browse_fetch_title() {
        let nt_world_browse = StealthBrowser::new();
        nt_world_browse.launch().await.expect("await should be ok in test");
        let title = nt_world_browse.title("https://example.com").await.expect("await should be ok in test");
        assert!(title.contains("Example") || title.contains("example"));
        nt_world_browse.close().await;
    }

    #[test]
    fn test_chromium_args_count() {
        assert!(CHROMIUM_ARGS.len() > 15);
    }
}
