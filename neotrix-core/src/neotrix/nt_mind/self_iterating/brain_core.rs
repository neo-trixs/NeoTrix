use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};

use crate::core::SourceAccessTracker;
use crate::neotrix::nt_world_model::TaskType;
use super::super::core::{CapabilityVector, KnowledgeSource, AbsorptionRecord, PerformanceEvaluator};
use super::super::memory::ReasoningBank;
use super::brain_seal::{SealEditStrategy, DefaultSealStrategy};
use super::brain_ewc::{FisherMatrix, WeightUpdateRecord};

/// AbsorbValidator trait
pub trait AbsorbValidator {
    fn validate_absorb(&self, after: &CapabilityVector) -> bool;
}

/// Default AbsorbValidator using PerformanceEvaluator
pub struct DefaultAbsorbValidator {
    pub task_type: TaskType,
    pub min_threshold: f64,
}

impl DefaultAbsorbValidator {
    pub fn new(task_type: TaskType, min_threshold: f64) -> Self {
        Self { task_type, min_threshold }
    }
}

impl AbsorbValidator for DefaultAbsorbValidator {
    fn validate_absorb(&self, after: &CapabilityVector) -> bool {
        let score = PerformanceEvaluator::evaluate(&self.task_type, after);
        score >= self.min_threshold
    }
}

/// Self-iteration trait
pub trait SelfIteration {
    type IterationResult;
    type Evaluation;
    fn iterate(&mut self) -> Self::IterationResult;
    fn evaluate(&self) -> Self::Evaluation;
    fn absorb_feedback(&mut self, feedback: f64);
    fn should_continue(&self, threshold: f64) -> bool;
}

#[derive(Debug)]
pub struct ReasoningBrain {
    pub capability: CapabilityVector,
    pub task_affinity: HashMap<TaskType, f64>,
    pub absorption_history: Vec<AbsorptionRecord>,
    pub learning_rate: f64,
    pub total_absorb_count: u64,
    pub custom_sources: HashMap<String, CapabilityVector>,
    pub source_access_tracker: SourceAccessTracker,
    pub harness_history: Vec<String>,
    pub harness_current: Option<String>,
    pub weight_history: Vec<WeightUpdateRecord>,
    pub learning_rate_budget: f64,
    pub max_budget: f64,
    pub strategy: Box<dyn SealEditStrategy>,
    pub fisher: Option<FisherMatrix>,
    pub ewc_lambda: f64,
}

impl ReasoningBrain {
    pub fn new() -> Self {
        Self {
            capability: CapabilityVector::default(),
            task_affinity: HashMap::new(),
            absorption_history: Vec::new(),
            learning_rate: 0.05,
            total_absorb_count: 0,
            custom_sources: HashMap::new(),
            source_access_tracker: SourceAccessTracker::new(3),
            harness_history: Vec::new(),
            harness_current: None,
            weight_history: Vec::new(),
            learning_rate_budget: 5.0,
            max_budget: 10.0,
            strategy: Box::new(DefaultSealStrategy),
            fisher: Some(FisherMatrix::new(23)),
            ewc_lambda: 0.5,
        }
    }

    pub fn register_knowledge_source(&mut self, name: &str, vector: CapabilityVector) {
        self.custom_sources.insert(name.to_string(), vector);
    }

    pub fn absorb_from_custom(&mut self, name: &str) -> bool {
        if let Some(vector) = self.custom_sources.get(name) {
            self.capability.update_from_other(vector, self.learning_rate);
            self.capability.normalize();
            self.absorption_history.push(AbsorptionRecord {
                source: KnowledgeSource::DesignPhilosophy,
                timestamp: Utc::now().timestamp() as u64,
                weight: self.learning_rate,
            });
            self.total_absorb_count += 1;
            true
        } else {
            false
        }
    }

    pub fn harness_update(&mut self, new_scaffold: &str) -> bool {
        let cost = (self.learning_rate * 0.15).max(0.02);
        if self.learning_rate_budget < cost {
            return false;
        }
        self.learning_rate_budget -= cost;
        self.harness_history.push(new_scaffold.to_string());
        self.harness_current = Some(new_scaffold.to_string());
        if let Some(idx) = CapabilityVector::index_from_name("experimental") {
            self.capability.arr_mut()[idx] = (self.capability.arr()[idx] + 0.03).min(1.0);
        }
        if let Some(idx) = CapabilityVector::index_from_name("synthesis") {
            self.capability.arr_mut()[idx] = (self.capability.arr()[idx] + 0.02).min(1.0);
        }
        self.capability.normalize();
        self.total_absorb_count += 1;
        true
    }

