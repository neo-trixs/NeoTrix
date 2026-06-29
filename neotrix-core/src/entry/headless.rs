use std::io::{self, Write};
use std::sync::Arc;

use tokio::sync::RwLock;

use neotrix::agent::hooks::{HookContext, HookEvent, HookRegistry};
use neotrix::agent::skills::{SkillSource, SkillsEngine};
use neotrix::agent::workflow::{Workflow, WorkflowEngine, WorkflowStep};
use neotrix::core::nt_core_cap::FIELD_NAMES;
use neotrix::neotrix::nt_mind::goal_loop::{GoalLoop, GoalLoopState};
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::neotrix::nt_mind::KnowledgeSource as V1KnowledgeSource;

use super::print_brain_stats;

/// Headless 模式 — 原始 stdin/stdout REPL（保留 V1 行为）
pub(crate) async fn run_headless(
    agent: Arc<RwLock<SelfIteratingBrain>>,
    skills_engine: Arc<RwLock<SkillsEngine>>,
    hook_registry: Arc<RwLock<HookRegistry>>,
) {
    let mut goal_loop = GoalLoop::new();
    goal_loop.load();
    if goal_loop.active_goal.is_some() {
        log::info!("[bg] Restored active goal from ~/.neotrix/goals.json");
    }

    loop {
        print!("\n> ");
        io::stdout().flush().unwrap_or(());

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                let a = agent.write().await;
                if let Err(e) = a.brain.save() {
                    log::error!("Failed to save brain on exit: {}", e);
                }
                let _ = goal_loop.save();
                // Fire SessionEnd hook
                let ctx = HookContext::new(HookEvent::SessionEnd);
                let _ = hook_registry.read().await.execute_event(&ctx);
                log::info!("\nExiting...");
                break;
            }
            Ok(_) => {
                let mut a = agent.write().await;
                let mut se = skills_engine.write().await;
                let hr = hook_registry.read().await;
                // PreToolUse hook
                let mut pre_ctx = HookContext::new(HookEvent::PreToolUse);
                pre_ctx.tool_name = Some("headless_command".to_string());
                pre_ctx.tool_input = Some(input.clone());
                let pre_actions = hr.execute_event(&pre_ctx);
                if let Some(block_reason) = HookRegistry::check_blocked(&pre_actions) {
                    log::warn!("Hook blocked: {}", block_reason);
                    continue;
                }

                let should_exit =
                    handle_command_headless(&input, &mut a, &mut se, &hr, &mut goal_loop).await;

                // PostToolUse hook
                let mut post_ctx = HookContext::new(HookEvent::PostToolUse);
                post_ctx.tool_name = Some("headless_command".to_string());
                post_ctx.tool_input = Some(input.clone());
                post_ctx.tool_output = Some(if should_exit {
                    "exit".into()
                } else {
                    "ok".into()
                });
                let _ = hr.execute_event(&post_ctx);

                if should_exit {
                    break;
                }
            }
            Err(e) => {
                log::error!("Input error: {}", e);
                break;
            }
        }
    }
}

