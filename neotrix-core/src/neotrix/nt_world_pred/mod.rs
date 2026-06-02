//! World Model Predictor — RSSM-style latent state prediction.
//!
//! Implements an ensemble of linear predictors that forecast the next
//! latent state from current latent + action encoding + context features.
//! Uncertainty is estimated from ensemble disagreement. Supports online
//! delta-rule updates from observations.

pub mod types;
pub mod predictor;

pub use types::{StatePrediction, PredictionInput, PredictorConfig};
pub use predictor::PredictorState;