    pub fn weight_update(&mut self, reward: f64) -> bool {
        let cost = (self.learning_rate * 0.2).max(0.03);
        if self.learning_rate_budget < cost {
            return false;
        }
        self.learning_rate_budget -= cost;
        let adjustment = (reward * 0.1).min(0.1);
        if let Some(idx) = CapabilityVector::index_from_name("inference_depth") {
            self.capability.arr_mut()[idx] = (self.capability.arr()[idx] + adjustment).min(1.0);
        }
        if let Some(idx) = CapabilityVector::index_from_name("domain_specificity") {
            self.capability.arr_mut()[idx] = (self.capability.arr()[idx] + adjustment).min(1.0);
        }
        self.capability.normalize();
        self.weight_history.push(WeightUpdateRecord {
            generation: self.total_absorb_count,
            reward,
            algorithm: None,
            timestamp: Utc::now().timestamp() as u64,
        });
        self.total_absorb_count += 1;
        true
    }

    pub fn sia_should_switch_to_weight(&self, recent_rewards: &[f64]) -> bool {
        if recent_rewards.len() < 3 {
            return false;
        }
        let last3 = &recent_rewards[recent_rewards.len().saturating_sub(3)..];
        let avg_improvement: f64 = last3.windows(2)
            .map(|w| w[1] - w[0])
            .sum::<f64>() / (last3.len() - 1) as f64;
        avg_improvement.abs() < 0.01
    }

    pub fn replenish_budget(&mut self, amount: f64) {
        self.learning_rate_budget = (self.learning_rate_budget + amount).min(self.max_budget);
    }

    pub fn budget_remaining(&self) -> f64 {
        self.learning_rate_budget
    }

    pub fn list_sources(&self) -> Vec<String> {
        let mut sources: Vec<String> = vec![
            KnowledgeSource::HeroUI.name().to_string(),
            KnowledgeSource::BaseUI.name().to_string(),
            KnowledgeSource::ArcUI.name().to_string(),
            KnowledgeSource::CortexUI.name().to_string(),
            KnowledgeSource::AgenticDS.name().to_string(),
            KnowledgeSource::DesignPhilosophy.name().to_string(),
            KnowledgeSource::DeepSeekTui.name().to_string(),
            KnowledgeSource::Codebuff.name().to_string(),
            KnowledgeSource::OpenClaude.name().to_string(),
            KnowledgeSource::Cairn.name().to_string(),
            KnowledgeSource::Orca.name().to_string(),
            KnowledgeSource::RedRun.name().to_string(),
            KnowledgeSource::AutonomousSpeedrunning.name().to_string(),
        ];
        sources.extend(self.custom_sources.keys().cloned());
        sources
    }

    pub fn initialize_with_design_knowledge(&mut self, bank: &mut ReasoningBank) {
        bank.initialize_with_design_knowledge();
        if let Some(idx) = CapabilityVector::index_from_name("typography") {
            self.capability.arr_mut()[idx] = 0.90;
        }
        if let Some(idx) = CapabilityVector::index_from_name("grid") {
            self.capability.arr_mut()[idx] = 0.88;
        }
        if let Some(idx) = CapabilityVector::index_from_name("color") {
            self.capability.arr_mut()[idx] = 0.92;
        }
        if let Some(idx) = CapabilityVector::index_from_name("whitespace") {
            self.capability.arr_mut()[idx] = 0.85;
        }
        if let Some(idx) = CapabilityVector::index_from_name("accessibility") {
            self.capability.arr_mut()[idx] = 0.82;
        }
        if let Some(idx) = CapabilityVector::index_from_name("compound_composition") {
            self.capability.arr_mut()[idx] = 0.90;
        }
        if let Some(idx) = CapabilityVector::index_from_name("tailwind_proficiency") {
            self.capability.arr_mut()[idx] = 0.88;
        }
        if let Some(idx) = CapabilityVector::index_from_name("react_aria_usage") {
            self.capability.arr_mut()[idx] = 0.85;
        }
        if let Some(idx) = CapabilityVector::index_from_name("ai_native_states") {
            self.capability.arr_mut()[idx] = 0.95;
        }
        if let Some(idx) = CapabilityVector::index_from_name("semantic_layer") {
            self.capability.arr_mut()[idx] = 0.91;
        }
        if let Some(idx) = CapabilityVector::index_from_name("quality_gates") {
            self.capability.arr_mut()[idx] = 0.87;
        }
        if let Some(idx) = CapabilityVector::index_from_name("verification") {
            self.capability.arr_mut()[idx] = 0.89;
        }
        self.capability.normalize();
    }

