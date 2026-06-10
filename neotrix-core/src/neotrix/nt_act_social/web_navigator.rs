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

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};

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

/// CDP 消息
#[derive(Debug)]
#[allow(dead_code)]
struct CdpMessage {
    id: u64,
    method: String,
    params: Value,
}

/// 页面内容提取结果
#[derive(Debug, Clone)]
pub struct PageExtract {
    pub title: String,
    pub text: String,
    pub url: String,
    pub links: Vec<String>,
    pub meta: HashMap<String, String>,
    pub html: Option<String>,
    pub screenshot_b64: Option<String>,
}

/// 登录凭据
#[derive(Debug, Clone)]
pub struct LoginCredentials {
    pub username_field: String,    // CSS selector
    pub password_field: String,
    pub username: String,
    pub password: String,
    pub submit_selector: Option<String>,
    pub wait_for_url: Option<String>, // 登录成功后等待的 URL 模式
}

impl Default for LoginCredentials {
    fn default() -> Self {
        Self {
            username_field: "input[name='text'], input[autocomplete='username'], input[name='session[username_or_email]']".into(),
            password_field: "input[type='password'], input[name='session[password]']".into(),
            username: String::new(),
            password: String::new(),
            submit_selector: None,
            wait_for_url: None,
        }
    }
}

/// 人类行为模拟 — 让浏览器操作不可预测以规避检测
///
/// 核心原理：
///   机器行为是低熵的（固定间隔、固定步长、无鼠标轨迹）。
///   人类行为是高熵的（随机延时、变幅滚动、光标漂移、无意识交互）。
///   反检测系统的本质是熵检测。增加行为熵 = 降低被识别概率。
pub struct HumanBehavior {
    rng: Mutex<StdRng>,
    /// 行为指纹 — 每个会话生成一次，保持一致性
    pub scroll_speed: f64,
    pub pause_duration: (u64, u64), // min/max ms
    pub scroll_variance: f64,
    pub mouse_trail: bool,
    pub interaction_rate: f64,
}

impl HumanBehavior {
    pub fn new() -> Self {
        let mut thread_rng = rand::thread_rng();
        let rng = StdRng::from_rng(&mut thread_rng).unwrap_or_else(|_| StdRng::from_entropy());
        Self {
            rng: Mutex::new(rng),
            // 每个会话随机化行为参数 — 避免全局指纹
            scroll_speed: 0.3 + thread_rng.gen::<f64>() * 0.4,
            pause_duration: (800 + thread_rng.gen_range(0..1200),
                             2000 + thread_rng.gen_range(0..3000)),
            scroll_variance: 0.2 + thread_rng.gen::<f64>() * 0.3,
            mouse_trail: true,
            interaction_rate: 0.05 + thread_rng.gen::<f64>() * 0.1,
        }
    }

    /// 随机延时 (近似高斯分布: Box-Muller → clamp)
    pub fn random_pause(&self) -> Duration {
        let mut rng = self.rng.lock().unwrap();
        let (min, max) = self.pause_duration;
        let mean = (min + max) as f64 / 2.0;
        let std_dev = (max - min) as f64 / 6.0;
        // Box-Muller 近似高斯
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let delay = (mean + z * std_dev).clamp(min as f64, max as f64);
        Duration::from_millis(delay as u64)
    }

    /// 生成人类风格的滚动偏移 — 不是每次都滚到底
    pub fn human_scroll_offset(&self, viewport_height: u64, total_height: u64) -> u64 {
        let mut rng = self.rng.lock().unwrap();
        let base = viewport_height as f64 * (0.6 + rng.gen::<f64>() * 0.4);
        let variance = base * self.scroll_variance * (if rng.gen_bool(0.3) { -1.0 } else { 1.0 });
        let offset = (base + variance).max(viewport_height as f64 * 0.2) as u64;
        offset.min(total_height.saturating_sub(viewport_height))
    }

    /// 模拟光标移动轨迹 (通过 CDP Input.dispatchMouseEvent)
    pub fn mouse_move_js(&self, x: f64, y: f64, steps: usize) -> String {
        let mut rng = self.rng.lock().unwrap();
        let mut points = String::new();
        for i in 0..steps {
            let t = (i + 1) as f64 / steps as f64;
            // 贝塞尔曲线模拟自然鼠标轨迹
            let cx = x * t + (x * 0.1) * (t * (1.0 - t)) * 4.0;
            let cy = y * t + (y * 0.1) * (t * (1.0 - t)) * 4.0;
            let jitter_x = (rng.gen::<f64>() - 0.5) * 2.0;
            let jitter_y = (rng.gen::<f64>() - 0.5) * 2.0;
            points.push_str(&format!("{{x:{},y:{}}},", cx + jitter_x, cy + jitter_y));
        }
        points
    }

