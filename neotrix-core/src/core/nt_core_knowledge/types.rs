use crate::core::CapabilityVector;
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

pub use crate::core::nt_core_shared_types::TaskType;

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
    SiaHarnessUpdate, // scaffold 改写能力
    SiaWeightUpdate,  // RL weight 更新能力
    SiaFeedbackLoop,  // 三体反馈循环架构
    // 🆕 2026-05-30: DGM-HyperAgents (arXiv 2603.19461, Meta FAIR)
    HyperAgents, // 自指涉自我改进
    // 🆕 2026-06-10: Social media feed (X, Reddit, etc.) — negentropy scored
    SocialFeed,
    // 🆕 2026-06-17: alphaXiv research discussion platform
    AlphaXiv,
    // 🆕 2026-06-17: HarnessX — composable adaptive agent harness (arXiv 2606.14249)
    HarnessX,
    // 🆕 2026-06-17: DeepMind AGI→ASI pathway framework (arXiv 2606.12683)
    AgiToAsi,
    // 🆕 2026-06-17: MiniMax Sparse Attention — blockwise sparse attention (arXiv 2606.13392)
    MsaSparseAttention,
    // 🆕 2026-06-17: ExpRL — exploratory RL for LLM mid-training (arXiv 2606.17024)
    ExpRl,
    // 🆕 2026-06-17: VibeThinker-3B — small model verifiable reasoning (arXiv 2606.16140)
    VibeThinker3B,
    // 🆕 2026-06-17: GLM-5.2 — long-horizon agentic coding (Z.ai)
    Glm52,
    // 🆕 2026-06-17: Qwen-RobotWorld — embodied world model (arXiv 2606.17030)
    QwenRobotWorld,
    // 🆕 2026-06-17: VisualClaw — real-time personalized physical agent (arXiv 2606.16295)
    VisualClaw,
    // 🆕 2026-06-23: Vercel Geist — design token system & paradigm
    Geist,
    // 🆕 2026-06-23: VisualTasteLab — VI-first brand-to-design workflow
    VisualTasteLab,
    // 🆕 2026-06-23: DecantrDesign — 3-layer guard rules (DECANTR/scafford/section)
    DecantrDesign,
    // 🆕 2026-06-23: UIReasoning — 11-dimension UI design reasoning framework
    UIReasoning,
    // 🆕 2026-06-23: UISkill — 5-mode design system generation (architect/build/theme/motion/audit)
    UISkill,
}

/// Source of a knowledge entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSourceType {
    Wikipedia,
    ArXiv,
    SemanticScholar,
    GitHub,
    Book,
    WebPage,
    KnowledgeBase,
    UserInput,
    Inferred,
    PdfLocal,
    /// 用户从对话中收藏的书签URL — 可由 BookmarkManager 提升而来
    Bookmark,
}

/// A knowledge entry with metadata for forgetting/importance scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub body: String,
    pub summary: String,
    pub source: KnowledgeSourceType,
    pub source_url: String,
    pub tags: Vec<String>,
    pub dimensions: Vec<String>,
    pub embedding: Option<Vec<f64>>,
    pub confidence: f64,
    pub importance: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub access_count: u64,
    pub related_ids: Vec<String>,
    pub provenance_hash: Option<[u8; 32]>,
    pub cross_references: Vec<(String, [u8; 32])>,
    /// Evidence record IDs linking this entry to verifiable sources
    pub evidence_ids: Vec<u64>,
    /// Bi-temporal validity: valid_from (inclusive) — None means unbounded past
    pub valid_from: Option<i64>,
    /// Bi-temporal validity: valid_to (inclusive) — None means unbounded future
    pub valid_to: Option<i64>,
}

impl KnowledgeEntry {
    /// Create a new knowledge entry with minimal required fields.
    pub fn new(title: &str, body: &str, source: KnowledgeSourceType, source_url: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            body: body.to_string(),
            summary: body.chars().take(300).collect(),
            source,
            source_url: source_url.to_string(),
            tags: Vec::new(),
            dimensions: Vec::new(),
            embedding: None,
            confidence: 0.7,
            importance: 0.5,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            related_ids: Vec::new(),
            provenance_hash: None,
            cross_references: Vec::new(),
            evidence_ids: Vec::new(),
            valid_from: None,
            valid_to: None,
        }
    }
}

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
