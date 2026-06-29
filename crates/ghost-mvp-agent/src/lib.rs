pub mod a2a;
pub mod config;
pub mod engine;
#[cfg(not(test))]
pub mod scheduler;
pub mod state;

#[cfg(test)]
mod tests {
    use super::a2a::{self, AppContext};
    use super::config::Config;
    use super::engine;
    use super::state::{AppState, Store, KpiRecord, AnalysisRecord, PublishRecord, TaskRecord};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::RwLock;

    fn unique_dir() -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("ghost_mvp_test_{}", ts))
    }

    fn test_config(data_dir: &std::path::PathBuf) -> Config {
        Config {
            data_dir: data_dir.clone(),
            ..Config::default()
        }
    }

    async fn test_context(data_dir: &std::path::PathBuf) -> Arc<AppContext> {
        let config = test_config(data_dir);
        let store = Store::new(&config.data_dir);
        Arc::new(AppContext {
            config,
            state_store: store,
            tasks: RwLock::new(HashMap::new()),
            registry_client: None,
            http_client: reqwest::Client::new(),
        })
    }

    // ── Config ──────────────────────────────────────────

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert_eq!(cfg.agent_name, "ghost-mvp");
        assert_eq!(cfg.agent_version, "1.0.0");
        assert_eq!(cfg.http_port, 8890);
        assert_eq!(cfg.discovery_port, 42069);
        assert_eq!(cfg.hui_mei_enabled, false);
        assert!(cfg.letmepost_api_key.is_none());
        assert!(cfg.devto_api_key.is_none());
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let cfg = Config::default();
        let toml_str = toml::to_string(&cfg).expect("serialize config");
        let deserialized: Config = toml::from_str(&toml_str).expect("deserialize config");
        assert_eq!(deserialized.agent_name, cfg.agent_name);
        assert_eq!(deserialized.http_port, cfg.http_port);
        assert_eq!(deserialized.hui_mei_enabled, cfg.hui_mei_enabled);
    }

    #[test]
    fn test_config_custom_toml() {
        let toml_str = r#"
agent_name = "my-custom-agent"
agent_version = "2.0.0"
http_port = 9999
discovery_port = 43000
data_dir = "/tmp/test"
schedule_analysis_interval_hours = 24
schedule_geo_audit_interval_days = 7
hui_mei_enabled = true
"#;
        let cfg: Config = toml::from_str(toml_str).expect("parse custom config");
        assert_eq!(cfg.agent_name, "my-custom-agent");
        assert_eq!(cfg.agent_version, "2.0.0");
        assert_eq!(cfg.http_port, 9999);
        assert_eq!(cfg.discovery_port, 43000);
        assert!(cfg.hui_mei_enabled);
    }

    #[test]
    fn test_config_optional_fields() {
        let toml_str = r#"
agent_name = "test"
agent_version = "1.0"
http_port = 8000
discovery_port = 8001
data_dir = "/tmp"
schedule_analysis_interval_hours = 48
schedule_geo_audit_interval_days = 14
letmepost_api_key = "sk-test-123"
devto_api_key = "devto-test-456"
"#;
        let cfg: Config = toml::from_str(toml_str).expect("parse config with optional fields");
        assert_eq!(cfg.letmepost_api_key, Some("sk-test-123".into()));
        assert_eq!(cfg.devto_api_key, Some("devto-test-456".into()));
    }

    // ── AppState ────────────────────────────────────────

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.version, "1.0.0");
        assert!(state.analyses.is_empty());
        assert!(state.publications.is_empty());
        assert!(state.tasks.is_empty());
        assert!(state.kpi_history.is_empty());
        assert_eq!(state.total_analyses, 0);
        assert_eq!(state.total_patterns, 0);
        assert_eq!(state.total_capabilities, 0);
        assert_eq!(state.total_publications, 0);
    }

    #[test]
    fn test_app_state_json_roundtrip() {
        let state = AppState {
            version: "2.0.0".into(),
            total_analyses: 10,
            total_patterns: 42,
            total_capabilities: 7,
            total_publications: 3,
            ..AppState::default()
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let deserialized: AppState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.version, "2.0.0");
        assert_eq!(deserialized.total_analyses, 10);
        assert_eq!(deserialized.total_patterns, 42);
        assert_eq!(deserialized.total_capabilities, 7);
        assert_eq!(deserialized.total_publications, 3);
    }

    #[test]
    fn test_record_serialization() {
        let record = AnalysisRecord {
            id: "abc-123".into(),
            repo_url: "https://github.com/user/repo".into(),
            repo_name: "repo".into(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            patterns_found: vec!["pattern1".into()],
            capabilities_proposed: vec!["cap1".into()],
            content_angles: vec!["angle1".into()],
            report_path: Some("/tmp/report.md".into()),
        };
        let json = serde_json::to_string(&record).expect("serialize analysis record");
        let deserialized: AnalysisRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, "abc-123");
        assert_eq!(deserialized.repo_url, "https://github.com/user/repo");
        assert!(deserialized.completed_at.is_none());
        assert_eq!(deserialized.patterns_found, vec!["pattern1"]);
    }

    #[test]
    fn test_publish_record_serialization() {
        let record = PublishRecord {
            id: "pub-1".into(),
            platform: "x".into(),
            title: "Hello world".into(),
            url: Some("https://x.com/user/status/1".into()),
            published_at: chrono::Utc::now(),
            status: "published".into(),
        };
        let json = serde_json::to_string(&record).expect("serialize publish record");
        let deserialized: PublishRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.platform, "x");
        assert_eq!(deserialized.title, "Hello world");
        assert_eq!(deserialized.status, "published");
    }

    #[test]
    fn test_task_record_serialization() {
        let record = TaskRecord {
            id: "task-1".into(),
            task_type: "analysis".into(),
            status: "completed".into(),
            created_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            result: Some(serde_json::json!({"patterns": 5})),
        };
        let json = serde_json::to_string(&record).expect("serialize task record");
        let deserialized: TaskRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.task_type, "analysis");
        assert!(deserialized.result.is_some());
    }

    #[test]
    fn test_kpi_record_serialization() {
        let record = KpiRecord {
            date: "2026-06-21".into(),
            repos_analyzed: 5,
            patterns_extracted: 23,
            capabilities_proposed: 3,
            articles_published: 1,
            geo_score: Some(0.85),
        };
        let json = serde_json::to_string(&record).expect("serialize kpi");
        let deserialized: KpiRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.repos_analyzed, 5);
        assert_eq!(deserialized.geo_score, Some(0.85));
    }

    // ── Store ───────────────────────────────────────────

    #[tokio::test]
    async fn test_store_new_and_persist() {
        let dir = unique_dir();
        let config = test_config(&dir);
        let store = Store::new(&config.data_dir);
        let state = store.read().await;
        assert_eq!(state.total_analyses, 0);
        assert!(state.analyses.is_empty());
        drop(state);

        store.persist().await;

        // Re-create store from same dir to test persistence
        let store2 = Store::new(&config.data_dir);
        let state2 = store2.read().await;
        assert_eq!(state2.total_analyses, 0);
        drop(state2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_record_analysis_and_counts() {
        let dir = unique_dir();
        let config = test_config(&dir);
        let store = Store::new(&config.data_dir);

        store
            .record_analysis(
                "https://github.com/user/repo1",
                "repo1",
                vec!["pattern-a".into(), "pattern-b".into()],
                vec!["cap-1".into()],
                vec!["angle-1".into()],
                None,
            )
            .await;

        {
            let state = store.read().await;
            assert_eq!(state.total_analyses, 1);
            assert_eq!(state.total_patterns, 2);
            assert_eq!(state.total_capabilities, 1);
            assert_eq!(state.total_publications, 0);
            assert_eq!(state.analyses.len(), 1);
            assert_eq!(state.analyses[0].repo_name, "repo1");
        }

        store
            .record_analysis(
                "https://github.com/user/repo2",
                "repo2",
                vec![],
                vec![],
                vec![],
                Some("/tmp/report.md".into()),
            )
            .await;

        {
            let state = store.read().await;
            assert_eq!(state.total_analyses, 2);
            assert_eq!(state.analyses.len(), 2);
            assert!(state.analyses[1].report_path.is_some());
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_record_publication() {
        let dir = unique_dir();
        let config = test_config(&dir);
        let store = Store::new(&config.data_dir);

        store
            .record_publication("x", "Hello world", Some("https://x.com/1".into()), "published")
            .await;

        {
            let state = store.read().await;
            assert_eq!(state.total_publications, 1);
            assert_eq!(state.publications.len(), 1);
            assert_eq!(state.publications[0].platform, "x");
        }

        store
            .record_publication("zhihu", "Test post", None, "draft")
            .await;

        {
            let state = store.read().await;
            assert_eq!(state.total_publications, 2);
            assert_eq!(state.publications[1].platform, "zhihu");
            assert!(state.publications[1].url.is_none());
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_store_bounded_cap() {
        let dir = unique_dir();
        let config = test_config(&dir);
        let store = Store::new(&config.data_dir);

        for i in 0..1100 {
            store
                .record_analysis(
                    &format!("https://github.com/user/repo{}", i),
                    &format!("repo{}", i),
                    vec![],
                    vec![],
                    vec![],
                    None,
                )
                .await;
        }

        {
            let state = store.read().await;
            assert_eq!(state.total_analyses, 1100);
            assert!(state.analyses.len() <= 1000);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Engine ──────────────────────────────────────────

    #[tokio::test]
    async fn test_generate_content_returns_placeholder() {
        let cfg = Config::default();
        let client = reqwest::Client::new();
        let result = engine::generate_content(&cfg, "AI", "x", &client).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Generated content for x on topic: AI"));
        assert!(content.contains("ghost-mvp"));
    }

    #[tokio::test]
    async fn test_publish_dry_run_when_no_api_key() {
        let cfg = Config::default();
        let client = reqwest::Client::new();
        let result = engine::publish_to_platform(&cfg, "x", "test content", &client).await;
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("[dry-run]"));
        assert!(msg.contains("x"));

        let result2 = engine::publish_to_platform(&cfg, "zhihu", "test content", &client).await;
        assert!(result2.is_ok());
        let msg2 = result2.unwrap();
        assert!(msg2.contains("[dry-run]"));
        assert!(msg2.contains("zhihu"));
        assert!(msg2.contains("HuiMei"));

        let result3 = engine::publish_to_platform(&cfg, "dev.to", "test content", &client).await;
        assert!(result3.is_ok());
        let msg3 = result3.unwrap();
        assert!(msg3.contains("[dry-run]"));
    }

    #[tokio::test]
    async fn test_publish_unsupported_platform() {
        let cfg = Config::default();
        let client = reqwest::Client::new();
        let result = engine::publish_to_platform(&cfg, "unknown-platform", "content", &client).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported platform"));
    }

    // ── A2A —─────────────────────────────────────────

    #[test]
    fn test_build_agent_card_has_expected_fields() {
        let cfg = Config::default();
        let card = a2a::build_agent_card(&cfg);
        assert_eq!(card.name, "ghost-mvp");
        assert_eq!(card.version, "1.0.0");
        assert!(card.description.contains("publishing"));
        assert!(card.url.contains("8890"));
    }

    #[test]
    fn test_build_agent_card_skills() {
        let cfg = Config {
            http_port: 9000,
            ..Config::default()
        };
        let card = a2a::build_agent_card(&cfg);
        assert!(card.url.contains("9000"));
        assert_eq!(card.skills.len(), 5);
        let skill_names: Vec<&str> = card.skills.iter().map(|s| s.id.as_str()).collect();
        assert!(skill_names.contains(&"open-source-analysis"));
        assert!(skill_names.contains(&"content-generation"));
        assert!(skill_names.contains(&"publishing"));
        assert!(skill_names.contains(&"geo-audit"));
        assert!(skill_names.contains(&"status"));
    }

    #[tokio::test]
    async fn test_process_instruction_help() {
        let dir = unique_dir();
        let ctx = test_context(&dir).await;

        let result = a2a::process_instruction(&ctx, "help").await;
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("available commands"));
        assert!(text.contains("analyze"));
        assert!(text.contains("generate"));
        assert!(text.contains("publish"));

        let result2 = a2a::process_instruction(&ctx, "?").await;
        assert!(result2.is_ok());

        let result3 = a2a::process_instruction(&ctx, "帮助").await;
        assert!(result3.is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_process_instruction_unknown() {
        let dir = unique_dir();
        let ctx = test_context(&dir).await;

        let result = a2a::process_instruction(&ctx, "foobar").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Unknown command"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_process_instruction_status() {
        let dir = unique_dir();
        let ctx = test_context(&dir).await;

        let result = a2a::process_instruction(&ctx, "status").await;
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("analyses"));
        assert!(text.contains("publications"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_process_instruction_generate() {
        let dir = unique_dir();
        let ctx = test_context(&dir).await;

        let result = a2a::process_instruction(&ctx, "generate content for x about AI trends").await;
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("x"));
        assert!(text.contains("AI trends"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_process_instruction_status_chinese() {
        let dir = unique_dir();
        let ctx = test_context(&dir).await;

        let result = a2a::process_instruction(&ctx, "状态").await;
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_process_instruction_generate_chinese() {
        let dir = unique_dir();
        let ctx = test_context(&dir).await;

        let result = a2a::process_instruction(&ctx, "生成内容 for x about 测试").await;
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
