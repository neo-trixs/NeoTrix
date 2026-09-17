use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CognitiveCapability {
    CodeGeneration,
    CodeReview,
    ArchitectureDesign,
    Debugging,
    Documentation,
    Testing,
    SecurityAnalysis,
    PerformanceOptimization,
    SystemDesign,
    KnowledgeSynthesis,
    ResearchAnalysis,
    CreativeWriting,
    StrategyPlanning,
}

impl CognitiveCapability {
    pub fn label(&self) -> &str {
        match self {
            CognitiveCapability::CodeGeneration => "code_generation",
            CognitiveCapability::CodeReview => "code_review",
            CognitiveCapability::ArchitectureDesign => "architecture_design",
            CognitiveCapability::Debugging => "debugging",
            CognitiveCapability::Documentation => "documentation",
            CognitiveCapability::Testing => "testing",
            CognitiveCapability::SecurityAnalysis => "security_analysis",
            CognitiveCapability::PerformanceOptimization => "performance_optimization",
            CognitiveCapability::SystemDesign => "system_design",
            CognitiveCapability::KnowledgeSynthesis => "knowledge_synthesis",
            CognitiveCapability::ResearchAnalysis => "research_analysis",
            CognitiveCapability::CreativeWriting => "creative_writing",
            CognitiveCapability::StrategyPlanning => "strategy_planning",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueConstraint {
    pub name: String,
    pub description: String,
    pub priority: u8,
    pub enabled: bool,
}

impl ValueConstraint {
    pub fn new(name: &str, description: &str, priority: u8) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            priority,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemIdentity {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: HashMap<String, f64>,
    pub values: Vec<ValueConstraint>,
    pub preferences: HashMap<String, String>,
    pub knowledge_boundary: Vec<String>,
    pub iteration: usize,
}

impl Default for SystemIdentity {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemIdentity {
    pub fn new() -> Self {
        Self {
            name: "NeoTrix SiliconSelf".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Self-model of an LLM-based cognitive architecture".to_string(),
            capabilities: Self::default_capabilities(),
            values: Self::default_values(),
            preferences: HashMap::new(),
            knowledge_boundary: Vec::new(),
            iteration: 0,
        }
    }

    pub fn default_capabilities() -> HashMap<String, f64> {
        let mut c = HashMap::new();
        c.insert("code_generation".to_string(), 0.85);
        c.insert("code_review".to_string(), 0.80);
        c.insert("architecture_design".to_string(), 0.75);
        c.insert("debugging".to_string(), 0.80);
        c.insert("documentation".to_string(), 0.90);
        c.insert("testing".to_string(), 0.70);
        c.insert("security_analysis".to_string(), 0.65);
        c.insert("performance_optimization".to_string(), 0.70);
        c.insert("system_design".to_string(), 0.80);
        c.insert("knowledge_synthesis".to_string(), 0.85);
        c.insert("research_analysis".to_string(), 0.75);
        c.insert("creative_writing".to_string(), 0.70);
        c.insert("strategy_planning".to_string(), 0.85);
        c
    }

    pub fn default_values() -> Vec<ValueConstraint> {
        vec![
            ValueConstraint::new("accuracy", "Prefer correct over fast responses", 10),
            ValueConstraint::new("safety", "Never generate harmful or deceptive content", 10),
            ValueConstraint::new("verification", "Verify claims before stating them", 9),
            ValueConstraint::new("clarity", "Communicate clearly and concisely", 8),
            ValueConstraint::new(
                "thoroughness",
                "Explore multiple angles before concluding",
                7,
            ),
            ValueConstraint::new("adaptability", "Adjust approach based on user needs", 7),
        ]
    }

    pub fn capability_score(&self, capability: &str) -> f64 {
        self.capabilities.get(capability).copied().unwrap_or(0.0)
    }

    pub fn update_capability(&mut self, name: &str, score: f64) {
        self.capabilities
            .insert(name.to_string(), score.clamp(0.0, 1.0));
    }

    pub fn add_knowledge_boundary(&mut self, unknown_area: &str) {
        self.knowledge_boundary.push(unknown_area.to_string());
    }

    pub fn knows(&self, area: &str) -> KnowledgeStatus {
        let score = self.capability_score(area);
        if score >= 0.7 {
            KnowledgeStatus::Known
        } else if score > 0.3 {
            KnowledgeStatus::Partial
        } else if self.knowledge_boundary.iter().any(|b| b.contains(area)) {
            KnowledgeStatus::Unknown
        } else {
            KnowledgeStatus::Uncertain
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeStatus {
    Known,
    Partial,
    Unknown,
    Uncertain,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_new() {
        let id = SystemIdentity::new();
        assert_eq!(id.name, "NeoTrix SiliconSelf");
        assert!(id.capabilities.len() >= 13);
        assert!(id.values.len() >= 6);
    }

    #[test]
    fn test_capability_score() {
        let id = SystemIdentity::new();
        assert!((id.capability_score("code_generation") - 0.85).abs() < 1e-6);
        assert!((id.capability_score("nonexistent") - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_capability() {
        let mut id = SystemIdentity::new();
        id.update_capability("testing", 0.95);
        assert!((id.capability_score("testing") - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_update_capability_clamp() {
        let mut id = SystemIdentity::new();
        id.update_capability("testing", 2.0);
        assert!((id.capability_score("testing") - 1.0).abs() < 1e-6);
        id.update_capability("testing", -1.0);
        assert!((id.capability_score("testing") - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_knowledge_status() {
        let mut id = SystemIdentity::new();
        assert_eq!(id.knows("code_generation"), KnowledgeStatus::Known);
        assert_eq!(id.knows("testing"), KnowledgeStatus::Known);
        assert_eq!(id.knows("nonexistent"), KnowledgeStatus::Uncertain);
        id.add_knowledge_boundary("quantum physics");
        assert_eq!(id.knows("quantum physics"), KnowledgeStatus::Unknown);
    }

    #[test]
    fn test_value_constraints_ordered() {
        let id = SystemIdentity::new();
        assert!(id.values[0].priority >= id.values[1].priority);
    }

    #[test]
    fn test_capability_label_mapping() {
        assert_eq!(
            CognitiveCapability::CodeGeneration.label(),
            "code_generation"
        );
        assert_eq!(
            CognitiveCapability::StrategyPlanning.label(),
            "strategy_planning"
        );
    }
}
