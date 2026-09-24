//! interactive — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::{dim, err, info, init_brain, print_brain_stats, set_default_model_from_config, success, tokio_runtime, warn};
use neotrix::config::NeoTrixConfig;
use super::{desktop, headless, standalone};

pub fn run_standalone_mode(stage: usize) {
    let rt = tokio_runtime();
    rt.block_on(async {
        standalone::run_standalone(stage).await;
    });
}

pub fn run_headless_mode(_cfg: &NeoTrixConfig, profile: &str) {
    use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;
    use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
    use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;

    use neotrix::agent::hooks::{EccHookRegistry, HookContext, HookEvent};
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::tool::mcp::{McpToolDef, McpTransport};
    use neotrix::agent::tool::McpRegistry;
    use neotrix::agent::{AgentTeam, ProcessType};
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

    let rt = tokio_runtime();
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);

        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.load_cortex();
        set_default_model_from_config(&mut agent);
        agent.init_reasoning_engine();
        agent.quality_threshold = 0.7;
        agent.auto_absorb = true;
        agent.auto_memory_iteration = true;
        agent.memory_iteration_interval = 5;

        let has_engine = agent.reasoning_engine.is_some();
        if has_engine {
            println!(
                "{}: {} {}",
                info("ReasoningEngine"),
                success("active"),
                info("(LLM connected)")
            );
        } else {
            println!(
                "{}: {}",
                warn("ReasoningEngine"),
                warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)")
            );
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        // E2 观测: SkillsEngine load 包裹 (纯观测, 不改变控制流/返回值/错误路径)
        let e2_start_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let skill_count = skills_engine.init().len();
        {
            let e2_end_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(e2_start_ms);
            let mut e2_crystal =
                neotrix::neotrix::nt_crystal_core::crystal_state::CrystalState::new("entry");
            e2_crystal.record_execution(
                "skills_engine.init".to_string(),
                "entry".to_string(),
                format!("{} local skills loaded", skill_count),
                true,
                e2_end_ms.saturating_sub(e2_start_ms),
            );
        }
        // TODO(E2-next): Evolver 反馈接线 — SkillCandidate.performance_history 不在作用域, 待统一通道接入 (不跨文件新建依赖)
        println!(
            "{}: {} ",
            info("SkillsEngine"),
            success(format!("{} local skills loaded", skill_count))
        );
        println!(
            "  -> {} /skills list to browse, /skills ecc <id> to load from ECC community",
            info("/skills")
        );

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

        let mut orchestrator = neotrix::agent::tool::ToolOrchestrator::default();
        orchestrator.register_native_all(mcp_registry.as_native_tools());
        // set_tool_orchestrator removed with cli::commands
        println!(
            "{}: {} native MCP tools absorbed via McpToolAdapter",
            info("ToolOrchestrator"),
            success(mcp_registry.tool_count().to_string())
        );
        // set_mcp_registry removed with cli::commands
        println!(
            "{}: {} ({})",
            info("McpRegistry"),
            success("ready"),
            info("use /mcp list")
        );
        let mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = EccHookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!(
            "{}: {} {}",
            info("EccHookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)")
        );

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = EccHookRegistry::check_blocked(&hook_actions) {
            eprintln!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
        bg_goal_loop.load();
        let agent_team = Arc::new(Mutex::new(AgentTeam::new(
            "default",
            ProcessType::Sequential,
        )));
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);
        tokio::spawn(async move {
            let mut bg = BackgroundLoop::new(bg_agent)
                .with_goal_loop(bg_goal_loop)
                .with_nt_world_model(WorldModelV2::new(8, 64));
            // 插件目录 HMR watch (revertible_effects C4 接线): ~/.neotrix/plugins/
            // 事务化 load_batch/hot_reload 被真实后台消费。目录不存在则先创建。
            if let Some(home) = dirs::home_dir() {
                let plugin_dir = home.join(".neotrix").join("plugins");
                if std::fs::create_dir_all(&plugin_dir).is_ok() {
                    bg = bg.with_plugin_watch(plugin_dir);
                } else {
                    log::warn!("[entry] cannot create plugin dir; plugin HMR disabled");
                }
            }
            #[cfg(feature = "stealth-net")]
            {
                bg = bg.with_world_consciousness();
            }
            bg.start().await;
        });

        // Session Recovery
        {
            use neotrix::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager;
            let recovery_mgr = SessionRecoveryManager::new("default").with_auto_recover(true);
            if let Some(snapshot) = recovery_mgr.load_latest_snapshot() {
                println!(
                    "{}: {} (session #{}, {} messages)",
                    info("SessionRecovery"),
                    success("restored"),
                    snapshot.session_id,
                    snapshot.message_count
                );
            } else {
                println!(
                    "{}: {} — no previous session found",
                    info("SessionRecovery"),
                    dim("fresh start")
                );
            }
        }

        // AGENTS.md
        {
            use neotrix::l1_action::nt_io::nt_io_agents_md::AgentsMdReader;
            let agents_reader = AgentsMdReader::new();
            if let Ok(rules) = agents_reader.load_project_rules(std::path::Path::new(".")) {
                if !rules.is_empty() {
                    println!(
                        "{}: {} ({} sections)",
                        info("AGENTS.md"),
                        success("loaded"),
                        rules.sections.len()
                    );
                } else {
                    println!("{}: {} — no rules found", info("AGENTS.md"), dim("skipped"));
                }
            }
        }

        let sp = indicatif::ProgressBar::new_spinner();
        match indicatif::ProgressStyle::default_spinner().template("{spinner:.blue} {msg}") {
            Ok(style) => sp.set_style(style),
            Err(e) => eprintln!("{}: invalid spinner template: {}", err("Error"), e),
        }
        sp.set_message("starting headless mode...");
        headless::run_headless(agent, skills_engine, hook_registry, mcp_registry).await;
        sp.finish_and_clear();
    });
}

