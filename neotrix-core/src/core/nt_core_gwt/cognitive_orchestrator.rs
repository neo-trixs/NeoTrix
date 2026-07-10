#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use super::cognitive_tick::{
    CognitiveEngine, CognitiveTickConfig, CognitiveTickReport, ContentItem,
    CognitiveAgent, AgentType, create_default_agents,
};
use super::module_def::{
    OrchestratorAgent, OrchestratorPhase, SpecialistModule, SpecialistType,
};
use super::workspace::GlobalWorkspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveOrchestratorReport {
    pub tick_id: u64,
    pub phase: OrchestratorPhase,
    pub agents_activated: usize,
    pub specialists_activated: usize,
    pub broadcast_content: String,
    pub entropy_level: f64,
    pub convergence: f64,
}

pub struct CognitiveOrchestrator {
    pub cognitive_engine: CognitiveEngine,
    pub orchestrator: OrchestratorAgent,
    pub specialists: Vec<SpecialistModule>,
    pub specialist_map: HashMap<AgentType, Vec<SpecialistType>>,
    pub workspace: GlobalWorkspace,
    tick_count: u64,
}

impl CognitiveOrchestrator {
    pub fn new(
        name: &str,
        task_id: &str,
        spec: &str,
        config: CognitiveTickConfig,
    ) -> Self {
        let cognitive_engine = CognitiveEngine::new(config);
        let orchestrator = OrchestratorAgent::new(name, task_id, spec);
        let workspace = GlobalWorkspace::new(0.5);
        let specialist_map = Self::build_default_specialist_map();
        let specialists = Self::create_default_specialists();
        Self { cognitive_engine, orchestrator, specialists, specialist_map, workspace, tick_count: 0 }
    }

    fn build_default_specialist_map() -> HashMap<AgentType, Vec<SpecialistType>> {
        let mut map = HashMap::new();
        map.insert(AgentType::Perceptual, vec![SpecialistType::PatternMatcher, SpecialistType::AnomalyDetector]);
        map.insert(AgentType::Motor, vec![SpecialistType::CodeAnalyzer, SpecialistType::Planner]);
        map.insert(AgentType::Associative, vec![SpecialistType::KnowledgeRetriever, SpecialistType::KnowledgeIntegrator]);
        map.insert(AgentType::Reflective, vec![SpecialistType::ReflectionEngine, SpecialistType::MetaCognitionAnalyst]);
        map.insert(AgentType::Attentional, vec![SpecialistType::GoalPrioritizer, SpecialistType::RiskAssessor]);
        map
    }

    fn create_default_specialists() -> Vec<SpecialistModule> {
        vec![
            SpecialistModule::new(SpecialistType::Planner, "planner".into()),
            SpecialistModule::new(SpecialistType::CodeAnalyzer, "code-analyzer".into()),
            SpecialistModule::new(SpecialistType::PatternMatcher, "pattern-matcher".into()),
            SpecialistModule::new(SpecialistType::KnowledgeRetriever, "knowledge-retriever".into()),
            SpecialistModule::new(SpecialistType::ReflectionEngine, "reflection-engine".into()),
            SpecialistModule::new(SpecialistType::MetaCognitionAnalyst, "meta-analyst".into()),
            SpecialistModule::new(SpecialistType::GoalPrioritizer, "goal-prioritizer".into()),
            SpecialistModule::new(SpecialistType::RiskAssessor, "risk-assessor".into()),
        ]
    }

