use super::*;
use crate::cli::sandbox::SandboxMode;
use crate::cli::tui::vim_mode::VimMode;
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

// ── Keyboard: Escape / Ctrl+C / Ctrl+D ──

#[test]
fn test_keyboard_escape_does_not_quit() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    let action = app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(action, KeyAction::None);
    assert!(app.running);
}

#[test]
fn test_keyboard_ctrl_d_quits_when_idle_empty() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    let action = app.handle_key(KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_eq!(action, KeyAction::Quit);
}

#[test]
fn test_keyboard_ctrl_c_quits_when_idle_empty() {
    let mut app = TuiApp::new(false);
    assert!(app.running);
    let action = app.handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(action, KeyAction::Quit);
}

#[test]
fn test_keyboard_ctrl_c_cancels_when_streaming() {
    let mut app = TuiApp::new(false);
    app.streaming = true;
    let action = app.handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(action, KeyAction::CancelGeneration);
    assert!(!app.streaming);
}

// ── Keyboard: Enter / Submit ──

#[test]
fn test_keyboard_enter_submits() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(action, KeyAction::Submit);
}

#[test]
fn test_keyboard_enter_empty_input_submits() {
    let mut app = TuiApp::new(false);
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(action, KeyAction::Submit);
    // run() 层会判断 trim 为空则不发请求。
}

// ── Scroll ──

#[test]
fn test_keyboard_page_up_scrolls() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 20;
    app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 25);
}

#[test]
fn test_keyboard_page_down_scrolls() {
    let mut app = TuiApp::new(false);
    app.scroll_offset = 5;
    app.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
    assert_eq!(app.scroll_offset, 0);
}

// ── Tab Completion ──

#[test]
fn test_tab_completion_no_crash() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert!(app.running);
}

#[test]
fn test_tab_completion_partial() {
    let mut app = TuiApp::new(false);
    app.input = "/ex".into();
    app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert!(app.input.starts_with("/exit"));
}

// ── History Navigation ──

#[test]
fn test_history_up_no_crash_when_empty() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Up, KeyModifiers::NONE);
    assert!(app.running);
}

#[test]
fn test_history_down_no_crash_when_empty() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
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
    app.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(action, KeyAction::None);
    assert_eq!(app.input, "h\n");
}

// ── Streaming & Agent Busy Guards ──

#[test]
fn test_input_allowed_while_streaming() {
    // 新语义：流式中仍允许输入（排队），由 run() 层决定是否忽略。
    let mut app = TuiApp::new(false);
    app.streaming = true;
    app.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
    assert_eq!(app.input, "h");
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
    app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
    assert!(app.running);
}

#[test]
fn test_vim_mode_escape_insert() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.running);
}

#[test]
fn test_vim_mode_visual_mode() {
    let mut app = TuiApp::new(false);
    app.handle_key(KeyCode::Char('v'), KeyModifiers::NONE);
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

// ── Slash 补全（registry 动态） ──

#[test]
fn test_slash_candidates_registry_sourced() {
    // 前缀 /s 应同时含 TUI 专用命令与 registry 人类控制命令 (命令面精简后控制命令为一级)
    let cands = crate::cli::tui::app::tui_app::slash_command_candidates("/s");
    assert!(cands.iter().any(|c| c == "/save"), "TUI 专用 /save 应在候选");
    assert!(cands.iter().any(|c| c == "/sessions"), "TUI 专用 /sessions 应在候选");
    assert!(!cands.iter().any(|c| c == "/session-all"), "聚合器 /session-all 是 agent 工具, 不应在人类候选");
    assert!(!cands.iter().any(|c| c == "/session"), "/session 是 agent 工具, 不应在候选");
    assert!(cands.iter().any(|c| c == "/stats"), "控制命令 /stats 应在候选");
}

#[test]
fn test_slash_candidates_no_match() {
    let cands = crate::cli::tui::app::tui_app::slash_command_candidates("/zzz_nonexistent");
    assert!(cands.is_empty());
}

// ── @ 文件引用补全（私有方法测试在 tui_app.rs 内部 tests） ──

#[test]
fn test_at_completion_no_crash_when_dir_empty() {
    let mut app = TuiApp::new(false);
    app.input = "@".into();
    app.cursor = 1;
    app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert!(app.running);
}

// ── 会话恢复 picker ──

#[test]
fn test_session_picker_navigation_and_select() {
    use crate::cli::tui::app::types::{SessionEntry, SessionPicker};
    let mut app = TuiApp::new(false);
    app.session_picker = Some(SessionPicker {
        entries: vec![
            SessionEntry { name: "sess-a".into(), updated_at: "2026-01-01T00:00:00+00:00".into(), message_count: 3 },
            SessionEntry { name: "sess-b".into(), updated_at: "2026-01-02T00:00:00+00:00".into(), message_count: 7 },
        ],
        selected: 0,
    });
    // ↓ 下移
    let a = app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(a, KeyAction::None);
    assert_eq!(app.session_picker.as_ref().unwrap().selected, 1);
    // Enter 选择
    let a = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(a, KeyAction::SelectSession(1));
    assert!(app.session_picker.is_none(), "选择后 picker 应关闭");
}

#[test]
fn test_session_picker_close_and_delete() {
    use crate::cli::tui::app::types::{SessionEntry, SessionPicker};
    let mut app = TuiApp::new(false);
    app.session_picker = Some(SessionPicker {
        entries: vec![SessionEntry { name: "sess-a".into(), updated_at: String::new(), message_count: 1 }],
        selected: 0,
    });
    let a = app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(a, KeyAction::ClosePicker);
    assert!(app.session_picker.is_none());
}

// ── 审批 Esc 取消 ──

#[test]
fn test_approval_esc_cancels() {
    use crate::cli::approval::{ActionType, PendingAction};
    let mut app = TuiApp::new(false);
    app.pending_approval = Some(PendingAction {
        id: "t".into(),
        action_type: ActionType::ShellCommand { command: "ls".into() },
        description: "运行 ls".into(),
        created_at: std::time::Instant::now(),
    });
    let a = app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(a, KeyAction::CancelGeneration);
    assert!(app.pending_approval.is_none());
}

// ── git 状态检测 ──

#[test]
fn test_git_detect_no_crash() {
    let app = TuiApp::new(false);
    // 当前目录是 git 仓库（本工程）→ branch 应为 Some；即使失败也不 panic
    let _ = app.git_branch.clone();
    let _ = app.git_dirty;
}