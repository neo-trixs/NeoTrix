//! Emotion feedback from game outcomes.
//!
//! Maps game results (win/loss/draw) and difficulty to appraisal signals
//! following the emotion-as-appraisal pattern (Lazarus, Scherer).

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Types (self-contained)
// ═══════════════════════════════════════════════════════════════════

/// Episode result summary — mirrors the public fields we need.
#[derive(Debug, Clone)]
pub struct EpisodeResult {
    pub score: f64,
    pub trajectory_reward: f64,
    pub turns_played: usize,
}

/// Difficulty tier (constellation levels 0-5).
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

// ═══════════════════════════════════════════════════════════════════
// Appraisal
// ═══════════════════════════════════════════════════════════════════

/// Appraisal signal — three dimensions following Scherer's CPM model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalSignal {
    /// Novelty: how unexpected was the outcome? [0, 1]
    pub novelty: f64,
    /// Goal conduciveness: did the outcome help or hinder goals? [0, 1]
    pub goal_conduciveness: f64,
    /// Coping potential: agent's perceived ability to handle the outcome. [0, 1]
    pub coping: f64,
}

/// Determine outcome category from trajectory reward.
fn classify_outcome(reward: f64) -> &'static str {
    if reward > 0.0 {
        "win"
    } else if reward < 0.0 {
        "loss"
    } else {
        "draw"
    }
}

/// Compute appraisal from episode result and difficulty.
pub fn compute_appraisal(result: &EpisodeResult, difficulty: &Difficulty) -> AppraisalSignal {
    let outcome = classify_outcome(result.trajectory_reward);
    let (novelty, goal_conduciveness, coping) = match outcome {
        "win" => (0.3, 0.9, 0.8),
        "loss" => (0.2, 0.1, 0.3),
        _ => (0.1, 0.5, 0.5),
    };

    // Difficulty scaling: higher constellation boosts confidence on win
    let diff_bonus = difficulty.constellation() as f64 * 0.03;
    let coping_scaled = if outcome == "win" {
        (coping + diff_bonus).min(1.0)
    } else if outcome == "loss" {
        (coping - diff_bonus * 0.5).max(0.0)
    } else {
        coping
    };

    AppraisalSignal {
        novelty,
        goal_conduciveness,
        coping: coping_scaled,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Pressure
// ═══════════════════════════════════════════════════════════════════

/// Pressure signal for urgency during high-turn games.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureSignal {
    /// Normalized turns elapsed [0, 1].
    pub turn_ratio: f64,
    /// Pressure intensity: turn_ratio² (quadratic ramp).
    pub intensity: f64,
    /// Whether the game is in the "time pressure" zone (>70% turns used).
    pub in_pressure_zone: bool,
}

/// Compute pressure from trajectory and max turns.
pub fn compute_pressure(turns_played: usize, max_turns: usize) -> PressureSignal {
    if max_turns == 0 {
        return PressureSignal {
            turn_ratio: 0.0,
            intensity: 0.0,
            in_pressure_zone: false,
        };
    }
    let ratio = (turns_played as f64 / max_turns as f64).min(1.0);
    PressureSignal {
        turn_ratio: ratio,
        intensity: ratio * ratio,
        in_pressure_zone: ratio > 0.7,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Feedback Report
// ═══════════════════════════════════════════════════════════════════

/// Emotion label mapped from appraisal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionLabel {
    Neutral,
    Joy,
    Sadness,
    Anger,
    Fear,
    Trust,
    Disgust,
    Surprise,
    Anticipation,
    Confused,
    Thinking,
}

impl EmotionLabel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Joy => "joy",
            Self::Sadness => "sadness",
            Self::Anger => "anger",
            Self::Fear => "fear",
            Self::Trust => "trust",
            Self::Disgust => "disgust",
            Self::Surprise => "surprise",
            Self::Anticipation => "anticipation",
            Self::Confused => "confused",
            Self::Thinking => "thinking",
        }
    }
}

/// Map appraisal to dominant emotion label.
fn appraisal_to_emotion(appraisal: &AppraisalSignal) -> EmotionLabel {
    let gc = appraisal.goal_conduciveness;
    let cop = appraisal.coping;
    let nov = appraisal.novelty;

    if gc > 0.7 && cop > 0.7 {
        EmotionLabel::Joy
    } else if gc < 0.3 && cop < 0.3 {
        EmotionLabel::Sadness
    } else if gc < 0.3 && cop > 0.5 {
        EmotionLabel::Anger
    } else if gc < 0.3 && cop < 0.5 {
        EmotionLabel::Fear
    } else if nov > 0.7 {
        EmotionLabel::Surprise
    } else if gc > 0.5 && cop > 0.5 {
        EmotionLabel::Trust
    } else if gc > 0.5 && cop <= 0.5 {
        EmotionLabel::Anticipation
    } else {
        EmotionLabel::Neutral
    }
}

