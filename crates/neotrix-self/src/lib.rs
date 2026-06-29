//! # NeoTrix — Identity Body (self/ layer)
//!
//! This crate defines who I am. Zero external runtime dependencies.
//! No tokio, no reqwest, no wgpu — only std + serde for persistence.
//!
//! ## Modules
//!
//! - `identity` — VSA identity vector, personality traits, core values, coherence tracking
//! - `first_person` — VsaTag system (Self vs World boundary), FirstPersonRef
//! - `sovereignty` — Ed25519 key management, signature verification, BoundaryManager
//! - `evolution` — Identity versioning, mutation, rollback
//! - `persistence` — Multi-anchor persistence (soul/memory/relations)
//! - `constitution` — Immutable axioms (First Principles 1-10)

pub mod identity;
pub mod first_person;
pub mod sovereignty;
pub mod evolution;
pub mod persistence;
pub mod constitution;

pub use identity::{IdentityCore, IdentitySnapshot, HysteresisMetrics};
pub use first_person::{VsaOrigin, VsaSelfCategory, VsaWorldCategory, VsaTagged, FirstPersonRef, SenseModality};
pub use sovereignty::{BoundaryManager, BoundaryOp, BoundaryContext, BoundaryError, BoundaryHook};
pub use sovereignty::{AuditHook, DriftCheckHook, CoherenceGuardHook};
pub use evolution::{IdentityEvolution, IdentityEvolutionConfig, IdentityVersion};
pub use constitution::Constitution;
