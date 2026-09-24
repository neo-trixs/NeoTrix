//! agent — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;

use super::{err, tokio_runtime};
use super::brain::ensure_provider_env_from_config;

/// NT-AGENT 模式 — NeoTrix 作为主体的对话驱动循环。
///
/// 架构目标：LLM 降级为后端能力（`LlmProvider`），决策循环由 Rust 的
/// `AgentLoop` 持有。本入口：
///   1. 初始化 GatewayV2（provider 路由/熔断/限流）
///   2. 装配 MCP 原生工具（ToolOrchestrator → AgentLoop 工具集）
///   3. 启动交互 REPL：每轮 `loop_.turn(input)` 驱动 用户→LLM→工具→回答
pub fn run_agent_mode(profile: &str) {
    use neotrix::agent::tool::mcp::{McpToolDef, McpTransport};
    use neotrix::agent::tool::McpRegistry;
    use neotrix::l1_action::nt_io::nt_io_agent_loop::AgentLoop;
    use neotrix::l1_action::nt_io::nt_io_provider::factory::create_gateway_async;
    use std::io::{self, Write};

    const NT_CORE_SYSTEM_PROMPT: &str = "\
You are NT-CORE, the orchestrating brain of the NeoTrix system. \
You hold state, route work, and decide. The language model you are part of is a \
backend reasoning engine you call — not your master. Answer the user directly. \
You have tools available; call them when they help. Be concise and evidence-first.";

    let rt = tokio_runtime();
    rt.block_on(async {
        ensure_provider_env_from_config();

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![McpToolDef {
            name: "neotrix_info".to_string(),
            description: "NeoTrix MCP system info".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Local {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({"type": "object"}),
            schema_version: None,
            ..Default::default()
        }];
        builtin_tools.extend(Vec::<neotrix::agent::tool::mcp::McpToolDef>::new());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);

        // 意识核心能力面: src/cli/commands 已删除, awareness_core_tools() 为空;
        // 工具面由 MCP 注册表提供, 意图路由由 nt_auto_orchestrator 分类。
        let mut tools = mcp_registry.as_native_tools();
        tools.extend(neotrix::l5_cognition::nt_core::nt_io_awareness_core::awareness_core_tools());

        let gateway = create_gateway_async().await;
        let default_model = std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| {
            let cfg = neotrix::config::NeoTrixConfig::load();
            cfg.default_model
                .clone()
                .unwrap_or_else(|| "default".to_string())
        });

        let mut loop_ = AgentLoop::new(Arc::new(gateway), &default_model, NT_CORE_SYSTEM_PROMPT)
            .with_tools(tools);
        let _ = profile;

        println!("╭─ NeoTrix Agent Loop ─────────────────────────────╮");
        println!("│  NT-CORE 作为主体 · LLM 作为后端推理引擎        │");
        println!(
            "│  model: {} · tools: {}          │",
            loop_.model(),
            loop_.tool_count()
        );
        println!("│  /exit 退出 · /tools 查看工具 · /hist 查看历史  │");
        println!("╰──────────────────────────────────────────────────╯");

        loop {
            print!("\n❯ ");
            io::stdout().flush().unwrap_or(());
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = input.trim();
                    match trimmed {
                        "/exit" | "/q" => {
                            println!("Exiting.");
                            break;
                        }
                        "/tools" => {
                            for t in loop_.tool_count()..loop_.tool_count() {
                                let _ = t;
                            }
                            println!("{} tools registered", loop_.tool_count());
                            for inv in &loop_.tool_log {
                                println!(
                                    "  {} → {}: {}",
                                    if inv.success { "✓" } else { "✗" },
                                    inv.name,
                                    inv.output
                                );
                            }
                        }
                        "/hist" => {
                            println!("{} messages in history", loop_.history_len());
                        }
                        _ if !trimmed.is_empty() => match loop_.turn(trimmed).await {
                            Ok(response) => println!("\n{}", response),
                            Err(e) => eprintln!("\n{} {}", err("Error:"), e),
                        },
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    break;
                }
            }
        }
    });
}

/// NT-AGENT TUI 模式 — 基于 ratatui 的完整对话终端。
///
/// **STUB** — cli::tui 模块已移除，此函数仅打印错误并返回。
pub fn run_agent_tui(_profile: &str) {
    eprintln!(
        "{} TUI 模块已移除，请使用 --headless 或 web 模式",
        err("Error")
    );
}
