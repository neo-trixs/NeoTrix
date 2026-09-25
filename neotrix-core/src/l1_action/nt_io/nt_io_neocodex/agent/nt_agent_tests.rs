// ── NeoCodex agent tests (moved verbatim from agent.rs; explicit imports added per split discipline) ──

use super::*;
use super::super::provider::ProviderInfo;
use super::super::wire::NeoCodexAttachment;
use super::super::provider::{ModelCapability, NeoCodexMode};
use super::super::wire::WireEvent;
use crate::l1_action::nt_io::nt_io_provider::types::{LlmRequest, Message, Role};
use base64::Engine as _;

    #[test]
    fn test_agent_state() {
        let state = AgentState::new();
        assert_eq!(state.mode, NeoCodexMode::Agent);
        assert_eq!(state.turn_count, 0);
    }

    #[test]
    fn test_dual_mode_toggle() {
        let mut agent = NeoCodexAgent::new("test-session");
        assert_eq!(agent.state.mode, NeoCodexMode::Agent);
        agent.toggle_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Shell);
        agent.toggle_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Agent);
        agent.set_plan_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Plan);
    }

    #[test]
    fn test_budget_react_messages_evicts_oldest() {
        let mut messages = vec![
            Message::new(Role::System, "system prompt"),
            Message::new(Role::User, "first request"),
            Message::assistant_with_calls("tool call a", vec![]),
            Message::tool("a very long tool result that blows any budget", "call-0"),
        ];
        NeoCodexAgent::budget_react_messages(&mut messages, 1);
        assert_eq!(messages[0].role, Role::System);
        assert_eq!(messages.last().unwrap().role, Role::Tool);
        assert!(messages.len() < 4);
    }

    #[test]
    fn test_build_messages_preserves_summary_and_tool_roles() {
        // Regression 1: Layer-4 distilled "summary" turns were mapped to
        // Role::System and skipped, silently discarding the compaction context.
        // Regression 2: "tool" history turns were mapped to Role::User, breaking
        // the ReAct message protocol (tool results must be Role::Tool).
        let mut agent = NeoCodexAgent::new("role-test");
        agent.context.push("system", "the system prompt".into(), 5);
        agent.context.push("user", "question".into(), 10);
        agent.context.push("assistant", "thinking".into(), 10);
        agent.context.push("tool", "tool output".into(), 10);
        agent
            .context
            .push("summary", "distilled earlier context".into(), 10);

        let messages = NeoCodexAgent::build_messages(&agent, "current input");
        let summary_msg = messages.iter().find(|m| m.content.contains("distilled"));
        let tool_msg = messages.iter().find(|m| m.content.contains("tool output"));

        assert!(
            summary_msg.is_some(),
            "summary turn must survive into messages"
        );
        assert_eq!(summary_msg.unwrap().role, Role::Assistant);
        assert_eq!(tool_msg.unwrap().role, Role::Tool);
        assert_eq!(messages.last().unwrap().role, Role::User);
        assert_eq!(messages.last().unwrap().content, "current input");
        assert!(
            messages.iter().filter(|m| m.role == Role::System).count() == 1,
            "only the real system prompt is kept"
        );
    }

    #[test]
    fn test_agent_process_basic() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut agent = NeoCodexAgent::new("test-agent");
        rt.block_on(async {
            let response = agent.process("Hello, NeoCodex!").await;
            assert!(!response.is_empty());
            assert_eq!(agent.state.turn_count, 1);
        });
    }

    #[test]
    fn test_goal_lifecycle() {
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Fix the bug", 5);
        assert!(agent.state.goal_active);
    }

    #[test]
    fn test_goal_wire_id_matches_queue_id() {
        // Regression: add_goal recorded a wire GoalUpdate id derived from
        // turn_count, while GoalQueue::add generated a different id for the
        // queue entry — consumers correlating wire records with queue goals
        // would never match. The wire id now comes from the queue entry.
        let mut agent = NeoCodexAgent::new("goal-id-test");
        agent.add_goal("Goal one", 3);
        agent.add_goal("Goal two", 3);
        let queued: Vec<&String> = agent.goals.goals.iter().map(|g| &g.id).collect();
        assert_eq!(queued.len(), 2);
        assert_ne!(
            queued[0], queued[1],
            "distinct goals must have distinct ids"
        );
        for gid in queued {
            assert!(
                agent.wire.events.iter().any(|e| matches!(
                    e, WireEvent::GoalUpdate { id, .. } if id == gid
                )),
                "wire event must use the same id as the queue entry"
            );
        }
    }

    #[test]
    fn test_goal_completes_and_resets_active() {
        // Regression: check_goals was never wired into the turn loop,
        // iterations never incremented, and goal_active never reset — the
        // goal queue could only grow, never progress.
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Fix the bug", 2);
        assert!(agent.state.goal_active);

        // 1st call: promotes queued goal to active, iterations 0→1, not complete.
        assert!(agent.check_goals().is_none());
        assert_eq!(agent.goals.active.as_ref().unwrap().iterations, 1);

        // 2nd call: iterations 1→2 == max, completes.
        assert!(
            agent.check_goals().is_none(),
            "single goal: completion returns None (no next)"
        );
        assert!(agent.goals.active.is_none());
        assert!(
            !agent.state.goal_active,
            "goal_active must reset once queue drains"
        );
        assert_eq!(agent.goals.completed.len(), 1);
    }

    #[test]
    fn test_chained_goals_advance_to_next() {
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Goal A", 1);
        agent.add_goal("Goal B", 1);
        assert!(agent.state.goal_active);

        // Call 1: promotes A to active, increments 0→1 == max, completes A and
        // returns the description of the next goal B.
        assert_eq!(agent.check_goals().unwrap_or_default(), "Goal B");
        assert_eq!(agent.goals.completed.len(), 1);

        // Call 2: B is already active, increments 0→1 == max, completes B.
        // No next goal → returns None and goal_active resets.
        assert!(agent.check_goals().is_none());
        assert_eq!(agent.goals.completed.len(), 2);
        assert!(!agent.state.goal_active);
    }

    #[test]
    fn test_extract_tool_call() {
        let content = "Let me read the file.\n\n<tool name=\"read\">Cargo.toml</tool>";
        let (name, args) = NeoCodexAgent::extract_tool_call(content).unwrap();
        assert_eq!(name, "read");
        assert_eq!(args, "Cargo.toml");
        assert!(NeoCodexAgent::extract_tool_call("no tool here").is_none());
    }

    #[test]
    fn test_react_loop_falls_back_without_provider() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-test");
            let out = agent.react_loop("hello", 2).await;
            // Default catalog (opencode stub) is not a real provider → None
            assert!(out.is_none());
        });
    }

    #[test]
    fn test_resume_session_restores_context() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("neocodex_test_resume");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("resume.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        for event in [
            WireEvent::UserMessage {
                content: "hello".into(),
                timestamp: 1,
                attachments: None,
            },
            WireEvent::AgentMessage {
                content: "hi there".into(),
                timestamp: 2,
            },
            WireEvent::ModeChange {
                from: NeoCodexMode::Agent,
                to: NeoCodexMode::Plan,
            },
        ] {
            let line = serde_json::to_string(&event).unwrap();
            writeln!(f, "{}", line).unwrap();
        }
        drop(f);

        let mut agent = NeoCodexAgent::new("resume-test");
        agent.wire.path = path;
        let n = agent.resume_session();
        assert_eq!(n, 3);
        assert_eq!(agent.state.mode, NeoCodexMode::Plan);
        assert!(agent.context.turns.len() >= 2);
        assert!(agent.state.tokens_used > 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resume_empty_session_is_noop() {
        let mut agent = NeoCodexAgent::new("resume-empty");
        agent.wire.path = std::env::temp_dir().join("neocodex_missing.jsonl");
        let n = agent.resume_session();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_react_loop_with_configured_provider() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-wired");
            agent.provider.sync_from_real();
            // Find a local/resolvable provider (ollama first in catalog)
            if let Some(idx) = agent
                .provider
                .providers
                .iter()
                .position(|p| p.name == "ollama")
            {
                agent.provider.active = idx;
                // to_llm_provider succeeds for ollama even without network (request only)
                let provider = agent.provider.to_llm_provider();
                assert!(provider.is_some());
            }
        });
    }

    #[test]
    fn test_react_loop_stream_no_provider_is_noop() {
        // D-streaming closure: without a resolvable provider the streaming
        // loop must not panic and must return an error outcome (F1: caller
        // surfaces it via event, never persists it as a valid answer).
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-stream-empty");
            agent.provider.providers.clear();
            agent.provider.providers.push(ProviderInfo::default());
            agent.provider.active = 0;
            let mut seen = Vec::new();
            let mut tools = Vec::new();
            let result = agent
                .react_loop_stream(
                    "hi",
                    3,
                    |t| {
                        seen.push(t.to_string());
                        true
                    },
                    |n, _, _, _, _| {
                        tools.push(n.to_string());
                        true
                    },
                )
                .await;
            assert!(result.content.is_none());
            assert!(result.error.is_some(), "provider-less stream must error");
            assert!(seen.is_empty());
            assert!(tools.is_empty());
        });
    }

    #[test]
    fn test_react_loop_stream_provider_supports_streaming() {
        // All real providers implement stream_complete; verify the trait is
        // reachable through the resolved provider (no network needed).
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-stream-wired");
            agent.provider.sync_from_real();
            if let Some(idx) = agent
                .provider
                .providers
                .iter()
                .position(|p| p.name == "ollama")
            {
                agent.provider.active = idx;
                let provider = agent.provider.to_llm_provider();
                assert!(provider.is_some());
                let provider = provider.unwrap();
                // Building a stream request must not panic; a stopped local
                // server yields Err(LlmError) which is the expected path.
                let request = LlmRequest {
                    model: "test".into(),
                    messages: vec![Message::new(Role::User, "hi")],
                    temperature: None,
                    max_tokens: 16,
                    tools: vec![],
                    image_data: None,
                    thinking_budget: None,
                    provider_params: Default::default(),
                    constraint_json: None,
                    structured_output: None,
                    cacheable_prefix_tokens: None,
                };
                let _ = provider.stream_complete(&request).await;
            }
        });
    }

    #[test]
    fn test_tool_grounding_records_calls() {
        // D25 closure: record_tool_result must be invoked on real tool execution
        let mut agent = NeoCodexAgent::new("grounding");
        agent.tool_grounding.record_tool_result("read", true, true);
        agent.tool_grounding.record_tool_result("read", true, false);
        assert_eq!(agent.tool_grounding.total_calls, 2);
        assert!(agent.tool_grounding.degraded_tools().len() <= 1);
        let report = agent.health_report();
        assert_eq!(report.tool_call_count, agent.state.tool_call_count, "health report mirrors state tool calls");
    }

    #[test]
    fn test_build_request_bridges_image_for_text_only_model() {
        // A real 4x4 PNG encoded as base64 → a valid VisionBridge decode target.
        let png = {
            use image::RgbImage;
            let mut buf = std::io::Cursor::new(Vec::new());
            let img = RgbImage::from_pixel(4, 4, image::Rgb([255, 0, 0]));
            image::DynamicImage::ImageRgb8(img)
                .write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode");
            buf.into_inner()
        };
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);

        let mut agent = NeoCodexAgent::new("vision-bridge");
        // Force a text-only active provider (default catalog: opencode stub, no Vision cap).
        agent.wire.record(WireEvent::UserMessage {
            content: "describe the image".into(),
            timestamp: 1,
            attachments: Some(vec![NeoCodexAttachment {
                name: "shot.png".into(),
                size: png.len() as u64,
                mime_type: "image/png".into(),
                data: Some(b64.clone()),
            }]),
        });
        let messages = vec![
            Message::new(Role::System, "system"),
            Message::new(Role::User, "describe the image"),
        ];
        let req = agent
            .build_request(messages.clone())
            .expect("request built");
        // Text-only model → image must be bridged into the user message, image_data None.
        assert!(
            req.image_data.is_none(),
            "image_data must be dropped for text-only"
        );
        let user_msg = req.messages.iter().find(|m| m.role == Role::User).unwrap();
        assert!(
            user_msg.content.contains("<image_evidence>"),
            "evidence must be injected: {}",
            user_msg.content
        );
        assert!(user_msg.content.contains("dimensions: 4x4"));
        // image_data None → data-URI wrap not applied (no vision).
    }

    #[test]
    fn test_build_request_keeps_image_for_vision_model() {
        let mut agent = NeoCodexAgent::new("vision-native");
        agent.provider.add_provider(ProviderInfo {
            name: "vision".into(),
            model: "gpt-4o".into(),
            capabilities: vec![ModelCapability::Vision],
            context_limit: 100_000,
            cost_per_m_input: 1.0,
            cost_per_m_output: 1.0,
        });
        agent.provider.active = 1;
        let b64 = "iVBORw0KGgo=";
        agent.wire.record(WireEvent::UserMessage {
            content: "describe".into(),
            timestamp: 1,
            attachments: Some(vec![NeoCodexAttachment {
                name: "shot.png".into(),
                size: 10,
                mime_type: "image/png".into(),
                data: Some(b64.into()),
            }]),
        });
        let messages = vec![Message::new(Role::User, "describe")];
        let req = agent.build_request(messages).expect("request built");
        // Vision-capable model → raw base64 upgraded to a data URI and preserved.
        let uri = req
            .image_data
            .expect("image_data preserved for vision model");
        assert!(
            uri.starts_with("data:image/"),
            "data URI expected, got {uri}"
        );
        assert!(!req.messages[0].content.contains("<image_evidence>"));
    }
