//! OSINT Automation Engine — parallel module orchestrator.
//!
//! Provides `OsintModule` trait, concrete modules (DNS, WHOIS), and
//! `ReconEngine` for parallel execution with aggregated reporting.
//!
//! R-P48: Zero external binary dependencies.
//! R-P122: Localized maintenance — each module scoped to its own block.

pub mod dns_module;
pub mod module;
pub mod recon_engine;
pub mod whois_module;

pub use dns_module::{DnsModule, DnsRecord, DnsResult};
pub use module::{Finding, ModuleCategory, ModuleInput, ModuleOutput, OsintModule};
pub use recon_engine::{ReconEngine, ReconReport, TimelineEntry};
pub use whois_module::{WhoisModule, WhoisResult};