    /// 生成人类阅读的 JS — 随机暂停、随机滚动、偶尔悬停
    pub fn human_scroll_js(&self, max_scrolls: usize, viewport_height: u64) -> String {
        let mut rng = self.rng.lock().unwrap();
        let scrolls = 2 + rng.gen_range(0..=max_scrolls.max(4) - 2);
        let mut js = String::from("(async () => { const vp = window.innerHeight; const th = document.body.scrollHeight; let same = 0; let prev = 0;");
        for _ in 0..scrolls {
            let pause = 800 + rng.gen_range(0..=3000);
            let direction: &str = if rng.gen_bool(0.85) { "down" } else { "up" };
            let amount = if direction == "down" {
                viewport_height as f64 * (0.4 + rng.gen::<f64>() * 0.6)
            } else {
                viewport_height as f64 * (0.2 + rng.gen::<f64>() * 0.4)
            };
            js.push_str(&format!(
                "window.scrollBy({{top:{},behavior:'smooth'}});await new Promise(r=>setTimeout(r,{}));",
                if direction == "down" { amount as i64 } else { -(amount as i64) },
                pause
            ));
            // 偶尔悬停 — 模拟阅读
            if rng.gen_bool(0.3) {
                let hover = 1500 + rng.gen_range(0..=4000);
                js.push_str(&format!("await new Promise(r=>setTimeout(r,{}));", hover));
            }
        }
        js.push_str("return document.body.scrollHeight; })();");
        js
    }

    /// 生成随机用户代理 (Chrome 125-129 range)
    pub fn random_user_agent(&self) -> String {
        let mut rng = self.rng.lock().unwrap();
        let majors = [125, 126, 127, 128, 129];
        let major = majors[rng.gen_range(0..majors.len())];
        let build = rng.gen_range(5000..7000);
        let patch = rng.gen_range(100..500);
        format!(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.{}.{} Safari/537.36",
            major, build, patch
        )
    }

    /// 模拟人类: 随机等待 + 滚动阅读
    pub fn simulate_reading(&self, session_id: &str, nav: &WebNavigator, seconds: u64) -> Result<(), String> {
        let mut rng = self.rng.lock().unwrap();
        let _ = rng.gen::<f64>(); // 保持 RNG 活跃

        // 初始阅读延迟
        std::thread::sleep(Duration::from_millis(500 + rng.gen_range(0..=2000)));

        let vp_height = nav.evaluate_js(session_id, "window.innerHeight")
            .and_then(|v| v.as_u64().ok_or("no vp".into()))
            .unwrap_or(1080);

        let scrolls = (seconds / 3).max(2) as usize;
        for i in 0..scrolls {
            let scroll_js = self.human_scroll_js(1, vp_height);
            let _ = nav.evaluate_js(session_id, &scroll_js);

            // 偶尔悬停 + 鼠标移动
            if rng.gen_bool(self.interaction_rate) && i > 0 {
                let x = rng.gen_range(100..800) as f64;
                let y = rng.gen_range(100..600) as f64;
                let trail = self.mouse_move_js(x, y, 5 + rng.gen_range(0..=10));
                nav.evaluate_js(session_id, &format!(
                    "(function(){{const t=[{}];let i=0;function m(){{if(i>=t.length)return;const p=t[i++];window.dispatchEvent(new MouseEvent('mousemove',{{clientX:p.x,clientY:p.y, bubbles:true}}));requestAnimationFrame(m);}}m();}})();",
                    trail
                ))?;
            }

            std::thread::sleep(self.random_pause());
        }
        Ok(())
    }
}

