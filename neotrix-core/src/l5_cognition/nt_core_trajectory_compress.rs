use crate::core::nt_core_hex::FullReasoningState;
use crate::core::nt_core_prm::{AgentTrajectory, TrajectoryStep};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionLevel {
    Light,
    #[default]
    Medium,
    Aggressive,
}

#[derive(Debug, Clone)]
pub struct TrajectoryCompressionReport {
    pub original_steps: usize,
    pub compressed_steps: usize,
    pub original_chars: usize,
    pub compressed_chars: usize,
    pub removed_redundant: usize,
    pub removed_low_value: usize,
}

impl TrajectoryCompressionReport {
    pub fn compression_ratio(&self) -> f64 {
        if self.original_chars == 0 {
            return 1.0;
        }
        self.compressed_chars as f64 / self.original_chars as f64
    }
}

pub struct TrajectoryCompressor {
    level: CompressionLevel,
    max_io_chars: usize,
    min_score_threshold: f64,
}

impl Default for TrajectoryCompressor {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Medium,
            max_io_chars: 500,
            min_score_threshold: 0.3,
        }
    }
}

impl TrajectoryCompressor {
    pub fn new(level: CompressionLevel) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    pub fn with_io_limit(mut self, limit: usize) -> Self {
        self.max_io_chars = limit;
        self
    }
    pub fn with_score_threshold(mut self, threshold: f64) -> Self {
        self.min_score_threshold = threshold;
        self
    }

    pub fn compress_trajectory(
        &self,
        trajectory: &AgentTrajectory,
    ) -> (AgentTrajectory, TrajectoryCompressionReport) {
        let original_steps = trajectory.steps.len();
        let original_chars: usize = trajectory
            .steps
            .iter()
            .map(|s| s.input.len() + s.output.len())
            .sum();

        let mut compressed: Vec<TrajectoryStep> = Vec::new();
        let mut removed_redundant = 0;
        let mut removed_low_value = 0;

        for (i, step) in trajectory.steps.iter().enumerate() {
            if self.level as u8 >= CompressionLevel::Light as u8 {
                if let Some(last) = compressed.last() {
                    if last.specialist == step.specialist
                        && last.e8_mode == step.e8_mode
                        && !step.success
                        && last.action == step.action
                    {
                        removed_redundant += 1;
                        continue;
                    }
                }
            }

            if self.level == CompressionLevel::Aggressive {
                if let Some(ref reward) = step.external_reward {
                    if *reward < self.min_score_threshold && i > 0 {
                        removed_low_value += 1;
                        continue;
                    }
                }
            }

            let mut s = step.clone();
            if self.level as u8 >= CompressionLevel::Medium as u8 {
                if s.input.len() > self.max_io_chars {
                    let keep = self.max_io_chars.min(s.input.len());
                    s.input = format!("...{}", &s.input[s.input.len() - keep..]);
                }
                if s.output.len() > self.max_io_chars {
                    let keep = self.max_io_chars.min(s.output.len());
                    s.output = format!("{}...", &s.output[..keep]);
                }
            }

            compressed.push(s);
        }

        let compressed_chars: usize = compressed
            .iter()
            .map(|s| s.input.len() + s.output.len())
            .sum();

        let mut result = trajectory.clone();
        result.steps = compressed;

        let report = TrajectoryCompressionReport {
            original_steps,
            compressed_steps: result.steps.len(),
            original_chars,
            compressed_chars,
            removed_redundant,
            removed_low_value,
        };

        (result, report)
    }

