pub mod builder;
pub mod metrics;
pub mod severity;

pub use self::builder::CvssBuilder;
pub use self::metrics::{
    AttackComplexity, AttackVector, Availability, Confidentiality, ExploitCodeMaturity, Integrity,
    PrivilegesRequired, RemediationLevel, ReportConfidence, Scope, UserInteraction,
};
pub use self::severity::{CvssScore, Severity};

#[cfg(test)]
mod tests;
