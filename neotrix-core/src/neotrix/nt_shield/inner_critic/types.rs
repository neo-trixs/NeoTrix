#![allow(dead_code)]
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CriticSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for CriticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CriticSeverity::Info => write!(f, "info"),
            CriticSeverity::Low => write!(f, "low"),
            CriticSeverity::Medium => write!(f, "medium"),
            CriticSeverity::High => write!(f, "high"),
            CriticSeverity::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesignViolation {
    pub id: String,
    pub name: String,
    pub severity: CriticSeverity,
    pub description: String,
    pub location: String,
    pub suggestion: String,
}

pub trait AntiPatternDetector: Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, content: &str) -> Vec<DesignViolation>;
}