// ── T14b 语义迁移桥接（A 侧，调用方侧）：team::AgentRole → L1 正典 AgentCard ──
// 旧 struct 定义与旧调用点不动（E2 接线）；正典文件只读。
// 映射：id＝调用方传入；name←role.name；description←role.backstory；
// tags←role.tools 克隆；role_chain←vec![role.role]；其余走 AgentCard::new 正典默认。
// 语义缺口：goal 字段无处可放（正典无对应位），本次丢弃，E2 需裁决去向。
pub fn team_role_to_card(
    id: &str,
    role: &neotrix::agent::team::AgentRole,
) -> neotrix::l1_action::nt_infra_agent_card::AgentCard {
    let mut card =
        neotrix::l1_action::nt_infra_agent_card::AgentCard::new(id, &role.name, &role.backstory);
    card.tags = role.tools.clone();
    card.role.role_chain = vec![role.role.clone()];
    card
}

pub fn run_interactive(cfg: &NeoTrixConfig, profile: &str) {
    run_interactive_with_ephemeral(cfg, profile, false)
}

pub fn run_interactive_with_ephemeral(cfg: &NeoTrixConfig, profile: &str, ephemeral: bool) {
    use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;
    use neotrix::l5_cognition::nt_mind::nt_mind::panorama_pipeline::PanoramaPipeline;
    use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
    use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;

    use neotrix::agent::hooks::{EccHookRegistry, HookContext, HookEvent};
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::tool::mcp::{McpToolDef, McpTransport};
    use neotrix::agent::tool::McpRegistry;
    use neotrix::agent::{AgentRole, AgentTeam, ProcessType};
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

    if let Some(level) = &cfg.log_level {
        std::env::set_var("RUST_LOG", format!("neotrix={}", level));
    }

    let rt = tokio_runtime();
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);

        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.load_cortex();
        set_default_model_from_config(&mut agent);
        agent.init_reasoning_engine();
        agent.quality_threshold = 0.7;
        agent.auto_absorb = true;
        agent.auto_memory_iteration = true;
        agent.memory_iteration_interval = 5;

        let has_engine = agent.reasoning_engine.is_some();
        if has_engine {
            println!(
                "{}: {} {}",
                info("ReasoningEngine"),
                success("active"),
                info("(LLM connected)")
            );
        } else {
            println!(
                "{}: {}",
                warn("ReasoningEngine"),
                warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)")
            );
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        // E2 观测: SkillsEngine load 包裹 (纯观测, 不改变控制流/返回值/错误路径)
        let e2_start_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let skill_count = skills_engine.init().len();
        {
            let e2_end_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(e2_start_ms);
            let mut e2_crystal =
                neotrix::neotrix::nt_crystal_core::crystal_state::CrystalState::new("entry");
            e2_crystal.record_execution(
                "skills_engine.init".to_string(),
                "entry".to_string(),
                format!("{} local skills loaded", skill_count),
                true,
                e2_end_ms.saturating_sub(e2_start_ms),
            );
        }
        // TODO(E2-next): Evolver 反馈接线 — SkillCandidate.performance_history 不在作用域, 待统一通道接入 (不跨文件新建依赖)
        println!(
            "{}: {} ",
            info("SkillsEngine"),
            success(format!("{} local skills loaded", skill_count))
        );
        println!(
            "  -> {} /skills list to browse, /skills ecc <id> to load from ECC community",
            info("/skills")
        );

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

        let mut orchestrator = neotrix::agent::tool::ToolOrchestrator::default();
        orchestrator.register_native_all(mcp_registry.as_native_tools());
        // set_tool_orchestrator and set_mcp_registry removed with cli::commands
        println!(
            "{}: {} ({})",
            info("McpRegistry"),
            success("ready"),
            info("use /mcp list")
        );
        let _mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = EccHookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!(
            "{}: {} {}",
            info("EccHookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)")
        );

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = EccHookRegistry::check_blocked(&hook_actions) {
            eprintln!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let _skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry: Arc<RwLock<EccHookRegistry>> = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
        bg_goal_loop.load();
        if bg_goal_loop.active_goal.is_some() {
            println!(
                "{} {}",
                info("[bg]"),
                info("Restored background goal from ~/.neotrix/goals.json")
            );
        }

        let agent_team = Arc::new(Mutex::new(AgentTeam::new(
            "default",
            ProcessType::Sequential,
        )));
        {
            let mut team = agent_team.lock().unwrap_or_else(|e| e.into_inner());
            // TODO(T14-next): E2 接线后删除旧构造（goal/tools 去向待 E2 裁决，见 team_role_to_card 注记）
            let legacy_role = AgentRole {
                name: "planner".into(),
                role: "Task Planner".into(),
                goal: "Break down complex tasks into sub-tasks".into(),
                backstory: "Strategic planner with systems thinking".into(),
                tools: vec!["reason".into()],
            };
            // T14 语义迁移：并行产出正典卡并注册（行为不变，旧路径保留）
            let card = team_role_to_card("planner", &legacy_role);
            neotrix::l1_action::nt_infra_agent_card::agent_card_register(card);
            team.add_agent(legacy_role);
        }
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);

        let mut panorama = PanoramaPipeline::new();
        let bg_kb: Option<
            std::sync::Arc<neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>,
        >;
        if let Ok(kb) = neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
            let kb = std::sync::Arc::new(kb);
            panorama.attach_kb(kb.clone());
            bg_kb = Some(kb);
        } else {
            bg_kb = None;
        }
        tokio::spawn(async move {
            let mut bg = BackgroundLoop::new(bg_agent)
                .with_goal_loop(bg_goal_loop)
                .with_nt_world_model(WorldModelV2::new(8, 64))
                .with_panorama(panorama)
                .with_kb(bg_kb)
                .with_exploration_pipeline(std::path::PathBuf::from("."))
                .with_knowledge_chain(std::path::PathBuf::from("."))
                .with_agent_discovery(42069);
            #[cfg(feature = "stealth-net")]
            {
                bg = bg.with_world_consciousness();
            }
            bg.start().await;
        });

        // PreToolUse hook — entering interactive TUI session
        {
            let hr = hook_registry.read().await;
            let mut pre_ctx = HookContext::new(HookEvent::PreToolUse);
            pre_ctx.tool_name = Some("tui_session".to_string());
            pre_ctx.tool_input = Some("interactive_mode".to_string());
            let pre_actions = hr.execute_event(&pre_ctx);
            if let Some(block_reason) = EccHookRegistry::check_blocked(&pre_actions) {
                eprintln!("Hook blocked TUI session: {}", block_reason);
            }
        }

        // Session Recovery — 加载上次会话快照
        {
            use neotrix::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager;
            let recovery_mgr = SessionRecoveryManager::new("default").with_auto_recover(true);
            if let Some(snapshot) = recovery_mgr.load_latest_snapshot() {
                println!(
                    "{}: {} (session #{}, {} messages, {} e8 states)",
                    info("SessionRecovery"),
                    success("restored"),
                    snapshot.session_id,
                    snapshot.message_count,
                    snapshot.e8_state_sequence.len()
                );
            } else {
                println!(
                    "{}: {} — no previous session found",
                    info("SessionRecovery"),
                    dim("fresh start")
                );
            }
        }

        // AGENTS.md — 扫描项目规则文件
        {
            use neotrix::l1_action::nt_io::nt_io_agents_md::AgentsMdReader;
            let agents_reader = AgentsMdReader::new();
            if let Ok(rules) = agents_reader.load_project_rules(std::path::Path::new(".")) {
                if !rules.is_empty() {
                    let sections: Vec<&str> = rules.sections.keys().map(|k| k.as_str()).collect();
                    println!(
                        "{}: {} ({}) — {} sections: {}",
                        info("AGENTS.md"),
                        success("loaded"),
                        rules
                            .source_files
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        rules.sections.len(),
                        sections.join(", ")
                    );
                } else {
                    println!(
                        "{}: {} — no rules found in current directory",
                        info("AGENTS.md"),
                        dim("skipped")
                    );
                }
            }
        }

        desktop::run_tui(agent, ephemeral).await;

        // PostToolUse hook — exiting TUI session
        {
            let hr = hook_registry.read().await;
            let mut post_ctx = HookContext::new(HookEvent::PostToolUse);
            post_ctx.tool_name = Some("tui_session".to_string());
            post_ctx.tool_output = Some("TUI session ended".to_string());
            let _ = hr.execute_event(&post_ctx);
        }
    });
}
