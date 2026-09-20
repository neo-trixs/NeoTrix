//! Multi-perspective deliberation council -- inspired by council-of-high-intelligence.
//! Assigns hard questions to deliberately different analytical personas,
//! forces direct disagreement, and returns verdicts preserving dissent.

/// Trait for council personas
pub trait CouncilPersona: Send + Sync {
    fn name(&self) -> &str;
    fn primary_lens(&self) -> &str;
    fn blind_spots(&self) -> &[&str];
    fn analyze(&self, question: &str, evidence: &[EvidenceItem]) -> PersonaStance;
}

/// Evidence label types
#[derive(Debug, Clone)]
pub enum EvidenceLabel {
    Fact,
    Inference,
    Assumption,
    Unknown,
}

/// An evidence item in deliberation
#[derive(Debug, Clone)]
pub struct EvidenceItem {
    pub content: String,
    pub label: EvidenceLabel,
    pub source: String,
}

/// A persona's stance on a question
#[derive(Debug, Clone)]
pub struct PersonaStance {
    pub persona: String,
    pub position: String,
    pub confidence: f64,
    pub dissent: Option<String>,
    pub kill_criteria: Vec<String>,
}

/// Council verdict
#[derive(Debug, Clone)]
pub struct CouncilVerdict {
    pub question: String,
    pub recommendation: String,
    pub unresolved: Vec<String>,
    pub dissent: Vec<PersonaStance>,
    pub kill_criteria: Vec<String>,
    pub next_step: String,
    pub evidence_used: Vec<EvidenceItem>,
}

/// Council mode
#[derive(Debug, Clone)]
pub enum CouncilMode {
    Full,    // Independent analysis + cross-examination + synthesis
    Quick,   // Restate + rapid analysis + final positions
    Duo,     // Opening positions + direct response + final statements
}

/// A council that convenes multiple personas
pub struct Council {
    personas: Vec<Box<dyn CouncilPersona>>,
}

impl Council {
    pub fn new() -> Self {
        Self { personas: Vec::new() }
    }

    pub fn add_persona(&mut self, persona: Box<dyn CouncilPersona>) {
        self.personas.push(persona);
    }

    pub fn convene(&self, question: &str, _mode: CouncilMode) -> CouncilVerdict {
        let stances: Vec<PersonaStance> = self.personas
            .iter()
            .map(|p| p.analyze(question, &[]))
            .collect();

        // Find dissent (positions with low confidence or explicit dissent)
        let dissent: Vec<PersonaStance> = stances.iter()
            .filter(|s| s.dissent.is_some() || s.confidence < 0.5)
            .cloned()
            .collect();

        let recommendation = if dissent.is_empty() {
            "Consensus reached".into()
        } else {
            format!("{} dissenting positions", dissent.len())
        };

        CouncilVerdict {
            question: question.into(),
            recommendation,
            unresolved: vec!["Evidence gaps identified".into()],
            dissent,
            kill_criteria: vec!["Implementation fails validation".into()],
            next_step: "Review dissent and gather evidence".into(),
            evidence_used: Vec::new(),
        }
    }

    pub fn list_personas(&self) -> Vec<&str> {
        self.personas.iter().map(|p| p.name()).collect()
    }
}

impl Default for Council {
    fn default() -> Self { Self::new() }
}
