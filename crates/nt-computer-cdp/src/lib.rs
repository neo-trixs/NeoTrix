//! `nt-computer-cdp` — `neotrix-neobot::nt_computer::CdpTransport` 的 chromiumoxide 实现。
//!
//! # 为什么单开一个 crate
//!
//! core（neotrix-core）依赖 neobot，而 neobot 的 `CdpTransport` 是 pub trait。
//! 若直接把实现放在 core，用户要注入 CDP 后端就必须把整个 core 拉进来——
//! 悬置体在桌面便携端是不本质的、全链化。抽成微 crate 后：
//!
//! -core**→**nt-computer-cdp（只依赖 chromiumoxide + tokio + neotrix-neobot 的 trait）；
//! - neobot 仍保持**无 chromiumoxide** 依赖；
//! - CLI / desktop / 任何消费方可选：不连不引入，不再需要 core 全包。
//!
//! # 实现口径
//!
//! - `CoreCdpTransport::new()` 内部 持有专用 current_thread runtime 与 page。
//! - `evaluate` 同步桥接 via `runtime.block_on`。
//! ⛔ 如需 async 宿主，请用 async server 直接驱动；不要在 tokio runtime 内部再次
//!    `block_on`（会死锁/panic）。当前工具网关是同步上下文，符合预期。

use std::sync::Arc;

use chromiumoxide::{Browser, BrowserConfig as CdpConfig, Page};
use futures::StreamExt;
use neotrix_neobot::nt_computer::CdpTransport;
use neotrix_neobot::nt_error::NtBotError;

fn chrome_path() -> String {
    if cfg!(target_os = "macos") {
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string()
    } else if cfg!(target_os = "windows") {
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string()
    } else {
        "google-chrome".to_string()
    }
}

/// CDP transport over chromiumoxide。
pub struct CoreCdpTransport {
    runtime: tokio::runtime::Runtime,
    browser: Arc<Browser>,
    page: Arc<tokio::sync::Mutex<Option<Page>>>,
}

impl CoreCdpTransport {
    /// 启动浏览器并进入 blank 页面；失败返回 `NtBotError::Io`。
    pub async fn new() -> Result<Self, NtBotError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| NtBotError::Io(format!("cdp runtime: {e}")))?;

        let mut builder = CdpConfig::builder()
            .chrome_executable(chrome_path())
            .no_sandbox()
            .launch_timeout(std::time::Duration::from_secs(60))
            .disable_default_args();
        let cfg = builder
            .build()
            .map_err(|e| NtBotError::Io(format!("cdp config: {e}")))?;
        let (browser, mut handler) = Browser::launch(cfg)
            .await
            .map_err(|e| NtBotError::Io(format!("cdp launch: {e}")))?;

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| NtBotError::Io(format!("cdp page: {e}")))?;

        let page = Arc::new(tokio::sync::Mutex::new(Some(page)));
        let browser = Arc::new(browser);
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    eprintln!("[nt-computer-cdp] handler: {e:?}");
                }
            }
        });

        Ok(Self { runtime, browser, page })
    }

    async fn page_slot(&self) -> Result<Page, NtBotError> {
        let mut guard = self.page.lock().await;
        match guard.as_ref() {
            Some(p) => Ok(p.clone()),
            None => Err(NtBotError::Io("cdp: page closed".to_owned())),
        }
    }
}

impl CdpTransport for CoreCdpTransport {
    fn evaluate(&self, script: &str) -> Result<String, NtBotError> {
        let script = script.to_owned();
        let page = self.runtime.block_on(async move { self.page_slot().await })?;
        let value = self
            .runtime
            .block_on(async move { page.evaluate(script.clone()).await })
            .map_err(|e| NtBotError::Io(format!("cdp evaluate: {e}")))?;
        let json: serde_json::Value = value
            .into_value()
            .map_err(|e| NtBotError::Io(format!("cdp evaluate: value not json: {e}")))?;
        json.as_str()
            .map(str::to_owned)
            .ok_or_else(|| NtBotError::Io("cdp evaluate: 非字符串结果".to_owned()))
    }

    fn current_url(&self) -> Option<String> {
        let page = self.runtime.block_on(async move { self.page_slot().await }).ok()?;
        self.runtime.block_on(async move { page.url().await }).ok().flatten()
    }
}

/// 工厂：消费方拿到真接线的 computer backend。
///
/// ⛔ 未带 chrome 二进制 ⇒ 如实报错；**禁止**包装成「假可用」。
pub async fn computer_backend() -> Result<neotrix_neobot::nt_computer::CdpBackend<CoreCdpTransport>, NtBotError> {
    Ok(neotrix_neobot::nt_computer::CdpBackend::new(
        CoreCdpTransport::new().await?,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn transport_type_checks() {
        fn is_impl<T: neotrix_neobot::nt_computer::CdpTransport>() {}
        is_impl::<super::CoreCdpTransport>();
    }
}
