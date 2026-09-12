use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

/// File drop result sent to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDropResult {
    pub accepted: Vec<String>,
    pub rejected: Vec<String>,
}

/// Manages file drag-and-drop with extension allowlist
pub struct FileDropManager {
    allowed_extensions: Vec<String>,
}

impl FileDropManager {
    pub fn new() -> Self {
        Self {
            allowed_extensions: vec![
                ".gguf".into(),
                ".onnx".into(),
                ".pdf".into(),
                ".md".into(),
                ".txt".into(),
                ".json".into(),
                ".csv".into(),
                ".png".into(),
                ".jpg".into(),
                ".jpeg".into(),
                ".gif".into(),
                ".webp".into(),
                ".safetensors".into(),
                ".bin".into(),
                ".pt".into(),
                ".pth".into(),
            ],
        }
    }

    pub fn is_allowed(&self, path: &str) -> bool {
        let p = PathBuf::from(path);
        match p.extension() {
            Some(ext) => {
                let ext_str = format!(".{}", ext.to_string_lossy());
                self.allowed_extensions
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(&ext_str))
            }
            None => false,
        }
    }

    pub fn handle_drop(&self, paths: Vec<String>) -> FileDropResult {
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        for path in paths {
            if self.is_allowed(&path) {
                accepted.push(path);
            } else {
                rejected.push(path);
            }
        }
        FileDropResult { accepted, rejected }
    }

    pub fn add_extension(&mut self, ext: String) {
        let ext = if ext.starts_with('.') {
            ext
        } else {
            format!(".{}", ext)
        };
        if !self.allowed_extensions.contains(&ext) {
            self.allowed_extensions.push(ext);
        }
    }
}

/// Tauri command: filter dropped files by allowed extensions
#[tauri::command]
pub fn handle_file_drop(paths: Vec<String>) -> FileDropResult {
    let manager = FileDropManager::new();
    manager.handle_drop(paths)
}

/// Set up file drop event listener on the main window.
/// Emits `neotrix://file-drop` to the frontend with the filtered results.
pub fn setup_file_drop_listener(app: &AppHandle) {
    let manager = std::sync::Mutex::new(FileDropManager::new());
    let app_handle = app.clone();

    if let Some(window) = app.get_webview_window("main") {
        let mgr = manager;
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::FileDrop { paths, .. } = event {
                let paths_str: Vec<String> = paths
                    .iter()
                    .filter_map(|p| p.to_str().map(String::from))
                    .collect();
                let result = mgr.lock().unwrap().handle_drop(paths_str);
                if !result.accepted.is_empty() {
                    let _ = app_handle.emit("neotrix://file-drop", &result);
                }
            }
        });
    }
}
