use std::sync::{Arc, Mutex};

use futures::FutureExt;
use tokio::sync::RwLock;

use neotrix::core::nt_core_util::{A2A_DEFAULT_PORT, A2A_INTERNAL_PORT};
use neotrix::neotrix::nt_expert_routing::WorldModelV2;
use neotrix::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::neotrix::nt_mind_background_loop::{BackgroundLoop, ConsciousnessIntegration};

use neotrix::agent::hooks::{HookContext, HookEvent, HookRegistry};
use neotrix::agent::skills::SkillsEngine;
use neotrix::agent::{AgentRole, AgentTeam, ProcessType};

use crate::config::NeoTrixConfig;
// TODO(SelfIsNotFile): In Wave A, replace RulesInjection::load() with
// RulesInjection::from_self_model() once the interactive entry point has
// access to SelfModelGenerator (e.g. via ConsciousnessIntegration after
// BackgroundLoop starts). The self-model text comes from
// SelfModelGenerator::last_model() — no file read required.
use crate::rules::RulesInjection;

use super::{info, init_brain, print_brain_stats, success, warn};

pub fn run_interactive(cfg: &NeoTrixConfig, profile: &str) {
    run_interactive_with_ephemeral(cfg, profile, false)
}

pub fn run_interactive_with_ephemeral(cfg: &NeoTrixConfig, profile: &str, ephemeral: bool) {
    if let Some(level) = &cfg.log_level {
        std::env::set_var("RUST_LOG", format!("neotrix={}", level));
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);

        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.load_cortex();
        agent.init_reasoning_engine();
        agent.quality_threshold = 0.7;
        agent.auto_absorb = true;
        agent.auto_memory_iteration = true;
        agent.memory_iteration_interval = 5;

        // ── CLAUDE.md 4-level hierarchy ──
        #[allow(deprecated)]
        let rules_injection = RulesInjection::load();
        let has_rules = !rules_injection.merged.is_empty();
        if has_rules {
            log::info!(
                "{}: {} {}",
                info("RulesInjection"),
                success(format!("{} layers loaded", rules_injection.layers.len())),
                info(format!("({} chars)", rules_injection.merged.len()))
            );
        }

        let has_engine = agent.reasoning_engine.is_some();
        if has_engine {
            log::info!(
                "{}: {} {}",
                info("ReasoningEngine"),
                success("active"),
                info("(LLM connected)")
            );
        } else {
            log::info!(
                "{}: {}",
                warn("ReasoningEngine"),
                warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)")
            );
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        let skill_count = skills_engine.init().len();
        log::info!(
            "{}: {} {}",
            info("SkillsEngine"),
            success(format!("{} local skills loaded", skill_count)),
            ""
        );
        log::info!(
            "  -> {} /skills list to browse, /skills ecc <id> to load from ECC community",
            info("/skills")
        );

        let mut hook_registry = HookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        log::info!(
            "{}: {} {}",
            info("HookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)")
        );

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = HookRegistry::check_blocked(&hook_actions) {
            log::warn!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let _skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry: Arc<RwLock<HookRegistry>> = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg_goal_loop.load();
        if bg_goal_loop.active_goal.is_some() {
            log::info!(
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
            team.add_agent(AgentRole {
                name: "planner".into(),
                role: "Task Planner".into(),
                goal: "Break down complex tasks into sub-tasks".into(),
                backstory: "Strategic planner with systems thinking".into(),
                tools: vec!["reason".into()],
            });
        }
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);

        let bg_handle = tokio::spawn(async move {
            if let Err(e) = std::panic::AssertUnwindSafe(async {
                let mut bg = BackgroundLoop::new(bg_agent)
                    .with_consciousness(ConsciousnessIntegration::new())
                    .with_goal_loop(bg_goal_loop)
                    .with_nt_world_model(WorldModelV2::new(8, 64))
                    .with_panorama(PanoramaPipeline::new())
                    .with_nt_world_crawl(std::path::PathBuf::from("."))
                    .with_exploration_pipeline(std::path::PathBuf::from("."))
                    .with_knowledge_chain(std::path::PathBuf::from("."))
                    .with_agent_discovery(A2A_INTERNAL_PORT)
                    .with_a2a_server_default(A2A_DEFAULT_PORT)
                    .with_adaptive_controller_default();
                #[cfg(feature = "stealth-net")]
                {
                    bg = bg.with_world_consciousness();
                }
                bg.start().await;
            })
            .catch_unwind()
            .await
            {
                log::error!("[interactive] background loop panicked: {:?}", e);
            }
        });

        // PreToolUse hook — entering interactive TUI session
        {
            let hr = hook_registry.read().await;
            let mut pre_ctx = HookContext::new(HookEvent::PreToolUse);
            pre_ctx.tool_name = Some("tui_session".to_string());
            pre_ctx.tool_input = Some("interactive_mode".to_string());
            let pre_actions = hr.execute_event(&pre_ctx);
            if let Some(block_reason) = HookRegistry::check_blocked(&pre_actions) {
                log::warn!("Hook blocked TUI session: {}", block_reason);
            }
        }

        super::desktop::run_tui(agent, ephemeral).await;

        // Cancel background loop on TUI exit so the axum/A2A server doesn't
        // outlive the interactive session.
        bg_handle.abort();

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
