use super::*;
use crossterm::event::{KeyCode, KeyModifiers};

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
    assert_eq!(app.status_text, "Ready | Provider: not configured");
}

// ── Keyboard: Escape & Exit ──

#[test]
fn test_keyboard_escape_sets_running_false() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(!app.running);
}

#[test]
fn test_keyboard_ctrl_d_sets_running_false() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    app.handle_key(KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert!(!app.running);
}

// ── Keyboard: Enter / Submit ──

#[test]
fn test_keyboard_enter_submits_non_empty_input() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
    let result = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(result, Some("hi".to_string()));
    assert!(app.input.is_empty());
    assert_eq!(app.command_history.entries.len(), 1);
    assert_eq!(app.command_history.entries[0], "hi");
    assert!(!app.multi_line);
}

#[test]
fn test_keyboard_enter_empty_input_no_submit() {
    let mut app = TuiApp::new(false);
    let result = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(result, None);
    assert!(app.command_history.entries.is_empty());
}

#[test]
fn test_keyboard_alt_enter_multi_line() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    let result = app.handle_key(KeyCode::Enter, KeyModifiers::ALT);
    assert_eq!(result, None);
    assert!(app.multi_line);
    assert!(app.input.contains('\n'));
}

#[test]
fn test_keyboard_multi_line_enter_stays_in_multi_line() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Enter, KeyModifiers::ALT);
    app.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);
    let result = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(result, None);
    assert_eq!(app.input, "a\nb\n");
}

// ── Keyboard: PageUp / PageDown ──

#[test]
fn test_keyboard_page_up_scrolls_up() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 20;
    app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 10);
}

#[test]
fn test_keyboard_page_up_clamps_at_zero() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 5;
    app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_keyboard_page_up_zero_stays_zero() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_keyboard_page_down_scrolls_down() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 10);
}

#[test]
fn test_keyboard_page_down_accumulates() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
    app.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 20);
}

// ── Keyboard: Ctrl shortcuts ──

#[test]
fn test_keyboard_ctrl_s_sets_save_status() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(app.status_text, "Saving...");
}

#[test]
fn test_keyboard_ctrl_p_sets_provider_status() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('p'), KeyModifiers::CONTROL);
    assert_eq!(app.status_text, "Switch Provider... | not yet implemented");
}

#[test]
fn test_keyboard_ctrl_n_creates_new_session() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(app.sessions.len(), 2);
    assert_eq!(app.active_session, 1);
    assert_eq!(app.sessions[1].id, "s-2");
}

#[test]
fn test_keyboard_ctrl_w_removes_session() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('n'), KeyModifiers::CONTROL);
    app.handle_key(KeyCode::Char('w'), KeyModifiers::CONTROL);
    assert_eq!(app.sessions.len(), 1);
}

#[test]
fn test_keyboard_ctrl_w_single_session_no_op() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('w'), KeyModifiers::CONTROL);
    assert_eq!(app.sessions.len(), 1);
}

#[test]
fn test_keyboard_ctrl_l_clears_messages() {
    let mut app = TuiApp::new(false);
    app.push_message("user", "hello".to_string());
    app.push_message("assistant", "world".to_string());
    assert_eq!(app.active_session().messages.len(), 2);
    app.handle_key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert!(app.active_session().messages.is_empty());
}

// ── Keyboard: Typing characters ──

#[test]
fn test_keyboard_typing_appends_to_input() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('H'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('l'), KeyModifiers::NONE);
    assert_eq!(app.input, "Hel");
}

#[test]
fn test_keyboard_backspace_removes_char() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
    assert_eq!(app.input, "a");
}

#[test]
fn test_keyboard_backspace_empty_does_nothing() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
    assert!(app.input.is_empty());
}

// ── Keyboard: Tab completion ──

#[test]
fn test_keyboard_tab_completes_command() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.input, "/help");
}

#[test]
fn test_keyboard_tab_no_match_keeps_input() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('x'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.input, "x");
}

// ── Keyboard: History navigation ──

#[test]
fn test_keyboard_up_down_history() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(app.input, "b");
    app.handle_key(KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(app.input, "a");
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.input, "b");
}

#[test]
fn test_keyboard_up_empty_history_does_nothing() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Up, KeyModifiers::NONE);
    assert!(app.input.is_empty());
}

// ── Sessions: navigation ──

#[test]
fn test_keyboard_left_switches_session() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('n'), KeyModifiers::CONTROL);
    app.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(app.active_session, 0);
}

#[test]
fn test_keyboard_left_at_first_session_stays() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(app.active_session, 0);
}

#[test]
fn test_keyboard_right_switches_session() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('n'), KeyModifiers::CONTROL);
    app.handle_key(KeyCode::Right, KeyModifiers::NONE);
    assert_eq!(app.active_session, 1);
}

#[test]
fn test_keyboard_right_at_last_session_stays() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Right, KeyModifiers::NONE);
    assert_eq!(app.active_session, 0);
}

// ── Key: Toggle Thinking ──

#[test]
fn test_keyboard_t_toggles_thinking_status() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('t'), KeyModifiers::NONE);
    assert_eq!(app.status_text, "Thinking toggled");
}

// ── Unhandled keys ──

#[test]
fn test_keyboard_unhandled_key_returns_none() {
    let mut app = TuiApp::new(false);
    let result = app.handle_key(KeyCode::F(1), KeyModifiers::NONE);
    assert_eq!(result, None);
}

