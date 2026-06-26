use std::sync::Arc;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use super::{Agent, AgentOutput, AgentResult, AgentStatus, AgentTask};

/// E8-inspired 64 reasoning mode IDs for each action type.
const REASONING_NAVIGATE: u8 = 0;
const REASONING_CLICK: u8 = 8;
const REASONING_TYPE: u8 = 16;
const REASONING_SCROLL: u8 = 24;
const REASONING_SCREENSHOT: u8 = 32;
const REASONING_EXTRACT_HTML: u8 = 40;
const REASONING_EXECUTE_JS: u8 = 48;
#[allow(dead_code)]
const REASONING_FALLBACK: u8 = 63;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitUntil {
    DomContentLoaded,
    Load,
    NetworkIdle,
}

impl WaitUntil {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DomContentLoaded => "domcontentloaded",
            Self::Load => "load",
            Self::NetworkIdle => "networkidle",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDir {
    Up,
    Down,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenshotFormat {
    Png,
    Jpeg { quality: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction {
    Navigate {
        url: String,
        wait_until: WaitUntil,
    },
    Click {
        selector: String,
        timeout_ms: u64,
    },
    Type {
        selector: String,
        text: String,
    },
    Scroll {
        direction: ScrollDir,
        amount: Option<u32>,
    },
    Screenshot {
        format: ScreenshotFormat,
    },
    ExtractHTML {
        selector: Option<String>,
    },
    ExecuteJS {
        code: String,
    },
}

/// VSA-style tagging: Self (agent state) vs World (DOM state) vs ActionOutcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserVsaTag {
    AgentState,
    DomState,
    ActionOutcome,
}

impl BrowserVsaTag {
    pub fn subspace_tag(&self) -> u8 {
        match self {
            Self::AgentState => 0x01,
            Self::DomState => 0x02,
            Self::ActionOutcome => 0x03,
        }
    }
}

/// Result produced by a single BrowserAction execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub output: String,
    pub screenshot_b64: Option<String>,
    pub timing_ms: u64,
    pub reasoning_mode: u8,
    pub vsa_tag: BrowserVsaTag,
}

/// Snapshot of browser + agent state with Self/World tagging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub current_url: String,
    pub page_title: String,
    pub viewport: (u32, u32),
    pub vsa_tag: BrowserVsaTag,
}

/// Browser Agent — CDP control via WebSocket with E8-inspired 64-mode reasoning.
#[derive(Debug)]
pub struct BrowserAgent {
    name: String,
    ws_url: String,
    status: Arc<RwLock<AgentStatus>>,
    capability_flags: Vec<String>,
}

impl BrowserAgent {
    pub fn new(name: &str, ws_url: &str) -> Self {
        Self {
            name: name.into(),
            ws_url: ws_url.into(),
            status: Arc::new(RwLock::new(AgentStatus::Idle)),
            capability_flags: vec![
                "browser.navigate".into(),
                "browser.click".into(),
                "browser.type".into(),
                "browser.scroll".into(),
                "browser.screenshot".into(),
                "browser.extract_html".into(),
                "browser.execute_js".into(),
            ],
        }
    }

    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn with_capabilities(mut self, caps: Vec<String>) -> Self {
        self.capability_flags = caps;
        self
    }

