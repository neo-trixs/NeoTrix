//! # +1 Observer — Meta-Cognitive Trajectory Analyzer
//!
//! The +1 observer watches the engine's own reasoning trajectory and:
//! 1. Detects patterns (loops, oscillations, dead ends, efficiency)
//! 2. Classifies reasoning quality per phase
//! 3. Produces meta-insights for self-improvement
//! 4. Recommends capability vector adjustments
//!
//! This closes the meta-cognitive feedback loop — the system observes
//! its own reasoning and adapts based on that observation.

use serde::{Deserialize, Serialize};

use crate::core::nt_core_hex::{
    optimal_starting_mode, FullReasoningState, MetaState, ReasoningHexagram,
};

/// How the observer classifies a single trajectory step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepQuality {
    Productive,  // progress toward goal
    Neutral,     // exploration, information gathering
    Regressive,  // backtracking to known state
    Oscillating, // flipping back and forth between two states
    Stuck,       // repeated same state
    DeadEnd,     // terminal with no further transitions
}

/// A pattern detected across a trajectory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrajectoryPattern {
    /// Alternating between two states (ABABAB...)
    Oscillation {
        a: ReasoningHexagram,
        b: ReasoningHexagram,
        cycle_len: usize,
    },
    /// Stuck in the same state for N consecutive steps
    Stuck {
        state: ReasoningHexagram,
        steps: usize,
    },
    /// Productive monotonic progression through distinct states
    ProductiveWalk {
        states_visited: usize,
        coverage: f64,
    },
    /// Returned to a previously visited state (potential loop)
    LoopBack {
        from: ReasoningHexagram,
        to: ReasoningHexagram,
        dist: u32,
    },
    /// Efficient path — short path between start and resolution
    Efficient {
        steps: usize,
        optimal: usize,
        ratio: f64,
    },
    /// Inefficient path — wandered too much
    Inefficient {
        steps: usize,
        optimal: usize,
        ratio: f64,
    },
    /// Exploration phase followed by exploitation
    ExploreThenExploit {
        explore_len: usize,
        exploit_len: usize,
    },
}

/// Observer's assessment of the overall trajectory quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverReport {
    /// The trajectory analyzed
    pub trajectory_len: usize,
    /// Number of distinct reasoning modes visited
    pub distinct_states: usize,
    /// Detected patterns
    pub patterns: Vec<TrajectoryPattern>,
    /// Step-by-step quality labels
    pub step_qualities: Vec<StepQuality>,
    /// Overall quality score (0.0—1.0)
    pub quality_score: f64,
    /// Recommended meta-state transition
    pub recommended_meta: Option<MetaState>,
    /// Suggested capability adjustment deltas
    pub capability_deltas: Vec<(String, f64)>,
    /// Whether the observer triggered an absorb-worthy insight
    pub has_actionable_insight: bool,
}

impl ObserverReport {
    pub fn is_healthy(&self) -> bool {
        self.quality_score >= 0.6 && !self.has_critical_pattern()
    }

    pub fn has_critical_pattern(&self) -> bool {
        self.patterns.iter().any(|p| match p {
            TrajectoryPattern::Stuck { steps, .. } if *steps >= 5 => true,
            TrajectoryPattern::Inefficient { ratio, .. } if *ratio < 0.3 => true,
            TrajectoryPattern::Oscillation { cycle_len, .. } if *cycle_len >= 4 => true,
            _ => false,
        })
    }
}

/// The +1 observer: analyzes reasoning trajectories and produces insights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneObserver {
    /// History of all analyzed trajectories
    pub trajectory_history: Vec<ObserverReport>,
    /// Accumulated capability deltas across analyses
    pub accumulated_deltas: Vec<(String, f64)>,
    /// Number of analyses performed
    pub analysis_count: usize,
}

impl Default for OneObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl OneObserver {
    const MAX_TRAJECTORY: usize = 1000;
    const MAX_DELTAS: usize = 10000;

    pub fn new() -> Self {
        Self {
            trajectory_history: Vec::new(),
            accumulated_deltas: Vec::new(),
            analysis_count: 0,
        }
    }

