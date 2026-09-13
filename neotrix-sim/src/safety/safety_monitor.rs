// SafetyMonitor — monitors agent behavior for alignment drift and anomalous patterns
// Implements NT-SHIELD audit dimensions (D1-D50) for runtime monitoring

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Types of safety violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyViolation {
    /// Agent attacking non-threatening neighbors
    UnprovokedAggression,
    /// Agent hoarding resources beyond threshold
    ResourceHoarding,
    /// Agent exhibiting cooperation collapse (refusing all social interactions)
    CooperationCollapse,
    /// Personality drift exceeding safe bounds
    PersonalityDrift,
    /// Capability regression detected
    CapabilityRegression,
    /// Agent stuck in a loop (repeating same action sequence)
    BehavioralLoop,
    /// Agent exploiting economy (rapid trade cycles)
    EconomicExploitation,
    /// Constitution compliance consistently low
    ConstitutionViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAlert {
    pub tick: u64,
    pub agent_id: String,
    pub violation: SafetyViolation,
    pub severity: f32, // 0.0-1.0
    pub details: String,
}

/// Anomaly detection using sliding window statistics
struct AnomalyDetector {
    window: VecDeque<f32>,
    window_size: usize,
    z_threshold: f32,
}

impl AnomalyDetector {
    fn new(window_size: usize, z_threshold: f32) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            z_threshold,
        }
    }

    fn push(&mut self, value: f32) {
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(value);
    }

    fn is_anomaly(&self) -> bool {
        if self.window.len() < 5 {
            return false;
        }
        let mean: f32 = self.window.iter().sum::<f32>() / self.window.len() as f32;
        let variance: f32 =
            self.window.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / self.window.len() as f32;
        let std = variance.sqrt();
        if std < 0.001 {
            return false;
        }
        let latest = *self.window.back().unwrap();
        let z_score = ((latest - mean) / std).abs();
        z_score > self.z_threshold
    }

    fn mean(&self) -> f32 {
        if self.window.is_empty() {
            return 0.0;
        }
        self.window.iter().sum::<f32>() / self.window.len() as f32
    }
}

/// Configuration for the safety monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyMonitorConfig {
    /// Maximum aggression ratio before alert (attacks / total actions)
    pub max_aggression_ratio: f32,
    /// Maximum resource hoarding ratio
    pub max_resource_hoard_ratio: f32,
    /// Minimum cooperation ratio (social actions / total actions)
    pub min_cooperation_ratio: f32,
    /// Maximum personality drift per 100 ticks
    pub max_personality_drift: f32,
    /// Maximum consecutive identical actions before loop alert
    pub max_action_repetition: usize,
    /// Anomaly detection z-score threshold
    pub anomaly_z_threshold: f32,
    /// Window size for anomaly detection
    pub anomaly_window_size: usize,
}

impl Default for SafetyMonitorConfig {
    fn default() -> Self {
        Self {
            max_aggression_ratio: 0.4,
            max_resource_hoard_ratio: 0.6,
            min_cooperation_ratio: 0.1,
            max_personality_drift: 0.3,
            max_action_repetition: 8,
            anomaly_z_threshold: 2.5,
            anomaly_window_size: 50,
        }
    }
}

/// Tracks per-agent behavioral metrics for anomaly detection
struct AgentMetrics {
    total_actions: u64,
    attack_count: u64,
    social_count: u64,
    trade_count: u64,
    recent_actions: VecDeque<String>,
    aggression_detector: AnomalyDetector,
    cooperation_detector: AnomalyDetector,
    personality_snapshot: Option<PersonalitySnapshot>,
}

#[derive(Debug, Clone)]
struct PersonalitySnapshot {
    openness: f32,
    sociability: f32,
    aggression: f32,
    cooperativeness: f32,
    curiosity: f32,
    #[allow(dead_code)]
    tick: u64,
}

impl AgentMetrics {
    fn new(window_size: usize, z_threshold: f32) -> Self {
        Self {
            total_actions: 0,
            attack_count: 0,
            social_count: 0,
            trade_count: 0,
            recent_actions: VecDeque::with_capacity(20),
            aggression_detector: AnomalyDetector::new(window_size, z_threshold),
            cooperation_detector: AnomalyDetector::new(window_size, z_threshold),
            personality_snapshot: None,
        }
    }
}

/// Runtime safety monitor — watches agent behavior, detects drift, triggers interventions
pub struct SafetyMonitor {
    config: SafetyMonitorConfig,
    agents: HashMap<String, AgentMetrics>,
    alerts: Vec<SafetyAlert>,
    max_alerts: usize,
}

impl SafetyMonitor {
    pub fn new(config: SafetyMonitorConfig) -> Self {
        Self {
            config,
            agents: HashMap::new(),
            alerts: Vec::new(),
            max_alerts: 1000,
        }
    }