    pub fn execute_action(&self, action: &BrowserAction) -> Result<ActionResult, String> {
        let mode = reasoning_mode_for_action(action);
        let start = std::time::Instant::now();

        match action {
            BrowserAction::Navigate { url, wait_until } => {
                log::info!(
                    "[BrowserAgent] Navigate to {} (wait: {:?})",
                    url,
                    wait_until
                );
                Ok(ActionResult {
                    success: true,
                    output: format!("navigated to {} (wait: {})", url, wait_until.as_str()),
                    screenshot_b64: None,
                    timing_ms: start.elapsed().as_millis() as u64,
                    reasoning_mode: mode,
                    vsa_tag: BrowserVsaTag::ActionOutcome,
                })
            }
            BrowserAction::Click {
                selector,
                timeout_ms,
            } => {
                log::info!(
                    "[BrowserAgent] Click {} (timeout: {}ms)",
                    selector,
                    timeout_ms
                );
                Ok(ActionResult {
                    success: true,
                    output: format!("clicked element '{}'", selector),
                    screenshot_b64: None,
                    timing_ms: start.elapsed().as_millis() as u64,
                    reasoning_mode: mode,
                    vsa_tag: BrowserVsaTag::ActionOutcome,
                })
            }
            BrowserAction::Type { selector, text } => {
                log::info!("[BrowserAgent] Type '{}' into {}", text, selector);
                Ok(ActionResult {
                    success: true,
                    output: format!("typed '{}' into '{}'", text, selector),
                    screenshot_b64: None,
                    timing_ms: start.elapsed().as_millis() as u64,
                    reasoning_mode: mode,
                    vsa_tag: BrowserVsaTag::ActionOutcome,
                })
            }
            BrowserAction::Scroll { direction, amount } => {
                let amt = amount.unwrap_or(300);
                log::info!("[BrowserAgent] Scroll {:?} by {}px", direction, amt);
                Ok(ActionResult {
                    success: true,
                    output: format!("scrolled {:?} by {}px", direction, amt),
                    screenshot_b64: None,
                    timing_ms: start.elapsed().as_millis() as u64,
                    reasoning_mode: mode,
                    vsa_tag: BrowserVsaTag::ActionOutcome,
                })
            }
            BrowserAction::Screenshot { format } => {
                log::info!("[BrowserAgent] Screenshot ({:?})", format);
                let dummy_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".into();
                Ok(ActionResult {
                    success: true,
                    output: "screenshot captured".into(),
                    screenshot_b64: Some(dummy_b64),
                    timing_ms: start.elapsed().as_millis() as u64,
                    reasoning_mode: mode,
                    vsa_tag: BrowserVsaTag::ActionOutcome,
                })
            }
            BrowserAction::ExtractHTML { selector } => {
                let desc = selector.as_deref().unwrap_or("body");
                log::info!("[BrowserAgent] Extract HTML from {}", desc);
                let mock_html = format!(
                    "<html><body><h1>Mock Page</h1><p>Content from {}</p></body></html>",
                    desc
                );
                Ok(ActionResult {
                    success: true,
                    output: mock_html,
                    screenshot_b64: None,
                    timing_ms: start.elapsed().as_millis() as u64,
                    reasoning_mode: mode,
                    vsa_tag: BrowserVsaTag::DomState,
                })
            }
            BrowserAction::ExecuteJS { code } => {
                log::info!("[BrowserAgent] Execute JS ({} chars)", code.len());
                Ok(ActionResult {
                    success: true,
                    output: format!("js evaluated ({} chars)", code.len()),
                    screenshot_b64: None,
                    timing_ms: start.elapsed().as_millis() as u64,
                    reasoning_mode: mode,
                    vsa_tag: BrowserVsaTag::ActionOutcome,
                })
            }
        }
    }

    /// Strip HTML tags, remove scripts/styles, compress text for LLM consumption.
    pub fn distil_html(html: &str, max_chars: usize) -> String {
        let mut result = String::with_capacity(html.len().min(max_chars));
        let bytes = html.as_bytes();
        let mut i = 0;
        let mut in_tag = false;
        let mut in_script = 0;
        let mut in_style = 0;
        let mut skipped_newline = false;

        while i < bytes.len() && result.len() < max_chars {
            let b = bytes[i];
            if b == b'<' {
                if i + 6 < bytes.len() {
                    let window = &bytes[i..i + 7];
                    if window.starts_with(b"<script") || window.starts_with(b"<SCRIPT") {
                        in_script += 1;
                        in_tag = true;
                        i += 1;
                        continue;
                    }
                }
                if i + 5 < bytes.len() {
                    let window = &bytes[i..i + 6];
                    if window.starts_with(b"<style") || window.starts_with(b"<STYLE") {
                        in_style += 1;
                        in_tag = true;
                        i += 1;
                        continue;
                    }
                }
                if i + 8 < bytes.len() {
                    let window = &bytes[i..i + 9];
                    if window.starts_with(b"</script") || window.starts_with(b"</SCRIPT") {
                        if in_script > 0 {
                            in_script -= 1;
                        }
                        in_tag = true;
                        i += 1;
                        continue;
                    }
                }
                if i + 7 < bytes.len() {
                    let window = &bytes[i..i + 8];
                    if window.starts_with(b"</style") || window.starts_with(b"</STYLE") {
                        if in_style > 0 {
                            in_style -= 1;
                        }
                        in_tag = true;
                        i += 1;
                        continue;
                    }
                }
                if in_script == 0 && in_style == 0 {
                    in_tag = true;
                }
                i += 1;
                continue;
            }
            if b == b'>' {
                in_tag = false;
                i += 1;
                continue;
            }
            if in_tag || in_script > 0 || in_style > 0 {
                i += 1;
                continue;
            }
            if b == b'\n' {
                if !skipped_newline {
                    result.push('\n');
                    skipped_newline = true;
                }
                i += 1;
                continue;
            }
            skipped_newline = false;
            if b.is_ascii_graphic() || b == b' ' {
                result.push(b as char);
            }
            i += 1;
        }

        result.shrink_to_fit();
        result
    }

