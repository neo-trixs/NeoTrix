use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialistType {
    PatternMatcher,
    AnomalyDetector,
    KnowledgeRetriever,
    CodeAnalyzer,
    Planner,
    KnowledgeIntegrator,
    GoalPrioritizer,
    RiskAssessor,
    CreativityGenerator,
    ReflectionEngine,
    MetaCognitionAnalyst,
    AISecurity,
    ImageGenerator,
}

/// Environment-domain patterns this specialist has proven effective in.
/// Maps environment name → list of proven behavioral patterns.
pub type HarnessEvidence = HashMap<String, Vec<String>>;

#[derive(Debug, Clone)]
pub struct SpecialistModule {
    pub name: String,
    pub specialist_type: SpecialistType,
    pub module_type: SpecialistType,
    pub activation: f64,
    /// Life-Harness inspired evidence: which environments this specialist
    /// has succeeded in and what procedural patterns were effective.
    pub harness_evidence: HarnessEvidence,
}

impl SpecialistModule {
    pub fn new(specialist_type: SpecialistType, name: String) -> Self {
        let module_type = specialist_type;
        Self { name, specialist_type, module_type, activation: 0.0, harness_evidence: HashMap::new() }
    }

    pub fn activate(&mut self, salience: f64) {
        self.activation += salience;
    }

    /// Boost activation if this specialist has proven harness patterns for this env.
    pub fn apply_harness_boost(&mut self, env: &str, base_multiplier: f64) -> f64 {
        if let Some(patterns) = self.harness_evidence.get(env) {
            let boost = patterns.len() as f64 * base_multiplier;
            self.activation += boost;
            boost
        } else {
            for (known_env, patterns) in &self.harness_evidence {
                if env.contains(known_env) || known_env.contains(env) {
                    let boost = patterns.len() as f64 * base_multiplier * 0.5;
                    self.activation += boost;
                    return boost;
                }
            }
            0.0
        }
    }

    pub fn record_harness_evidence(&mut self, env: &str, pattern: &str) {
        self.harness_evidence.entry(env.to_string()).or_default().push(pattern.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_module_new() {
        let m = SpecialistModule::new(SpecialistType::Planner, "planner-1".into());
        assert_eq!(m.name, "planner-1");
        assert_eq!(m.activation, 0.0);
    }

    #[test]
    fn test_specialist_module_activate() {
        let mut m = SpecialistModule::new(SpecialistType::AnomalyDetector, "detector".into());
        m.activate(0.5);
        assert!((m.activation - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_specialist_module_activate_stacks() {
        let mut m = SpecialistModule::new(SpecialistType::PatternMatcher, "pm".into());
        m.activate(0.3);
        m.activate(0.4);
        assert!((m.activation - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_specialist_types_are_distinct() {
        let types = vec![
            SpecialistType::PatternMatcher,
            SpecialistType::AnomalyDetector,
            SpecialistType::KnowledgeRetriever,
            SpecialistType::CodeAnalyzer,
            SpecialistType::Planner,
            SpecialistType::KnowledgeIntegrator,
            SpecialistType::GoalPrioritizer,
            SpecialistType::RiskAssessor,
            SpecialistType::CreativityGenerator,
            SpecialistType::ReflectionEngine,
            SpecialistType::MetaCognitionAnalyst,
            SpecialistType::AISecurity,
            SpecialistType::ImageGenerator,
        ];
        let mut unique = types.clone();
        unique.sort_by_key(|t| *t as u8);
        unique.dedup();
        assert_eq!(types.len(), unique.len());
        assert_eq!(types.len(), 13);
    }
}
