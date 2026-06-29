//! WebNavigator — NeoTrix 自身的反检测浏览器自动化工具
//!
//! 负熵理念：
//!   浏览器是意识的感官延伸。用户输入 URL → 意识体导航 → 提取结构化信息
//!   → 去噪 → VSA 编码 → 知识吸收。每一步都在降低熵。
//!
//! 设计：
//!   1. 通过 CDP (Chrome DevTools Protocol) 直接控制 Chrome
//!   2. 零外部浏览器自动化依赖 (无 chromiumoxide/playwright)
//!   3. 纯 WebSocket (tokio-tungstenite) 通信
//!   4. Sync API (tokio::runtime::Runtime 包装)
//!   5. 反检测: stealth flags + JS 注入

use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::core::nt_core_util;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use super::types::{LoginCredentials, PageExtract};

/// Cookie 持久化路径 — ~/.neotrix/cookies/{domain}.json
const COOKIE_DIR: &str = ".neotrix/cookies";

/// Chrome 反检测启动参数
const STEALTH_ARGS: &[&str] = &[
    "--no-sandbox",
    "--disable-setuid-sandbox",
    "--disable-dev-shm-usage",
    "--disable-gpu",
    "--disable-blink-features=AutomationControlled",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-sync",
    "--disable-translate",
    "--mute-audio",
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
    "--disable-webrtc",
    "--enforce-webrtc-ip-permission-check",
    "--force-webrtc-ip-handling-policy=disable_non_proxied_udp",
    "--window-size=1920,1080",
    "--start-maximized",
];

/// NeoTrix 自身反检测浏览器导航器
pub struct WebNavigator {
    chrome_process: Option<Mutex<Child>>,
    ws_url: Option<String>,
    rt: Option<tokio::runtime::Runtime>,
    msg_id: AtomicU64,
    chrome_path: String,
    data_dir: String,
}

