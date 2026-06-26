use std::collections::{HashMap, VecDeque};

use super::self_introspection::{CorrectiveAction, DefectPattern, DiagnosticSnapshot};

/// RL state vector extracted from consciousness metrics.
///
/// This is the "S" in the MDP tuple (S, A, R, S').
/// Each field is a normalized numeric feature that can feed
/// into value/advantage estimation or curiosity-driven exploration.
#[derive(Debug, Clone)]
pub struct MirrorState {
    pub cycle: u64,
    pub handler_count: usize,
    pub hot_ratio: f64,
    pub warm_ratio: f64,
    pub cold_ratio: f64,
    pub total_calls: u64,
    pub pending_introspection_actions: usize,
    pub negentropy: f64,
    pub cognitive_load: f64,
    pub arousal: f64,
    pub coherence: f64,
    pub active_handler_count: usize,
}

impl MirrorState {
    /// Extract a fixed-length feature vector for downstream RL.
    /// Length: 12
    pub fn to_feature_vec(&self) -> Vec<f64> {
        vec![
            self.cycle as f64,
            self.handler_count as f64,
            self.hot_ratio,
            self.warm_ratio,
            self.cold_ratio,
            self.total_calls as f64,
            self.pending_introspection_actions as f64,
            self.negentropy,
            self.cognitive_load,
            self.arousal,
            self.coherence,
            self.active_handler_count as f64,
        ]
    }

    /// Build a MirrorState from a DiagnosticSnapshot and consciousness metrics.
    pub fn build_state(
        snapshot: &DiagnosticSnapshot,
        handler_count: usize,
        hot_ratio: f64,
        warm_ratio: f64,
        cold_ratio: f64,
        total_calls: u64,
        negentropy: f64,
        cognitive_load: f64,
        arousal: f64,
        coherence: f64,
    ) -> Self {
        MirrorState {
            cycle: snapshot.cycle,
            handler_count,
            hot_ratio,
            warm_ratio,
            cold_ratio,
            total_calls,
            pending_introspection_actions: snapshot.pending_actions,
            negentropy,
            cognitive_load,
            arousal,
            coherence,
            active_handler_count: snapshot.active_handler_count,
        }
    }
}

/// An RL transition: (state, action, reward, next_state).
///
/// Each time a corrective action is executed, a transition is recorded.
/// The reward is the ΔNegentropy between pre- and post-action states.
#[derive(Debug, Clone)]
pub struct MirrorTransition {
    pub state: MirrorState,
    pub action: DefectPattern,
    pub action_priority: u8,
    pub reward: f64,
    pub next_state: MirrorState,
    pub cycle: u64,
    pub executed: bool,
}

impl MirrorTransition {
    pub fn description(&self) -> String {
        let action_desc = format!("{:?}", self.action);
        format!(
            "cycle={} action={} reward={:+.4} priority={} executed={}",
            self.cycle, action_desc, self.reward, self.action_priority, self.executed
        )
    }
}

/// Action-value estimate for a particular defect pattern.
#[derive(Debug, Clone)]
pub struct ActionValue {
    pub pattern_key: String,
    pub estimated_value: f64,
    pub visit_count: u32,
    pub avg_reward: f64,
}

/// Operational Mirror — bridges introspection defects to RL state/action/reward.
///
/// This turns the IntrospectionEngine from a rule-based detector into
/// an RL agent: each CorrectiveAction is an action in an MDP where:
///   - State  = MirrorState (consciousness + snapshot metrics)
///   - Action = CorrectiveAction pattern type
///   - Reward = ΔNegentropy (improvement in epistemic order)
///
/// Over time, the mirror learns which actions produce positive rewards
/// under which states, enabling value-guided auto-correction.
pub struct OperationalMirror {
    /// FIFO buffer of recent transitions
    pub transitions: VecDeque<MirrorTransition>,
    pub max_transitions: usize,
    /// Cumulative reward summed over all transitions
    pub cumulative_reward: f64,
    /// Per-pattern action-value estimates
    action_values: HashMap<String, ActionValue>,
}

impl OperationalMirror {
    pub fn new() -> Self {
        Self {
            transitions: VecDeque::with_capacity(200),
            max_transitions: 200,
            cumulative_reward: 0.0,
            action_values: HashMap::new(),
        }
    }

