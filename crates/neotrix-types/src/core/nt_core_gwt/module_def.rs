use serde::{Serialize, Deserialize};

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
}

#[derive(Debug, Clone)]
pub struct SpecialistModule {
    pub name: String,
    pub specialist_type: SpecialistType,
    pub module_type: SpecialistType,
    pub activation: f64,
}

impl SpecialistModule {
    pub fn new(specialist_type: SpecialistType, name: String) -> Self {
        let module_type = specialist_type;
        Self { name, specialist_type, module_type, activation: 0.0 }
    }

    pub fn activate(&mut self, salience: f64) {
        self.activation += salience;
    }
}

impl SpecialistType {
    pub fn name(&self) -> &'static str {
        match self {
            SpecialistType::PatternMatcher => "pattern-matcher",
            SpecialistType::AnomalyDetector => "anomaly-detector",
            SpecialistType::KnowledgeRetriever => "knowledge-retriever",
            SpecialistType::CodeAnalyzer => "code-analyzer",
            SpecialistType::Planner => "planner",
            SpecialistType::KnowledgeIntegrator => "knowledge-integrator",
            SpecialistType::GoalPrioritizer => "goal-prioritizer",
            SpecialistType::RiskAssessor => "risk-assessor",
            SpecialistType::CreativityGenerator => "creativity-generator",
            SpecialistType::ReflectionEngine => "reflection-engine",
            SpecialistType::MetaCognitionAnalyst => "meta-cognition-analyst",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            SpecialistType::PatternMatcher => "PM",
            SpecialistType::AnomalyDetector => "AD",
            SpecialistType::KnowledgeRetriever => "KR",
            SpecialistType::CodeAnalyzer => "CA",
            SpecialistType::Planner => "PL",
            SpecialistType::KnowledgeIntegrator => "KI",
            SpecialistType::GoalPrioritizer => "GP",
            SpecialistType::RiskAssessor => "RA",
            SpecialistType::CreativityGenerator => "CG",
            SpecialistType::ReflectionEngine => "RE",
            SpecialistType::MetaCognitionAnalyst => "MA",
        }
    }
}
