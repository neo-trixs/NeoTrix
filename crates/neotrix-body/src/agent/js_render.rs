use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::browser_agent::{BrowserAction, BrowserAgent, ScreenshotFormat, WaitUntil};

/// Snapshot of a fully rendered page including JS execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPage {
    pub html: String,
    pub final_url: String,
    pub screenshot_b64: Option<String>,
    pub timing_ms: u64,
    pub vsa_fingerprint: [u64; 4],
}

/// JS rendering engine — wraps BrowserAgent to handle JS-heavy pages.
#[derive(Debug)]
pub struct JsRenderer {
    browser_agent: Arc<BrowserAgent>,
    js_timeout_ms: u64,
}

impl JsRenderer {
    pub fn new(browser_agent: Arc<BrowserAgent>, js_timeout_ms: u64) -> Self {
        Self {
            browser_agent,
            js_timeout_ms,
        }
    }

    pub fn browser_agent(&self) -> &BrowserAgent {
        &self.browser_agent
    }

    pub fn js_timeout_ms(&self) -> u64 {
        self.js_timeout_ms
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.js_timeout_ms = timeout_ms;
        self
    }

    /// Navigate, wait for page load, extract rendered HTML + screenshot.
    pub fn render_url(&self, url: &str, wait_for: WaitUntil) -> Result<RenderedPage, String> {
        let start = Instant::now();

        let nav_action = BrowserAction::Navigate {
            url: url.into(),
            wait_until: wait_for,
        };
        self.browser_agent.execute_action(&nav_action)?;

        let extract_action = BrowserAction::ExtractHTML { selector: None };
        let html_result = self.browser_agent.execute_action(&extract_action)?;

        let ss_action = BrowserAction::Screenshot {
            format: ScreenshotFormat::Png,
        };
        let ss_result = self.browser_agent.execute_action(&ss_action).ok();

        let elapsed = start.elapsed().as_millis() as u64;
        let html_out = html_result.output;
        let fp = compute_vsa_fingerprint(&html_out, url, elapsed);

        Ok(RenderedPage {
            html: html_out,
            final_url: url.into(),
            screenshot_b64: ss_result.and_then(|r| r.screenshot_b64),
            timing_ms: elapsed,
            vsa_fingerprint: fp,
        })
    }

    /// Execute custom JS after navigation, then extract resulting page state.
    pub fn render_with_js(&self, url: &str, js_code: &str) -> Result<RenderedPage, String> {
        let start = Instant::now();

        let nav_action = BrowserAction::Navigate {
            url: url.into(),
            wait_until: WaitUntil::NetworkIdle,
        };
        self.browser_agent.execute_action(&nav_action)?;

        let js_action = BrowserAction::ExecuteJS {
            code: js_code.into(),
        };
        self.browser_agent.execute_action(&js_action)?;

        let extract_action = BrowserAction::ExtractHTML { selector: None };
        let html_result = self.browser_agent.execute_action(&extract_action)?;

        let ss_action = BrowserAction::Screenshot {
            format: ScreenshotFormat::Png,
        };
        let ss_result = self.browser_agent.execute_action(&ss_action).ok();

        let elapsed = start.elapsed().as_millis() as u64;
        let html_out = html_result.output;
        let fp = compute_vsa_fingerprint(&html_out, url, elapsed);

        Ok(RenderedPage {
            html: html_out,
            final_url: url.into(),
            screenshot_b64: ss_result.and_then(|r| r.screenshot_b64),
            timing_ms: elapsed,
            vsa_fingerprint: fp,
        })
    }

    /// Convenience: render and distill HTML in one step.
    pub fn render_distilled(&self, url: &str, max_chars: usize) -> Result<RenderedPage, String> {
        let raw = self.render_url(url, WaitUntil::NetworkIdle)?;
        let distilled = BrowserAgent::distil_html(&raw.html, max_chars);
        Ok(RenderedPage {
            html: distilled,
            vsa_fingerprint: compute_vsa_fingerprint(&raw.html, &raw.final_url, raw.timing_ms),
            ..raw
        })
    }
}

