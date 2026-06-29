//! # Unified Session Persistence
//!
//! Single canonical format for all session persistence in NeoTrix.
//! Replaces ~20 fragmented persistence pathways and 15+ JSON schemas.
//!
//! ## Architecture
//!
//! - `unified_record` — Canonical `UnifiedSessionRecord` type that all subsystems write.
//! - `nts_backend` — NTSSEG-backed persistent store (`RT_SESSION` record type).
//! - `migration` — One-way shims from old formats (`CrossSessionMemory` JSON,
//!   `SessionTranscript` JSONL, plain chat logs) to `UnifiedSessionRecord`.

pub mod migration;
pub mod nts_backend;
pub mod unified_record;
