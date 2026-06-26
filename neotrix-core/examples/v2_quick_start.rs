//! NeoTrix V2 — 快速入门示例
//!
//! 展示 V2 架构的核心功能：
//! - AgentTeam 多 Agent 协作 (Sequential / Hierarchical / Debate)
//! - Workflow 引擎
//! - Skills 引擎
//! - MCP Registry
//! - TUI 启动

use log;
use neotrix::agent::tools::{McpRegistry, McpToolDef, McpTransport};
use neotrix::agent::{
    AgentRole, AgentTeam, ProcessType, SkillsEngine, Workflow, WorkflowEngine, WorkflowStep,
};

fn main() {
    log::info!("╭──────────────────────────────────────╮");
    log::info!("│        NeoTrix V2 Quick Start        │");
    log::info!("╰──────────────────────────────────────╯\n");

    // 1. AgentTeam — 多 Agent 协作
    log::info!("► 1. AgentTeam (Sequential)");
    let mut team = AgentTeam::new("research-team", ProcessType::Sequential);
    team.add_agent(AgentRole {
        name: "研究员".into(),
        role: "Research Analyst".into(),
        goal: "收集和分析信息".into(),
        backstory: "资深研究员".into(),
        tools: vec!["web_search".into()],
    });
    team.add_agent(AgentRole {
        name: "写手".into(),
        role: "Content Writer".into(),
        goal: "撰写报告".into(),
        backstory: "资深写手".into(),
        tools: Vec::new(),
    });
    let results = team.run_sequential("2026 AI 发展趋势");
    log::info!(
        "   团队: {} ({} agents, {}/{} 成功)",
        team.name,
        team.agents.len(),
        results.iter().filter(|r| r.success).count(),
        results.len()
    );

    // 2. Workflow 引擎
    log::info!("\n► 2. Workflow Engine");
    let mut engine = WorkflowEngine::new();
    engine.register(Workflow {
        name: "research-pipeline".into(),
        description: "研究流水线".into(),
        steps: vec![
            WorkflowStep::Parallel {
                name: "research".into(),
                steps: vec![
                    WorkflowStep::AgentTask {
                        name: "web-search".into(),
                        task_description: "搜索最新资料".into(),
                    },
                    WorkflowStep::AgentTask {
                        name: "paper-review".into(),
                        task_description: "分析论文".into(),
                    },
                ],
            },
            WorkflowStep::AgentTask {
                name: "summarize".into(),
                task_description: "汇总结果".into(),
            },
        ],
    });
    let wf_results = engine.run("research-pipeline", "AI agents");
    log::info!(
        "   工作流: {} steps, 全部成功: {}",
        wf_results.len(),
        wf_results.iter().all(|r| r.success)
    );

    // 3. Skills 引擎
    log::info!("\n► 3. Skills Engine");
    let mut skills = SkillsEngine::new();
    let found = skills.init();
    log::info!("   发现 Skills: {} 个", found.len());

    let prompt = skills.prepare_prompt("写 Rust 代码", "你是一个 AI 助手");
    log::info!("   注入后 prompt 长度: {} 字符", prompt.len());

    // 4. MCP Registry
    log::info!("\n► 4. MCP Registry");
    let mut mcp = McpRegistry::new();
    mcp.register_stdio(
        "echo",
        "echo",
        &["hello"],
        vec![McpToolDef {
            name: "greet".into(),
            description: "打招呼".into(),
            server_name: "echo".into(),
            transport: McpTransport::Stdio {
                command: "echo".into(),
                args: vec!["hello".into()],
            },
            input_schema: serde_json::json!({}),
        }],
    );
    log::info!(
        "   注册服务: {} 个, 工具: {} 个",
        mcp.server_count(),
        mcp.tool_count()
    );
    log::info!(
        "   推荐工具 'search': {:?}",
        mcp.recommend_tools("search the web", 3)
    );

    // 5. 启动方式
    log::info!("\n► 5. 启动方式");
    log::info!("   cargo run               # TUI 模式 (默认)");
    log::info!("   cargo run -- --headless  # Headless REPL");
    log::info!("   cargo run -- --serve     # HTTP API (port 3000)");
    log::info!("   cargo tauri dev          # 桌面端 (需要 Tauri)");

    log::info!("\n╭──────────────────────────────────────╮");
    log::info!("│   NeoTrix V2 就绪!                   │");
    log::info!("╰──────────────────────────────────────╯");
}