/// Complete emotion feedback report for a game episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackReport {
    pub emotion_label: EmotionLabel,
    pub appraisal: AppraisalSignal,
    pub pressure: PressureSignal,
    /// Estimated phi change: positive for wins, negative for losses.
    pub phi_delta: f64,
}

/// Generate a complete feedback report from a trajectory and difficulty.
pub fn generate_feedback(
    trajectory_reward: f64,
    turns_played: usize,
    max_turns: usize,
    difficulty: &Difficulty,
) -> FeedbackReport {
    let result = EpisodeResult {
        score: trajectory_reward,
        trajectory_reward,
        turns_played,
    };
    let appraisal = compute_appraisal(&result, difficulty);
    let pressure = compute_pressure(turns_played, max_turns);
    let emotion_label = appraisal_to_emotion(&appraisal);

    let phi_delta = if trajectory_reward > 0.0 {
        0.05 + difficulty.constellation() as f64 * 0.01
    } else if trajectory_reward < 0.0 {
        -0.03 - difficulty.constellation() as f64 * 0.005
    } else {
        0.0
    };

    FeedbackReport {
        emotion_label,
        appraisal,
        pressure,
        phi_delta,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appraisal_win() {
        let result = EpisodeResult {
            score: 1.0,
            trajectory_reward: 1.0,
            turns_played: 10,
        };
        let app = compute_appraisal(&result, &Difficulty::Tutorial);
        assert_eq!(app.novelty, 0.3);
        assert_eq!(app.goal_conduciveness, 0.9);
        assert_eq!(app.coping, 0.8);
    }

    #[test]
    fn test_appraisal_loss() {
        let result = EpisodeResult {
            score: 0.0,
            trajectory_reward: -1.0,
            turns_played: 20,
        };
        let app = compute_appraisal(&result, &Difficulty::Tutorial);
        assert_eq!(app.novelty, 0.2);
        assert_eq!(app.goal_conduciveness, 0.1);
        assert_eq!(app.coping, 0.3);
    }

    #[test]
    fn test_appraisal_draw() {
        let result = EpisodeResult {
            score: 0.5,
            trajectory_reward: 0.0,
            turns_played: 15,
        };
        let app = compute_appraisal(&result, &Difficulty::Tutorial);
        assert_eq!(app.novelty, 0.1);
        assert_eq!(app.goal_conduciveness, 0.5);
        assert_eq!(app.coping, 0.5);
    }

    #[test]
    fn test_difficulty_scaling_win() {
        let result = EpisodeResult {
            score: 1.0,
            trajectory_reward: 1.0,
            turns_played: 10,
        };
        let low = compute_appraisal(&result, &Difficulty::Tutorial);
        let high = compute_appraisal(&result, &Difficulty::Master);
        assert!(high.coping > low.coping);
    }

    #[test]
    fn test_pressure_basic() {
        let p = compute_pressure(5, 10);
        assert!((p.turn_ratio - 0.5).abs() < 1e-10);
        assert!((p.intensity - 0.25).abs() < 1e-10);
        assert!(!p.in_pressure_zone);
    }

    #[test]
    fn test_pressure_in_zone() {
        let p = compute_pressure(8, 10);
        assert!(p.in_pressure_zone);
    }

    #[test]
    fn test_pressure_zero_max() {
        let p = compute_pressure(5, 0);
        assert_eq!(p.turn_ratio, 0.0);
    }

    #[test]
    fn test_generate_feedback_win() {
        let report = generate_feedback(1.0, 8, 10, &Difficulty::Expert);
        assert_eq!(report.emotion_label, EmotionLabel::Joy);
        assert!(report.phi_delta > 0.0);
    }

    #[test]
    fn test_generate_feedback_loss() {
        let report = generate_feedback(-1.0, 20, 25, &Difficulty::Tutorial);
        assert!(matches!(
            report.emotion_label,
            EmotionLabel::Sadness | EmotionLabel::Fear | EmotionLabel::Anger
        ));
        assert!(report.phi_delta < 0.0);
    }

    #[test]
    fn test_generate_feedback_draw() {
        let report = generate_feedback(0.0, 15, 20, &Difficulty::Journeyman);
        assert_eq!(report.phi_delta, 0.0);
    }
}
