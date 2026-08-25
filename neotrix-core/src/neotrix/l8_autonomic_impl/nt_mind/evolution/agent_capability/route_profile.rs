//! Route Profile — capability routing profile (stub)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteProfile {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub constraints: HashMap<String, String>,
    pub priority: u8,
}

impl RouteProfile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCategory {
    Reasoning,
    Coding,
    Analysis,
    Creative,
    Memory,
    Perception,
    Action,
}

impl Default for TaskCategory {
    fn default() -> Self {
        TaskCategory::Reasoning
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskRoutingPreference {
    pub preferred_providers: Vec<String>,
    pub fallback_providers: Vec<String>,
    pub max_cost_usd: Option<f32>,
    pub max_latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct RouteProfileManager {
    profiles: std::collections::HashMap<String, RouteProfile>,
}

impl RouteProfileManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register(&mut self, profile: RouteProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }
    
    pub fn get(&self, name: &str) -> Option<&RouteProfile> {
        self.profiles.get(name)
    }
}

pub fn load_profiles() -> Vec<RouteProfile> {
    vec![
        RouteProfile::new("default"),
        RouteProfile::new("high-throughput"),
        RouteProfile::new("low-latency"),
        RouteProfile::new("stealth"),
    ]
}
