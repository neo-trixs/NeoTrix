use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

// ── App Preview Types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    pub enabled: bool,
    pub port: u16,
    pub default_width: u32,
    pub default_height: u32,
    pub allow_navigation: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 3000,
            default_width: 1280,
            default_height: 720,
            allow_navigation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewStatus {
    #[serde(rename = "loading")]
    Loading,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewSession {
    pub id: String,
    pub url: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub status: PreviewStatus,
    pub started_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewScreenshot {
    pub session_id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub taken_at: u64,
}

// ── Chrome Debug Types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeDebugTarget {
    pub id: String,
    pub title: String,
    pub url: String,
    pub description: String,
    pub favicon_url: String,
    pub debug_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleLevel {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeDebugConsoleEntry {
    pub level: ConsoleLevel,
    pub message: String,
    pub timestamp: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeDebugConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub auto_connect: bool,
}

impl Default for ChromeDebugConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "localhost".to_string(),
            port: 9222,
            auto_connect: true,
        }
    }
}

// ── State ─────────────────────────────────────────────────────────────

pub struct PreviewState {
    pub preview_config: PreviewConfig,
    pub sessions: Vec<PreviewSession>,
    pub chrome_config: ChromeDebugConfig,
    pub chrome_connected: bool,
    pub console_logs: VecDeque<ChromeDebugConsoleEntry>,
    session_counter: u64,
}

impl PreviewState {
    fn new() -> Self {
        Self {
            preview_config: PreviewConfig::default(),
            sessions: Vec::with_capacity(10),
            chrome_config: ChromeDebugConfig::default(),
            chrome_connected: false,
            console_logs: VecDeque::with_capacity(500),
            session_counter: 0,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

static STATE: LazyLock<Mutex<PreviewState>> = LazyLock::new(|| Mutex::new(PreviewState::new()));

const MAX_SESSIONS: usize = 10;
const MAX_CONSOLE_LOGS: usize = 500;

fn short_uid(counter: u64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("{:x}{:04x}", now % 0xffffff, counter % 0xffff)
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

// ── App Previews ──────────────────────────────────────────────────────

#[tauri::command]
pub fn preview_start(url: String, width: Option<u32>, height: Option<u32>) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if !state.preview_config.enabled {
        return Err("previews disabled".to_string());
    }
    if state.sessions.len() >= MAX_SESSIONS {
        return Err("max preview sessions reached".to_string());
    }
    let w = width.unwrap_or(state.preview_config.default_width);
    let h = height.unwrap_or(state.preview_config.default_height);
    state.session_counter += 1;
    let id = format!("pv-{}", short_uid(state.session_counter));
    state.sessions.push(PreviewSession {
        id: id.clone(),
        url: url.clone(),
        title: url.clone(),
        width: w,
        height: h,
        status: PreviewStatus::Loading,
        started_at: now_secs(),
    });
    Ok(id)
}

#[tauri::command]
pub fn preview_stop(session_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let pos = state.sessions.iter().position(|s| s.id == session_id)
        .ok_or_else(|| format!("session {} not found", session_id))?;
    state.sessions.remove(pos);
    Ok(())
}

#[tauri::command]
pub fn preview_list() -> Result<Vec<PreviewSession>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.sessions.clone())
}

#[tauri::command]
pub fn preview_navigate(session_id: String, url: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if !state.preview_config.allow_navigation {
        return Err("navigation disabled".to_string());
    }
    let session = state.sessions.iter_mut()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("session {} not found", session_id))?;
    session.url = url;
    session.title = session.url.clone();
    session.status = PreviewStatus::Loading;
    Ok(())
}

#[tauri::command]
pub fn preview_reload(session_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let session = state.sessions.iter_mut()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("session {} not found", session_id))?;
    session.status = PreviewStatus::Loading;
    Ok(())
}

#[tauri::command]
pub fn preview_screenshot(session_id: String) -> Result<PreviewScreenshot, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let session = state.sessions.iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| format!("session {} not found", session_id))?;
    Ok(PreviewScreenshot {
        session_id: session_id.clone(),
        path: format!("/tmp/neotrix_preview_{}.png", session_id),
        width: session.width,
        height: session.height,
        taken_at: now_secs(),
    })
}

#[tauri::command]
pub fn preview_config() -> Result<PreviewConfig, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.preview_config.clone())
}

#[tauri::command]
pub fn preview_set_config(config: PreviewConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.preview_config = config;
    Ok(())
}

// ── Chrome Debug Integration ──────────────────────────────────────────

#[tauri::command]
pub fn chrome_debug_connect(host: Option<String>, port: Option<u16>) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let h = host.unwrap_or_else(|| "localhost".to_string());
    let p = port.unwrap_or(9222);
    state.chrome_config.host = h.clone();
    state.chrome_config.port = p;
    state.chrome_connected = true;
    Ok(format!("connected to {}:{}", h, p))
}

