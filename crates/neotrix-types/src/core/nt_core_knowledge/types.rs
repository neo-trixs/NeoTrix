use serde::{Deserialize, Serialize};
use crate::core::CapabilityVector;

/// Categorizes the nature of a task for capability routing and performance prediction.
///
/// **Canonical definition** — single source of truth for all NeoTrix layers.
/// Strategy: keep all 12 original variant names + add new variants from other modules.
/// No existing variant is deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    // ── 旧 12 variants (判别值保留) ──
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

    // ── 从 model_router 合并 ──
    Chat,           // = General 的子集，保留用于路由
    Math,           // 独立变体
    Creative,       // = Design 的子集
    DataProcessing, // 独立变体
    Multimodal,     // 独立变体

    // ── 从 resource_router 合并 ──
    SimpleQA,
    ComplexReasoning,
    CreativeWriting,
    KnowledgeRetrieval,
    DataAnalysis,

    // ── 从 generation_classifier 合并 ──
    Extraction,
    Summarization,
    ToolUse,

    // ── 从 universal_model 合并 ──
    Completion,
    Embedding,
    Reranking,
    ImageGeneration,
    AudioGeneration,
    VideoGeneration,

    // ── 从 god_agent 合并 ──
    Debugging,
    Architecture,
    Documentation,
    Testing,
    SystemAdmin,
    FileOperations,
    AgentTask,

    // ── 扩展 ──
    Custom,
}

impl TaskType {
    /// Returns a static string representation of the task type.
    pub fn as_str(&self) -> &'static str {
        match self {
            // 旧 12
            TaskType::General => "general",
            TaskType::Design => "design",
            TaskType::CodeAnalysis => "code_analysis",
            TaskType::CodeGeneration => "code_generation",
            TaskType::CodeReview => "code_review",
            TaskType::Security => "security",
            TaskType::Planning => "planning",
            TaskType::Reflection => "reflection",
            TaskType::UIDesign => "ui_design",
            TaskType::Research => "research",
            TaskType::Learning => "learning",
            TaskType::MetaCognition => "meta_cognition",
            // model_router
            TaskType::Chat => "chat",
            TaskType::Math => "math",
            TaskType::Creative => "creative",
            TaskType::DataProcessing => "data_processing",
            TaskType::Multimodal => "multimodal",
            // resource_router
            TaskType::SimpleQA => "simple_qa",
            TaskType::ComplexReasoning => "complex_reasoning",
            TaskType::CreativeWriting => "creative_writing",
            TaskType::KnowledgeRetrieval => "knowledge_retrieval",
            TaskType::DataAnalysis => "data_analysis",
            // generation_classifier
            TaskType::Extraction => "extraction",
            TaskType::Summarization => "summarization",
            TaskType::ToolUse => "tool_use",
            // universal_model
            TaskType::Completion => "completion",
            TaskType::Embedding => "embedding",
            TaskType::Reranking => "reranking",
            TaskType::ImageGeneration => "image_generation",
            TaskType::AudioGeneration => "audio_generation",
            TaskType::VideoGeneration => "video_generation",
            // god_agent
            TaskType::Debugging => "debugging",
            TaskType::Architecture => "architecture",
            TaskType::Documentation => "documentation",
            TaskType::Testing => "testing",
            TaskType::SystemAdmin => "system_admin",
            TaskType::FileOperations => "file_operations",
            TaskType::AgentTask => "agent_task",
            // extension
            TaskType::Custom => "custom",
        }
    }

    /// Classifies task type from a text description (heuristic).
    pub fn from_description(text: &str) -> Self {
        let lower = text.to_lowercase();
        if lower.contains("code") || lower.contains("implement") || lower.contains("build") {
            TaskType::CodeGeneration
        } else if lower.contains("review") || lower.contains("audit") {
            TaskType::CodeReview
        } else if lower.contains("design") || lower.contains("ui") || lower.contains("layout") {
            TaskType::Design
        } else if lower.contains("debug") || lower.contains("fix") || lower.contains("error") {
            TaskType::Debugging
        } else if lower.contains("test") || lower.contains("spec") {
            TaskType::Testing
        } else if lower.contains("doc") || lower.contains("readme") {
            TaskType::Documentation
        } else if lower.contains("research") || lower.contains("investigate") {
            TaskType::Research
        } else if lower.contains("plan") || lower.contains("architect") {
            TaskType::Planning
        } else if lower.contains("security") || lower.contains("vuln") {
            TaskType::Security
        } else if lower.contains("summar") {
            TaskType::Summarization
        } else if lower.contains("embed") {
            TaskType::Embedding
        } else {
            TaskType::General
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_type_as_str_covers_all_variants() {
        assert_eq!(TaskType::General.as_str(), "general");
        assert_eq!(TaskType::CodeGeneration.as_str(), "code_generation");
        assert_eq!(TaskType::MetaCognition.as_str(), "meta_cognition");
        assert_eq!(TaskType::Chat.as_str(), "chat");
        assert_eq!(TaskType::ComplexReasoning.as_str(), "complex_reasoning");
        assert_eq!(TaskType::Summarization.as_str(), "summarization");
        assert_eq!(TaskType::Embedding.as_str(), "embedding");
        assert_eq!(TaskType::Debugging.as_str(), "debugging");
        assert_eq!(TaskType::AgentTask.as_str(), "agent_task");
        assert_eq!(TaskType::Custom.as_str(), "custom");
    }

    #[test]
    fn task_type_from_description() {
        assert_eq!(TaskType::from_description("implement the parser"), TaskType::CodeGeneration);
        assert_eq!(TaskType::from_description("review this PR"), TaskType::CodeReview);
        assert_eq!(TaskType::from_description("design the UI"), TaskType::Design);
        assert_eq!(TaskType::from_description("fix the bug"), TaskType::Debugging);
        assert_eq!(TaskType::from_description("write tests"), TaskType::Testing);
        assert_eq!(TaskType::from_description("research papers"), TaskType::Research);
        assert_eq!(TaskType::from_description("general task"), TaskType::General);
    }
}
