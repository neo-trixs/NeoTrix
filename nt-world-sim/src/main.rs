use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use neotrix_sim::world_sim::{WorldSim, WorldSimConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimStateDto {
    pub tick: u64,
    pub agent_count: usize,
    pub alive_count: usize,
    pub world_width: f32,
    pub world_height: f32,
    pub mean_energy: f32,
    pub mean_health: f32,
    pub mean_hunger: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfoDto {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub energy: f32,
    pub health: f32,
    pub hunger: f32,
    pub age: u64,
    pub alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMapData {
    pub width: f32,
    pub height: f32,
    pub agents: Vec<AgentInfoDto>,
    pub tick: u64,
}

pub struct SimState {
    pub sim: Mutex<WorldSim>,
}

#[tauri::command]
async fn sim_get_state(state: State<'_, Arc<SimState>>) -> Result<SimStateDto, String> {
    let sim = state.sim.lock().await;
    let alive: Vec<_> = sim.agents.iter().filter(|a| a.core.alive).collect();
    let mean_energy = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.energy).sum::<f32>() / alive.len() as f32 };
    let mean_health = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.health).sum::<f32>() / alive.len() as f32 };
    let mean_hunger = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.hunger).sum::<f32>() / alive.len() as f32 };

    Ok(SimStateDto {
        tick: sim.tick,
        agent_count: sim.agents.len(),
        alive_count: alive.len(),
        world_width: sim.config.world_width,
        world_height: sim.config.world_height,
        mean_energy,
        mean_health,
        mean_hunger,
    })
}

#[tauri::command]
async fn sim_tick(state: State<'_, Arc<SimState>>, n: u32) -> Result<SimStateDto, String> {
    let mut sim = state.sim.lock().await;
    for _ in 0..n {
        sim.tick().await;
    }
    let alive: Vec<_> = sim.agents.iter().filter(|a| a.core.alive).collect();
    let mean_energy = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.energy).sum::<f32>() / alive.len() as f32 };
    let mean_health = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.health).sum::<f32>() / alive.len() as f32 };
    let mean_hunger = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.hunger).sum::<f32>() / alive.len() as f32 };

    Ok(SimStateDto {
        tick: sim.tick,
        agent_count: sim.agents.len(),
        alive_count: alive.len(),
        world_width: sim.config.world_width,
        world_height: sim.config.world_height,
        mean_energy,
        mean_health,
        mean_hunger,
    })
}

#[tauri::command]
async fn sim_get_agents(state: State<'_, Arc<SimState>>) -> Result<Vec<AgentInfoDto>, String> {
    let sim = state.sim.lock().await;
    Ok(sim.agents.iter().map(|a| AgentInfoDto {
        id: a.core.id.clone(),
        x: a.core.position.x,
        y: a.core.position.y,
        energy: a.core.energy,
        health: a.core.health,
        hunger: a.core.hunger,
        age: a.core.age,
        alive: a.core.alive,
    }).collect())
}

#[tauri::command]
async fn sim_get_world_map(state: State<'_, Arc<SimState>>) -> Result<WorldMapData, String> {
    let sim = state.sim.lock().await;
    Ok(WorldMapData {
        width: sim.config.world_width,
        height: sim.config.world_height,
        agents: sim.agents.iter().map(|a| AgentInfoDto {
            id: a.core.id.clone(),
            x: a.core.position.x,
            y: a.core.position.y,
            energy: a.core.energy,
            health: a.core.health,
            hunger: a.core.hunger,
            age: a.core.age,
            alive: a.core.alive,
        }).collect(),
        tick: sim.tick,
    })
}

#[tauri::command]
async fn sim_select_agent(
    state: State<'_, Arc<SimState>>,
    id: String,
) -> Result<Option<AgentInfoDto>, String> {
    let sim = state.sim.lock().await;
    Ok(sim.agents.iter().find(|a| a.core.id == id).map(|a| AgentInfoDto {
        id: a.core.id.clone(),
        x: a.core.position.x,
        y: a.core.position.y,
        energy: a.core.energy,
        health: a.core.health,
        hunger: a.core.hunger,
        age: a.core.age,
        alive: a.core.alive,
    }))
}

#[tauri::command]
async fn sim_inject_action(
    state: State<'_, Arc<SimState>>,
    action: String,
) -> Result<String, String> {
    let _sim = state.sim.lock().await;
    // TODO: map action string to AgentAction and inject
    Ok(format!("action '{}' queued for next tick", action))
}

fn main() {
    let config = WorldSimConfig::default();
    let sim = WorldSim::new(config);
    let state = Arc::new(SimState {
        sim: Mutex::new(sim),
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            sim_get_state,
            sim_tick,
            sim_get_agents,
            sim_get_world_map,
            sim_select_agent,
            sim_inject_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
