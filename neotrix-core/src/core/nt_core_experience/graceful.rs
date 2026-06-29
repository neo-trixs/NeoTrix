use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum DegradationLevel {
    Full,
    Limited,
    Minimal,
    None,
}

impl DegradationLevel {
    pub fn is_available(&self) -> bool {
        matches!(
            self,
            DegradationLevel::Full | DegradationLevel::Limited | DegradationLevel::Minimal
        )
    }

    pub fn ordinal(&self) -> u8 {
        match self {
            DegradationLevel::Full => 3,
            DegradationLevel::Limited => 2,
            DegradationLevel::Minimal => 1,
            DegradationLevel::None => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubsystemHealth {
    Healthy,
    Degraded {
        level: DegradationLevel,
        reason: String,
    },
    Failed {
        reason: String,
        since: u64,
    },
    Recovering {
        progress: f64,
    },
}

impl SubsystemHealth {
    pub fn is_operational(&self) -> bool {
        match self {
            SubsystemHealth::Healthy => true,
            SubsystemHealth::Degraded { level, .. } => level.is_available(),
            SubsystemHealth::Failed { .. } => false,
            SubsystemHealth::Recovering { .. } => true,
        }
    }

    pub fn can_handle(&self, capability: &str) -> bool {
        match self {
            SubsystemHealth::Healthy => true,
            SubsystemHealth::Degraded { level, .. } => match level {
                DegradationLevel::Full => true,
                DegradationLevel::Limited => {
                    let essential = ["io", "self_maintenance", "core_reasoning"];
                    essential.contains(&capability)
                }
                DegradationLevel::Minimal => {
                    let basic = ["io", "self_maintenance"];
                    basic.contains(&capability)
                }
                DegradationLevel::None => false,
            },
            SubsystemHealth::Failed { .. } => false,
            SubsystemHealth::Recovering { progress: _ } => true,
        }
    }

    pub fn degradation_level(&self) -> DegradationLevel {
        match self {
            SubsystemHealth::Healthy => DegradationLevel::Full,
            SubsystemHealth::Degraded { level, .. } => level.clone(),
            SubsystemHealth::Failed { .. } => DegradationLevel::None,
            SubsystemHealth::Recovering { .. } => DegradationLevel::Limited,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DegradationPolicy {
    pub crash_safe: bool,
    pub auto_recover: bool,
    pub max_retries: u32,
    pub cooldown_secs: u64,
}

impl Default for DegradationPolicy {
    fn default() -> Self {
        DegradationPolicy {
            crash_safe: true,
            auto_recover: true,
            max_retries: 3,
            cooldown_secs: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FallbackResponse {
    pub response: String,
    pub level: DegradationLevel,
    pub note: Option<String>,
}

impl FallbackResponse {
    pub fn new(response: String, level: DegradationLevel) -> Self {
        FallbackResponse {
            response,
            level,
            note: None,
        }
    }

    pub fn with_note(response: String, level: DegradationLevel, note: String) -> Self {
        FallbackResponse {
            response,
            level,
            note: Some(note),
        }
    }
}

#[derive(Clone)]
pub struct GracefulDegradationManager {
    pub subsystems: HashMap<String, SubsystemHealth>,
    pub capability_map: HashMap<String, Vec<String>>,
    pub fallback_map: HashMap<String, String>,
    pub recovery_attempts: HashMap<String, u32>,
    policies: HashMap<String, DegradationPolicy>,
}

impl GracefulDegradationManager {
    pub fn new() -> Self {
        GracefulDegradationManager {
            subsystems: HashMap::new(),
            capability_map: HashMap::new(),
            fallback_map: HashMap::new(),
            recovery_attempts: HashMap::new(),
            policies: HashMap::new(),
        }
    }

    pub fn register_subsystem(&mut self, name: String, capabilities: Vec<String>) {
        self.subsystems
            .insert(name.clone(), SubsystemHealth::Healthy);
        for cap in &capabilities {
            self.capability_map
                .entry(cap.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());
        }
        self.recovery_attempts.insert(name.clone(), 0);
        self.policies.insert(name, DegradationPolicy::default());
    }

    pub fn register_subsystem_with_policy(
        &mut self,
        name: String,
        capabilities: Vec<String>,
        policy: DegradationPolicy,
    ) {
        self.subsystems
            .insert(name.clone(), SubsystemHealth::Healthy);
        for cap in &capabilities {
            self.capability_map
                .entry(cap.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());
        }
        self.recovery_attempts.insert(name.clone(), 0);
        self.policies.insert(name, policy);
    }

    pub fn register_fallback(&mut self, capability: String, fallback: String) {
        self.fallback_map.insert(capability, fallback);
    }

    pub fn report_failure(&mut self, subsystem: &str, reason: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.subsystems.insert(
            subsystem.to_string(),
            SubsystemHealth::Failed { reason, since: now },
        );
    }

    pub fn report_degradation(&mut self, subsystem: &str, level: DegradationLevel, reason: String) {
        self.subsystems.insert(
            subsystem.to_string(),
            SubsystemHealth::Degraded { level, reason },
        );
    }

    pub fn attempt_recovery(&mut self, subsystem: &str) -> bool {
        let attempts = self.recovery_attempts.get(subsystem).copied().unwrap_or(0);
        let policy = self.policies.get(subsystem).cloned().unwrap_or_default();

        if attempts >= policy.max_retries {
            return false;
        }

        self.recovery_attempts
            .insert(subsystem.to_string(), attempts + 1);

        match self.subsystems.get(subsystem) {
            Some(SubsystemHealth::Failed { .. }) | Some(SubsystemHealth::Degraded { .. }) => {
                self.subsystems.insert(
                    subsystem.to_string(),
                    SubsystemHealth::Recovering { progress: 0.5 },
                );
                true
            }
            _ => false,
        }
    }

    pub fn mark_recovered(&mut self, subsystem: &str) {
        self.subsystems
            .insert(subsystem.to_string(), SubsystemHealth::Healthy);
        self.recovery_attempts.insert(subsystem.to_string(), 0);
    }

    pub fn execute_with_degradation<F, T>(
        &mut self,
        capability: &str,
        primary: F,
        fallback: Option<fn() -> T>,
    ) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let required = self
            .capability_map
            .get(capability)
            .cloned()
            .unwrap_or_default();

        let primary_ok = required.is_empty()
            || required.iter().any(|s| {
                self.subsystems
                    .get(s)
                    .map(|h| h.can_handle(capability))
                    .unwrap_or(false)
            });

        if primary_ok {
            match primary() {
                Ok(val) => return Ok(val),
                Err(_) => {}
            }
        }

        if let Some(fallback_name) = self.fallback_map.get(capability) {
            if self
                .subsystems
                .get(fallback_name)
                .map(|h| h.is_operational())
                .unwrap_or(false)
            {
                if let Some(fb_fn) = fallback {
                    return Ok(fb_fn());
                }
            }
        }

        if let Some(fb_fn) = fallback {
            return Ok(fb_fn());
        }

        Err(format!(
            "capability '{}' unavailable: no operational subsystems or fallbacks",
            capability
        ))
    }

    pub fn global_degradation_level(&self) -> DegradationLevel {
        let mut worst = DegradationLevel::Full;
        for health in self.subsystems.values() {
            let level = health.degradation_level();
            if level.ordinal() < worst.ordinal() {
                worst = level;
            }
        }
        worst
    }

    pub fn operational_capabilities(&self) -> Vec<String> {
        self.capability_map
            .keys()
            .filter(|cap| {
                let subsystem_names = self.capability_map.get(*cap).unwrap();
                subsystem_names.iter().any(|name| {
                    self.subsystems
                        .get(name)
                        .map(|h| h.is_operational())
                        .unwrap_or(false)
                }) || self.fallback_map.contains_key(*cap)
            })
            .cloned()
            .collect()
    }

    pub fn subsystem_health(&self, name: &str) -> Option<SubsystemHealth> {
        self.subsystems.get(name).cloned()
    }

    pub fn policy(&self, name: &str) -> DegradationPolicy {
        self.policies.get(name).cloned().unwrap_or_default()
    }

    pub fn with_reasoning_modules() -> Self {
        let mut mgr = Self::new();

        mgr.register_subsystem_with_policy(
            "mcts_reasoner".into(),
            vec![
                "tree_search".into(),
                "planning".into(),
                "multi_step_reasoning".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 3,
                cooldown_secs: 15,
            },
        );
        mgr.register_fallback(
            "tree_search".into(),
            "Use heuristic search: what-if analysis without full MCTS".into(),
        );

        mgr.register_subsystem_with_policy(
            "parallel_hypothesis".into(),
            vec![
                "hypothesis_evaluation".into(),
                "belief_update".into(),
                "abduction".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 2,
                cooldown_secs: 10,
            },
        );
        mgr.register_fallback(
            "hypothesis_evaluation".into(),
            "Sequential single-hypothesis reasoning".into(),
        );

        mgr.register_subsystem_with_policy(
            "dead_end_detector".into(),
            vec![
                "loop_detection".into(),
                "divergence_monitoring".into(),
                "reasoning_health".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 5,
                cooldown_secs: 5,
            },
        );
        mgr.register_fallback(
            "loop_detection".into(),
            "Simple step counter: max 50 steps".into(),
        );

        mgr.register_subsystem_with_policy(
            "epistemic_humility".into(),
            vec![
                "uncertainty_expression".into(),
                "deferral".into(),
                "calibrated_dont_know".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: false,
                max_retries: 1,
                cooldown_secs: 60,
            },
        );
        mgr.register_fallback(
            "uncertainty_expression".into(),
            "Raw confidence without calibration".into(),
        );

        mgr.register_subsystem_with_policy(
            "process_reward_model".into(),
            vec![
                "step_evaluation".into(),
                "reward_shaping".into(),
                "trajectory_scoring".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 3,
                cooldown_secs: 20,
            },
        );
        mgr.register_fallback(
            "step_evaluation".into(),
            "Uniform step reward: each step = equal weight".into(),
        );

        mgr.register_subsystem_with_policy(
            "bidirectional_pruner".into(),
            vec![
                "pre_eval_pruning".into(),
                "post_eval_pruning".into(),
                "dedup".into(),
                "pareto_optimization".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 2,
                cooldown_secs: 30,
            },
        );
        mgr.register_fallback(
            "pre_eval_pruning".into(),
            "No pruning: evaluate all paths".into(),
        );

        mgr.register_subsystem_with_policy(
            "strategy_selector".into(),
            vec![
                "strategy_selection".into(),
                "self_healing".into(),
                "failure_tracking".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 4,
                cooldown_secs: 10,
            },
        );
        mgr.register_fallback(
            "strategy_selection".into(),
            "Default to MCTS fallback".into(),
        );

        mgr.register_subsystem_with_policy(
            "process_calibration".into(),
            vec![
                "trajectory_calibration".into(),
                "step_calibration".into(),
                "process_confidence".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 3,
                cooldown_secs: 25,
            },
        );
        mgr.register_fallback(
            "trajectory_calibration".into(),
            "Global calibration only".into(),
        );

        mgr.register_subsystem_with_policy(
            "counterfactual_simulator".into(),
            vec![
                "what_if_analysis".into(),
                "vsa_perturbation".into(),
                "alternative_paths".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: false,
                max_retries: 2,
                cooldown_secs: 30,
            },
        );
        mgr.register_fallback(
            "what_if_analysis".into(),
            "Skip counterfactual analysis".into(),
        );

        mgr.register_subsystem_with_policy(
            "gwt_self_interrupt".into(),
            vec![
                "cycle_interrupt".into(),
                "entropy_monitoring".into(),
                "recovery_signaling".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 3,
                cooldown_secs: 15,
            },
        );
        mgr.register_fallback(
            "cycle_interrupt".into(),
            "Cycle timeout: hard abort at 2x limit".into(),
        );

        mgr.register_subsystem_with_policy(
            "curiosity_exploration".into(),
            vec![
                "novelty_detection".into(),
                "information_gain".into(),
                "exploration_planning".into(),
            ],
            DegradationPolicy {
                crash_safe: true,
                auto_recover: true,
                max_retries: 2,
                cooldown_secs: 20,
            },
        );
        mgr.register_fallback("novelty_detection".into(), "Random exploration".into());

        mgr
    }

    pub fn reasoning_modules_health(&self) -> HashMap<String, SubsystemHealth> {
        let reasoning_names: [&str; 11] = [
            "mcts_reasoner",
            "parallel_hypothesis",
            "dead_end_detector",
            "epistemic_humility",
            "process_reward_model",
            "bidirectional_pruner",
            "strategy_selector",
            "process_calibration",
            "counterfactual_simulator",
            "gwt_self_interrupt",
            "curiosity_exploration",
        ];
        reasoning_names
            .iter()
            .filter_map(|name| {
                self.subsystems
                    .get(*name)
                    .map(|h| (name.to_string(), h.clone()))
            })
            .collect()
    }

    pub fn has_all_reasoning_healthy(&self) -> bool {
        self.reasoning_modules_health()
            .values()
            .all(|h| matches!(h, SubsystemHealth::Healthy))
    }

    pub fn degraded_reasoning_modules(&self) -> Vec<&String> {
        self.subsystems
            .iter()
            .filter(|(_, health)| !matches!(health, SubsystemHealth::Healthy))
            .map(|(name, _)| name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_subsystem() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem(
            "vision".into(),
            vec!["image_processing".into(), "object_detection".into()],
        );
        assert_eq!(mgr.subsystems.len(), 1);
        assert_eq!(
            mgr.subsystem_health("vision"),
            Some(SubsystemHealth::Healthy)
        );
        assert!(mgr.capability_map.contains_key("image_processing"));
        assert!(mgr.capability_map.contains_key("object_detection"));
    }

    #[test]
    fn test_failure_and_degradation_detection() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem("vision".into(), vec!["image_processing".into()]);

        mgr.report_failure("vision", "OOM error".into());
        let health = mgr.subsystem_health("vision").unwrap();
        assert!(!health.is_operational());
        assert!(!health.can_handle("image_processing"));

        mgr.report_degradation(
            "vision",
            DegradationLevel::Limited,
            "memory pressure".into(),
        );
        let health = mgr.subsystem_health("vision").unwrap();
        assert!(health.is_operational());
        assert!(!health.can_handle("image_processing"));
        assert!(health.can_handle("io"));
    }

    #[test]
    fn test_recovery_success() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem("audio".into(), vec!["speech_recognition".into()]);
        mgr.report_failure("audio", "crash".into());

        let ok = mgr.attempt_recovery("audio");
        assert!(ok);
        let health = mgr.subsystem_health("audio").unwrap();
        assert_eq!(health, SubsystemHealth::Recovering { progress: 0.5 });

        mgr.mark_recovered("audio");
        assert_eq!(
            mgr.subsystem_health("audio"),
            Some(SubsystemHealth::Healthy)
        );
    }

    #[test]
    fn test_recovery_max_retries() {
        let mut mgr = GracefulDegradationManager::new();
        let policy = DegradationPolicy {
            crash_safe: true,
            auto_recover: true,
            max_retries: 2,
            cooldown_secs: 10,
        };
        mgr.register_subsystem_with_policy("memory".into(), vec!["storage".into()], policy);
        mgr.report_failure("memory", "corruption".into());

        assert!(mgr.attempt_recovery("memory"));
        assert!(mgr.attempt_recovery("memory"));
        assert!(!mgr.attempt_recovery("memory"));
    }

    #[test]
    fn test_execute_with_degradation_primary_success() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem("search".into(), vec!["web_search".into()]);

        let result =
            mgr.execute_with_degradation("web_search", || Ok::<_, String>(42), None::<fn() -> i32>);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_execute_with_degradation_fallback() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem("search".into(), vec!["web_search".into()]);
        mgr.register_fallback("web_search".into(), "local_cache".into());
        mgr.register_subsystem("local_cache".into(), vec!["web_search".into()]);

        mgr.report_failure("search", "down".into());

        let result: Result<i32, String> = mgr.execute_with_degradation(
            "web_search",
            || Err::<i32, String>("primary failed".into()),
            Some(|| 99i32),
        );
        assert_eq!(result, Ok(99));
    }

    #[test]
    fn test_execute_with_degradation_minimal() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem("search".into(), vec!["web_search".into()]);
        mgr.report_failure("search", "down".into());

        let result: Result<i32, String> = mgr.execute_with_degradation(
            "web_search",
            || Err::<i32, String>("primary failed".into()),
            Some(|| -1i32),
        );
        assert_eq!(result, Ok(-1));
    }

    #[test]
    fn test_global_degradation_level() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem("vision".into(), vec!["img".into()]);
        mgr.register_subsystem("audio".into(), vec!["snd".into()]);

        assert_eq!(mgr.global_degradation_level(), DegradationLevel::Full);

        mgr.report_degradation("vision", DegradationLevel::Limited, "slow".into());
        assert_eq!(mgr.global_degradation_level(), DegradationLevel::Limited);

        mgr.report_failure("audio", "dead".into());
        assert_eq!(mgr.global_degradation_level(), DegradationLevel::None);
    }

    #[test]
    fn test_operational_capabilities_filtering() {
        let mut mgr = GracefulDegradationManager::new();
        mgr.register_subsystem(
            "vision".into(),
            vec!["image_processing".into(), "ocr".into()],
        );
        mgr.register_subsystem("audio".into(), vec!["speech".into()]);

        let caps = mgr.operational_capabilities();
        assert!(caps.contains(&"image_processing".to_string()));
        assert!(caps.contains(&"speech".to_string()));

        mgr.report_failure("vision", "crash".into());
        let caps = mgr.operational_capabilities();
        assert!(!caps.contains(&"image_processing".to_string()));
        assert!(caps.contains(&"speech".to_string()));
    }

    #[test]
    fn test_subsystem_health_can_handle_minimal() {
        let health = SubsystemHealth::Degraded {
            level: DegradationLevel::Minimal,
            reason: "low resources".into(),
        };
        assert!(health.can_handle("io"));
        assert!(health.can_handle("self_maintenance"));
        assert!(!health.can_handle("web_search"));
        assert!(!health.can_handle("image_processing"));
    }

    #[test]
    fn test_degradation_level_ordinal() {
        assert_eq!(DegradationLevel::Full.ordinal(), 3);
        assert_eq!(DegradationLevel::Limited.ordinal(), 2);
        assert_eq!(DegradationLevel::Minimal.ordinal(), 1);
        assert_eq!(DegradationLevel::None.ordinal(), 0);
    }

    #[test]
    fn test_with_reasoning_modules_contains_all() {
        let mgr = GracefulDegradationManager::with_reasoning_modules();
        let expected: [&str; 11] = [
            "mcts_reasoner",
            "parallel_hypothesis",
            "dead_end_detector",
            "epistemic_humility",
            "process_reward_model",
            "bidirectional_pruner",
            "strategy_selector",
            "process_calibration",
            "counterfactual_simulator",
            "gwt_self_interrupt",
            "curiosity_exploration",
        ];
        for name in &expected {
            assert!(
                mgr.subsystems.contains_key(*name),
                "missing subsystem: {}",
                name
            );
        }
        assert!(mgr.fallback_map.contains_key("tree_search"));
        assert!(mgr.fallback_map.contains_key("hypothesis_evaluation"));
        assert!(mgr.fallback_map.contains_key("loop_detection"));
        assert!(mgr.fallback_map.contains_key("what_if_analysis"));
    }

    #[test]
    fn test_reasoning_modules_health_returns_healthy() {
        let mgr = GracefulDegradationManager::with_reasoning_modules();
        let health = mgr.reasoning_modules_health();
        for (name, h) in &health {
            assert_eq!(h, &SubsystemHealth::Healthy, "{} should be Healthy", name);
        }
        assert_eq!(
            health.len(),
            11,
            "should have reasoning_modules_health entries for all 11 modules"
        );
    }

    #[test]
    fn test_degraded_reasoning_modules_empty_initially() {
        let mgr = GracefulDegradationManager::with_reasoning_modules();
        let degraded = mgr.degraded_reasoning_modules();
        assert!(
            degraded.is_empty(),
            "expected no degraded modules, got: {:?}",
            degraded
        );
    }
}
