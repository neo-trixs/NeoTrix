use std::sync::Mutex;
use tauri::{command, State, Emitter};
use neotrix::neotrix::nt_mind::{ReasoningBrain, ReasoningBank};
use neotrix::neotrix::nt_mind::KnowledgeSource;
use neotrix::neotrix::nt_io_avatar::{AvatarIdentity, MessageDirection, DistillationEngine, DistillationFlowEvent, UserAvatar, AuthRequest};
use super::{BrainStats, ChainStats};

#[command]
pub fn get_brain_stats(brain: State<'_, Mutex<ReasoningBrain>>, bank: State<'_, Mutex<ReasoningBank>>) -> BrainStats {
    let brain = brain.lock().expect("Brain mutex poisoned in get_brain_stats");
    let bank = bank.lock().expect("Bank mutex poisoned in get_brain_stats");
    let stats = brain.get_statistics();
    BrainStats {
        iteration: 0,
        absorb_count: brain.total_absorb_count,
        capability_sum: stats.capability_sum,
        memory_count: bank.memories().len(),
        engine_active: false,
        capability_vector: brain.capability.arr.to_vec(),
        dimension_names: (0..brain.capability.total_dim()).map(|i| format!("dim_{}", i)).collect(),
    }
}

#[command]
pub fn absorb_source(brain: State<'_, Mutex<ReasoningBrain>>, source: String) -> Result<String, String> {
    let source = match source.to_lowercase().as_str() {
        "heroui" => KnowledgeSource::HeroUI,
        "baseui" => KnowledgeSource::BaseUI,
        "arcui" => KnowledgeSource::ArcUI,
        "cortexui" => KnowledgeSource::CortexUI,
        "agenticds" => KnowledgeSource::AgenticDS,
        "designphilosophy" => KnowledgeSource::DesignPhilosophy,
        _ => return Err(format!("Unknown source: {}. Options: HeroUI, BaseUI, ArcUI, CortexUI, AgenticDS, DesignPhilosophy", source)),
    };
    let mut brain = brain.lock().expect("Brain mutex poisoned in absorb_source");
    brain.absorb(source);
    Ok(format!("Absorbed {}", brain.total_absorb_count))
}

#[command]
pub fn brain_stats(brain: State<'_, Mutex<ReasoningBrain>>, bank: State<'_, Mutex<ReasoningBank>>) -> BrainStats {
    get_brain_stats(brain, bank)
}

#[command]
pub fn search_knowledge(query: String) -> Result<String, String> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("neotrix");
    let knowledge_path = config_dir.join("knowledge.json");
    if !knowledge_path.exists() {
        return Ok("[]".into());
    }
    let content = std::fs::read_to_string(&knowledge_path).map_err(|e| e.to_string())?;
    let entries: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();
    let q = query.to_lowercase();
    let results: Vec<serde_json::Value> = entries.into_iter()
        .filter(|e| {
            let title = e["title"].as_str().unwrap_or("").to_lowercase();
            let content = e["content"].as_str().unwrap_or("").to_lowercase();
            title.contains(&q) || content.contains(&q)
        })
        .take(10)
        .map(|e| {
            serde_json::json!({
                "id": e["id"],
                "title": e["title"],
                "content": e["content"].as_str().unwrap_or("").chars().take(200).collect::<String>(),
                "relevance": 1.0
            })
        })
        .collect();
    serde_json::to_string(&results).map_err(|e| e.to_string())
}

#[command]
pub fn get_user_avatar(engine: State<'_, Mutex<DistillationEngine>>) -> UserAvatar {
    engine.lock().expect("DistillationEngine mutex poisoned").get_avatar().clone()
}

#[command]
pub fn get_distillation_flow(engine: State<'_, Mutex<DistillationEngine>>) -> DistillationFlowEvent {
    engine.lock().expect("DistillationEngine mutex poisoned").get_flow()
}

#[command]
pub fn distill_message(app: tauri::AppHandle, engine: State<'_, Mutex<DistillationEngine>>, text: String) -> DistillationFlowEvent {
    let mut eng = engine.lock().expect("DistillationEngine mutex poisoned");
    let event = eng.distill_message(&text);
    let _ = app.emit("distillation-update", &event);
    let _ = app.emit("avatar-updated", eng.get_avatar());
    event
}

#[command]
pub fn set_user_identity(app: tauri::AppHandle, engine: State<'_, Mutex<DistillationEngine>>, name: String) -> UserAvatar {
    let mut eng = engine.lock().expect("DistillationEngine mutex poisoned");
    eng.set_identity(&name);
    let avatar = eng.get_avatar().clone();
    let _ = app.emit("distillation-update", &eng.get_flow());
    let _ = app.emit("avatar-updated", &avatar);
    avatar
}

#[command]
pub fn get_identity(engine: State<'_, Mutex<DistillationEngine>>) -> Option<AvatarIdentity> {
    engine.lock().expect("DistillationEngine mutex poisoned").identity.clone()
}

#[command]
pub fn get_chain_stats(engine: State<'_, Mutex<DistillationEngine>>) -> ChainStats {
    let eng = engine.lock().expect("DistillationEngine mutex poisoned");
    let outbound = eng.chain.query_by_direction(&MessageDirection::Outbound).len();
    let inbound = eng.chain.query_by_direction(&MessageDirection::Inbound).len();
    ChainStats {
        total_entries: eng.chain.len(),
        outbound_count: outbound,
        inbound_count: inbound,
        genesis_hash: eng.chain.genesis_hash.clone(),
        chain_valid: eng.chain.entries.is_empty() || eng.chain.verify_chain(
            &eng.identity.as_ref().map(|i| i.secret()).unwrap_or_default()
        ),
        identity_name: eng.avatar.identity_name.clone(),
        identity_edition: eng.avatar.edition,
    }
}

#[command]
pub fn brain_write_back(engine: State<'_, Mutex<DistillationEngine>>, text: String) -> usize {
    engine.lock().expect("DistillationEngine mutex poisoned").brain_write_back(&text)
}

#[command]
pub fn auto_distill(engine: State<'_, Mutex<DistillationEngine>>) -> String {
    engine.lock().expect("DistillationEngine mutex poisoned").auto_distill()
}

#[command]
pub fn request_capability(engine: State<'_, Mutex<DistillationEngine>>, capability: String, reasoning: String) -> AuthRequest {
    engine.lock().expect("DistillationEngine mutex poisoned").request_capability(&capability, &reasoning)
}

#[command]
pub fn check_auth(engine: State<'_, Mutex<DistillationEngine>>, capability: String) -> bool {
    engine.lock().expect("DistillationEngine mutex poisoned").check_auth(&capability)
}

#[command]
pub fn grant_capability(engine: State<'_, Mutex<DistillationEngine>>, capability: String) -> bool {
    engine.lock().expect("DistillationEngine mutex poisoned").grant_capability(&capability)
}

#[command]
pub fn revoke_capability(engine: State<'_, Mutex<DistillationEngine>>, capability: String) -> bool {
    engine.lock().expect("DistillationEngine mutex poisoned").revoke_capability(&capability)
}
