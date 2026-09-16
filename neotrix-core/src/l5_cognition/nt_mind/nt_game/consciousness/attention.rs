//! Attention feedback from game performance.
//!
//! Maps cognitive skills exercised during gameplay to attention domain scores,
//! guiding the GWT attention router toward relevant cognitive domains.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Types (self-contained)
// ═══════════════════════════════════════════════════════════════════

/// Cognitive skills that a game exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CognitiveSkill {
    Planning,
    Optimization,
    PatternRecognition,
    EmotionRegulation,
    AttentionAllocation,
    MultiStepReasoning,
    Cooperation,
    AdversarialThinking,
    SpatialReasoning,
    CodeReasoning,
}

/// Difficulty tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Tutorial = 0,
    Apprentice = 1,
    Journeyman = 2,
    Expert = 3,
    Master = 4,
    Transcendent = 5,
}

impl Difficulty {
    pub fn constellation(&self) -> u8 {
        *self as u8
    }
}

/// Game trajectory summary (simplified).
#[derive(Debug, Clone)]
pub struct Trajectory {
    pub steps: Vec<TrajectoryStep>,
    pub total_reward: f64,
    pub length: usize,
}

/// Simplified trajectory step.
#[derive(Debug, Clone)]
pub struct TrajectoryStep {
    pub reward: f64,
    pub hexagram: Option<u8>,
    pub action_kind: String,
}

/// Game metadata (simplified).
#[derive(Debug, Clone)]
pub struct GameMeta {
    pub target_skills: Vec<CognitiveSkill>,
    pub difficulty: Difficulty,
}

// ═══════════════════════════════════════════════════════════════════
// Attention Mapping
// ═══════════════════════════════════════════════════════════════════

/// Maps CognitiveSkill to attention domain names.
pub struct AttentionMapping;

impl AttentionMapping {
    /// Get the attention domain name for a cognitive skill.
    pub fn domain_name(skill: &CognitiveSkill) -> &'static str {
        match skill {
            CognitiveSkill::Planning => "planning",
            CognitiveSkill::Optimization => "planning",
            CognitiveSkill::PatternRecognition => "pattern_match",
            CognitiveSkill::EmotionRegulation => "self_reflection",
            CognitiveSkill::AttentionAllocation => "self_reflection",
            CognitiveSkill::MultiStepReasoning => "code",
            CognitiveSkill::Cooperation => "goal_alignment",
            CognitiveSkill::AdversarialThinking => "risk_assessment",
            CognitiveSkill::SpatialReasoning => "pattern_match",
            CognitiveSkill::CodeReasoning => "code",
        }
    }

    /// Get all mapped domains.
    pub fn all_domains() -> Vec<&'static str> {
        vec![
            "planning",
            "pattern_match",
            "self_reflection",
            "code",
            "goal_alignment",
            "risk_assessment",
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════
// Domain Scores
// ═══════════════════════════════════════════════════════════════════

/// Compute attention domain scores from trajectory performance.
///
/// Each skill exercised by the game gets a base score of 0.3.
/// Positive trajectory reward boosts scores by +0.3 (win) or -0.1 (loss).
/// Higher difficulty adds a +0.05 per constellation level bonus.
pub fn compute_domain_scores(trajectory: &Trajectory, meta: &GameMeta) -> HashMap<String, f64> {
    let mut scores: HashMap<String, f64> = HashMap::new();
    let base = 0.3;

    // Reward modifier
    let reward_mod = if trajectory.total_reward > 0.0 {
        0.3
    } else if trajectory.total_reward < 0.0 {
        -0.1
    } else {
        0.0
    };

    // Difficulty bonus
    let diff_bonus = meta.difficulty.constellation() as f64 * 0.05;

    for skill in &meta.target_skills {
        let domain = AttentionMapping::domain_name(skill).to_string();
        let score = (base + reward_mod + diff_bonus).clamp(0.0, 1.0);
        scores.insert(domain, score);
    }

    // Ensure all mapped domains have a minimum score
    for domain in AttentionMapping::all_domains() {
        scores
            .entry(domain.to_string())
            .or_insert_with(|| base.clamp(0.0, 1.0));
    }

    scores
}

// ═══════════════════════════════════════════════════════════════════
// Intensity
// ═══════════════════════════════════════════════════════════════════

/// Compute attention intensity tier from difficulty.
pub fn compute_intensity(difficulty: &Difficulty) -> &'static str {
    match difficulty.constellation() {
        0..=1 => "lite",
        2..=3 => "full",
        _ => "ultra",
    }
}

