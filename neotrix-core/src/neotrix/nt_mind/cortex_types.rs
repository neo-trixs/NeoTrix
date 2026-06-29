use crate::core::nt_core_edit::MicroEdit;
use crate::neotrix::nt_world_e8::E8WorldModel;
use crate::neotrix::nt_world_infer::ActiveInferenceEngine;
use crate::neotrix::nt_world_jepa::JepaPredictor;
use serde::{Deserialize, Serialize};

pub const DEFAULT_HORIZON: usize = 5;
pub const DEFAULT_N_SAMPLES: usize = 10;
pub const DEFAULT_ACTION_DIM: usize = 32;
pub const ANOMALY_ENERGY_THRESHOLD: f64 = 2.0;
pub const CONFIDENCE_LOW_THRESHOLD: f64 = 0.3;
pub const FE_DIVERGENCE_THRESHOLD: f64 = 5.0;
pub const FORECAST_QUALITY_ALPHA: f64 = 0.3;
pub const QUALITY_REPAIR_THRESHOLD: f64 = 0.4;
pub const MIN_CONSECUTIVE_DEGRADATIONS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedStep {
    pub step: usize,
    pub latent_mean: Vec<f64>,
    pub latent_variance: Vec<f64>,
    pub hexagram_state: Vec<f64>,
    pub free_energy: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonForecast {
    pub trajectory: Vec<PredictedStep>,
    pub cumulative_fe: f64,
    pub avg_confidence: f64,
    pub anomaly_predicted: bool,
    pub divergence_step: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub best_action: Vec<f64>,
    pub forecast: HorizonForecast,
    pub action_rankings: Vec<(usize, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub forecast_cumulative_fe: f64,
    pub forecast_confidence: f64,
    pub prediction_error: f64,
    pub actual_fe: f64,
    pub was_anomaly: bool,
    pub cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairSignal {
    pub diagnosis: String,
    pub severity: f64,
    pub suggested_edits: Vec<MicroEdit>,
    pub target_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveCortex {
    pub jepa: JepaPredictor,
    pub e8: E8WorldModel,
    pub ai: ActiveInferenceEngine,
    pub horizon: usize,
    pub n_samples: usize,
    pub action_dim: usize,
    pub forecast_history: Vec<HorizonForecast>,
    pub fe_timeline: Vec<f64>,
    pub forecast_quality: f64,
    pub consecutive_degradations: usize,
    pub outcome_history: Vec<OutcomeRecord>,
    pub cycle: u64,
}
