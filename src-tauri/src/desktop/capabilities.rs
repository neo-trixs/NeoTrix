//! Desktop Capabilities — 平台能力抽象 trait
//!
//! 定义桌面应用所需的核心能力契约。
//! 实现者负责平台特定逻辑（Tauri/plugin 调用）。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ========== Core Types ==========

/// 更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
    pub release_notes: String,
    pub published_at: Option<String>,
    pub sha256: Option<String>,
}

/// 快捷键动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutAction {
    pub shortcut: String,
    pub action_id: String,
    pub description: String,
}

/// 文件关联配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAssociation {
    pub extension: String,
    pub mime_type: String,
    pub description: String,
    pub icon: Option<String>,
}

/// 通知优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub title: String,
    pub body: String,
    pub priority: NotificationPriority,
    pub icon: Option<String>,
    pub sound: Option<String>,
    pub actions: Vec<NotificationAction>,
}

/// 通知动作按钮
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
}

// ========== Desktop Capabilities Trait ==========

/// 桌面能力抽象 trait
///
/// 定义桌面应用所需的核心 OS 能力。
/// 通过 async trait 支持跨平台异步实现。
#[async_trait]
pub trait DesktopCapabilities: Send + Sync {
    // ----- 系统托盘 -----

    /// 初始化系统托盘
    async fn setup_tray(&self) -> Result<(), String>;

    /// 更新托盘图标提示
    async fn update_tray_tooltip(&self, tooltip: &str) -> Result<(), String>;

    /// 显示/隐藏托盘
    async fn set_tray_visible(&self, visible: bool) -> Result<(), String>;

    // ----- 通知 -----

    /// 发送系统通知
    async fn show_notification(&self, config: NotificationConfig) -> Result<(), String>;

    /// 请求通知权限
    async fn request_notification_permission(&self) -> Result<bool, String>;

    // ----- 快捷键 -----

    /// 注册全局快捷键
    async fn register_shortcut(&self, shortcut: &str, action_id: String) -> Result<(), String>;

    /// 注销全局快捷键
    async fn unregister_shortcut(&self, shortcut: &str) -> Result<(), String>;

    /// 列出已注册的快捷键
    async fn list_shortcuts(&self) -> Result<Vec<ShortcutAction>, String>;

    // ----- 文件关联 -----

    /// 注册文件关联
    async fn register_file_association(&self, association: FileAssociation) -> Result<(), String>;

    /// 注销文件关联
    async fn unregister_file_association(&self, extension: &str) -> Result<(), String>;

    /// 列出已注册的文件关联
    async fn list_file_associations(&self) -> Result<Vec<FileAssociation>, String>;

    // ----- 自动更新 -----

    /// 检查更新
    async fn check_update(&self) -> Result<Option<UpdateInfo>, String>;

    /// 下载并安装更新
    async fn install_update(&self, info: &UpdateInfo) -> Result<(), String>;

    /// 获取当前版本
    fn current_version(&self) -> String;

    // ----- 窗口管理 -----

    /// 显示主窗口
    async fn show_main_window(&self) -> Result<(), String>;

    /// 隐藏主窗口
    async fn hide_main_window(&self) -> Result<(), String>;

    /// 设置窗口标题
    async fn set_window_title(&self, title: &str) -> Result<(), String>;

    /// 窗口是否可见
    async fn is_window_visible(&self) -> Result<bool, String>;

    // ----- 剪贴板 -----

    /// 写入剪贴板
    async fn write_clipboard(&self, text: &str) -> Result<(), String>;

    /// 读取剪贴板
    async fn read_clipboard(&self) -> Result<String, String>;

    // ----- 进程 -----

    /// 打开外部链接（浏览器）
    async fn open_url(&self, url: &str) -> Result<(), String>;

    /// 打开文件管理器
    async fn open_file_manager(&self, path: &str) -> Result<(), String>;

    /// 退出应用
    async fn exit_app(&self, code: i32) -> Result<(), String>;
}

// ========== Default NotificationConfig ==========

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            priority: NotificationPriority::Normal,
            icon: None,
            sound: None,
            actions: vec![],
        }
    }
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::Normal
    }
}

// ========== Helper Functions ==========

/// 构造快捷通知配置
pub fn quick_notification(title: impl Into<String>, body: impl Into<String>) -> NotificationConfig {
    NotificationConfig {
        title: title.into(),
        body: body.into(),
        ..Default::default()
    }
}

/// 构造高优先级通知配置
pub fn urgent_notification(title: impl Into<String>, body: impl Into<String>) -> NotificationConfig {
    NotificationConfig {
        title: title.into(),
        body: body.into(),
        priority: NotificationPriority::Urgent,
        ..Default::default()
    }
}