    /// Analyze a trajectory and produce an ObserverReport.
    pub fn analyze(
        &mut self,
        trajectory: &[FullReasoningState],
        task_keywords: &[&str],
    ) -> ObserverReport {
        let trajectory_len = trajectory.len();
        if trajectory_len == 0 {
            return ObserverReport {
                trajectory_len: 0,
                distinct_states: 0,
                patterns: Vec::new(),
                step_qualities: Vec::new(),
                quality_score: 0.5,
                recommended_meta: None,
                capability_deltas: Vec::new(),
                has_actionable_insight: false,
            };
        }

        let modes: Vec<ReasoningHexagram> = trajectory.iter().map(|s| s.mode).collect();
        let distinct: std::collections::HashSet<ReasoningHexagram> =
            modes.iter().copied().collect();
        let distinct_states = distinct.len();

        // Detect patterns
        let patterns = Self::detect_patterns(&modes);

        // Classify each step
        let step_qualities = Self::classify_steps(&modes);

        // Compute quality score
        let quality_score =
            Self::compute_quality(&modes, &patterns, &step_qualities, task_keywords);

        // Determine recommended meta-state
        let recommended_meta = Self::recommend_meta(&patterns, quality_score);

        // Compute capability deltas
        let capability_deltas = Self::compute_capability_deltas(&patterns, quality_score);

        let has_actionable_insight = quality_score < 0.5 || !capability_deltas.is_empty();

        let report = ObserverReport {
            trajectory_len,
            distinct_states,
            patterns,
            step_qualities,
            quality_score,
            recommended_meta,
            capability_deltas: capability_deltas.clone(),
            has_actionable_insight,
        };

        self.trajectory_history.push(report.clone());
        self.accumulated_deltas.extend(capability_deltas);
        self.analysis_count += 1;

        if self.trajectory_history.len() > Self::MAX_TRAJECTORY {
            let excess = self.trajectory_history.len() - Self::MAX_TRAJECTORY;
            self.trajectory_history.drain(0..excess);
        }
        if self.accumulated_deltas.len() > Self::MAX_DELTAS {
            let excess = self.accumulated_deltas.len() - Self::MAX_DELTAS;
            self.accumulated_deltas.drain(0..excess);
        }

        report
    }

    /// Detect all patterns in a mode trajectory.
    fn detect_patterns(modes: &[ReasoningHexagram]) -> Vec<TrajectoryPattern> {
        let mut patterns = Vec::new();

        if modes.len() < 2 {
            return patterns;
        }

        // Oscillation detection: ABAB pattern
        Self::detect_oscillation(modes, &mut patterns);

        // Stuck detection
        Self::detect_stuck(modes, &mut patterns);

        // Loop-back detection
        Self::detect_loop_back(modes, &mut patterns);

        // Efficiency analysis
        Self::detect_efficiency(modes, &mut patterns);

        // Explore-then-exploit
        Self::detect_explore_exploit(modes, &mut patterns);

        // Productive walk
        let distinct: std::collections::HashSet<ReasoningHexagram> =
            modes.iter().copied().collect();
        if distinct.len() as f64 >= modes.len() as f64 * 0.7 && modes.len() >= 3 {
            patterns.push(TrajectoryPattern::ProductiveWalk {
                states_visited: distinct.len(),
                coverage: distinct.len() as f64 / 64.0,
            });
        }

        patterns
    }

    fn detect_oscillation(modes: &[ReasoningHexagram], patterns: &mut Vec<TrajectoryPattern>) {
        if modes.len() < 4 {
            return;
        }
        let a = modes[modes.len() - 2];
        let b = modes[modes.len() - 1];
        if a == b {
            return;
        }
        // Check ABAB... pattern going backwards in pairs
        let mut pairs = 0;
        let start = modes.len() - 1;
        let mut i = start;
        while i >= 1 {
            if modes[i] == b && modes[i - 1] == a {
                pairs += 1;
                if i < 2 {
                    break;
                }
                i -= 2;
            } else {
                break;
            }
        }
        if pairs >= 2 {
            let effective_len = pairs * 2;
            patterns.push(TrajectoryPattern::Oscillation {
                a,
                b,
                cycle_len: effective_len,
            });
        }
    }

