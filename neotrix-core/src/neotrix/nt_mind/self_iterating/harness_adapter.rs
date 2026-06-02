use crate::core::CapabilityVector;
use crate::core::nt_core_gwt::module_def::SpecialistType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::neotrix::nt_memory_kb::{KnowledgeBase, KnowledgeNode, NodeType};

/// A serializable harness behavior profile that captures how a harness
/// should adapt its interface for a given environment.
///
/// Inspired by Life-Harness: runtime interface adaptation converts recurring
/// interaction failures into reusable interventions across:
/// - environment contracts
/// - procedural skills
/// - action realization
/// - trajectory regulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessProfile {
    /// Name of the source model this profile was evolved from
    pub source_model: String,
    /// Environment contracts: constraints and invariants
    pub environment_contracts: Vec<String>,
    /// Procedural skills: domain-specific procedures encoded as text
    pub procedural_skills: Vec<String>,
    /// Action realization patterns: how to map intent to action
    pub action_realizations: Vec<String>,
    /// Trajectory regulation rules: when to retry, rollback, or abort
    pub trajectory_rules: Vec<String>,
    /// Per-specialist adaptations indexed by SpecialistType
    pub specialist_adaptations: HashMap<SpecialistType, Vec<String>>,
    /// Performance delta observed when applying this profile
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

    pub fn add_action(&mut self, action: &str) {
        self.action_realizations.push(action.to_string());
    }

    pub fn add_trajectory_rule(&mut self, rule: &str) {
        self.trajectory_rules.push(rule.to_string());
    }

    pub fn add_specialist_adaptation(&mut self, st: SpecialistType, adaptation: &str) {
        self.specialist_adaptations
            .entry(st)
            .or_default()
            .push(adaptation.to_string());
    }

    pub fn merge(&mut self, other: &HarnessProfile) {
        for c in &other.environment_contracts {
            if !self.environment_contracts.contains(c) {
                self.environment_contracts.push(c.clone());
            }
        }
        for s in &other.procedural_skills {
            if !self.procedural_skills.contains(s) {
                self.procedural_skills.push(s.clone());
            }
        }
    }

    pub fn to_prompt_suffix(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if !self.environment_contracts.is_empty() {
            parts.push(format!("[Environment Contracts]\n{}", self.environment_contracts.join("\n")));
        }
        if !self.procedural_skills.is_empty() {
            parts.push(format!("[Procedural Skills]\n{}", self.procedural_skills.join("\n")));
        }
        if !self.action_realizations.is_empty() {
            parts.push(format!("[Action Realizations]\n{}", self.action_realizations.join("\n")));
        }
        if !self.trajectory_rules.is_empty() {
            parts.push(format!("[Trajectory Rules]\n{}", self.trajectory_rules.join("\n")));
        }

        parts.join("\n\n")
    }
}

/// Life-Harness inspired runtime adapter for cross-model harness transfer.
///
/// The key insight from Life-Harness: harnesses evolved only from Qwen3-4B-Instruct
/// trajectories transfer to 17 other models, showing that runtime interface adaptation
/// captures reusable environment-side structure rather than model-specific behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessAdapter {
    /// Profiles indexed by environment name
    pub profiles: HashMap<String, HarnessProfile>,
    /// Currently active profile
    pub active: Option<String>,
    /// Transfer history: (source_model, target_model, environment, delta)
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

        // Apply procedural skill knowledge from the source profile
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

    /// Serialize all profiles into KnowledgeBase as HarnessProfile nodes.
    /// Each profile is stored as one KnowledgeNode with JSON metadata.
    pub fn save_to_kb(&self, kb: &KnowledgeBase) -> Result<usize, String> {
        let mut count = 0;
        for (env, profile) in &self.profiles {
            let json = serde_json::to_value(profile).map_err(|e| format!("Serialize profile: {}", e))?;
            let title = format!("HarnessProfile: {}", env);
            let summary = format!(
                "Harness profile for environment '{}' from model '{}' with {} contracts, {} skills, delta={}",
                env,
                profile.source_model,
                profile.environment_contracts.len(),
                profile.procedural_skills.len(),
                profile.performance_delta,
            );
            let node = KnowledgeNode {
                id: format!("harness-profile-{}", env),
                node_type: NodeType::HarnessProfile,
                title,
                summary: Some(summary),
                content: None,
                url: Some(format!("harness://profile/{}", env)),
                domain: Some("harness".to_string()),
                language: "en".to_string(),
                confidence: 0.9,
                importance: 0.6,
                created_at: 0,
                updated_at: 0,
                access_count: 0,
                metadata: Some(json),
            };
            kb.insert_node(&node)?;
            count += 1;
        }
        Ok(count)
    }

    /// Load all HarnessProfile nodes from KnowledgeBase into this adapter.
    pub fn load_from_kb(kb: &KnowledgeBase) -> Result<Self, String> {
        let nodes = kb.search_by_type(&NodeType::HarnessProfile, 100)?;
        let mut profiles = HashMap::new();
        for node in &nodes {
            if let Some(ref meta) = node.metadata {
                if let Ok(profile) = serde_json::from_value::<HarnessProfile>(meta.clone()) {
                    let env = node.title.strip_prefix("HarnessProfile: ").unwrap_or(&node.title).to_string();
                    profiles.insert(env, profile);
                }
            }
        }
        Ok(Self {
            profiles,
            active: None,
            transfer_history: Vec::new(),
        })
    }
}

impl Default for HarnessAdapter {
    fn default() -> Self {
        Self::new()
    }
}