impl Default for HumanBehavior {
    fn default() -> Self {
        Self::new()
    }
}

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
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
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
        if let Ok(output) = Command::new("which")
            .arg("google-chrome")
            .output()
        {
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

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("tokio runtime: {}", e))?;

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

        let mut child = cmd.spawn()
            .map_err(|e| format!("spawn Chrome: {} (path: {})", e, self.chrome_path))?;

        let stderr = child.stderr.take()
            .ok_or_else(|| "no stderr".to_string())?;
        let reader = BufReader::new(stderr);

        // 从 stderr 读取 DevTools URL
        let mut ws_url = None;
        for line in reader.lines() {
            let line = line.map_err(|e| format!("read stderr: {}", e))?;
            if line.contains("DevTools listening on ws://") {
                // Extract ws:// URL
                if let Some(start) = line.find("ws://") {
                    let end = line[start..].find(char::is_whitespace).unwrap_or(line.len() - start);
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

    /// 获取页面列表 (CDP Target.getTargets)
    #[allow(dead_code)]
    fn get_page_targets(&self) -> Result<Vec<Value>, String> {
        let ws_url = self.ws_url.as_ref().ok_or("not launched")?;
        let rt = self.rt.as_ref().ok_or("no runtime")?;

        rt.block_on(async {
            let (mut ws, _) = connect_async(ws_url)
                .await
                .map_err(|e| format!("WS connect: {}", e))?;

            let id = self.next_id();
            let msg = json!({
                "id": id,
                "method": "Target.getTargets",
                "params": {}
            });
            ws.send(Message::Text(msg.to_string().into()))
                .await
                .map_err(|e| format!("send: {}", e))?;

            loop {
                match ws.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let v: Value = serde_json::from_str(&text)
                            .map_err(|e| format!("parse: {}", e))?;
                        if v.get("id").and_then(|i| i.as_u64()) == Some(id) {
                            let targets = v["result"]["targetInfos"]
                                .as_array()
                                .cloned()
                                .unwrap_or_default();
                            return Ok(targets);
                        }
                    }
                    Some(Ok(_)) => continue,
                    Some(Err(e)) => return Err(format!("WS error: {}", e)),
                    None => return Err("WS closed".into()),
                }
            }
        })
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
                        let v: Value = serde_json::from_str(&text)
                            .map_err(|e| format!("parse: {}", e))?;
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
                                        let v2: Value = serde_json::from_str(&t).unwrap();
                                        if v2.get("id").and_then(|i| i.as_u64()) == Some(attach_id) {
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
                        let v: Value = serde_json::from_str(&text)
                            .map_err(|e| format!("parse: {}", e))?;
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
        std::thread::sleep(Duration::from_millis(2000));
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
            let desc = result["exceptionDetails"]["text"].as_str().unwrap_or("js error");
            return Err(format!("JS error: {}", desc));
        }

        Ok(result["result"]["value"].clone())
    }

    /// 点击元素
    pub fn click(&self, session_id: &str, selector: &str) -> Result<(), String> {
        let js = format!(r#"
(async () => {{
    const el = document.querySelector('{}');
    if (!el) throw new Error('selector not found: {}');
    el.scrollIntoView({{behavior:'instant', block:'center'}});
    await new Promise(r => setTimeout(r, 300));
    el.click();
    return 'clicked';
}})();
"#, selector.replace('\'', "\\'"), selector);
        self.evaluate_js(session_id, &js)?;
        Ok(())
    }

    /// 填写表单字段
    pub fn fill(&self, session_id: &str, selector: &str, value: &str) -> Result<(), String> {
        let js = format!(r#"
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
"#, selector, value.replace('\'', "\\'"));
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

        let meta_raw = parsed["meta"].as_object()
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
            .unwrap_or_default();

        Ok(PageExtract {
            title: parsed["title"].as_str().unwrap_or("").to_string(),
            text: parsed["text"].as_str().unwrap_or("").to_string(),
            url: self.get_url(session_id).unwrap_or_default(),
            links: parsed["links"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
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
        result["data"].as_str()
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

        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let dir = PathBuf::from(&home).join(COOKIE_DIR);
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("{}.json", domain.replace('.', "_")));
        let data = serde_json::to_string_pretty(&cookies)
            .map_err(|e| format!("serialize cookies: {}", e))?;
        std::fs::write(&path, data)
            .map_err(|e| format!("write cookies: {}", e))?;
        log::info!("[navigator] saved {} cookies to {:?}", cookies.as_array().map(|a| a.len()).unwrap_or(0), path);
        Ok(())
    }

    /// 从磁盘加载 cookies (CDP Network.setCookies)
    pub fn load_cookies(&self, session_id: &str, domain: &str) -> Result<bool, String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        let path = PathBuf::from(&home).join(COOKIE_DIR).join(format!("{}.json", domain.replace('.', "_")));
        if !path.exists() {
            return Ok(false);
        }
        let data = std::fs::read_to_string(&path)
            .map_err(|e| format!("read cookies: {}", e))?;
        let cookies: Vec<Value> = serde_json::from_str(&data)
            .map_err(|e| format!("parse cookies: {}", e))?;
        if cookies.is_empty() {
            return Ok(false);
        }
        self.send_cdp(
            session_id,
            "Network.setCookies",
            json!({"cookies": cookies}),
            Duration::from_secs(5),
        )?;
        log::info!("[navigator] loaded {} cookies from {:?}", cookies.len(), path);
        Ok(true)
    }

    /// 滚动到底部 (支持无限加载)
    pub fn scroll_to_bottom(&self, session_id: &str, max_count: usize) -> Result<usize, String> {
        let js = format!(r#"
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
"#, max = if max_count > 0 { max_count } else { 20 });
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
                std::thread::sleep(Duration::from_millis(3000));
                let current_url = self.get_url(session_id).unwrap_or_default();
                if !current_url.contains("login") && !current_url.contains("auth") && !current_url.contains("flow") {
                    log::info!("[navigator] cookies valid — skipping login");
                    return self.extract_page(session_id);
                }
            }
            log::info!("[navigator] cookies expired — re-login needed");
        }

        // 1. 导航到登录页
        self.navigate(session_id, login_url)?;
        std::thread::sleep(Duration::from_millis(2000));

        // 2. 填写用户名
        self.fill(session_id, &credentials.username_field, &credentials.username)?;

        // 3. 填写密码
        self.fill(session_id, &credentials.password_field, &credentials.password)?;

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
        std::thread::sleep(Duration::from_millis(5000));

        // 6. 如果是 X.com 两步验证 (username → 下一步 → password)
        //    检测是否有 password 字段出现
        let check_password_js = r#"
(function() {
    const pwFields = document.querySelectorAll('input[type="password"]');
    return pwFields.length > 0 && pwFields[0].offsetParent !== null;
})();
"#;
        if let Ok(Value::Bool(true)) = self.evaluate_js(session_id, check_password_js) {
            self.fill(session_id, &credentials.password_field, &credentials.password)?;
            std::thread::sleep(Duration::from_millis(1000));
            let submit_js = r#"
(function() {
    const btn = document.querySelector('button[type="submit"]');
    if (btn) { btn.click(); return 'clicked'; }
    return 'no btn';
})();
"#;
            self.evaluate_js(session_id, submit_js)?;
            std::thread::sleep(Duration::from_millis(3000));
        }

        // 7. 等待目标 URL (如果指定了)
        if let Some(wait_url) = &credentials.wait_for_url {
            for _ in 0..30 {
                if let Ok(url) = self.get_url(session_id) {
                    if url.contains(wait_url) {
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(1000));
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
        // 保留 profile 目录 (cookie 持久化)
        log::info!("[navigator] Chrome closed, profile kept at {}", self.data_dir);
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
    fn test_login_credentials_default() {
        let creds = LoginCredentials::default();
        assert!(creds.username_field.contains("username"));
        assert!(creds.password_field.contains("password"));
    }

    #[test]
    fn test_page_extract_default() {
        let extract = PageExtract {
            title: "Test".into(),
            text: "Hello".into(),
            url: "https://example.com".into(),
            links: vec![],
            meta: HashMap::new(),
            html: None,
            screenshot_b64: None,
        };
        assert_eq!(extract.title, "Test");
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
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
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

    #[test]
    fn test_human_behavior_random_pause() {
        let hb = HumanBehavior::new();
        for _ in 0..10 {
            let d = hb.random_pause();
            assert!(d.as_millis() >= hb.pause_duration.0 as u128);
            assert!(d.as_millis() <= hb.pause_duration.1 as u128);
        }
    }

    #[test]
    fn test_human_behavior_scroll_offset() {
        let hb = HumanBehavior::new();
        let offset = hb.human_scroll_offset(1080, 5000);
        assert!(offset <= 5000);
    }

    #[test]
    fn test_human_behavior_scroll_js() {
        let hb = HumanBehavior::new();
        let js = hb.human_scroll_js(5, 1080);
        assert!(js.contains("scrollBy"));
        assert!(js.contains("await"));
    }

    #[test]
    fn test_human_behavior_user_agent() {
        let hb = HumanBehavior::new();
        let ua = hb.random_user_agent();
        assert!(ua.contains("Chrome"));
        assert!(ua.contains("Safari"));
    }

    #[test]
    fn test_human_behavior_mouse_move() {
        let hb = HumanBehavior::new();
        let trail = hb.mouse_move_js(500.0, 300.0, 5);
        assert!(trail.contains("x"));
        assert!(trail.contains("y"));
    }

    #[test]
    fn test_human_behavior_params_randomized() {
        let hb1 = HumanBehavior::new();
        let hb2 = HumanBehavior::new();
        // 两次实例化的参数应该几乎总是不同
        assert!(
            (hb1.scroll_speed - hb2.scroll_speed).abs() > 0.001
            || hb1.pause_duration != hb2.pause_duration
            || (hb1.interaction_rate - hb2.interaction_rate).abs() > 0.001
        );
    }
}
