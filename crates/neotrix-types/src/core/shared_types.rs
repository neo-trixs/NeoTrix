//! Unified shared types for all NT-* domains.
use std::fmt;
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
    Pass,
}

impl Severity {
    pub fn from_score(score: f64) -> Self {
        if score >= 9.0 { Severity::Critical }
        else if score >= 7.0 { Severity::High }
        else if score >= 5.0 { Severity::Medium }
        else if score >= 3.0 { Severity::Low }
        else if score >= 0.1 { Severity::Info }
        else { Severity::Informational }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Error => "Error",
            Severity::Medium => "Medium",
            Severity::Warning => "Warning",
            Severity::Low => "Low",
            Severity::Info => "Info",
            Severity::Informational => "Informational",
            Severity::Pass => "Pass",
        }
    }
    pub fn numeric(&self) -> u8 {
        match self {
            Severity::Critical => 8, Severity::High => 7, Severity::Error => 6,
            Severity::Medium => 5, Severity::Warning => 4, Severity::Low => 3,
            Severity::Info => 2, Severity::Informational => 1, Severity::Pass => 0,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
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
