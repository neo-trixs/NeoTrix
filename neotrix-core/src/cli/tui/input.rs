//! 输入系统 — 复用现有 input_editor.rs 思路
//! 当前通过 TuiApp::handle_key 实现基本输入
// Future: vim mode, multi-line mode (tab completion in TuiApp)

use super::app::TuiApp;

pub fn handle_input(app: &mut TuiApp, c: char) {
    app.input.push(c);
}

pub fn handle_backspace(app: &mut TuiApp) {
    app.input.pop();
}

pub fn handle_enter(app: &mut TuiApp) -> bool {
    let trimmed = app.input.trim().to_string();
    if trimmed.is_empty() {
        return false;
    }
    app.command_history.push(trimmed);
    true
}
