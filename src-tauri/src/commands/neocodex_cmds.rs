use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::RwLock;

use neotrix::neotrix::l1_body_impl::nt_io_neocodex::{
    NeoCodexAgent, NeoCodexMode, NeoCodexUI, NeoCodexHealthReport, EvolutionLoop,
    WireSession,
};
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

// ===== NeoCodex agent state =====

static NEOCODEX_AGENT: std::sync::LazyLock<Arc<RwLock<Option<NeoCodexAgent>>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(None)));

static NEOCODEX_UI: std::sync::LazyLock<Mutex<NeoCodexUI>> =
    std::sync::LazyLock::new(|| NeoCodexUI::new("neotrix-tauri-session"));

static NEOCODEX_GENERATING: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));

// ===== NeoCodex Tauri Commands =====

#[tauri::command]
pub async fn neocodex_send_message(content: String) -> Result<String, String> {
    let mut generating = NEOCODEX_GENERATING.lock().unwrap_or_else(|e| e.into_inner());
    if *generating {
        return Err("already generating".to_string());
    }
    *generating = true;
    drop(generating);

    let agent = NEOCODEX_AGENT.read().await;
    let agent = match agent.as_ref() {
        Some(a) => a.clone(),
        None => {
            let mut agent = NeoCodexAgent::new("neotrix-tauri");
            agent.provider.sync_from_real();
            *NEOCODEX_AGENT.write().await = Some(agent.clone());
            agent
        }
    };
    drop(agent);

    let mut agent = NEOCODEX_AGENT.write().await;
    let agent = agent.as_mut().ok_or("agent not initialized")?;

    let result = agent.react_loop(&content, 4).await;

    *NEOCODEX_GENERATING.lock().unwrap_or_else(|e| e.into_inner()) = false;

    Ok(result)
}

#[tauri::command]
pub async fn neocodex_health_report() -> Result<NeoCodexHealthReport, String> {
    let agent = NEOCODEX_AGENT.read().await;
    match agent.as_ref() {
        Some(a) => Ok(a.health_report()),
        None => Ok(NeoCodexHealthReport {
            mode: NeoCodexMode::Agent,
            turns: 0,
            tools_used: 0,
            tokens_used: 0,
            context_usage: 0.0,
            provider_count: 0,
            provider_resolvable: false,
            session_writable: false,
            total_cost_usd: 0.0,
            consciousness_score: 0.0,
            brain_connected: false,
            event_bus_active: false,
            evolution_iterations: 0,
            evolution_fixes_applied: 0,
            provider_model: "none".to_string(),
            session_path: String::new(),
            wire_path: String::new(),
        }),
    }
}

#[tauri::command]
pub async fn neocodex_self_audit() -> Result<String, String> {
    let agent = NEOCODEX_AGENT.read().await;
    match agent.as_ref() {
        Some(a) => {
            let audit = a.self_audit();
            Ok(format!(
                "Self-audit: {} checks, {} failures",
                audit.total, audit.failures.len()
            ))
        }
        None => Ok("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_evolution_state() -> Result<String, String> {
    let agent = NEOCODEX_AGENT.read().await;
    match agent.as_ref() {
        Some(a) => Ok(a.evolution.summary()),
        None => Ok("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_resume() -> Result<usize, String> {
    let mut agent = NEOCODEX_AGENT.write().await;
    match agent.as_mut() {
        Some(a) => Ok(a.resume_session()),
        None => Err("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_mode_toggle() -> Result<String, String> {
    let mut agent = NEOCODEX_AGENT.write().await;
    match agent.as_mut() {
        Some(a) => {
            let mode = a.toggle_mode();
            Ok(format!("{:?}", mode))
        }
        None => Err("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_add_goal(desc: String, max_iter: u64) -> Result<String, String> {
    let mut agent = NEOCODEX_AGENT.write().await;
    match agent.as_mut() {
        Some(a) => {
            a.add_goal(desc, max_iter);
            Ok(format!("goal added: {} (max {} iters)", desc, max_iter))
        }
        None => Err("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_evolution_step() -> Result<String, String> {
    let mut agent = NEOCODEX_AGENT.write().await;
    match agent.as_mut() {
        Some(a) => {
            EvolutionLoop::step(a);
            Ok(format!(
                "iteration {} ({} fixes applied)",
                a.evolution.iteration, a.evolution.fixes_applied
            ))
        }
        None => Err("agent not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn neocodex_provider_config() -> Result<String, String> {
    let agent = NEOCODEX_AGENT.read().await;
    match agent.as_ref() {
        Some(a) => Ok(format!(
            "provider={} model={} resolvable={} providers={}",
            a.config.provider_name,
            a.active_model(),
            a.provider.is_resolvable(),
            a.provider.providers.len()
        )),
        None => Ok("agent not initialized".to_string()),
    }
}