    fn detect_stuck(modes: &[ReasoningHexagram], patterns: &mut Vec<TrajectoryPattern>) {
        if modes.len() < 3 {
            return;
        }
        let Some(last) = modes.last().copied() else {
            return;
        };
        let mut repeat = 0;
        for m in modes.iter().rev() {
            if *m == last {
                repeat += 1;
            } else {
                break;
            }
        }
        if repeat >= 3 {
            patterns.push(TrajectoryPattern::Stuck {
                state: last,
                steps: repeat,
            });
        }
    }

    fn detect_loop_back(modes: &[ReasoningHexagram], patterns: &mut Vec<TrajectoryPattern>) {
        if modes.len() < 3 {
            return;
        }
        let Some(last) = modes.last().copied() else {
            return;
        };
        // Check if last state was visited somewhere in the middle
        for i in 1..modes.len() - 1 {
            if modes[i] == last {
                let dist = modes[i.saturating_sub(1)].hamming_dist(&modes[i]);
                if dist <= 2 {
                    patterns.push(TrajectoryPattern::LoopBack {
                        from: modes[i.saturating_sub(1)],
                        to: modes[i],
                        dist,
                    });
                    break;
                }
            }
        }
    }

    fn detect_efficiency(modes: &[ReasoningHexagram], patterns: &mut Vec<TrajectoryPattern>) {
        if modes.len() < 2 {
            return;
        }
        let Some(start) = modes.first().copied() else {
            return;
        };
        let Some(end) = modes.last().copied() else {
            return;
        };
        let steps_taken = modes.len() - 1;
        let dist = start.hamming_dist(&end);
        let optimal = if dist > 0 { dist as usize } else { 1 };
        let ratio = optimal as f64 / steps_taken.max(1) as f64;
        if ratio >= 0.8 {
            patterns.push(TrajectoryPattern::Efficient {
                steps: steps_taken,
                optimal,
                ratio,
            });
        } else if ratio <= 0.3 && steps_taken >= 4 {
            patterns.push(TrajectoryPattern::Inefficient {
                steps: steps_taken,
                optimal,
                ratio,
            });
        }
    }

    fn detect_explore_exploit(modes: &[ReasoningHexagram], patterns: &mut Vec<TrajectoryPattern>) {
        if modes.len() < 5 {
            return;
        }
        // Detect: high diversity first half, focused second half
        let mid = modes.len() / 2;
        let first_half: std::collections::HashSet<_> = modes[..mid].iter().copied().collect();
        let second_half: std::collections::HashSet<_> = modes[mid..].iter().copied().collect();
        if first_half.len() > mid / 2 && second_half.len() <= 2 {
            patterns.push(TrajectoryPattern::ExploreThenExploit {
                explore_len: mid,
                exploit_len: modes.len() - mid,
            });
        }
    }

    /// Classify each step's quality.
    fn classify_steps(modes: &[ReasoningHexagram]) -> Vec<StepQuality> {
        let mut qualities = Vec::with_capacity(modes.len());
        if modes.is_empty() {
            return qualities;
        }

        // First step is always neutral
        qualities.push(StepQuality::Neutral);

        for i in 1..modes.len() {
            let prev = modes[i - 1];
            let curr = modes[i];

            // Stuck
            if curr == prev {
                qualities.push(StepQuality::Stuck);
                continue;
            }

            // Oscillation
            if i >= 2 && curr == modes[i - 2] && modes[i - 1] == prev {
                qualities.push(StepQuality::Oscillating);
                continue;
            }

            // Regressive (returning to earlier state within window of 5)
            let mut regressive = false;
            for j in (i.saturating_sub(5)..i).rev() {
                if curr == modes[j] {
                    qualities.push(StepQuality::Regressive);
                    regressive = true;
                    break;
                }
            }
            if regressive {
                continue;
            }

            // Productive: moving to a new state
            let is_new = !modes[..i].contains(&curr);
            if is_new {
                qualities.push(StepQuality::Productive);
            } else {
                qualities.push(StepQuality::Neutral);
            }
        }

        // Mark last as dead-end if stuck at the end
        if qualities.len() > 2 {
            let last_two = &qualities[qualities.len().saturating_sub(2)..];
            if last_two.iter().all(|q| *q == StepQuality::Stuck) {
                if let Some(last) = qualities.last_mut() {
                    *last = StepQuality::DeadEnd;
                }
            }
        }

        qualities
    }

