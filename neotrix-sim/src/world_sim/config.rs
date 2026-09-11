use serde::{Serialize, Deserialize};

/// Configuration for the simulation world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSimConfig {
    pub world_width: f32,
    pub world_height: f32,
    pub initial_agents: usize,
    pub min_agents: usize,
    pub max_agents: usize,
    pub seed: u64,
    pub ticks_per_hour: u64,
    pub evolution_interval: u64,
    pub phi_compute_interval: u64,
}

impl Default for WorldSimConfig {
    fn default() -> Self {
        Self {
            world_width: 1000.0,
            world_height: 1000.0,
            initial_agents: 10,
            min_agents: 5,
            max_agents: 50,
            seed: 42,
            ticks_per_hour: 100,
            evolution_interval: 500,
            phi_compute_interval: 50,
        }
    }
}

/// Evolution generation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRecord {
    pub generation: u64,
    pub population: usize,
    pub mean_fitness: f64,
    pub max_fitness: f64,
    pub species_count: usize,
    pub mean_phi: f64,
    pub global_coherence: f64,
    pub pressure: f64,
}

/// Snapshot of simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub time: String,
    pub population: usize,
    pub mean_phi: f64,
    pub mean_coherence: f64,
    pub species_count: usize,
    pub evolution_generations: usize,
    pub resources_total: usize,
    pub resources_depleted: usize,
    pub total_relationships: usize,
    pub total_trades: u64,
    pub emotion_dominant: String,
    pub emotion_valence: f32,
    pub emotion_arousal: f32,
    pub emotion_dominance: f32,
    pub safety_alerts: usize,
    pub audit_entries: usize,
}
