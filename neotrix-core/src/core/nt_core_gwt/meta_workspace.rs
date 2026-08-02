//! Phase 9.1 — Meta-Workspace (二阶工作空间 · 自指观察).
//!
//! CTM-AI §5 / GWA §4.1: a second-order workspace that observes the primary
//! GlobalWorkspace — its content, winning specialist behavior, activation
//! dynamics, entropy health, and sparse-gate choices — and registers
//! meta-observations of the form:
//!
//!   - "专家 A 激活频率过高" (expert A over-activates)
//!   - "工作空间熵异常" (workspace entropy anomalous)
//!   - "gate 收敛到同一专家组" (gate stuck on one expert group)
//!
//! These meta-observations feed back as context into `InnerSpeech`, giving the
//! consciousness layer the ability to reason about its own behavior (self-talk
//! about itself rather than just about the task).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Number of recent cycles the meta-workspace retains for pattern detection.
pub const META_WINDOW: usize = 16;
/// Threshold (fraction of window) above which an expert is "over-activating".
pub const OVERACTIVATE_FRAC: f64 = 0.4;
/// Minimum window fill before over-activation is a meaningful signal (avoids
/// firing on the first few cycles where every winner is trivially "frequent").
pub const MIN_SAMPLES: usize = 8;

/// A single second-order observation about the primary workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaObservation {
    /// Machine-readable tag (e.g. "overactivation", "entropy_anomaly").
    pub tag: String,
    /// Natural-language description of the observation.
    pub message: String,
    /// Quantitative signal strength (0..1).
    pub severity: f64,
    /// Cycle index when observed.
    pub cycle: u64,
}

/// Observed state of the primary workspace fed into the meta-workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryObservation {
    /// Index of the winning specialist this cycle.
    pub winner: usize,
    /// Name of the winning specialist.
    pub winner_name: String,
    /// Shannon entropy of the activation distribution.
    pub entropy: f64,
    /// Sparse-gate active expert indices this cycle (if any).
    pub gated_experts: Vec<usize>,
    /// Total number of specialists.
    pub specialist_count: usize,
}

/// Phase 9.1 — the observer workspace (second-order consciousness loop).
///
/// Maintains a rolling window of primary-workspace observations, detects
/// behavioral patterns (over-activation, entropy anomalies, gate fixation),
/// and produces meta-observations that can be injected as `InnerSpeech` context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWorkspace {
    /// Rolling window of recent primary observations (most recent first).
    pub history: VecDeque<PrimaryObservation>,
    /// Meta-observations generated so far (bounded).
    pub observations: VecDeque<MetaObservation>,
    /// Per-expert activation counts over the window (for over-activation).
    pub activation_counts: Vec<u64>,
    /// Cycle counter.
    pub cycle: u64,
    /// Maximum history window.
    pub window: usize,
}

