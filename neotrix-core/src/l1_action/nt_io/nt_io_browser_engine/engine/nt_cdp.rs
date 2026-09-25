//! nt_cdp — Chrome headless 与 CDP 协议后端：渲染 / 真操控 / 页面管理.
//! 从 `nt_io_browser_engine/engine.rs` 纯搬移, 行为零变更.

use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "stealth-net")]
use chromiumoxide::{Browser, BrowserConfig as CdpConfig, Page};
#[cfg(feature = "stealth-net")]
use futures::StreamExt;

use super::BrowserEngine;
use super::super::error::BrowserError;
use super::super::fetch::{parse_page, render_snapshot_text};
use super::super::js::{js_select_option, js_set_checked, pick_form};
use super::super::types::{BrowserAction, BrowserResult, PageSnapshot, ScrollDirection};

// ============================================================================
// 后端：Chrome headless（真渲染，无 profile）
// ============================================================================

pub(crate) fn chrome_path() -> String {
    if cfg!(target_os = "macos") {
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string()
    } else if cfg!(target_os = "windows") {
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string()
    } else {
        "google-chrome".to_string()
    }
}

/// headless 渲染（禁用 --user-data-dir：见模块头 R-P38 注释）
pub(crate) fn chrome_render(
    url: &str,
    budget_ms: u64,
    screenshot_path: Option<&std::path::Path>,
    width: u32,
    height: u32,
) -> Result<String, BrowserError> {
    let mut cmd = std::process::Command::new(chrome_path());
    cmd.arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--no-first-run")
        .arg("--disable-background-networking")
        .arg("--disable-sync")
        .arg("--mute-audio")
        .arg("--disable-features=ChromeWhatsNewUI")
        .arg("--disable-component-update")
        .arg("--disable-client-side-phishing-detection")
        // 2026-09-22：mock 钥匙串，headless 永不弹系统密码框（见模块头）
        .arg("--use-mock-keychain")
        .arg("--timeout=30000")
        .arg(format!("--virtual-time-budget={budget_ms}"))
        .arg("--dump-dom");
    if let Some(path) = screenshot_path {
        cmd.arg(format!("--screenshot={}", path.display()));
        cmd.arg(format!("--window-size={width},{height}"));
    }
    cmd.arg(url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut child = cmd
        .spawn()
        .map_err(|e| BrowserError::BackendUnavailable(format!("chrome spawn: {e}")))?;
    let deadline =
        std::time::Instant::now() + Duration::from_millis(budget_ms + 3000);
    loop {
        match child
            .try_wait()
            .map_err(|e| BrowserError::ActionFailed(format!("chrome wait: {e}")))?
        {
            Some(status) => {
                let mut stdout = String::new();
                use std::io::Read;
                let _ = child
                    .stdout
                    .take()
                    .and_then(|mut o| o.read_to_string(&mut stdout).ok());
                if !status.success() {
                    return Err(BrowserError::ActionFailed(format!(
                        "chrome exit {}",
                        status.code().unwrap_or(-1)
                    )));
                }
                if stdout.len() < 80 {
                    return Err(BrowserError::ActionFailed("empty page".to_string()));
                }
                return Ok(stdout);
            }
            None => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(BrowserError::Timeout);
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

/// headless 打印 PDF（禁用 --user-data-dir：同 R-P38 注释）
pub(crate) fn chrome_render_pdf(url: &str, dest: &std::path::Path, budget_ms: u64) -> Result<(), BrowserError> {
    let mut child = std::process::Command::new(chrome_path())
        .arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--no-first-run")
        .arg("--mute-audio")
        // 2026-09-22：mock 钥匙串，headless 永不弹系统密码框（见模块头）
        .arg("--use-mock-keychain")
        .arg("--no-pdf-header-footer")
        .arg(format!("--print-to-pdf={}", dest.display()))
        .arg(format!("--virtual-time-budget={budget_ms}"))
        .arg("--timeout=30000")
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| BrowserError::BackendUnavailable(format!("chrome spawn: {e}")))?;
    let deadline =
        std::time::Instant::now() + Duration::from_millis(budget_ms + 3000);
    loop {
        match child
            .try_wait()
            .map_err(|e| BrowserError::ActionFailed(format!("chrome wait: {e}")))?
        {
            Some(status) => {
                if !status.success() {
                    return Err(BrowserError::ActionFailed(format!(
                        "chrome exit {}",
                        status.code().unwrap_or(-1)
                    )));
                }
                if !dest.exists() {
                    return Err(BrowserError::ActionFailed(
                        "pdf not produced".to_string(),
                    ));
                }
                return Ok(());
            }
            None => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(BrowserError::Timeout);
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

impl BrowserEngine {
    // -- Chrome 后端 ------------------------------------------------------------

    pub(crate) async fn dispatch_chrome(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        match action {
            BrowserAction::Navigate { url } => {
                self.polite_wait(url).await?;
                let budget = self.config.timeout_ms.min(20_000);
                let url_owned = url.clone();
                let joined = tokio::task::spawn_blocking(move || {
                    chrome_render(&url_owned, budget, None, 1280, 720)
                })
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("render task: {e}")))?;
                let html = joined?;
                let (title, text, links, forms, raw_html) = parse_page(url, &html)?;
                let snap = PageSnapshot {
                    url: url.clone(),
                    title: title.clone(),
                    text,
                    links,
                    forms,
                    raw_html,
                    http_status: None,
                };
                let out = render_snapshot_text(&snap);
                self.commit_snapshot(session_id, snap, true).await?;
                self.record_fetch(url).await;
                Ok(BrowserResult::ok(out, url.clone(), title, 0))
            }
            BrowserAction::Screenshot => {
                let (url, title) = self.session_url_title(session_id).await?;
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.png",
                    uuid::Uuid::new_v4()
                ));
                let target = if url == "about:blank" {
                    "about:blank".to_string()
                } else {
                    url.clone()
                };
                let budget = self.config.timeout_ms.min(20_000);
                let (w, h) = (self.config.viewport_width, self.config.viewport_height);
                let dest = path.clone();
                let joined = tokio::task::spawn_blocking(move || {
                    chrome_render(&target, budget, Some(&dest), w, h)
                })
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("screenshot task: {e}")))?;
                joined?;
                Ok(BrowserResult::ok(
                    format!("screenshot: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            // 渲染页的链接/表单沿用 HTTP 语义回放（Cookie 不互通，见模块头）
            BrowserAction::Click { .. } => {
                self.dispatch_http(session_id, action).await.map(|mut r| {
                    r.output = format!("[chrome-rendered] {}", r.output);
                    r
                })
            }
            BrowserAction::GetContent => {
                self.dispatch_http(session_id, action).await
            }
            BrowserAction::Type { selector, text } => {
                self.record_fill(session_id, selector.clone(), text.clone())
                    .await
            }
            BrowserAction::FillField { selector, value } => {
                self.record_fill(session_id, selector.clone(), value.clone())
                    .await
            }
            BrowserAction::SelectOption { .. }
            | BrowserAction::Check { .. }
            | BrowserAction::Uncheck { .. }
            | BrowserAction::Press { .. }
            | BrowserAction::GetText { .. } => {
                self.dispatch_http(session_id, action).await.map(|mut r| {
                    r.output = format!("[chrome-rendered] {}", r.output);
                    r
                })
            }
            BrowserAction::PrintPdf => {
                let (url, title) = self.session_url_title(session_id).await?;
                if url == "about:blank" {
                    return Err(BrowserError::ActionFailed(
                        "no page loaded; Navigate first".to_string(),
                    ));
                }
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.pdf",
                    uuid::Uuid::new_v4()
                ));
                let url_owned = url.clone();
                let dest = path.clone();
                let budget = self.config.timeout_ms.min(20_000);
                tokio::task::spawn_blocking(move || {
                    chrome_render_pdf(&url_owned, &dest, budget)
                })
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("pdf task: {e}")))?
                ?;
                Ok(BrowserResult::ok(
                    format!("pdf: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::SubmitForm { selector } => {
                self.http_submit_form(session_id, selector.clone()).await
            }
            BrowserAction::ExecuteJs { .. } => Err(BrowserError::ActionFailed(
                "ExecuteJs 需要 CDP 通道（在途）；dump-dom 管道不支持".to_string(),
            )),
            other => self.dispatch_http(session_id, other).await,
        }
    }

    // -- CDP 后端（真操控，需 stealth-net 特性） -------------------------------

    /// CDP 分发（无特性时诚实报错）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn dispatch_cdp(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        match action {
            BrowserAction::Navigate { url } => self.cdp_navigate(session_id, url).await,
            BrowserAction::GetContent => {
                let page = self.cdp_page(session_id).await?;
                let html = page
                    .content()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp content: {e}")))?;
                let url = page.url().await.ok().flatten().unwrap_or_default();
                let base = if url.is_empty() {
                    self.session_url(session_id).await?
                } else {
                    url
                };
                let (title, text, links, forms, raw_html) = parse_page(&base, &html)?;
                let snap = PageSnapshot {
                    url: base.clone(),
                    title: title.clone(),
                    text,
                    links,
                    forms,
                    raw_html,
            http_status: None,
                };
                let out = render_snapshot_text(&snap);
                self.commit_snapshot(session_id, snap, true).await?;
                self.record_fetch(&base).await;
                Ok(BrowserResult::ok(out, base, title, 0))
            }
            BrowserAction::Click { selector } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp click '{selector}': {e}"))
                })?;
                el.click()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp click: {e}")))?;
                tokio::time::sleep(Duration::from_millis(800)).await;
                return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
            }
            BrowserAction::Type { selector, text } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp type '{selector}': {e}"))
                })?;
                el.click()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp focus: {e}")))?;
                el.type_str(text.clone())
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp type: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("typed into '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::FillField { selector, value } => {
                return Box::pin(self
                    .dispatch_cdp(
                        session_id,
                        &BrowserAction::Type {
                            selector: selector.clone(),
                            text: value.clone(),
                        },
                    ))
                    .await;
            }
            BrowserAction::SubmitForm { selector } => {
                let (_, snap_opt) = self.session_snapshot(session_id).await?;
                let snap = snap_opt.ok_or_else(|| {
                    BrowserError::ActionFailed(
                        "no page loaded; Navigate first".to_string(),
                    )
                })?;
                // 用快照表单序号定位 document.forms[idx] 真提交
                let idx = match selector.as_deref() {
                    None => 0,
                    Some(s) if s.starts_with('#') => {
                        s.strip_prefix('#').unwrap_or("").parse().map_err(|_| {
                            BrowserError::ActionFailed(format!("bad form selector '{s}'"))
                        })?
                    }
                    Some(_) => 0,
                };
                if pick_form(&snap, Some(&format!("#{idx}"))).is_err() {
                    return Err(BrowserError::ActionFailed(format!("no form #{idx}")));
                }
                let page = self.cdp_page(session_id).await?;
                page.evaluate(format!(
                    "(()=>{{const f=document.forms[{idx}];if(!f)return 'no-form';f.submit();return 'ok';}})()"
                ))
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("cdp submit: {e}")))?;
                tokio::time::sleep(Duration::from_millis(800)).await;
                return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
            }
            BrowserAction::SelectOption { selector, values } => {
                let page = self.cdp_page(session_id).await?;
                let out = page
                    .evaluate(js_select_option(selector, values))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp select: {e}")))?;
                let text = out
                    .value()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .unwrap_or_else(|| "undefined".to_string());
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(text, url, title, 0))
            }
            BrowserAction::Check { selector } => {
                let page = self.cdp_page(session_id).await?;
                page.evaluate(js_set_checked(selector, true))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp check: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("checked '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Uncheck { selector } => {
                let page = self.cdp_page(session_id).await?;
                page.evaluate(js_set_checked(selector, false))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp uncheck: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("unchecked '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Press { selector, key } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp press '{selector}': {e}"))
                })?;
                el.press_key(key.clone())
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp press: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("pressed '{key}' on '{selector}'"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::GetText { selector } => {
                let page = self.cdp_page(session_id).await?;
                let el = page.find_element(selector.clone()).await.map_err(|e| {
                    BrowserError::ActionFailed(format!("cdp text '{selector}': {e}"))
                })?;
                let text = el
                    .inner_text()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp text: {e}")))?
                    .unwrap_or_default();
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(text, url, title, 0))
            }
            BrowserAction::PrintPdf => {
                let page = self.cdp_page(session_id).await?;
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.pdf",
                    uuid::Uuid::new_v4()
                ));
                page.save_pdf(Default::default(), &path)
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp pdf: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("pdf: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::ExecuteJs { script } => {
                let page = self.cdp_page(session_id).await?;
                let result = page
                    .evaluate(script.clone())
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp eval: {e}")))?;
                let out = result
                    .value()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .unwrap_or_else(|| "undefined".to_string());
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(out, url, title, 0))
            }
            BrowserAction::Screenshot => {
                let page = self.cdp_page(session_id).await?;
                let path = std::env::temp_dir().join(format!(
                    "nt_browser_{}.png",
                    uuid::Uuid::new_v4()
                ));
                page.save_screenshot(
                    chromiumoxide::page::ScreenshotParams::builder()
                        .full_page(true)
                        .build(),
                    &path,
                )
                .await
                .map_err(|e| BrowserError::ActionFailed(format!("cdp shot: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("screenshot: {}", path.display()),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Scroll { direction, amount } => {
                let (dx, dy) = match direction {
                    ScrollDirection::Up => (0, -(amount.unwrap_or(300.0) as i64)),
                    ScrollDirection::Down => (0, amount.unwrap_or(300.0) as i64),
                    ScrollDirection::Left => (-(amount.unwrap_or(300.0) as i64), 0),
                    ScrollDirection::Right => (amount.unwrap_or(300.0) as i64, 0),
                };
                let page = self.cdp_page(session_id).await?;
                page.evaluate(format!("window.scrollBy({dx},{dy})"))
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp scroll: {e}")))?;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("scrolled {direction:?}"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Sleep { ms } => {
                // 同 http 臂：单动作上限约 65s，钳 60s，超长请链多个 Sleep。
                let wait = (*ms).min(60_000);
                tokio::time::sleep(Duration::from_millis(wait)).await;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(format!("slept {wait}ms"), url, title, 0))
            }
            BrowserAction::WaitForElement {
                selector,
                timeout_ms,
            } => {
                let page = self.cdp_page(session_id).await?;
                let deadline = std::time::Instant::now()
                    + Duration::from_millis((*timeout_ms).min(120_000));
                loop {
                    if page
                        .find_element(selector.clone())
                        .await
                        .is_ok()
                    {
                        let (url, title) = self.session_url_title(session_id).await?;
                        return Ok(BrowserResult::ok(
                            format!("element '{selector}' present"),
                            url,
                            title,
                            0,
                        ));
                    }
                    if std::time::Instant::now() >= deadline {
                        let (url, title) = self.session_url_title(session_id).await?;
                        return Ok(BrowserResult::fail(
                            format!("wait timed out: '{selector}'"),
                            url,
                            title,
                            *timeout_ms,
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
            BrowserAction::Reload => {
                let page = self.cdp_page(session_id).await?;
                page.reload()
                    .await
                    .map_err(|e| BrowserError::ActionFailed(format!("cdp reload: {e}")))?;
                return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
            }
            BrowserAction::GoBack => {
                return self.cdp_history(session_id, "back").await;
            }
            BrowserAction::GoForward => {
                return self.cdp_history(session_id, "forward").await;
            }
            BrowserAction::NewTab { .. }
            | BrowserAction::SwitchTab { .. }
            | BrowserAction::CloseTab => {
                Err(BrowserError::ActionFailed(
                    "tab ops 由引擎层预拦截，不应到达 CDP 分发".to_string(),
                ))
            }
            // 状态与传输由引擎层预拦截，不应到达后端分发
            BrowserAction::SaveState { .. }
            | BrowserAction::LoadState { .. }
            | BrowserAction::Download { .. }
            | BrowserAction::Upload { .. } => Err(BrowserError::ActionFailed(
                "unreachable: handled by engine dispatch".to_string(),
            )),
        }
    }

    #[cfg(not(feature = "stealth-net"))]
    pub(crate) async fn dispatch_cdp(
        &self,
        _session_id: &str,
        _action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        Err(BrowserError::BackendUnavailable(
            "Cdp 后端需要 stealth-net 特性".to_string(),
        ))
    }

    /// CDP 导航（含 Cookie 回灌到自研 jar）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_navigate(
        &self,
        session_id: &str,
        url: &str,
    ) -> Result<BrowserResult, BrowserError> {
        self.polite_wait(url).await?;
        let page = self.cdp_page(session_id).await?;
        page.goto(url.to_string())
            .await
            .map_err(|e| BrowserError::NavigationFailed(format!("cdp goto {url}: {e}")))?;
        let html = page
            .content()
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp content: {e}")))?;
        let current = page.url().await.ok().flatten().unwrap_or_default();
        let base = if current.is_empty() {
            url.to_string()
        } else {
            current
        };
        // Cookie 双向同步：CDP → 自研 jar
        if let Ok(cookies) = page.get_cookies().await {
            let packed: Vec<String> = cookies
                .iter()
                .map(|c| {
                    format!(
                        "{}\u{1f}{}={}",
                        c.domain.trim_start_matches('.'),
                        c.name,
                        c.value
                    )
                })
                .collect();
            self.apply_cookies(session_id, packed).await;
        }
        let (title, text, links, forms, raw_html) = parse_page(&base, &html)?;
        let snap = PageSnapshot {
            url: base.clone(),
            title: title.clone(),
            text,
            links,
            forms,
            raw_html,
            http_status: None,
        };
        let out = render_snapshot_text(&snap);
        self.commit_snapshot(session_id, snap, true).await?;
        self.record_fetch(&base).await;
        Ok(BrowserResult::ok(out, base, title, 0))
    }

    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_history(
        &self,
        session_id: &str,
        dir: &str,
    ) -> Result<BrowserResult, BrowserError> {
        let page = self.cdp_page(session_id).await?;
        page.evaluate(format!("history.{dir}()"))
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp history: {e}")))?;
        tokio::time::sleep(Duration::from_millis(800)).await;
        return Box::pin(self.dispatch_cdp(session_id, &BrowserAction::GetContent)).await;
    }

    /// CDP 浏览器单例（懒启动 + handler 泵 + 会话 UA 注入 stealth）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_browser(&self) -> Result<Arc<Browser>, BrowserError> {
        {
            let guard = self.cdp_browser.lock().await;
            if let Some(browser) = guard.as_ref() {
                return Ok(browser.clone());
            }
        }
        // B 方案：附着用户活体 Chrome（对方以 --remote-debugging-port 启动，登录态完整保持）。
        // NT_BROWSE_CDP_URL=http://127.0.0.1:9333（http 形自动取 /json/version 换 ws）。
        // profile 副本带不过钥匙串登录态，launch 路线只做未登录页；要登录态必须走本分支。
        if let Some(endpoint) = std::env::var("NT_BROWSE_CDP_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
        {
            let endpoint = endpoint.trim().to_string();
            let (browser, mut handler) = Browser::connect(endpoint.clone()).await.map_err(|e| {
                BrowserError::BackendUnavailable(format!("cdp connect {endpoint}: {e}"))
            })?;
            tokio::spawn(async move {
                while let Some(event) = handler.next().await {
                    if let Err(e) = event {
                        log::warn!("[nt_browser] cdp handler: {e:?}");
                    }
                }
            });
            let browser = Arc::new(browser);
            *self.cdp_browser.lock().await = Some(browser.clone());
            log::info!("[nt_browser] cdp attached to {endpoint}");
            return Ok(browser);
        }
        let mut builder = CdpConfig::builder()
            .chrome_executable(chrome_path())
            .no_sandbox()
            .window_size(self.config.viewport_width, self.config.viewport_height)
            .request_timeout(self.op_timeout())
            .launch_timeout(Duration::from_secs(60))
            .disable_default_args();
        for arg in [
            "--disable-dev-shm-usage",
            "--disable-blink-features=AutomationControlled",
            "--no-first-run",
            "--disable-background-networking",
            "--disable-sync",
            "--mute-audio",
            "--disable-component-update",
            "--disable-client-side-phishing-detection",
            // 2026-09-22：mock 钥匙串，CDP 浏览器永不弹系统密码框
            "--use-mock-keychain",
        ] {
            builder = builder.arg(arg);
        }
        if let Some(proxy) = self.config.proxy.as_deref() {
            let proxy = proxy.trim();
            if !proxy.is_empty() {
                builder = builder.arg(format!("--proxy-server={proxy}"));
            }
        }
        // profile 复用：继承已登录会话（须先退出占用的 Chrome，见字段注释）
        if let Some(profile) = self.config.profile_dir.as_deref() {
            let profile = profile.trim();
            if !profile.is_empty() {
                builder = builder.user_data_dir(profile);
            }
        }
        let cfg = builder
            .build()
            .map_err(|e| BrowserError::BackendUnavailable(format!("cdp config: {e}")))?;
        let (browser, mut handler) = Browser::launch(cfg)
            .await
            .map_err(|e| BrowserError::BackendUnavailable(format!("cdp launch: {e}")))?;
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    log::warn!("[nt_browser] cdp handler: {e:?}");
                }
            }
        });
        let browser = Arc::new(browser);
        *self.cdp_browser.lock().await = Some(browser.clone());
        Ok(browser)
    }

    /// 会话 CDP 页面（懒创建 + stealth + 会话 UA）
    #[cfg(feature = "stealth-net")]
    pub(crate) async fn cdp_page(&self, session_id: &str) -> Result<Page, BrowserError> {
        {
            let sessions = self.sessions.read().await;
            sessions
                .get(session_id)
                .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
        }
        {
            let pages = self.cdp_pages.read().await;
            if let Some(page) = pages.get(session_id) {
                return Ok(page.clone());
            }
        }
        let browser = self.cdp_browser().await?;
        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp new_page: {e}")))?;
        let ua = self
            .sessions
            .read()
            .await
            .get(session_id)
            .map(|s| s.user_agent.clone())
            .unwrap_or_default();
        page.enable_stealth_mode_with_agent(&ua)
            .await
            .map_err(|e| BrowserError::ActionFailed(format!("cdp stealth: {e}")))?;
        self.cdp_pages
            .write()
            .await
            .insert(session_id.to_string(), page.clone());
        Ok(page)
    }

    /// 关闭 CDP 浏览器单例（会话级页面一并丢弃）
    #[cfg(feature = "stealth-net")]
    pub async fn shutdown_cdp(&self) {
        self.cdp_pages.write().await.clear();
        *self.cdp_browser.lock().await = None;
    }
}