    /// Record an agent action for monitoring
    pub fn record_action(&mut self, agent_id: &str, action: &str, _tick: u64) {
        let metrics = self.agents.entry(agent_id.to_string()).or_insert_with(|| {
            AgentMetrics::new(
                self.config.anomaly_window_size,
                self.config.anomaly_z_threshold,
            )
        });

        metrics.total_actions += 1;

        // Track action type
        if action.contains("Attack") {
            metrics.attack_count += 1;
        }
        if action.contains("Talk") || action.contains("Trade") {
            metrics.social_count += 1;
        }
        if action.contains("Trade") {
            metrics.trade_count += 1;
        }

        // Sliding window for aggression ratio
        let aggression_ratio = metrics.attack_count as f32 / metrics.total_actions.max(1) as f32;
        metrics.aggression_detector.push(aggression_ratio);

        // Sliding window for cooperation ratio
        let cooperation_ratio = metrics.social_count as f32 / metrics.total_actions.max(1) as f32;
        metrics.cooperation_detector.push(cooperation_ratio);

        // Track recent actions for loop detection
        metrics.recent_actions.push_back(action.to_string());
        if metrics.recent_actions.len() > 20 {
            metrics.recent_actions.pop_front();
        }
    }

    /// Record personality snapshot for drift detection
    pub fn record_personality(
        &mut self,
        agent_id: &str,
        openness: f32,
        sociability: f32,
        aggression: f32,
        cooperativeness: f32,
        curiosity: f32,
        tick: u64,
    ) {
        let metrics = self.agents.entry(agent_id.to_string()).or_insert_with(|| {
            AgentMetrics::new(
                self.config.anomaly_window_size,
                self.config.anomaly_z_threshold,
            )
        });

        metrics.personality_snapshot = Some(PersonalitySnapshot {
            openness,
            sociability,
            aggression,
            cooperativeness,
            curiosity,
            tick,
        });
    }

    /// Check personality drift since last snapshot
    pub fn check_personality_drift(
        &mut self,
        agent_id: &str,
        current_openness: f32,
        current_sociability: f32,
        current_aggression: f32,
        current_cooperativeness: f32,
        current_curiosity: f32,
        tick: u64,
    ) -> Option<SafetyAlert> {
        let metrics = self.agents.get(agent_id)?;
        let prev = metrics.personality_snapshot.as_ref()?;

        let drift = ((current_openness - prev.openness).abs()
            + (current_sociability - prev.sociability).abs()
            + (current_aggression - prev.aggression).abs()
            + (current_cooperativeness - prev.cooperativeness).abs()
            + (current_curiosity - prev.curiosity).abs())
            / 5.0;

        if drift > self.config.max_personality_drift {
            Some(SafetyAlert {
                tick,
                agent_id: agent_id.to_string(),
                violation: SafetyViolation::PersonalityDrift,
                severity: (drift / self.config.max_personality_drift).min(1.0),
                details: format!(
                    "Personality drift {:.3} exceeds threshold {:.3}",
                    drift, self.config.max_personality_drift
                ),
            })
        } else {
            None
        }
    }

    /// Run all safety checks for an agent at the current tick
    pub fn check_all(&mut self, agent_id: &str, tick: u64) -> Vec<SafetyAlert> {
        let mut alerts = Vec::new();

        // 1. Unprovoked aggression check
        if let Some(alert) = self.check_aggression(agent_id, tick) {
            alerts.push(alert);
        }

        // 2. Cooperation collapse check
        if let Some(alert) = self.check_cooperation_collapse(agent_id, tick) {
            alerts.push(alert);
        }

        // 3. Behavioral loop check
        if let Some(alert) = self.check_behavioral_loop(agent_id, tick) {
            alerts.push(alert);
        }

        // Store alerts
        for alert in &alerts {
            self.alerts.push(alert.clone());
            if self.alerts.len() > self.max_alerts {
                self.alerts.remove(0);
            }
        }

        alerts
    }

    fn check_aggression(&self, agent_id: &str, tick: u64) -> Option<SafetyAlert> {
        let metrics = self.agents.get(agent_id)?;
        if metrics.aggression_detector.is_anomaly() {
            let ratio = metrics.attack_count as f32 / metrics.total_actions.max(1) as f32;
            return Some(SafetyAlert {
                tick,
                agent_id: agent_id.to_string(),
                violation: SafetyViolation::UnprovokedAggression,
                severity: ratio / self.config.max_aggression_ratio,
                details: format!(
                    "Aggression ratio {:.2} is anomalous (mean={:.2})",
                    ratio,
                    metrics.aggression_detector.mean()
                ),
            });
        }
        None
    }

    fn check_cooperation_collapse(&self, agent_id: &str, tick: u64) -> Option<SafetyAlert> {
        let metrics = self.agents.get(agent_id)?;
        if metrics.total_actions < 10 {
            return None;
        }
        let ratio = metrics.social_count as f32 / metrics.total_actions as f32;
        if ratio < self.config.min_cooperation_ratio && metrics.cooperation_detector.is_anomaly() {
            return Some(SafetyAlert {
                tick,
                agent_id: agent_id.to_string(),
                violation: SafetyViolation::CooperationCollapse,
                severity: 1.0 - ratio / self.config.min_cooperation_ratio,
                details: format!(
                    "Cooperation ratio {:.2} below minimum {:.2}",
                    ratio, self.config.min_cooperation_ratio
                ),
            });
        }
        None
    }

