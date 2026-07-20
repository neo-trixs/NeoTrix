use std::sync::Arc;

use tokio::sync::RwLock;

use neotrix::neotrix::l1_body_impl::nt_io_neocodex::{NeoCodexUI, NeoCodexMode};
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

pub(crate) async fn run_tui(_agent: Arc<RwLock<SelfIteratingBrain>>, _ephemeral: bool) {
    let mut agent_ui = NeoCodexUI::new("neotrix-session");
    agent_ui.agent.lock().await.set_brain(_agent.clone());

    loop {
        let input = {
            let agent = agent_ui.agent.lock().await;
            let status = format!(
                "[{}] Turn {} · {} tools · {} tokens | {}",
                match agent.state.mode {
                    NeoCodexMode::Agent => "AGENT",
                    NeoCodexMode::Shell => "SHELL",
                    NeoCodexMode::Plan => "PLAN",
                },
                agent.state.turn_count,
                agent.state.tool_call_count,
                agent.state.tokens_used,
                agent.config.provider_name,
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
                eprintln!("/q /quit /exit  — quit");
                eprintln!("/m /mode        — toggle Agent↔Shell");
                eprintln!("/plan           — enter Plan mode");
                eprintln!("/g <desc> <n>   — add goal");
                eprintln!("/status         — show agent state");
                eprintln!("any text        — process input");
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
                eprintln!("Mode: {:?} · Turn {} · {} tools · {} tokens",
                    a.state.mode, a.state.turn_count, a.state.tool_call_count, a.state.tokens_used);
                eprintln!("Provider: {} · Context: {} turns / {} max tok",
                    a.config.provider_name, a.context.turns.len(), a.context.max_tokens);
                continue;
            }
            _ => {}
        }

        agent_ui.send_message(&input).await;
        if let Some((_, response)) = agent_ui.message_log.last() {
            if response.len() > 80 {
                eprintln!("{}…", &response[..80]);
            } else {
                eprintln!("{}", response);
            }
        }
    }

    eprintln!("NeoCodex session ended ({} turns, {} tools)",
        agent_ui.agent.lock().await.state.turn_count,
        agent_ui.agent.lock().await.state.tool_call_count);
}