/// Compute a 4 x u64 VSA-style content fingerprint from the rendered page.
fn compute_vsa_fingerprint(html: &str, url: &str, timing_ms: u64) -> [u64; 4] {
    use std::hash::{Hash, Hasher};

    let mut state = fxhash();
    html.len().hash(&mut state);
    url.hash(&mut state);
    timing_ms.hash(&mut state);
    let h0 = state.finish();

    let mut state = fxhash();
    for chunk in html.as_bytes().chunks(256).take(4) {
        chunk.hash(&mut state);
    }
    let h1 = state.finish();

    let mut state = fxhash();
    let words: Vec<&str> = html.split_whitespace().collect();
    words.len().hash(&mut state);
    words.iter().take(20).for_each(|w| w.hash(&mut state));
    let h2 = state.finish();

    let mut state = fxhash();
    timing_ms.hash(&mut state);
    url.as_bytes().hash(&mut state);
    html.len().hash(&mut state);
    let h3 = state.finish();

    [h0, h1, h2, h3]
}

fn fxhash() -> std::collections::hash_map::DefaultHasher {
    std::collections::hash_map::DefaultHasher::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Agent;

    fn fixture() -> JsRenderer {
        let agent = Arc::new(BrowserAgent::new("render-test", "ws://localhost:9222"));
        JsRenderer::new(agent, 30_000)
    }

    #[test]
    fn test_render_url() {
        let renderer = fixture();
        let page = renderer
            .render_url("https://example.com", WaitUntil::Load)
            .unwrap();
        assert!(page.html.contains("Mock Page"));
        assert_eq!(page.final_url, "https://example.com");
        assert!(page.timing_ms < 1000);
        assert_eq!(page.vsa_fingerprint.len(), 4);
    }

    #[test]
    fn test_render_with_js() {
        let renderer = fixture();
        let page = renderer
            .render_with_js(
                "https://example.com",
                "document.body.style.backgroundColor = 'red';",
            )
            .unwrap();
        assert_eq!(page.final_url, "https://example.com");
        assert!(!page.html.is_empty());
    }

    #[test]
    fn test_render_distilled() {
        let renderer = fixture();
        let page = renderer
            .render_distilled("https://example.com", 200)
            .unwrap();
        assert!(page.html.len() <= 200);
        assert!(page.html.contains("Mock"));
    }

    #[test]
    fn test_render_url_empty_url() {
        let renderer = fixture();
        let page = renderer
            .render_url("", WaitUntil::DomContentLoaded)
            .unwrap();
        assert_eq!(page.final_url, "");
    }

    #[test]
    fn test_vsa_fingerprint_uniqueness() {
        let fp1 = compute_vsa_fingerprint("<html>hello</html>", "https://a.com", 100);
        let fp2 = compute_vsa_fingerprint("<html>world</html>", "https://b.com", 200);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_vsa_fingerprint_consistency() {
        let fp1 = compute_vsa_fingerprint("<html>test</html>", "https://x.com", 50);
        let fp2 = compute_vsa_fingerprint("<html>test</html>", "https://x.com", 50);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_js_renderer_constructor() {
        let agent = Arc::new(BrowserAgent::new("ctor-test", "ws://localhost:9222"));
        let renderer = JsRenderer::new(agent.clone(), 5000);
        assert_eq!(renderer.js_timeout_ms(), 5000);
        assert_eq!(renderer.browser_agent().name(), "ctor-test");
    }

    #[test]
    fn test_with_timeout() {
        let agent = Arc::new(BrowserAgent::new("timeout-test", "ws://localhost:9222"));
        let renderer = JsRenderer::new(agent, 5000).with_timeout(10000);
        assert_eq!(renderer.js_timeout_ms(), 10000);
    }
}
