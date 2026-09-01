//! L3 Embodiment Layer - Shield Modules

pub mod nt_shield;

pub mod nt_shield_agentic_scan;
pub mod nt_shield_audit;
pub mod nt_shield_comm;
pub mod nt_shield_oversight;
pub mod nt_shield_propagation_guard;
pub mod nt_shield_recon;
pub mod nt_shield_sandbox;

#[cfg(feature = "sandbox")]
pub mod nt_shield_sandbox_entry;

pub mod nt_shield_sentry;

#[cfg(feature = "stealth-net")]
pub mod nt_shield_stealth_net;

pub mod nt_shield_traffic;
