//! nt_http — HTTP 请求/响应后端：快照抓取 / 导航 / 表单提交 / 等待.
//! 从 `nt_io_browser_engine/engine.rs` 纯搬移, 行为零变更.

use std::time::Duration;

use scraper::{Html, Selector};

use super::BrowserEngine;
use super::super::cookies::CookieJar;
use super::super::error::BrowserError;
use super::super::fetch::{
    http_get, http_submit, multipart_submit, parse_page, render_snapshot_text,
};
use super::super::js::pick_form;
use super::super::types::{BrowserAction, BrowserResult, PageSnapshot, verify_for};

impl BrowserEngine {
    pub(crate) fn blocking_timeout(&self) -> Duration {
        self.op_timeout()
    }

    // -- HTTP 后端 --------------------------------------------------------------

    pub(crate) async fn http_fetch_snapshot(
        &self,
        url: &str,
        jar: &CookieJar,
        ua: &str,
        auth: Option<(String, String)>,
    ) -> Result<(PageSnapshot, Vec<String>), BrowserError> {
        self.polite_wait(url).await?;
        let timeout = self.blocking_timeout();
        let fetched = match http_get(&self.client, url, jar, ua, auth, timeout).await {
            Err(BrowserError::RateLimited(secs)) => {
                self.record_rate_limited(url, secs).await;
                return Err(BrowserError::RateLimited(secs));
            }
            other => other?,
        };
        let applied_host = url::Url::parse(&fetched.final_url)
            .map(|u| u.host_str().unwrap_or("").to_string())
            .unwrap_or_default();
        let (title, text, links, forms, raw_html) =
            parse_page(&fetched.final_url, &fetched.body)?;
        Ok((
            PageSnapshot {
                url: fetched.final_url,
                title,
                text,
                links,
                forms,
                raw_html,
                http_status: Some(fetched.status),
            },
            // 返回 set-cookie 由调用方存入会话 jar（附带响应 host）
            fetched
                .set_cookies
                .into_iter()
                .map(|c| format!("{applied_host}\u{1f}{c}"))
                .collect(),
        ))
    }

