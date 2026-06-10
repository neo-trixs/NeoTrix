/// Intrinsic value system: curiosity, exploration drive, learning signals

/// Sources of intrinsic reward
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicRewardSource {
    Curiosity,
    KnowledgeGap,
    PredictionError,
    Novelty,
}

/// An intrinsic reward signal
#[derive(Debug, Clone)]
pub struct IntrinsicReward {
    pub source: IntrinsicRewardSource,
    pub value: f64,
    pub description: String,
}

impl IntrinsicReward {
    pub fn new(source: IntrinsicRewardSource, value: f64, description: &str) -> Self {
        Self { source, value: value.clamp(0.0, 1.0), description: description.to_string() }
    }
}

/// Compute curiosity reward from prediction error
pub fn curiosity_reward(prediction_error: f64) -> f64 {
    (prediction_error * 0.1).min(0.5)
}

/// Compute exploration drive from knowledge gap density
pub fn exploration_drive(knowledge_gaps: usize, total_known: usize) -> f64 {
    if total_known == 0 { return 0.5; }
    let ratio = knowledge_gaps as f64 / total_known as f64;
    (ratio * 0.3).min(0.5)
}

/// Compute novelty bonus for recent unique discoveries
pub fn novelty_bonus(unique_discoveries: usize, window: usize) -> f64 {
    if window == 0 { return 0.0; }
    let rate = unique_discoveries as f64 / window as f64;
    (rate * 0.2).min(0.3)
}

/// Aggregate all intrinsic reward signals
pub fn aggregate_intrinsic_reward(
    prediction_error: f64,
    knowledge_gaps: usize,
    total_known: usize,
    unique_discoveries: usize,
    window: usize,
) -> Vec<IntrinsicReward> {
    vec![
        IntrinsicReward::new(
            IntrinsicRewardSource::Curiosity,
            curiosity_reward(prediction_error),
            "curiosity from prediction error",
        ),
        IntrinsicReward::new(
            IntrinsicRewardSource::KnowledgeGap,
            exploration_drive(knowledge_gaps, total_known),
            "exploration drive from knowledge gaps",
        ),
        IntrinsicReward::new(
            IntrinsicRewardSource::Novelty,
            novelty_bonus(unique_discoveries, window),
            "novelty from recent discoveries",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curiosity_reward_bounds() {
        let r = curiosity_reward(10.0);
        assert!(r >= 0.0);
        assert!(r <= 0.5);
    }

    #[test]
    fn test_curiosity_reward_zero() {
        let r = curiosity_reward(0.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_exploration_drive_no_gaps() {
        let d = exploration_drive(0, 100);
        assert_eq!(d, 0.0);
    }

    #[test]
    fn test_exploration_drive_bounded() {
        let d = exploration_drive(1000, 1);
        assert!(d <= 0.5);
    }

    #[test]
    fn test_exploration_drive_unknown_zero() {
        let d = exploration_drive(5, 0);
        assert_eq!(d, 0.5);
    }

    #[test]
    fn test_novelty_bonus_bounds() {
        let n = novelty_bonus(10, 20);
        assert!(n >= 0.0);
        assert!(n <= 0.3);
    }

    #[test]
    fn test_novelty_bonus_zero_window() {
        let n = novelty_bonus(10, 0);
        assert_eq!(n, 0.0);
    }

    #[test]
    fn test_aggregate_contains_all_sources() {
        let rewards = aggregate_intrinsic_reward(2.0, 5, 100, 3, 50);
        assert_eq!(rewards.len(), 3);
        let sources: Vec<IntrinsicRewardSource> = rewards.iter().map(|r| r.source).collect();
        assert!(sources.contains(&IntrinsicRewardSource::Curiosity));
        assert!(sources.contains(&IntrinsicRewardSource::KnowledgeGap));
        assert!(sources.contains(&IntrinsicRewardSource::Novelty));
    }

    #[test]
    fn test_intrinsic_reward_clamped() {
        let r = IntrinsicReward::new(IntrinsicRewardSource::Curiosity, 5.0, "too high");
        assert!(r.value <= 1.0);
        let r = IntrinsicReward::new(IntrinsicRewardSource::Curiosity, -1.0, "too low");
        assert!(r.value >= 0.0);
    }
}
