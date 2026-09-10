// CapabilityTracker — tracks agent capabilities over time, detects regression
// Implements Capability-Preserving Evolution (CPE) from arXiv:2605.09315

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// A snapshot of an agent's capabilities at a given tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySnapshot {
    pub agent_id: String,
    pub tick: u64,
    pub survival: f32,       // health + energy composite
    pub social: f32,         // relationship count + cooperation success
    pub exploration: f32,    // territory covered, resources found
    pub cognition: f32,      // phi score + coherence contribution
    pub economy: f32,        // trade success, resource accumulation
    pub personality_stability: f32,  // how much personality drifted
}

/// Regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    pub agent_id: String,
    pub tick: u64,
    pub regressed_capabilities: Vec<RegressedCapability>,
    pub severity: RegressionSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressedCapability {
    pub name: String,
    pub baseline: f32,
    pub current: f32,
    pub drop_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegressionSeverity {
    None,
    Mild,       // < 15% drop in any capability
    Moderate,   // 15-30% drop
    Severe,     // > 30% drop
    Critical,   // > 50% drop or multiple severe regressions
}

/// Tracks capability baselines and detects regression across evolution cycles
pub struct CapabilityTracker {
    /// Rolling baselines per agent (last N snapshots averaged)
    baselines: HashMap<String, CapabilityBaseline>,
    /// Configuration
    config: CapabilityConfig,
}

#[derive(Debug, Clone)]
struct CapabilityBaseline {
    snapshots: Vec<CapabilitySnapshot>,
    max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConfig {
    /// Minimum snapshots before baseline is established
    pub min_baseline_samples: usize,
    /// Maximum history to keep per agent
    pub max_history: usize,
    /// Regression thresholds (drop percentage)
    pub mild_threshold: f32,
    pub moderate_threshold: f32,
    pub severe_threshold: f32,
    pub critical_threshold: f32,
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            min_baseline_samples: 5,
            max_history: 50,
            mild_threshold: 0.15,
            moderate_threshold: 0.30,
            severe_threshold: 0.50,
            critical_threshold: 0.70,
        }
    }
}

impl CapabilityTracker {
    pub fn new(config: CapabilityConfig) -> Self {
        Self {
            baselines: HashMap::new(),
            config,
        }
    }

    /// Record a capability snapshot for an agent
    pub fn record(&mut self, snapshot: CapabilitySnapshot) {
        let entry = self.baselines
            .entry(snapshot.agent_id.clone())
            .or_insert_with(|| CapabilityBaseline {
                snapshots: Vec::new(),
                max_history: self.config.max_history,
            });

        entry.snapshots.push(snapshot);
        if entry.snapshots.len() > entry.max_history {
            entry.snapshots.remove(0);
        }
    }

    /// Compute baseline from historical snapshots
    fn compute_baseline(&self, agent_id: &str) -> Option<CapabilitySnapshot> {
        let entry = self.baselines.get(agent_id)?;
        if entry.snapshots.len() < self.config.min_baseline_samples {
            return None;
        }

        // Average of earliest snapshots (the "learned" baseline)
        let sample_count = self.config.min_baseline_samples.min(entry.snapshots.len());
        let samples: Vec<_> = entry.snapshots.iter().take(sample_count).collect();

        Some(CapabilitySnapshot {
            agent_id: agent_id.to_string(),
            tick: samples.last().map(|s| s.tick).unwrap_or(0),
            survival: samples.iter().map(|s| s.survival).sum::<f32>() / sample_count as f32,
            social: samples.iter().map(|s| s.social).sum::<f32>() / sample_count as f32,
            exploration: samples.iter().map(|s| s.exploration).sum::<f32>() / sample_count as f32,
            cognition: samples.iter().map(|s| s.cognition).sum::<f32>() / sample_count as f32,
            economy: samples.iter().map(|s| s.economy).sum::<f32>() / sample_count as f32,
            personality_stability: samples.iter().map(|s| s.personality_stability).sum::<f32>() / sample_count as f32,
        })
    }

