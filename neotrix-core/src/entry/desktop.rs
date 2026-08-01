use std::sync::Arc;

use tokio::sync::RwLock;

use neotrix::neotrix::l1_body_impl::nt_io_neocodex::{NeoCodexUI, NeoCodexMode, NeoCodexAgent};
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

/// Run a single evolution-loop iteration against the agent (diagnose → fix).
/// Kept behind a free function so the loop can be triggered from the TUI
/// without holding the mutex across the whole command.
pub(crate) async fn step_evolution(agent: &mut NeoCodexAgent) {
    neotrix::neotrix::l1_body_impl::nt_io_neocodex::EvolutionLoop::step(agent);
}


pub(crate) async fn run_tui(_agent: Arc<RwLock<SelfIteratingBrain>>, _ephemeral: bool) {
    let mut agent_ui = NeoCodexUI::new("neotrix-session");
    agent_ui.agent.lock().await.set_brain(_agent.clone());

    // Cycle 159b fix: ensure a real (non-stub) provider is selected at startup
    // so the ReAct loop is production-reachable instead of always falling back
    // to the deterministic stub. Honors NEOTRIX_PROVIDER env.
    {
        let mut agent = agent_ui.agent.lock().await;
        agent.provider.ensure_production_provider();
        if agent.provider.is_resolvable() {
            eprintln!("[provider] active: {} ({})", agent.provider.active_model(), agent.config.provider_name);
        } else {
            eprintln!("[provider] warning: no resolvable provider (set NEOTRIX_PROVIDER / API keys) — falling back to stub");
        }
    }

    // Cycle 159: session continuity — resume prior turns from the wire file (G2)
    {
        let mut agent = agent_ui.agent.lock().await;
        let resumed = agent.resume_session();
        if resumed > 0 {
            eprintln!("[resume] restored {} prior events from {}", resumed, agent.wire.path.display());
        }
    }

    loop {
        let input = {
            let agent = agent_ui.agent.lock().await;
            let report = agent.health_report();
            let status = format!(
                "[{}] Turn {} · {} tools · {} tokens · ctx {:.0}% | {} | {}",
                match agent.state.mode {
                    NeoCodexMode::Agent => "AGENT",
                    NeoCodexMode::Shell => "SHELL",
                    NeoCodexMode::Plan => "PLAN",
                },
                agent.state.turn_count,
                agent.state.tool_call_count,
                agent.state.tokens_used,
                report.context_usage * 100.0,
                agent.config.provider_name,
                agent.evolution.summary(),
            );
            eprint!("\r{} $ ", status);
            let mut buf = String::new();
            if std::io::stdin().read_line(&mut buf).is_err() || buf.trim().is_empty() {
                eprintln!();
                break;
            }
            buf.trim().to_string()
        };

        match input.as_str() {
            "/q" | "/quit" | "/exit" => break,
            "/mode" | "/m" => {
                let new_mode = agent_ui.agent.lock().await.toggle_mode();
                eprintln!("Mode: {:?}", new_mode);
                continue;
            }
            "/plan" => {
                agent_ui.agent.lock().await.set_plan_mode();
                eprintln!("Plan mode: enter goal description");
                continue;
            }
            "/help" | "/h" => {
                eprintln!("/q /quit /exit   — quit");
                eprintln!("/m /mode         — toggle Agent↔Shell");
                eprintln!("/plan            — enter Plan mode");
                eprintln!("/g <desc> <n>    — add goal");
                eprintln!("/status          — show agent state");
                eprintln!("/health          — run self-audit health checks");
                eprintln!("/evo             — run one evolution iteration");
                eprintln!("/resume          — restore session from wire file");
                eprintln!("any text         — process input");
                continue;
            }
            cmd if cmd.starts_with("/g ") => {
                let parts: Vec<&str> = cmd[3..].rsplitn(2, ' ').collect();
                let (desc, max_iter) = if parts.len() == 2 {
                    (parts[1], parts[0].parse::<u64>().unwrap_or(5))
                } else {
                    (parts[0], 5)
                };
                agent_ui.agent.lock().await.add_goal(desc, max_iter);
                eprintln!("Goal added: {} (max {} iters)", desc, max_iter);
                continue;
            }
            "/status" => {
                let a = agent_ui.agent.lock().await;
                let report = a.health_report();
                eprintln!("Mode: {:?} · Turn {} · {} tools · {} tokens",
                    a.state.mode, a.state.turn_count, a.state.tool_call_count, a.state.tokens_used);
                eprintln!("Provider: {} ({}) · Context: {} turns / {} max tok ({:.0}%)",
                    a.config.provider_name, report.provider_model,
                    a.context.turns.len(), a.context.max_tokens, report.context_usage * 100.0);
                eprintln!("Evolution: {}", a.evolution.summary());
                continue;
            }
            "/health" => {
                let a = agent_ui.agent.lock().await;
                let report = a.health_report();
                let failures = report.failed_checks();
                if failures.is_empty() {
                    eprintln!("[health] {} ✅ all checks pass", report.summary());
                } else {
                    eprintln!("[health] {} ❌ {} failures:", report.summary(), failures.len());
                    for f in &failures {
                        eprintln!("  - {}", f);
                    }
                }
                continue;
            }
            "/evo" => {
                let mut a = agent_ui.agent.lock().await;
                let before = a.evolution.iteration;
                crate::entry::desktop::step_evolution(&mut a).await;
                eprintln!("[evo] iteration {} → {} ({} fixes applied total)",
                    before, a.evolution.iteration, a.evolution.fixes_applied);
                continue;
            }
            "/resume" => {
                let mut a = agent_ui.agent.lock().await;
                let n = a.resume_session();
                eprintln!("[resume] restored {} prior events", n);
                continue;
            }
            _ => {}
        }

        agent_ui.send_message(&input).await;
        if let Some((_, response)) = agent_ui.message_log.last() {
            if response.len() > 80 {
                eprintln!("{}…", response.chars().take(80).collect::<String>());
            } else {
                eprintln!("{}", response);
            }
        }
    }

    eprintln!("NeoCodex session ended ({} turns, {} tools)",
        agent_ui.agent.lock().await.state.turn_count,
        agent_ui.agent.lock().await.state.tool_call_count);
}
