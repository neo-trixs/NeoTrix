use serde::{Deserialize, Serialize};

/// Health scoring system for NeoTrix plugins and subsystems.
/// Pattern from CodeFlow A-F grading + Pond self-healing.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthGrade {
    A, // Excellent (0.9-1.0)
    B, // Good (0.8-0.9)
    C, // Fair (0.7-0.8)
    D, // Poor (0.6-0.7)
    F, // Failing (<0.6)
}

impl std::fmt::Display for HealthGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A (Excellent)"),
            Self::B => write!(f, "B (Good)"),
            Self::C => write!(f, "C (Fair)"),
            Self::D => write!(f, "D (Poor)"),
            Self::F => write!(f, "F (Failing)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthWeights {
    pub plugin_uptime: f64,      // default: 0.25
    pub db_integrity: f64,       // default: 0.20
    pub error_rate: f64,         // default: 0.25
    pub response_latency: f64,   // default: 0.15
    pub dependency_health: f64,  // default: 0.15
}

impl Default for HealthWeights {
    fn default() -> Self {
        Self {
            plugin_uptime: 0.25,
            db_integrity: 0.20,
            error_rate: 0.25,
            response_latency: 0.15,
            dependency_health: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub uptime_secs: u64,
    pub total_calls: u64,
    pub error_count: u64,
    pub avg_latency_ms: f64,
    pub db_ok: bool,
    pub dependencies_ok: bool,
}

impl HealthMetrics {
    pub fn uptime_score(&self) -> f64 {
        (self.uptime_secs as f64 / 3600.0).min(1.0)
    }

    pub fn error_rate_score(&self) -> f64 {
        if self.total_calls == 0 { return 1.0; }
        let rate = self.error_count as f64 / self.total_calls as f64;
        (1.0 - rate).max(0.0)
    }

    pub fn latency_score(&self) -> f64 {
        (1.0 - self.avg_latency_ms / 5000.0).max(0.0)
    }
}

pub struct HealthScorer {
    pub weights: HealthWeights,
}

impl Default for HealthScorer {
    fn default() -> Self {
        Self { weights: HealthWeights::default() }
    }
}

impl HealthScorer {
    pub fn new(weights: HealthWeights) -> Self {
        Self { weights }
    }

    pub fn score(&self, metrics: &HealthMetrics) -> HealthGrade {
        let raw = self.weights.plugin_uptime * metrics.uptime_score()
            + self.weights.db_integrity * if metrics.db_ok { 1.0 } else { 0.0 }
            + self.weights.error_rate * metrics.error_rate_score()
            + self.weights.response_latency * metrics.latency_score()
            + self.weights.dependency_health * if metrics.dependencies_ok { 1.0 } else { 0.0 };

        match raw {
            x if x >= 0.9 => HealthGrade::A,
            x if x >= 0.8 => HealthGrade::B,
            x if x >= 0.7 => HealthGrade::C,
            x if x >= 0.6 => HealthGrade::D,
            _ => HealthGrade::F,
        }
    }
}

/// System-wide health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall: HealthGrade,
    pub plugins: Vec<PluginHealth>,
    pub db_healthy: bool,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealth {
    pub name: String,
    pub grade: HealthGrade,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
}
