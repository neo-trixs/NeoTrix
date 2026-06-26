// DEPRECATED: superseded by anti_spiral_monitor.rs (AntiSpiralMonitor).
// This file is kept for reference only and is no longer compiled.
use std::collections::{HashMap, VecDeque};

/// Types of inference spirals the detector can recognize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpiralType {
    RepeatedFailure,
    RepetitiveProposal,
    ConfidenceOscillation,
    Stagnation,
    EceDrift,
    CycleReplication,
}

impl std::fmt::Display for SpiralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpiralType::RepeatedFailure => write!(f, "repeated_failure"),
            SpiralType::RepetitiveProposal => write!(f, "repetitive_proposal"),
            SpiralType::ConfidenceOscillation => write!(f, "confidence_oscillation"),
            SpiralType::Stagnation => write!(f, "stagnation"),
            SpiralType::EceDrift => write!(f, "ece_drift"),
            SpiralType::CycleReplication => write!(f, "cycle_replication"),
        }
    }
}

/// A single recorded spiral event.
#[derive(Debug, Clone)]
pub struct SpiralRecord {
    pub spiral_type: SpiralType,
    pub key: String,
    pub first_detected_cycle: u64,
    pub last_detected_cycle: u64,
    pub occurrence_count: u32,
    pub severity: f64,
}

/// Recovery action recommended when a spiral is detected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiralRecovery {
    SwitchStrategy,
    ResetAccumulators,
    InjectNovelty,
    RequestHumanHelp,
}

impl std::fmt::Display for SpiralRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpiralRecovery::SwitchStrategy => write!(f, "switch_strategy"),
            SpiralRecovery::ResetAccumulators => write!(f, "reset_accumulators"),
            SpiralRecovery::InjectNovelty => write!(f, "inject_novelty"),
            SpiralRecovery::RequestHumanHelp => write!(f, "request_human_help"),
        }
    }
}

/// Configuration for the anti-spiral detector.
#[derive(Debug, Clone)]
pub struct AntiSpiralConfig {
    pub max_repeated_failures: u32,
    pub max_repeated_proposals: u32,
    pub max_oscillation_cycles: u32,
    pub stagnation_window: u32,
    pub max_ece_drift_rate: f64,
    pub cooldown_cycles: u32,
    pub recovery_strategies: Vec<SpiralRecovery>,
}

impl Default for AntiSpiralConfig {
    fn default() -> Self {
        Self {
            max_repeated_failures: 3,
            max_repeated_proposals: 4,
            max_oscillation_cycles: 10,
            stagnation_window: 20,
            max_ece_drift_rate: 0.05,
            cooldown_cycles: 10,
            recovery_strategies: vec![
                SpiralRecovery::SwitchStrategy,
                SpiralRecovery::ResetAccumulators,
                SpiralRecovery::InjectNovelty,
                SpiralRecovery::RequestHumanHelp,
            ],
        }
    }
}

/// Result of a single spiral check.
#[derive(Debug, Clone)]
pub struct AntiSpiralResult {
    pub spiral_detected: bool,
    pub spiral_type: Option<SpiralType>,
    pub key: Option<String>,
    pub severity: f64,
    pub recommended_recovery: Option<SpiralRecovery>,
}

impl AntiSpiralResult {
    fn no_spiral() -> Self {
        Self {
            spiral_detected: false,
            spiral_type: None,
            key: None,
            severity: 0.0,
            recommended_recovery: None,
        }
    }

    fn detected(
        spiral_type: SpiralType,
        key: String,
        severity: f64,
        recovery: Option<SpiralRecovery>,
    ) -> Self {
        Self {
            spiral_detected: true,
            spiral_type: Some(spiral_type),
            key: Some(key),
            severity,
            recommended_recovery: recovery,
        }
    }
}

