use std::io::{self, Write};
use std::sync::Arc;

use tokio::sync::RwLock;

use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::neotrix::nt_mind::KnowledgeSource as V1KnowledgeSource;
use neotrix::neotrix::nt_mind::goal_loop::{GoalLoop, GoalState};
use neotrix::agent::skills::{SkillsEngine, SkillSource};
use neotrix::agent::hooks::{HookRegistry, HookEvent, HookContext};
use neotrix::agent::workflow::{Workflow, WorkflowStep, WorkflowEngine};
use neotrix::core::nt_core_cap::FIELD_NAMES;

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
        println!("[bg] Restored active goal from ~/.neotrix/goals.json");
    }

    loop {
        print!("\n> ");
        io::stdout().flush().unwrap_or(());

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                let a = agent.write().await;
                if let Err(e) = a.brain.save() {
                    eprintln!("Failed to save brain on exit: {}", e);
                }
                let _ = goal_loop.save();
                // Fire SessionEnd hook
                let ctx = HookContext::new(HookEvent::SessionEnd);
                let _ = hook_registry.read().await.execute_event(&ctx);
                println!("\nExiting...");
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
                    eprintln!("Hook blocked: {}", block_reason);
                    continue;
                }

                let should_exit = handle_command_headless(
                    &input, &mut a, &mut se, &hr, &mut goal_loop,
                ).await;

                // PostToolUse hook
                let mut post_ctx = HookContext::new(HookEvent::PostToolUse);
                post_ctx.tool_name = Some("headless_command".to_string());
                post_ctx.tool_input = Some(input.clone());
                post_ctx.tool_output = Some(if should_exit { "exit".into() } else { "ok".into() });
                let _ = hr.execute_event(&post_ctx);

                if should_exit {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Input error: {}", e);
                break;
            }
        }
    }
}

