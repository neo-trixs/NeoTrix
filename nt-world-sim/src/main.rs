use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
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
    pub heightmap: Vec<Vec<f32>>,
    pub biomes: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimFullState {
    pub tick: u64,
    pub agents: Vec<AgentInfoDto>,
    pub day_time: f32,
    pub weather: String,
    pub population: usize,
    pub alive_count: usize,
    pub mean_fitness: f32,
    pub food_count: usize,
    pub species_count: usize,
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
async fn sim_tick(state: State<'_, Arc<SimState>>, n: u32, app: AppHandle) -> Result<SimStateDto, String> {
    let mut sim = state.sim.lock().await;
    for _ in 0..n {
        sim.tick().await;
    }
    let alive: Vec<_> = sim.agents.iter().filter(|a| a.core.alive).collect();
    let mean_energy = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.energy).sum::<f32>() / alive.len() as f32 };
    let mean_health = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.health).sum::<f32>() / alive.len() as f32 };
    let mean_hunger = if alive.is_empty() { 0.0 } else { alive.iter().map(|a| a.core.hunger).sum::<f32>() / alive.len() as f32 };

    let dto = SimStateDto {
        tick: sim.tick,
        agent_count: sim.agents.len(),
        alive_count: alive.len(),
        world_width: sim.config.world_width,
        world_height: sim.config.world_height,
        mean_energy,
        mean_health,
        mean_hunger,
    };
    let _ = app.emit("sim-update", &dto);
    Ok(dto)
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
    let heightmap: Vec<Vec<f32>> = sim.heightmap.data().iter().map(|row| row.clone()).collect();
    let biomes: Vec<Vec<String>> = sim.biome_map.data().iter().map(|row| {
        row.iter().map(|b| format!("{:?}", b)).collect()
    }).collect();
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
        heightmap,
        biomes,
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

#[tauri::command]
async fn sim_get_full_state(state: State<'_, Arc<SimState>>) -> Result<SimFullState, String> {
    let sim = state.sim.lock().await;
    let alive: Vec<_> = sim.agents.iter().filter(|a| a.core.alive).collect();
    let alive_count = alive.len();
    let mean_fitness = if alive.is_empty() {
        0.0
    } else {
        alive.iter().map(|a| a.core.health).sum::<f32>() / alive.len() as f32
    };

    let food_count = sim.resources.total_nodes() - sim.resources.depleted_nodes();
    let species_count = sim.speciation.species_count();
    let day_time = sim.daynight.time_of_day;
    let weather = sim.weather.display();

    Ok(SimFullState {
        tick: sim.tick,
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
        day_time,
        weather,
        population: sim.agents.len(),
        alive_count,
        mean_fitness,
        food_count,
        species_count,
    })
}

fn start_tick_loop(app: AppHandle, state: Arc<SimState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let full = {
                let mut sim = state.sim.lock().await;
                sim.tick().await;
                let alive: Vec<_> = sim.agents.iter().filter(|a| a.core.alive).collect();
                let alive_count = alive.len();
                let mean_fitness = if alive.is_empty() {
                    0.0
                } else {
                    alive.iter().map(|a| a.core.health).sum::<f32>() / alive.len() as f32
                };
                let food_count = sim.resources.total_nodes() - sim.resources.depleted_nodes();
                let species_count = sim.speciation.species_count();
                let day_time = sim.daynight.time_of_day;
                let weather = sim.weather.display();
                SimFullState {
                    tick: sim.tick,
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
                    day_time,
                    weather,
                    population: sim.agents.len(),
                    alive_count,
                    mean_fitness,
                    food_count,
                    species_count,
                }
            };
            let _ = app.emit("sim-update", &full);
        }
    });
}

fn start_heartbeat_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
        loop {
            interval.tick().await;
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let _ = app.emit("sim-heartbeat", serde_json::json!({"ts": ts}));
        }
    });
}

fn main() {
    let config = WorldSimConfig::default();
    let sim = WorldSim::new(config);
    let state = Arc::new(SimState {
        sim: Mutex::new(sim),
    });

    let state_clone = state.clone();
    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            start_tick_loop(app.handle().clone(), state_clone);
            start_heartbeat_loop(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sim_get_state,
            sim_tick,
            sim_get_agents,
            sim_get_world_map,
            sim_select_agent,
            sim_inject_action,
            sim_get_full_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
