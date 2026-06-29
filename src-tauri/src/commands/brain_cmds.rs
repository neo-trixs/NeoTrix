use std::sync::{Arc, Mutex};
use tauri::{command, State};
use tokio::sync::RwLock;
use neotrix::SelfIteratingBrain;
use neotrix::nt_mind::KnowledgeSource;
use super::BrainStats;

#[command]
pub fn get_brain_stats(agent: State<'_, Arc<RwLock<SelfIteratingBrain>>>) -> BrainStats {
    let agent = agent.blocking_read();
    let stats = agent.brain.get_statistics();
    BrainStats {
        iteration: agent.iteration,
        absorb_count: agent.brain.total_absorb_count,
        capability_sum: stats.capability_sum,
        memory_count: agent.reasoning_bank.memories().len(),
        engine_active: agent.reasoning_engine.is_some(),
        capability_vector: agent.brain.capability.arr.to_vec(),
        dimension_names: (0..agent.brain.capability.total_dim()).map(|i| format!("dim_{}", i)).collect(),
    }
}

#[command]
pub fn absorb_source(agent: State<'_, Arc<RwLock<SelfIteratingBrain>>>, source: String) -> Result<String, String> {
    let source = match source.to_lowercase().as_str() {
        "heroui" => KnowledgeSource::HeroUI,
        "baseui" => KnowledgeSource::BaseUI,
        "arcui" => KnowledgeSource::ArcUI,
        "cortexui" => KnowledgeSource::CortexUI,
        "agenticds" => KnowledgeSource::AgenticDS,
        "designphilosophy" => KnowledgeSource::DesignPhilosophy,
        _ => return Err(format!("Unknown source: {}. Options: HeroUI, BaseUI, ArcUI, CortexUI, AgenticDS, DesignPhilosophy", source)),
    };
    let mut agent = agent.blocking_write();
    agent.brain.absorb(source);
    Ok(format!("Absorbed {}", agent.brain.total_absorb_count))
}

#[command]
pub fn brain_stats(agent: State<'_, Arc<RwLock<SelfIteratingBrain>>>) -> BrainStats {
    get_brain_stats(agent)
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

/// Write user text into the consciousness pipeline's pending input buffer.
/// Returns the number of items in the queue after pushing.
/// This is the entry point for user chat input to flow through ConsciousnessIntegration.
#[command]
pub fn brain_write_back(
    text: String,
    pending: State<'_, Arc<Mutex<Vec<String>>>>,
) -> Result<usize, String> {
    let mut guard = pending.lock().map_err(|e| format!("lock error: {}", e))?;
    guard.push(text);
    Ok(guard.len())
}

/// Drain the consciousness response output buffer and return all entries.
/// The background loop pushes CI response text here; this command drains them
/// for the frontend to display.
#[command]
pub fn read_consciousness_response(
    output: State<'_, Arc<Mutex<Vec<String>>>>,
) -> Result<Vec<String>, String> {
    let mut guard = output.lock().map_err(|e| format!("lock error: {}", e))?;
    let drained: Vec<String> = guard.drain(..).collect();
    Ok(drained)
}

// DistillationEngine removed — see git history for archived commands
