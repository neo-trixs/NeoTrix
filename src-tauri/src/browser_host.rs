use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub url: String,
    pub title: String,
    pub is_open: bool,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self { url: "about:blank".into(), title: "Browser".into(), is_open: false }
    }
}

/// 提取的页面内容
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: String,
    pub html: String,
    pub text: String,
}

impl PageContent {
    #[allow(dead_code)]
    pub fn summary(&self, max_chars: usize) -> String {
        let text = if self.text.len() > max_chars {
            self.text.chars().take(max_chars).collect::<String>() + "..."
        } else {
            self.text.clone()
        };
        format!("# {}\n\n{}\n\n[source: {}]", self.title, text, self.url)
    }
}

/// 浏览器窗口管理器
pub struct BrowserHost;

#[allow(dead_code)]
impl BrowserHost {
    pub fn open_or_navigate(app: &AppHandle, url: &str) -> Result<BrowserState, String> {
        let parsed = url.parse().map_err(|e| format!("invalid url: {}", e))?;
        let window_id = "neotrix-browser";

        if let Some(window) = app.get_webview_window(window_id) {
            let _ = window.navigate(parsed);
            let _ = window.show();
            let _ = window.set_focus();
            return Ok(BrowserState {
                url: url.to_string(),
                title: "Loading...".into(),
                is_open: true,
            });
        }

        let window = WebviewWindowBuilder::new(app, window_id, tauri::WebviewUrl::External(parsed))
            .title("NeoTrix Browser")
            .inner_size(1024.0, 768.0)
            .min_inner_size(400.0, 300.0)
            .resizable(true)
            .fullscreen(false)
            .build()
            .map_err(|e| format!("failed to create browser window: {}", e))?;

        let close_handle = app.clone();
        // 窗口关闭由 Tauri 自动管理, 不需要轮询
        // 浏览器窗口关闭时, 前端通过 emit("browser:closed") 通知状态变更
        let _window_clone = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let _ = close_handle.emit("browser:closed", ());
            }
        });

        Ok(BrowserState {
            url: url.to_string(),
            title: "Loading...".into(),
            is_open: true,
        })
    }

    pub fn execute_js(app: &AppHandle, script: &str) -> Result<(), String> {
        let window = app
            .get_webview_window("neotrix-browser")
            .ok_or_else(|| "browser window not open".to_string())?;
        window.eval(script).map_err(|e| format!("js eval error: {}", e))
    }

    pub fn go_back(app: &AppHandle) -> Result<(), String> {
        Self::execute_js(app, "window.history.back()")
    }

    pub fn go_forward(app: &AppHandle) -> Result<(), String> {
        Self::execute_js(app, "window.history.forward()")
    }

    pub fn reload(app: &AppHandle) -> Result<(), String> {
        Self::execute_js(app, "location.reload()")
    }

    pub fn close(app: &AppHandle) -> Result<(), String> {
        if let Some(window) = app.get_webview_window("neotrix-browser") {
            window.close().map_err(|e| format!("close error: {}", e))
        } else {
            Err("browser window not open".to_string())
        }
    }

    /// 服务端获取页面内容 (通过 HTTP, 绕过 eval 无法返回值的问题)
    pub fn fetch_page_content(url: &str) -> Result<PageContent, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("NeoTrix/1.0")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| format!("http client error: {}", e))?;

        let resp = client.get(url).send().map_err(|e| format!("fetch error: {}", e))?;
        let final_url = resp.url().to_string();
        let html = resp.text().map_err(|e| format!("read error: {}", e))?;

        let title = extract_title(&html);
        let text = strip_html(&html);

        Ok(PageContent {
            url: final_url,
            title,
            html,
            text,
        })
    }
}

/// 从 HTML 提取 <title>
#[allow(dead_code)]
fn extract_title(html: &str) -> String {
    for line in html.lines() {
        if let Some(start) = line.find("<title") {
            if let Some(rel_start) = line[start..].find('>') {
                let content_start = start + rel_start;
                let after_tag = &line[content_start + 1..];
                if let Some(end) = after_find("</title>", after_tag) {
                    return after_tag[..end].trim().to_string();
                }
            }
        }
    }
    String::new()
}

#[allow(dead_code)]
fn after_find(pat: &str, s: &str) -> Option<usize> {
    s.find(pat)
}

