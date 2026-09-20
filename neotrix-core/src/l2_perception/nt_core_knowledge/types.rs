use serde::{Deserialize, Serialize};

/// Maturity level of a KnowledgeSource, per TENSA multi-fidelity epistemology.
///
/// Progression: Candidate → Reviewed → Validated → GroundTruth.
/// Each level maps to a confidence score used in consolidated knowledge queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MaturityLevel {
    Candidate,
    Reviewed,
    Validated,
    GroundTruth,
}

impl MaturityLevel {
    /// Promote one level. Returns `None` if already `GroundTruth`.
    pub fn promote(&self) -> Option<Self> {
        match self {
            MaturityLevel::Candidate => Some(MaturityLevel::Reviewed),
            MaturityLevel::Reviewed => Some(MaturityLevel::Validated),
            MaturityLevel::Validated => Some(MaturityLevel::GroundTruth),
            MaturityLevel::GroundTruth => None,
        }
    }

    /// Map maturity to a numeric confidence in [0.0, 1.0].
    pub fn confidence(&self) -> f64 {
        match self {
            MaturityLevel::Candidate => 0.25,
            MaturityLevel::Reviewed => 0.5,
            MaturityLevel::Validated => 0.75,
            MaturityLevel::GroundTruth => 1.0,
        }
    }
}

// Re-export TaskType from neotrix-types to avoid duplicate type definitions
pub use neotrix_types::core::TaskType;

/// Origin of a reward signal — external (verification tools, user) or internal (self-evaluated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardSource {
    External,
    Internal,
}

impl RewardSource {
    /// Priority multiplier: External rewards count 2x vs Internal.
    pub fn priority_multiplier(&self) -> f64 {
        match self {
            RewardSource::External => 2.0,
            RewardSource::Internal => 1.0,
        }
    }
}

/// 情感奖励上下文 (affective → SEAL reward, 设计: docs/1-DESIGN/affective-reward-context.md)。
/// 语义: 用户情感观测作为奖励的**引导信号, 不做裁判** — 情感幅度上限
/// `signal_weight` 被 `combine_reward_with_affective` cap 至 0.3, 防止「讨好用户」
/// 替代「验证通过」成为进化主信号。冷启动抑制: `interactions < 3` 时情感信号不生效。
#[derive(Debug, Clone, Copy)]
pub struct AffectiveFeedback {
    pub valence: f64,          // 0..1, 用户情绪愉悦度 (UserAffectSnapshot::valence)
    pub arousal: f64,          // 0..1, 唤醒度 (UserAffectSnapshot::arousal)
    pub stage: u8,             // RelationshipStage 序数 (0=Stranger..4=Bond)
    pub interactions: u32,     // 交互计数 (冷启动门限 <3 抑制)
    pub signal_weight: f64,    // 0..1, 情感信号占总奖励比例 (cap 0.3)
}

/// Trait for objects that can provide domain-specific knowledge with capability vectors.
/// Re-exported from L0; implemented here for `KnowledgeSource`.
pub use crate::l0_substrate::nt_core_substrate_types::KnowledgeProvider;

/// A known external knowledge source that can be absorbed into the ReasoningBrain.
/// Re-exported from L0 to enforce substrate invariant (L0 defines, L2 extends).
pub use crate::l0_substrate::nt_core_substrate_types::KnowledgeSource;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maturity_promote_candidate_to_reviewed() {
        assert_eq!(
            MaturityLevel::Candidate.promote(),
            Some(MaturityLevel::Reviewed)
        );
    }

    #[test]
    fn test_maturity_promote_reviewed_to_validated() {
        assert_eq!(
            MaturityLevel::Reviewed.promote(),
            Some(MaturityLevel::Validated)
        );
    }

    #[test]
    fn test_maturity_promote_validated_to_ground_truth() {
        assert_eq!(
            MaturityLevel::Validated.promote(),
            Some(MaturityLevel::GroundTruth)
        );
    }

    #[test]
    fn test_maturity_promote_ground_truth_returns_none() {
        assert_eq!(MaturityLevel::GroundTruth.promote(), None);
    }

    #[test]
    fn test_maturity_confidence_values() {
        assert!((MaturityLevel::Candidate.confidence() - 0.25).abs() < 1e-9);
        assert!((MaturityLevel::Reviewed.confidence() - 0.5).abs() < 1e-9);
        assert!((MaturityLevel::Validated.confidence() - 0.75).abs() < 1e-9);
        assert!((MaturityLevel::GroundTruth.confidence() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_reward_source_external_priority_multiplier() {
        assert!((RewardSource::External.priority_multiplier() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_reward_source_internal_priority_multiplier() {
        assert!((RewardSource::Internal.priority_multiplier() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_task_type_variants_have_distinct_discriminants() {
        assert_ne!(TaskType::General as u8, TaskType::Design as u8);
        assert_ne!(TaskType::CodeAnalysis as u8, TaskType::CodeGeneration as u8);
    }

    #[test]
    fn test_maturity_level_ordering() {
        assert!(MaturityLevel::Candidate < MaturityLevel::Reviewed);
        assert!(MaturityLevel::Reviewed < MaturityLevel::Validated);
        assert!(MaturityLevel::Validated < MaturityLevel::GroundTruth);
    }

    #[test]
    fn test_knowledge_source_count() {
        let sources = vec![
            KnowledgeSource::HeroUI,
            KnowledgeSource::BaseUI,
            KnowledgeSource::ArcUI,
            KnowledgeSource::CortexUI,
            KnowledgeSource::AgenticDS,
        ];
        assert_eq!(sources.len(), 5);
    }
}