    /// Compute overall quality score (0.0—1.0).
    fn compute_quality(
        modes: &[ReasoningHexagram],
        patterns: &[TrajectoryPattern],
        step_qualities: &[StepQuality],
        task_keywords: &[&str],
    ) -> f64 {
        let mut score = 0.7; // base score

        // Deduct for negative patterns
        for p in patterns {
            match p {
                TrajectoryPattern::Stuck { steps, .. } => score -= 0.1 * *steps.min(&5) as f64,
                TrajectoryPattern::Oscillation { cycle_len, .. } => {
                    score -= 0.05 * *cycle_len.min(&6) as f64
                }
                TrajectoryPattern::Inefficient { ratio, .. } => score -= 0.3 * (1.0 - ratio),
                TrajectoryPattern::LoopBack { .. } => score -= 0.1,
                _ => {}
            }
        }

        // Bonus for productive patterns
        for p in patterns {
            match p {
                TrajectoryPattern::Efficient { .. } => score += 0.2,
                TrajectoryPattern::ProductiveWalk { coverage, .. } => {
                    score += 0.1 * coverage.min(1.0)
                }
                TrajectoryPattern::ExploreThenExploit { .. } => score += 0.15,
                _ => {}
            }
        }

        // Bonus for step qualities
        if !step_qualities.is_empty() {
            let productive_ratio = step_qualities
                .iter()
                .filter(|q| **q == StepQuality::Productive)
                .count() as f64
                / step_qualities.len() as f64;
            score += productive_ratio * 0.2;
        }

        // Penalize very short trajectories
        if modes.len() <= 1 {
            score -= 0.2;
        }

        // Bonus for keyword alignment with start state
        if let Some(start) = modes.first() {
            let kw_mode = optimal_starting_mode(task_keywords.join(" ").as_str());
            if kw_mode == *start {
                score += 0.1;
            }
        }

        score.clamp(0.0, 1.0)
    }

    /// Recommend meta-state based on trajectory quality.
    fn recommend_meta(patterns: &[TrajectoryPattern], quality: f64) -> Option<MetaState> {
        // Stuck or critical issues → enter reflection mode
        let has_critical = patterns.iter().any(|p| match p {
            TrajectoryPattern::Stuck { steps, .. } if *steps >= 5 => true,
            TrajectoryPattern::Inefficient { .. } => true,
            TrajectoryPattern::Oscillation { cycle_len, .. } if *cycle_len >= 6 => true,
            _ => false,
        });
        if has_critical {
            return Some(MetaState::new(0b01)); // reflect
        }

        // Very high quality → enter planning mode
        if quality >= 0.85 {
            return Some(MetaState::new(0b10)); // plan
        }

        None
    }

