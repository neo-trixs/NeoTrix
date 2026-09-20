//! AI Infrastructure module for NeoTrix.
//!
//! Integrates patterns from arcboxlabs/arcbox:
//! - Isolated environments for AI agents
//! - Fast boot manager (<200ms target)
//! - OCI-compatible container abstraction
//! - Boot and resource metrics tracking
//! - Multi-agent inspection and repair system

pub mod boot;
pub mod inspection;
pub mod isolate;
pub mod metrics;
pub mod oci_runtime;

pub use boot::FastBootManager;
pub use inspection::InspectionOrchestrator;
pub use isolate::IsolatedEnvironment;
pub use metrics::FleetMetrics;
pub use oci_runtime::OciRuntime;
