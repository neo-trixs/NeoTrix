use serde::{Deserialize, Serialize};
use crate::core::CapabilityVector;

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

/// Categorizes the nature of a task for capability routing and performance prediction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    General = 0,
    Design = 1,
    CodeAnalysis = 2,
    CodeGeneration = 3,
    CodeReview = 4,
    Security = 5,
    Planning = 6,
    Reflection = 7,
    UIDesign = 8,
    Research = 9,
    Learning = 10,
    Debugging = 11,
}

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

/// Trait for objects that can provide domain-specific knowledge with capability vectors.
pub trait KnowledgeProvider {
    fn name(&self) -> &str;
    fn capability_vector(&self) -> CapabilityVector;
    fn source_weight(&self) -> f64;
}

/// A known external knowledge source that can be absorbed into the ReasoningBrain.
///
/// Each variant maps to a real project/tool and provides a CapabilityVector
/// representing its strengths across 23 core dimensions plus extension axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeSource {
    HeroUI,
    BaseUI,
    ArcUI,
    CortexUI,
    AgenticDS,
    DesignPhilosophy,
    Hyperframes,
    Betterleaks,
    YaoWebsecurity,
    Botasaurus,
    ReactDoctor,
    OpenPencil,
    AiTrader,
    SesameRobot,
    EverOS,
    MattPocockSkills,
    NestedLearning,
    AutonomousGoal,
    AwesomeDesignSkills,
    // 🆕 2026-05-15: 外部来源吸收
    DeepSeekTui,
    Codebuff,
    OpenClaude,
    Cairn,
    Orca,
    RedRun,
    AutonomousSpeedrunning,
    // 🆕 2026-05-15: Memory/自改进集群
    Synesis,
    MemOS,
    Reflexio,
    Mem0,
    Mnemosyne,
    OriMnemos,
    OPSD,
    AttentionMechanism,
    PatchFile,
    KeyVault,
    SealLoop,
    // 🆕 2026-05-23: HashCortX 融合吸收
    HashCortxAgents,
    HashCortxSecurity,
    HashCortxSwarm,
    HashCortxFailover,
    // 🆕 2026-05-24: Ancient Chinese cosmology / unified field theory
    HetuLuoshu,
    YijingBinary,
    FivePhasesGauge,
    ThreeCosmologies,
    HuainanziCalendar,
    ZhangHengSeismoscope,
    MawangduiAstronomy,
    ShaoYongCosmology,
    DayanNumber,
    // 🆕 2026-05-29: Adam's Law — Textual Frequency Law (arXiv 2604.02176)
    AdamsLaw,
    // 🆕 2026-05-30: Consciousness / VSA / JEPA 核心理论
    IntegratedInformationTheory,
    GlobalWorkspaceTheory,
    ActiveInference,
    VSAHyperdim,
    JEPAWorldModel,
    PredictiveCoding,
    OrchOR,
    AttentionSchema,
    // 🆕 2026-05-30: SIA — Self-Improving AI (arXiv 2605.27276)
    SiaHarnessUpdate,       // scaffold 改写能力
    SiaWeightUpdate,        // RL weight 更新能力
    SiaFeedbackLoop,        // 三体反馈循环架构
    // 🆕 2026-05-30: DGM-HyperAgents (arXiv 2603.19461, Meta FAIR)
    HyperAgents,            // 自指涉自我改进
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maturity_promote_candidate_to_reviewed() {
        assert_eq!(MaturityLevel::Candidate.promote(), Some(MaturityLevel::Reviewed));
    }

    #[test]
    fn test_maturity_promote_reviewed_to_validated() {
        assert_eq!(MaturityLevel::Reviewed.promote(), Some(MaturityLevel::Validated));
    }

    #[test]
    fn test_maturity_promote_validated_to_ground_truth() {
        assert_eq!(MaturityLevel::Validated.promote(), Some(MaturityLevel::GroundTruth));
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
            KnowledgeSource::HeroUI, KnowledgeSource::BaseUI, KnowledgeSource::ArcUI,
            KnowledgeSource::CortexUI, KnowledgeSource::AgenticDS,
        ];
        assert_eq!(sources.len(), 5);
    }
}
