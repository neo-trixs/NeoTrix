//! System Notification Manager
//!
//! Wraps tauri-plugin-notification to provide typed notification APIs.

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

pub struct NotificationManager {
    app: AppHandle,
}

impl NotificationManager {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// 显示通知
    pub fn show_notification(&self, title: &str, body: &str) -> Result<(), String> {
        self.app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|e| format!("Failed to show notification: {}", e))
    }

    /// 显示新消息通知
    pub fn notify_new_message(&self, session_title: &str, preview: &str) -> Result<(), String> {
        self.show_notification(&format!("新消息 - {}", session_title), preview)
    }

    /// 显示任务完成通知
    pub fn notify_task_complete(&self, task_name: &str) -> Result<(), String> {
        self.show_notification("任务完成", &format!("{} 已完成", task_name))
    }

    /// 显示模型下载完成通知
    pub fn notify_model_downloaded(&self, model_name: &str) -> Result<(), String> {
        self.show_notification("模型下载完成", &format!("{} 已准备就绪", model_name))
    }

    /// 显示错误通知
    pub fn notify_error(&self, error: &str) -> Result<(), String> {
        self.show_notification("错误", error)
    }
}
