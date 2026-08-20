
    #[test]
    fn test_basic() {
        assert!(true);
    }

    #[test]
    fn test_save_load_tui_session_roundtrip() {
        // 用隔离 base 目录验证 save/load 闭环（不污染真实 ~/.neotrix KB）。
        
        use neotrix::cli::tui::session_store::SessionStore;

        let tmp = std::env::temp_dir().join(format!("nt-tui-session-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("create tmp");

        // 用 with_base 隔离的 store 直接验证 SessionStore 落盘逻辑。
        let mut store = SessionStore::with_base(tmp.clone());
        let now = chrono::Utc::now().to_rfc3339();
        let data = neotrix::cli::tui::session_store::SessionData {
            id: "s-1".into(),
            name: "roundtrip".into(),
            messages: vec!["[user] hello".into(), "[assistant] hi there".into()],
            created_at: now.clone(),
            updated_at: now,
        };
        store.save_session("roundtrip", &data).expect("save ok");

        // 重新打开验证持久化。
        let store2 = SessionStore::with_base(tmp.clone());
        let loaded = store2.load_session("roundtrip").expect("load ok");
        assert_eq!(loaded.name, "roundtrip");
        assert_eq!(loaded.messages.len(), 2);
        assert!(loaded.messages[0].contains("hello"));
        assert!(loaded.messages[1].contains("hi there"));

        // session-logs/*.md 应已落盘。
        let md = tmp.join("session-logs").join("roundtrip.md");
        assert!(md.exists(), "session-logs markdown should exist");
        let content = std::fs::read_to_string(&md).expect("read md");
        assert!(content.contains("hello"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_shell_direct_echo() {
        // `!` 前缀直跑：echo 成功 + exit 0
        let (code, stdout, stderr) = super::run_shell_direct("echo hello-neotrix").expect("shell runs");
        assert_eq!(code, 0);
        assert_eq!(stdout, "hello-neotrix");
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_shell_direct_nonzero_exit() {
        // 非零退出码应透传
        let (code, stdout, stderr) = super::run_shell_direct("exit 3").expect("shell runs");
        assert_eq!(code, 3);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_shell_direct_stderr_captured() {
        // stderr 应被捕获而非丢弃
        let (code, _stdout, stderr) = super::run_shell_direct("echo boom >&2").expect("shell runs");
        assert_eq!(code, 0);
        assert_eq!(stderr, "boom");
    }

    #[test]
    fn test_slash_command_dispatch() {
        use neotrix::cli::tui::TuiApp;
        let mut app = TuiApp::new(true);
        use super::SlashResult;
        // /clear
        app.push_message("user", "x".into());
        assert!(matches!(super::handle_slash_tui(&mut app, "/clear", None), SlashResult::Handled));
        assert!(app.sessions[0].messages.is_empty());
        // /new
        assert!(matches!(super::handle_slash_tui(&mut app, "/new", None), SlashResult::Handled));
        assert_eq!(app.sessions.len(), 2);
        // /exit
        assert!(matches!(super::handle_slash_tui(&mut app, "/exit", None), SlashResult::Quit));
        // 未知命令透传
        assert!(matches!(super::handle_slash_tui(&mut app, "/bogus", None), SlashResult::NotHandled));
    }

    #[test]
    fn test_slash_compact_undo_redo_handled() {
        use super::SlashResult;
        use neotrix::cli::tui::TuiApp;
        let mut app = TuiApp::new(true);
        // 无 agent 时 /model 无参仅显示提示（Handled）。
        assert!(matches!(super::handle_slash_tui(&mut app, "/model", None), SlashResult::Handled));
        assert!(app.status_text.contains("用法: /model"));
        // /compact 无参（会话为空 → 提示过短）。
        assert!(matches!(super::handle_slash_tui(&mut app, "/compact", None), SlashResult::Handled));
        assert!(app.status_text.contains("无需压缩"));
        // /undo /redo /cost /status /copy /theme /export 均 Handled 且不 panic。
        assert!(matches!(super::handle_slash_tui(&mut app, "/undo", None), SlashResult::Handled));
        assert!(matches!(super::handle_slash_tui(&mut app, "/redo", None), SlashResult::Handled));
        assert!(matches!(super::handle_slash_tui(&mut app, "/cost", None), SlashResult::Handled));
        assert!(matches!(super::handle_slash_tui(&mut app, "/status", None), SlashResult::Handled));
        assert!(matches!(super::handle_slash_tui(&mut app, "/copy", None), SlashResult::Handled));
        assert!(app.status_text.contains("没有可复制"));
        assert!(matches!(super::handle_slash_tui(&mut app, "/theme gruvbox", None), SlashResult::Handled));
        assert_eq!(app.theme_name, "gruvbox");
        assert!(matches!(super::handle_slash_tui(&mut app, "/models", None), SlashResult::Handled));
        // /export 无参写入默认文件名。
        app.push_message("user", "hello".into());
        assert!(matches!(super::handle_slash_tui(&mut app, "/export", None), SlashResult::Handled));
        let path = format!("session-{}.md", app.active_session().id);
        assert!(std::path::Path::new(&path).exists(), "导出文件应存在");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_diff_slash_with_literal_text_opens_viewer() {
        use neotrix::cli::tui::TuiApp;
        let mut app = TuiApp::new(true);
        // 含换行的参数 → 视为命令行直接传入的 diff 文本（不触发 git 调用）。
        let diff_text = "diff --git a/x.rs b/x.rs\n@@ -1,2 +1,3 @@\n-old\n+new\n";
        assert!(matches!(
            super::handle_slash_tui(&mut app, &format!("/diff {}", diff_text), None),
            super::SlashResult::Handled
        ));
        assert!(app.diff_active(), "/diff 应打开 diff 查看模式");
        let viewer = app.diff_viewer.as_ref().expect("diff viewer");
        assert_eq!(viewer.blocks.len(), 1, "应解析出一个 diff block");
        assert!(!viewer.is_empty());
        // q 退出
        app.handle_key(crossterm::event::KeyCode::Char('q'), crossterm::event::KeyModifiers::NONE);
        assert!(!app.diff_active(), "q 应退出 diff 查看模式");
    }

    #[test]
    fn test_diff_slash_empty_reports_no_content() {
        use neotrix::cli::tui::TuiApp;
        let mut app = TuiApp::new(true);
        // 空 diff 文本 → 不进入查看模式，状态栏提示无内容。
        assert!(matches!(
            super::handle_slash_tui(&mut app, "/diff \n", None),
            super::SlashResult::Handled
        ));
        assert!(!app.diff_active());
        assert!(app.status_text.contains("无 diff 内容"));
    }
