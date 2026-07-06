use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthProfile {
    pub profile_id: String,
    pub behavior_patterns: Vec<BehaviorPattern>,
    pub fingerprint: String,
    pub last_used: i64,
    pub success_rate: f64,
}

impl StealthProfile {
    pub fn new(
        profile_id: &str,
        behavior_patterns: Vec<BehaviorPattern>,
        fingerprint: String,
    ) -> Self {
        Self {
            profile_id: profile_id.to_string(),
            behavior_patterns,
            fingerprint,
            last_used: 0,
            success_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_type: PatternType,
    pub parameters: HashMap<String, f64>,
    pub weight: f64,
}

impl BehaviorPattern {
    pub fn new(pattern_type: PatternType, parameters: HashMap<String, f64>, weight: f64) -> Self {
        Self {
            pattern_type,
            parameters,
            weight: weight.max(0.0).min(1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    TimingJitter,
    RequestRandomization,
    NoiseInjection,
    ProfileRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthReport {
    pub current_stealth_level: f64,
    pub active_profiles: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub risk_assessment: f64,
}