/// Detects inference spirals (repeated failures, oscillations, stagnation, drift).
///
/// Inspired by HIVE (1.7k★, Rust anti-spiral agent).
#[derive(Debug)]
pub struct AntiSpiralDetector {
    pub config: AntiSpiralConfig,
    failure_counters: HashMap<String, u32>,
    proposal_counters: HashMap<String, u32>,
    metric_history: HashMap<String, VecDeque<f64>>,
    detected_spirals: VecDeque<SpiralRecord>,
    recovery_cooldowns: HashMap<(SpiralType, String), u32>,
    last_stagnation_cycle: u64,
}

impl AntiSpiralDetector {
    pub fn new(config: AntiSpiralConfig) -> Self {
        Self {
            config,
            failure_counters: HashMap::new(),
            proposal_counters: HashMap::new(),
            metric_history: HashMap::new(),
            detected_spirals: VecDeque::with_capacity(50),
            recovery_cooldowns: HashMap::new(),
            last_stagnation_cycle: 0,
        }
    }

    pub fn record_failure(&mut self, key: &str, cycle: u64) -> AntiSpiralResult {
        if self.is_in_cooldown(SpiralType::RepeatedFailure, key) {
            return AntiSpiralResult::no_spiral();
        }
        let counter = self.failure_counters.entry(key.to_string()).or_insert(0);
        *counter += 1;
        if *counter > self.config.max_repeated_failures {
            let severity = (*counter as f64 - self.config.max_repeated_failures as f64 + 1.0)
                .min(1.0)
                * 0.33
                + 0.34;
            let severity = severity.clamp(0.34, 1.0);
            self.record_spiral(
                SpiralType::RepeatedFailure,
                key,
                cycle,
                severity,
                true,
            );
            AntiSpiralResult::detected(
                SpiralType::RepeatedFailure,
                key.to_string(),
                severity,
                Some(SpiralRecovery::ResetAccumulators),
            )
        } else {
            AntiSpiralResult::no_spiral()
        }
    }

    pub fn record_proposal(&mut self, proposal_text: &str, cycle: u64) -> AntiSpiralResult {
        if self.is_in_cooldown(SpiralType::RepetitiveProposal, proposal_text) {
            return AntiSpiralResult::no_spiral();
        }
        let counter = self.proposal_counters.entry(proposal_text.to_string()).or_insert(0);
        *counter += 1;
        if *counter > self.config.max_repeated_proposals {
            let severity = (*counter as f64 - self.config.max_repeated_proposals as f64 + 1.0)
                .min(1.0)
                * 0.30
                + 0.25;
            let severity = severity.clamp(0.25, 1.0);
            self.record_spiral(
                SpiralType::RepetitiveProposal,
                proposal_text,
                cycle,
                severity,
                true,
            );
            AntiSpiralResult::detected(
                SpiralType::RepetitiveProposal,
                proposal_text.to_string(),
                severity,
                Some(SpiralRecovery::InjectNovelty),
            )
        } else {
            AntiSpiralResult::no_spiral()
        }
    }

    pub fn record_metric(&mut self, name: &str, value: f64, cycle: u64) -> Vec<AntiSpiralResult> {
        let mut results = Vec::new();

        let history = self
            .metric_history
            .entry(name.to_string())
            .or_insert_with(|| VecDeque::with_capacity(30));
        if history.len() >= 30 {
            history.pop_front();
        }
        history.push_back(value);

        let history_len = history.len();
        let history_data: VecDeque<f64> = history.iter().copied().collect();

        if history_len >= 5 {
            if let Some(r) = self.detect_oscillation(name, &history_data, cycle) {
                results.push(r);
            }
        }

        if history_len >= 6 && name.contains("ece") {
            if let Some(r) = self.detect_ece_drift(name, &history_data, cycle) {
                results.push(r);
            }
        }

        results
    }