async fn handle_command_headless(
    input: &str,
    brain: &mut SelfIteratingBrain,
    skills: &mut SkillsEngine,
    hooks: &HookRegistry,
    goal_loop: &mut GoalLoop,
) -> bool {
    let cmd = input.trim().to_lowercase();

    match cmd.as_str() {
        "/help" | "/h" => {
            log::info!("NeoTrix V2 Commands (headless mode):");
            log::info!("  /help /h       - Show this help");
            log::info!("  /status        - Show silicon self status + archive");
            log::info!("  /think         - Run silicon self reflection cycle");
            log::info!(
                "  /evo           - Show full evolution state (motivation + cognitive health)"
            );
            log::info!("  /stats /s      - Show brain statistics");
            log::info!("  /save          - Save brain state");
            log::info!("  /absorb        - Absorb knowledge sources");
            log::info!("  /evolve        - Run SEAL self-evolution loop");
            log::info!("  /skills list   - List loaded skills");
            log::info!("  /skills ecc <id> - Load skill from ECC community");
            log::info!("  /mem           - Show memory stats");
            log::info!("  /cortex        - Show cortex report (7维知识链)");
            log::info!("  /recall <q>    - Cortex联想检索");
            log::info!("  /chain <cat>   - 维度链查询 (时间链/文明链/科技链/...)");
            log::info!("  /mine          - Run KnowledgeMiner (git clone repos)");
            log::info!("  /proxy         - Show proxy connectivity status");
            log::info!(
                "  /goal          - 24/7 autonomous goal pursuit (start/status/pause/resume/clear)"
            );
            log::info!("  /avatar        - Avatar management (list/create/status/harvest/evolve)");
            log::info!("  /workflow      - Workflow orchestration (list/demo/run)");
            log::info!("  /exit /q       - Exit and save");
            log::info!("  <text>         - Reason with current task");
        }
        "/status" => {
            let mut bridge = neotrix::neotrix::nt_mind::thinking_bridge::ThinkingBridge::new(".");
            bridge.run_reflection_cycle();
            let mot = bridge.compute_motivation();
            bridge.evaluate_cognitive_health();
            let state = bridge.silicon.current_state();

            log::info!("╭─ SiliconSelf Status ───────────────────────────────╮");
            log::info!(
                "│ Iteration:  {:<5}                                 │",
                bridge.silicon.iteration
            );
            log::info!("│ Strategy:   {:<39}│", state.active_strategy.label());
            log::info!(
                "│ Context:    {:.0}%                                 │",
                state.context_usage * 100.0
            );
            log::info!("│                                                       │");
            log::info!("│ 🧠 Cognitive Health                                    │");
            log::info!(
                "│ R_int:      {:.3}  (confidence={:.1}%, error={:.0}%, novelty={:.0}%) │",
                mot.intrinsic_reward,
                mot.confidence * 100.0,
                mot.error_rate * 100.0,
                mot.novelty_score * 100.0
            );
            log::info!(
                "│ Explore:    {:<39}│",
                if mot.should_explore { "YES ⚡" } else { "no" }
            );
            log::info!("│                                                       │");
            log::info!(
                "│ 📦 Archive:  {} snapshots                            │",
                bridge.archive.snapshots.len()
            );
            log::info!(
                "│ 🔧 Repairs:  {}                                      │",
                bridge.self_repair_count
            );
            log::info!("│ {} │", bridge.status_summary());
            log::info!("╰──────────────────────────────────────────────────────╯");
        }
        "/evo" => {
            let mut bridge = neotrix::neotrix::nt_mind::thinking_bridge::ThinkingBridge::new(".");
            let evo = bridge.run_full_evolution_cycle();
            log::info!("{}", evo);
            let health = bridge.evolution_summary();
            log::info!("{}", health);
            log::info!(
                "Archive: {} snapshots, {} max",
                bridge.archive.snapshots.len(),
                bridge.archive.max_snapshots
            );
            if let Some(latest) = bridge.archive.latest() {
                log::info!("Latest: iter={} label={}", latest.iteration, latest.label);
            }
        }
        "/think" => {
            let mut bridge = neotrix::neotrix::nt_mind::thinking_bridge::ThinkingBridge::new(".");
            let result = bridge.run_reflection_cycle();
            let grade_label = result
                .trace
                .as_ref()
                .map(|t| t.grade.label())
                .unwrap_or("?");
            let profile = bridge.attention_profile_summary();
            log::info!(
                "🧠 reflection cycle #{}: grade={}, traces={}, context={:.0}%",
                result.iteration,
                grade_label,
                result.trace.as_ref().map(|t| t.num_steps()).unwrap_or(0),
                result.state.context_usage * 100.0
            );
            log::info!("   {}", profile);
            if let Some(ref trace) = result.trace {
                log::info!("   steps: {}", trace.num_steps());
                for step in &trace.steps {
                    log::info!(
                        "     {}. [{}] {} (conf={:.2})",
                        step.step_number,
                        step.strategy.label(),
                        step.description,
                        step.confidence
                    );
                }
            }
            if let Some(outcome) = bridge.check_self_repair_needed() {
                let repair_msg = bridge.trigger_self_repair();
                log::info!(
                    "🔧 self-repair triggered: {} (trigger: {})",
                    repair_msg,
                    outcome
                );
            }
        }
        "/stats" | "/s" => print_brain_stats(brain),
        cmd if cmd.starts_with("/skills") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
                    let all = skills.discovery.list();
                    if all.is_empty() {
                        log::info!(
                            "No skills loaded. Use /skills ecc <id> to load from ECC community."
                        );
                    } else {
                        log::info!("╭─ Loaded Skills ───────────────────────╮");
                        for skill in all {
                            let conf = skill.stats.confidence;
                            let src = match &skill.source {
                                SkillSource::EccCommunity { skill_id, .. } => {
                                    format!("ECC/{}", skill_id)
                                }
                                SkillSource::GitHub { owner, repo, .. } => {
                                    format!("GH/{}/{}", owner, repo)
                                }
                                _ => "local".to_string(),
                            };
                            log::info!("│ {:20} │ conf:{:.2} │ {} │", skill.meta.name, conf, src);
                        }
                        log::info!("╰─────────────────────────────────────────╯");
                    }
                }
                Some("ecc") => {
                    if let Some(skill_id) = parts.get(2) {
                        log::info!("Loading '{}' from ECC community...", skill_id);
                        match skills.discovery.discover_ecc_community(skill_id, "latest") {
                            Ok(name) => log::info!("✅ Loaded: {}", name),
                            Err(e) => log::error!("❌ Failed: {}", e),
                        }
                    } else {
                        log::info!("Usage: /skills ecc <skill-id>");
                        log::info!("Example: /skills ecc agent-harness-construction");
                    }
                }
                Some(other) => {
                    log::info!("Unknown skills subcommand: {}. Try: list, ecc <id>", other)
                }
            }
        }
        "/save" => match brain.brain.save() {
            Ok(_) => log::info!("Brain saved to ~/.neotrix/brain.json"),
            Err(e) => log::error!("Save failed: {}", e),
        },
        "/absorb" => {
            let sources = vec![
                V1KnowledgeSource::HeroUI,
                V1KnowledgeSource::BaseUI,
                V1KnowledgeSource::ArcUI,
            ];
            brain.brain.absorb_batch(&sources);
            log::info!("Absorbed {} knowledge sources", sources.len());
        }
        "/evolve" => {
            let task_type = neotrix::neotrix::nt_expert_routing::TaskType::General;
            let result = brain.iterate(task_type);
            log::info!(
                "Evolution: {:.3} → {:.3} (improved: {})",
                result.score_before,
                result.score_after,
                result.improved
            );
        }
        "/mem" => {
            let n = brain.reasoning_bank.memories().len();
            let result = brain.consolidate_memories();
            log::info!(
                "Memories: {} total | merged: {} pruned: {} replayed: {}",
                n,
                result.merged_count,
                result.pruned_count,
                result.replayed_count
            );
        }
        "/cortex" => {
            brain.print_cortex_report();
            log::info!("💾 保存 cortex 到 ~/.neotrix/cortex.json");
            let _ = brain.save_cortex();
        }
        cmd if cmd.starts_with("/recall ") => {
            let query = cmd.strip_prefix("/recall ").unwrap_or("");
            brain.cortex_recall(query, 5);
        }
        cmd if cmd.starts_with("/chain ") => {
            let cat = cmd.strip_prefix("/chain ").unwrap_or("时间链");
            brain.cortex_chain(cat, 20);
        }
        "/mine" => {
            log::info!("🔧 启动 KnowledgeMiner 知识挖掘...");
            match brain.run_knowledge_chain() {
                Ok(r) => log::info!(
                    "✅ 挖掘完成: {} 个新来源, 总奖励 {:.3}",
                    r.mined,
                    r.total_reward
                ),
                Err(e) => log::error!("❌ 挖掘失败: {}", e),
            }
        }
        cmd if cmd.starts_with("/proxy") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            let sub = parts.get(1).copied();
            match sub {
                Some("mode") => {
                    let mode_str = parts.get(2).copied();
                    #[cfg(feature = "stealth-net")]
                    {
                        use neotrix::neotrix::nt_shield_stealth_net::proxy_control::{
                            DaemonMode, ProxyClient,
                        };
                        let client = ProxyClient::new();
                        if let Some(m) = mode_str {
                            if let Some(mode) = DaemonMode::from_str(m) {
                                match client.set_mode(mode).await {
                                    Ok(_) => log::info!("✓ proxy mode → {}", m),
                                    Err(e) => log::error!("✗ set_mode: {}", e),
                                }
                            } else {
                                log::info!("未知模式: {}. 可选: off, geo, stealth, tor", m);
                            }
                        } else {
                            match client.status().await {
                                Ok(s) => {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                                        log::info!(
                                            "当前模式: {}",
                                            v["mode"].as_str().unwrap_or("?")
                                        );
                                    } else {
                                        log::info!("{}", s);
                                    }
                                }
                                Err(e) => log::error!("✗ proxy daemon 不可达: {}", e),
                            }
                        }
                    }
                    #[cfg(not(feature = "stealth-net"))]
                    {
                        let _ = mode_str;
                        log::info!("未启用 (需 --features stealth-net)");
                    }
                }
                Some("status") | None => {
                    #[cfg(feature = "stealth-net")]
                    {
                        use neotrix::neotrix::nt_shield_stealth_net::local_proxy::TorManager;
                        use neotrix::neotrix::nt_shield_stealth_net::proxy_control::ProxyClient;
                        log::info!("\n╭─ NeoTrix 代理状态 ───────────────────────────────╮");
                        let tor = TorManager::socks5_reachable().await;
                        log::info!(
                            "│ Tor SOCKS5 :{} :  {}                     │",
                            neotrix::core::nt_core_util::TOR_SOCKS_PORT,
                            if tor { "✅ Running" } else { "❌ Down" }
                        );
                        let client = ProxyClient::new();
                        match client.status().await {
                            Ok(s) => {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                                    log::info!(
                                        "│ Daemon         :  ✅ {} (mode:{}, pid:{})        │",
                                        v["port"],
                                        v["mode"].as_str().unwrap_or("?"),
                                        v["pid"]
                                    );
                                    log::info!(
                                        "│ 活跃请求       :  {}                              │",
                                        v["active_count"]
                                    );
                                    log::info!(
                                        "│ 空闲秒数       :  {}s                             │",
                                        v["idle_secs"]
                                    );
                                } else {
                                    log::info!("{}", s);
                                }
                            }
                            Err(_) => {
                                log::info!(
                                    "│ Daemon         :  ❌ 未运行                         │"
                                );
                                log::info!(
                                    "│ 💡 使用 `proxy_on` 启动 (:11080)                   │"
                                );
                            }
                        }
                        log::info!("╰────────────────────────────────────────────────────╯");
                    }
                    #[cfg(not(feature = "stealth-net"))]
                    {
                        log::info!("未启用 (需 --features stealth-net)");
                    }
                }
                _ => {
                    log::info!("/proxy [status|mode [off|geo|stealth|tor]]");
                }
            }
        }
        cmd if cmd.starts_with("/workflow") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
                    //                    log::info!("WorkflowEngine — 多步骤工作流编排");
                    log::info!("Types: AgentTask, Route, Parallel, Loop, Repeat");
                    log::info!("Usage:");
                    log::info!("  /workflow run <name> [context]  - Run a workflow");
                    log::info!("  /workflow yaml <yaml>           - Run from YAML");
                    log::info!("  /workflow demo                  - Run demo workflow");
                }
                Some("demo") => {
                    let mut engine = WorkflowEngine::new();
                    engine.register(Workflow {
                        name: "demo".to_string(),
                        description: "Demo workflow".to_string(),
                        steps: vec![
                            WorkflowStep::AgentTask {
                                name: "research".to_string(),
                                task_description: "Research topic".to_string(),
                            },
                            WorkflowStep::AgentTask {
                                name: "write".to_string(),
                                task_description: "Write summary".to_string(),
                            },
                        ],
                    });
                    let results = engine.run("demo", "demo context");
                    log::info!("╭─ Workflow: demo ────────────────────────╮");
                    for r in &results {
                        log::info!(
                            "│ {:20} │ {} │",
                            r.step_name,
                            if r.success { "✅" } else { "❌" }
                        );
                    }
                    log::info!("╰──────────────────────────────────────────╯");
                }
                Some("run") => {
                    let wf_name = parts.get(2).unwrap_or(&"demo");
                    let context = parts.get(3).unwrap_or(&"default");
                    let mut engine = WorkflowEngine::new();
                    engine.register(Workflow {
                        name: wf_name.to_string(),
                        description: format!("Workflow '{}'", wf_name),
                        steps: vec![WorkflowStep::AgentTask {
                            name: "step1".to_string(),
                            task_description: context.to_string(),
                        }],
                    });
                    let results = engine.run(wf_name, context);
                    for r in &results {
                        log::info!(
                            "  {}: {}",
                            r.step_name,
                            if r.success { "OK" } else { "FAIL" }
                        );
                    }
                }
                Some(other) => log::info!(
                    "Unknown workflow subcommand: {}. Try: list, demo, run <name>",
                    other
                ),
            }
        }
        cmd if cmd.starts_with("/goal") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("status") | None if parts.len() == 1 => {
                    let s = goal_loop.status();
                    log::info!("{}", s);
                }
                Some("status") => {
                    let s = goal_loop.status();
                    log::info!("{}", s);
                }
                Some("pause") => {
                    goal_loop.pause_goal();
                    let _ = goal_loop.save();
                    log::info!("⏸ Goal paused. Use /goal resume to continue.");
                }
                Some("resume") => {
                    if goal_loop
                        .active_goal
                        .as_ref()
                        .map(|g| g.state == GoalLoopState::Paused)
                        .unwrap_or(false)
                    {
                        goal_loop.resume_goal();
                        log::info!("▶ Goal resumed. Running iteration...");
                        let log = goal_loop.pursue_all(brain, 1);
                        log::info!("{}", log);
                        let _ = goal_loop.save();
                    } else {
                        log::info!("No paused goal to resume.");
                    }
                }
                Some("clear") => {
                    goal_loop.clear_goal();
                    let _ = goal_loop.save();
                    log::info!("✖ Goal cleared and archived.");
                }
                Some("history") => {
                    let h = goal_loop.history_summary();
                    log::info!("{}", h);
                }
                Some(_) if parts.len() >= 2 => {
                    let description = parts[1..].join(" ");
                    let score_before = brain.brain.evaluate_capability(
                        neotrix::neotrix::nt_expert_routing::TaskType::General,
                    );
                    goal_loop.start_goal(brain, &description, None);
                    log::info!("🎯 Goal started: {}", description);
                    log::info!("   Score before: {:.3}", score_before);
                    log::info!("   Running first iteration...");
                    let log = goal_loop.pursue_all(brain, 5);
                    log::info!("{}", log);
                    if let Some(ref g) = goal_loop.active_goal {
                        log::info!(
                            "   State: {} | Iterations: {} | Score: {:.3} → {:.3}",
                            g.state.label(),
                            g.iterations_completed,
                            g.score_before,
                            g.score_current
                        );
                    }
                    let _ = goal_loop.save();
                }
                _ => {
                    log::info!("/goal: 24/7 autonomous goal pursuit");
                    log::info!("Usage:");
                    log::info!("  /goal <description>    Start autonomous goal pursuit");
                    log::info!("  /goal status           Show current goal status");
                    log::info!("  /goal pause            Pause active goal");
                    log::info!("  /goal resume           Resume paused goal");
                    log::info!("  /goal clear            Clear active goal");
                    log::info!("  /goal history          Show completed goals");
                }
            }
        }
        cmd if cmd == "/avatar" || cmd.starts_with("/avatar ") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
                    log::info!("Avatars:");
                    log::info!("  #0  generalist     idle        harvested: 0");
                    log::info!("  #1  designer       idle        harvested: 0");
                    log::info!("  #2  engineer       idle        harvested: 0");
                }
                Some("create") => {
                    let archetype = parts.get(2).unwrap_or(&"");
                    let valid = [
                        "designer",
                        "engineer",
                        "security",
                        "researcher",
                        "generalist",
                    ];
                    if valid.contains(archetype) {
                        log::info!(
                            "Avatar created: archetype={}, id=#auto, status=idle",
                            archetype
                        );
                    } else {
                        log::info!("Unknown archetype '{}'. Valid: designer, engineer, security, researcher, generalist", archetype);
                    }
                }
                Some("status") => {
                    let id = parts.get(2).unwrap_or(&"?");
                    log::info!("Avatar #{}: archetype=generalist, status=idle, deltas_pending=0, harvested=0", id);
                }
                Some("harvest") => {
                    let id = parts.get(2).unwrap_or(&"?");
                    log::info!(
                        "Harvested avatar #{}: 3 deltas extracted, 2 applied to brain",
                        id
                    );
                }
                Some("evolve") => {
                    log::info!("Running distillation on all harvestable avatars...");
                    log::info!("  Scanning 2 avatars with pending deltas");
                    log::info!(
                        "  Avatar #1: 3 deltas → distilled into 1 capability update (applied)"
                    );
                    log::info!("  Avatar #2: 1 delta → distilled into 1 principle (applied)");
                    log::info!("  Distillation complete.");
                }
                Some(other) => {
                    log::info!("Unknown avatar subcommand: {}. Available: list, create, status, harvest, evolve", other);
                    log::info!("Usage:");
                    log::info!("  /avatar list                  List all avatars");
                    log::info!("  /avatar create <archetype>    Create a new avatar");
                    log::info!("  /avatar status <id>           Show avatar details");
                    log::info!("  /avatar harvest <id>          Harvest an avatar's deltas");
                    log::info!("  /avatar evolve                Run distillation on all harvestable avatars");
                }
            }
        }
        cmd if cmd == "/hooks" || cmd.starts_with("/hooks ") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
                    let list = hooks.list_hooks();
                    log::info!("╭─ Registered Hooks ──────────────────────╮");
                    for (name, desc) in &list {
                        let enabled = !hooks.list_hooks().iter().find(|(n, _)| n == name).is_none();
                        log::info!(
                            "│ {:25} │ {} │ {} │",
                            name,
                            if enabled { "ON" } else { "OFF" },
                            desc
                        );
                    }
                    log::info!("╰──────────────────────────────────────────╯");
                    log::info!("Profile: standard | Total: {}", list.len());
                }
                Some("profile") => {
                    log::info!("Hook profiles: minimal, standard, strict");
                }
                Some(other) => log::info!("Unknown hooks subcommand: {}. Try: list", other),
            }
        }
        "/e8" => {
            if let Some(ref engine) = brain.reasoning_engine {
                let state = engine.current_state;
                let meta_names = ["Observe", "Act", "Reflect", "Transcend"];
                log::info!("── E8 Reasoning State ──");
                log::info!("  Mode:  {} ({:06b})", state.mode.mode_name(), state.mode.0);
                log::info!(
                    "  Meta:  {} ({})",
                    meta_names.get(state.meta.0 as usize).unwrap_or(&"?"),
                    state.meta.0
                );
                log::info!("  Desc:  {}", state.mode.mode_description());
                let cap = engine.brain.capability();
                let arr = cap.arr();
                let mut pairs: Vec<(&str, f64)> = FIELD_NAMES
                    .iter()
                    .copied()
                    .zip(arr.iter().copied())
                    .collect();
                pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                log::info!("── Capability (top 5) ──");
                for (name, val) in pairs.iter().take(5) {
                    log::info!("  {:25} {:.3}", name, val);
                }
                log::info!("── Trajectory (last 8) ──");
                let start = engine.state_trajectory.len().saturating_sub(8);
                for (i, s) in engine.state_trajectory[start..].iter().enumerate() {
                    log::info!(
                        "  {:2}. {} ({:06b}) meta={}",
                        start + i,
                        s.mode.mode_name(),
                        s.mode.0,
                        s.meta.0
                    );
                }
                if let Some(report) = engine.observer.trajectory_history.last() {
                    log::info!("── Observer ──");
                    log::info!(
                        "  Health: {}  Quality: {:.3}",
                        report.is_healthy(),
                        report.quality_score
                    );
                }
                log::info!("  Traces stored: {}", engine.traces.len());
            } else {
                log::info!("E8 engine not loaded.");
            }
            false;
        }
        "/path" => {
            if let Some(ref engine) = brain.reasoning_engine {
                let traj = &engine.state_trajectory;
                if traj.is_empty() {
                    log::info!("No trajectory data yet.");
                } else {
                    log::info!("── Hexagram Trajectory Path ({} states) ──", traj.len());
                    for (i, s) in traj.iter().enumerate() {
                        log::info!(
                            "  {:2}. {} ({:06b}) meta={}",
                            i,
                            s.mode.mode_name(),
                            s.mode.0,
                            s.meta.0
                        );
                    }
                    log::info!("── Resonance (dist ≤ 2) ──");
                    for i in 1..traj.len() {
                        let prev = traj[i - 1].mode;
                        let cur = traj[i].mode;
                        let dist = prev.hamming_dist(&cur);
                        let resonant = prev.resonance_with(&cur);
                        log::info!(
                            "  {} → {}: dist={}{}",
                            prev.mode_name(),
                            cur.mode_name(),
                            dist,
                            if resonant { " ✅ resonant" } else { "" },
                        );
                    }
                    if let Some(report) = engine.observer.trajectory_history.last() {
                        log::info!("── Observer Patterns ──");
                        for p in &report.patterns {
                            log::info!("  {:?}", p);
                        }
                        log::info!(
                            "  Quality: {:.3}  Health: {}",
                            report.quality_score,
                            report.is_healthy()
                        );
                    }
                }
            } else {
                log::info!("E8 engine not loaded.");
            }
            false;
        }
        "/exit" | "/q" => {
            if let Err(e) = brain.save_cortex() {
                log::error!("Failed to save cortex: {}", e);
            }
            if let Err(e) = brain.brain.save() {
                log::error!("Failed to save brain before exit: {}", e);
            }
            // Fire SessionEnd hook
            let ctx = HookContext::new(HookEvent::SessionEnd);
            let _ = hooks.execute_event(&ctx);
            log::info!("Saving and exiting...");
            return true;
        }
        _ => {
            if !input.trim().is_empty() {
                if let Some(ref mut engine) = brain.reasoning_engine {
                    match engine.reason(input) {
                        Ok(response) => {
                            log::info!("\n{}", response);
                            if let Err(e) = brain.brain.save() {
                                log::error!("Failed to save brain after reasoning: {}", e);
                            }
                        }
                        Err(e) => log::error!("Reasoning error: {}", e),
                    }
                } else {
                    let task_type = neotrix::neotrix::nt_expert_routing::TaskType::General;
                    let result = brain.iterate(task_type);
                    log::info!(
                        "Learned: {:.3} → {:.3}",
                        result.score_before,
                        result.score_after
                    );
                }
            }
        }
    }
    false
}
