//! hdlib 2.0 — Hyperdimensional Computing Library
//!
//! A clean wrapper over NeoTrix's VSA backends, providing a unified
//! API for hyperdimensional computing operations. All core operations
//! (bind, bundle, permute, similarity) are exposed via the `ops` module,
//! and the most useful backend types are re-exported here for convenience.

// ── Core VSA backend types ────────────────────────────────────────────
pub use crate::core::nt_core_hcube::{
    BinaryVsaBackend, HLBBind, MapVsaBackend, QuantizedVSA, RotationBind, RotationCodebook,
    SignFlipVsa, SparseBinaryVSA, VSAEngine, VsaBackend, VsaError, VsaVector,
};

// ── Encoders ─────────────────────────────────────────────────────────
pub use crate::core::nt_core_hcube::{AdaptiveVsaEncoder, EncoderMode, TrainableVsaEncoder};

// ── High-level inference & alignment ──────────────────────────────────
#[cfg(feature = "simd-vsa")]
pub use crate::core::nt_core_hcube::CrossModalAligner;
pub use crate::core::nt_core_hcube::MultiHeadResonator;

// ── NAG VSA utilities ────────────────────────────────────────────────
pub use crate::core::nt_core_hcube::{
    batch_nag_bundle, gated_nag_bundle, nag_bundle, nag_similarity, normalize,
};

// ── MANAR attention (from GWT module) ────────────────────────────────
pub use crate::core::nt_core_gwt::manar_attention::{ConceptSlot, ManarAttention, ManarConfig};

// ── VSA dimension constant ───────────────────────────────────────────
/// Default HD computing vector dimension (4096).
pub const HD_DIM: usize = 4096;

/// Unified HD vector type — a dense `Vec<f64>`.
pub type HdVector = Vec<f64>;

pub mod ops;

#[cfg(test)]
mod bench;
