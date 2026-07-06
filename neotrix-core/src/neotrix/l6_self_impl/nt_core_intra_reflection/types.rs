use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionInput {
    pub reasoning_trace: Vec<String>,
    pub e8_mode_history: Vec<u8>,
    pub outcome_success: Option<bool>,
    pub execution_time_ms: u64,
    pub error_count: u32,
}

impl ReflectionInput {
    pub fn new(
        reasoning_trace: Vec<String>,
        e8_mode_history: Vec<u8>,
        outcome_success: Option<bool>,
        execution_time_ms: u64,
        error_count: u32,
    ) -> Self {
        Self {
            reasoning_trace,
            e8_mode_history,
            outcome_success,
            execution_time_ms,
            error_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionReport {
    pub coherence_score: f64,
    pub efficiency_score: f64,
    pub error_density: f64,
    pub mode_stability: f64,
    pub bottleneck_hops: Vec<String>,
    pub suggestions: Vec<String>,
    pub timestamp: i64,
}

impl ReflectionReport {
    pub fn new(
        coherence_score: f64,
        efficiency_score: f64,
        error_density: f64,
        mode_stability: f64,
        bottleneck_hops: Vec<String>,
        suggestions: Vec<String>,
        timestamp: i64,
    ) -> Self {
        Self {
            coherence_score,
            efficiency_score,
            error_density,
            mode_stability,
            bottleneck_hops,
            suggestions,
            timestamp,
        }
    }
}

pub(crate) const MIN_EXPECTED_STEPS: usize = 3;