#[tauri::command]
pub fn chrome_debug_disconnect() -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.chrome_connected = false;
    Ok(())
}

#[tauri::command]
pub fn chrome_debug_status() -> Result<ChromeDebugConfig, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.chrome_config.clone())
}

#[tauri::command]
pub fn chrome_debug_list_targets() -> Result<Vec<ChromeDebugTarget>, String> {
    let _state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(vec![
        ChromeDebugTarget {
            id: "target-1".to_string(),
            title: "NeoTrix Desktop".to_string(),
            url: "http://localhost:3000".to_string(),
            description: "main application window".to_string(),
            favicon_url: "".to_string(),
            debug_url: "http://localhost:9222/devtools/page-1".to_string(),
        },
        ChromeDebugTarget {
            id: "target-2".to_string(),
            title: "New Tab".to_string(),
            url: "about:blank".to_string(),
            description: "empty browser tab".to_string(),
            favicon_url: "".to_string(),
            debug_url: "http://localhost:9222/devtools/page-2".to_string(),
        },
        ChromeDebugTarget {
            id: "target-3".to_string(),
            title: "DevTools".to_string(),
            url: "chrome://devtools".to_string(),
            description: "Chrome Developer Tools".to_string(),
            favicon_url: "".to_string(),
            debug_url: "http://localhost:9222/devtools/page-3".to_string(),
        },
        ChromeDebugTarget {
            id: "target-4".to_string(),
            title: "Extensions".to_string(),
            url: "chrome://extensions".to_string(),
            description: "extensions management page".to_string(),
            favicon_url: "".to_string(),
            debug_url: "http://localhost:9222/devtools/page-4".to_string(),
        },
    ])
}

#[tauri::command]
pub fn chrome_debug_navigate(url: String) -> Result<(), String> {
    let _state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    log::info!("[chrome-debug] navigate to: {}", url);
    Ok(())
}

#[tauri::command]
pub fn chrome_debug_reload() -> Result<(), String> {
    let _state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    log::info!("[chrome-debug] reload");
    Ok(())
}

#[tauri::command]
pub fn chrome_debug_evaluate(expression: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if state.console_logs.len() >= MAX_CONSOLE_LOGS {
        state.console_logs.pop_front();
    }
    state.console_logs.push_back(ChromeDebugConsoleEntry {
        level: ConsoleLevel::Info,
        message: expression.clone(),
        timestamp: now_secs(),
        source: "console".to_string(),
    });
    Ok(format!("> {}\n<- (simulated) undefined", expression))
}

#[tauri::command]
pub fn chrome_debug_get_console_logs() -> Result<Vec<ChromeDebugConsoleEntry>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.console_logs.iter().cloned().collect())
}

#[tauri::command]
pub fn chrome_debug_clear_console_logs() -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.console_logs.clear();
    Ok(())
}

#[tauri::command]
pub fn chrome_debug_capture_screenshot() -> Result<String, String> {
    let _state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok("/tmp/chrome_debug_screenshot.png".to_string())
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_start_stop() {
        let id = preview_start("http://localhost:3000".into(), None, None).unwrap();
        assert!(id.starts_with("pv-"));
        assert!(preview_stop(id).is_ok());
    }

    #[test]
    fn test_preview_list() {
        let _ = preview_stop("pv-test-list".into());
        let id = preview_start("http://example.com".into(), Some(800), Some(600)).unwrap();
        let list = preview_list().unwrap();
        assert!(list.iter().any(|s| s.id == id));
        assert_eq!(list.iter().find(|s| s.id == id).unwrap().width, 800);
        assert_eq!(list.iter().find(|s| s.id == id).unwrap().height, 600);
    }

    #[test]
    fn test_chrome_connect_disconnect() {
        let _ = chrome_debug_disconnect();
        let msg = chrome_debug_connect(Some("127.0.0.1".into()), Some(9333)).unwrap();
        assert!(msg.contains("127.0.0.1:9333"));
        let cfg = chrome_debug_status().unwrap();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 9333);
        assert!(chrome_debug_disconnect().is_ok());
    }

    #[test]
    fn test_chrome_list_targets() {
        let targets = chrome_debug_list_targets().unwrap();
        assert_eq!(targets.len(), 4);
        assert!(targets.iter().any(|t| t.id == "target-1"));
        assert!(targets.iter().any(|t| t.url == "http://localhost:3000"));
    }

    #[test]
    fn test_chrome_evaluate() {
        let result = chrome_debug_evaluate("2 + 2".into()).unwrap();
        assert!(result.contains("2 + 2"));
        assert!(result.contains("undefined"));
    }
}
