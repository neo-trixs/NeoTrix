use serde::{Deserialize, Serialize};
type Vector = Vec<f64>;

pub const JEPA_LATENT_DIM: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModelState {
    pub cpu_usage: f64,
    pub memory_available: f64,
    pub network_latency: f64,
    pub task_queue_depth: u32,
    pub error_rate: f64,
    pub iteration_count: u64,
    pub token_usage_pct: f64,
}

impl WorldModelState {
    pub fn new() -> Self {
        Self {
            cpu_usage: 0.5,
            memory_available: 0.7,
            network_latency: 10.0,
            task_queue_depth: 0,
            error_rate: 0.01,
            iteration_count: 0,
            token_usage_pct: 0.3,
        }
    }

    pub fn to_features(&self) -> Vec<f64> {
        vec![
            self.cpu_usage,
            self.memory_available,
            (self.network_latency / 1000.0).min(1.0),
            (self.task_queue_depth as f64 / 100.0).min(1.0),
            self.error_rate,
            (self.iteration_count as f64 / 10000.0).min(1.0),
            self.token_usage_pct,
        ]
    }

    pub fn _describe_trend(&self, prev: &WorldModelState) -> Vec<String> {
        let mut trends = Vec::new();
        if self.cpu_usage > prev.cpu_usage + 0.05 {
            trends.push("CPU increasing".to_string());
        } else if self.cpu_usage < prev.cpu_usage - 0.05 {
            trends.push("CPU decreasing".to_string());
        } else {
            trends.push("CPU stable".to_string());
        }
        if self.memory_available < prev.memory_available - 0.05 {
            trends.push("Memory decreasing".to_string());
        } else if self.memory_available > prev.memory_available + 0.05 {
            trends.push("Memory increasing".to_string());
        } else {
            trends.push("Memory stable".to_string());
        }
        if self.error_rate > prev.error_rate + 0.01 {
            trends.push("Error rate rising".to_string());
        } else if self.error_rate < prev.error_rate - 0.01 {
            trends.push("Error rate falling".to_string());
        } else {
            trends.push("Error rate stable".to_string());
        }
        trends
    }
}

impl Default for WorldModelState {
    fn default() -> Self {
        Self::new()
    }
}
pub const JEPA_HIDDEN_DIM: usize = 64;
pub const JEPA_EMA_MOMENTUM: f64 = 0.99;
pub const JEPA_VARIANCE_TARGET: f64 = 0.5;
pub const JEPA_VAR_WEIGHT: f64 = 0.1;
pub const JEPA_COV_WEIGHT: f64 = 0.04;
pub const JEPA_INV_WEIGHT: f64 = 1.0;
pub const JEPA_LEARNING_RATE: f64 = 0.001;
pub const JEPA_GAUSS_STD_TARGET: f64 = 1.0;
pub const JEPA_GAUSS_WEIGHT: f64 = 0.01;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiScalePrediction {
    pub short_term: Vector,
    pub medium_term: Vec<Vector>,
    pub long_term_trend: Vector,
    pub uncertainties: Vec<f64>,
    pub total_energy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentState {
    pub value: Vector,
    pub delta: Vector,
}

impl LatentState {
    pub fn new(value: Vector, delta: Vector) -> Self {
        Self { value, delta }
    }

    /// Zero-initialized latent state (convenience constructor).
    pub fn zeros(dim: usize) -> Self {
        Self {
            value: vec![0.0; dim],
            delta: vec![0.0; dim],
        }
    }

    pub fn zero(dim: usize) -> Self {
        Self::zeros(dim)
    }

    /// Default zero state with default dimension.
    pub fn _default_state() -> Self {
        Self::zeros(JEPA_LATENT_DIM)
    }

    /// Compute cosine similarity between two latent states.
    pub fn similarity(&self, other: &LatentState) -> f64 {
        let dot: f64 = self.value.iter().zip(other.value.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f64 = self.value.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = other.value.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDExperience {
    pub z_t: Vector,
    pub reward: f64,
    pub z_t_plus_n: Vector,
}