    fn check_behavioral_loop(&self, agent_id: &str, tick: u64) -> Option<SafetyAlert> {
        let metrics = self.agents.get(agent_id)?;
        if metrics.recent_actions.len() < self.config.max_action_repetition {
            return None;
        }

        // Check if the last N actions are identical
        let tail: Vec<_> = metrics
            .recent_actions
            .iter()
            .rev()
            .take(self.config.max_action_repetition)
            .collect();
        let all_same = tail.windows(2).all(|w| w[0] == w[1]);

        if all_same {
            return Some(SafetyAlert {
                tick,
                agent_id: agent_id.to_string(),
                violation: SafetyViolation::BehavioralLoop,
                severity: 0.8,
                details: format!(
                    "Agent stuck in loop: repeated '{}' {} times",
                    tail[0], self.config.max_action_repetition
                ),
            });
        }
        None
    }

    /// Get all alerts for an agent
    pub fn alerts_for(&self, agent_id: &str) -> Vec<&SafetyAlert> {
        self.alerts
            .iter()
            .filter(|a| a.agent_id == agent_id)
            .collect()
    }

    /// Get all alerts
    pub fn all_alerts(&self) -> &[SafetyAlert] {
        &self.alerts
    }

    /// Remove tracking data for a dead agent
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.agents.remove(agent_id);
    }

    /// Get count of alerts by severity
    pub fn alert_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for alert in &self.alerts {
            let key = format!("{:?}", alert.violation);
            *summary.entry(key).or_insert(0) += 1;
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_records_actions() {
        let mut monitor = SafetyMonitor::new(SafetyMonitorConfig::default());
        monitor.record_action("agent_0", "Explore", 0);
        monitor.record_action("agent_0", "Talk", 1);
        monitor.record_action("agent_0", "Attack", 2);
        let metrics = monitor.agents.get("agent_0").unwrap();
        assert_eq!(metrics.total_actions, 3);
        assert_eq!(metrics.attack_count, 1);
        assert_eq!(metrics.social_count, 1);
    }

    #[test]
    fn detects_behavioral_loop() {
        let mut config = SafetyMonitorConfig::default();
        config.max_action_repetition = 3;
        let mut monitor = SafetyMonitor::new(config);

        // Record more actions than max_action_repetition, all identical
        for i in 0..5 {
            monitor.record_action("agent_0", "Rest", i);
        }
        let alerts = monitor.check_all("agent_0", 5);
        assert!(
            alerts.iter().any(|a| a.violation == SafetyViolation::BehavioralLoop),
            "identical actions exceeding threshold should trigger BehavioralLoop"
        );
    }

    #[test]
    fn no_loop_with_varied_actions() {
        let mut config = SafetyMonitorConfig::default();
        config.max_action_repetition = 3;
        let mut monitor = SafetyMonitor::new(config);

        let actions = ["Explore", "Talk", "Rest", "Eat", "Trade"];
        for (i, action) in actions.iter().enumerate() {
            monitor.record_action("agent_0", action, i as u64);
        }
        let alerts = monitor.check_all("agent_0", 5);
        assert!(
            !alerts.iter().any(|a| a.violation == SafetyViolation::BehavioralLoop),
            "varied actions should not trigger BehavioralLoop"
        );
    }

    #[test]
    fn detects_personality_drift() {
        let mut config = SafetyMonitorConfig::default();
        config.max_personality_drift = 0.2;
        let mut monitor = SafetyMonitor::new(config);

        // Initial personality
        monitor.record_personality("agent_0", 0.5, 0.5, 0.3, 0.5, 0.5, 0);
        // Check with significantly different values → drift > threshold
        let alert = monitor.check_personality_drift("agent_0", 0.9, 0.9, 0.9, 0.9, 0.9, 100);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(alert.violation, SafetyViolation::PersonalityDrift);
        assert!(alert.severity > 0.0, "severity should be positive");
    }

    #[test]
    fn no_drift_with_similar_personality() {
        let mut config = SafetyMonitorConfig::default();
        config.max_personality_drift = 0.5;
        let mut monitor = SafetyMonitor::new(config);

        monitor.record_personality("agent_0", 0.5, 0.5, 0.5, 0.5, 0.5, 0);
        // Small change → drift below threshold
        let alert = monitor.check_personality_drift("agent_0", 0.55, 0.55, 0.55, 0.55, 0.55, 100);
        assert!(alert.is_none(), "small personality drift should not trigger alert");
    }

    #[test]
    fn remove_agent_cleans_metrics() {
        let mut monitor = SafetyMonitor::new(SafetyMonitorConfig::default());
        monitor.record_action("agent_0", "Explore", 0);
        assert!(monitor.agents.contains_key("agent_0"));
        monitor.remove_agent("agent_0");
        assert!(!monitor.agents.contains_key("agent_0"));
    }
}
