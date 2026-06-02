//! ratatui 交互式终端
//!
//! 四面板布局：
//! - 左：会话列表
//! - 右上：聊天输出
//! - 右下：输入行
//! - 底：状态栏

pub mod app;
pub mod layout;
pub mod input;
pub mod output;
pub mod themes;
pub mod session_store;
pub mod diff_viewer;
pub mod history;
pub mod vim_mode;

pub use app::TuiApp;
pub use themes::{Theme, theme_by_name, theme_list};
pub use session_store::{SessionStore, SessionData};
pub use history::CommandHistory;

#[cfg(test)]
mod tests;
