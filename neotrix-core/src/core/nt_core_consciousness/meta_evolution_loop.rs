
use super::consciousness_architecture::{
    ArchitectureHealth, ArchitectureLayer, CapabilityStatus,
    ConsciousnessArchitecture, EvolutionPlan, GapSeverity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionAction {
    CreateModule,
    UpgradeStatus,
    WireDependency,
    RefactorExisting,
    Deprecate,
}

impl EvolutionAction {
    pub fn name(&self) -> &'static str {
        match self {
            EvolutionAction::CreateModule => "create_module",
            EvolutionAction::UpgradeStatus => "upgrade_status",
            EvolutionAction::WireDependency => "wire_dependency",
            EvolutionAction::RefactorExisting => "refactor_existing",
            EvolutionAction::Deprecate => "deprecate",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvolutionRecommendation {
    pub target_capability: String,
    pub target_layer: ArchitectureLayer,
    pub action: EvolutionAction,
    pub priority: u8,
    pub rationale: String,
    pub estimated_lines: usize,
    pub module_path_hint: Option<String>,
    pub gap_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EvolutionAttempt {
    pub id: u64,
    pub timestamp: u64,
    pub recommendation: EvolutionRecommendation,
    pub outcome: EvolutionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvolutionOutcome {
    Pending,
    Succeeded,
    Failed(String),
    PartiallyComplete(String),
}

#[derive(Debug, Clone)]
pub struct MetaEvolutionConfig {
    /// Min health threshold to trigger evolution (0.0-1.0)
    pub trigger_threshold: f64,
    /// Whether to auto-evolve the weakest layer
    pub auto_evolve: bool,
    /// Max recommendations per assessment
    pub max_recommendations: usize,
    /// History retention
    pub max_history: usize,
}

impl Default for MetaEvolutionConfig {
    fn default() -> Self {
        Self {
            trigger_threshold: 0.6,
            auto_evolve: false,
            max_recommendations: 3,
            max_history: 100,
        }
    }
}

pub struct MetaArchitectureEvolutionLoop {
    config: MetaEvolutionConfig,
    arch: ConsciousnessArchitecture,
    history: Vec<EvolutionAttempt>,
    attempt_counter: u64,
    last_health: Option<ArchitectureHealth>,
}

impl MetaArchitectureEvolutionLoop {
    pub fn new(config: MetaEvolutionConfig) -> Self {
        Self {
            config,
            arch: ConsciousnessArchitecture::new(),
            history: Vec::new(),
            attempt_counter: 0,
            last_health: None,
        }
    }

    pub fn config(&self) -> &MetaEvolutionConfig {
        &self.config
    }
    pub fn arch(&self) -> &ConsciousnessArchitecture {
        &self.arch
    }
    pub fn history(&self) -> &[EvolutionAttempt] {
        &self.history
    }

    /// Full assessment cycle: compute health → identify weakest layer → generate recommendations.
    pub fn assess(&mut self) -> Vec<EvolutionRecommendation> {
        let health = self.arch.assess_health();
        self.last_health = Some(health.clone());

        let weakest_layer = self.find_weakest_layer(&health);
        let mut recommendations = Vec::new();

        // Find missing capabilities in the weakest layer
        let missing_caps = self.arch.capabilities_by_layer(weakest_layer);
        for cap in &missing_caps {
            if recommendations.len() >= self.config.max_recommendations {
                break;
            }
            if cap.status == CapabilityStatus::Missing || cap.status == CapabilityStatus::Partial {
                let action = if cap.module_path.is_none() {
                    EvolutionAction::CreateModule
                } else if cap.status == CapabilityStatus::Partial {
                    EvolutionAction::UpgradeStatus
                } else {
                    EvolutionAction::WireDependency
                };

                let health_score = health
                    .layer_scores
                    .get(&weakest_layer)
                    .copied()
                    .unwrap_or(0.0);
                let priority = if health_score < 0.3 {
                    0
                } else if health_score < 0.6 {
                    1
                } else {
                    2
                };

                recommendations.push(EvolutionRecommendation {
                    target_capability: cap.id.clone(),
                    target_layer: weakest_layer,
                    action,
                    priority,
                    rationale: format!(
                        "Layer '{:?}' health={:.2}: {} is {:?}",
                        weakest_layer, health_score, cap.name, cap.status
                    ),
                    estimated_lines: cap.estimated_lines,
                    module_path_hint: cap.module_path.clone(),
                    gap_ids: cap.gap_ids.clone(),
                });
            }
        }

        // Also check critical gaps (survival level)
        for gap in self.arch.gaps_by_layer(weakest_layer) {
            if gap.severity == GapSeverity::Survival {
                let already = recommendations.iter().any(|r| r.gap_ids.contains(&gap.id));
                if !already && recommendations.len() < self.config.max_recommendations {
                    recommendations.push(EvolutionRecommendation {
                        target_capability: gap.target_capability.clone(),
                        target_layer: weakest_layer,
                        action: EvolutionAction::CreateModule,
                        priority: 0,
                        rationale: format!("Survival gap {}: {}", gap.id, gap.description),
                        estimated_lines: 500,
                        module_path_hint: None,
                        gap_ids: vec![gap.id.clone()],
                    });
                }
            }
        }

        recommendations.sort_by_key(|r| r.priority);
        recommendations.truncate(self.config.max_recommendations);
        recommendations
    }

    /// Record an evolution attempt and its outcome.
    pub fn record_attempt(
        &mut self,
        recommendation: EvolutionRecommendation,
        outcome: EvolutionOutcome,
    ) {
        self.attempt_counter += 1;
        let attempt = EvolutionAttempt {
            id: self.attempt_counter,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            recommendation,
            outcome,
        };
        self.history.push(attempt);
        if self.history.len() > self.config.max_history {
            self.history.remove(0);
        }
    }

    /// Get the evolution success rate over the last N attempts.
    pub fn success_rate(&self, recent_n: usize) -> f64 {
        let n = recent_n.min(self.history.len());
        if n == 0 {
            return 0.0;
        }
        let succeeded = self
            .history
            .iter()
            .rev()
            .take(n)
            .filter(|a| matches!(a.outcome, EvolutionOutcome::Succeeded))
            .count();
        succeeded as f64 / n as f64
    }

    /// Assess if architecture has improved since last check.
    pub fn health_trend(&self) -> Trend {
        let current = self.arch.assess_health();
        match &self.last_health {
            Some(prev) => {
                if current.overall_health > prev.overall_health + 0.01 {
                    Trend::Improving
                } else if current.overall_health < prev.overall_health - 0.01 {
                    Trend::Declining
                } else {
                    Trend::Stable
                }
            }
            None => Trend::Stable,
        }
    }

    fn find_weakest_layer(&self, health: &ArchitectureHealth) -> ArchitectureLayer {
        *health
            .layer_scores
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(layer, _)| layer)
            .unwrap_or(&ArchitectureLayer::Cognition)
    }

    /// Generate a full evolution plan (delegates to ConsciousnessArchitecture).
    pub fn generate_plan(&self) -> EvolutionPlan {
        self.arch.generate_evolution_plan()
    }

    /// Summary of current architecture state and recommendations.
    pub fn summary(&mut self) -> String {
        let health = self.arch.assess_health();
        let recs = self.assess();
        format!(
            "health={:.2} | weakest={:?} | recs={} | success_rate={:.2} | trend={:?}",
            health.overall_health,
            self.find_weakest_layer(&health),
            recs.len(),
            self.success_rate(10),
            self.health_trend(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Improving,
    Declining,
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MetaEvolutionConfig::default();
        assert!((config.trigger_threshold - 0.6).abs() < 1e-6);
        assert!(!config.auto_evolve);
    }

    #[test]
    fn test_assess_returns_recommendations() {
        let mut meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let recs = meta.assess();
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_record_attempt() {
        let mut meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let recs = meta.assess();
        if !recs.is_empty() {
            meta.record_attempt(recs[0].clone(), EvolutionOutcome::Succeeded);
            assert_eq!(meta.history.len(), 1);
        }
    }

    #[test]
    fn test_success_rate() {
        let mut meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let recs = meta.assess();
        if recs.len() >= 2 {
            meta.record_attempt(recs[0].clone(), EvolutionOutcome::Succeeded);
            meta.record_attempt(recs[1].clone(), EvolutionOutcome::Failed("test".into()));
            assert!((meta.success_rate(10) - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_empty_history_success_rate() {
        let meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        assert!((meta.success_rate(10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_plan() {
        let meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let plan = meta.generate_plan();
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_summary_format() {
        let mut meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let s = meta.summary();
        assert!(s.contains("health="));
        assert!(s.contains("weakest="));
    }

    #[test]
    fn test_find_weakest_layer() {
        let meta = MetaArchitectureEvolutionLoop::new(MetaEvolutionConfig::default());
        let health = meta.arch.assess_health();
        let weakest = meta.find_weakest_layer(&health);
        assert!(ArchitectureLayer::all().contains(&weakest));
    }
}
