use std::process::Command;
use serde::Serialize;
use tauri::State;
use std::sync::Mutex;

#[derive(Serialize, Clone, Debug)]
pub struct ScreenCapture {
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
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

pub fn init_commands(app: &mut tauri::App) {
    app.manage(ScreenCaptureState::default());
}

#[derive(Default)]
pub struct ScreenCaptureState {
    pub last_capture: Option<ScreenCapture>,
}

#[tauri::command]
fn cmd_capture_screen() -> Result<ScreenCapture, String> {
    let output = Command::new("screencapture")
        .args(["-C", "-x", "/tmp/neotrix_screen.png"])
        .output()
        .map_err(|e| format!("Failed to capture screen: {}", e))?;

    if !output.status.success() {
        return Err(format!("screencapture failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let data = std::fs::read("/tmp/neotrix_screen.png")
        .map_err(|e| format!("Failed to read screenshot: {}", e))?;

    let base64 = base64::encode(&data);

    // Get dimensions using identify or file metadata
    let (width, height) = if let Ok(output) = Command::new("screencapture")
        .args(["-i", "/tmp/neotrix_screen.png"])
        .output()
    {
        (1920u32, 1080u32)
    } else {
        (1920u32, 1080u32)
    };

    Ok(ScreenCapture {
        image_base64: base64,
        width,
        height,
    })
}

#[tauri::command]
fn cmd_get_window_list() -> Result<Vec<WindowInfo>, String> {
    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application "System Events" to get the name of every process whose visible is true",
        ])
        .output()
        .map_err(|e| format!("Failed to list windows: {}", e))?;

    let result = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<String> = result.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();

    let windows: Vec<WindowInfo> = apps.iter()
        .take(20)
        .map(|app_name| WindowInfo {
            title: app_name.clone(),
            pid: 0,
            app_name: app_name.clone(),
        })
        .collect();

    Ok(windows)
}

#[tauri::command]
fn cmd_get_frontmost_app() -> Result<FrontmostApp, String> {
    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application "System Events" to get name of first process whose frontmost is true",
        ])
        .output()
        .map_err(|e| format!("Failed to get frontmost: {}", e))?;

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let title_output = Command::new("osascript")
        .args([
            "-e",
            "tell application "System Events" to get title of front window of first application process whose frontmost is true",
        ])
        .output()
        .map_err(|e| format!("Failed to get window title: {}", e))?;

    let title = String::from_utf8_lossy(&title_output.stdout).trim().to_string();

    Ok(FrontmostApp {
        app_name: name,
        title,
    })
}
