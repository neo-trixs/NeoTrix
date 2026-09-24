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

pub mod consciousness_core;
pub mod consciousness_tree;
pub mod cad_consciousness;
pub mod gwt;
pub mod context;
pub mod consciousness_crystal;
pub mod consciousness_subsystem;
pub mod context_engine;
pub mod echo_terminal;
pub mod panic_recovery;
pub mod second_brain;
pub mod iit_phi;
pub mod kernel_types;
pub mod state;

// Migrated from L5 cognition — standalone consciousness modules
pub mod source_hierarchy;
pub mod cognitive_load;
pub mod bubble_wall;
pub mod vsa_tag;

// Legacy facade re-exports
pub mod legacy;

// Feature re-exports
pub mod features;
