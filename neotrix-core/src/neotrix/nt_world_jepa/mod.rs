//! JEPA World Model — 联合嵌入预测架构
//!
//! Based on LeCun (2022) "A Path Towards Autonomous Machine Intelligence":
//!   - Prediction in abstract latent space (not pixel space)
//!   - Energy-Based Model — low energy = prediction matches target
//!   - VICReg regularization — prevents representation collapse
//!   - LeWorldModel (2026) — dual loss: next-embedding prediction + Gaussian regularizer
//!
//! Core equation:
//!   S_y = Predictor(Encoder(x))           # predict y's representation from x
//!   E = ||S_y - sg(TargetEncoder(y))||²    # Energy = prediction error (sg=stop gradient)
//!   L = E + λ_var·V(z) + λ_cov·C(z)        # VICReg total loss
//!
//! Integrated with NeoTrix:
//!   - CapabilityVector as abstract representation space
//!   - E8 64-hexagram as hidden state space
//!   - HyperCube for knowledge-enhanced prediction
//!   - SelectableOperator for state-space temporal dynamics

pub mod types;
pub mod encoder;
pub mod predictor;
pub mod loss;
pub mod sigreg;
pub mod td_jepa;
pub mod rgm_jepa;
mod world_model;

#[cfg(test)]
mod tests;

// Re-exports — all pub items accessible at crate::neotrix::nt_world_jepa::*
pub use types::{
    JEPA_LATENT_DIM, JEPA_HIDDEN_DIM, JEPA_EMA_MOMENTUM, JEPA_VARIANCE_TARGET,
    JEPA_VAR_WEIGHT, JEPA_COV_WEIGHT, JEPA_INV_WEIGHT, JEPA_LEARNING_RATE,
    JEPA_GAUSS_STD_TARGET, JEPA_GAUSS_WEIGHT,
    MultiScalePrediction, LatentState, TDExperience, WorldModelState,
};
pub use encoder::JepaEncoder;
pub use predictor::JepaPredictor;
pub use loss::{EnergyModel, VicRegLoss};
pub use sigreg::SIGReg;
pub use td_jepa::{TDTarget, TDDynamics};
pub use rgm_jepa::{CGBlock, MultiScaleJEPA, RGMLatent};
pub use world_model::JepaWorldModel;
