use serde::{Deserialize, Serialize};
use crate::core::CapabilityVector;

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
    MetaCognition = 50,
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
    // 🆕 2026-05-29: Strix security absorption
    SecurityAttacks,
    // 🆕 2026-05-28: 10-repo absorption
    LiteParse,
    SmartSearch,
    AQBot,
    AionUi,
    CyberVerse,
    Hotpush,
    InfiniteCanvas,
    AutoDocxProofread,
    OpenSwe,
    // 🆕 2026-05-29: P2-1 8 project injection
    PiMonolith,
    ClawCode,
    HermesAgent,
    Bernstein,
    Mastra,
    Omi,
    Crush,
    QwenCode,
    // 🆕 2026-05-29: 7-repo absorption
    LlmWiki,
    // 🆕 2026-05-29: 花叔达尔文.skill — autonomous skill optimizer
    DarwinSkill,
    // 🆕 2026-05-29: Self-evolving skill papers + agent harness
    SkillOpt,
    MuseAutoskill,
    FeynmanAgent,
    AwesomeArchitecture,
    VulnGym,
    ZepMemory,
    HindsightMemory,
    CogneeMemory,
    SageMemory,
    ApexMem,
    LangMem,
    LettaMemory,
    // 🆕 2026-05-29: maigret — GitHub OSINT profile scanner
    Maigret,
    // 🆕 2026-05-29: taste-skill — multi-judge skill quality evaluation
    TasteSkill,
    // 🆕 2026-05-29: Understand-Anything — self-understanding system
    UnderstandAnything,
    // 🆕 2026-05-29: carbon-code — carbon-aware code optimization
    CarbonCode,
    // 🆕 2026-05-29: LLM-Arch — LLM architecture knowledge base
    LlmArch,
    // 🆕 2026-05-29: SPEAR — CodeAct APE prompt optimizer (arXiv 2605.26275)
    Spear,
    // 🆕 2026-05-29: SIA — Self Improving AI with H/W updates (arXiv 2605.27276)
    Sia,
    // 🆕 2026-05-29: SkillsGate — visual skill manager for AI agents
    SkillsGate,
    // 🆕 2026-05-29: Adam's Law — Textual Frequency Law (arXiv 2604.02176)
    AdamsLaw,
}
