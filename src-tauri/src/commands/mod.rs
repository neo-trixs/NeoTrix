//! V2 Tauri 命令 — NeoTrix V2 架构的 Tauri 后端
//!
//! 分为 6 组:
//! - brain: ReasoningBrain 交互
//! - session: 会话管理
//! - agent: Agent 控制
//! - project: 项目文件
//! - mcp: MCP 工具
//! - pty: PTY 终端管理

pub mod pty;
pub mod types;
pub mod proxy_cmds;
pub mod brain_cmds;
pub mod session_cmds;
pub mod agent_cmds;
pub mod project_cmds;
pub mod diff_cmds;
pub mod perms_cmds;
pub mod mcp_cmds;
pub mod sync_cmds;

// Re-exports for convenience (frontend/tauri registration)
pub use types::*;
pub use proxy_cmds::*;
pub use brain_cmds::*;
pub use session_cmds::*;
pub use agent_cmds::*;
pub use project_cmds::*;
pub use diff_cmds::*;
pub use perms_cmds::*;
pub use mcp_cmds::*;
pub use sync_cmds::*;

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;
    use neotrix::neotrix::provider::LlmProviderType;

    // ===== parse_git_diff =====

    #[test]
    fn test_parse_git_diff_empty() {
        let blocks = parse_git_diff("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_git_diff_added_removed() {
        let input = "+added line\n-removed line\n unchanged line\n";
        let blocks = parse_git_diff(input);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].r#type, "added");
        assert_eq!(blocks[0].content, "added line");
        assert_eq!(blocks[1].r#type, "removed");
        assert_eq!(blocks[1].content, "removed line");
        assert_eq!(blocks[2].r#type, "unchanged");
    }

    #[test]
    fn test_parse_git_diff_skips_metadata() {
        let input = "\
diff --git a/file b/file
index abc..def 100644
--- a/file
+++ b/file
@@ -1,3 +1,4 @@
+new
 old
\\ No newline at end of file
";
        let blocks = parse_git_diff(input);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].content, "new");
        assert_eq!(blocks[0].r#type, "added");
        assert_eq!(blocks[1].content, " old");
        assert_eq!(blocks[1].r#type, "unchanged");
    }

    #[test]
    fn test_parse_git_diff_double_prefix() {
        let input = "+++keep\n---keep\n";
        let blocks = parse_git_diff(input);
        assert_eq!(blocks.len(), 0);
    }

    // ===== session_create =====

    #[test]
    fn test_session_create() {
        let info = session_create("test-session".into());
        assert_eq!(info.name, "test-session");
        assert_eq!(info.message_count, 0);
        assert!(info.created > 0);
        assert!(info.id.starts_with("s-"));
    }

    #[test]
    fn test_session_create_empty_name() {
        let info = session_create("".into());
        assert_eq!(info.name, "");
        assert!(info.id.starts_with("s-"));
    }

    // ===== session_list =====

    #[test]
    fn test_session_list_returns_default() {
        let sessions = session_list();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "default");
        assert_eq!(sessions[0].name, "默认会话");
    }

    // ===== payload_to_provider_config =====

    #[test]
    fn test_payload_to_provider_config_openai() {
        let payload = ProviderConfigPayload {
            id: "openai".into(),
            name: "OpenAI".into(),
            model: "gpt-4".into(),
            api_key: "sk-test".into(),
            base_url: Some("https://api.openai.com".into()),
            learning_rate: 0.01,
        };
        let config = payload_to_provider_config(&payload);
        assert_eq!(config.provider_type, LlmProviderType::OpenAI);
        assert_eq!(config.api_key, Some("sk-test".into()));
        assert_eq!(config.model, Some("gpt-4".into()));
    }

    #[test]
    fn test_payload_to_provider_config_anthropic() {
        let payload = ProviderConfigPayload {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            model: "claude-3".into(),
            api_key: "sk-ant-test".into(),
            base_url: None,
            learning_rate: 0.05,
        };
        let config = payload_to_provider_config(&payload);
        assert_eq!(config.provider_type, LlmProviderType::Anthropic);
    }

    #[test]
    fn test_payload_to_provider_config_unknown_defaults_openai() {
        let payload = ProviderConfigPayload {
            id: "unknown-provider".into(),
            name: "Unknown".into(),
            model: "some-model".into(),
            api_key: "".into(),
            base_url: None,
            learning_rate: 0.0,
        };
        let config = payload_to_provider_config(&payload);
        assert_eq!(config.provider_type, LlmProviderType::OpenAI);
        assert_eq!(config.api_key, Some("".into()));
    }

    // ===== ProxyStatus default =====

    #[test]
    fn test_proxy_status_default() {
        let s = ProxyStatus::default();
        assert!(!s.running);
        assert_eq!(s.mode, "off");
        assert_eq!(s.port, 11080);
    }

    // ===== cmd_session_* (LazyLock statics) =====

    #[test]
    fn test_cmd_session_create_and_list() {
        let id = cmd_session_create("cmd-test".into()).unwrap();
        assert!(!id.is_empty());

        let list = cmd_session_list().unwrap();
        assert!(list.iter().any(|s| s.id == id));
    }

    #[test]
    fn test_cmd_session_switch_found() {
        let id = cmd_session_create("switch-test".into()).unwrap();
        assert!(cmd_session_switch(id).is_ok());
    }

    #[test]
    fn test_cmd_session_switch_not_found() {
        assert!(cmd_session_switch("nonexistent-id".into()).is_err());
    }

    #[test]
    fn test_cmd_session_delete() {
        let id = cmd_session_create("delete-test".into()).unwrap();
        assert!(cmd_session_delete(id.clone()).is_ok());

        let list = cmd_session_list().unwrap();
        assert!(!list.iter().any(|s| s.id == id));
    }

    #[test]
    fn test_cmd_session_delete_nonexistent() {
        assert!(cmd_session_delete("ghost".into()).is_ok());
    }

    // ===== cmd_agent_* =====

    #[test]
    fn test_cmd_agent_start_stop_status() {
        assert!(cmd_agent_start("test agent task".into()).is_ok());

        let status = cmd_agent_status().unwrap();
        assert!(status.running);
        assert_eq!(status.current_task, Some("test agent task".into()));

        assert!(cmd_agent_stop().is_ok());
        let status = cmd_agent_status().unwrap();
        assert!(!status.running);
        assert!(status.current_task.is_none());
    }

    #[test]
    fn test_cmd_agent_status_initial() {
        // Reset state to avoid ordering dependency with test_cmd_agent_start_stop_status
        let _ = cmd_agent_stop();
        let status = cmd_agent_status().unwrap();
        assert!(!status.running);
    }

    // ===== cmd_permission_* =====

    #[test]
    fn test_cmd_permission_request_and_approve() {
        let req = cmd_permission_request("read".into(), "/tmp/test".into()).unwrap();
        assert_eq!(req.action, "read");
        assert_eq!(req.target, "/tmp/test");
        assert!(req.id.starts_with("perm-"));
        assert!(req.timestamp > 0);

        assert!(cmd_permission_approve(req.id.clone()).is_ok());
    }

    #[test]
    fn test_cmd_permission_request_and_deny() {
        let req = cmd_permission_request("write".into(), "/etc/config".into()).unwrap();
        assert!(cmd_permission_deny(req.id.clone()).is_ok());
    }

    #[test]
    fn test_cmd_permission_approve_nonexistent() {
        assert!(cmd_permission_approve("perm-nonexistent".into()).is_err());
    }

    #[test]
    fn test_cmd_permission_deny_nonexistent() {
        assert!(cmd_permission_deny("perm-nonexistent".into()).is_err());
    }
}