    pub fn compress_state_trajectory(
        &self,
        states: &[FullReasoningState],
    ) -> Vec<FullReasoningState> {
        if self.level as u8 >= CompressionLevel::Light as u8 {
            let mut compressed: Vec<FullReasoningState> = Vec::new();
            for state in states {
                if let Some(last) = compressed.last() {
                    if last.mode == state.mode {
                        continue;
                    }
                }
                compressed.push(*state);
            }
            compressed
        } else {
            states.to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hex::ReasoningHexagram;
    use crate::core::nt_core_traits::SpecialistType;

    fn mk_step(
        idx: usize,
        sp: SpecialistType,
        mode: u8,
        inp: &str,
        out: &str,
        ok: bool,
        r: Option<f64>,
    ) -> TrajectoryStep {
        TrajectoryStep {
            step_idx: idx,
            specialist: sp,
            e8_mode: ReasoningHexagram::new(mode),
            action: "act".into(),
            input: inp.into(),
            output: out.into(),
            duration_ms: Some(100),
            success: ok,
            external_reward: r,
        }
    }

    #[test]
    fn test_light_removes_redundant_consecutive() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(
            0,
            SpecialistType::PatternMatcher,
            1,
            "a",
            "b",
            false,
            None,
        ));
        t.push(mk_step(
            1,
            SpecialistType::PatternMatcher,
            1,
            "c",
            "d",
            false,
            None,
        ));
        t.push(mk_step(
            2,
            SpecialistType::AnomalyDetector,
            1,
            "e",
            "f",
            true,
            None,
        ));
        let c = TrajectoryCompressor::new(CompressionLevel::Light);
        let (r, rep) = c.compress_trajectory(&t);
        assert_eq!(r.steps.len(), 2);
        assert_eq!(rep.removed_redundant, 1);
    }

    #[test]
    fn test_medium_truncates_long_io() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(
            0,
            SpecialistType::Planner,
            1,
            &"x".repeat(1000),
            &"y".repeat(1000),
            true,
            None,
        ));
        let c = TrajectoryCompressor::new(CompressionLevel::Medium).with_io_limit(100);
        let (r, _) = c.compress_trajectory(&t);
        assert!(r.steps[0].input.len() <= 200);
        assert!(r.steps[0].output.len() <= 200);
    }

    #[test]
    fn test_aggressive_drops_low_reward() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(
            0,
            SpecialistType::Planner,
            1,
            "a",
            "b",
            true,
            Some(0.9),
        ));
        t.push(mk_step(
            1,
            SpecialistType::RiskAssessor,
            1,
            "c",
            "d",
            true,
            Some(0.1),
        ));
        let c = TrajectoryCompressor::new(CompressionLevel::Aggressive).with_score_threshold(0.3);
        let (r, rep) = c.compress_trajectory(&t);
        assert_eq!(r.steps.len(), 1);
        assert_eq!(rep.removed_low_value, 1);
    }

    #[test]
    fn test_compress_state_trajectory_deduplicates() {
        let s = vec![
            FullReasoningState::new(
                ReasoningHexagram::new(0),
                crate::core::nt_core_hex::MetaState::new(0),
            ),
            FullReasoningState::new(
                ReasoningHexagram::new(0),
                crate::core::nt_core_hex::MetaState::new(0),
            ),
            FullReasoningState::new(
                ReasoningHexagram::new(1),
                crate::core::nt_core_hex::MetaState::new(0),
            ),
        ];
        let c = TrajectoryCompressor::new(CompressionLevel::Light);
        assert_eq!(c.compress_state_trajectory(&s).len(), 2);
    }

    #[test]
    fn test_report_has_compression_ratio() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(
            0,
            SpecialistType::PatternMatcher,
            1,
            "in",
            "out",
            true,
            None,
        ));
        t.push(mk_step(
            1,
            SpecialistType::PatternMatcher,
            1,
            "in2",
            "out2",
            false,
            None,
        ));
        let c = TrajectoryCompressor::new(CompressionLevel::Light);
        let (_, rep) = c.compress_trajectory(&t);
        assert_eq!(rep.original_steps, 2);
        assert_eq!(rep.removed_redundant, 1);
        assert!(rep.compression_ratio() < 1.0);
    }

    #[test]
    fn test_no_redundant_removal_with_different_specialist() {
        let mut t = AgentTrajectory::new(1, "t".into());
        t.push(mk_step(
            0,
            SpecialistType::PatternMatcher,
            1,
            "a",
            "b",
            true,
            None,
        ));
        t.push(mk_step(
            1,
            SpecialistType::AnomalyDetector,
            1,
            "c",
            "d",
            false,
            None,
        ));
        let c = TrajectoryCompressor::new(CompressionLevel::Light);
        let (r, rep) = c.compress_trajectory(&t);
        assert_eq!(r.steps.len(), 2);
        assert_eq!(rep.removed_redundant, 0);
    }
}