#[test]
fn test_keyboard_ctrl_plus_char_does_not_type() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('x'), KeyModifiers::CONTROL);
    assert!(app.input.is_empty());
}

// ── Command Parsing ──

#[test]
fn test_process_submitted_text_recognizes_slash_commands() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/help", None);
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[test]
fn test_process_submitted_text_non_command_returns_err() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("hello world", None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "hello world");
}

#[test]
fn test_process_submitted_text_unknown_command() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/unknown_cmd", None);
    assert!(result.is_ok());
    assert!(!result.unwrap().success);
}

#[test]
fn test_process_submitted_text_absorb_command() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/absorb", None);
    assert!(result.is_ok());
}

#[test]
fn test_process_submitted_text_evolve_command() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/evolve", None);
    assert!(result.is_ok());
}

#[test]
fn test_process_submitted_text_mem_command() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/mem", None);
    assert!(result.is_ok());
}

#[test]
fn test_process_submitted_text_save_command() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/save", None);
    assert!(result.is_ok());
}

#[test]
fn test_process_submitted_text_agent_command() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/agent", None);
    assert!(result.is_ok());
}

#[test]
fn test_process_submitted_text_mcp_command() {
    let mut app = TuiApp::new(false);
    let result = app.process_submitted_text("/mcp", None);
    assert!(result.is_ok());
}

// ── Scroll Offset Clamping ──

#[test]
fn test_scroll_offset_cannot_go_below_zero() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
    app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_scroll_offset_increases_with_page_down() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 0;
    app.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 10);
}

#[test]
fn test_scroll_offset_reset_on_session_switch() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 42;
    app.handle_key(KeyCode::Char('n'), KeyModifiers::CONTROL);
    app.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 0);
}

// ── Streaming ──

#[test]
fn test_stream_token_starts_streaming() {
    let mut app = TuiApp::new(false);
    assert!(!app.streaming);
    app.stream_token("assistant", "Hello");
    assert!(app.streaming);
    assert_eq!(app.streaming_role, "assistant");
    assert_eq!(app.streaming_text, "Hello");
    assert!(app.agent_busy);
}

#[test]
fn test_stream_token_appends() {
    let mut app = TuiApp::new(false);
    app.stream_token("assistant", "Hello ");
    app.stream_token("assistant", "world");
    assert_eq!(app.streaming_text, "Hello world");
    assert_eq!(app.token_count, 2);
}

#[test]
fn test_stream_finish_persists_message() {
    let mut app = TuiApp::new(false);
    app.stream_token("assistant", "Hello world");
    app.stream_finish();
    assert!(!app.streaming);
    assert!(!app.agent_busy);
    assert_eq!(app.active_session().messages.len(), 1);
    assert_eq!(app.active_session().messages[0].role, "assistant");
    assert_eq!(app.active_session().messages[0].content, "Hello world");
}

#[test]
fn test_stream_finish_empty_does_not_push() {
    let mut app = TuiApp::new(false);
    app.stream_finish();
    assert!(app.active_session().messages.is_empty());
}

#[test]
fn test_update_tokens_per_sec() {
    let mut app = TuiApp::new(false);
    assert_eq!(app.tokens_per_sec, 0.0);
    app.update_tokens_per_sec(12.5);
    assert!((app.tokens_per_sec - 12.5).abs() < 1e-10);
}

// ── Messages ──

#[test]
fn test_push_message_adds_to_active_session() {
    let mut app = TuiApp::new(false);
    assert!(app.active_session().messages.is_empty());
    app.push_message("user", "Hello".to_string());
    assert_eq!(app.active_session().messages.len(), 1);
    assert_eq!(app.active_session().messages[0].role, "user");
    assert_eq!(app.active_session().messages[0].content, "Hello");
}

#[test]
fn test_push_message_multiple_messages() {
    let mut app = TuiApp::new(false);
    app.push_message("user", "hello".to_string());
    app.push_message("assistant", "world".to_string());
    assert_eq!(app.active_session().messages.len(), 2);
    assert_eq!(app.active_session().messages[0].role, "user");
    assert_eq!(app.active_session().messages[1].role, "assistant");
}

// ── Session Switching ──

#[test]
fn test_active_session_returns_correct_session() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('n'), KeyModifiers::CONTROL);
    app.push_message("user", "in session 2".to_string());
    app.active_session = 0;
    assert!(app.active_session().messages.is_empty());
    app.active_session = 1;
    assert_eq!(app.active_session().messages.len(), 1);
}

// ── Thinking Toggle ──

#[test]
fn test_toggle_thinking_with_no_assistant_does_nothing() {
    let mut app = TuiApp::new(false);
    app.push_message("user", "hello".to_string());
    app.toggle_thinking();
    assert!(app.thinking_expanded.is_empty());
}

#[test]
fn test_toggle_thinking_with_assistant_no_think_blocks() {
    let mut app = TuiApp::new(false);
    app.push_message("assistant", "hello".to_string());
    app.toggle_thinking();
    assert!(app.thinking_expanded.is_empty());
}

// ── Status text after Enter with command flow ──

#[test]
fn test_enter_clears_input_and_resets_multi_line() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('x'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('z'), KeyModifiers::NONE);
    let result = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(result, Some("xyz".to_string()));
    assert!(app.input.is_empty());
    assert!(!app.multi_line);
}
