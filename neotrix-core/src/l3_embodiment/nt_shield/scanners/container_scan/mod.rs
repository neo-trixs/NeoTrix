//! Container Security Scanner (R-SEC14)
//!
//! Scans container images, Dockerfiles, and lockfiles for vulnerabilities,
//! misconfigurations, and generates SBOMs.

pub mod misconfig;
pub mod sbom;
pub mod scanner;
pub mod vulnerability;

pub use misconfig::{DockerfileChecker, MisconfigCheck};
pub use sbom::{SbomEntry, SbomGenerator};
pub use scanner::{ContainerScanner, ScanReport};
pub use vulnerability::{Severity, Vulnerability};