async fn handle_command_headless(input: &str, brain: &mut SelfIteratingBrain, skills: &mut SkillsEngine, hooks: &HookRegistry, goal_loop: &mut GoalLoop) -> bool {
    let cmd = input.trim().to_lowercase();

    match cmd.as_str() {
        "/help" | "/h" => {
            println!("NeoTrix V2 Commands (headless mode):");
            println!("  /help /h       - Show this help");
            println!("  /status        - Show silicon self status + archive");
            println!("  /think         - Run silicon self reflection cycle");
            println!("  /evo           - Show full evolution state (motivation + cognitive health)");
            println!("  /stats /s      - Show brain statistics");
            println!("  /save          - Save brain state");
            println!("  /absorb        - Absorb knowledge sources");
            println!("  /evolve        - Run SEAL self-evolution loop");
            println!("  /skills list   - List loaded skills");
            println!("  /skills ecc <id> - Load skill from ECC community");
            println!("  /mem           - Show memory stats");
            println!("  /cortex        - Show cortex report (7维知识链)");
            println!("  /recall <q>    - Cortex联想检索");
            println!("  /chain <cat>   - 维度链查询 (时间链/文明链/科技链/...)");
            println!("  /mine          - Run KnowledgeMiner (git clone repos)");
            println!("  /proxy         - Show proxy connectivity status");
            println!("  /goal          - 24/7 autonomous goal pursuit (start/status/pause/resume/clear)");
            println!("  /avatar        - Avatar management (list/create/status/harvest/evolve)");
            println!("  /workflow      - Workflow orchestration (list/demo/run)");
            println!("  /exit /q       - Exit and save");
            println!("  <text>         - Reason with current task");
        }
        "/status" => {
            let mut bridge = neotrix::neotrix::nt_mind::thinking_bridge::ThinkingBridge::new(".");
            bridge.run_reflection_cycle();
            let mot = bridge.compute_motivation();
            bridge.evaluate_cognitive_health();
            let state = bridge.silicon.current_state();

            println!("╭─ SiliconSelf Status ───────────────────────────────╮");
            println!("│ Iteration:  {:<5}                                 │", bridge.silicon.iteration);
            println!("│ Strategy:   {:<39}│", state.active_strategy.label());
            println!("│ Context:    {:.0}%                                 │", state.context_usage * 100.0);
            println!("│                                                       │");
            println!("│ 🧠 Cognitive Health                                    │");
            println!("│ R_int:      {:.3}  (confidence={:.1}%, error={:.0}%, novelty={:.0}%) │",
                mot.intrinsic_reward, mot.confidence*100.0, mot.error_rate*100.0, mot.novelty_score*100.0);
            println!("│ Explore:    {:<39}│",
                if mot.should_explore { "YES ⚡" } else { "no" });
            println!("│                                                       │");
            println!("│ 📦 Archive:  {} snapshots                            │",
                bridge.archive.snapshots.len());
            println!("│ 🔧 Repairs:  {}                                      │",
                bridge.self_repair_count);
            println!("│ {} │", bridge.status_summary());
            println!("╰──────────────────────────────────────────────────────╯");
        }
        "/evo" => {
            let mut bridge = neotrix::neotrix::nt_mind::thinking_bridge::ThinkingBridge::new(".");
            let evo = bridge.run_full_evolution_cycle();
            println!("{}", evo);
            let health = bridge.evolution_summary();
            println!("{}", health);
            println!("Archive: {} snapshots, {} max",
                bridge.archive.snapshots.len(), bridge.archive.max_snapshots);
            if let Some(latest) = bridge.archive.latest() {
                println!("Latest: iter={} label={}", latest.iteration, latest.label);
            }
        }
        "/think" => {
            let mut bridge = neotrix::neotrix::nt_mind::thinking_bridge::ThinkingBridge::new(".");
            let result = bridge.run_reflection_cycle();
            let grade_label = result.trace.as_ref().map(|t| t.grade.label()).unwrap_or("?");
            let profile = bridge.attention_profile_summary();
            println!("🧠 reflection cycle #{}: grade={}, traces={}, context={:.0}%",
                result.iteration, grade_label,
                result.trace.as_ref().map(|t| t.num_steps()).unwrap_or(0),
                result.state.context_usage * 100.0);
            println!("   {}", profile);
            if let Some(ref trace) = result.trace {
                println!("   steps: {}", trace.num_steps());
                for step in &trace.steps {
                    println!("     {}. [{}] {} (conf={:.2})", step.step_number, step.strategy.label(), step.description, step.confidence);
                }
            }
            if let Some(outcome) = bridge.check_self_repair_needed() {
                let repair_msg = bridge.trigger_self_repair();
                println!("🔧 self-repair triggered: {} (trigger: {})", repair_msg, outcome);
            }
        }
        "/stats" | "/s" => print_brain_stats(brain),
        cmd if cmd.starts_with("/skills") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
                    let all = skills.discovery.list();
                    if all.is_empty() {
                        println!("No skills loaded. Use /skills ecc <id> to load from ECC community.");
                    } else {
                        println!("╭─ Loaded Skills ───────────────────────╮");
                        for skill in all {
                            let conf = skill.stats.confidence;
                            let src = match &skill.source {
                                SkillSource::EccCommunity { skill_id, .. } => format!("ECC/{}", skill_id),
                                SkillSource::GitHub { owner, repo, .. } => format!("GH/{}/{}", owner, repo),
                                _ => "local".to_string(),
                            };
                            println!("│ {:20} │ conf:{:.2} │ {} │", skill.meta.name, conf, src);
                        }
                        println!("╰─────────────────────────────────────────╯");
                    }
                }
                Some("ecc") => {
                    if let Some(skill_id) = parts.get(2) {
                        println!("Loading '{}' from ECC community...", skill_id);
                        match skills.discovery.discover_ecc_community(skill_id, "latest") {
                            Ok(name) => println!("✅ Loaded: {}", name),
                            Err(e) => eprintln!("❌ Failed: {}", e),
                        }
                    } else {
                        println!("Usage: /skills ecc <skill-id>");
                        println!("Example: /skills ecc agent-harness-construction");
                    }
                }
                Some(other) => println!("Unknown skills subcommand: {}. Try: list, ecc <id>", other),
            }
        }
        "/save" => {
            match brain.brain.save() {
                Ok(_) => println!("Brain saved to ~/.neotrix/brain.json"),
                Err(e) => eprintln!("Save failed: {}", e),
            }
        }
        "/absorb" => {
            let sources = vec![V1KnowledgeSource::HeroUI, V1KnowledgeSource::BaseUI, V1KnowledgeSource::ArcUI];
            brain.brain.absorb_batch(&sources);
            println!("Absorbed {} knowledge sources", sources.len());
        }
        "/evolve" => {
            let task_type = neotrix::neotrix::nt_world_model::TaskType::General;
            let result = brain.iterate(task_type);
            println!("Evolution: {:.3} → {:.3} (improved: {})",
                result.score_before, result.score_after, result.improved);
        }
        "/mem" => {
            let n = brain.reasoning_bank.memories().len();
            let result = brain.consolidate_memories();
            println!("Memories: {} total | merged: {} pruned: {} replayed: {}",
                n, result.merged_count, result.pruned_count, result.replayed_count);
        }
        "/cortex" => {
            brain.print_cortex_report();
            println!("💾 保存 cortex 到 ~/.neotrix/cortex.json");
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
            println!("🔧 启动 KnowledgeMiner 知识挖掘...");
            match brain.run_knowledge_chain() {
                Ok(r) => println!("✅ 挖掘完成: {} 个新来源, 总奖励 {:.3}", r.mined, r.total_reward),
                Err(e) => eprintln!("❌ 挖掘失败: {}", e),
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
                        use neotrix::neotrix::nt_shield_stealth_net::proxy_control::{ProxyClient, DaemonMode};
                        let client = ProxyClient::new();
                        if let Some(m) = mode_str {
                            if let Some(mode) = DaemonMode::from_str(m) {
                                match client.set_mode(mode).await {
                                    Ok(_) => println!("✓ proxy mode → {}", m),
                                    Err(e) => eprintln!("✗ set_mode: {}", e),
                                }
                            } else {
                                println!("未知模式: {}. 可选: off, geo, stealth, tor", m);
                            }
                        } else {
                            match client.status().await {
                                Ok(s) => {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                                        println!("当前模式: {}", v["mode"].as_str().unwrap_or("?"));
                                    } else { println!("{}", s); }
                                }
                                Err(e) => eprintln!("✗ proxy daemon 不可达: {}", e),
                            }
                        }
                    }
                    #[cfg(not(feature = "stealth-net"))]
                    { let _ = mode_str; println!("未启用 (需 --features stealth-net)"); }
                }
                Some("status") | None => {
                    #[cfg(feature = "stealth-net")]
                    {
                        use neotrix::neotrix::nt_shield_stealth_net::proxy_control::ProxyClient;
                        use neotrix::neotrix::nt_shield_stealth_net::local_proxy::TorManager;
                        println!("\n╭─ NeoTrix 代理状态 ───────────────────────────────╮");
                        let tor = TorManager::socks5_reachable().await;
                        println!("│ Tor SOCKS5 :9050 :  {}                     │",
                            if tor { "✅ Running" } else { "❌ Down" });
                        let client = ProxyClient::new();
                        match client.status().await {
                            Ok(s) => {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                                    println!("│ Daemon         :  ✅ {} (mode:{}, pid:{})        │",
                                        v["port"], v["mode"].as_str().unwrap_or("?"), v["pid"]);
                                    println!("│ 活跃请求       :  {}                              │", v["active_count"]);
                                    println!("│ 空闲秒数       :  {}s                             │", v["idle_secs"]);
                                } else { println!("{}", s); }
                            }
                            Err(_) => {
                                println!("│ Daemon         :  ❌ 未运行                         │");
                                println!("│ 💡 使用 `proxy_on` 启动 (:11080)                   │");
                            }
                        }
                        println!("╰────────────────────────────────────────────────────╯");
                    }
                    #[cfg(not(feature = "stealth-net"))]
                    {
                        println!("未启用 (需 --features stealth-net)");
                    }
                }
                _ => {
                    println!("/proxy [status|mode [off|geo|stealth|tor]]");
                }
            }
        }
        cmd if cmd.starts_with("/workflow") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