    /// Record a transition when a corrective action is about to be executed.
    ///
    /// `state` and `next_state` capture consciousness metrics before and after.
    /// `reward`` is the ΔNegentropy (or any scalar improvement signal).
    pub fn record_transition(
        &mut self,
        state: MirrorState,
        action: &CorrectiveAction,
        reward: f64,
        next_state: MirrorState,
    ) {
        let transition = MirrorTransition {
            state,
            action: action.pattern.clone(),
            action_priority: action.priority,
            reward,
            next_state,
            cycle: action.detected_at_cycle,
            executed: action.executed,
        };
        self.transitions.push_back(transition.clone());
        while self.transitions.len() > self.max_transitions {
            self.transitions.pop_front();
        }
        self.cumulative_reward += reward;

        // Update action-value estimates
        let key = format!("{:?}", action.pattern);
        let entry = self.action_values.entry(key).or_insert(ActionValue {
            pattern_key: format!("{:?}", action.pattern),
            estimated_value: 0.0,
            visit_count: 0,
            avg_reward: 0.0,
        });
        entry.visit_count += 1;
        // Online exponential moving average
        let alpha = 0.3;
        entry.avg_reward = entry.avg_reward * (1.0 - alpha) + reward * alpha;
        entry.estimated_value = entry.avg_reward;
    }