    pub fn tick(&mut self) -> CognitiveOrchestratorReport {
        self.tick_count += 1;
        let tick_report = self.cognitive_engine.tick();
        let winning_content = tick_report.broadcast_content.clone();
        self.orchestrator.advance_phase(&winning_content);
        for item in &self.cognitive_engine.workspace {
            let agent_type = self.cognitive_engine.agents.iter()
                .find(|h| h.agent.id == item.source_agent)
                .map(|h| h.agent.agent_type);
            if let Some(atype) = agent_type {
                if let Some(specialist_types) = self.specialist_map.get(&atype) {
                    for st in specialist_types {
                        for specialist in self.specialists.iter_mut() {
                            if specialist.specialist_type == *st {
                                specialist.activate(item.salience * 0.1);
                            }
                        }
                    }
                }
            }
        }
        if let Some(ref content_item) = self.cognitive_engine.workspace.iter()
            .max_by(|a, b| a.salience.partial_cmp(&b.salience).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
        {
            self.workspace.broadcast(&content_item.content);
        }
        let specialists_activated = self.specialists.iter()
            .filter(|s| s.activation > 0.0)
            .count();
        CognitiveOrchestratorReport {
            tick_id: self.tick_count,
            phase: self.orchestrator.phase,
            agents_activated: tick_report.agents_activated,
            specialists_activated,
            broadcast_content: winning_content,
            entropy_level: tick_report.entropy_level,
            convergence: tick_report.convergence,
        }
    }

    pub fn run_task(&mut self, max_ticks: usize) -> Vec<CognitiveOrchestratorReport> {
        let mut reports = Vec::with_capacity(max_ticks);
        for _ in 0..max_ticks {
            if self.orchestrator.is_complete() {
                break;
            }
            let report = self.tick();
            reports.push(report);
        }
        reports
    }

    pub fn assign_specialist(&mut self, st: SpecialistType) {
        self.orchestrator.assign(st);
    }

    pub fn specialist_activation_report(&self) -> HashMap<String, f64> {
        self.specialists.iter().map(|s| (s.name.clone(), s.activation)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orchestrator() {
        let config = CognitiveTickConfig::default();
        let orch = CognitiveOrchestrator::new("test", "task-1", "test task", config);
        assert_eq!(orch.orchestrator.phase, OrchestratorPhase::Backlog);
        assert_eq!(orch.specialists.len(), 8);
        assert_eq!(orch.specialist_map.len(), 5);
    }

    #[test]
    fn test_single_tick() {
        let config = CognitiveTickConfig::default();
        let mut orch = CognitiveOrchestrator::new("test", "task-1", "test", config);
        let report = orch.tick();
        assert!(report.tick_id > 0);
        assert!(report.agents_activated > 0);
        assert!(!report.broadcast_content.is_empty());
    }

    #[test]
    fn test_run_task_completes() {
        let config = CognitiveTickConfig::default();
        let mut orch = CognitiveOrchestrator::new("test", "task-1", "test", config);
        let reports = orch.run_task(20);
        assert!(reports.len() > 0);
        assert!(reports.len() <= 20);
        assert!(orch.orchestrator.is_complete() || reports.len() == 20);
    }

    #[test]
    fn test_specialist_activation() {
        let config = CognitiveTickConfig::default();
        let mut orch = CognitiveOrchestrator::new("test", "task-1", "test", config);
        for _ in 0..3 {
            orch.tick();
        }
        let activation = orch.specialist_activation_report();
        assert!(activation.len() == 8);
        let total: f64 = activation.values().sum();
        assert!(total >= 0.0);
    }

    #[test]
    fn test_assign_specialist() {
        let config = CognitiveTickConfig::default();
        let mut orch = CognitiveOrchestrator::new("test", "task-1", "test", config);
        orch.assign_specialist(SpecialistType::Planner);
        assert_eq!(orch.orchestrator.assigned_specialists.len(), 1);
    }

    #[test]
    fn test_phase_progresses() {
        let config = CognitiveTickConfig::default();
        let mut orch = CognitiveOrchestrator::new("test", "task-1", "test", config);
        let phases: Vec<OrchestratorPhase> = (0..5).map(|_| {
            let report = orch.tick();
            report.phase
        }).collect();
        for i in 1..phases.len() {
            assert!(phases[i] as u8 >= phases[i-1] as u8,
                "phases should monotonically advance");
        }
    }

    #[test]
    fn test_report_tick_id_monotonic() {
        let config = CognitiveTickConfig::default();
        let mut orch = CognitiveOrchestrator::new("test", "task-1", "test", config);
        let mut prev_id = 0u64;
        for _ in 0..3 {
            let report = orch.tick();
            assert!(report.tick_id > prev_id);
            prev_id = report.tick_id;
        }
    }

    #[test]
    fn test_multiple_ticks_convergence() {
        let config = CognitiveTickConfig::default();
        let mut orch = CognitiveOrchestrator::new("test", "task-1", "test", config);
        let first = orch.tick();
        let last = orch.tick();
        assert!(last.convergence.is_finite());
    }
}