    pub(crate) async fn apply_cookies(&self, session_id: &str, packed: Vec<String>) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            for item in packed {
                if let Some((host, raw)) = item.split_once('\u{1f}') {
                    session
                        .jar
                        .store_from_headers(host, std::iter::once(raw));
                }
            }
            // 同步扁平视图（兼容旧字段）
            session.cookies.clear();
            for (domain, bucket) in session
                .jar
                .entries
                .iter()
            {
                for c in bucket {
                    session
                        .cookies
                        .insert(format!("{}:{}", domain, c.name), c.value.clone());
                }
            }
        }
    }

    /// 401/403 自愈：热加载 token → 重试一次 → 仍败 → AuthExpired（带精确指引）
    pub(crate) async fn authed_navigate(
        &self,
        session_id: &str,
        url: &str,
    ) -> Result<(PageSnapshot, Vec<String>), BrowserError> {
        let (jar, ua) = self.session_http_ctx(session_id).await?;
        let auth = self.session_auth_header(session_id).await?;
        match self
            .http_fetch_snapshot(url, &jar, &ua, auth.clone())
            .await
        {
            Err(BrowserError::AuthRejected(_)) => {
                let auth2 = self.session_auth_header(session_id).await?;
                match self
                    .http_fetch_snapshot(url, &jar, &ua, auth2)
                    .await
                {
                    Err(BrowserError::AuthRejected(_)) => {
                        Err(BrowserError::AuthExpired {
                            hint: self.auth_hint(session_id).await,
                        })
                    }
                    other => other,
                }
            }
            other => other,
        }
    }

    pub(crate) async fn auth_hint(&self, session_id: &str) -> String {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .and_then(|s| s.auth.as_ref().map(|a| a.config.hint()))
            .unwrap_or_else(|| "token 被拒：请检查认证配置后重试".to_string())
    }

    /// HTTP 导航（独立函数：Click/Reload/回退直接复用，避免 async 递归）
    pub(crate) async fn navigate_http(
        &self,
        session_id: &str,
        url: &str,
    ) -> Result<BrowserResult, BrowserError> {
        let (snap, packed) = self.authed_navigate(session_id, url).await?;
        self.apply_cookies(session_id, packed).await;
        let out = render_snapshot_text(&snap);
        let verify = verify_for(url, &snap);
        let (curl, title) = (snap.url.clone(), snap.title.clone());
        self.commit_snapshot(session_id, snap, true).await?;
        self.record_fetch(&curl).await;
        Ok(BrowserResult::ok_verified(out, curl, title, 0, verify))
    }

    pub(crate) async fn dispatch_http(
        &self,
        session_id: &str,
        action: &BrowserAction,
    ) -> Result<BrowserResult, BrowserError> {
        match action {
            BrowserAction::Navigate { url } => self.navigate_http(session_id, url).await,
            BrowserAction::GetContent => {
                let (url, snap) = self.session_snapshot(session_id).await?;
                match snap {
                    Some(s) => Ok(BrowserResult::ok(
                        render_snapshot_text(&s),
                        url,
                        s.title.clone(),
                        0,
                    )),
                    None => Err(BrowserError::ActionFailed(
                        "no page loaded; Navigate first".to_string(),
                    )),
                }
            }
            BrowserAction::Click { selector } => {
                let raw = self.session_raw_html(session_id).await?;
                let doc = Html::parse_document(&raw);
                // 仅支持有 href 的 <a>（诚实面：submit 请走 SubmitForm）
                let css = format!("a[href]{selector}");
                let target = Selector::parse(&css)
                    .ok()
                    .and_then(|sel| doc.select(&sel).next())
                    .and_then(|el| el.value().attr("href").map(str::to_string))
                    .or_else(|| {
                        Selector::parse(selector).ok().and_then(|sel| {
                            doc.select(&sel).next().and_then(|el| {
                                if el.value().name() == "a" {
                                    el.value().attr("href").map(str::to_string)
                                } else {
                                    None
                                }
                            })
                        })
                    });
                match target {
                    Some(href) => {
                        let base = self.session_url(session_id).await?;
                        let abs = url::Url::parse(&base)
                            .and_then(|b| b.join(&href))
                            .map(|u| u.to_string())
                            .unwrap_or(href);
                        return self.navigate_http(session_id, &abs).await;
                    }
                    None => Err(BrowserError::ActionFailed(format!(
                        "click: no navigable link matches '{selector}'; submit 控件请用 SubmitForm"
                    ))),
                }
            }
            BrowserAction::Type { selector, text } => {
                self.record_fill(session_id, selector.clone(), text.clone())
                    .await
            }
            BrowserAction::FillField { selector, value } => {
                self.record_fill(session_id, selector.clone(), value.clone())
                    .await
            }
            BrowserAction::SelectOption { selector, values } => {
                // 下拉单选取首值记录（多选提交时由服务端解释）
                let value = values.first().cloned().unwrap_or_default();
                self.record_fill(session_id, selector.clone(), value)
                    .await
            }
            BrowserAction::Check { selector } => {
                let (key, default) =
                    self.resolve_fill_key(session_id, selector).await?;
                let value = if default.is_empty() {
                    "on".to_string()
                } else {
                    default
                };
                self.record_fill(session_id, key, value).await
            }
            BrowserAction::Uncheck { selector } => {
                self.drop_fill(session_id, selector).await
            }
            BrowserAction::Press { selector, key } => {
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("press recorded: '{key}' on '{selector}'（Http 无焦点概念）"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::GetText { selector } => {
                let raw = self.session_raw_html(session_id).await?;
                let doc = Html::parse_document(&raw);
                let sel = Selector::parse(selector).map_err(|_| {
                    BrowserError::ActionFailed(format!("bad selector '{selector}'"))
                })?;
                let text = doc
                    .select(&sel)
                    .map(|el| {
                        el.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    })
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(text, url, title, 0))
            }
            BrowserAction::PrintPdf => Err(BrowserError::ActionFailed(
                "PrintPdf 需要 ChromeHeadless/Cdp 后端（Http 无渲染）".to_string(),
            )),
            BrowserAction::SubmitForm { selector } => {
                self.http_submit_form(session_id, selector.clone()).await
            }
            BrowserAction::Reload => {
                let url = self.session_url(session_id).await?;
                return self.navigate_http(session_id, &url).await;
            }
            BrowserAction::GoBack => {
                let prev = self.pop_history(session_id, true).await?;
                return self.navigate_http(session_id, &prev).await;
            }
            BrowserAction::GoForward => {
                let next = self.pop_history(session_id, false).await?;
                return self.navigate_http(session_id, &next).await;
            }
            BrowserAction::WaitForElement {
                selector,
                timeout_ms,
            } => self.http_wait(session_id, selector, *timeout_ms).await,
            BrowserAction::Sleep { ms } => {
                // 单动作上限 ≈ op_timeout×2+5s：超 60s 必撞外层超时，
                // 故钳 60s；更长等待请链多个 Sleep（见 sessions/colab_stage8.json）。
                let wait = (*ms).min(60_000);
                tokio::time::sleep(Duration::from_millis(wait)).await;
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!("slept {wait}ms (http: record-only)"),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Scroll { direction, amount } => {
                let (url, title) = self.session_url_title(session_id).await?;
                Ok(BrowserResult::ok(
                    format!(
                        "scroll recorded ({direction:?}, {:?}); 静态抓取无可视滚动",
                        amount.unwrap_or(300.0)
                    ),
                    url,
                    title,
                    0,
                ))
            }
            BrowserAction::Screenshot => Err(BrowserError::ActionFailed(
                "screenshot 需要 ChromeHeadless 后端（Http 无渲染）".to_string(),
            )),
            BrowserAction::ExecuteJs { .. } => Err(BrowserError::ActionFailed(
                "ExecuteJs 需要 CDP 通道（在途）；Http 后端不支持".to_string(),
            )),
            // Tab 簿记（通常由 dispatch 预拦截；保留分支以完备匹配）
            BrowserAction::NewTab { url } => self.new_tab(session_id, url.clone()).await,
            BrowserAction::SwitchTab { index } => {
                self.switch_tab(session_id, *index).await
            }
            BrowserAction::CloseTab => self.close_tab(session_id).await,
            // 状态与传输由引擎层预拦截，不应到达后端分发
            BrowserAction::SaveState { .. }
            | BrowserAction::LoadState { .. }
            | BrowserAction::Download { .. }
            | BrowserAction::Upload { .. } => Err(BrowserError::ActionFailed(
                "unreachable: handled by engine dispatch".to_string(),
            )),
        }
    }

    pub(crate) async fn http_submit_form(
        &self,
        session_id: &str,
        selector: Option<String>,
    ) -> Result<BrowserResult, BrowserError> {
        let (snap, fills, uploads) = {
            let sessions = self.sessions.read().await;
            let s = sessions
                .get(session_id)
                .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
            (
                s.snapshot.clone().ok_or_else(|| {
                    BrowserError::ActionFailed("no page loaded; Navigate first".to_string())
                })?,
                s.pending_fills.clone(),
                s.pending_uploads.clone(),
            )
        };
        let form = pick_form(&snap, selector.as_deref())?;
        let mut params: Vec<(String, String)> = Vec::new();
        // 文件字段走 multipart（其余跳过文本组装）
        let mut upload_parts: Vec<(String, Vec<String>)> = Vec::new();
        for f in &form.fields {
            if f.kind == "file" {
                if let Some(paths) = uploads.get(&f.name) {
                    if !paths.is_empty() {
                        upload_parts.push((f.name.clone(), paths.clone()));
                    }
                }
                continue;
            }
            if f.kind == "submit"
                || f.kind == "button"
                || f.kind == "image"
                || f.kind == "reset"
            {
                continue;
            }
            let v = fills.get(&f.name).cloned().unwrap_or(f.value.clone());
            // 未勾选的 checkbox/radio 值为空则跳过
            if (f.kind == "checkbox" || f.kind == "radio") && v.is_empty() {
                continue;
            }
            params.push((f.name.clone(), v));
        }
        let (jar, ua) = self.session_http_ctx(session_id).await?;
        let timeout = self.blocking_timeout();
        let action = form.action.clone();
        let method = form.method.clone();
        self.polite_wait(&action).await?;
        let auth = self.session_auth_header(session_id).await?;
        let submit_once = |auth: Option<(String, String)>| {
            let client = self.client.clone();
            let jar = jar.clone();
            let ua = ua.clone();
            let action = action.clone();
            let method = method.clone();
            let params = params.clone();
            let upload_parts = upload_parts.clone();
            async move {
                if upload_parts.is_empty() {
                    let page = http_submit(
                        &client, &action, &method, &params, &jar, &ua, auth, timeout,
                    )
                    .await?;
                    let h = url::Url::parse(&page.final_url)
                        .map(|u| u.host_str().unwrap_or("").to_string())
                        .unwrap_or_default();
                    return Ok::<_, BrowserError>((page, h));
                }
                let page = multipart_submit(
                    &client, &action, &params, &upload_parts, &jar, &ua, auth, timeout,
                )
                .await?;
                let h = url::Url::parse(&page.final_url)
                    .map(|u| u.host_str().unwrap_or("").to_string())
                    .unwrap_or_default();
                Ok::<_, BrowserError>((page, h))
            }
        };
        let (fetched, host) = match submit_once(auth).await {
            Err(BrowserError::AuthRejected(_)) => {
                let auth2 = self.session_auth_header(session_id).await?;
                match submit_once(auth2).await {
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
        self.apply_cookies(
            session_id,
            fetched
                .set_cookies
                .into_iter()
                .map(|c| format!("{host}\u{1f}{c}"))
                .collect(),
        )
        .await;
        let (title, text, links, forms, raw_html) =
            parse_page(&fetched.final_url, &fetched.body)?;
        let status = fetched.status;
        let snap = PageSnapshot {
            url: fetched.final_url.clone(),
            title: title.clone(),
            text,
            links,
            forms,
            raw_html,
            http_status: Some(status),
        };
        let out = render_snapshot_text(&snap);
        let verify = verify_for(&action, &snap);
        // 提交成功后清空待填值与待传文件
        {
            let mut sessions = self.sessions.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                s.pending_fills.clear();
                s.pending_uploads.clear();
            }
        }
        self.commit_snapshot(session_id, snap, true).await?;
        self.record_fetch(&fetched.final_url).await;
        Ok(BrowserResult::ok_verified(
            out,
            fetched.final_url,
            title,
            0,
            verify,
        ))
    }

    pub(crate) async fn http_wait(
        &self,
        session_id: &str,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<BrowserResult, BrowserError> {
        let deadline =
            std::time::Instant::now() + Duration::from_millis(timeout_ms.min(120_000));
        loop {
            {
                let sessions = self.sessions.read().await;
                let s = sessions
                    .get(session_id)
                    .ok_or_else(|| BrowserError::SessionNotFound(session_id.to_string()))?;
                if let Some(snap) = &s.snapshot {
                    let doc = Html::parse_document(&snap.raw_html);
                    if Selector::parse(selector)
                        .ok()
                        .map(|sel| doc.select(&sel).next().is_some())
                        .unwrap_or(false)
                    {
                        let (url, title) = (s.current_url.clone(), s.title.clone());
                        return Ok(BrowserResult::ok(
                            format!("element '{selector}' present"),
                            url,
                            title,
                            0,
                        ));
                    }
                }
            }
            if std::time::Instant::now() >= deadline {
                let (url, title) = self.session_url_title(session_id).await?;
                return Ok(BrowserResult::fail(
                    format!("wait timed out: '{selector}'"),
                    url,
                    title,
                    timeout_ms,
                ));
            }
            // 重抓一次再判定
            let url = self.session_url(session_id).await?;
            if url != "about:blank" {
                let (jar, ua) = self.session_http_ctx(session_id).await?;
                let auth = self.session_auth_header(session_id).await.unwrap_or(None);
                if let Ok((snap, packed)) = self.http_fetch_snapshot(&url, &jar, &ua, auth).await {
                    self.apply_cookies(session_id, packed).await;
                    self.commit_snapshot(session_id, snap, false).await?;
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
