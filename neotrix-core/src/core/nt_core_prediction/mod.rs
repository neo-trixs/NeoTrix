//! # Prediction Module (merged)
//!
//! Consolidates:
//!   - JEPA World Model (joint embedding predictive architecture)
//!   - HyperCube knowledge-augmented prediction bridge
//!   - RSSM-style ensemble predictor
//!
//! ## JEPA — Joint Embedding Predictive Architecture
//! Based on LeCun (2022) "A Path Towards Autonomous Machine Intelligence":
//!   - Prediction in abstract latent space (not pixel space)
//!   - Energy-Based Model — low energy = prediction matches target
//!   - VICReg regularization — prevents representation collapse
//!   - LeWorldModel (2026) — dual loss: next-embedding prediction + Gaussian regularizer
//!
//! ## Knowledge-Augmented Prediction
//! Bridges JEPA latent predictions with HyperCube knowledge storage:
//!   1. Predict state → encode as HyperCoord → query similar history
//!   2. Historical knowledge → adjust prediction (knowledge enhancement)
//!   3. New prediction → store to HyperCube (experience replay)
//!
//! ## RSSM-Style Ensemble Predictor
//! Ensemble of linear predictors forecasting the next latent state from
//! current latent + action encoding + context features. Uncertainty estimated
//! from ensemble disagreement.

pub mod encoder;
pub mod loss;
pub mod nt_world_pred_hcube;
pub mod predictor;
pub mod rgm_jepa;
pub mod td_jepa;
pub mod types;

pub mod rssm;

mod world_model;

#[cfg(test)]
mod tests;

// JEPA re-exports
pub use encoder::JepaEncoder;
pub use loss::{EnergyModel, VicRegLoss};
pub use predictor::{EMAJepaPredictor, JepaPredictor};
pub use rgm_jepa::{CGBlock, MultiScaleJEPA, RGMLatent};
pub use td_jepa::{TDDynamics, TDTarget};
pub use types::{
    LatentState, MultiScalePrediction, TDExperience, WorldModelState, JEPA_COV_WEIGHT,
    JEPA_EMA_MOMENTUM, JEPA_GAUSS_STD_TARGET, JEPA_GAUSS_WEIGHT, JEPA_HIDDEN_DIM, JEPA_INV_WEIGHT,
    JEPA_LATENT_DIM, JEPA_LEARNING_RATE, JEPA_VARIANCE_TARGET, JEPA_VAR_WEIGHT,
};
pub use world_model::JepaWorldModel;

// Knowledge-augmented prediction re-exports
pub use nt_world_pred_hcube::{
    HyperCoordEncoder, KnowledgeAugmentedPredictor, PredictionMemory, ReplayBuffer, CUBE_STORE_DIM,
    KNOWLEDGE_INFLUENCE, MAX_RETRIEVAL, SIMILARITY_THRESHOLD,
};

// RSSM re-exports (from rssm::)
pub use rssm::{PredictionInput, PredictorConfig, PredictorState, StatePrediction};