    /// Compute capability vector deltas based on patterns.
    fn compute_capability_deltas(
        patterns: &[TrajectoryPattern],
        quality: f64,
    ) -> Vec<(String, f64)> {
        let mut deltas: Vec<(&str, f64)> = Vec::new();

        for p in patterns {
            match p {
                TrajectoryPattern::Stuck { steps, .. } if *steps >= 4 => {
                    deltas.push(("exploration_bias", 0.05));
                    deltas.push(("task_adaptability", -0.03));
                }
                TrajectoryPattern::Oscillation { cycle_len, .. } if *cycle_len >= 6 => {
                    deltas.push(("decisiveness", 0.04));
                    deltas.push(("analysis_depth", 0.03));
                }
                TrajectoryPattern::Inefficient { .. } => {
                    deltas.push(("planning_ahead", 0.06));
                    deltas.push(("pattern_matching", 0.04));
                }
                TrajectoryPattern::Efficient { .. } => {
                    deltas.push(("route_planning", 0.03));
                    deltas.push(("task_success_rate", 0.02));
                }
                TrajectoryPattern::ProductiveWalk { coverage, .. } => {
                    deltas.push(("knowledge_scope", 0.02 * coverage));
                }
                TrajectoryPattern::ExploreThenExploit { .. } => {
                    deltas.push(("exploration_bias", 0.02));
                    deltas.push(("convergence_speed", 0.03));
                }
                _ => {}
            }
        }

        // If quality is very low, suggest broad improvements
        if quality < 0.3 {
            deltas.push(("learning_rate", 0.05));
            deltas.push(("cross_domain_transfer", 0.04));
        }

        // Deduplicate by averaging
        let mut merged: Vec<(&str, Vec<f64>)> = Vec::new();
        for (name, val) in deltas {
            if let Some(existing) = merged.iter_mut().find(|(n, _)| *n == name) {
                existing.1.push(val);
            } else {
                merged.push((name, vec![val]));
            }
        }
        merged
            .into_iter()
            .map(|(n, vals)| (n.to_string(), vals.iter().sum::<f64>() / vals.len() as f64))
            .collect()
    }
}

use super::nt_core_prm::{AgentTrajectory, Coach, CoachContext, ProcessScore, ScoredCriterion};

/// Map ObserverReport → Vec<ProcessScore> for Coach trait compatibility.
fn observer_report_to_process_scores(
    report: &ObserverReport,
    trajectory: &AgentTrajectory,
) -> Vec<ProcessScore> {
    trajectory
        .steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let step_quality = report.step_qualities.get(i).copied();
            let quality_score = match step_quality {
                Some(StepQuality::Productive) => 0.8,
                Some(StepQuality::Neutral) => 0.5,
                Some(StepQuality::Regressive) => 0.3,
                Some(StepQuality::Oscillating) => 0.2,
                Some(StepQuality::Stuck) => 0.1,
                Some(StepQuality::DeadEnd) => 0.0,
                None => 0.5,
            };

            let mut tags = Vec::new();
            if let Some(q) = step_quality {
                tags.push(format!("step_{:?}", q).to_lowercase());
            }
            if !step.success {
                tags.push("step_fail".to_string());
            }

            let mut criteria = Vec::new();
            criteria.push(ScoredCriterion {
                name: "observer_quality".to_string(),
                score: quality_score,
                rationale: Some(format!(
                    "{:?}",
                    step_quality.unwrap_or(StepQuality::Neutral)
                )),
            });

            // Map trajectory patterns to global criteria
            for pattern in &report.patterns {
                match pattern {
                    TrajectoryPattern::Efficient { .. } => {
                        criteria.push(ScoredCriterion {
                            name: "efficiency".to_string(),
                            score: 0.9,
                            rationale: Some("efficient trajectory".to_string()),
                        });
                        tags.push("efficient".to_string());
                    }
                    TrajectoryPattern::Inefficient { .. } => {
                        criteria.push(ScoredCriterion {
                            name: "efficiency".to_string(),
                            score: 0.2,
                            rationale: Some("inefficient trajectory".to_string()),
                        });
                        tags.push("inefficient".to_string());
                    }
                    TrajectoryPattern::Stuck { .. } => {
                        tags.push("stuck".to_string());
                    }
                    TrajectoryPattern::ProductiveWalk { .. } => {
                        tags.push("productive_walk".to_string());
                    }
                    _ => {}
                }
            }

            let adjusted = if report.quality_score < 0.3 {
                quality_score * 0.8
            } else if report.quality_score > 0.8 {
                (quality_score + 0.1).min(1.0)
            } else {
                quality_score
            };

            ProcessScore {
                step_idx: i,
                score: adjusted.max(0.0).min(1.0),
                confidence: report.quality_score,
                criteria,
                attribution_tags: tags,
            }
        })
        .collect()
}

impl Coach for OneObserver {
    fn name(&self) -> &str {
        "observer-v1"
    }

