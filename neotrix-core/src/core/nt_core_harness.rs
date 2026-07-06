use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_traits::SpecialistType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A serializable harness behavior profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessProfile {
    pub source_model: String,
    pub environment_contracts: Vec<String>,
    pub procedural_skills: Vec<String>,
    pub action_realizations: Vec<String>,
    pub trajectory_rules: Vec<String>,
    pub specialist_adaptations: HashMap<SpecialistType, Vec<String>>,
    pub performance_delta: f64,
}

impl HarnessProfile {
    pub fn new(source_model: &str) -> Self {
        Self {
            source_model: source_model.to_string(),
            environment_contracts: Vec::new(),
            procedural_skills: Vec::new(),
            action_realizations: Vec::new(),
            trajectory_rules: Vec::new(),
            specialist_adaptations: HashMap::new(),
            performance_delta: 0.0,
        }
    }

    pub fn add_contract(&mut self, contract: &str) {
        self.environment_contracts.push(contract.to_string());
    }

    pub fn add_skill(&mut self, skill: &str) {
        self.procedural_skills.push(skill.to_string());
    }

    pub fn add_realization(&mut self, realization: &str) {
        self.action_realizations.push(realization.to_string());
    }

    pub fn add_rule(&mut self, rule: &str) {
        self.trajectory_rules.push(rule.to_string());
    }

    pub fn add_specialist_adaptation(&mut self, specialist: SpecialistType, adaptation: &str) {
        self.specialist_adaptations.entry(specialist).or_default().push(adaptation.to_string());
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("Source model: {}", self.source_model));
        parts.push(format!("Contracts ({}): {}", self.environment_contracts.len(), self.environment_contracts.join(", ")));
        parts.push(format!("Skills ({}): {}", self.procedural_skills.len(), self.procedural_skills.join(", ")));
        parts.push(format!("Realizations ({}): {}", self.action_realizations.len(), self.action_realizations.join(", ")));
        parts.push(format!("Trajectory rules ({}): {}", self.trajectory_rules.len(), self.trajectory_rules.join(", ")));
        parts.push(format!("Performance delta: {}", self.performance_delta));
        parts.join("\n\n")
    }
}

/// Life-Harness inspired runtime adapter for cross-model harness transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessAdapter {
    pub profiles: HashMap<String, HarnessProfile>,
    pub active: Option<String>,
    pub transfer_history: Vec<(String, String, String, f64)>,
}

impl HarnessAdapter {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            active: None,
            transfer_history: Vec::new(),
        }
    }

    pub fn register_profile(&mut self, env: &str, profile: HarnessProfile) {
        self.profiles.insert(env.to_string(), profile);
    }

    pub fn activate(&mut self, env: &str) -> Option<&HarnessProfile> {
        if self.profiles.contains_key(env) {
            self.active = Some(env.to_string());
            self.profiles.get(env)
        } else {
            None
        }
    }

    pub fn active_profile(&self) -> Option<&HarnessProfile> {
        self.active.as_ref().and_then(|env| self.profiles.get(env))
    }

    pub fn transfer_to(
        &mut self,
        source_env: &str,
        target_model: &str,
        capability: &mut CapabilityVector,
    ) -> bool {
        let profile = match self.profiles.get(source_env) {
            Some(p) => p,
            None => return false,
        };

        if let Some(idx) = CapabilityVector::index_from_name("synthesis") {
            let boost = profile.performance_delta * 0.1;
            capability.arr_mut()[idx] = (capability.arr()[idx] + boost).min(1.0);
        }
        if let Some(idx) = CapabilityVector::index_from_name("domain_specificity") {
            let boost = profile.procedural_skills.len() as f64 * 0.02;
            capability.arr_mut()[idx] = (capability.arr()[idx] + boost).min(1.0);
        }
        capability.normalize();

        self.transfer_history.push((
            profile.source_model.clone(),
            target_model.to_string(),
            source_env.to_string(),
            profile.performance_delta,
        ));

        true
    }

    pub fn record_transfer_result(&mut self, _env: &str, delta: f64) {
        if let Some(profile) = self.active.as_ref().and_then(|e| self.profiles.get_mut(e)) {
            profile.performance_delta = delta;
        }
    }
}

impl Default for HarnessAdapter {
    fn default() -> Self {
        Self::new()
    }
}
