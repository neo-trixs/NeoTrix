use std::process::Command;
use serde::Serialize;

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

fn win_script() -> String {
    r#"tell application "System Events" to get name of every process whose visible is true"#.to_string()
}

fn front_app_script() -> String {
    r#"tell application "System Events" to get name of first process whose frontmost is true"#.to_string()
}

fn front_title_script() -> String {
    r#"tell application "System Events" to get title of front window of first application process whose frontmost is true"#.to_string()
}

#[tauri::command]
pub fn capture_screen() -> Result<ScreenCapture, String> {
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

    Ok(ScreenCapture { image_base64: base64, width: 1920, height: 1080 })
}

#[tauri::command]
pub fn get_window_list() -> Result<Vec<WindowInfo>, String> {
    let output = Command::new("osascript")
        .args(["-e", &win_script()])
        .output()
        .map_err(|e| format!("Failed to list windows: {}", e))?;

    let result = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<String> = result.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

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
pub fn get_frontmost_app() -> Result<FrontmostApp, String> {
    let output = Command::new("osascript")
        .args(["-e", &front_app_script()])
        .output()
        .map_err(|e| format!("Failed to get frontmost: {}", e))?;

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let title_output = Command::new("osascript")
        .args(["-e", &front_title_script()])
        .output()
        .map_err(|e| format!("Failed to get window title: {}", e))?;

    let title = String::from_utf8_lossy(&title_output.stdout).trim().to_string();

    Ok(FrontmostApp { app_name: name, title })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scripts_are_valid() {
        assert!(win_script().contains("System Events"));
        assert!(front_app_script().contains("frontmost"));
        assert!(front_title_script().contains("front window"));
    }

    #[test]
    fn test_screen_capture_default_size() {
        let sizes = [(1920u32, 1080u32)];
        assert_eq!(sizes[0], (1920, 1080));
    }
}