// ═══════════════════════════════════════════════════════════════════
// Attention Report
// ═══════════════════════════════════════════════════════════════════

/// Complete attention feedback report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionReport {
    pub domain_scores: HashMap<String, f64>,
    pub intensity: String,
    pub recommended_focus: Vec<String>,
}

/// Generate attention report from trajectory and game meta.
pub fn generate_attention_report(trajectory: &Trajectory, meta: &GameMeta) -> AttentionReport {
    let domain_scores = compute_domain_scores(trajectory, meta);
    let intensity = compute_intensity(&meta.difficulty).to_string();

    // Recommended focus: domains with score > 0.5
    let recommended_focus: Vec<String> = domain_scores
        .iter()
        .filter(|(_, &v)| v > 0.5)
        .map(|(k, _)| k.clone())
        .collect();

    AttentionReport {
        domain_scores,
        intensity,
        recommended_focus,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trajectory(reward: f64, length: usize) -> Trajectory {
        Trajectory {
            steps: vec![],
            total_reward: reward,
            length,
        }
    }

    fn make_meta(skills: Vec<CognitiveSkill>, diff: Difficulty) -> GameMeta {
        GameMeta {
            target_skills: skills,
            difficulty: diff,
        }
    }

    #[test]
    fn test_domain_name_mapping() {
        assert_eq!(
            AttentionMapping::domain_name(&CognitiveSkill::Planning),
            "planning"
        );
        assert_eq!(
            AttentionMapping::domain_name(&CognitiveSkill::PatternRecognition),
            "pattern_match"
        );
        assert_eq!(
            AttentionMapping::domain_name(&CognitiveSkill::CodeReasoning),
            "code"
        );
    }

    #[test]
    fn test_compute_domain_scores_win() {
        let traj = make_trajectory(1.0, 10);
        let meta = make_meta(vec![CognitiveSkill::Planning], Difficulty::Tutorial);
        let scores = compute_domain_scores(&traj, &meta);
        let planning = scores.get("planning").unwrap();
        assert!(*planning > 0.3);
    }

    #[test]
    fn test_compute_domain_scores_loss() {
        let traj = make_trajectory(-1.0, 20);
        let meta = make_meta(vec![CognitiveSkill::Planning], Difficulty::Tutorial);
        let scores = compute_domain_scores(&traj, &meta);
        let planning = scores.get("planning").unwrap();
        assert!(*planning < 0.3);
    }

    #[test]
    fn test_compute_intensity() {
        assert_eq!(compute_intensity(&Difficulty::Tutorial), "lite");
        assert_eq!(compute_intensity(&Difficulty::Journeyman), "full");
        assert_eq!(compute_intensity(&Difficulty::Master), "ultra");
    }

    #[test]
    fn test_generate_attention_report() {
        let traj = make_trajectory(1.0, 10);
        let meta = make_meta(
            vec![CognitiveSkill::Planning, CognitiveSkill::PatternRecognition],
            Difficulty::Expert,
        );
        let report = generate_attention_report(&traj, &meta);
        assert_eq!(report.intensity, "full");
        assert!(report.domain_scores.contains_key("planning"));
        assert!(report.domain_scores.contains_key("pattern_match"));
    }

    #[test]
    fn test_all_domains_present() {
        let traj = make_trajectory(0.5, 5);
        let meta = make_meta(vec![], Difficulty::Tutorial);
        let scores = compute_domain_scores(&traj, &meta);
        for domain in AttentionMapping::all_domains() {
            assert!(scores.contains_key(domain));
        }
    }
}