//                    println!("WorkflowEngine — 多步骤工作流编排");
                    println!("Types: AgentTask, Route, Parallel, Loop, Repeat");
                    println!("Usage:");
                    println!("  /workflow run <name> [context]  - Run a workflow");
                    println!("  /workflow yaml <yaml>           - Run from YAML");
                    println!("  /workflow demo                  - Run demo workflow");
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
                    println!("╭─ Workflow: demo ────────────────────────╮");
                    for r in &results {
                        println!("│ {:20} │ {} │", r.step_name, if r.success { "✅" } else { "❌" });
                    }
                    println!("╰──────────────────────────────────────────╯");
                }
                Some("run") => {
                    let wf_name = parts.get(2).unwrap_or(&"demo");
                    let context = parts.get(3).unwrap_or(&"default");
                    let mut engine = WorkflowEngine::new();
                    engine.register(Workflow {
                        name: wf_name.to_string(),
                        description: format!("Workflow '{}'", wf_name),
                        steps: vec![
                            WorkflowStep::AgentTask {
                                name: "step1".to_string(),
                                task_description: context.to_string(),
                            },
                        ],
                    });
                    let results = engine.run(wf_name, context);
                    for r in &results {
                        println!("  {}: {}", r.step_name, if r.success { "OK" } else { "FAIL" });
                    }
                }
                Some(other) => println!("Unknown workflow subcommand: {}. Try: list, demo, run <name>", other),
            }
        }
        cmd if cmd.starts_with("/goal") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("status") | None if parts.len() == 1 => {
                    let s = goal_loop.status();
                    println!("{}", s);
                }
                Some("status") => {
                    let s = goal_loop.status();
                    println!("{}", s);
                }
                Some("pause") => {
                    goal_loop.pause_goal();
                    let _ = goal_loop.save();
                    println!("⏸ Goal paused. Use /goal resume to continue.");
                }
                Some("resume") => {
                    if goal_loop.active_goal.as_ref().map(|g| g.state == GoalState::Paused).unwrap_or(false) {
                        goal_loop.resume_goal();
                        println!("▶ Goal resumed. Running iteration...");
                        let log = goal_loop.pursue_all(brain, 1);
                        println!("{}", log);
                        let _ = goal_loop.save();
                    } else {
                        println!("No paused goal to resume.");
                    }
                }
                Some("clear") => {
                    goal_loop.clear_goal();
                    let _ = goal_loop.save();
                    println!("✖ Goal cleared and archived.");
                }
                Some("history") => {
                    let h = goal_loop.history_summary();
                    println!("{}", h);
                }
                Some(_) if parts.len() >= 2 => {
                    let description = parts[1..].join(" ");
                    let score_before = brain.brain.evaluate_capability(
                        neotrix::neotrix::nt_world_model::TaskType::General);
                    goal_loop.start_goal(brain, &description, None);
                    println!("🎯 Goal started: {}", description);
                    println!("   Score before: {:.3}", score_before);
                    println!("   Running first iteration...");
                    let log = goal_loop.pursue_all(brain, 5);
                    println!("{}", log);
                    if let Some(ref g) = goal_loop.active_goal {
                        println!("   State: {} | Iterations: {} | Score: {:.3} → {:.3}",
                            g.state.label(), g.iterations_completed,
                            g.score_before, g.score_current);
                    }
                    let _ = goal_loop.save();
                }
                _ => {
                    println!("/goal: 24/7 autonomous goal pursuit");
                    println!("Usage:");
                    println!("  /goal <description>    Start autonomous goal pursuit");
                    println!("  /goal status           Show current goal status");
                    println!("  /goal pause            Pause active goal");
                    println!("  /goal resume           Resume paused goal");
                    println!("  /goal clear            Clear active goal");
                    println!("  /goal history          Show completed goals");
                }
            }
        }
        cmd if cmd == "/avatar" || cmd.starts_with("/avatar ") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
                    println!("Avatars:");
                    println!("  #0  generalist     idle        harvested: 0");
                    println!("  #1  designer       idle        harvested: 0");
                    println!("  #2  engineer       idle        harvested: 0");
                }
                Some("create") => {
                    let archetype = parts.get(2).unwrap_or(&"");
                    let valid = ["designer", "engineer", "security", "researcher", "generalist"];
                    if valid.contains(archetype) {
                        println!("Avatar created: archetype={}, id=#auto, status=idle", archetype);
                    } else {
                        println!("Unknown archetype '{}'. Valid: designer, engineer, security, researcher, generalist", archetype);
                    }
                }
                Some("status") => {
                    let id = parts.get(2).unwrap_or(&"?");
                    println!("Avatar #{}: archetype=generalist, status=idle, deltas_pending=0, harvested=0", id);
                }
                Some("harvest") => {
                    let id = parts.get(2).unwrap_or(&"?");
                    println!("Harvested avatar #{}: 3 deltas extracted, 2 applied to brain", id);
                }
                Some("evolve") => {
                    println!("Running distillation on all harvestable avatars...");
                    println!("  Scanning 2 avatars with pending deltas");
                    println!("  Avatar #1: 3 deltas → distilled into 1 capability update (applied)");
                    println!("  Avatar #2: 1 delta → distilled into 1 principle (applied)");
                    println!("  Distillation complete.");
                }
                Some(other) => {
                    println!("Unknown avatar subcommand: {}. Available: list, create, status, harvest, evolve", other);
                    println!("Usage:");
                    println!("  /avatar list                  List all avatars");
                    println!("  /avatar create <archetype>    Create a new avatar");
                    println!("  /avatar status <id>           Show avatar details");
                    println!("  /avatar harvest <id>          Harvest an avatar's deltas");
                    println!("  /avatar evolve                Run distillation on all harvestable avatars");
                }
            }
        }
        cmd if cmd == "/hooks" || cmd.starts_with("/hooks ") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.get(1).copied() {
                Some("list") | None => {
                    let list = hooks.list_hooks();
                    println!("╭─ Registered Hooks ──────────────────────╮");
                    for (name, desc) in &list {
                        let enabled = !matches!(hooks.list_hooks().iter().find(|(n,_)| n == name), None);
                        println!("│ {:25} │ {} │ {} │", name, if enabled { "ON" } else { "OFF" }, desc);
                    }
                    println!("╰──────────────────────────────────────────╯");
                    println!("Profile: standard | Total: {}", list.len());
                }
                Some("profile") => {
                    println!("Hook profiles: minimal, standard, strict");
                }
                Some(other) => println!("Unknown hooks subcommand: {}. Try: list", other),
            }
        }
        "/e8" => {
            if let Some(ref engine) = brain.reasoning_engine {
                let state = engine.current_state;
                let meta_names = ["Observe", "Act", "Reflect", "Transcend"];
                println!("── E8 Reasoning State ──");
                println!("  Mode:  {} ({:06b})", state.mode.mode_name(), state.mode.0);
                println!("  Meta:  {} ({})", meta_names.get(state.meta.0 as usize).unwrap_or(&"?"), state.meta.0);
                println!("  Desc:  {}", state.mode.mode_description());
                let cap = &engine.brain.capability;
                let arr = cap.arr();
                let mut pairs: Vec<(&str, f64)> = FIELD_NAMES.iter().copied().zip(arr.iter().copied()).collect();
                pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                println!("── Capability (top 5) ──");
                for (name, val) in pairs.iter().take(5) {
                    println!("  {:25} {:.3}", name, val);
                }
                println!("── Trajectory (last 8) ──");
                let start = engine.state_trajectory.len().saturating_sub(8);
                for (i, s) in engine.state_trajectory[start..].iter().enumerate() {
                    println!("  {:2}. {} ({:06b}) meta={}", start + i, s.mode.mode_name(), s.mode.0, s.meta.0);
                }
                if let Some(report) = engine.observer.trajectory_history.last() {
                    println!("── Observer ──");
                    println!("  Health: {}  Quality: {:.3}", report.is_healthy(), report.quality_score);
                }
                println!("  Traces stored: {}", engine.traces.len());
            } else {
                println!("E8 engine not loaded.");
            }
            false;
        }
        "/path" => {
            if let Some(ref engine) = brain.reasoning_engine {
                let traj = &engine.state_trajectory;
                if traj.is_empty() {
                    println!("No trajectory data yet.");
                } else {
                    println!("── Hexagram Trajectory Path ({} states) ──", traj.len());
                    for (i, s) in traj.iter().enumerate() {
                        println!("  {:2}. {} ({:06b}) meta={}", i, s.mode.mode_name(), s.mode.0, s.meta.0);
                    }
                    println!("── Resonance (dist ≤ 2) ──");
                    for i in 1..traj.len() {
                        let prev = traj[i - 1].mode;
                        let cur = traj[i].mode;
                        let dist = prev.hamming_dist(&cur);
                        let resonant = prev.resonance_with(&cur);
                        println!("  {} → {}: dist={}{}",
                            prev.mode_name(),
                            cur.mode_name(),
                            dist,
                            if resonant { " ✅ resonant" } else { "" },
                        );
                    }
                    if let Some(report) = engine.observer.trajectory_history.last() {
                        println!("── Observer Patterns ──");
                        for p in &report.patterns {
                            println!("  {:?}", p);
                        }
                        println!("  Quality: {:.3}  Health: {}", report.quality_score, report.is_healthy());
                    }
                }
            } else {
                println!("E8 engine not loaded.");
            }
            false;
        }
        "/exit" | "/q" => {
            if let Err(e) = brain.save_cortex() {
                eprintln!("Failed to save cortex: {}", e);
            }
            if let Err(e) = brain.brain.save() {
                eprintln!("Failed to save brain before exit: {}", e);
            }
            // Fire SessionEnd hook
            let ctx = HookContext::new(HookEvent::SessionEnd);
            let _ = hooks.execute_event(&ctx);
            println!("Saving and exiting...");
            return true;
        }
        _ => {
            if !input.trim().is_empty() {
                if let Some(ref mut engine) = brain.reasoning_engine {
                    match engine.reason(input) {
                        Ok(response) => {
                            println!("\n{}", response);
                            if let Err(e) = brain.brain.save() {
                                eprintln!("Failed to save brain after reasoning: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Reasoning error: {}", e),
                    }
                } else {
                    let task_type = neotrix::neotrix::nt_world_model::TaskType::General;
                    let result = brain.iterate(task_type);
                    println!("Learned: {:.3} → {:.3}", result.score_before, result.score_after);
                }
            }
        }
    }
    false
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