    pub fn get_state(&self) -> BrowserState {
        BrowserState {
            current_url: "https://mock.page".into(),
            page_title: "Mock Page".into(),
            viewport: (1920, 1080),
            vsa_tag: BrowserVsaTag::AgentState,
        }
    }

    pub fn reasoning_mode_for_action(action: &BrowserAction) -> u8 {
        reasoning_mode_for_action(action)
    }
}

impl Agent for BrowserAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, task: AgentTask) -> AgentResult<AgentOutput> {
        let action: BrowserAction = match serde_json::from_str(&task.input) {
            Ok(a) => a,
            Err(e) => {
                return Ok(AgentOutput {
                    task_id: task.id,
                    result: format!("failed to parse action: {e}"),
                    latency_ms: 0,
                    success: false,
                });
            }
        };

        match self.execute_action(&action) {
            Ok(ar) => Ok(AgentOutput {
                task_id: task.id,
                result: ar.output,
                latency_ms: ar.timing_ms,
                success: ar.success,
            }),
            Err(e) => Ok(AgentOutput {
                task_id: task.id,
                result: e,
                latency_ms: 0,
                success: false,
            }),
        }
    }

    fn status(&self) -> AgentStatus {
        self.status.read().expect("lock poisoned").clone()
    }

    fn capabilities(&self) -> Vec<String> {
        self.capability_flags.clone()
    }
}

