//! NeoTrix Consciousness Crate
//!
//! L5 Consciousness systems — state machines, Global Workspace Theory,
//! crystals, trees, CAD consciousness, IIT-phi, context management, emergence.
//!
//! # Architecture
//!
//! ```text
//! ConsciousnessCore ←→ ConsciousnessTree ←→ GWT
//!       ↓                    ↓                ↓
//!  CrystalState      ContextBudget     AttentionRouter
//!       ↓                    ↓                ↓
//!  CAD Consciousness    IIT Phi       Echo Terminal
//! ```

#![forbid(unsafe_code)]

pub mod echo_terminal;

// Migrated from L5 cognition — standalone consciousness modules
pub mod source_hierarchy;
pub mod cognitive_load;
pub mod bubble_wall;
pub mod vsa_tag;

// Legacy facade re-exports
pub mod legacy;

// Feature re-exports