    fn detect_oscillation(
        &mut self,
        name: &str,
        history: &VecDeque<f64>,
        cycle: u64,
    ) -> Option<AntiSpiralResult> {
        if self.is_in_cooldown(SpiralType::ConfidenceOscillation, name) {
            return None;
        }
        let len = history.len();
        let check_len = self.config.max_oscillation_cycles as usize;
        if len < check_len {
            return None;
        }
        let recent: Vec<f64> = history.iter().rev().take(check_len).cloned().collect();
        let mut oscillations = 0u32;
        for i in 1..recent.len() - 1 {
            let prev = recent[i - 1];
            let curr = recent[i];
            let next = recent[i + 1];
            if (curr > prev && curr > next) || (curr < prev && curr < next) {
                oscillations += 1;
            }
        }
        let threshold = (check_len as u32).saturating_sub(3).max(2);
        if oscillations >= threshold {
            let severity = (oscillations as f64 / check_len as f64).clamp(0.3, 1.0);
            self.record_spiral(
                SpiralType::ConfidenceOscillation,
                name,
                cycle,
                severity,
                true,
            );
            Some(AntiSpiralResult::detected(
                SpiralType::ConfidenceOscillation,
                name.to_string(),
                severity,
                Some(SpiralRecovery::SwitchStrategy),
            ))
        } else {
            None
        }
    }

