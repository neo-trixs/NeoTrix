//! Tauri Implementation of DesktopCapabilities
//!
//! Bridges the platform-agnostic trait to Tauri-specific APIs.

use async_trait::async_trait;
use tauri::Manager;

use super::capabilities::{
    DesktopCapabilities, FileAssociation, NotificationConfig, NotificationPriority, ShortcutAction,
    UpdateInfo,
};
use crate::notifications::NotificationManager;

pub struct TauriDesktopCapabilities {
    app: tauri::AppHandle,
}

impl TauriDesktopCapabilities {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

#[async_trait]
impl DesktopCapabilities for TauriDesktopCapabilities {
    // ----- 系统托盘 -----

    async fn setup_tray(&self) -> Result<(), String> {
        Ok(())
    }

    async fn update_tray_tooltip(&self, _tooltip: &str) -> Result<(), String> {
        Ok(())
    }

    async fn set_tray_visible(&self, _visible: bool) -> Result<(), String> {
        Ok(())
    }

    // ----- 通知 -----

    async fn show_notification(&self, config: NotificationConfig) -> Result<(), String> {
        let manager = self.app.state::<NotificationManager>();
        manager.show_notification(&config.title, &config.body)
    }

    async fn request_notification_permission(&self) -> Result<bool, String> {
        Ok(true)
    }

    // ----- 快捷键 -----

    async fn register_shortcut(&self, _shortcut: &str, _action_id: String) -> Result<(), String> {
        Ok(())
    }

    async fn unregister_shortcut(&self, _shortcut: &str) -> Result<(), String> {
        Ok(())
    }

    async fn list_shortcuts(&self) -> Result<Vec<ShortcutAction>, String> {
        Ok(vec![])
    }

    // ----- 文件关联 -----

    async fn register_file_association(&self, _association: FileAssociation) -> Result<(), String> {
        Ok(())
    }

    async fn unregister_file_association(&self, _extension: &str) -> Result<(), String> {
        Ok(())
    }

    async fn list_file_associations(&self) -> Result<Vec<FileAssociation>, String> {
        Ok(vec![])
    }

    // ----- 自动更新 -----

    async fn check_update(&self) -> Result<Option<UpdateInfo>, String> {
        Ok(None)
    }

    async fn install_update(&self, _info: &UpdateInfo) -> Result<(), String> {
        Ok(())
    }

    fn current_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    // ----- 窗口管理 -----

    async fn show_main_window(&self) -> Result<(), String> {
        if let Some(window) = self.app.get_webview_window("main") {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn hide_main_window(&self) -> Result<(), String> {
        if let Some(window) = self.app.get_webview_window("main") {
            window.hide().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn set_window_title(&self, title: &str) -> Result<(), String> {
        if let Some(window) = self.app.get_webview_window("main") {
            window.set_title(title).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn is_window_visible(&self) -> Result<bool, String> {
        if let Some(window) = self.app.get_webview_window("main") {
            Ok(window.is_visible().unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    // ----- 剪贴板 -----

    async fn write_clipboard(&self, _text: &str) -> Result<(), String> {
        Ok(())
    }

    async fn read_clipboard(&self) -> Result<String, String> {
        Ok(String::new())
    }

    // ----- 进程 -----

    async fn open_url(&self, url: &str) -> Result<(), String> {
        open::that(url).map_err(|e| e.to_string())
    }

    async fn open_file_manager(&self, path: &str) -> Result<(), String> {
        open::that(path).map_err(|e| e.to_string())
    }

    async fn exit_app(&self, code: i32) -> Result<(), String> {
        self.app.exit(code);
        Ok(())
    }
}
