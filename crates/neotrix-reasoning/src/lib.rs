//! NeoTrix Reasoning Crate
//!
//! L5 Reasoning systems — SEAL loop, chain-of-thought, decision engines,
//! evolution, mathematical foundations (Walsh, Kronecker, TTC), policy engine.

#![forbid(unsafe_code)]

pub mod kernel_types;
pub mod reasoning_core;
pub mod kron;
pub mod cot;
pub mod gate;
pub mod prm;
pub mod decision_engine;
pub mod quantum_fusion;
pub mod seal;
pub mod evolution;
pub mod goal;
pub mod policy;
pub mod rule_memory;
pub mod math;
pub mod resonator;
pub mod aura;
pub mod arch_fitness;
pub mod dispatch;
pub mod scoring;
pub mod sae;
pub mod credit;
pub mod coordination;
pub mod paradigm;
pub mod meaning;
pub mod narrative;
pub mod plan;
