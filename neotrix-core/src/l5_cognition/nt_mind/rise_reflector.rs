//! # RISEReflector — Future Self-Distillation
//!
//! Future self-distillation from arXiv 2607.05188 (RSI path).

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationResult {
    pub knowledge_transferred: usize,
    pub fidelity_score: f64,
    pub distilled_features: Vec<String>,
    pub temporal_distance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureProjection {
    pub projected_capabilities: Vec<String>,
    pub projected_knowledge: Vec<String>,
    pub confidence: f64,
    pub time_horizon: u64,
    pub projected_state_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfComparison {
    pub capability_gap: f64,
    pub knowledge_gap: f64,
    pub overall_gap: f64,
    pub improvement_areas: Vec<String>,
    pub strength_areas: Vec<String>,
}

pub struct RISEReflector {
    time_horizon: u64,
    distillation_threshold: f64,
    max_projections: usize,
    projection_count: Arc<Mutex<usize>>,
    history: Arc<Mutex<Vec<DistillationResult>>>,
}

impl RISEReflector {
    pub fn new(time_horizon: u64, distillation_threshold: f64) -> Self {
        Self {
            time_horizon,
            distillation_threshold,
            max_projections: 100,
            projection_count: Arc::new(Mutex::new(0)),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn distill(&self, current_state: &[String], source_data: &[String]) -> DistillationResult {
        let features = self.extract_features(current_state, source_data);
        let fidelity = self.compute_fidelity(current_state, source_data);
        let knowledge_transferred = features.len();
        let result = DistillationResult {
            knowledge_transferred,
            fidelity_score: fidelity,
            distilled_features: features,
            temporal_distance: self.time_horizon,
        };
        self.history.lock().unwrap().push(result.clone());
        result
    }

    pub fn project_future(&self, current_capabilities: &[String], growth_rate: f64) -> FutureProjection {
        let count = self.projection_count.lock().unwrap();
        if *count >= self.max_projections {
            drop(count);
            *self.projection_count.lock().unwrap() = 0;
        }
        let projected_capabilities = current_capabilities.iter().map(|c| format!("{}+", c)).collect();
        let projected_knowledge = current_capabilities.iter().enumerate().map(|(_i, c)| format!("{}[t+{}]", c, self.time_horizon)).collect();
        let confidence = self.compute_projection_confidence(current_capabilities, growth_rate);
        let projected_state_hash = self.hash_projection(current_capabilities, growth_rate);
        *self.projection_count.lock().unwrap() += 1;
        FutureProjection {
            projected_capabilities,
            projected_knowledge,
            confidence,
            time_horizon: self.time_horizon,
            projected_state_hash,
        }
    }

    pub fn compare_self(&self, current_state: &[String], future_projection: &FutureProjection) -> SelfComparison {
        let capability_gap = self.compute_gap(current_state, &future_projection.projected_capabilities);
        let knowledge_gap = self.compute_gap(current_state, &future_projection.projected_knowledge);
        let overall_gap = (capability_gap + knowledge_gap) / 2.0;
        let improvement_areas = current_state.iter().filter(|s| !future_projection.projected_capabilities.iter().any(|p| p.contains(s.as_str()))).cloned().collect();
        let strength_areas = current_state.iter().filter(|s| future_projection.projected_capabilities.iter().any(|p| p.contains(s.as_str()))).cloned().collect();
        SelfComparison { capability_gap, knowledge_gap, overall_gap, improvement_areas, strength_areas }
    }

    fn extract_features(&self, state: &[String], data: &[String]) -> Vec<String> {
        let mut features = Vec::new();
        for s in state {
            if data.iter().any(|d| d.contains(s)) {
                features.push(format!("feat:{}", s));
            }
        }
        if features.is_empty() { features.push("general_knowledge".to_string()); }
        features
    }

    fn compute_fidelity(&self, state: &[String], data: &[String]) -> f64 {
        if state.is_empty() || data.is_empty() { return 0.0; }
        let overlap = state.iter().filter(|s| data.iter().any(|d| d.contains(s.as_str()))).count();
        (overlap as f64) / (state.len() as f64).max(1.0)
    }

    fn compute_projection_confidence(&self, capabilities: &[String], growth_rate: f64) -> f64 {
        if capabilities.is_empty() { return 0.5; }
        let base_confidence = (growth_rate * 0.5).min(1.0);
        base_confidence * (1.0 / (1.0 + self.time_horizon as f64 * 0.01))
    }

    fn hash_projection(&self, capabilities: &[String], growth_rate: f64) -> String {
        let content = format!("{:?}:{:.2}", capabilities, growth_rate);
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn compute_gap(&self, current: &[String], future: &[String]) -> f64 {
        if current.is_empty() { return 1.0; }
        let matched = current.iter().filter(|c| future.iter().any(|f| f.contains(c.as_str()))).count();
        1.0 - (matched as f64) / (current.len() as f64).max(1.0)
    }

    pub fn get_history(&self) -> Vec<DistillationResult> {
        self.history.lock().unwrap().clone()
    }

    pub fn projection_count(&self) -> usize {
        *self.projection_count.lock().unwrap()
    }
}

impl Default for RISEReflector {
    fn default() -> Self { Self::new(10, 0.7) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_distill() { let r = RISEReflector::default(); let result = r.distill(&["cap1".to_string()], &["cap1 info".to_string()]); assert!(result.fidelity_score >= 0.0 && result.fidelity_score <= 1.0); }
    #[test] fn test_project_future() { let r = RISEReflector::default(); let proj = r.project_future(&["cap1".to_string()], 1.5); assert!(!proj.projected_capabilities.is_empty()); }
    #[test] fn test_compare_self() { let r = RISEReflector::default(); let proj = FutureProjection { projected_capabilities: vec!["cap1+".to_string()], projected_knowledge: vec!["cap1[t+10]".to_string()], confidence: 0.8, time_horizon: 10, projected_state_hash: "abc".to_string() }; let cmp = r.compare_self(&["cap1".to_string()], &proj); assert!(cmp.overall_gap >= 0.0); }
}
