use crate::core::nt_core_hex::FullReasoningState;

/// EWHR bridge connecting E8 state trajectory analysis to hypothesis generation.
pub struct E8EwhrBridge {
    pub enabled: bool,
    pub min_confidence: f64,
}

impl E8EwhrBridge {
    pub fn new() -> Self {
        Self { enabled: true, min_confidence: 0.3 }
    }

    pub fn analyze_trajectory(
        &self,
        trajectory: &[FullReasoningState],
        task: &str,
    ) -> Vec<String> {
        if !self.enabled || trajectory.len() < 2 {
            return vec![];
        }

        let mut findings = Vec::new();

        // 1. Detect repeating patterns (same state repeated 2+ times consecutively)
        let mut run_start = 0;
        for i in 1..=trajectory.len() {
            if i == trajectory.len()
                || trajectory[i].mode.0 != trajectory[run_start].mode.0
                || trajectory[i].meta.0 != trajectory[run_start].meta.0
            {
                let run_len = i - run_start;
                if run_len >= 2 {
                    let sig = trajectory[run_start].signature();
                    findings.push(format!(
                        "Stuck state: mode={} meta={} repeated {} times consecutively (sig={}) — suggests deterministic loop",
                        trajectory[run_start].mode.0,
                        trajectory[run_start].meta.0,
                        run_len,
                        sig,
                    ));
                }
                run_start = i;
            }
        }

        // 2. Detect oscillating patterns (ABAB...)
        if trajectory.len() >= 4 {
            let mut osc_start = None;
            for i in 0..trajectory.len() - 2 {
                let a = trajectory[i].mode.0;
                let b = trajectory[i + 1].mode.0;
                if a != b
                    && trajectory[i + 2].mode.0 == a
                    && (i + 3 >= trajectory.len() || trajectory[i + 3].mode.0 == b)
                {
                    if osc_start.is_none() {
                        osc_start = Some(i);
                    }
                } else if let Some(start) = osc_start.take() {
                    let span = i - start + 1;
                    if span >= 4 {
                        findings.push(format!(
                            "Oscillation: modes {}↔{} over {} steps (positions {}-{}) — suggests indecision between two strategies",
                            trajectory[start].mode.0,
                            trajectory[start + 1].mode.0,
                            span,
                            start,
                            i,
                        ));
                    }
                }
            }
            if let Some(start) = osc_start {
                let span = trajectory.len() - start;
                if span >= 4 {
                    findings.push(format!(
                        "Oscillation (ongoing): modes {}↔{} over {} steps — active indecision",
                        trajectory[start].mode.0,
                        trajectory[start + 1].mode.0,
                        span,
                    ));
                }
            }
        }

        // 3. Task-specific trajectory summary
        let mode_seq: Vec<u8> = trajectory.iter().map(|s| s.mode.0).collect();
        findings.push(format!(
            "Task '{}': {} steps, modes {:?}, unique modes {}, trajectory length {}",
            task,
            trajectory.len(),
            mode_seq,
            {
                let mut unique = mode_seq.clone();
                unique.sort();
                unique.dedup();
                unique.len()
            },
            trajectory.len(),
        ));

        findings
    }
}

impl Default for E8EwhrBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hex::{FullReasoningState, MetaState, ReasoningHexagram};

    fn make_state(mode: u8, meta: u8) -> FullReasoningState {
        FullReasoningState {
            mode: ReasoningHexagram::new(mode),
            meta: MetaState(meta),
        }
    }

    #[test]
    fn test_bridge_disabled_returns_empty() {
        let bridge = E8EwhrBridge { enabled: false, min_confidence: 0.3 };
        let traj = vec![make_state(1, 1), make_state(2, 2)];
        let findings = bridge.analyze_trajectory(&traj, "test");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_empty_trajectory_returns_empty() {
        let bridge = E8EwhrBridge::new();
        assert!(bridge.analyze_trajectory(&[], "test").is_empty());
    }

    #[test]
    fn test_single_step_returns_empty() {
        let bridge = E8EwhrBridge::new();
        let traj = vec![make_state(1, 1)];
        let findings = bridge.analyze_trajectory(&traj, "test");
        assert!(findings.is_empty(), "single step (<2) returns empty");
    }

    #[test]
    fn test_stuck_state_detected() {
        let bridge = E8EwhrBridge::new();
        let traj = vec![
            make_state(5, 1),
            make_state(5, 1),
            make_state(5, 1),
            make_state(6, 0),
        ];
        let findings = bridge.analyze_trajectory(&traj, "stuck test");
        let stuck: Vec<&str> = findings.iter().filter(|f| f.contains("Stuck state")).map(|s| s.as_str()).collect();
        assert_eq!(stuck.len(), 1);
        assert!(stuck[0].contains("mode=5"));
    }

    #[test]
    fn test_multiple_stuck_runs_detected() {
        let bridge = E8EwhrBridge::new();
        let traj = vec![
            make_state(1, 0), make_state(1, 0),
            make_state(2, 0),
            make_state(3, 0), make_state(3, 0), make_state(3, 0),
        ];
        let findings = bridge.analyze_trajectory(&traj, "multi stuck");
        let stuck: Vec<&str> = findings.iter().filter(|f| f.contains("Stuck state")).map(|s| s.as_str()).collect();
        assert_eq!(stuck.len(), 2);
    }

    #[test]
    fn test_oscillation_detected() {
        let bridge = E8EwhrBridge::new();
        let traj = vec![
            make_state(10, 0), make_state(11, 0),
            make_state(10, 0), make_state(11, 0),
        ];
        let findings = bridge.analyze_trajectory(&traj, "osc test");
        let osc: Vec<&str> = findings.iter().filter(|f| f.contains("Oscillation")).map(|s| s.as_str()).collect();
        assert!(!osc.is_empty(), "Should detect ABAB oscillation");
    }

    #[test]
    fn test_oscillation_requires_minimum_length() {
        let bridge = E8EwhrBridge::new();
        let traj = vec![make_state(10, 0), make_state(11, 0), make_state(10, 0)];
        let findings = bridge.analyze_trajectory(&traj, "short osc");
        let osc: Vec<&str> = findings.iter().filter(|f| f.contains("Oscillation")).map(|s| s.as_str()).collect();
        assert!(osc.is_empty(), "3-step is too short for oscillation");
    }

    #[test]
    fn test_trajectory_summary_always_present() {
        let bridge = E8EwhrBridge::new();
        let traj = vec![make_state(1, 0), make_state(2, 0), make_state(3, 0)];
        let findings = bridge.analyze_trajectory(&traj, "summary check");
        let summary: Vec<&str> = findings.iter().filter(|f| f.contains("Task")).map(|s| s.as_str()).collect();
        assert_eq!(summary.len(), 1);
        assert!(summary[0].contains("summary check"));
        assert!(summary[0].contains("3 steps"));
    }
}