    pub fn initialize_with_everos_knowledge(&mut self, bank: &mut ReasoningBank) {
        bank.initialize_with_everos_knowledge();
        if let Some(idx) = CapabilityVector::index_from_name("inference_depth") {
            self.capability.arr_mut()[idx] = 0.90;
        }
        if let Some(idx) = CapabilityVector::index_from_name("synthesis") {
            self.capability.arr_mut()[idx] = 0.88;
        }
        if let Some(idx) = CapabilityVector::index_from_name("domain_specificity") {
            self.capability.arr_mut()[idx] = 0.85;
        }
        if let Some(idx) = CapabilityVector::index_from_name("semantic_layer") {
            self.capability.arr_mut()[idx] = 0.90;
        }
        if let Some(idx) = CapabilityVector::index_from_name("analysis") {
            self.capability.arr_mut()[idx] = 0.85;
        }
        self.capability.normalize();
    }

    pub fn initialize_with_awesome_design_knowledge(&mut self, bank: &mut ReasoningBank) {
        bank.initialize_with_awesome_design_knowledge();
        if let Some(idx) = CapabilityVector::index_from_name("figma_tokens") {
            self.capability.arr_mut()[idx] = 0.85;
        }
        if let Some(idx) = CapabilityVector::index_from_name("color_system") {
            self.capability.arr_mut()[idx] = 0.90;
        }
        if let Some(idx) = CapabilityVector::index_from_name("typography_scale") {
            self.capability.arr_mut()[idx] = 0.88;
        }
        if let Some(idx) = CapabilityVector::index_from_name("spacing_grid") {
            self.capability.arr_mut()[idx] = 0.85;
        }
        if let Some(idx) = CapabilityVector::index_from_name("component_anatomy") {
            self.capability.arr_mut()[idx] = 0.87;
        }
        if let Some(idx) = CapabilityVector::index_from_name("accessibility_patterns") {
            self.capability.arr_mut()[idx] = 0.85;
        }
        self.capability.normalize();
    }

    pub fn initialize_with_chinese_cosmology_knowledge(&mut self, bank: &mut ReasoningBank) {
        bank.initialize_with_chinese_cosmology_knowledge();
        if let Some(idx) = CapabilityVector::index_from_name("inference_depth") {
            self.capability.arr_mut()[idx] = 0.92;
        }
        if let Some(idx) = CapabilityVector::index_from_name("analysis") {
            self.capability.arr_mut()[idx] = 0.90;
        }
        if let Some(idx) = CapabilityVector::index_from_name("synthesis") {
            self.capability.arr_mut()[idx] = 0.92;
        }
        if let Some(idx) = CapabilityVector::index_from_name("domain_specificity") {
            self.capability.arr_mut()[idx] = 0.90;
        }
        self.capability.normalize();
    }
}

/// ReasoningBrain metadata (for persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMetadata {
    pub capability: CapabilityVector,
    pub task_affinity: HashMap<TaskType, f64>,
    pub absorption_history: Vec<AbsorptionRecord>,
    pub learning_rate: f64,
    pub total_absorb_count: u64,
    pub custom_sources: HashMap<String, CapabilityVector>,
}

impl Clone for ReasoningBrain {
    fn clone(&self) -> Self {
        Self {
            capability: self.capability.clone(),
            task_affinity: self.task_affinity.clone(),
            absorption_history: self.absorption_history.clone(),
            learning_rate: self.learning_rate,
            total_absorb_count: self.total_absorb_count,
            custom_sources: self.custom_sources.clone(),
            source_access_tracker: self.source_access_tracker.clone(),
            harness_history: self.harness_history.clone(),
            harness_current: self.harness_current.clone(),
            weight_history: self.weight_history.clone(),
            learning_rate_budget: self.learning_rate_budget,
            max_budget: self.max_budget,
            strategy: Box::new(DefaultSealStrategy),
            fisher: self.fisher.clone(),
            ewc_lambda: self.ewc_lambda,
        }
    }
}

impl Default for ReasoningBrain {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
