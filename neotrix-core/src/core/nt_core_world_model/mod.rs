//! # LoopWM — Looped World Model (G370-G372)
//!
//! Implements the looped latent-state refinement world model from
//! "Looped World Models" (arXiv 2606.18208).
//!
//! ## Sub-modules
//!
//! - `looped_dynamics` — Core LoopWM engine with K-iteration refinement
//! - `spectral_constraint` — Spectral radius constraint for stable dynamics
//! - `adaptive_exit` — Adaptive early-exit gating for computation depth
//! - `latent_state` — VSA-encoded latent state representation
//! - `action_embedding` — Action encoding for the world model

pub mod action_embedding;
pub mod adaptive_exit;
pub mod latent_state;
pub mod looped_dynamics;
pub mod spectral_constraint;

pub use action_embedding::ActionEmbedding;
pub use adaptive_exit::AdaptiveExitGate;
pub use latent_state::VsaLatentState;
pub use looped_dynamics::{DynamicsError, LoopedDynamics};
pub use spectral_constraint::SpectralConstraint;