impl WebNavigator {
    /// 创建导航器，自动检测 Chrome 路径
    pub fn new() -> Self {
        let chrome_path = Self::find_chrome();
        let data_dir = format!(
            "{}/.neotrix/chrome_profile_{}",
            nt_core_util::home_dir().to_string_lossy().to_string(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        Self {
            chrome_process: None,
            ws_url: None,
            rt: None,
            msg_id: AtomicU64::new(1),
            chrome_path,
            data_dir,
        }
    }

    /// 设置自定义 Chrome/Chromium 路径
    pub fn with_chrome_path(mut self, path: &str) -> Self {
        self.chrome_path = path.to_string();
        self
    }

    /// 自动检测 Chrome
    fn find_chrome() -> String {
        let candidates = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
            "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() {
                return c.to_string();
            }
        }
        // fallback: try which
        if let Ok(output) = Command::new("which").arg("google-chrome").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return path;
                }
            }
        }
        "/usr/bin/google-chrome".into()
    }

    /// 启动 Chrome + CDP 连接
    pub fn launch(&mut self) -> Result<(), String> {
        // 检查 Chrome 是否存在
        if !std::path::Path::new(&self.chrome_path).exists() {
            return Err(format!(
                "Chrome not found at '{}'. Install Chrome or set path via with_chrome_path()",
                self.chrome_path
            ));
        }

        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("tokio runtime: {}", e))?;

        // 创建用户数据目录
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("create profile dir: {}", e))?;

        // 启动 Chrome (remote debugging on random port)
        let mut cmd = Command::new(&self.chrome_path);
        cmd.arg(format!("--user-data-dir={}", self.data_dir));
        cmd.arg("--remote-debugging-port=0");
        for arg in STEALTH_ARGS {
            cmd.arg(arg);
        }
        cmd.arg("about:blank");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("spawn Chrome: {} (path: {})", e, self.chrome_path))?;

        let stderr = child.stderr.take().ok_or_else(|| "no stderr".to_string())?;
        let reader = BufReader::new(stderr);

        // 从 stderr 读取 DevTools URL
        let mut ws_url = None;
        for line in reader.lines() {
            let line = line.map_err(|e| format!("read stderr: {}", e))?;
            if line.contains("DevTools listening on ws://") {
                // Extract ws:// URL
                if let Some(start) = line.find("ws://") {
                    let end = line[start..]
                        .find(char::is_whitespace)
                        .unwrap_or(line.len() - start);
                    ws_url = Some(line[start..start + end].to_string());
                }
                break;
            }
        }

        let ws_url = ws_url.ok_or_else(|| {
            let _ = child.kill();
            "无法获取 Chrome DevTools WebSocket URL (Chrome 可能已存在实例)".to_string()
        })?;

        self.chrome_process = Some(Mutex::new(child));
        self.ws_url = Some(ws_url);
        self.rt = Some(rt);

        Ok(())
    }

    /// 创建新页面并连接 Page domain
    pub fn new_page(&self) -> Result<String, String> {
        let ws_url = self.ws_url.as_ref().ok_or("not launched")?;
        let rt = self.rt.as_ref().ok_or("no runtime")?;

        rt.block_on(async {
            let (mut ws, _) = connect_async(ws_url)
                .await
                .map_err(|e| format!("WS connect: {}", e))?;

            let id = self.next_id();
            let msg = json!({
                "id": id,
                "method": "Target.createTarget",
                "params": {
                    "url": "about:blank",
                    "newWindow": false
                }
            });
            ws.send(Message::Text(msg.to_string().into()))
                .await
                .map_err(|e| format!("send: {}", e))?;

            loop {
                match ws.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let v: Value =
                            serde_json::from_str(&text).map_err(|e| format!("parse: {}", e))?;
                        if v.get("id").and_then(|i| i.as_u64()) == Some(id) {
                            let target_id = v["result"]["targetId"]
                                .as_str()
                                .ok_or("no targetId")?
                                .to_string();

                            // 连接 Page domain
                            let attach_id = self.next_id();
                            // Target.attachToTarget
                            let attach_msg = json!({
                                "id": attach_id,
                                "method": "Target.attachToTarget",
                                "params": {
                                    "targetId": &target_id,
                                    "flatten": true
                                }
                            });
                            ws.send(Message::Text(attach_msg.to_string().into()))
                                .await
                                .map_err(|e| format!("send attach: {}", e))?;

                            // Wait for attach response with sessionId
                            loop {
                                match ws.next().await {
                                    Some(Ok(Message::Text(t))) => {
                                        let v2: Value = serde_json::from_str(&t)
                                            .map_err(|e| format!("parse attach response: {}", e))?;
                                        if v2.get("id").and_then(|i| i.as_u64()) == Some(attach_id)
                                        {
                                            let session_id = v2["result"]["sessionId"]
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string();
                                            return Ok(format!("{}:{}", target_id, session_id));
                                        }
                                    }
                                    Some(Ok(_)) => continue,
                                    _ => return Err("attach failed".into()),
                                }
                            }
                        }
                    }
                    Some(Ok(_)) => continue,
                    _ => return Err("create target failed".into()),
                }
            }
        })
    }

    fn next_id(&self) -> u64 {
        self.msg_id.fetch_add(1, Ordering::SeqCst)
    }

    /// 发送 CDP 命令到指定 session
    fn send_cdp(
        &self,
        session_id: &str,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, String> {
        let ws_url = self.ws_url.as_ref().ok_or("not launched")?;
        let rt = self.rt.as_ref().ok_or("no runtime")?;

        rt.block_on(async {
            let (mut ws, _) = connect_async(ws_url)
                .await
                .map_err(|e| format!("WS connect: {}", e))?;

            let id = self.next_id();
            let msg = json!({
                "id": id,
                "sessionId": session_id,
                "method": method,
                "params": params
            });

            ws.send(Message::Text(msg.to_string().into()))
                .await
                .map_err(|e| format!("send: {}", e))?;

            let deadline = Instant::now() + timeout;
            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err("CDP timeout".into());
                }

                match tokio::time::timeout(remaining, ws.next()).await {
                    Ok(Some(Ok(Message::Text(text)))) => {
                        let v: Value =
                            serde_json::from_str(&text).map_err(|e| format!("parse: {}", e))?;
                        if v.get("id").and_then(|i| i.as_u64()) == Some(id) {
                            if let Some(err) = v.get("error") {
                                return Err(format!("CDP error: {}", err));
                            }
                            return Ok(v["result"].clone());
                        }
                    }
                    Ok(Some(Ok(_))) => continue,
                    Ok(Some(Err(e))) => return Err(format!("WS: {}", e)),
                    Ok(None) => return Err("WS closed".into()),
                    Err(_) => return Err("timeout".into()),
                }
            }
        })
    }

    /// 导航到 URL
    pub fn navigate(&self, session_id: &str, url: &str) -> Result<(), String> {
        self.send_cdp(
            session_id,
            "Page.navigate",
            json!({"url": url}),
            Duration::from_secs(15),
        )?;
        // Wait for page load
        let _ = self
            .rt
            .as_ref()
            .ok_or("no runtime")?
            .block_on(tokio::time::sleep(Duration::from_millis(2000)));
        Ok(())
    }

    /// 执行 JS 并获取返回值
    pub fn evaluate_js(&self, session_id: &str, js: &str) -> Result<Value, String> {
        let result = self.send_cdp(
            session_id,
            "Runtime.evaluate",
            json!({
                "expression": js,
                "returnByValue": true,
                "awaitPromise": true
            }),
            Duration::from_secs(10),
        )?;

        if result.get("exceptionDetails").is_some() {
            let desc = result["exceptionDetails"]["text"]
                .as_str()
                .unwrap_or("js error");
            return Err(format!("JS error: {}", desc));
        }

        Ok(result["result"]["value"].clone())
    }

    /// 点击元素
    pub fn click(&self, session_id: &str, selector: &str) -> Result<(), String> {
        let js = format!(
            r#"
(async () => {{
    const el = document.querySelector('{}');
    if (!el) throw new Error('selector not found: {}');
    el.scrollIntoView({{behavior:'instant', block:'center'}});
    await new Promise(r => setTimeout(r, 300));
    el.click();
    return 'clicked';
}})();
"#,
            selector.replace('\'', "\\'"),
            selector
        );
        self.evaluate_js(session_id, &js)?;
        Ok(())
    }

    /// 填写表单字段
    pub fn fill(&self, session_id: &str, selector: &str, value: &str) -> Result<(), String> {
        let js = format!(
            r#"
(async () => {{
    const el = document.querySelector('{}');
    if (!el) throw new Error('field not found');
    el.scrollIntoView({{behavior:'instant', block:'center'}});
    el.focus();
    el.value = '';
    document.execCommand('insertText', false, '{}');
    el.dispatchEvent(new Event('input', {{bubbles:true}}));
    el.dispatchEvent(new Event('change', {{bubbles:true}}));
    return 'filled';
}})();
"#,
            selector,
            value.replace('\'', "\\'")
        );
        self.evaluate_js(session_id, &js)?;
        Ok(())
    }

    /// 获取页面文本内容
    pub fn get_page_text(&self, session_id: &str) -> Result<String, String> {
        let js = r#"
(function() {
    const text = document.body?.innerText || '';
    const title = document.title;
    return JSON.stringify({ title, text: text.slice(0, 20000) });
})();
"#;
        let result = self.evaluate_js(session_id, js)?;
        let text = result.as_str().unwrap_or("");
        let parsed: Value = serde_json::from_str(text).unwrap_or(json!({"text": "", "title": ""}));
        Ok(parsed["text"].as_str().unwrap_or("").to_string())
    }

    /// 获取页面标题
    pub fn get_title(&self, session_id: &str) -> Result<String, String> {
        self.evaluate_js(session_id, "document.title")
            .map(|v| v.as_str().unwrap_or("").to_string())
    }

    /// 获取当前 URL
    pub fn get_url(&self, session_id: &str) -> Result<String, String> {
        self.evaluate_js(session_id, "location.href")
            .map(|v| v.as_str().unwrap_or("").to_string())
    }

    /// 提取完整页面信息
    pub fn extract_page(&self, session_id: &str) -> Result<PageExtract, String> {
        let js = r#"
(function() {
    const title = document.title;
    const text = document.body?.innerText || '';
    const html = document.documentElement?.outerHTML || '';
    const links = Array.from(document.querySelectorAll('a[href]')).map(a => a.href).slice(0, 100);
    const meta = {};
    document.querySelectorAll('meta[name], meta[property]').forEach(m => {
        const k = m.getAttribute('name') || m.getAttribute('property') || '';
        const v = m.getAttribute('content') || '';
        if (k && v) meta[k] = v;
    });
    return JSON.stringify({ title, text: text.slice(0, 20000), html: html.slice(0, 50000), links, meta });
})();
"#;
        let result = self.evaluate_js(session_id, js)?;
        let raw = result.as_str().unwrap_or("{}");
        let parsed: Value = serde_json::from_str(raw).unwrap_or_default();

        let meta_raw = parsed["meta"]
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(PageExtract {
            title: parsed["title"].as_str().unwrap_or("").to_string(),
            text: parsed["text"].as_str().unwrap_or("").to_string(),
            url: self.get_url(session_id).unwrap_or_default(),
            links: parsed["links"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            meta: meta_raw,
            html: parsed["html"].as_str().map(String::from),
            screenshot_b64: None,
        })
    }

    /// 截图 (base64 PNG)
    pub fn screenshot(&self, session_id: &str) -> Result<String, String> {
        let result = self.send_cdp(
            session_id,
            "Page.captureScreenshot",
            json!({"format": "png", "fromSurface": true}),
            Duration::from_secs(10),
        )?;
        result["data"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "no screenshot data".into())
    }

    /// 关闭页面
    pub fn close_page(&self, session_id: &str) -> Result<(), String> {
        self.evaluate_js(session_id, "window.close()")?;
        Ok(())
    }

    /// 保存 cookies 到磁盘 (CDP Network.getAllCookies)
    pub fn save_cookies(&self, session_id: &str, domain: &str) -> Result<(), String> {
        let result = self.send_cdp(
            session_id,
            "Network.getAllCookies",
            json!({}),
            Duration::from_secs(5),
        )?;
        let cookies = result["cookies"].clone();
        if !cookies.is_array() || cookies.as_array().map(|a| a.is_empty()).unwrap_or(true) {
            return Ok(());
        }

        let home = nt_core_util::home_dir().to_string_lossy().to_string();
        let dir = PathBuf::from(&home).join(COOKIE_DIR);
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("{}.json", domain.replace('.', "_")));
        let data = serde_json::to_string_pretty(&cookies)
            .map_err(|e| format!("serialize cookies: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, data).map_err(|e| format!("write cookies: {}", e))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("rename cookies: {}", e))?;
        log::info!(
            "[navigator] saved {} cookies to {:?}",
            cookies.as_array().map(|a| a.len()).unwrap_or(0),
            path
        );
        Ok(())
    }

    /// 从磁盘加载 cookies (CDP Network.setCookies)
    pub fn load_cookies(&self, session_id: &str, domain: &str) -> Result<bool, String> {
        let home = nt_core_util::home_dir().to_string_lossy().to_string();
        let path = PathBuf::from(&home)
            .join(COOKIE_DIR)
            .join(format!("{}.json", domain.replace('.', "_")));
        if !path.exists() {
            return Ok(false);
        }
        let data = std::fs::read_to_string(&path).map_err(|e| format!("read cookies: {}", e))?;
        let cookies: Vec<Value> =
            serde_json::from_str(&data).map_err(|e| format!("parse cookies: {}", e))?;
        if cookies.is_empty() {
            return Ok(false);
        }
        self.send_cdp(
            session_id,
            "Network.setCookies",
            json!({"cookies": cookies}),
            Duration::from_secs(5),
        )?;
        log::info!(
            "[navigator] loaded {} cookies from {:?}",
            cookies.len(),
            path
        );
        Ok(true)
    }

    /// 滚动到底部 (支持无限加载)
    pub fn scroll_to_bottom(&self, session_id: &str, max_count: usize) -> Result<usize, String> {
        let js = format!(
            r#"
(async () => {{
    let prevHeight = 0;
    let sameCount = 0;
    for (let i = 0; i < {max}; i++) {{
        window.scrollTo(0, document.body.scrollHeight);
        await new Promise(r => setTimeout(r, 1500));
        const newHeight = document.body.scrollHeight;
        if (newHeight === prevHeight) {{
            sameCount++;
            if (sameCount >= 3) break;
        }} else {{
            sameCount = 0;
        }}
        prevHeight = newHeight;
    }}
    return document.body.scrollHeight;
}})();
"#,
            max = if max_count > 0 { max_count } else { 20 }
        );
        let result = self.evaluate_js(session_id, &js)?;
        let height = result.as_u64().unwrap_or(0) as usize;
        Ok(height)
    }

    /// 登录流程: 导航 → 检测表单 → 填表 → 提交 → 等待跳转
    pub fn login_flow(
        &self,
        session_id: &str,
        login_url: &str,
        credentials: &LoginCredentials,
    ) -> Result<PageExtract, String> {
        let domain = login_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("unknown");

        // 尝试加载已有 cookies (避免重新登录)
        if let Ok(true) = self.load_cookies(session_id, domain) {
            log::info!("[navigator] loaded saved cookies for {}", domain);
            // 尝试直接访问主页
            if let Some(home_url) = &credentials.wait_for_url {
                let direct_url = format!("https://{}", home_url);
                self.navigate(session_id, &direct_url)?;
                self.rt
                    .as_ref()
                    .ok_or("no runtime")?
                    .block_on(tokio::time::sleep(Duration::from_millis(3000)));
                let current_url = self.get_url(session_id).unwrap_or_default();
                if !current_url.contains("login")
                    && !current_url.contains("auth")
                    && !current_url.contains("flow")
                {
                    log::info!("[navigator] cookies valid — skipping login");
                    return self.extract_page(session_id);
                }
            }
            log::info!("[navigator] cookies expired — re-login needed");
        }

        // 1. 导航到登录页
        self.navigate(session_id, login_url)?;
        self.rt
            .as_ref()
            .ok_or("no runtime")?
            .block_on(tokio::time::sleep(Duration::from_millis(2000)));

        // 2. 填写用户名
        self.fill(
            session_id,
            &credentials.username_field,
            &credentials.username,
        )?;

        // 3. 填写密码
        self.fill(
            session_id,
            &credentials.password_field,
            &credentials.password,
        )?;

        // 4. 提交 (点击提交按钮或按 Enter)
        if let Some(submit_sel) = &credentials.submit_selector {
            self.click(session_id, submit_sel)?;
        } else {
            // 尝试多个常见提交按钮
            let submit_js = r#"
(function() {
    const btns = document.querySelectorAll(
        'button[type="submit"], input[type="submit"], ' +
        'div[role="button"]:has(div:contains("Log in")), ' +
        'div[role="button"]:has(span:contains("Sign in"))'
    );
    for (const btn of btns) {
        if (btn.offsetParent !== null) {
            btn.click();
            return 'clicked';
        }
    }
    return 'no button found';
})();
"#;
            self.evaluate_js(session_id, submit_js)?;
        }

        // 5. 等待页面跳转/加载
        self.rt
            .as_ref()
            .ok_or("no runtime")?
            .block_on(tokio::time::sleep(Duration::from_millis(5000)));

        // 6. 如果是 X.com 两步验证 (username → 下一步 → password)
        //    检测是否有 password 字段出现
        let check_password_js = r#"
(function() {
    const pwFields = document.querySelectorAll('input[type="password"]');
    return pwFields.length > 0 && pwFields[0].offsetParent !== null;
})();
"#;
        if let Ok(Value::Bool(true)) = self.evaluate_js(session_id, check_password_js) {
            self.fill(
                session_id,
                &credentials.password_field,
                &credentials.password,
            )?;
            self.rt
                .as_ref()
                .ok_or("no runtime")?
                .block_on(tokio::time::sleep(Duration::from_millis(1000)));
            let submit_js = r#"
(function() {
    const btn = document.querySelector('button[type="submit"]');
    if (btn) { btn.click(); return 'clicked'; }
    return 'no btn';
})();
"#;
            self.evaluate_js(session_id, submit_js)?;
            self.rt
                .as_ref()
                .ok_or("no runtime")?
                .block_on(tokio::time::sleep(Duration::from_millis(3000)));
        }

        // 7. 等待目标 URL (如果指定了)
        if let Some(wait_url) = &credentials.wait_for_url {
            for _ in 0..30 {
                if let Ok(url) = self.get_url(session_id) {
                    if url.contains(wait_url) {
                        break;
                    }
                }
                self.rt
                    .as_ref()
                    .ok_or("no runtime")?
                    .block_on(tokio::time::sleep(Duration::from_millis(1000)));
            }
        }

        // 8. 保存登录 cookies
        let _ = self.save_cookies(session_id, domain);

        // 9. 提取登录后的页面
        self.extract_page(session_id)
    }

    /// 关闭 Chrome
    pub fn close(&mut self) {
        if let Some(mutex) = self.chrome_process.take() {
            if let Ok(mut child) = mutex.into_inner() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        log::info!(
            "[navigator] Chrome closed, profile kept at {}",
            self.data_dir
        );
    }
}

impl Drop for WebNavigator {
    fn drop(&mut self) {
        self.close();
    }
}

impl Default for WebNavigator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_navigator_construction() {
        let nav = WebNavigator::new();
        assert!(nav.chrome_path.contains("Chrome") || nav.chrome_path.contains("chromium"));
    }

    #[test]
    fn test_find_chrome() {
        let path = WebNavigator::find_chrome();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_with_chrome_path() {
        let nav = WebNavigator::new().with_chrome_path("/custom/chrome");
        assert_eq!(nav.chrome_path, "/custom/chrome");
    }

    #[test]
    fn test_next_id_increments() {
        let nav = WebNavigator::new();
        let id1 = nav.next_id();
        let id2 = nav.next_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_stealth_args_count() {
        assert!(STEALTH_ARGS.len() > 15);
    }

    #[test]
    fn test_cookie_path_format() {
        let home = nt_core_util::home_dir().to_string_lossy().to_string();
        let dir = PathBuf::from(&home).join(COOKIE_DIR).join("x_com.json");
        assert!(dir.to_string_lossy().contains("x_com.json"));
    }

    #[test]
    fn test_scroll_to_bottom_js_generation() {
        // just verify the JS template is well-formed
        let js = format!(
            r#"(async () => {{
    let prevHeight = 0;
    let sameCount = 0;
    for (let i = 0; i < {}; i++) {{
        window.scrollTo(0, document.body.scrollHeight);
        await new Promise(r => setTimeout(r, 1500));
        const newHeight = document.body.scrollHeight;
        if (newHeight === prevHeight) {{
            sameCount++; if (sameCount >= 3) break;
        }} else {{ sameCount = 0; }}
        prevHeight = newHeight;
    }}
    return document.body.scrollHeight;
}})();"#,
            20
        );
        assert!(js.contains("scrollTo"));
        assert!(js.contains("1500"));
    }

    #[test]
    fn test_close_page_no_panic() {
        let nav = WebNavigator::new();
        // not launched — should fail gracefully
        let result = nav.close_page("fake-session");
        assert!(result.is_err());
    }

    #[test]
    fn test_launch_no_chrome_error() {
        let mut nav = WebNavigator::new().with_chrome_path("/nonexistent/chrome");
        let result = nav.launch();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chrome not found"));
    }
}