/// 剥离 HTML 标签, 返回纯文本
#[allow(dead_code)]
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut skip_chars = 0usize;
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if skip_chars > 0 {
            skip_chars -= 1;
            i += 1;
            continue;
        }
        let c = chars[i];
        if !in_tag && c == '<' {
            // 检测 <script 或 <style
            if i + 6 < chars.len() {
                let tag_start: String = chars[i..(i + 7.min(chars.len() - i))].iter().collect();
                if tag_start.starts_with("<script") || tag_start.starts_with("<SCRIPT") {
                    in_script = true;
                    in_tag = true;
                    i += 1;
                    continue;
                }
                if tag_start.starts_with("<style") || tag_start.starts_with("<STYLE") {
                    in_style = true;
                    in_tag = true;
                    i += 1;
                    continue;
                }
            }
            in_tag = true;
            i += 1;
            continue;
        }
        if in_tag && c == '>' {
            in_tag = false;
            if in_script {
                // 检测 </script>
                if i + 8 < chars.len() {
                    let end: String = chars[i..(i + 9.min(chars.len() - i))].iter().collect();
                    if end.starts_with("</script") || end.starts_with("</SCRIPT") {
                        in_script = false;
                        // 跳过 </script> 剩余
                        if let Some(close) = end.find('>') {
                            skip_chars = close;
                        }
                    }
                }
            }
            if in_style
                && i + 7 < chars.len() {
                    let end: String = chars[i..(i + 8.min(chars.len() - i))].iter().collect();
                    if end.starts_with("</style") || end.starts_with("</STYLE") {
                        in_style = false;
                        if let Some(close) = end.find('>') {
                            skip_chars = close;
                        }
                    }
                }
            // 添加空格代替标签
            if !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
            i += 1;
            continue;
        }
        if !in_tag && !in_script && !in_style {
            if c.is_whitespace() {
                if !out.ends_with(' ') && !out.is_empty() {
                    out.push(' ');
                }
            } else {
                // 解码常见 HTML 实体
                match c {
                    '&' => {
                        let rest: String = chars[i..].iter().collect();
                        if rest.starts_with("&amp;") { out.push('&'); skip_chars = 4; }
                        else if rest.starts_with("&lt;") { out.push('<'); skip_chars = 3; }
                        else if rest.starts_with("&gt;") { out.push('>'); skip_chars = 3; }
                        else if rest.starts_with("&quot;") { out.push('"'); skip_chars = 5; }
                        else if rest.starts_with("&#39;") || rest.starts_with("&#x27;") { out.push('\''); skip_chars = rest.starts_with("&#39;") as usize * 4 + rest.starts_with("&#x27;") as usize * 5; }
                        else if rest.starts_with("&nbsp;") { out.push(' '); skip_chars = 5; }
                        else { out.push('&'); }
                    }
                    _ => out.push(c),
                }
            }
        }
        i += 1;
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Hello World</title></head><body></body></html>";
        assert_eq!(extract_title(html), "Hello World");
    }

    #[test]
    fn test_extract_title_empty() {
        let html = "<html><head></head><body></body></html>";
        assert_eq!(extract_title(html), "");
    }

    #[test]
    fn test_strip_html_simple() {
        let html = "<p>Hello <b>World</b></p>";
        let text = strip_html(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains('<'));
        assert!(!text.contains('>'));
    }

    #[test]
    fn test_strip_html_removes_script() {
        let html = "<html><head><script>alert('xss');</script></head><body><p>Hello</p></body></html>";
        let text = strip_html(html);
        assert!(text.contains("Hello"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("xss"));
    }

    #[test]
    fn test_strip_html_removes_style() {
        let html = "<html><head><style>body { color: red; }</style></head><body><p>Hello</p></body></html>";
        let text = strip_html(html);
        assert!(text.contains("Hello"));
        assert!(!text.contains("color"));
    }

    #[test]
    fn test_strip_html_entities() {
        let html = "<p>AT&amp;T &lt;test&gt;</p>";
        let text = strip_html(html);
        assert!(text.contains("AT&T"));
        assert!(text.contains("<test>"));
    }

    #[test]
    fn test_page_content_summary() {
        let pc = PageContent {
            url: "https://example.com".into(),
            title: "Test".into(),
            html: String::new(),
            text: "Hello World".into(),
        };
        let s = pc.summary(100);
        assert!(s.contains("Test"));
        assert!(s.contains("Hello World"));
        assert!(s.contains("example.com"));
    }

    #[test]
    fn test_page_content_summary_truncate() {
        let pc = PageContent {
            url: "https://example.com".into(),
            title: "Test".into(),
            html: String::new(),
            text: "A".repeat(1000),
        };
        let s = pc.summary(50);
        assert!(s.len() < 200); // 标题 + url 开销之外的内容被截断
    }
}
