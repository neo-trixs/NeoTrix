use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// L0 共享专型枚举（定义在 nt_core_traits 中以防 L4→L5 反向依赖）
pub use crate::core::nt_core_traits::SpecialistType;

/// Environment-domain patterns this specialist has proven effective in.
/// Maps environment name → list of proven behavioral patterns.
pub type HarnessEvidence = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistModule {
    pub name: String,
    pub specialist_type: SpecialistType,
    pub module_type: SpecialistType,
    pub activation: f64,
    /// Life-Harness inspired evidence: which environments this specialist
    /// has succeeded in and what procedural patterns were effective.
    pub harness_evidence: HarnessEvidence,
}

/// Orchestrator phase state machine (agtx-inspired kanban).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrchestratorPhase {
    Backlog,
    Planning,
    Running,
    Review,
    Done,
    Cancelled,
}

impl OrchestratorPhase {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Planning | Self::Running | Self::Review)
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }
    pub fn advance(&self) -> Self {
        match self {
            Self::Backlog => Self::Planning,
            Self::Planning => Self::Running,
            Self::Running => Self::Review,
            Self::Review => Self::Done,
            Self::Done | Self::Cancelled => *self,
        }
    }
}

/// OrchestratorAgent (agtx-inspired): manages sub-agent task lifecycle.
/// One AI agent routes tasks to appropriate specialists, creates isolated
/// worktrees, and coordinates multi-phase execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorAgent {
    pub name: String,
    pub task_id: String,
    pub phase: OrchestratorPhase,
    pub assigned_specialists: Vec<SpecialistType>,
    pub current_specialist: Option<SpecialistType>,
    pub phase_results: Vec<(OrchestratorPhase, String)>,
    pub worktree_path: Option<String>,
    pub task_spec: String,
}

impl OrchestratorAgent {
    pub fn new(name: &str, task_id: &str, spec: &str) -> Self {
        Self {
            name: name.to_string(),
            task_id: task_id.to_string(),
            phase: OrchestratorPhase::Backlog,
            assigned_specialists: Vec::new(),
            current_specialist: None,
            phase_results: Vec::new(),
            worktree_path: None,
            task_spec: spec.to_string(),
        }
    }

    pub fn assign(&mut self, specialist: SpecialistType) {
        self.assigned_specialists.push(specialist);
    }

    pub fn advance_phase(&mut self, result: &str) -> OrchestratorPhase {
        self.phase_results.push((self.phase, result.to_string()));
        self.phase = self.phase.advance();
        self.phase
    }

    pub fn current_specialist_for_phase(&self) -> Option<SpecialistType> {
        match self.phase {
            OrchestratorPhase::Backlog => None,
            OrchestratorPhase::Planning => self
                .assigned_specialists
                .iter()
                .find(|s| matches!(s, SpecialistType::Planner | SpecialistType::GoalPrioritizer))
                .copied(),
            OrchestratorPhase::Running => self.assigned_specialists.first().copied(),
            OrchestratorPhase::Review => self
                .assigned_specialists
                .iter()
                .find(|s| {
                    matches!(
                        s,
                        SpecialistType::ReflectionEngine | SpecialistType::MetaCognitionAnalyst
                    )
                })
                .copied(),
            OrchestratorPhase::Done | OrchestratorPhase::Cancelled => None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.phase.is_terminal()
    }
}

impl SpecialistModule {
    pub fn new(specialist_type: SpecialistType, name: String) -> Self {
        let module_type = specialist_type;
        Self {
            name,
            specialist_type,
            module_type,
            activation: 0.0,
            harness_evidence: HashMap::new(),
        }
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
        self.harness_evidence
            .entry(env.to_string())
            .or_default()
            .push(pattern.to_string());
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
            SpecialistType::EvidenceWeightedHypothesis,
            SpecialistType::Orchestrator,
        ];
        let mut unique = types.clone();
        unique.sort_by_key(|t| *t as u8);
        unique.dedup();
        assert_eq!(types.len(), unique.len());
        assert_eq!(types.len(), 15);
    }

    #[test]
    fn test_orchestrator_phase_lifecycle() {
        let phase = OrchestratorPhase::Backlog;
        assert!(!phase.is_active());
        assert!(!phase.is_terminal());
        let p1 = phase.advance();
        assert_eq!(p1, OrchestratorPhase::Planning);
        assert!(p1.is_active());
        let p2 = p1.advance();
        assert_eq!(p2, OrchestratorPhase::Running);
        let p3 = p2.advance();
        assert_eq!(p3, OrchestratorPhase::Review);
        let p4 = p3.advance();
        assert_eq!(p4, OrchestratorPhase::Done);
        assert!(p4.is_terminal());
    }

    #[test]
    fn test_orchestrator_agent_creation() {
        let agent = OrchestratorAgent::new("orch-1", "task-42", "Implement RAG pipeline");
        assert_eq!(agent.phase, OrchestratorPhase::Backlog);
        assert!(agent.assigned_specialists.is_empty());
    }

    #[test]
    fn test_orchestrator_agent_assign_and_advance() {
        let mut agent = OrchestratorAgent::new("orch-1", "task-42", "Test");
        agent.assign(SpecialistType::Planner);
        agent.assign(SpecialistType::CodeAnalyzer);
        assert_eq!(agent.assigned_specialists.len(), 2);
        let phase = agent.advance_phase("plan ready");
        assert_eq!(phase, OrchestratorPhase::Planning);
        let specialist = agent.current_specialist_for_phase();
        assert_eq!(specialist, Some(SpecialistType::Planner));
    }

    #[test]
    fn test_orchestrator_agent_complete() {
        let mut agent = OrchestratorAgent::new("orch-1", "task-1", "short");
        agent.advance_phase("start");
        agent.advance_phase("plan");
        agent.advance_phase("execute");
        agent.advance_phase("review");
        assert!(agent.is_complete());
        assert_eq!(agent.phase_results.len(), 4);
    }
}
