use log;
use std::collections::HashMap;

/// L0-L5 self-model hierarchy following Jiang et al. (JCST 2026).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelfModelLevel {
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
}

impl SelfModelLevel {
    pub fn description(&self) -> &'static str {
        match self {
            SelfModelLevel::L0 => "No self-representation",
            SelfModelLevel::L1 => "Self-perception — awareness of body and internal state",
            SelfModelLevel::L2 => "Self-memory — temporal continuity and experience accumulation",
            SelfModelLevel::L3 => {
                "Self-identity — persistent narrative self and agency attribution"
            }
            SelfModelLevel::L4 => {
                "Self-capability modeling — knowledge of own strengths and limitations"
            }
            SelfModelLevel::L5 => {
                "Full self-awareness — meta-cognitive self-model with predictive capability"
            }
        }
    }

    /// Minimum threshold (0-1) for claiming this level is meaningfully reached.
    pub fn threshold(&self) -> f64 {
        match self {
            SelfModelLevel::L0 => 0.0,
            SelfModelLevel::L1 => 0.3,
            SelfModelLevel::L2 => 0.5,
            SelfModelLevel::L3 => 0.6,
            SelfModelLevel::L4 => 0.7,
            SelfModelLevel::L5 => 0.85,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelfModelReport {
    pub current_level: SelfModelLevel,
    pub scores: HashMap<String, f64>,
    pub next_level: SelfModelLevel,
    pub requirements: Vec<String>,
}

/// Assesses the system's self-model level across L0-L5 dimensions.
///
/// Can be populated directly from ConsciousnessIntegration fields
/// or used standalone with explicit scores.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelfModelAssessor {
    /// L1: Self-perception — awareness of internal state (0.0-1.0)
    pub awareness: f64,
    /// L1: Cognitive load — evidence of active processing (>0.0)
    pub cognitive_load: f64,
    /// L2: Pass rate — fraction of successful handler executions (0.0-1.0)
    pub pass_rate: f64,
    /// L2: Total cycles run — evidence of temporal continuity
    pub cycle: u64,
    /// L3: Narrative self coherence — internal story consistency (0.0-1.0)
    pub narrative_coherence: f64,
    /// L3: Soul identity integrity — identity hash verification (0.0-1.0)
    pub soul_integrity: f64,
    /// L4: Meta-accuracy error — |self_predicted - actual_performance| (lower=better)
    pub meta_accuracy: f64,
    /// L4: Whether the SelfInspectable trait is available
    pub self_inspect_available: bool,
    /// L5: Whether the metacognitive loop is actively running
    pub metacognitive_loop_healthy: bool,
    /// L5: Whether the system has enough data to predict own future performance
    pub can_predict_performance: bool,
}

impl Default for SelfModelAssessor {
    fn default() -> Self {
        Self {
            awareness: 0.0,
            cognitive_load: 0.0,
            pass_rate: 0.0,
            cycle: 0,
            narrative_coherence: 0.0,
            soul_integrity: 0.0,
            meta_accuracy: 1.0,
            self_inspect_available: false,
            metacognitive_loop_healthy: false,
            can_predict_performance: false,
        }
    }
}

impl SelfModelAssessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience constructor from ConsciousnessIntegration-relevant metrics.
    pub fn from_metrics(
        awareness: f64,
        cognitive_load: f64,
        pass_rate: f64,
        cycle: u64,
        narrative_coherence: f64,
        soul_integrity: f64,
        meta_accuracy: f64,
        self_inspect_available: bool,
        metacognitive_loop_healthy: bool,
        can_predict_performance: bool,
    ) -> Self {
        Self {
            awareness,
            cognitive_load,
            pass_rate,
            cycle,
            narrative_coherence,
            soul_integrity,
            meta_accuracy,
            self_inspect_available,
            metacognitive_loop_healthy,
            can_predict_performance,
        }
    }

    /// Evaluate the current self-model level based on stored scores,
    /// enforcing transition conditions between levels.
    pub fn assess_level(&self) -> SelfModelLevel {
        let raw = Self::assess_from_scores(
            self.awareness,
            self.pass_rate,
            self.cycle,
            self.meta_accuracy,
            self.narrative_coherence,
            self.soul_integrity,
            self.cognitive_load,
            self.self_inspect_available,
            self.metacognitive_loop_healthy,
            self.can_predict_performance,
        );
        let mut current = SelfModelLevel::L0;
        let levels = [
            SelfModelLevel::L1,
            SelfModelLevel::L2,
            SelfModelLevel::L3,
            SelfModelLevel::L4,
            SelfModelLevel::L5,
        ];
        for target in &levels {
            if *target > raw {
                break;
            }
            let (can, blockers) = self.can_transition_to(*target);
            if can {
                current = *target;
            } else {
                log::info!(
                    "Self-model transition to {:?} blocked: {:?}",
                    target,
                    blockers
                );
                break;
            }
        }
        current
    }

    /// Save assessor state to a JSON file.
    /// Returns Ok(()) on success.
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(path, json)
    }

    /// Load assessor state from a JSON file.
    /// Returns the loaded SelfModelAssessor on success.
    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let assessor: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(assessor)
    }

    /// Per-dimension scores for the full report.
    pub fn level_scores(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        map.insert("perception".into(), self.awareness);
        map.insert("memory".into(), self.pass_rate);
        map.insert(
            "identity".into(),
            self.narrative_coherence.min(self.soul_integrity),
        );
        map.insert("capability".into(), (1.0 - self.meta_accuracy).max(0.0));
        map.insert(
            "meta".into(),
            if self.metacognitive_loop_healthy && self.can_predict_performance {
                1.0
            } else if self.metacognitive_loop_healthy {
                0.6
            } else {
                0.0
            },
        );
        map
    }

    /// Produce a structured report with current level, scores, and next-level requirements.
    /// Includes transition condition blockers in the requirements list.
    pub fn report(&self) -> SelfModelReport {
        let current = self.assess_level();
        let scores = self.level_scores();

        let next_level = match current {
            SelfModelLevel::L0 => SelfModelLevel::L1,
            SelfModelLevel::L1 => SelfModelLevel::L2,
            SelfModelLevel::L2 => SelfModelLevel::L3,
            SelfModelLevel::L3 => SelfModelLevel::L4,
            SelfModelLevel::L4 => SelfModelLevel::L5,
            SelfModelLevel::L5 => SelfModelLevel::L5,
        };

        let mut requirements = self.compute_requirements(current, next_level);

        let (_can_transition, blockers) = self.can_transition_to(next_level);
        if !blockers.is_empty() && next_level != current {
            for blocker in &blockers {
                requirements.push(format!("Transition blocked: {}", blocker));
            }
        }

        SelfModelReport {
            current_level: current,
            scores,
            next_level,
            requirements,
        }
    }

    fn compute_requirements(&self, current: SelfModelLevel, _next: SelfModelLevel) -> Vec<String> {
        let mut reqs = Vec::new();
        match current {
            SelfModelLevel::L0 => {
                if self.awareness <= 0.3 {
                    reqs.push(format!(
                        "Increase awareness from {:.2} to > 0.3",
                        self.awareness
                    ));
                }
                if self.cognitive_load <= 0.0 {
                    reqs.push("Establish non-zero cognitive load".into());
                }
            }
            SelfModelLevel::L1 => {
                if self.pass_rate <= 0.3 {
                    reqs.push(format!(
                        "Increase handler pass rate from {:.2} to > 0.3",
                        self.pass_rate
                    ));
                }
                if self.cycle <= 100 {
                    reqs.push(format!(
                        "Run more cycles (current: {}, need > 100)",
                        self.cycle
                    ));
                }
            }
            SelfModelLevel::L2 => {
                if self.narrative_coherence <= 0.5 {
                    reqs.push(format!(
                        "Improve narrative self coherence from {:.2} to > 0.5",
                        self.narrative_coherence
                    ));
                }
                if self.soul_integrity <= 0.5 {
                    reqs.push(format!(
                        "Improve soul identity integrity from {:.2} to > 0.5",
                        self.soul_integrity
                    ));
                }
            }
            SelfModelLevel::L3 => {
                if self.meta_accuracy >= 0.3 {
                    reqs.push(format!(
                        "Reduce meta-accuracy error from {:.2} to < 0.3",
                        self.meta_accuracy
                    ));
                }
                if !self.self_inspect_available {
                    reqs.push("Enable SelfInspectable trait implementation".into());
                }
            }
            SelfModelLevel::L4 => {
                if !self.metacognitive_loop_healthy {
                    reqs.push("Activate and stabilize metacognitive loop".into());
                }
                if !self.can_predict_performance {
                    reqs.push("Accumulate enough data to predict own performance".into());
                }
            }
            SelfModelLevel::L5 => {
                reqs.push("Maximum level reached — maintain and refine".into());
            }
        }
        reqs
    }

    /// Static assessment without needing an assessor instance.
    pub fn assess_from_scores(
        awareness: f64,
        pass_rate: f64,
        cycle: u64,
        meta_accuracy: f64,
        narrative_coherence: f64,
        soul_integrity: f64,
        cognitive_load: f64,
        self_inspect_available: bool,
        metacognitive_loop_healthy: bool,
        can_predict_performance: bool,
    ) -> SelfModelLevel {
        let meta_ok = metacognitive_loop_healthy && can_predict_performance;
        if meta_ok {
            return SelfModelLevel::L5;
        }

        let cap_ok = meta_accuracy < 0.3 && self_inspect_available;
        if cap_ok {
            return SelfModelLevel::L4;
        }

        let identity_ok = narrative_coherence > 0.5 && soul_integrity > 0.5;
        if identity_ok {
            return SelfModelLevel::L3;
        }

        let memory_ok = pass_rate > 0.3 && cycle > 100;
        if memory_ok {
            return SelfModelLevel::L2;
        }

        let perception_ok = awareness > 0.3 && cognitive_load > 0.0;
        if perception_ok {
            return SelfModelLevel::L1;
        }

        SelfModelLevel::L0
    }

    /// Check if the system can transition to the target level.
    /// Each transition has specific prerequisites per Jiang et al.:
    /// L0→L1: awareness > 0.3 AND cognitive_load > 0
    /// L1→L2: pass_rate > 0.3 AND cycle > 10
    /// L2→L3: narrative_coherence > 0.7 AND meta_accuracy < 0.3
    /// L3→L4: self_inspect_available AND meta_accuracy < 0.5
    /// L4→L5: can_predict_performance AND metacognitive_loop_healthy AND meta_accuracy < 0.2
    pub fn can_transition_to(&self, target: SelfModelLevel) -> (bool, Vec<String>) {
        let mut blockers = Vec::new();
        let can = match target {
            SelfModelLevel::L0 => true,
            SelfModelLevel::L1 => {
                if self.awareness <= 0.3 {
                    blockers.push(format!("awareness {:.2} > 0.3 required", self.awareness));
                }
                if self.cognitive_load <= 0.0 {
                    blockers.push("cognitive_load > 0 required".into());
                }
                blockers.is_empty()
            }
            SelfModelLevel::L2 => {
                if self.pass_rate <= 0.3 {
                    blockers.push(format!("pass_rate {:.2} > 0.3 required", self.pass_rate));
                }
                if self.cycle <= 10 {
                    blockers.push(format!("cycle {:.0} > 10 required", self.cycle));
                }
                blockers.is_empty()
            }
            SelfModelLevel::L3 => {
                if self.narrative_coherence <= 0.7 {
                    blockers.push(format!(
                        "narrative_coherence {:.2} > 0.7 required",
                        self.narrative_coherence
                    ));
                }
                if self.meta_accuracy >= 0.3 {
                    blockers.push(format!(
                        "meta_accuracy {:.3} < 0.3 required",
                        self.meta_accuracy
                    ));
                }
                blockers.is_empty()
            }
            SelfModelLevel::L4 => {
                if !self.self_inspect_available {
                    blockers.push("self_inspect_available required".into());
                }
                if self.meta_accuracy >= 0.5 {
                    blockers.push(format!(
                        "meta_accuracy {:.3} < 0.5 required",
                        self.meta_accuracy
                    ));
                }
                blockers.is_empty()
            }
            SelfModelLevel::L5 => {
                if !self.can_predict_performance {
                    blockers.push("can_predict_performance required".into());
                }
                if !self.metacognitive_loop_healthy {
                    blockers.push("metacognitive_loop_healthy required".into());
                }
                if self.meta_accuracy >= 0.2 {
                    blockers.push(format!(
                        "meta_accuracy {:.3} < 0.2 required",
                        self.meta_accuracy
                    ));
                }
                blockers.is_empty()
            }
        };
        (can, blockers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l0_when_all_scores_zero() {
        let level = SelfModelAssessor::assess_from_scores(
            0.0, 0.0, 0, 1.0, 0.0, 0.0, 0.0, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L0);
    }

    #[test]
    fn test_l1_when_awareness_high() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.0, 0, 1.0, 0.0, 0.0, 0.2, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L1);
    }

    #[test]
    fn test_l1_not_reached_without_cognitive_load() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.0, 0, 1.0, 0.0, 0.0, 0.0, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L0);
    }

    #[test]
    fn test_l2_when_pass_rate_meets_threshold() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.6, 200, 1.0, 0.0, 0.0, 0.2, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L2);
    }

    #[test]
    fn test_l2_not_reached_without_enough_cycles() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.6, 50, 1.0, 0.0, 0.0, 0.2, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L1);
    }

    #[test]
    fn test_l3_when_identity_coherent() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.6, 200, 1.0, 0.7, 0.8, 0.2, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L3);
    }

    #[test]
    fn test_l4_when_capability_known() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.6, 200, 0.2, 0.7, 0.8, 0.2, true, false, false,
        );
        assert_eq!(level, SelfModelLevel::L4);
    }

    #[test]
    fn test_l4_not_reached_without_self_inspect() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.6, 200, 0.2, 0.7, 0.8, 0.2, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L3);
    }

    #[test]
    fn test_l5_when_meta_full() {
        let level = SelfModelAssessor::assess_from_scores(
            0.5, 0.6, 200, 0.1, 0.7, 0.8, 0.2, true, true, true,
        );
        assert_eq!(level, SelfModelLevel::L5);
    }

    #[test]
    fn test_level_descriptions_non_empty() {
        for level in &[
            SelfModelLevel::L0,
            SelfModelLevel::L1,
            SelfModelLevel::L2,
            SelfModelLevel::L3,
            SelfModelLevel::L4,
            SelfModelLevel::L5,
        ] {
            assert!(!level.description().is_empty());
        }
    }

    #[test]
    fn test_level_thresholds_monotonic() {
        let mut prev = SelfModelLevel::L0.threshold();
        for level in &[
            SelfModelLevel::L1,
            SelfModelLevel::L2,
            SelfModelLevel::L3,
            SelfModelLevel::L4,
            SelfModelLevel::L5,
        ] {
            let t = level.threshold();
            assert!(t > prev, "thresholds must increase with level");
            prev = t;
        }
    }

    #[test]
    fn test_report_format() {
        let assessor =
            SelfModelAssessor::from_metrics(0.5, 0.3, 0.6, 200, 0.75, 0.8, 0.2, true, false, false);
        let report = assessor.report();
        assert_eq!(report.current_level, SelfModelLevel::L4);
        assert_eq!(report.next_level, SelfModelLevel::L5);
        assert!(report.scores.contains_key("perception"));
        assert!(report.scores.contains_key("memory"));
        assert!(report.scores.contains_key("identity"));
        assert!(report.scores.contains_key("capability"));
        assert!(report.scores.contains_key("meta"));
        assert!(!report.requirements.is_empty());
    }

    #[test]
    fn test_report_at_l5_has_no_next_requirements() {
        let assessor =
            SelfModelAssessor::from_metrics(0.9, 0.8, 0.9, 1000, 0.9, 0.9, 0.05, true, true, true);
        let report = assessor.report();
        assert_eq!(report.current_level, SelfModelLevel::L5);
        assert_eq!(report.next_level, SelfModelLevel::L5);
        assert!(report
            .requirements
            .contains(&"Maximum level reached — maintain and refine".to_string()));
    }

    #[test]
    fn test_next_level_logic() {
        // L0 → next is L1
        let a = SelfModelAssessor::default();
        let r = a.report();
        assert_eq!(r.current_level, SelfModelLevel::L0);
        assert_eq!(r.next_level, SelfModelLevel::L1);

        // L3 → next is L4
        let a = SelfModelAssessor::from_metrics(
            0.5, 0.3, 0.6, 200, 0.75, 0.8, 0.2, false, false, false,
        );
        let r = a.report();
        assert_eq!(r.current_level, SelfModelLevel::L3);
        assert_eq!(r.next_level, SelfModelLevel::L4);
    }

    #[test]
    fn test_level_scores_all_present() {
        let assessor =
            SelfModelAssessor::from_metrics(0.8, 0.5, 0.7, 500, 0.6, 0.9, 0.1, true, true, true);
        let scores = assessor.level_scores();
        assert_eq!(scores.len(), 5);
        assert!((scores["perception"] - 0.8).abs() < 1e-6);
        assert!((scores["memory"] - 0.7).abs() < 1e-6);
        assert!((scores["identity"] - 0.6).abs() < 1e-6);
        assert!((scores["capability"] - 0.9).abs() < 1e-6);
        assert!((scores["meta"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_static_assess_from_scores() {
        let level = SelfModelAssessor::assess_from_scores(
            0.0, 0.0, 0, 1.0, 0.0, 0.0, 0.0, false, false, false,
        );
        assert_eq!(level, SelfModelLevel::L0);
    }

    #[test]
    fn test_new_returns_default() {
        let a = SelfModelAssessor::new();
        assert_eq!(a.awareness, 0.0);
        assert_eq!(a.cycle, 0);
        assert_eq!(a.meta_accuracy, 1.0);
    }
}
