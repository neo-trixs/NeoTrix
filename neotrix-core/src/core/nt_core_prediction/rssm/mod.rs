//! World Model Predictor — RSSM-style latent state prediction.
//!
//! Implements an ensemble of linear predictors that forecast the next
//! latent state from current latent + action encoding + context features.
//! Uncertainty is estimated from ensemble disagreement. Supports online
//! delta-rule updates from observations.

pub mod predictor;
pub mod types;

pub use predictor::PredictorState;
pub use types::{PredictionInput, PredictorConfig, StatePrediction};
