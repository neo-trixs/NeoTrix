// Safety Guardrails Module — NT-SHIELD integration for self-evolving agents
// Capability preservation, alignment monitoring, evolution constraints, audit trail

pub mod audit_trail;
pub mod capability_tracker;
pub mod evolution_constraints;
pub mod safety_monitor;

pub use audit_trail::*;
pub use capability_tracker::*;
pub use evolution_constraints::*;
pub use safety_monitor::*;
