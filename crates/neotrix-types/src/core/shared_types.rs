//! Unified shared types for all NT-* domains.
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Error,
    Medium,
    Warning,
    Low,
    Info,
    Informational,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Error => "ERROR",
            Severity::Medium => "MEDIUM",
            Severity::Warning => "WARNING",
            Severity::Low => "LOW",
            Severity::Info => "INFO",
            Severity::Informational => "INFORMATIONAL",
        }
    }
    pub fn numeric(&self) -> u8 {
        match self {
            Severity::Critical => 8, Severity::High => 7, Severity::Error => 6,
            Severity::Medium => 5, Severity::Warning => 4, Severity::Low => 3,
            Severity::Info => 2, Severity::Informational => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NtDomain {
    NtCore, NtMind, NtMemory, NtWorld, NtAct, NtIo, NtShield,
    NtMeta, NtRepair, NtGovernance, NtNexus, NtPhysical, NtFeel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus { Healthy, Degraded, Unhealthy, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState { Pending, Running, Completed, Failed, Cancelled, Timeout }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrendDirection { Rising, Stable, Falling, Volatile }