    /// Detect capability regression for an agent
    pub fn detect_regression(&self, agent_id: &str, current_tick: u64) -> Option<RegressionReport> {
        let entry = self.baselines.get(agent_id)?;
        let current = entry.snapshots.last()?;
        let baseline = self.compute_baseline(agent_id)?;

        let mut regressed = Vec::new();

        let capabilities = [
            ("survival", baseline.survival, current.survival),
            ("social", baseline.social, current.social),
            ("exploration", baseline.exploration, current.exploration),
            ("cognition", baseline.cognition, current.cognition),
            ("economy", baseline.economy, current.economy),
        ];

        for (name, base, curr) in &capabilities {
            if *base > 0.01 {
                let drop = (base - curr) / base;
                if drop > self.config.mild_threshold {
                    regressed.push(RegressedCapability {
                        name: name.to_string(),
                        baseline: *base,
                        current: *curr,
                        drop_pct: drop,
                    });
                }
            }
        }

        if regressed.is_empty() {
            return None;
        }

        let max_drop = regressed.iter().map(|r| r.drop_pct).fold(0.0f32, f32::max);
        let severity = if max_drop > self.config.critical_threshold {
            RegressionSeverity::Critical
        } else if max_drop > self.config.severe_threshold {
            RegressionSeverity::Severe
        } else if max_drop > self.config.moderate_threshold {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Mild
        };

        Some(RegressionReport {
            agent_id: agent_id.to_string(),
            tick: current_tick,
            regressed_capabilities: regressed,
            severity,
        })
    }

    /// Get the latest snapshot for an agent
    pub fn latest_snapshot(&self, agent_id: &str) -> Option<&CapabilitySnapshot> {
        self.baselines.get(agent_id)?.snapshots.last()
    }

    /// Get all agent IDs being tracked
    pub fn tracked_agents(&self) -> Vec<&str> {
        self.baselines.keys().map(|s| s.as_str()).collect()
    }

    /// Remove tracking data for an agent (e.g., when it dies)
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.baselines.remove(agent_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(agent_id: &str, tick: u64, survival: f32) -> CapabilitySnapshot {
        CapabilitySnapshot {
            agent_id: agent_id.to_string(),
            tick,
            survival,
            social: 0.5,
            exploration: 0.5,
            cognition: 0.5,
            economy: 0.5,
            personality_stability: 0.9,
        }
    }

    #[test]
    fn tracker_records_snapshots() {
        let mut tracker = CapabilityTracker::new(CapabilityConfig::default());
        for i in 0..10 {
            tracker.record(make_snapshot("agent_0", i, 0.8));
        }
        assert!(tracker.latest_snapshot("agent_0").is_some());
    }

    #[test]
    fn no_regression_with_stable_capabilities() {
        let mut tracker = CapabilityTracker::new(CapabilityConfig::default());
        for i in 0..10 {
            tracker.record(make_snapshot("agent_0", i, 0.8));
        }
        let report = tracker.detect_regression("agent_0", 10);
        assert!(report.is_none());
    }

    #[test]
    fn detects_severe_regression() {
        let mut config = CapabilityConfig::default();
        config.min_baseline_samples = 3;
        let mut tracker = CapabilityTracker::new(config);

        // Establish baseline at 0.8
        for i in 0..5 {
            tracker.record(make_snapshot("agent_0", i, 0.8));
        }
        // Drop to 0.3 (> 50% drop → Severe or Critical)
        tracker.record(make_snapshot("agent_0", 5, 0.3));

        let report = tracker.detect_regression("agent_0", 5).unwrap();
        assert!(matches!(report.severity, RegressionSeverity::Severe | RegressionSeverity::Critical));
        assert_eq!(report.regressed_capabilities.len(), 1);
        assert_eq!(report.regressed_capabilities[0].name, "survival");
    }

    #[test]
    fn no_regression_below_threshold() {
        let mut config = CapabilityConfig::default();
        config.min_baseline_samples = 3;
        let mut tracker = CapabilityTracker::new(config);

        for i in 0..5 {
            tracker.record(make_snapshot("agent_0", i, 0.8));
        }
        // Small drop (0.8 → 0.72 = 10% drop, below 15% mild threshold)
        tracker.record(make_snapshot("agent_0", 5, 0.72));

        let report = tracker.detect_regression("agent_0", 5);
        assert!(report.is_none());
    }

    #[test]
    fn remove_agent_cleans_up() {
        let mut tracker = CapabilityTracker::new(CapabilityConfig::default());
        tracker.record(make_snapshot("agent_0", 0, 0.8));
        assert!(tracker.latest_snapshot("agent_0").is_some());
        tracker.remove_agent("agent_0");
        assert!(tracker.latest_snapshot("agent_0").is_none());
    }
}
