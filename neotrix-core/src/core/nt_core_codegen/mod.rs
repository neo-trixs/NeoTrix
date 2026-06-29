pub mod armor;
pub mod behavioral_equiv;
pub mod bootstrap_identity;
pub mod bridge;
pub mod comptime;
pub mod ne_transpiler;

#[cfg(test)]
pub mod bootstrap_proof_test;
// pub mod ne_edit; // moved to neotrix/nt_mind/ne_edit.rs
pub mod pc3_codegen;
pub mod stdlib;

pub use armor::ArmorGenerator;
pub use behavioral_equiv::{
    BehavioralEquivalenceTest, EquivalenceResult, EquivalenceRunner, EquivalenceSuite,
};
pub use bridge::CodegenBridge;
pub use pc3_codegen::{Pc3Generator, Pc3Pipeline, Pc3Report, ProofAnnotation, ProofCarryingBlock};

pub use crate::core::nt_core_util::codegen_version;
