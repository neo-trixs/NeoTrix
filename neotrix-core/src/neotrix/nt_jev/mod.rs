//! # nt_jev — Unified Decision Primitives for NeoTrix
//!
//! JEV (Judgment-Evidence-Verdict) provides three structured decision primitives
//! that replace 63+ ad-hoc decision enums across the codebase:
//!
//! - **Noul**: Boolean probability (0.0–1.0) — "Is this true?"
//! - **Choice**: Select one from fixed options — "Which one?"
//! - **Score**: Rate on ordered scale — "How much?"
//!
//! Every answer carries:
//! - `confidence` — distributional集中度 (0.0–1.0)
//! - `needs_review` — human-in-the-loop flag
//! - `reason` — optional explanation
//! - `status` — Selected/Scored/Review/Error
//!
//! ## Architecture Position
//!
//! ```text
//! L0 Substrate ──── (no decisions)
//! L1 Action ─────── ActionRequest → JevDecision
//! L2 Perception ─── Source verdict → JevDecision
//! L3 Shield ─────── SafetyDecision → JevDecision ← PRIMARY GATE
//! L4 Memory ─────── AdmissionDecision → JevDecision
//! L5 Cognition ──── Verdict/GateDecision → JevDecision ← PRIMARY GATE
//! L6 Meta ───────── SupervisorDecision → JevDecision
//! ```

#![forbid(unsafe_code)]

pub mod primitives;
pub mod conversion;
pub mod validation;
pub mod gate;

pub use primitives::*;
pub use conversion::*;
pub use validation::*;
pub use gate::*;