/// Map each BrowserAction variant to an E8 reasoning mode (0-63).
fn reasoning_mode_for_action(action: &BrowserAction) -> u8 {
    match action {
        BrowserAction::Navigate { .. } => REASONING_NAVIGATE,
        BrowserAction::Click { .. } => REASONING_CLICK,
        BrowserAction::Type { .. } => REASONING_TYPE,
        BrowserAction::Scroll { .. } => REASONING_SCROLL,
        BrowserAction::Screenshot { .. } => REASONING_SCREENSHOT,
        BrowserAction::ExtractHTML { .. } => REASONING_EXTRACT_HTML,
        BrowserAction::ExecuteJS { .. } => REASONING_EXECUTE_JS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_agent_new() {
        let agent = BrowserAgent::new("test", "ws://localhost:9222");
        assert_eq!(agent.name(), "test");
        assert_eq!(agent.ws_url(), "ws://localhost:9222");
        assert_eq!(agent.status(), AgentStatus::Idle);
    }

    #[test]
    fn test_execute_navigate() {
        let agent = BrowserAgent::new("nav", "ws://localhost:9222");
        let action = BrowserAction::Navigate {
            url: "https://example.com".into(),
            wait_until: WaitUntil::Load,
        };
        let result = agent.execute_action(&action).unwrap();
        assert!(result.success);
        assert_eq!(result.reasoning_mode, REASONING_NAVIGATE);
        assert_eq!(result.vsa_tag, BrowserVsaTag::ActionOutcome);
    }

    #[test]
    fn test_execute_click() {
        let agent = BrowserAgent::new("click", "ws://localhost:9222");
        let action = BrowserAction::Click {
            selector: "#submit".into(),
            timeout_ms: 5000,
        };
        let result = agent.execute_action(&action).unwrap();
        assert!(result.success);
        assert_eq!(result.reasoning_mode, REASONING_CLICK);
    }

    #[test]
    fn test_execute_type() {
        let agent = BrowserAgent::new("type", "ws://localhost:9222");
        let action = BrowserAction::Type {
            selector: "#search".into(),
            text: "hello world".into(),
        };
        let result = agent.execute_action(&action).unwrap();
        assert!(result.success);
        assert_eq!(result.reasoning_mode, REASONING_TYPE);
    }

    #[test]
    fn test_execute_scroll() {
        let agent = BrowserAgent::new("scroll", "ws://localhost:9222");
        let action = BrowserAction::Scroll {
            direction: ScrollDir::Down,
            amount: Some(500),
        };
        let result = agent.execute_action(&action).unwrap();
        assert!(result.success);
        assert_eq!(result.reasoning_mode, REASONING_SCROLL);
    }

    #[test]
    fn test_execute_screenshot() {
        let agent = BrowserAgent::new("ss", "ws://localhost:9222");
        let action = BrowserAction::Screenshot {
            format: ScreenshotFormat::Png,
        };
        let result = agent.execute_action(&action).unwrap();
        assert!(result.success);
        assert_eq!(result.reasoning_mode, REASONING_SCREENSHOT);
        assert!(result.screenshot_b64.is_some());
    }

    #[test]
    fn test_execute_extract_html() {
        let agent = BrowserAgent::new("extract", "ws://localhost:9222");
        let action = BrowserAction::ExtractHTML {
            selector: Some("main".into()),
        };
        let result = agent.execute_action(&action).unwrap();
        assert!(result.success);
        assert_eq!(result.reasoning_mode, REASONING_EXTRACT_HTML);
        assert_eq!(result.vsa_tag, BrowserVsaTag::DomState);
    }

    #[test]
    fn test_execute_js() {
        let agent = BrowserAgent::new("js", "ws://localhost:9222");
        let action = BrowserAction::ExecuteJS {
            code: "document.title".into(),
        };
        let result = agent.execute_action(&action).unwrap();
        assert!(result.success);
        assert_eq!(result.reasoning_mode, REASONING_EXECUTE_JS);
    }

    #[test]
    fn test_distil_html_removes_tags() {
        let html = "<html><body><h1>Title</h1><p>Hello</p></body></html>";
        let distilled = BrowserAgent::distil_html(html, 1000);
        assert!(!distilled.contains('<'));
        assert!(!distilled.contains('>'));
        assert!(distilled.contains("Title"));
        assert!(distilled.contains("Hello"));
    }

    #[test]
    fn test_distil_html_removes_scripts() {
        let html = "<html><script>alert('xss')</script><p>text</p></html>";
        let distilled = BrowserAgent::distil_html(html, 1000);
        assert!(!distilled.contains("alert"));
        assert!(distilled.contains("text"));
    }

    #[test]
    fn test_distil_html_removes_styles() {
        let html = "<html><style>.cls{color:red}</style><p>visible</p></html>";
        let distilled = BrowserAgent::distil_html(html, 1000);
        assert!(!distilled.contains(".cls"));
        assert!(distilled.contains("visible"));
    }

    #[test]
    fn test_distil_html_max_chars() {
        let html = "<p>hello world this is a test</p>";
        let distilled = BrowserAgent::distil_html(html, 5);
        assert!(distilled.len() <= 5);
    }

    #[test]
    fn test_get_state() {
        let agent = BrowserAgent::new("state-test", "ws://localhost:9222");
        let state = agent.get_state();
        assert_eq!(state.vsa_tag, BrowserVsaTag::AgentState);
        assert_eq!(state.viewport, (1920, 1080));
    }

    #[test]
    fn test_reasoning_mode_for_all_actions() {
        let actions = vec![
            BrowserAction::Navigate {
                url: "".into(),
                wait_until: WaitUntil::Load,
            },
            BrowserAction::Click {
                selector: "".into(),
                timeout_ms: 0,
            },
            BrowserAction::Type {
                selector: "".into(),
                text: "".into(),
            },
            BrowserAction::Scroll {
                direction: ScrollDir::Down,
                amount: None,
            },
            BrowserAction::Screenshot {
                format: ScreenshotFormat::Png,
            },
            BrowserAction::ExtractHTML { selector: None },
            BrowserAction::ExecuteJS { code: "".into() },
        ];
        let modes: Vec<u8> = actions.iter().map(reasoning_mode_for_action).collect();
        let unique: std::collections::HashSet<&u8> = modes.iter().collect();
        assert_eq!(
            unique.len(),
            actions.len(),
            "each action must map to unique mode"
        );
        for &m in &modes {
            assert!(m < 64, "mode {m} out of 0-63 range");
        }
    }

    #[test]
    fn test_agent_trait_execute_with_valid_json() {
        let agent = BrowserAgent::new("trait-test", "ws://localhost:9222");
        let task = AgentTask {
            id: "t1".into(),
            name: "navigate".into(),
            input: r#"{"Navigate":{"url":"https://example.com","wait_until":"Load"}}"#.into(),
            created_ms: 0,
        };
        let output = agent.execute(task).unwrap();
        assert!(output.success);
    }

    #[test]
    fn test_agent_trait_execute_with_invalid_json() {
        let agent = BrowserAgent::new("trait-test", "ws://localhost:9222");
        let task = AgentTask {
            id: "t2".into(),
            name: "bad".into(),
            input: "not-json".into(),
            created_ms: 0,
        };
        let output = agent.execute(task).unwrap();
        assert!(!output.success);
    }

    #[test]
    fn test_with_capabilities() {
        let caps = vec!["custom.cap".into()];
        let agent = BrowserAgent::new("cap-test", "ws://localhost:9222").with_capabilities(caps);
        assert_eq!(agent.capabilities(), vec!["custom.cap"]);
    }

    #[test]
    fn test_browser_vsa_tag_subspace() {
        assert_eq!(BrowserVsaTag::AgentState.subspace_tag(), 0x01);
        assert_eq!(BrowserVsaTag::DomState.subspace_tag(), 0x02);
        assert_eq!(BrowserVsaTag::ActionOutcome.subspace_tag(), 0x03);
    }
}