impl Default for MetaWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaWorkspace {
    /// Create a meta-workspace with the default observation window.
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(META_WINDOW),
            observations: VecDeque::with_capacity(32),
            activation_counts: Vec::new(),
            cycle: 0,
            window: META_WINDOW,
        }
    }

    /// Register one primary-workspace cycle and detect second-order patterns.
    pub fn observe(&mut self, obs: PrimaryObservation) -> Vec<MetaObservation> {
        self.cycle += 1;
        self.history.push_front(obs);
        if self.history.len() > self.window {
            self.history.pop_back();
        }
        self.detect()
    }

    /// Detect meta-patterns over the current window and register observations.
    fn detect(&mut self) -> Vec<MetaObservation> {
        let mut fresh = Vec::new();
        let n = self.history.len() as f64;
        if n < 2.0 {
            return fresh;
        }

        // 1) Per-expert activation frequency over the window.
        let count = self.specialist_count();
        self.activation_counts = vec![0; count.max(1)];
        for h in &self.history {
            if (h.winner as usize) < self.activation_counts.len() {
                self.activation_counts[h.winner as usize] += 1;
            }
        }
        let mut overactivation: Vec<MetaObservation> = Vec::new();
        if self.history.len() >= MIN_SAMPLES {
            for (expert, &c) in self.activation_counts.iter().enumerate() {
                let frac = c as f64 / n;
                if frac >= OVERACTIVATE_FRAC {
                    let name = self
                        .history
                        .iter()
                        .find(|h| h.winner as usize == expert)
                        .map(|h| h.winner_name.clone())
                        .unwrap_or_else(|| format!("specialist_{expert}"));
                    overactivation.push(MetaObservation {
                        tag: "overactivation".to_string(),
                        message: format!(
                            "[meta] 专家 {name} ({expert}) 激活频率过高: {:.0}% 的窗口周期",
                            frac * 100.0
                        ),
                        severity: ((frac - OVERACTIVATE_FRAC) / (1.0 - OVERACTIVATE_FRAC)).clamp(0.0, 1.0),
                        cycle: self.cycle,
                    });
                }
            }
            for mo in overactivation {
                self.push_observation(mo.clone());
                fresh.push(mo);
            }
        }

        // 2) Entropy anomaly: mean window entropy far from healthy band.
        let mean_entropy = self
            .history
            .iter()
            .map(|h| h.entropy)
            .sum::<f64>()
            / n;
        if mean_entropy < 0.5 || mean_entropy > 2.5 {
            let mo = MetaObservation {
                tag: "entropy_anomaly".to_string(),
                message: format!(
                    "[meta] 工作空间熵异常 (mean={mean_entropy:.2}) — 注意状态 {}",
                    if mean_entropy < 0.5 { "趋固定 (fixation)" } else { "发散 (scattered)" }
                ),
                severity: ((mean_entropy - 0.5).max(2.5 - mean_entropy)).clamp(0.0, 1.0),
                cycle: self.cycle,
            };
            self.push_observation(mo.clone());
            fresh.push(mo);
        }

        // 3) Gate fixation: sparse gate repeatedly selects the same top group.
        if self.history.len() >= 3 {
            let first = &self.history[0];
            let all_same = self
                .history
                .iter()
                .take(3)
                .all(|h| h.gated_experts == first.gated_experts)
                && !first.gated_experts.is_empty();
            if all_same {
                let mo = MetaObservation {
                    tag: "gate_fixation".to_string(),
                    message: format!(
                        "[meta] 稀疏门控连续收敛到同一专家组 {:?} — 探索风险",
                        first.gated_experts
                    ),
                    severity: 0.7,
                    cycle: self.cycle,
                };
                self.push_observation(mo.clone());
                fresh.push(mo);
            }
        }

        fresh
    }

    /// Number of specialists observed (from the most recent observation).
    fn specialist_count(&self) -> usize {
        self.history.front().map(|h| h.specialist_count).unwrap_or(0)
    }

    /// Push an observation, keeping the ring bounded.
    fn push_observation(&mut self, mo: MetaObservation) {
        self.observations.push_front(mo);
        while self.observations.len() > 32 {
            self.observations.pop_back();
        }
    }

    /// Render all pending meta-observations as InnerSpeech context lines.
    ///
    /// Called before inner speech so the self-talk can reference its own state.
    pub fn context_block(&self, limit: usize) -> String {
        self.observations
            .iter()
            .take(limit)
            .map(|o| o.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Total distinct observations emitted so far.
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Clear all observation and history state.
    pub fn reset(&mut self) {
        self.history.clear();
        self.observations.clear();
        self.activation_counts.clear();
        self.cycle = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(winner: usize, name: &str, entropy: f64, gated: Vec<usize>) -> PrimaryObservation {
        PrimaryObservation {
            winner,
            winner_name: name.to_string(),
            entropy,
            gated_experts: gated,
            specialist_count: 8,
        }
    }

    #[test]
    fn test_empty_window_no_observations() {
        let mut mw = MetaWorkspace::new();
        let out = mw.observe(obs(0, "e0", 1.0, vec![0, 1]));
        assert!(out.is_empty());
    }

    #[test]
    fn test_overactivation_detected() {
        let mut mw = MetaWorkspace::new();
        let mut out = Vec::new();
        for _ in 0..10 {
            out.extend(mw.observe(obs(2, "Planner", 1.0, vec![2])));
        }
        assert!(out.iter().any(|o| o.tag == "overactivation"));
        assert!(out.iter().any(|o| o.message.contains("Planner")));
    }

    #[test]
    fn test_entropy_anomaly_detected_low() {
        let mut mw = MetaWorkspace::new();
        let mut out = Vec::new();
        for _ in 0..6 {
            out.extend(mw.observe(obs(0, "e0", 0.2, vec![0])));
        }
        assert!(out.iter().any(|o| o.tag == "entropy_anomaly"));
        assert!(out.iter().any(|o| o.message.contains("固定")));
    }

    #[test]
    fn test_gate_fixation_detected() {
        let mut mw = MetaWorkspace::new();
        let mut out = Vec::new();
        for _ in 0..5 {
            out.extend(mw.observe(obs(1, "e1", 1.2, vec![3, 4])));
        }
        assert!(out.iter().any(|o| o.tag == "gate_fixation"));
    }

    #[test]
    fn test_no_false_gate_fixation_on_varying_gates() {
        let mut mw = MetaWorkspace::new();
        let mut out = Vec::new();
        for i in 0..5 {
            out.extend(mw.observe(obs(1, "e1", 1.2, vec![i % 4, (i + 1) % 4])));
        }
        assert!(!out.iter().any(|o| o.tag == "gate_fixation"));
    }

    #[test]
    fn test_window_bounded() {
        let mut mw = MetaWorkspace::new();
        for i in 0..40 {
            mw.observe(obs(i % 8, "e", 1.0, vec![]));
        }
        assert!(mw.history.len() <= META_WINDOW);
    }

    #[test]
    fn test_healthy_window_no_observations() {
        let mut mw = MetaWorkspace::new();
        let mut out = Vec::new();
        // Rotating winners, balanced entropy, varying gates → no alarms.
        for i in 0..20 {
            let e = 1.0 + (i % 3) as f64 * 0.3;
            out.extend(mw.observe(obs(i % 8, "e", e, vec![i % 4])));
        }
        assert!(!out.iter().any(|o| o.tag == "overactivation"));
        assert!(!out.iter().any(|o| o.tag == "entropy_anomaly"));
        assert!(!out.iter().any(|o| o.tag == "gate_fixation"));
    }

    #[test]
    fn test_context_block_renders_observations() {
        let mut mw = MetaWorkspace::new();
        for _ in 0..8 {
            mw.observe(obs(2, "Planner", 0.2, vec![2]));
        }
        let ctx = mw.context_block(10);
        assert!(ctx.contains("[meta]"));
        assert!(mw.observation_count() > 0);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut mw = MetaWorkspace::new();
        for _ in 0..8 {
            mw.observe(obs(2, "Planner", 0.2, vec![2]));
        }
        assert!(mw.observation_count() > 0);
        mw.reset();
        assert_eq!(mw.observation_count(), 0);
        assert!(mw.history.is_empty());
        assert_eq!(mw.cycle, 0);
    }
}
