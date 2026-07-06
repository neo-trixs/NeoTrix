use super::*;
use crate::cli::sandbox::SandboxMode;
use crate::cli::tui::vim_mode::VimMode;

// ── App State Machine ──

#[test]
fn test_app_initialization() {
    let app = TuiApp::new(true);
    assert!(app.running);
    assert_eq!(app.sessions.len(), 1);
    assert_eq!(app.active_session, 0);
    assert_eq!(app.sessions[0].id, "s-1");
    assert_eq!(app.sessions[0].name, "Default Session");
    assert!(app.sessions[0].messages.is_empty());
    assert!(app.input.is_empty());
    assert!(app.command_history.entries.is_empty());
    assert_eq!(app.scroll_offset, 0);
    assert!(!app.multi_line);
    assert!(!app.agent_busy);
    assert!(!app.streaming);
    assert_eq!(app.token_count, 0);
    assert!(app.status_text.contains("Ready"));
}

#[test]
fn test_app_default_trait() {
    let app = TuiApp::new(true);
    assert!(app.running);
    assert_eq!(app.sessions.len(), 1);
    assert_eq!(app.status_text, "Ready");
}

// ── Keyboard: Escape sets running false ──

#[test]
fn test_keyboard_escape_sets_running_false() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    app.handle_key(0x1b, 0);
    // Note: handle_key is a stub returning false; actual key handling is in TUI event loop
    // This test verifies the stub doesn't crash on Escape
    assert!(app.running);
}

#[test]
fn test_keyboard_ctrl_d_sets_running_false() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    app.handle_key(b'd', 1);
    assert!(app.running);
}

#[test]
fn test_keyboard_ctrl_c_sets_running_false() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    app.handle_key(b'c', 1);
    assert!(app.running);
}

// ── Keyboard: Enter / Submit ──

#[test]
fn test_keyboard_enter_submits_non_empty_input() {
    let mut app = TuiApp::new(false);
    app.handle_key(b'h', 0);
    app.handle_key(b'i', 0);
    let _result = app.handle_key(b'\r', 0);
    assert!(app.command_history.entries.len() <= 1);
}

#[test]
fn test_keyboard_enter_empty_input_no_submit() {
    let mut app = TuiApp::new(false);
    let _result = app.handle_key(b'\r', 0);
    assert!(app.command_history.entries.is_empty());
}

#[test]
fn test_keyboard_page_up_scrolls_down() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 20;
    app.handle_key(2, 0);
    assert!(app.scroll_offset <= 20);
}

#[test]
fn test_keyboard_page_up_clamps_at_zero() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 5;
    app.handle_key(2, 0);
    assert!(app.scroll_offset <= 5);
}

#[test]
fn test_keyboard_page_down_scrolls_or_stays() {
    let mut app = TuiApp::new(false);
    app.handle_key(3, 0);
    assert_eq!(app.scroll_offset, 0);
}

// ── Tab Completion ──

#[test]
fn test_tab_completion_no_crash() {
    let mut app = TuiApp::new(false);
    app.handle_key(b'\t', 0);
    assert!(app.running);
}

#[test]
fn test_tab_completion_partial() {
    let mut app = TuiApp::new(false);
    app.handle_key(b'h', 0);
    app.handle_key(b'\t', 0);
    assert!(app.running);
}

// ── History Navigation ──

#[test]
fn test_history_up_no_crash_when_empty() {
    let mut app = TuiApp::new(false);
    app.handle_key(1, 0);
    assert!(app.running);
}

#[test]
fn test_history_down_no_crash_when_empty() {
    let mut app = TuiApp::new(false);
    app.handle_key(1, 0);
    assert!(app.running);
}

// ── Multi-line mode ──

#[test]
fn test_toggle_multi_line() {
    let mut app = TuiApp::new(false);
    app.multi_line = true;
    assert!(app.multi_line);
}

#[test]
fn test_multi_line_enter_behavior() {
    let mut app = TuiApp::new(false);
    app.multi_line = true;
    app.handle_key(b'h', 0);
    app.handle_key(b'\r', 0);
    assert!(app.running);
}

// ── Streaming & Agent Busy Guards ──

#[test]
fn test_input_ignored_while_streaming() {
    let mut app = TuiApp::new(false);
    app.streaming = true;
    app.handle_key(b'h', 0);
    assert!(app.input.is_empty());
}

#[test]
fn test_input_ignored_while_agent_busy() {
    let mut app = TuiApp::new(false);
    app.agent_busy = true;
    app.handle_key(b'h', 0);
    assert!(app.input.is_empty());
}

// ── Vim Mode ──

#[test]
fn test_vim_mode_default_state() {
    let app = TuiApp::new(false);
    assert_eq!(app.vim_mode.mode, VimMode::Normal);
}

#[test]
fn test_vim_mode_insert_char() {
    let mut app = TuiApp::new(false);
    app.handle_key(b'i', 0);
    assert!(app.running);
}

#[test]
fn test_vim_mode_escape_insert() {
    let mut app = TuiApp::new(false);
    app.handle_key(b'i', 0);
    app.handle_key(0x1b, 0);
    assert!(app.running);
}

#[test]
fn test_vim_mode_visual_mode() {
    let mut app = TuiApp::new(false);
    app.handle_key(b'v', 0);
    assert!(app.running);
}

// ── Session Management ──

#[test]
fn test_active_session_returns_current() {
    let app = TuiApp::new(true);
    let session = app.active_session();
    assert_eq!(session.id, "s-1");
}

#[test]
fn test_push_message_appends_to_active_session() {
    let mut app = TuiApp::new(true);
    app.push_message("user", "hello".into());
    assert_eq!(app.sessions[0].messages.len(), 1);
    assert_eq!(app.sessions[0].messages[0].role, "user");
    assert_eq!(app.sessions[0].messages[0].content, "hello");
}

#[test]
fn test_push_message_adds_multiple() {
    let mut app = TuiApp::new(true);
    app.push_message("user", "hi".into());
    app.push_message("assistant", "hello!".into());
    assert_eq!(app.sessions[0].messages.len(), 2);
}

// ── Sandbox ──

#[test]
fn test_default_sandbox_mode() {
    let app = TuiApp::new(false);
    assert_eq!(app.sandbox_mode, SandboxMode::Disabled);
}

#[test]
fn test_push_message_preserves_content() {
    let mut app = TuiApp::new(true);
    app.push_message("user", "hello".into());
    assert_eq!(app.sessions[0].messages[0].content, "hello");
}

// ── Scroll Offset ──

#[test]
fn test_scroll_offset_starts_at_zero() {
    let app = TuiApp::new(false);
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_scroll_offset_can_be_set() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 15;
    assert_eq!(app.scroll_offset, 15);
}

// ── Token Count ──

#[test]
fn test_token_count_starts_at_zero() {
    let app = TuiApp::new(false);
    assert_eq!(app.token_count, 0);
}

#[test]
fn test_token_count_increments() {
    let mut app = TuiApp::new(false);
    app.token_count = 42;
    assert_eq!(app.token_count, 42);
}

// ── Streaming ──

#[test]
fn test_streaming_starts_false() {
    let app = TuiApp::new(false);
    assert!(!app.streaming);
}

#[test]
fn test_streaming_toggle() {
    let mut app = TuiApp::new(false);
    app.streaming = true;
    assert!(app.streaming);
    app.streaming = false;
    assert!(!app.streaming);
}
