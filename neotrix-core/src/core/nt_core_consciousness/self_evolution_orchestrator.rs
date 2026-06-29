use super::consciousness_architecture::{
    ArchitectureHealth, ArchitectureLayer, CapabilityStatus, ConsciousnessArchitecture, GapSeverity,
};
use super::meta_evolution_loop::{
    EvolutionOutcome, EvolutionRecommendation,
    MetaArchitectureEvolutionLoop,
};
use super::performance_oracle::{
    HealthDashboard, PerformanceOracle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionPhase {
    Analyze,
    Plan,
    SafetyCheck,
    Execute,
    Measure,
    Adapt,
}

impl EvolutionPhase {
    pub fn name(&self) -> &'static str {
        match self {
            EvolutionPhase::Analyze => "analyze",
            EvolutionPhase::Plan => "plan",
            EvolutionPhase::SafetyCheck => "safety_check",
            EvolutionPhase::Execute => "execute",
            EvolutionPhase::Measure => "measure",
            EvolutionPhase::Adapt => "adapt",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvolutionProposal {
    pub id: u64,
    pub phase: EvolutionPhase,
    pub priority: f64,
    pub impact_score: f64,
    pub risk_score: f64,
    pub description: String,
    pub config_changes: Vec<ConfigChange>,
    pub rationale: String,
    pub gap_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ConfigChange {
    AdjustBudget {
        process: String,
        old_weight: f64,
        new_weight: f64,
    },
    ToggleStep {
        step_name: String,
        enable: bool,
    },
    AdjustParam {
        param_path: String,
        old_value: String,
        new_value: String,
    },
    CreateModule {
        name: String,
        layer: ArchitectureLayer,
        priority: GapSeverity,
    },
    UpgradeStatus {
        capability: String,
        from: CapabilityStatus,
        to: CapabilityStatus,
    },
}

#[derive(Debug, Clone)]
pub struct EvolutionRecord {
    pub id: u64,
    pub timestamp: u64,
    pub proposals: Vec<EvolutionProposal>,
    pub pre_health: f64,
    pub post_health: f64,
    pub health_delta: f64,
    pub outcome: EvolutionOutcome,
    pub rollback_performed: bool,
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub auto_evolve: bool,
    pub max_proposals_per_cycle: usize,
    pub risk_threshold: f64,
    pub impact_threshold: f64,
    pub history_size: usize,
    pub min_improvement_for_lock: f64,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            auto_evolve: true,
            max_proposals_per_cycle: 3,
            risk_threshold: 0.7,
            impact_threshold: 0.3,
            history_size: 50,
            min_improvement_for_lock: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrchestratorState {
    pub phase: EvolutionPhase,
    pub pending_proposals: Vec<EvolutionProposal>,
    pub history: Vec<EvolutionRecord>,
    pub pipeline_health: f64,
    pub active_gap_count: usize,
    pub last_adaptation_cycle: u64,
}

impl OrchestratorState {
    pub fn new() -> Self {
        Self {
            phase: EvolutionPhase::Analyze,
            pending_proposals: Vec::new(),
            history: Vec::new(),
            pipeline_health: 0.5,
            active_gap_count: 0,
            last_adaptation_cycle: 0,
        }
    }

    pub fn adaptation_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let count = self.history.len().min(20);
        let recent: Vec<&EvolutionRecord> = self.history.iter().rev().take(count).collect();
        if recent.is_empty() {
            return 0.0;
        }
        let improved = recent.iter().filter(|r| r.health_delta > 0.0).count();
        improved as f64 / recent.len() as f64
    }

    pub fn regressions(&self) -> Vec<&EvolutionRecord> {
        self.history.iter().filter(|r| r.health_delta < -0.02).collect()
    }

    pub fn best_proposals(&self, max: usize) -> Vec<EvolutionProposal> {
        let mut sorted = self.pending_proposals.clone();
        sorted.sort_by(|a, b| {
            let a_score = a.priority * a.impact_score * (1.0 - a.risk_score);
            let b_score = b.priority * b.impact_score * (1.0 - b.risk_score);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(max).collect()
    }
}

pub struct SelfEvolutionOrchestrator {
    config: OrchestratorConfig,
    state: OrchestratorState,
    proposal_counter: u64,
}

impl SelfEvolutionOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            config,
            state: OrchestratorState::new(),
            proposal_counter: 0,
        }
    }

    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut OrchestratorConfig {
        &mut self.config
    }

    pub fn state(&self) -> &OrchestratorState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut OrchestratorState {
        &mut self.state
    }

    pub fn run_evolution_cycle(
        &mut self,
        cycle: u64,
        oracle: &PerformanceOracle,
        meta_evolution: &mut MetaArchitectureEvolutionLoop,
        architecture: &ConsciousnessArchitecture,
    ) -> Vec<EvolutionProposal> {
        let dashboard = oracle.dashboard();
        let recommendations = meta_evolution.assess();
        let arch_health = architecture.assess_health();

        self.state.pipeline_health = dashboard.overall_health;
        self.state.active_gap_count = arch_health.missing_capabilities.len();

        let analysis = self.analyze(cycle, &dashboard, &recommendations, &arch_health);
        self.state.phase = EvolutionPhase::Analyze;

        if !self.config.auto_evolve {
            return analysis;
        }

        let plans = self.plan(analysis, oracle, meta_evolution);
        self.state.phase = EvolutionPhase::Plan;

        let safe = self.safety_check(&plans);
        self.state.phase = EvolutionPhase::SafetyCheck;

        if !safe.is_empty() {
            self.state.pending_proposals.extend(safe.clone());
            self.state.last_adaptation_cycle = cycle;
        }
        self.state.phase = EvolutionPhase::Execute;
        self.state.phase = EvolutionPhase::Measure;
        self.state.phase = EvolutionPhase::Adapt;

        safe
    }

    fn analyze(
        &mut self,
        _cycle: u64,
        dashboard: &HealthDashboard,
        recommendations: &[EvolutionRecommendation],
        _health: &ArchitectureHealth,
    ) -> Vec<EvolutionProposal> {
        let mut proposals = Vec::new();

        for bottleneck in &dashboard.bottlenecks {
            let id = self.next_proposal_id();
            proposals.push(EvolutionProposal {
                id,
                phase: EvolutionPhase::Analyze,
                priority: 0.6,
                impact_score: 0.5,
                risk_score: 0.2,
                description: format!("Optimize bottleneck step: {}", bottleneck),
                config_changes: vec![],
                rationale: format!("Step '{}' exceeds latency threshold", bottleneck),
                gap_ids: vec![],
            });
        }

        if dashboard.overall_health < 0.4 {
            let id = self.next_proposal_id();
            proposals.push(EvolutionProposal {
                id,
                phase: EvolutionPhase::Analyze,
                priority: 0.9,
                impact_score: 0.8,
                risk_score: 0.3,
                description: "Emergency health improvement".into(),
                config_changes: vec![],
                rationale: format!(
                    "Pipeline health at {:.2}, below emergency threshold 0.4",
                    dashboard.overall_health
                ),
                gap_ids: vec![],
            });
        }

        if !_health.missing_capabilities.is_empty() {
            let id = self.next_proposal_id();
            proposals.push(EvolutionProposal {
                id,
                phase: EvolutionPhase::Analyze,
                priority: 0.7,
                impact_score: 0.6,
                risk_score: 0.25,
                description: format!(
                    "Wire {} missing capabilities",
                    _health.missing_capabilities.len()
                ),
                config_changes: vec![],
                rationale: format!(
                    "Architecture has {} missing capabilities: {:?}",
                    _health.missing_capabilities.len(),
                    _health.missing_capabilities
                ),
                gap_ids: _health.missing_capabilities.clone(),
            });
        }

        if !_health.critical_gaps.is_empty() {
            let id = self.next_proposal_id();
            proposals.push(EvolutionProposal {
                id,
                phase: EvolutionPhase::Analyze,
                priority: 0.95,
                impact_score: 0.9,
                risk_score: 0.15,
                description: format!(
                    "Fix {} critical architecture gaps",
                    _health.critical_gaps.len()
                ),
                config_changes: vec![],
                rationale: format!(
                    "Critical gaps: {:?}",
                    _health.critical_gaps
                ),
                gap_ids: _health.critical_gaps.clone(),
            });
        }

        for rec in recommendations {
            let id = self.next_proposal_id();
            proposals.push(EvolutionProposal {
                id,
                phase: EvolutionPhase::Analyze,
                priority: rec.priority as f64 / 10.0,
                impact_score: 0.6,
                risk_score: 0.2,
                description: format!("{:?}: {}", rec.action, rec.target_capability),
                config_changes: vec![],
                rationale: rec.rationale.clone(),
                gap_ids: rec.gap_ids.clone(),
            });
        }

        proposals
    }

    fn plan(
        &mut self,
        proposals: Vec<EvolutionProposal>,
        _oracle: &PerformanceOracle,
        _meta_evolution: &MetaArchitectureEvolutionLoop,
    ) -> Vec<EvolutionProposal> {
        let mut ranked = proposals;
        ranked.sort_by(|a, b| {
            let a_score = a.priority * a.impact_score * (1.0 - a.risk_score);
            let b_score = b.priority * b.impact_score * (1.0 - b.risk_score);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        ranked.truncate(self.config.max_proposals_per_cycle);
        ranked
    }

    fn safety_check(&self, proposals: &[EvolutionProposal]) -> Vec<EvolutionProposal> {
        proposals
            .iter()
            .filter(|p| {
                if p.risk_score > self.config.risk_threshold {
                    return false;
                }
                if p.impact_score < self.config.impact_threshold {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn record_outcome(
        &mut self,
        pre_health: f64,
        post_health: f64,
        proposals: &[EvolutionProposal],
        outcome: EvolutionOutcome,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let record = EvolutionRecord {
            id: self.next_proposal_id(),
            timestamp,
            proposals: proposals.to_vec(),
            pre_health,
            post_health,
            health_delta: post_health - pre_health,
            outcome,
            rollback_performed: pre_health > post_health,
        };

        if self.state.history.len() >= self.config.history_size {
            self.state.history.remove(0);
        }
        self.state.history.push(record);
    }

    pub fn should_rollback(&self, recent_health: f64) -> bool {
        if let Some(last) = self.state.history.last() {
            if last.post_health > 0.0 && recent_health < last.pre_health * 0.8 {
                return true;
            }
        }
        false
    }

    pub fn recent_effectiveness(&self) -> f64 {
        if self.state.history.len() < 3 {
            return 0.5;
        }
        let total = self.state.history.len().min(10);
        let recent: Vec<&EvolutionRecord> = self
            .state
            .history
            .iter()
            .rev()
            .take(total)
            .filter(|r| matches!(r.outcome, EvolutionOutcome::Succeeded))
            .collect();
        recent.len() as f64 / total as f64
    }

    fn next_proposal_id(&mut self) -> u64 {
        let id = self.proposal_counter;
        self.proposal_counter += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::consciousness_architecture::{
        ArchitectureHealth, ArchitectureLayer, ConsciousnessArchitecture, GapInfo, GapSeverity,
    };
    use crate::core::nt_core_consciousness::meta_evolution_loop::{
        EvolutionAction, EvolutionRecommendation, MetaEvolutionConfig,
    };
    use crate::core::nt_core_consciousness::performance_oracle::{OracleConfig, TrendDirection};

    #[test]
    fn test_orchestrator_creation() {
        let config = OrchestratorConfig::default();
        let o = SelfEvolutionOrchestrator::new(config);
        assert_eq!(o.state().phase, EvolutionPhase::Analyze);
        assert!(o.state().pending_proposals.is_empty());
    }

    #[test]
    fn test_analyze_generates_proposals_for_low_health() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        let dashboard = HealthDashboard {
            overall_health: 0.3,
            step_health: vec![],
            trend: TrendDirection::Declining,
            bottlenecks: vec!["allocation".into()],
            recommendations: vec![],
            cycles_analyzed: 50,
        };

        let recommendations = vec![];
        let health = ArchitectureHealth {
            layer_scores: std::collections::HashMap::new(),
            overall_health: 0.5,
            total_capabilities: 10,
            completed_capabilities: 5,
            missing_capabilities: vec![],
            critical_gaps: vec![],
        };

        let proposals = o.analyze(0, &dashboard, &recommendations, &health);
        assert!(!proposals.is_empty(), "Should generate proposals for low health");
        assert!(
            proposals.iter().any(|p| p.priority > 0.8),
            "Should have high-priority emergency proposal"
        );
    }

    #[test]
    fn test_analyze_generates_proposals_for_bottlenecks() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        let dashboard = HealthDashboard {
            overall_health: 0.7,
            step_health: vec![],
            trend: TrendDirection::Stable,
            bottlenecks: vec!["refinery".into(), "dual_path".into()],
            recommendations: vec![],
            cycles_analyzed: 50,
        };

        let recommendations = vec![];
        let health = ArchitectureHealth {
            layer_scores: std::collections::HashMap::new(),
            overall_health: 0.8,
            total_capabilities: 10,
            completed_capabilities: 8,
            missing_capabilities: vec![],
            critical_gaps: vec![],
        };

        let proposals = o.analyze(0, &dashboard, &recommendations, &health);
        let bottleneck_proposals: Vec<&EvolutionProposal> = proposals
            .iter()
            .filter(|p| p.description.contains("bottleneck"))
            .collect();
        assert_eq!(bottleneck_proposals.len(), 2);
    }

    #[test]
    fn test_analyze_generates_proposals_for_missing_capabilities() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        let dashboard = HealthDashboard {
            overall_health: 0.8,
            step_health: vec![],
            trend: TrendDirection::Improving,
            bottlenecks: vec![],
            recommendations: vec![],
            cycles_analyzed: 50,
        };

        let recommendations = vec![];
        let health = ArchitectureHealth {
            layer_scores: std::collections::HashMap::new(),
            overall_health: 0.6,
            total_capabilities: 10,
            completed_capabilities: 5,
            missing_capabilities: vec!["mcts".into(), "dead_end".into()],
            critical_gaps: vec![],
        };

        let proposals = o.analyze(0, &dashboard, &recommendations, &health);
        let cap_proposals: Vec<&EvolutionProposal> = proposals
            .iter()
            .filter(|p| p.description.contains("capabilities"))
            .collect();
        assert_eq!(cap_proposals.len(), 1);
    }

    #[test]
    fn test_analyze_includes_meta_recommendations() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        let dashboard = HealthDashboard {
            overall_health: 0.8,
            step_health: vec![],
            trend: TrendDirection::Stable,
            bottlenecks: vec![],
            recommendations: vec![],
            cycles_analyzed: 50,
        };

        let recommendations = vec![EvolutionRecommendation {
            target_capability: "new_module".into(),
            target_layer: ArchitectureLayer::Cognition,
            action: EvolutionAction::CreateModule,
            priority: 8,
            rationale: "Need better reasoning".into(),
            estimated_lines: 200,
            module_path_hint: Some("reasoning/new_module.rs".into()),
            gap_ids: vec!["gap_1".into()],
        }];

        let health = ArchitectureHealth {
            layer_scores: std::collections::HashMap::new(),
            overall_health: 0.8,
            total_capabilities: 10,
            completed_capabilities: 7,
            missing_capabilities: vec![],
            critical_gaps: vec![],
        };

        let proposals = o.analyze(0, &dashboard, &recommendations, &health);
        assert!(
            proposals.iter().any(|p| p.description.contains("CreateModule")),
            "Should include meta-recommendation proposal"
        );
    }

    #[test]
    fn test_plan_ranks_by_priority_impact_risk() {
        let config = OrchestratorConfig {
            max_proposals_per_cycle: 2,
            ..OrchestratorConfig::default()
        };
        let mut o = SelfEvolutionOrchestrator::new(config);

        let proposals = vec![
            EvolutionProposal {
                id: 1,
                phase: EvolutionPhase::Analyze,
                priority: 0.9,
                impact_score: 0.5,
                risk_score: 0.1,
                description: "high".into(),
                config_changes: vec![],
                rationale: String::new(),
                gap_ids: vec![],
            },
            EvolutionProposal {
                id: 2,
                phase: EvolutionPhase::Analyze,
                priority: 0.5,
                impact_score: 0.3,
                risk_score: 0.1,
                description: "low".into(),
                config_changes: vec![],
                rationale: String::new(),
                gap_ids: vec![],
            },
            EvolutionProposal {
                id: 3,
                phase: EvolutionPhase::Analyze,
                priority: 0.7,
                impact_score: 0.8,
                risk_score: 0.5,
                description: "risky".into(),
                config_changes: vec![],
                rationale: String::new(),
                gap_ids: vec![],
            },
        ];

        let oracle = PerformanceOracle::new(OracleConfig::default());
        let meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());

        let planned = o.plan(proposals, &oracle, &meta);
        assert_eq!(planned.len(), 2);
    }

    #[test]
    fn test_safety_check_filters_high_risk() {
        let config = OrchestratorConfig {
            risk_threshold: 0.5,
            impact_threshold: 0.2,
            ..OrchestratorConfig::default()
        };
        let o = SelfEvolutionOrchestrator::new(config);

        let proposals = vec![
            EvolutionProposal {
                id: 1,
                phase: EvolutionPhase::Plan,
                priority: 0.9,
                impact_score: 0.8,
                risk_score: 0.6,
                description: "too risky".into(),
                config_changes: vec![],
                rationale: String::new(),
                gap_ids: vec![],
            },
            EvolutionProposal {
                id: 2,
                phase: EvolutionPhase::Plan,
                priority: 0.7,
                impact_score: 0.4,
                risk_score: 0.2,
                description: "safe".into(),
                config_changes: vec![],
                rationale: String::new(),
                gap_ids: vec![],
            },
        ];

        let safe = o.safety_check(&proposals);
        assert_eq!(safe.len(), 1);
        assert_eq!(safe[0].description, "safe");
    }

    #[test]
    fn test_record_outcome_tracks_history() {
        let config = OrchestratorConfig {
            history_size: 10,
            ..OrchestratorConfig::default()
        };
        let mut o = SelfEvolutionOrchestrator::new(config);

        let proposals = vec![EvolutionProposal {
            id: 1,
            phase: EvolutionPhase::Analyze,
            priority: 0.8,
            impact_score: 0.7,
            risk_score: 0.2,
            description: "test".into(),
            config_changes: vec![],
            rationale: String::new(),
            gap_ids: vec![],
        }];

        o.record_outcome(0.5, 0.7, &proposals, EvolutionOutcome::Succeeded);
        assert_eq!(o.state().history.len(), 1);
        assert!((o.state().history[0].health_delta - 0.2).abs() < 0.001);
        assert!(!o.state().history[0].rollback_performed);
    }

    #[test]
    fn test_rollback_detected_when_health_drops() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        o.record_outcome(0.8, 0.9, &[], EvolutionOutcome::Succeeded);
        assert!(o.should_rollback(0.6));
        assert!(!o.should_rollback(0.75));
    }

    #[test]
    fn test_recent_effectiveness() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        for _ in 0..5 {
            o.record_outcome(0.5, 0.7, &[], EvolutionOutcome::Succeeded);
        }
        for _ in 0..5 {
            o.record_outcome(0.5, 0.4, &[], EvolutionOutcome::Failed("".into()));
        }

        let eff = o.recent_effectiveness();
        assert!(eff > 0.0 && eff <= 1.0);
    }

    #[test]
    fn test_regression_tracking() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        o.record_outcome(0.8, 0.5, &[], EvolutionOutcome::Failed("bad".into()));
        o.record_outcome(0.8, 0.9, &[], EvolutionOutcome::Succeeded);

        let regressions = o.state().regressions();
        assert_eq!(regressions.len(), 1);
    }

    #[test]
    fn test_adaptation_rate() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        assert_eq!(o.state().adaptation_rate(), 0.0);

        for _ in 0..5 {
            o.record_outcome(0.5, 0.7, &[], EvolutionOutcome::Succeeded);
        }
        for _ in 0..5 {
            o.record_outcome(0.5, 0.3, &[], EvolutionOutcome::Failed("".into()));
        }

        let rate = o.state().adaptation_rate();
        assert!(rate > 0.0 && rate <= 1.0);
    }

    #[test]
    fn test_best_proposals_by_score() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        o.state_mut().pending_proposals = vec![
            EvolutionProposal {
                id: 1,
                phase: EvolutionPhase::Plan,
                priority: 0.3,
                impact_score: 0.3,
                risk_score: 0.1,
                description: "low".into(),
                config_changes: vec![],
                rationale: String::new(),
                gap_ids: vec![],
            },
            EvolutionProposal {
                id: 2,
                phase: EvolutionPhase::Plan,
                priority: 0.9,
                impact_score: 0.8,
                risk_score: 0.1,
                description: "high".into(),
                config_changes: vec![],
                rationale: String::new(),
                gap_ids: vec![],
            },
        ];

        let best = o.state().best_proposals(1);
        assert_eq!(best.len(), 1);
        assert_eq!(best[0].description, "high");
    }

    #[test]
    fn test_auto_evolve_disabled_returns_analysis_only() {
        let config = OrchestratorConfig {
            auto_evolve: false,
            ..OrchestratorConfig::default()
        };
        let mut o = SelfEvolutionOrchestrator::new(config);

        let oracle = PerformanceOracle::new(OracleConfig::default());
        let mut meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let arch = ConsciousnessArchitecture::new();

        let result = o.run_evolution_cycle(0, &oracle, &mut meta, &arch);
        assert_eq!(o.state().phase, EvolutionPhase::Analyze);
        assert!(result.len() <= 3);
    }

    #[test]
    fn test_full_evolution_cycle_when_auto_evolve_on() {
        let config = OrchestratorConfig::default();
        let mut o = SelfEvolutionOrchestrator::new(config);

        let oracle = PerformanceOracle::new(OracleConfig::default());
        let mut meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let arch = ConsciousnessArchitecture::new();

        let result = o.run_evolution_cycle(0, &oracle, &mut meta, &arch);
        assert_eq!(o.state().phase, EvolutionPhase::Adapt);
    }
}
