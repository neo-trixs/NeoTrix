use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Define,
    Construct,
    Evolve,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Define => write!(f, "Define"),
            Stage::Construct => write!(f, "Construct"),
            Stage::Evolve => write!(f, "Evolve"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StageStatus {
    Inactive,
    Active,
    GateCheck,
    Passed,
    Failed(String),
}

impl fmt::Display for StageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StageStatus::Inactive => write!(f, "Inactive"),
            StageStatus::Active => write!(f, "Active"),
            StageStatus::GateCheck => write!(f, "GateCheck"),
            StageStatus::Passed => write!(f, "Passed"),
            StageStatus::Failed(reason) => write!(f, "Failed({})", reason),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StageGate {
    pub stage: Stage,
    pub checks: Vec<GateCheck>,
    pub passed: bool,
    pub failed_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GateCheck {
    pub name: String,
    pub description: String,
    pub check_type: CheckType,
    pub passed: bool,
}

#[derive(Debug, Clone)]
pub enum CheckType {
    ModulePresence { module_name: String },
    MetricThreshold { metric: String, min: f64, max: f64 },
    CircuitClosure { circuit_id: String },
    TestCoverage { min_pct: f64 },
    WiringIntegrity { expected_modules: usize },
}

impl fmt::Display for CheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckType::ModulePresence { module_name } => {
                write!(f, "ModulePresence({})", module_name)
            }
            CheckType::MetricThreshold { metric, min, max } => {
                write!(f, "MetricThreshold({} in [{}, {}])", metric, min, max)
            }
            CheckType::CircuitClosure { circuit_id } => write!(f, "CircuitClosure({})", circuit_id),
            CheckType::TestCoverage { min_pct } => write!(f, "TestCoverage({}%)", min_pct),
            CheckType::WiringIntegrity { expected_modules } => {
                write!(f, "WiringIntegrity({} modules)", expected_modules)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StageTransition {
    pub from: Stage,
    pub to: Stage,
    pub timestamp: Instant,
    pub gates_passed: usize,
    pub gates_total: usize,
}

#[derive(Debug, Clone)]
pub struct OuroborosConfig {
    pub auto_transition: bool,
    pub min_duration_secs: f64,
    pub require_all_gates: bool,
    pub max_transition_history: usize,
}

impl Default for OuroborosConfig {
    fn default() -> Self {
        Self {
            auto_transition: false,
            min_duration_secs: 60.0,
            require_all_gates: true,
            max_transition_history: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OuroborosStageManager {
    pub current_stage: Stage,
    pub stage_status: HashMap<Stage, StageStatus>,
    pub gates: HashMap<Stage, Vec<StageGate>>,
    pub transition_history: Vec<StageTransition>,
    pub config: OuroborosConfig,
    pub last_transition: Option<Instant>,
}

impl OuroborosStageManager {
    pub fn new(config: OuroborosConfig) -> Self {
        let mut stage_status = HashMap::new();
        stage_status.insert(Stage::Define, StageStatus::Active);
        stage_status.insert(Stage::Construct, StageStatus::Inactive);
        stage_status.insert(Stage::Evolve, StageStatus::Inactive);

        let mut gates = HashMap::new();

        let default_define = vec![StageGate {
            stage: Stage::Define,
            checks: vec![
                GateCheck {
                    name: "Identity Documented".into(),
                    description: "Self-identity specification is complete".into(),
                    check_type: CheckType::ModulePresence {
                        module_name: "soul_identity".into(),
                    },
                    passed: false,
                },
                GateCheck {
                    name: "Purpose Clarified".into(),
                    description: "Purpose and mission statement exists".into(),
                    check_type: CheckType::ModulePresence {
                        module_name: "self_understanding".into(),
                    },
                    passed: false,
                },
                GateCheck {
                    name: "Boundaries Defined".into(),
                    description: "Self-world boundary is specified".into(),
                    check_type: CheckType::ModulePresence {
                        module_name: "safety_gate".into(),
                    },
                    passed: false,
                },
            ],
            passed: false,
            failed_reason: None,
        }];

        let default_construct = vec![StageGate {
            stage: Stage::Construct,
            checks: vec![
                GateCheck {
                    name: "Core Modules Wired".into(),
                    description: "All core subsystems are wired".into(),
                    check_type: CheckType::WiringIntegrity {
                        expected_modules: 8,
                    },
                    passed: false,
                },
                GateCheck {
                    name: "ConsciousnessCycle Registered".into(),
                    description: "ConsciousnessCycle has all steps active".into(),
                    check_type: CheckType::CircuitClosure {
                        circuit_id: "consciousness_cycle".into(),
                    },
                    passed: false,
                },
                GateCheck {
                    name: "Pipeline Connected".into(),
                    description: "SelfEvolutionPipeline has all phases connected".into(),
                    check_type: CheckType::WiringIntegrity {
                        expected_modules: 4,
                    },
                    passed: false,
                },
            ],
            passed: false,
            failed_reason: None,
        }];

        let default_evolve = vec![StageGate {
            stage: Stage::Evolve,
            checks: vec![
                GateCheck {
                    name: "ECE < 0.15".into(),
                    description: "Expected Calibration Error below threshold".into(),
                    check_type: CheckType::MetricThreshold {
                        metric: "ece".into(),
                        min: 0.0,
                        max: 0.15,
                    },
                    passed: false,
                },
                GateCheck {
                    name: "MetaAccuracy > 0.7".into(),
                    description: "Meta-cognitive accuracy above threshold".into(),
                    check_type: CheckType::MetricThreshold {
                        metric: "meta_accuracy".into(),
                        min: 0.7,
                        max: 1.0,
                    },
                    passed: false,
                },
                GateCheck {
                    name: "CompositeLoss < 0.4".into(),
                    description: "Composite loss below threshold".into(),
                    check_type: CheckType::MetricThreshold {
                        metric: "composite_loss".into(),
                        min: 0.0,
                        max: 0.4,
                    },
                    passed: false,
                },
            ],
            passed: false,
            failed_reason: None,
        }];

        gates.insert(Stage::Define, default_define);
        gates.insert(Stage::Construct, default_construct);
        gates.insert(Stage::Evolve, default_evolve);

        Self {
            current_stage: Stage::Define,
            stage_status,
            gates,
            transition_history: Vec::new(),
            config,
            last_transition: None,
        }
    }

    pub fn register_gate(&mut self, stage: Stage, gate: StageGate) {
        self.gates.entry(stage).or_default().push(gate);
    }

    pub fn check_gates(&self, stage: Stage) -> (usize, usize, Vec<String>) {
        let failures = self
            .gates
            .get(&stage)
            .map(|gates| {
                gates
                    .iter()
                    .flat_map(|g| &g.checks)
                    .filter(|c| !c.passed)
                    .map(|c| format!("{}: {} ({})", c.name, c.description, c.check_type))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let total = self
            .gates
            .get(&stage)
            .map(|g| g.iter().flat_map(|g| &g.checks).count())
            .unwrap_or(0);
        let passed = total - failures.len();
        (passed, total, failures)
    }

    pub fn can_transition(&self, _target: Stage) -> bool {
        let (passed, total, _) = self.check_gates(self.current_stage);
        if self.config.require_all_gates {
            passed == total && total > 0
        } else {
            passed > 0
        }
    }

    pub fn transition(&mut self, target: Stage) -> Result<StageTransition, String> {
        if self.current_stage == target {
            return Err(format!("Already at stage {}", target));
        }

        let (passed, total, failures) = self.check_gates(self.current_stage);
        let can_pass = if self.config.require_all_gates {
            passed == total && total > 0
        } else {
            passed > 0
        };

        if !can_pass {
            let msg = format!(
                "Cannot transition from {}: gates {}/{} passed. Failures: {:?}",
                self.current_stage, passed, total, failures
            );
            return Err(msg);
        }

        let from = self.current_stage;
        let transition = StageTransition {
            from,
            to: target,
            timestamp: Instant::now(),
            gates_passed: passed,
            gates_total: total,
        };

        self.stage_status.insert(from, StageStatus::Passed);
        self.stage_status.insert(target, StageStatus::Active);
        self.current_stage = target;
        self.last_transition = Some(transition.timestamp);

        self.transition_history.push(transition.clone());
        if self.transition_history.len() > self.config.max_transition_history {
            self.transition_history.remove(0);
        }

        Ok(transition)
    }

    pub fn auto_transition(&mut self) -> Option<StageTransition> {
        if !self.config.auto_transition {
            return None;
        }

        if let Some(last) = self.last_transition {
            let elapsed = last.elapsed().as_secs_f64();
            if elapsed < self.config.min_duration_secs {
                return None;
            }
        }

        let target = match self.current_stage {
            Stage::Define => Stage::Construct,
            Stage::Construct => Stage::Evolve,
            Stage::Evolve => return None,
        };

        match self.transition(target) {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }

    pub fn stage_summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Current Stage: {}", self.current_stage));
        for stage in &[Stage::Define, Stage::Construct, Stage::Evolve] {
            let status = self
                .stage_status
                .get(stage)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".into());
            let (passed, total, _) = self.check_gates(*stage);
            lines.push(format!(
                "  {}: {} (gates {}/{})",
                stage, status, passed, total
            ));
        }
        lines.push(format!("Transitions: {}", self.transition_history.len()));
        lines.join("\n")
    }

    pub fn define_requirements(&mut self, requirements: Vec<GateCheck>) {
        self.stage_status.insert(Stage::Define, StageStatus::Active);
        self.gates.insert(
            Stage::Define,
            vec![StageGate {
                stage: Stage::Define,
                checks: requirements,
                passed: false,
                failed_reason: None,
            }],
        );
    }

    pub fn construct_modules(&mut self, module_names: &[&str]) {
        let checks: Vec<GateCheck> = module_names
            .iter()
            .enumerate()
            .map(|(i, name)| GateCheck {
                name: format!("Module {}: {}", i + 1, name),
                description: format!("Module {} is wired and registered", name),
                check_type: CheckType::ModulePresence {
                    module_name: name.to_string(),
                },
                passed: false,
            })
            .collect();

        self.gates.insert(
            Stage::Construct,
            vec![StageGate {
                stage: Stage::Construct,
                checks,
                passed: false,
                failed_reason: None,
            }],
        );
    }

    pub fn evolve_metrics(&mut self, metrics: &[(&str, f64)]) {
        let checks: Vec<GateCheck> = metrics
            .iter()
            .map(|(name, value)| GateCheck {
                name: format!("{} >= {}", name, value),
                description: format!("{} exceeds minimum threshold", name),
                check_type: CheckType::MetricThreshold {
                    metric: name.to_string(),
                    min: *value,
                    max: f64::MAX,
                },
                passed: false,
            })
            .collect();

        self.gates.insert(
            Stage::Evolve,
            vec![StageGate {
                stage: Stage::Evolve,
                checks,
                passed: false,
                failed_reason: None,
            }],
        );
    }

    pub fn reset(&mut self, stage: Stage) {
        self.stage_status.insert(stage, StageStatus::Active);
        if let Some(gates) = self.gates.get_mut(&stage) {
            for gate in gates.iter_mut() {
                gate.passed = false;
                gate.failed_reason = None;
                for check in gate.checks.iter_mut() {
                    check.passed = false;
                }
            }
        }
    }
}

/// Self-referential: output(t) → qualia(t+1) → input(t+2).
/// Feeds previous cycle output + qualia back as context for the next cycle.
#[derive(Debug, Clone)]
pub struct OuroborosLoop {
    pub last_qualia: Option<super::super::nt_core_consciousness::qualia_generator::Qualia5>,
    pub last_output: Option<String>,
    pub history: VecDeque<(super::super::nt_core_consciousness::qualia_generator::Qualia5, f64)>,
    pub enabled: bool,
}

impl OuroborosLoop {
    pub fn new() -> Self {
        Self {
            last_qualia: None,
            last_output: None,
            history: VecDeque::with_capacity(100),
            enabled: true,
        }
    }

    /// Store the output-qualia pair for the next cycle's self-reference.
    pub fn feed_output(
        &mut self,
        output: &str,
        current_qualia: &super::super::nt_core_consciousness::qualia_generator::Qualia5,
    ) {
        self.last_output = Some(output.to_string());
        self.last_qualia = Some(current_qualia.clone());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        self.history.push_back((current_qualia.clone(), timestamp));
        self.prune(100);
    }

    /// Generate a contextual hint for the next cycle based on previous output qualia.
    pub fn synthesize_input(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        self.last_qualia.as_ref().map(|q| {
            format!(
                "[ouroboros] Previous output had valence={:.2}, arousal={:.2}, interoception={:.2}. Adjust accordingly.",
                q.valence, q.arousal, q.interoception
            )
        })
    }

    /// Keep only the last `max_len` entries in history.
    pub fn prune(&mut self, max_len: usize) {
        while self.history.len() > max_len {
            self.history.pop_front();
        }
    }
}

impl Default for OuroborosLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starts_at_define() {
        let manager = OuroborosStageManager::new(OuroborosConfig::default());
        assert!(matches!(manager.current_stage, Stage::Define));
        let status = manager.stage_status.get(&Stage::Define);
        assert!(matches!(status, Some(StageStatus::Active)));
    }

    #[test]
    fn test_register_and_check_gates() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        let gate = StageGate {
            stage: Stage::Define,
            checks: vec![GateCheck {
                name: "Custom Check".into(),
                description: "A custom gate check".into(),
                check_type: CheckType::ModulePresence {
                    module_name: "test_mod".into(),
                },
                passed: true,
            }],
            passed: false,
            failed_reason: None,
        };
        manager.register_gate(Stage::Define, gate);
        let (passed, total, failures) = manager.check_gates(Stage::Define);
        assert!(passed >= 1);
        assert!(total >= 4);
        assert!(failures.iter().any(|f| f.contains("Identity Documented")));
    }

    #[test]
    fn test_transition_success() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        if let Some(gates) = manager.gates.get_mut(&Stage::Define) {
            for gate in gates.iter_mut() {
                gate.passed = true;
                for check in gate.checks.iter_mut() {
                    check.passed = true;
                }
            }
        }
        let transition = manager.transition(Stage::Construct);
        assert!(transition.is_ok());
        let t = transition.unwrap();
        assert!(matches!(t.from, Stage::Define));
        assert!(matches!(t.to, Stage::Construct));
        assert!(matches!(manager.current_stage, Stage::Construct));
    }

    #[test]
    fn test_transition_fails_if_gates_not_met() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        let result = manager.transition(Stage::Construct);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot transition from Define"));
    }

    #[test]
    fn test_can_transition() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        assert!(!manager.can_transition(Stage::Construct));
        if let Some(gates) = manager.gates.get_mut(&Stage::Define) {
            for gate in gates.iter_mut() {
                gate.passed = true;
                for check in gate.checks.iter_mut() {
                    check.passed = true;
                }
            }
        }
        assert!(manager.can_transition(Stage::Construct));
    }

    #[test]
    fn test_auto_transition_triggers() {
        let mut config = OuroborosConfig::default();
        config.auto_transition = true;
        config.min_duration_secs = 0.0;
        let mut manager = OuroborosStageManager::new(config);
        if let Some(gates) = manager.gates.get_mut(&Stage::Define) {
            for gate in gates.iter_mut() {
                gate.passed = true;
                for check in gate.checks.iter_mut() {
                    check.passed = true;
                }
            }
        }
        manager.last_transition = Some(Instant::now());
        let result = manager.auto_transition();
        assert!(result.is_some());
        let t = result.unwrap();
        assert!(matches!(t.from, Stage::Define));
        assert!(matches!(t.to, Stage::Construct));
    }

    #[test]
    fn test_auto_transition_waits_min_duration() {
        let mut config = OuroborosConfig::default();
        config.auto_transition = true;
        config.min_duration_secs = 3600.0;
        let mut manager = OuroborosStageManager::new(config);
        if let Some(gates) = manager.gates.get_mut(&Stage::Define) {
            for gate in gates.iter_mut() {
                gate.passed = true;
                for check in gate.checks.iter_mut() {
                    check.passed = true;
                }
            }
        }
        manager.last_transition = Some(Instant::now());
        let result = manager.auto_transition();
        assert!(result.is_none());
    }

    #[test]
    fn test_auto_transition_returns_none_at_evolve() {
        let mut config = OuroborosConfig::default();
        config.auto_transition = true;
        config.min_duration_secs = 0.0;
        let mut manager = OuroborosStageManager::new(config);
        manager.current_stage = Stage::Evolve;
        let result = manager.auto_transition();
        assert!(result.is_none());
    }

    #[test]
    fn test_stage_summary_contains_current() {
        let manager = OuroborosStageManager::new(OuroborosConfig::default());
        let summary = manager.stage_summary();
        assert!(summary.contains("Define"));
        assert!(summary.contains("Active"));
        assert!(summary.contains("gates"));
    }

    #[test]
    fn test_define_requirements() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        let checks = vec![GateCheck {
            name: "Identity".into(),
            description: "Has identity".into(),
            check_type: CheckType::ModulePresence {
                module_name: "identity".into(),
            },
            passed: false,
        }];
        manager.define_requirements(checks);
        let (passed, total, _) = manager.check_gates(Stage::Define);
        assert_eq!(total, 1);
        assert_eq!(passed, 0);
    }

    #[test]
    fn test_construct_modules() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        manager.construct_modules(&["mcts", "prm", "pruner"]);
        let (_, total, _) = manager.check_gates(Stage::Construct);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_evolve_metrics() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        manager.evolve_metrics(&[("accuracy", 0.8), ("f1", 0.75)]);
        let (_, total, _) = manager.check_gates(Stage::Evolve);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_reset() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        if let Some(gates) = manager.gates.get_mut(&Stage::Define) {
            for gate in gates.iter_mut() {
                gate.passed = true;
                for check in gate.checks.iter_mut() {
                    check.passed = true;
                }
            }
        }
        manager.reset(Stage::Define);
        let (passed, total, _) = manager.check_gates(Stage::Define);
        assert_eq!(passed, 0);
        assert!(total > 0);
    }

    #[test]
    fn test_transition_to_same_stage_fails() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        let err = manager.transition(Stage::Define).unwrap_err();
        assert!(err.contains("Already at stage"));
    }

    #[test]
    fn test_transition_history_capped() {
        let mut config = OuroborosConfig::default();
        config.max_transition_history = 2;
        let mut manager = OuroborosStageManager::new(config);
        if let Some(gates) = manager.gates.get_mut(&Stage::Define) {
            for gate in gates.iter_mut() {
                gate.passed = true;
                for check in gate.checks.iter_mut() {
                    check.passed = true;
                }
            }
        }
        assert!(manager.transition(Stage::Construct).is_ok());

        if let Some(gates) = manager.gates.get_mut(&Stage::Construct) {
            for gate in gates.iter_mut() {
                gate.passed = true;
                for check in gate.checks.iter_mut() {
                    check.passed = true;
                }
            }
        }
        assert!(manager.transition(Stage::Evolve).is_ok());

        assert_eq!(manager.transition_history.len(), 2);
    }

    #[test]
    fn test_stage_display() {
        assert_eq!(format!("{}", Stage::Define), "Define");
        assert_eq!(format!("{}", Stage::Construct), "Construct");
        assert_eq!(format!("{}", Stage::Evolve), "Evolve");
    }

    #[test]
    fn test_check_type_display() {
        let ct = CheckType::ModulePresence {
            module_name: "test".into(),
        };
        assert_eq!(format!("{}", ct), "ModulePresence(test)");
    }

    #[test]
    fn test_default_config() {
        let config = OuroborosConfig::default();
        assert!(!config.auto_transition);
        assert_eq!(config.min_duration_secs, 60.0);
        assert!(config.require_all_gates);
        assert_eq!(config.max_transition_history, 100);
    }

    #[test]
    fn test_check_gates_empty_stage() {
        let manager = OuroborosStageManager::new(OuroborosConfig::default());
        let (passed, total, failures) = manager.check_gates(Stage::Define);
        assert!(total > 0);
        assert!(passed < total);
        assert!(!failures.is_empty());
    }

    #[test]
    fn test_auto_transition_returns_none_when_disabled() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        manager.config.auto_transition = false;
        assert!(manager.auto_transition().is_none());
    }

    #[test]
    fn test_reset_preserves_stage() {
        let mut manager = OuroborosStageManager::new(OuroborosConfig::default());
        manager.reset(Stage::Define);
        assert!(matches!(manager.current_stage, Stage::Define));
    }

    // ── OuroborosLoop tests ──

    #[test]
    fn test_ouroboros_new() {
        let ol = OuroborosLoop::new();
        assert!(ol.last_qualia.is_none());
        assert!(ol.last_output.is_none());
        assert!(ol.enabled);
    }

    #[test]
    fn test_ouroboros_feed_and_synthesize() {
        use crate::core::nt_core_consciousness::qualia_generator::Qualia5;
        let mut ol = OuroborosLoop::new();
        let q = Qualia5 {
            interoception: 0.4,
            exteroception: 0.6,
            temporal_binding: 0.5,
            valence: 0.2,
            arousal: 0.7,
        };
        ol.feed_output("test_output", &q);
        assert!(ol.last_output.is_some());
        assert!(ol.last_qualia.is_some());
        let hint = ol.synthesize_input();
        assert!(hint.is_some());
        let hint_str = hint.unwrap();
        assert!(hint_str.contains("valence=0.20"));
        assert!(hint_str.contains("arousal=0.70"));
    }

    #[test]
    fn test_ouroboros_disabled() {
        use crate::core::nt_core_consciousness::qualia_generator::Qualia5;
        let mut ol = OuroborosLoop::new();
        ol.enabled = false;
        let q = Qualia5::default();
        ol.feed_output("test", &q);
        assert!(ol.synthesize_input().is_none());
    }

    #[test]
    fn test_ouroboros_prune() {
        use crate::core::nt_core_consciousness::qualia_generator::Qualia5;
        let mut ol = OuroborosLoop::new();
        let q = Qualia5::default();
        for i in 0..50 {
            ol.feed_output(&format!("out_{}", i), &q);
        }
        assert_eq!(ol.history.len(), 50);
        ol.prune(10);
        assert_eq!(ol.history.len(), 10);
    }

    #[test]
    fn test_ouroboros_synthesize_no_history() {
        let ol = OuroborosLoop::new();
        assert!(ol.synthesize_input().is_none());
    }

    #[test]
    fn test_ouroboros_default() {
        let ol = OuroborosLoop::default();
        assert!(ol.enabled);
    }
}
