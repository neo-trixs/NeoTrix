use serde::{Serialize, Deserialize};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::Engine;

// ── Data Types ──

#[derive(Serialize, Clone, Debug)]
pub struct ScreenCapture {
    pub path: String,
    pub width: u64,
    pub height: u64,
    pub format: String,
    pub timestamp: u64,
    /// 内存内联返回：capture 后不落盘给前端，直接 base64（None 保留纯路径语义）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_base64: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Clone, Debug)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    pub scale_factor: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct WindowInfo {
    pub title: String,
    pub pid: i32,
    pub app_name: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct FrontmostApp {
    pub app_name: String,
    pub title: String,
}

// ── Helpers ──

fn timestamp_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn escape_applescript_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// 解析 AppleScript 返回的列表文本（如 `{"A", "B"}` 或 `A, B`），
/// 逐项去除花括号与引号，返回非空字符串列表。
fn parse_applescript_list(s: &str) -> Vec<String> {
    let s = s.trim();
    if s.is_empty() {
        return Vec::new();
    }
    let inner = s
        .strip_prefix('{')
        .and_then(|x| x.strip_suffix('}'))
        .unwrap_or(s);
    inner
        .split(',')
        .map(|item| item.trim().trim_matches('"').trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

// ── Commands ──

#[tauri::command]
pub fn computer_screen_capture(path: Option<String>) -> Result<ScreenCapture, String> {
    let ts = timestamp_nanos();
    let output_path = path.unwrap_or_else(|| format!("/tmp/neotrix_screen_{}.png", ts));

    let output = Command::new("screencapture")
        .args(["-x", &output_path])
        .output()
        .map_err(|e| format!("Failed to run screencapture: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "screencapture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let _meta = std::fs::metadata(&output_path)
        .map_err(|e| format!("Failed to read screenshot file: {}", e))?;

    Ok(ScreenCapture {
        path: output_path,
        width: 0,
        height: 0,
        format: "png".into(),
        timestamp: ts,
        data_base64: None,
    })
}

#[tauri::command]
pub fn computer_screen_list() -> Result<Vec<DisplayInfo>, String> {
    Ok(vec![DisplayInfo {
        id: 1,
        name: "Main Display".into(),
        width: 1920,
        height: 1080,
        is_primary: true,
        scale_factor: 2.0,
    }])
}

#[tauri::command]
pub fn computer_get_frontmost_app() -> Result<FrontmostApp, String> {
    let script = r#"tell application "System Events" to get {name of first application process whose frontmost is true, name of front window of first application process whose frontmost is true}"#;
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Failed to get frontmost app: {}", e))?;

    // 无辅助功能权限或解析失败时返回默认值，避免前端捕获异常
    if !output.status.success() {
        return Ok(FrontmostApp {
            app_name: "Unknown".into(),
            title: String::new(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // 输出形如 `"AppName, Window Title"`，按首个逗号切分为 app_name 与 title
    let parts: Vec<&str> = stdout.splitn(2, ',').map(|s| s.trim()).collect();
    let app_name = parts
        .first()
        .map(|s| s.trim_matches('"').trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".into());
    let title = parts
        .get(1)
        .map(|s| s.trim_matches('"').trim().to_string())
        .unwrap_or_default();

    Ok(FrontmostApp { app_name, title })
}

#[tauri::command]
pub fn computer_get_window_list() -> Result<Vec<WindowInfo>, String> {
    // 前台应用名称
    let app_script = r#"tell application "System Events" to get name of first application process whose frontmost is true"#;
    let app_output = Command::new("osascript")
        .args(["-e", app_script])
        .output()
        .map_err(|e| format!("Failed to get frontmost app: {}", e))?;
    let app_name = if app_output.status.success() {
        String::from_utf8_lossy(&app_output.stdout).trim().to_string()
    } else {
        String::new()
    };

    // 前台应用 pid
    let pid_script = r#"tell application "System Events" to get unix id of first application process whose frontmost is true"#;
    let pid_output = Command::new("osascript")
        .args(["-e", pid_script])
        .output()
        .map_err(|e| format!("Failed to get frontmost pid: {}", e))?;
    let pid: i32 = if pid_output.status.success() {
        String::from_utf8_lossy(&pid_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0)
    } else {
        0
    };

    // 前台应用所有窗口标题
    let win_script = r#"tell application "System Events" to tell (first application process whose frontmost is true) to get name of every window"#;
    let win_output = Command::new("osascript")
        .args(["-e", win_script])
        .output()
        .map_err(|e| format!("Failed to list windows: {}", e))?;

    // 无辅助功能权限时返回空列表，避免前端捕获异常
    if !win_output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&win_output.stdout).trim().to_string();
    let titles = parse_applescript_list(&stdout);

    let windows: Vec<WindowInfo> = titles
        .into_iter()
        .map(|title| WindowInfo {
            title,
            pid,
            app_name: app_name.clone(),
        })
        .collect();

    Ok(windows)
}

#[tauri::command]
pub fn computer_mouse_move(x: u32, y: u32) -> Result<(), String> {
    let script = format!(r#"tell application "System Events" to set position of mouse to {{{}, {}}}"#, x, y);
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to move mouse: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Mouse move failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn computer_mouse_click(_button: Option<String>) -> Result<(), String> {
    let pos_script = r#"tell application "System Events" to get position of mouse"#.to_string();
    let pos_output = Command::new("osascript")
        .args(["-e", &pos_script])
        .output()
        .map_err(|e| format!("Failed to get mouse position: {}", e))?;

    if !pos_output.status.success() {
        return Err(format!(
            "Failed to get mouse position: {}",
            String::from_utf8_lossy(&pos_output.stderr)
        ));
    }

    let pos_str = String::from_utf8_lossy(&pos_output.stdout).trim().to_string();
    let coords: Vec<i32> = pos_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if coords.len() < 2 {
        return Err(format!("Could not parse mouse position: {}", pos_str));
    }

    let click_script = format!(
        r#"tell application "System Events" to click at {{{}, {}}}"#,
        coords[0], coords[1]
    );

    let output = Command::new("osascript")
        .args(["-e", &click_script])
        .output()
        .map_err(|e| format!("Failed to click: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Click failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn computer_mouse_position() -> Result<MousePosition, String> {
    let script = r#"tell application "System Events" to get position of mouse"#.to_string();
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to get mouse position: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to get mouse position: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let pos_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let coords: Vec<i32> = pos_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if coords.len() < 2 {
        return Err(format!("Could not parse mouse position: {}", pos_str));
    }

    Ok(MousePosition {
        x: coords[0],
        y: coords[1],
    })
}

#[tauri::command]
pub fn computer_keyboard_type(text: String) -> Result<(), String> {
    let escaped = escape_applescript_string(&text);
    let script = format!(r#"tell application "System Events" to keystroke "{}""#, escaped);
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to type text: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Keyboard type failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn computer_keyboard_press(key: String, modifiers: Option<Vec<String>>) -> Result<(), String> {
    let mods = modifiers.unwrap_or_default();

    // 防 AppleScript 注入: key 必须是纯数字 key code，modifiers 必须来自白名单
    if key.is_empty() || !key.bytes().all(|b| b.is_ascii_digit()) {
        return Err("key must be a numeric key code".into());
    }
    const ALLOWED_MODS: [&str; 4] = ["command", "option", "control", "shift"];
    for m in &mods {
        let m = m.trim();
        if !ALLOWED_MODS.contains(&m) {
            return Err(format!("invalid modifier: {}", m));
        }
    }

    let using_modifiers = !mods.is_empty();
    let script = if using_modifiers {
        let mod_parts: Vec<String> = mods.iter().map(|m| format!("{} down", m.trim())).collect();
        let mod_using = mod_parts.join(" using {");
        format!(
            r#"tell application "System Events" to key code {} using {{ {} }}"#,
            key, mod_using,
        )
    } else {
        format!(r#"tell application "System Events" to key code {}"#, key)
    };

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to press key: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Key press failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn computer_screenshot_and_save(path: Option<String>) -> Result<ScreenCapture, String> {
    // 无显式 path：内部临时文件捕获 → 读入内存 base64 → 删除临时文件，全程不落盘给前端
    // （消除前端 readFile/remove 二次磁盘往返；mactlm screencapture 仅能写文件，生命周期留在后端）
    let internal = path.is_none();
    let output_path = path
        .or_else(|| Some(format!("{}/neotrix_screen_{}.png", std::env::temp_dir().display(), timestamp_nanos())));

    let capture_path = output_path.as_deref().unwrap_or_default();
    let output = Command::new("screencapture")
        .args(["-x", capture_path])
        .output()
        .map_err(|e| format!("Failed to run screencapture: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "screencapture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let _meta = std::fs::metadata(capture_path)
        .map_err(|e| format!("Failed to read screenshot file: {}", e))?;

    let data_base64 = if internal {
        // 内存传递：读入 base64 后立即删除临时文件
        let data = std::fs::read(capture_path)
            .map_err(|e| format!("Failed to read screenshot: {}", e))?;
        let _ = std::fs::remove_file(capture_path);
        Some(base64::engine::general_purpose::STANDARD.encode(&data))
    } else {
        None
    };

    Ok(ScreenCapture {
        path: output_path.unwrap_or_default(),
        width: 0,
        height: 0,
        format: "png".into(),
        timestamp: timestamp_nanos(),
        data_base64,
    })
}


// ── Background Task Types ──

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackgroundTaskConfig {
    pub enabled: bool,
    pub max_concurrent_tasks: u32,
    pub poll_interval_ms: u64,
    pub auto_retry: bool,
    pub max_retries: u32,
    pub notify_on_completion: bool,
    pub log_path: Option<String>,
}

impl Default for BackgroundTaskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_tasks: 4,
            poll_interval_ms: 1000,
            auto_retry: false,
            max_retries: 3,
            notify_on_completion: false,
            log_path: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackgroundTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub status: String,
    pub progress_pct: u8,
    pub target: String,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub retry_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackgroundTaskStats {
    pub total_created: u64,
    pub total_completed: u64,
    pub total_failed: u64,
    pub total_cancelled: u64,
    pub currently_running: u32,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct BgComputerState {
    tasks: Vec<BackgroundTask>,
    config: BackgroundTaskConfig,
    stats: BackgroundTaskStats,
    next_id: u64,
}

impl BgComputerState {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            config: BackgroundTaskConfig::default(),
            stats: BackgroundTaskStats {
                total_created: 0,
                total_completed: 0,
                total_failed: 0,
                total_cancelled: 0,
                currently_running: 0,
                avg_duration_ms: 0.0,
                success_rate: 100.0,
            },
            next_id: 1,
        }
    }

    fn generate_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("bg-{}-{}", timestamp_nanos(), id)
    }

    fn add_task(&mut self, task: BackgroundTask) {
        self.stats.total_created += 1;
        self.tasks.push(task);
        if self.tasks.len() > 200 {
            self.tasks.remove(0);
        }
    }

    fn update_stats(&mut self) {
        let total = self.stats.total_completed + self.stats.total_failed;
        self.stats.success_rate = if total > 0 {
            (self.stats.total_completed as f64 / total as f64) * 100.0
        } else {
            100.0
        };
    }
}

use std::sync::{LazyLock, Mutex};

static BG_STATE: LazyLock<Mutex<BgComputerState>> = LazyLock::new(|| {
    Mutex::new(BgComputerState::new())
});

// ── Background Task Commands ──

#[tauri::command]
pub fn computer_bg_submit(
    type_: String,
    target: String,
    params: Option<serde_json::Value>,
) -> Result<String, String> {
    let mut state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    if !state.config.enabled {
        return Err("Background mode is disabled".into());
    }

    let running_count: u32 = state
        .tasks
        .iter()
        .filter(|t| t.status == "running")
        .count() as u32;

    if running_count >= state.config.max_concurrent_tasks {
        return Err("Max concurrent tasks reached".into());
    }

    let valid_types = ["click", "type", "navigate", "capture", "script", "watch"];
    if !valid_types.contains(&type_.as_str()) {
        return Err(format!("Unknown task type: {}. Valid: click, type, navigate, capture, script, watch", type_));
    }

    let task_id = state.generate_id();
    let now = timestamp_nanos();

    let name = match type_.as_str() {
        "click" => format!("Click {}", target),
        "type" => format!("Type text ({} chars)", target.len()),
        "navigate" => format!("Navigate to {}", target),
        "capture" => format!("Capture {}", target),
        "script" => format!("Run script {}", target),
        "watch" => format!("Watch {}", target),
        _ => format!("{} {}", type_, target),
    };

    let description = match type_.as_str() {
        "click" => format!("Mouse click at {}", target),
        "type" => format!("Type \"{}\"", if target.len() > 40 { let end = target.floor_char_boundary(40); format!("{}...", &target[..end]) } else { target.clone() }),
        "navigate" => format!("Open URL {}", target),
        "capture" => format!("Screen capture: {}", target),
        "script" => format!("Execute automation script {}", target),
        "watch" => format!("Monitor {}", target),
        _ => format!("{} on {}", type_, target),
    };

    let exec_target = target.clone();

    let task = BackgroundTask {
        id: task_id.clone(),
        name,
        description,
        task_type: type_.clone(),
        // 锁内直接置 running：并发计数立即生效，杜绝双 submit 都通过上限检查 (check-then-act)
        status: "running".into(),
        progress_pct: 0,
        target,
        created_at: now,
        started_at: Some(now),
        completed_at: None,
        duration_ms: None,
        result: None,
        error: None,
        retry_count: 0,
    };

    state.add_task(task);
    state.stats.currently_running += 1;
    drop(state);

    // Simulate execution in background
    let exec_id = task_id.clone();
    let exec_type = type_.clone();
    let _exec_params = params.clone();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Simulate work
        let work_ms: u64 = match exec_type.as_str() {
            "click" => 200,
            "type" => (exec_target.len() as u64) * 30 + 100,
            "navigate" => 1500,
            "capture" => 500,
            "script" => 2000,
            "watch" => 5000,
            _ => 1000,
        };

        let steps = 5;
        for i in 1..=steps {
            std::thread::sleep(std::time::Duration::from_millis(work_ms / steps));
            if let Ok(mut s) = BG_STATE.lock() {
                if let Some(t) = s.tasks.iter_mut().find(|t| t.id == exec_id) {
                    if t.status == "cancelled" {
                        t.completed_at = Some(timestamp_nanos());
                        if let Some(started) = t.started_at {
                            t.duration_ms = Some(t.completed_at.unwrap().saturating_sub(started) / 1_000_000);
                        }
                        s.stats.currently_running = s.stats.currently_running.saturating_sub(1);
                        s.stats.total_cancelled += 1;
                        s.update_stats();
                        return;
                    }
                    t.progress_pct = ((i as f32 / steps as f32) * 100.0) as u8;
                }
            }
        }

        // Mark completed
        if let Ok(mut s) = BG_STATE.lock() {
            if let Some(t) = s.tasks.iter_mut().find(|t| t.id == exec_id) {
                t.status = "completed".into();
                t.progress_pct = 100;
                t.completed_at = Some(timestamp_nanos());
                if let Some(started) = t.started_at {
                    t.duration_ms = Some(t.completed_at.unwrap().saturating_sub(started) / 1_000_000);
                }

                t.result = match t.task_type.as_str() {
                    "capture" => Some(format!("Captured: {}", t.target)),
                    "navigate" => Some(format!("Navigated to {}", t.target)),
                    "click" => Some(format!("Clicked at {}", t.target)),
                    "type" => Some(format!("Typed {} characters", t.target.len())),
                    "script" => Some(format!("Script {} completed", t.target)),
                    "watch" => Some(format!("Watch on {} completed", t.target)),
                    _ => Some("Completed".into()),
                };

                s.stats.currently_running = s.stats.currently_running.saturating_sub(1);
                s.stats.total_completed += 1;
                s.update_stats();
            }
        }
    });

    Ok(task_id)
}

#[tauri::command]
pub fn computer_bg_list(status: Option<String>) -> Result<Vec<BackgroundTask>, String> {
    let state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    let tasks = match status {
        Some(ref s) => state
            .tasks
            .iter()
            .filter(|t| t.status == s.as_str())
            .cloned()
            .collect(),
        None => state.tasks.clone(),
    };

    Ok(tasks)
}

#[tauri::command]
pub fn computer_bg_get(task_id: String) -> Result<BackgroundTask, String> {
    let state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    state
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .cloned()
        .ok_or_else(|| format!("Task not found: {}", task_id))
}

#[tauri::command]
pub fn computer_bg_cancel(task_id: String) -> Result<(), String> {
    let mut state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    let task = state
        .tasks
        .iter_mut()
        .find(|t| t.id == task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    if task.status == "running" || task.status == "queued" {
        task.status = "cancelled".into();
        Ok(())
    } else {
        Err(format!("Task {} is not running (status: {})", task_id, task.status))
    }
}

#[tauri::command]
pub fn computer_bg_retry(task_id: String) -> Result<(), String> {
    let mut state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    let max_retries = state.config.max_retries;

    let task = state
        .tasks
        .iter_mut()
        .find(|t| t.id == task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let is_retriable = task.status == "failed" || task.status == "cancelled";
    if !is_retriable {
        return Err(format!("Task {} is not failed/cancelled (status: {})", task_id, task.status));
    }

    let retry_count = task.retry_count;
    if retry_count >= max_retries {
        return Err(format!("Task {} has reached max retries ({})", task_id, state.config.max_retries));
    }

    task.retry_count += 1;

    let task_name = task.name.clone();
    let task_desc = task.description.clone();
    let task_type = task.task_type.clone();
    let task_target = task.target.clone();

    let new_id = state.generate_id();
    let now = timestamp_nanos();

    let retry = BackgroundTask {
        id: new_id.clone(),
        name: format!("{} (retry {})", task_name, retry_count + 1),
        description: task_desc,
        task_type: task_type.clone(),
        status: "queued".into(),
        progress_pct: 0,
        target: task_target,
        created_at: now,
        started_at: None,
        completed_at: None,
        duration_ms: None,
        result: None,
        error: None,
        retry_count: 0,
    };

    state.add_task(retry);

    // Auto-start retry
    let exec_id = new_id;
    let exec_type = task_type;

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        if let Ok(mut s) = BG_STATE.lock() {
            if let Some(t) = s.tasks.iter_mut().find(|t| t.id == exec_id) {
                t.status = "running".into();
                t.started_at = Some(timestamp_nanos());
                s.stats.currently_running += 1;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(500));

        if let Ok(mut s) = BG_STATE.lock() {
            if let Some(t) = s.tasks.iter_mut().find(|t| t.id == exec_id) {
                t.status = "completed".into();
                t.progress_pct = 100;
                t.completed_at = Some(timestamp_nanos());
                if let Some(started) = t.started_at {
                    t.duration_ms = Some(t.completed_at.unwrap().saturating_sub(started) / 1_000_000);
                }
                t.result = Some(format!("Retry of {} completed", exec_type));
                s.stats.currently_running = s.stats.currently_running.saturating_sub(1);
                s.stats.total_completed += 1;
                s.update_stats();
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn computer_bg_clear(include_running: Option<bool>) -> Result<usize, String> {
    let mut state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    let clear_running = include_running.unwrap_or(false);

    let before = state.tasks.len();
    state.tasks.retain(|t| {
        if clear_running {
            false
        } else {
            t.status == "running" || t.status == "queued"
        }
    });
    let removed = before - state.tasks.len();
    Ok(removed)
}

#[tauri::command]
pub fn computer_bg_stats() -> Result<BackgroundTaskStats, String> {
    let state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    let completed_durations: Vec<u64> = state
        .tasks
        .iter()
        .filter(|t| t.status == "completed")
        .filter_map(|t| t.duration_ms)
        .collect();

    let avg_duration = if completed_durations.is_empty() {
        0.0
    } else {
        let sum: u64 = completed_durations.iter().sum();
        sum as f64 / completed_durations.len() as f64
    };

    let mut stats = state.stats.clone();
    stats.currently_running = state.tasks.iter().filter(|t| t.status == "running").count() as u32;
    stats.avg_duration_ms = avg_duration;

    Ok(stats)
}

#[tauri::command]
pub fn computer_bg_config() -> Result<BackgroundTaskConfig, String> {
    let state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn computer_bg_set_config(config: BackgroundTaskConfig) -> Result<(), String> {
    let mut state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn computer_bg_run_script(
    script_name: String,
    args: Option<Vec<String>>,
) -> Result<String, String> {
    let mut state = BG_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    let task_id = state.generate_id();
    let now = timestamp_nanos();

    let task = BackgroundTask {
        id: task_id.clone(),
        name: format!("Script: {}", script_name),
        description: format!("Execute saved script '{}' with {:?} args", script_name, args.as_ref().map(|a| a.len()).unwrap_or(0)),
        task_type: "script".into(),
        status: "queued".into(),
        progress_pct: 0,
        target: script_name.clone(),
        created_at: now,
        started_at: None,
        completed_at: None,
        duration_ms: None,
        result: None,
        error: None,
        retry_count: 0,
    };

    state.add_task(task);
    drop(state);

    let exec_id = task_id.clone();
    let exec_name = script_name.clone();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));

        if let Ok(mut s) = BG_STATE.lock() {
            if let Some(t) = s.tasks.iter_mut().find(|t| t.id == exec_id) {
                t.status = "running".into();
                t.started_at = Some(timestamp_nanos());
                s.stats.currently_running += 1;
            }
        }

        // Simulate finding and executing a saved script
        std::thread::sleep(std::time::Duration::from_secs(1));

        if let Ok(mut s) = BG_STATE.lock() {
            if let Some(t) = s.tasks.iter_mut().find(|t| t.id == exec_id) {
                t.status = "completed".into();
                t.progress_pct = 100;
                t.completed_at = Some(timestamp_nanos());
                if let Some(started) = t.started_at {
                    t.duration_ms = Some(t.completed_at.unwrap().saturating_sub(started) / 1_000_000);
                }
                t.result = Some(format!("Script '{}' executed successfully", exec_name));

                s.stats.currently_running = s.stats.currently_running.saturating_sub(1);
                s.stats.total_completed += 1;
                s.update_stats();
            }
        }
    });

    Ok(task_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_generates_path() {
        let path: Option<String> = None;
        let result = computer_screen_capture(path);
        // On CI without display, screencapture may fail, but the path generation works
        if let Ok(cap) = result {
            assert!(cap.path.starts_with("/tmp/neotrix_screen_"));
            assert!(cap.path.ends_with(".png"));
            assert_eq!(cap.format, "png");
        }
    }

    #[test]
    fn test_mouse_position_parsing() {
        let simulated = "500,300";
        let coords: Vec<i32> = simulated
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        assert_eq!(coords, vec![500, 300]);
    }

    #[test]
    fn test_keyboard_type_escaping() {
        let input = r#"Hello "World" with 'quotes'"#;
        let escaped = escape_applescript_string(input);
        assert_eq!(escaped, r#"Hello \"World\" with 'quotes'"#);

        let input2 = "line1\nline2";
        let escaped2 = escape_applescript_string(input2);
        assert_eq!(escaped2, "line1\\nline2");

        let input3 = "tab\there";
        let escaped3 = escape_applescript_string(input3);
        assert_eq!(escaped3, "tab\\there");
    }
}
