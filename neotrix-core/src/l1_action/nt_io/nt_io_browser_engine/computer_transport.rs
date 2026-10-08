//! `computer_transport` — **D 期 core 半边**：把 core 已有的 chromiumoxide CDP
//! 能力接成 `neotrix_neobot::nt_computer::CdpTransport`。
//!
//! # 为什么在 core 而不是 neobot
//!
//! 依赖方向是 **core → neobot**（core 依赖 neobot）。所以「CDP 怎么连」只能
//! 由 core 实现 neobot 定义的口；反过来在 neobot 里写 CDP 客户端会造环，
//! 也会长出第二套浏览器栈（core 已经有 `BrowserEngine::cdp_browser()`：
//! 附着活体 Chrome `NT_BROWSE_CDP_URL` 或自行 launch，`stealth-net` 特性下）。
//!
//! # 同步/异步的桥
//!
//! neobot 的口是**同步**（工具网关在同步线程里调），chromiumoxide 是 async。
//! ⇒ 这里持有一个**专用 runtime**，在 `evaluate` 里 `block_on`。
//! ⛔ 明确不做的事：阻塞 tokio 运行时里的线程（那会 panic/死锁）。因此本类型
//! **只能在非 async 上下文使用**（工具网关正是这种上下文）；若未来要在 async
//! 里用，必须换成 `async fn` 版本的口，而不是在这里偷偷 `block_on`。

use std::sync::Arc;

use neotrix_neobot::nt_computer::CdpTransport;
use neotrix_neobot::nt_error::NtBotError;

/// core 侧 CDP transport：复用 `BrowserEngine` 的 CDP 单例 + 一个常驻页面。
pub struct CoreCdpTransport {
    runtime: tokio::runtime::Runtime,
    engine: Arc<crate::l1_action::nt_io::nt_io_browser_engine::engine::BrowserEngine>,
    page: Arc<tokio::sync::Mutex<Option<chromiumoxide::Page>>>,
}

impl CoreCdpTransport {
    /// 构造：建 runtime、取（或建）一个页面。
    ///
    /// 失败一律如实返回 `NtBotError::Io`/`Invalid`（**不**返回假可用对象）。
    pub fn new() -> Result<Self, NtBotError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| NtBotError::Io(format!("cdp runtime: {e}")))?;
        Ok(Self {
            runtime,
            engine: Arc::new(
                crate::l1_action::nt_io::nt_io_browser_engine::engine::BrowserEngine::new(
                    crate::l1_action::nt_io::nt_io_browser_engine::session::BrowserConfig::default(),
                ),
            ),
            page: Arc::new(tokio::sync::Mutex::new(None)),
        })
    }

    async fn page_slot(&self) -> Result<chromiumoxide::Page, NtBotError> {
        let mut guard = self.page.lock().await;
        if let Some(page) = guard.as_ref() {
            return Ok(page.clone());
        }
        let session = self.engine
            .create_session()
            .await
            .map_err(|e| NtBotError::Io(format!("cdp session: {e:?}")))?;
        let page = self.engine
            .cdp_page(&session)
            .await
            .map_err(|e| NtBotError::Io(format!("cdp page: {e:?}")))?;
        *guard = Some(page.clone());
        Ok(page)
    }
}

impl CdpTransport for CoreCdpTransport {
    fn evaluate(&self, script: &str) -> Result<String, NtBotError> {
        let script = script.to_owned();
        self.runtime.block_on(async move {
            let page = self.page_slot().await?;
            let value = page
                .evaluate(script.clone())
                .await
                .map_err(|e| NtBotError::Io(format!("cdp evaluate: {e}")))?;
            // `into_value()` 返 `Result<Value, _>`：非 JSON/不可解析 ⇒ 如实报不可读，
            // **不**退回成字符串猜测（那会把 undefined 变成 "None" 之类的假事实）。
            let json: serde_json::Value = value
                .into_value()
                .map_err(|e| NtBotError::Io(format!("cdp evaluate: value not json: {e}")))?;
            json.as_str()
                .map(str::to_owned)
                .ok_or_else(|| {
                    NtBotError::Io("cdp evaluate: 非字符串结果（如数值/对象）".to_owned())
                })
        })
    }

    fn current_url(&self) -> Option<String> {
        self.runtime.block_on(async move {
            let Ok(page) = self.page_slot().await else {
                return None;
            };
            // 本版本 chromiumoxide 的 `Page::url()` 直接给 `Option<String>`：
            // 取不到就是 `None`（**不**编一个 about:blank 假装有页）。
            page.url().await.ok().flatten()
        })
    }
}

/// 工厂：宿主（CLI/desktop）用它拿到一个**真的接了线**的 computer backend。
///
/// ⛔ 没有 chrome / 没开 `stealth-net` ⇒ 如实失败；调用方**不许**因此退化到
/// `NoopBackend` 却对外宣称「浏览器可控」。
#[cfg(feature = "stealth-net")]
pub fn computer_backend() -> Result<neotrix_neobot::nt_computer::CdpBackend<CoreCdpTransport>, NtBotError> {
    Ok(neotrix_neobot::nt_computer::CdpBackend::new(CoreCdpTransport::new()?))
}
