//! ratatui 交互式终端
//!
//! 四面板布局：
//! - 左：会话列表
//! - 右上：聊天输出
//! - 右下：输入行
//! - 底：状态栏

pub mod app;
pub mod diff_viewer;
pub mod history;
pub mod input;
pub mod layout;
pub mod output;
pub mod session_store;
pub mod themes;
pub mod vim_mode;

pub use app::TuiApp;
pub use history::CommandHistory;
pub use session_store::{SessionData, SessionStore};
pub use themes::{theme_by_name, theme_list, Theme};

#[cfg(test)]
mod tests;