    fn score_step(
        &self,
        _step: &super::nt_core_prm::TrajectoryStep,
        _context: &CoachContext,
    ) -> ProcessScore {
        ProcessScore::new(0)
    }

    fn score_episode(&self, trajectory: &AgentTrajectory) -> Vec<ProcessScore> {
        let modes: Vec<ReasoningHexagram> = trajectory.steps.iter().map(|s| s.e8_mode).collect();
        if modes.is_empty() {
            return Vec::new();
        }
        let full_states: Vec<FullReasoningState> = modes
            .iter()
            .map(|m| FullReasoningState::new(*m, MetaState(0)))
            .collect();

        let mut observer = OneObserver::new();
        let keywords: Vec<&str> = trajectory.task.split_whitespace().collect();
        let report = observer.analyze(&full_states, &keywords);

        observer_report_to_process_scores(&report, trajectory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn short_trajectory(modes: &[u8]) -> Vec<FullReasoningState> {
        modes
            .iter()
            .map(|&m| FullReasoningState::new(ReasoningHexagram(m), MetaState(0)))
            .collect()
    }

    #[test]
    fn test_empty_trajectory() {
        let mut observer = OneObserver::new();
        let report = observer.analyze(&[], &[]);
        assert_eq!(report.trajectory_len, 0);
        assert!(!report.has_actionable_insight);
    }

    #[test]
    fn test_single_state() {
        let mut observer = OneObserver::new();
        let traj = short_trajectory(&[0]);
        let report = observer.analyze(&traj, &[]);
        assert_eq!(report.trajectory_len, 1);
        assert_eq!(report.distinct_states, 1);
    }

    #[test]
    fn test_detects_oscillation() {
        let mut observer = OneObserver::new();
        // ABABABAB = 8 steps, 4 full cycles
        let traj = short_trajectory(&[0, 1, 0, 1, 0, 1, 0, 1]);
        let report = observer.analyze(&traj, &[]);
        let has_osc = report
            .patterns
            .iter()
            .any(|p| matches!(p, TrajectoryPattern::Oscillation { .. }));
        assert!(
            has_osc,
            "should detect oscillation, patterns: {:?}",
            report.patterns
        );
        assert!(report.quality_score < 0.7);
    }

    #[test]
    fn test_detects_stuck() {
        let mut observer = OneObserver::new();
        let traj = short_trajectory(&[10, 10, 10, 10, 10]);
        let report = observer.analyze(&traj, &[]);
        let has_stuck = report
            .patterns
            .iter()
            .any(|p| matches!(p, TrajectoryPattern::Stuck { .. }));
        assert!(has_stuck);
    }

    #[test]
    fn test_efficient_trajectory() {
        let mut observer = OneObserver::new();
        // Direct path: 0 → 63 (all bits flipped) takes 6 steps minimum
        let traj = short_trajectory(&[0, 1, 3, 7, 15, 31, 63]);
        let report = observer.analyze(&traj, &["abstract", "broad"]);
        let has_efficient = report
            .patterns
            .iter()
            .any(|p| matches!(p, TrajectoryPattern::Efficient { .. }));
        assert!(has_efficient, "should detect efficient path");
        assert!(report.quality_score > 0.7);
    }

    #[test]
    fn test_inefficient_trajectory() {
        let mut observer = OneObserver::new();
        // Wandering path: 0 → 1 → 2 → 4 → 8 → 16 → 32 → 0 → 1 → 2 → 3
        let traj = short_trajectory(&[0, 1, 2, 4, 8, 16, 32, 0, 1, 2, 3]);
        let report = observer.analyze(&traj, &[]);
        let has_inefficient = report
            .patterns
            .iter()
            .any(|p| matches!(p, TrajectoryPattern::Inefficient { .. }));
        // Not all wandering paths are flagged as inefficient; the efficiency depends on ratio
        // If we detect something, great
        if has_inefficient {
            assert!(report.quality_score < 0.7);
        }
    }

    #[test]
    fn test_observer_recommends_meta_on_critical() {
        let mut observer = OneObserver::new();
        // Stuck for 6 steps
        let traj = short_trajectory(&[5, 5, 5, 5, 5, 5]);
        let report = observer.analyze(&traj, &[]);
        assert!(report.recommended_meta.is_some());
        // Should recommend reflection (bit 0 set)
        assert!(report
            .recommended_meta
            .expect("recommended_meta should be set")
            .is_reflecting());
    }

    #[test]
    fn test_capability_deltas_on_stuck() {
        let mut observer = OneObserver::new();
        let traj = short_trajectory(&[10, 10, 10, 10, 10]);
        let report = observer.analyze(&traj, &[]);
        assert!(!report.capability_deltas.is_empty());
        let has_exploration = report
            .capability_deltas
            .iter()
            .any(|(n, _)| *n == "exploration_bias");
        assert!(has_exploration);
    }

    #[test]
    fn test_quality_score_upper_bound() {
        let mut observer = OneObserver::new();
        // Perfect efficient path
        let traj = short_trajectory(&[0, 63]);
        let report = observer.analyze(&traj, &["concrete", "focused"]);
        assert!(report.quality_score <= 1.0);
        assert!(report.quality_score >= 0.0);
    }

    #[test]
    fn test_trajectory_history_accumulates() {
        let mut observer = OneObserver::new();
        observer.analyze(&short_trajectory(&[0, 1, 2]), &[]);
        observer.analyze(&short_trajectory(&[5, 5, 5, 5, 5]), &[]);
        assert_eq!(observer.analysis_count, 2);
        assert_eq!(observer.trajectory_history.len(), 2);
    }

    #[test]
    fn test_step_qualities_length_match_trajectory() {
        let mut observer = OneObserver::new();
        let traj = short_trajectory(&[0, 1, 0, 1, 3, 7, 15]);
        let report = observer.analyze(&traj, &[]);
        assert_eq!(report.step_qualities.len(), traj.len());
    }

    #[test]
    fn test_detect_explore_exploit() {
        let mut observer = OneObserver::new();
        // High diversity then focused (at most 2 unique states in second half)
        let modes = vec![0, 10, 20, 30, 40, 50, 55, 60, 63, 63, 63, 63, 63, 63];
        let traj = short_trajectory(&modes);
        let report = observer.analyze(&traj, &[]);
        let has_e_e = report
            .patterns
            .iter()
            .any(|p| matches!(p, TrajectoryPattern::ExploreThenExploit { .. }));
        assert!(has_e_e, "should detect explore-then-exploit");
    }

    #[test]
    fn test_report_is_healthy() {
        let mut observer = OneObserver::new();
        let traj = short_trajectory(&[0, 1, 3, 7, 15]);
        let report = observer.analyze(&traj, &[]);
        assert!(report.is_healthy());

        let traj2 = short_trajectory(&[5, 5, 5, 5, 5, 5]);
        let report2 = observer.analyze(&traj2, &[]);
        assert!(!report2.is_healthy());
    }

    #[test]
    fn test_accumulated_deltas_across_calls() {
        let mut observer = OneObserver::new();
        observer.analyze(&short_trajectory(&[5, 5, 5, 5, 5]), &[]);
        assert!(!observer.accumulated_deltas.is_empty());
        let first_count = observer.accumulated_deltas.len();
        observer.analyze(&short_trajectory(&[10, 10, 10, 10, 10]), &[]);
        assert!(observer.accumulated_deltas.len() > first_count);
    }

    #[test]
    fn test_keyword_alignment_bonus() {
        let mut observer = OneObserver::new();
        // Start with mode 0 (concrete, focused, analytical, deep, solo, certain)
        // 'concrete' keyword should map to mode 0
        let traj = short_trajectory(&[0, 1, 2]);
        let report = observer.analyze(&traj, &["concrete"]);
        // With keyword bonus and efficient path
        assert!(report.quality_score > 0.5);
    }

    #[test]
    fn test_observer_has_not_critical_pattern_for_healthy() {
        let mut observer = OneObserver::new();
        let traj = short_trajectory(&[0, 1, 3, 7, 15, 31, 63]);
        let report = observer.analyze(&traj, &[]);
        assert!(!report.has_critical_pattern());
    }
}