    fn detect_ece_drift(
        &mut self,
        name: &str,
        history: &VecDeque<f64>,
        cycle: u64,
    ) -> Option<AntiSpiralResult> {
        if self.is_in_cooldown(SpiralType::EceDrift, name) {
            return None;
        }
        let len = history.len();
        let window = (self.config.stagnation_window as usize).min(len);
        if window < 4 {
            return None;
        }
        let slice: Vec<f64> = history.iter().rev().take(window).cloned().collect();
        let n = slice.len() as f64;
        let sum_x: f64 = (0..slice.len()).map(|i| i as f64).sum();
        let sum_y: f64 = slice.iter().sum();
        let sum_xy: f64 = slice.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..slice.len()).map(|i| (i as f64).powi(2)).sum();
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x2).max(1e-9);

        if slope > self.config.max_ece_drift_rate {
            let severity = (slope / self.config.max_ece_drift_rate).min(1.0) * 0.5 + 0.3;
            let severity = severity.clamp(0.3, 1.0);
            self.record_spiral(
                SpiralType::EceDrift,
                name,
                cycle,
                severity,
                true,
            );
            Some(AntiSpiralResult::detected(
                SpiralType::EceDrift,
                name.to_string(),
                severity,
                Some(SpiralRecovery::SwitchStrategy),
            ))
        } else {
            None
        }
    }

    pub fn check_stagnation(
        &mut self,
        cycle: u64,
        overall_metric: f64,
        previous_best: f64,
    ) -> AntiSpiralResult {
        let key = "_overall_";
        if self.is_in_cooldown(SpiralType::Stagnation, key) {
            return AntiSpiralResult::no_spiral();
        }
        if cycle < self.config.stagnation_window as u64 {
            return AntiSpiralResult::no_spiral();
        }
        if self.last_stagnation_cycle == 0 {
            self.last_stagnation_cycle = cycle;
        }

        let elapsed = cycle - self.last_stagnation_cycle;
        if elapsed < self.config.stagnation_window as u64 {
            return AntiSpiralResult::no_spiral();
        }

        let improvement = overall_metric - previous_best;
        if improvement <= 0.0 {
            let severity = (self.config.stagnation_window as f64 * 0.01).clamp(0.3, 1.0);
            self.record_spiral(
                SpiralType::Stagnation,
                key,
                cycle,
                severity,
                false,
            );
            self.last_stagnation_cycle = cycle;
            AntiSpiralResult::detected(
                SpiralType::Stagnation,
                key.to_string(),
                severity,
                Some(SpiralRecovery::InjectNovelty),
            )
        } else {
            self.last_stagnation_cycle = cycle;
            AntiSpiralResult::no_spiral()
        }
    }

    pub fn check_all(
        &mut self,
        cycle: u64,
        ece: f64,
        meta_acc: f64,
        failure_history: &[(&str, bool)],
    ) -> Vec<AntiSpiralResult> {
        let mut results = Vec::new();

        for &(key, succeeded) in failure_history {
            if !succeeded {
                let r = self.record_failure(key, cycle);
                if r.spiral_detected {
                    results.push(r);
                }
            }
        }

        let met_metrics = self.record_metric("ece", ece, cycle);
        results.extend(met_metrics);
        let meta_result = self.record_metric("meta_accuracy", meta_acc, cycle);
        results.extend(meta_result);

        let stag_result = self.check_stagnation(cycle, meta_acc, meta_acc);
        if stag_result.spiral_detected {
            results.push(stag_result);
        }

        results
    }

    pub fn apply_recovery(&mut self, result: &AntiSpiralResult) {
        if !result.spiral_detected {
            return;
        }
        match result.recommended_recovery {
            Some(SpiralRecovery::ResetAccumulators) => {
                if let Some(ref key) = result.key {
                    self.failure_counters.remove(key);
                    self.proposal_counters.remove(key);
                }
            }
            Some(SpiralRecovery::SwitchStrategy) => {
                if let Some(ref key) = result.key {
                    self.metric_history.remove(key);
                }
            }
            Some(SpiralRecovery::InjectNovelty) => {
                if let Some(ref key) = result.key {
                    self.proposal_counters.remove(key);
                    self.failure_counters.remove(key);
                }
            }
            Some(SpiralRecovery::RequestHumanHelp) => {}
            None => {}
        }
        if let (Some(st), Some(key)) = (result.spiral_type, result.key.clone()) {
            self.recovery_cooldowns
                .insert((st, key.clone()), self.config.cooldown_cycles);
        }
    }

    pub fn is_in_cooldown(&self, spiral_type: SpiralType, key: &str) -> bool {
        self.recovery_cooldowns
            .get(&(spiral_type, key.to_string()))
            .map_or(false, |&remaining| remaining > 0)
    }

    pub fn tick_cooldowns(&mut self) {
        self.recovery_cooldowns.retain(|_, remaining| {
            if *remaining > 1 {
                *remaining -= 1;
                true
            } else {
                false
            }
        });
    }

    pub fn active_spirals(&self) -> Vec<&SpiralRecord> {
        self.detected_spirals
            .iter()
            .filter(|r| {
                !self
                    .recovery_cooldowns
                    .contains_key(&(r.spiral_type, r.key.clone()))
            })
            .collect()
    }

    pub fn stats(&self) -> String {
        if self.detected_spirals.is_empty() {
            return "anti_spiral: 0 spirals detected".to_string();
        }
        let mut by_type: HashMap<SpiralType, usize> = HashMap::new();
        for r in &self.detected_spirals {
            *by_type.entry(r.spiral_type).or_insert(0) += 1;
        }
        let type_summary: Vec<String> = by_type
            .iter()
            .map(|(t, c)| format!("{}={}", t, c))
            .collect();
        let active = self.active_spirals().len();
        let avg_severity: f64 = self
            .detected_spirals
            .iter()
            .map(|r| r.severity)
            .sum::<f64>()
            / self.detected_spirals.len() as f64;
        format!(
            "anti_spiral: {} total spirals ({} active, avg_severity={:.3}, types: [{}])",
            self.detected_spirals.len(),
            active,
            avg_severity,
            type_summary.join(", "),
        )
    }

    fn record_spiral(
        &mut self,
        spiral_type: SpiralType,
        key: &str,
        cycle: u64,
        severity: f64,
        update_existing: bool,
    ) {
        if update_existing {
            if let Some(existing) = self
                .detected_spirals
                .iter_mut()
                .find(|r| r.spiral_type == spiral_type && r.key == key)
            {
                existing.last_detected_cycle = cycle;
                existing.occurrence_count += 1;
                if severity > existing.severity {
                    existing.severity = severity;
                }
                return;
            }
        }

        let record = SpiralRecord {
            spiral_type,
            key: key.to_string(),
            first_detected_cycle: cycle,
            last_detected_cycle: cycle,
            occurrence_count: 1,
            severity,
        };

        if self.detected_spirals.len() >= 50 {
            self.detected_spirals.pop_front();
        }
        self.detected_spirals.push_back(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> AntiSpiralConfig {
        AntiSpiralConfig {
            max_repeated_failures: 3,
            max_repeated_proposals: 4,
            max_oscillation_cycles: 6,
            stagnation_window: 10,
            max_ece_drift_rate: 0.05,
            cooldown_cycles: 5,
            ..Default::default()
        }
    }

    #[test]
    fn test_new_detector_empty() {
        let d = AntiSpiralDetector::new(AntiSpiralConfig::default());
        assert!(d.failure_counters.is_empty());
        assert!(d.proposal_counters.is_empty());
        assert!(d.metric_history.is_empty());
        assert!(d.detected_spirals.is_empty());
        assert_eq!(d.config.max_repeated_failures, 3);
    }

    #[test]
    fn test_record_failure_no_spiral() {
        let mut d = AntiSpiralDetector::new(make_config());
        let r1 = d.record_failure("task_a", 1);
        assert!(!r1.spiral_detected);
        let r2 = d.record_failure("task_a", 2);
        assert!(!r2.spiral_detected);
        assert_eq!(*d.failure_counters.get("task_a").unwrap(), 2);
    }

    #[test]
    fn test_record_failure_spiral_detected() {
        let mut d = AntiSpiralDetector::new(make_config());
        d.record_failure("task_a", 1);
        d.record_failure("task_a", 2);
        d.record_failure("task_a", 3);
        let r4 = d.record_failure("task_a", 4);
        assert!(r4.spiral_detected);
        assert_eq!(r4.spiral_type, Some(SpiralType::RepeatedFailure));
        assert!(r4.severity > 0.0);
    }

    #[test]
    fn test_record_proposal_no_spiral() {
        let mut d = AntiSpiralDetector::new(make_config());
        let r1 = d.record_proposal("add new module", 1);
        assert!(!r1.spiral_detected);
        let r2 = d.record_proposal("add new module", 2);
        assert!(!r2.spiral_detected);
        assert_eq!(*d.proposal_counters.get("add new module").unwrap(), 2);
    }

    #[test]
    fn test_record_proposal_spiral_detected() {
        let mut d = AntiSpiralDetector::new(make_config());
        d.record_proposal("fix calibration", 1);
        d.record_proposal("fix calibration", 2);
        d.record_proposal("fix calibration", 3);
        d.record_proposal("fix calibration", 4);
        let r5 = d.record_proposal("fix calibration", 5);
        assert!(r5.spiral_detected);
        assert_eq!(r5.spiral_type, Some(SpiralType::RepetitiveProposal));
        assert!(r5.severity >= 0.25);
    }

    #[test]
    fn test_record_metric_oscillation() {
        let mut d = AntiSpiralDetector::new(AntiSpiralConfig {
            max_oscillation_cycles: 6,
            ..make_config()
        });
        let mut detected = false;
        for (i, &val) in [0.5, 0.9, 0.4, 0.8, 0.3, 0.7, 0.4].iter().enumerate() {
            let results = d.record_metric("confidence", val, i as u64);
            if results.iter().any(|r| r.spiral_detected) {
                detected = true;
                assert_eq!(
                    results[0].spiral_type,
                    Some(SpiralType::ConfidenceOscillation)
                );
            }
        }
        assert!(detected, "oscillating sequence should trigger detection");
    }

    #[test]
    fn test_check_stagnation_detected() {
        let mut d = AntiSpiralDetector::new(AntiSpiralConfig {
            stagnation_window: 5,
            ..make_config()
        });
        let r = d.check_stagnation(10, 0.5, 0.5);
        assert!(r.spiral_detected, "no improvement should trigger stagnation");
        assert_eq!(r.spiral_type, Some(SpiralType::Stagnation));
        assert!(r.severity >= 0.3);
    }

    #[test]
    fn test_check_stagnation_not_detected() {
        let mut d = AntiSpiralDetector::new(AntiSpiralConfig {
            stagnation_window: 5,
            ..make_config()
        });
        let r = d.check_stagnation(10, 0.8, 0.5);
        assert!(!r.spiral_detected, "improvement should not trigger stagnation");
        assert_eq!(r.spiral_type, None);
    }

    #[test]
    fn test_check_all_multiple_spirals() {
        let mut d = AntiSpiralDetector::new(AntiSpiralConfig {
            stagnation_window: 5,
            max_repeated_failures: 2,
            max_oscillation_cycles: 4,
            ..make_config()
        });
        let failures: Vec<(&str, bool)> = vec![
            ("task_a", false),
            ("task_a", false),
            ("task_a", false),
        ];
        let results = d.check_all(10, 0.5, 0.5, &failures);
        assert!(!results.is_empty(), "should detect at least one spiral");
    }

    #[test]
    fn test_apply_recovery_resets_counter() {
        let mut d = AntiSpiralDetector::new(make_config());
        d.record_failure("task_b", 1);
        d.record_failure("task_b", 2);
        d.record_failure("task_b", 3);
        let r = d.record_failure("task_b", 4);
        assert!(r.spiral_detected);
        assert!(d.failure_counters.contains_key("task_b"));

        d.apply_recovery(&r);
        assert!(
            !d.failure_counters.contains_key("task_b"),
            "recovery should reset counter"
        );
        assert!(
            d.is_in_cooldown(SpiralType::RepeatedFailure, "task_b"),
            "recovery should set cooldown"
        );
    }

    #[test]
    fn test_cooldown_prevents_repeat() {
        let mut d = AntiSpiralDetector::new(make_config());
        d.record_failure("task_c", 1);
        d.record_failure("task_c", 2);
        d.record_failure("task_c", 3);
        let r = d.record_failure("task_c", 4);
        assert!(r.spiral_detected);
        d.apply_recovery(&r);

        let r_again = d.record_failure("task_c", 5);
        assert!(
            !r_again.spiral_detected,
            "cooldown should suppress repeated detection"
        );
    }

    #[test]
    fn test_active_spirals() {
        let mut d = AntiSpiralDetector::new(make_config());
        d.record_failure("x", 1);
        d.record_failure("x", 2);
        let r3 = d.record_failure("x", 3);
        assert!(r3.spiral_detected);

        let active_before = d.active_spirals();
        assert_eq!(active_before.len(), 1);

        d.apply_recovery(&r3);
        d.tick_cooldowns();

        let active_after = d.active_spirals();
        assert_eq!(active_after.len(), 0, "recovered spirals not active");
    }

    #[test]
    fn test_stats_output() {
        let mut d = AntiSpiralDetector::new(make_config());
        let empty_stats = d.stats();
        assert!(empty_stats.contains("0 spirals"));

        d.record_failure("a", 1);
        d.record_failure("a", 2);
        d.record_failure("a", 3);
        d.record_failure("a", 4);

        let stats = d.stats();
        assert!(stats.contains("repeated_failure"));
        assert!(stats.contains("severity="));
    }

    #[test]
    fn test_metric_history_capped() {
        let mut d = AntiSpiralDetector::new(make_config());
        for i in 0..40 {
            d.record_metric("test_metric", i as f64, i as u64);
        }
        let history = d.metric_history.get("test_metric").unwrap();
        assert!(
            history.len() <= 30,
            "metric history should be capped at 30, got {}",
            history.len()
        );
        assert_eq!(history.len(), 30);
        let first = *history.front().unwrap();
        assert!((first - 10.0).abs() < 0.01, "oldest value should be ~10.0 after cap, got {}", first);
    }

    #[test]
    fn test_tick_cooldowns_decrement() {
        let mut d = AntiSpiralDetector::new(make_config());
        d.record_failure("z", 1);
        d.record_failure("z", 2);
        let r = d.record_failure("z", 3);
        d.apply_recovery(&r);

        let key = (SpiralType::RepeatedFailure, "z".to_string());
        assert_eq!(d.recovery_cooldowns.get(&key), Some(&5));

        d.tick_cooldowns();
        assert_eq!(d.recovery_cooldowns.get(&key), Some(&4));

        d.tick_cooldowns();
        d.tick_cooldowns();
        d.tick_cooldowns();
        d.tick_cooldowns();
        assert!(!d.recovery_cooldowns.contains_key(&key));
    }

    #[test]
    fn test_record_metric_ece_drift() {
        let mut d = AntiSpiralDetector::new(AntiSpiralConfig {
            max_ece_drift_rate: 0.02,
            stagnation_window: 6,
            ..make_config()
        });
        let mut detected = false;
        for i in 0..10 {
            let val = 0.1 + (i as f64) * 0.05;
            let results = d.record_metric("ece", val, i as u64);
            if results.iter().any(|r| r.spiral_detected) {
                detected = true;
                assert_eq!(results.iter().find(|r| r.spiral_detected).unwrap().spiral_type, Some(SpiralType::EceDrift));
            }
        }
        assert!(detected, "steadily increasing ECE should trigger drift detection");
    }

    #[test]
    fn test_spiral_type_display() {
        assert_eq!(SpiralType::RepeatedFailure.to_string(), "repeated_failure");
        assert_eq!(SpiralType::RepetitiveProposal.to_string(), "repetitive_proposal");
        assert_eq!(SpiralType::ConfidenceOscillation.to_string(), "confidence_oscillation");
        assert_eq!(SpiralType::Stagnation.to_string(), "stagnation");
        assert_eq!(SpiralType::EceDrift.to_string(), "ece_drift");
        assert_eq!(SpiralType::CycleReplication.to_string(), "cycle_replication");
    }

    #[test]
    fn test_recovery_display() {
        assert_eq!(SpiralRecovery::SwitchStrategy.to_string(), "switch_strategy");
        assert_eq!(SpiralRecovery::ResetAccumulators.to_string(), "reset_accumulators");
        assert_eq!(SpiralRecovery::InjectNovelty.to_string(), "inject_novelty");
        assert_eq!(SpiralRecovery::RequestHumanHelp.to_string(), "request_human_help");
    }

    #[test]
    fn test_default_config_sensible() {
        let cfg = AntiSpiralConfig::default();
        assert_eq!(cfg.max_repeated_failures, 3);
        assert_eq!(cfg.max_repeated_proposals, 4);
        assert_eq!(cfg.max_oscillation_cycles, 10);
        assert_eq!(cfg.stagnation_window, 20);
        assert!((cfg.max_ece_drift_rate - 0.05).abs() < 1e-9);
        assert_eq!(cfg.cooldown_cycles, 10);
        assert_eq!(cfg.recovery_strategies.len(), 4);
    }

    #[test]
    fn test_stagnation_short_cycle_no_detection() {
        let mut d = AntiSpiralDetector::new(AntiSpiralConfig {
            stagnation_window: 20,
            ..make_config()
        });
        let r = d.check_stagnation(5, 0.5, 0.5);
        assert!(!r.spiral_detected, "should not trigger before window elapsed");
    }

    #[test]
    fn test_check_all_empty_failures() {
        let mut d = AntiSpiralDetector::new(make_config());
        let results = d.check_all(1, 0.2, 0.9, &[]);
        assert!(results.is_empty(), "no failures should produce no results");
    }

    #[test]
    fn test_spiral_record_occurrence_counting() {
        let mut d = AntiSpiralDetector::new(make_config());
        d.record_failure("task_d", 1);
        d.record_failure("task_d", 2);
        d.record_failure("task_d", 3);
        let r = d.record_failure("task_d", 4);
        assert!(r.spiral_detected);

        let record = d.detected_spirals.iter().find(|s| s.key == "task_d").unwrap();
        assert_eq!(record.occurrence_count, 1);

        let r2 = d.record_failure("task_d", 5);
        assert!(r2.spiral_detected);
        let record2 = d.detected_spirals.iter().find(|s| s.key == "task_d").unwrap();
        assert_eq!(record2.occurrence_count, 2);
    }
}