    /// Get action-value estimates for all observed patterns.
    pub fn action_values(&self) -> Vec<ActionValue> {
        let mut values: Vec<ActionValue> = self.action_values.values().cloned().collect();
        values.sort_by(|a, b| {
            b.estimated_value
                .partial_cmp(&a.estimated_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        values
    }

    /// Estimate the value of executing a particular action in the given state.
    /// Returns (estimated_value, visit_count) or (0.0, 0) for never-before-seen actions.
    pub fn estimate_value(&self, action: &CorrectiveAction, _state: &MirrorState) -> (f64, u32) {
        let key = format!("{:?}", action.pattern);
        match self.action_values.get(&key) {
            Some(av) => (av.estimated_value, av.visit_count),
            None => (0.0, 0),
        }
    }

    /// Number of transitions recorded.
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    /// Recent transitions as a report string.
    pub fn report(&self) -> String {
        let recent: Vec<String> = self
            .transitions
            .iter()
            .rev()
            .take(5)
            .map(|t| t.description())
            .collect();
        let top_actions: Vec<String> = self
            .action_values()
            .iter()
            .take(5)
            .map(|av| {
                format!(
                    "{}: value={:.4} visits={} avg_r={:.4}",
                    av.pattern_key, av.estimated_value, av.visit_count, av.avg_reward
                )
            })
            .collect();
        format!(
            "Mirror: {} transitions cum_r={:.4} | actions: [{}] | recent: [{}]",
            self.transitions.len(),
            self.cumulative_reward,
            top_actions.join(", "),
            recent.join("; "),
        )
    }
}

impl Default for OperationalMirror {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute ΔNegentropy reward from a negentropy delta.
/// Clamps to [-1.0, 1.0] and applies sigmoid squashing.
pub fn compute_reward(negentropy_before: f64, negentropy_after: f64) -> f64 {
    let delta = negentropy_after - negentropy_before;
    // Sigmoid squashing: maps (-∞, +∞) → (-1, 1)
    let clamped = delta.clamp(-10.0, 10.0);
    (clamped / (1.0 + clamped.abs())) * 1.0
}

/// Aggregate state statistics over multiple transitions.
#[derive(Debug, Clone, Default)]
pub struct MirrorStats {
    pub total_transitions: usize,
    pub cumulative_reward: f64,
    pub unique_actions_seen: usize,
    pub best_avg_action: String,
    pub best_avg_value: f64,
    pub worst_avg_action: String,
    pub worst_avg_value: f64,
}

impl OperationalMirror {
    pub fn stats(&self) -> MirrorStats {
        let action_vals = self.action_values();
        let unique = action_vals.len();
        let best = action_vals.first();
        let worst = action_vals.last();
        MirrorStats {
            total_transitions: self.transitions.len(),
            cumulative_reward: self.cumulative_reward,
            unique_actions_seen: unique,
            best_avg_action: best.map(|a| a.pattern_key.clone()).unwrap_or_default(),
            best_avg_value: best.map(|a| a.estimated_value).unwrap_or(0.0),
            worst_avg_action: worst.map(|a| a.pattern_key.clone()).unwrap_or_default(),
            worst_avg_value: worst.map(|a| a.estimated_value).unwrap_or(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(cycle: u64) -> DiagnosticSnapshot {
        DiagnosticSnapshot {
            cycle,
            active_handler_count: 50,
            pending_actions: 3,
            component_sizes: vec![("memory".into(), 500)],
            handler_frequencies: vec![("test".into(), 10)],
        }
    }

    fn make_state(snapshot: &DiagnosticSnapshot) -> MirrorState {
        MirrorState::build_state(snapshot, 84, 0.6, 0.25, 0.15, 1000, 0.5, 0.3, 0.7, 0.8)
    }

    fn make_action(priority: u8) -> CorrectiveAction {
        CorrectiveAction {
            pattern: DefectPattern::AccumulationWithoutPruning {
                component: "memory".into(),
                size: 500,
            },
            suggestion: "prune memory".into(),
            priority,
            detected_at_cycle: 1,
            executed: false,
        }
    }

    #[test]
    fn test_record_transition() {
        let mut mirror = OperationalMirror::new();
        let snap = make_snapshot(1);
        let state = make_state(&snap);
        let next_state = make_state(&DiagnosticSnapshot {
            cycle: 2,
            ..make_snapshot(2)
        });
        let action = make_action(120);

        mirror.record_transition(state, &action, 0.5, next_state);
        assert_eq!(mirror.transition_count(), 1);
        assert!((mirror.cumulative_reward - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_reward_positive_delta() {
        let r = compute_reward(0.3, 0.8);
        assert!(r > 0.0, "positive delta should yield positive reward");
    }

    #[test]
    fn test_compute_reward_negative_delta() {
        let r = compute_reward(0.8, 0.3);
        assert!(r < 0.0, "negative delta should yield negative reward");
    }

    #[test]
    fn test_compute_reward_zero_delta() {
        let r = compute_reward(0.5, 0.5);
        assert!((r).abs() < 1e-6, "zero delta should yield zero reward");
    }

    #[test]
    fn test_action_values_updated_on_record() {
        let mut mirror = OperationalMirror::new();
        let snap = make_snapshot(1);
        let state = make_state(&snap);
        let next = make_state(&DiagnosticSnapshot {
            cycle: 2,
            ..make_snapshot(2)
        });

        mirror.record_transition(state.clone(), &make_action(120), 0.3, next.clone());
        mirror.record_transition(state.clone(), &make_action(120), 0.7, next.clone());

        let values = mirror.action_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].visit_count, 2);
        // EMA: 0.0 * 0.7 + 0.3 * 0.3 = 0.09, then 0.09 * 0.7 + 0.7 * 0.3 = 0.273
        assert!((values[0].avg_reward - 0.273).abs() < 0.01);
    }

    #[test]
    fn test_multiple_action_types() {
        let mut mirror = OperationalMirror::new();
        let state = make_state(&make_snapshot(1));
        let next = make_state(&make_snapshot(2));

        let a1 = CorrectiveAction {
            pattern: DefectPattern::OverDiagnosis {
                handler: "h1".into(),
                snapshot_count: 10,
            },
            suggestion: "reduce polling".into(),
            priority: 100,
            detected_at_cycle: 1,
            executed: false,
        };
        let a2 = CorrectiveAction {
            pattern: DefectPattern::ExcessiveProbing {
                pattern: "test".into(),
                count: 20,
            },
            suggestion: "batch dispatch".into(),
            priority: 120,
            detected_at_cycle: 1,
            executed: false,
        };

        mirror.record_transition(state.clone(), &a1, 0.2, next.clone());
        mirror.record_transition(state.clone(), &a2, 0.8, next.clone());

        assert_eq!(mirror.action_values().len(), 2);
    }

    #[test]
    fn test_estimate_value_unknown_action() {
        let mirror = OperationalMirror::new();
        let action = make_action(100);
        let state = make_state(&make_snapshot(1));
        let (val, visits) = mirror.estimate_value(&action, &state);
        assert_eq!(val, 0.0);
        assert_eq!(visits, 0);
    }

    #[test]
    fn test_feature_vec_length() {
        let snap = make_snapshot(1);
        let state = make_state(&snap);
        assert_eq!(state.to_feature_vec().len(), 12);
    }

    #[test]
    fn test_stats_aggregation() {
        let mut mirror = OperationalMirror::new();
        let state = make_state(&make_snapshot(1));
        let next = make_state(&make_snapshot(2));
        mirror.record_transition(state, &make_action(120), 0.5, next);
        let stats = mirror.stats();
        assert_eq!(stats.total_transitions, 1);
        assert!((stats.cumulative_reward - 0.5).abs() < 1e-6);
        assert_eq!(stats.unique_actions_seen, 1);
    }

    #[test]
    fn test_max_transitions_respected() {
        let mut mirror = OperationalMirror::new();
        mirror.max_transitions = 5;
        let state = make_state(&make_snapshot(0));
        let next = make_state(&make_snapshot(0));
        for i in 0..10 {
            mirror.record_transition(
                state.clone(),
                &CorrectiveAction {
                    pattern: DefectPattern::OverDiagnosis {
                        handler: i.to_string(),
                        snapshot_count: i,
                    },
                    suggestion: "test".into(),
                    priority: 100,
                    detected_at_cycle: i as u64,
                    executed: false,
                },
                0.1,
                next.clone(),
            );
        }
        assert_eq!(mirror.transition_count(), 5);
    }

    #[test]
    fn test_report_format() {
        let mut mirror = OperationalMirror::new();
        let state = make_state(&make_snapshot(1));
        let next = make_state(&make_snapshot(2));
        mirror.record_transition(state, &make_action(120), 0.5, next);
        let report = mirror.report();
        assert!(report.contains("Mirror:"));
        assert!(report.contains("transitions"));
    }
}
