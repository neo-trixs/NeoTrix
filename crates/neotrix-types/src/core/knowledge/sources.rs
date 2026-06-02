use crate::core::CapabilityVector;
use crate::core::knowledge::KnowledgeSource;
use super::vectors_group_a;
use super::vectors_group_b;

impl KnowledgeSource {
    /// Human-readable name of this knowledge source (GitHub repo path or project name).
    pub fn name(&self) -> &'static str {
        match self {
            KnowledgeSource::HeroUI => "heroui-inc/heroui",
            KnowledgeSource::BaseUI => "mui/base-ui",
            KnowledgeSource::ArcUI => "arc-lo/ui",
            KnowledgeSource::CortexUI => "llcortex/cortexui",
            KnowledgeSource::AgenticDS => "aa-on-ai/agentic-design-system",
            KnowledgeSource::DesignPhilosophy => "huashu-design",
            KnowledgeSource::Hyperframes => "heygen-com/hyperframes",
            KnowledgeSource::Betterleaks => "betterleaks/betterleaks",
            KnowledgeSource::YaoWebsecurity => "yaojingang/yao-websecurity-skill",
            KnowledgeSource::Botasaurus => "omkarcloud/botasaurus",
            KnowledgeSource::ReactDoctor => "millionco/react-doctor",
            KnowledgeSource::OpenPencil => "ZSeven-W/openpencil",
            KnowledgeSource::AiTrader => "HKUDS/AI-Trader",
            KnowledgeSource::SesameRobot => "dorianborian/sesame-robot",
            KnowledgeSource::EverOS => "EverMind-AI/EverOS",
            KnowledgeSource::MattPocockSkills => "mattpocock/skills",
            KnowledgeSource::NestedLearning => "google-research/nested-learning",
            KnowledgeSource::AutonomousGoal => "openai/codex-cli-goal",
            KnowledgeSource::AwesomeDesignSkills => "awesome-design-skills/awesome-design-skills",
            KnowledgeSource::DeepSeekTui => "Hmbown/DeepSeek-TUI",
            KnowledgeSource::Codebuff => "CodebuffAI/codebuff",
            KnowledgeSource::OpenClaude => "Gitlawb/openclaude",
            KnowledgeSource::Cairn => "oritera/Cairn",
            KnowledgeSource::Orca => "stablyai/orca",
            KnowledgeSource::RedRun => "blacklanternsecurity/red-run",
            KnowledgeSource::AutonomousSpeedrunning => "PrimeIntellect-ai/experiments-autonomous-speedrunning",
            KnowledgeSource::Synesis => "andreycpu/synesis",
            KnowledgeSource::MemOS => "MemTensor/MemOS",
            KnowledgeSource::Reflexio => "ReflexioAI/reflexio",
            KnowledgeSource::Mem0 => "mem0ai/mem0",
            KnowledgeSource::Mnemosyne => "28naem-del/mnemosyne",
            KnowledgeSource::OriMnemos => "aayoawoyemi/ori-mnemos",
            KnowledgeSource::OPSD => "siyan-zhao/OPSD",
            KnowledgeSource::AttentionMechanism => "attention-mechanism-vsa",
            KnowledgeSource::PatchFile => "patch-file-editor",
            KnowledgeSource::KeyVault => "keyvault-dual-storage",
            KnowledgeSource::SealLoop => "seal-self-iteration",
            KnowledgeSource::HashCortxAgents => "hashcortx-agent-templates",
            KnowledgeSource::HashCortxSecurity => "hashcortx-security-guard",
            KnowledgeSource::HashCortxSwarm => "hashcortx-swarm-patterns",
            KnowledgeSource::HashCortxFailover => "hashcortx-failover-routing",
            KnowledgeSource::HetuLuoshu => "河图洛书 (Yellow River Map / Luo River Writing)",
            KnowledgeSource::YijingBinary => "易经二进制 (Yijing Binary Encoding — Shao Yong / Leibniz / Bouvet)",
            KnowledgeSource::FivePhasesGauge => "五行规范场 (Five Phases Gauge Theory — Standard Model mapping)",
            KnowledgeSource::ThreeCosmologies => "三大宇宙论 (Gai Tian / Hun Tian / Xuan Ye cosmology)",
            KnowledgeSource::HuainanziCalendar => "淮南子历法 (Huainanzi astronomical calendar / universal embedding)",
            KnowledgeSource::ZhangHengSeismoscope => "张衡地动仪 (Zhang Heng seismoscope — 2025 scientific restoration)",
            KnowledgeSource::MawangduiAstronomy => "马王堆天文 (Mawangdui silk manuscripts — 五星占/彗星图/导引图)",
            KnowledgeSource::ShaoYongCosmology => "皇极经世 (Shao Yong's cosmic cycle — 129600-year cosmology)",
            KnowledgeSource::DayanNumber => "大衍之数 (Dayan number 50→49 — Zhao Shuang / Jiao Weifang proof)",
            KnowledgeSource::SecurityAttacks => "usestrix/strix",
            KnowledgeSource::LiteParse => "run-llama/liteparse",
            KnowledgeSource::SmartSearch => "smartsearch-ai/search-router",
            KnowledgeSource::AQBot => "aqbot-ai/aqbot-gateway",
            KnowledgeSource::AionUi => "aion-ui/aion-scheduler",
            KnowledgeSource::CyberVerse => "dsd2077/CyberVerse",
            KnowledgeSource::Hotpush => "JackyST0/hotpush",
            KnowledgeSource::InfiniteCanvas => "basketikun/infinite-canvas",
            KnowledgeSource::AutoDocxProofread => "autodocx/autodocx-proofread",
            KnowledgeSource::OpenSwe => "langchain-ai/open-swe",
            KnowledgeSource::PiMonolith => "pi-mono/pi",
            KnowledgeSource::ClawCode => "clawcode/clawcode",
            KnowledgeSource::HermesAgent => "hermes-agent/hermes-agent",
            KnowledgeSource::Bernstein => "bernstein/bernstein",
            KnowledgeSource::Mastra => "mastra/mastra",
            KnowledgeSource::Omi => "omi/omi",
            KnowledgeSource::Crush => "crush-tui/crush",
            KnowledgeSource::QwenCode => "qwen-code/qwen-code",
            KnowledgeSource::LlmWiki => "nashsu/llm_wiki",
            KnowledgeSource::DarwinSkill => "alchaincyf/darwin-skill",
            // vectors_group_b variants
            KnowledgeSource::SkillOpt => "skill-opt",
            KnowledgeSource::MuseAutoskill => "muse-autoskill",
            KnowledgeSource::FeynmanAgent => "feynman-agent",
            KnowledgeSource::AwesomeArchitecture => "awesome-architecture",
            KnowledgeSource::VulnGym => "vuln-gym",
            KnowledgeSource::ZepMemory => "zep-memory",
            KnowledgeSource::HindsightMemory => "hindsight-memory",
            KnowledgeSource::CogneeMemory => "cognee-memory",
            KnowledgeSource::SageMemory => "sage-memory",
            KnowledgeSource::ApexMem => "apex-mem",
            KnowledgeSource::LangMem => "lang-mem",
            KnowledgeSource::LettaMemory => "letta-memory",
            KnowledgeSource::Maigret => "maigret",
            KnowledgeSource::TasteSkill => "taste-skill",
            KnowledgeSource::UnderstandAnything => "understand-anything",
            KnowledgeSource::CarbonCode => "carbon-code",
            KnowledgeSource::LlmArch => "llm-arch",
            KnowledgeSource::Spear => "spear-arxiv-2605-26275",
            KnowledgeSource::Sia => "sia-arxiv-2605-27276",
            KnowledgeSource::SkillsGate => "skillsgate/skillsgate",
            KnowledgeSource::AdamsLaw => "Adam's Law (arXiv 2604.02176) — Textual Frequency Law",
        }
    }

    /// Returns a 23-dim CapabilityVector representing this source's knowledge profile.
    pub fn capability_vector(&self) -> CapabilityVector {
        match vectors_group_a::capability_vector_group_a(self) {
            Some(cv) => cv,
            None => vectors_group_b::capability_vector_group_b(self),
        }
    }

    /// Returns all known knowledge sources as a flat vector.
    pub fn all() -> Vec<KnowledgeSource> {
        vec![
            KnowledgeSource::HeroUI, KnowledgeSource::BaseUI,
            KnowledgeSource::ArcUI, KnowledgeSource::CortexUI,
            KnowledgeSource::AgenticDS, KnowledgeSource::DesignPhilosophy,
            KnowledgeSource::Hyperframes, KnowledgeSource::Betterleaks,
            KnowledgeSource::YaoWebsecurity, KnowledgeSource::Botasaurus,
            KnowledgeSource::ReactDoctor, KnowledgeSource::OpenPencil,
            KnowledgeSource::AiTrader, KnowledgeSource::SesameRobot,
            KnowledgeSource::EverOS, KnowledgeSource::MattPocockSkills,
            KnowledgeSource::NestedLearning, KnowledgeSource::AutonomousGoal,
            KnowledgeSource::AwesomeDesignSkills, KnowledgeSource::DeepSeekTui,
            KnowledgeSource::Codebuff, KnowledgeSource::OpenClaude,
            KnowledgeSource::Cairn, KnowledgeSource::Orca,
            KnowledgeSource::RedRun, KnowledgeSource::AutonomousSpeedrunning,
            KnowledgeSource::Synesis, KnowledgeSource::MemOS,
            KnowledgeSource::Reflexio, KnowledgeSource::Mem0,
            KnowledgeSource::Mnemosyne, KnowledgeSource::OriMnemos,
            KnowledgeSource::OPSD, KnowledgeSource::AttentionMechanism,
            KnowledgeSource::PatchFile, KnowledgeSource::KeyVault,
            KnowledgeSource::SealLoop,
            KnowledgeSource::HashCortxAgents, KnowledgeSource::HashCortxSecurity,
            KnowledgeSource::HashCortxSwarm, KnowledgeSource::HashCortxFailover,
            KnowledgeSource::HetuLuoshu, KnowledgeSource::YijingBinary,
            KnowledgeSource::FivePhasesGauge, KnowledgeSource::ThreeCosmologies,
            KnowledgeSource::HuainanziCalendar, KnowledgeSource::ZhangHengSeismoscope,
            KnowledgeSource::MawangduiAstronomy, KnowledgeSource::ShaoYongCosmology,
            KnowledgeSource::DayanNumber,
            KnowledgeSource::SecurityAttacks,
            KnowledgeSource::LiteParse, KnowledgeSource::SmartSearch,
            KnowledgeSource::AQBot, KnowledgeSource::AionUi,
            KnowledgeSource::CyberVerse, KnowledgeSource::Hotpush,
            KnowledgeSource::InfiniteCanvas, KnowledgeSource::AutoDocxProofread,
            KnowledgeSource::OpenSwe,
            KnowledgeSource::PiMonolith,
            KnowledgeSource::ClawCode,
            KnowledgeSource::HermesAgent,
            KnowledgeSource::Bernstein,
            KnowledgeSource::Mastra,
            KnowledgeSource::Omi,
            KnowledgeSource::Crush,
            KnowledgeSource::QwenCode,
            KnowledgeSource::LlmWiki,
            KnowledgeSource::DarwinSkill,
            // vectors_group_b variants
            KnowledgeSource::SkillOpt,
            KnowledgeSource::MuseAutoskill,
            KnowledgeSource::FeynmanAgent,
            KnowledgeSource::AwesomeArchitecture,
            KnowledgeSource::VulnGym,
            KnowledgeSource::ZepMemory,
            KnowledgeSource::HindsightMemory,
            KnowledgeSource::CogneeMemory,
            KnowledgeSource::SageMemory,
            KnowledgeSource::ApexMem,
            KnowledgeSource::LangMem,
            KnowledgeSource::LettaMemory,
            KnowledgeSource::Maigret,
            KnowledgeSource::TasteSkill,
            KnowledgeSource::UnderstandAnything,
            KnowledgeSource::CarbonCode,
            KnowledgeSource::LlmArch,
            KnowledgeSource::Spear,
            KnowledgeSource::Sia,
            KnowledgeSource::SkillsGate,
            KnowledgeSource::AdamsLaw,
        ]
    }

    /// Priority weight for this source (0.0–1.0). Higher = more influential during absorption.
    pub fn source_weight(&self) -> f64 {
        match self {
            KnowledgeSource::DesignPhilosophy => 1.0,
            KnowledgeSource::BaseUI => 0.95,
            KnowledgeSource::HeroUI => 0.9,
            KnowledgeSource::ArcUI => 0.8,
            KnowledgeSource::CortexUI => 0.85,
            KnowledgeSource::AgenticDS => 0.8,
            KnowledgeSource::Hyperframes => 0.85,
            KnowledgeSource::Betterleaks => 0.9,
            KnowledgeSource::YaoWebsecurity => 0.85,
            KnowledgeSource::Botasaurus => 0.8,
            KnowledgeSource::ReactDoctor => 0.95,
            KnowledgeSource::OpenPencil => 0.85,
            KnowledgeSource::AiTrader => 0.75,
            KnowledgeSource::SesameRobot => 0.7,
            KnowledgeSource::EverOS => 0.9,
            KnowledgeSource::MattPocockSkills => 0.85,
            KnowledgeSource::NestedLearning => 0.95,
            KnowledgeSource::AutonomousGoal => 0.92,
            KnowledgeSource::AwesomeDesignSkills => 0.85,
            KnowledgeSource::DeepSeekTui => 0.92,
            KnowledgeSource::Codebuff => 0.85,
            KnowledgeSource::OpenClaude => 0.88,
            KnowledgeSource::Cairn => 0.85,
            KnowledgeSource::Orca => 0.8,
            KnowledgeSource::RedRun => 0.82,
            KnowledgeSource::AutonomousSpeedrunning => 0.78,
            KnowledgeSource::Synesis => 0.9,
            KnowledgeSource::MemOS => 0.92,
            KnowledgeSource::Reflexio => 0.88,
            KnowledgeSource::Mem0 => 0.93,
            KnowledgeSource::Mnemosyne => 0.85,
            KnowledgeSource::OriMnemos => 0.82,
            KnowledgeSource::OPSD => 0.86,
            KnowledgeSource::AttentionMechanism => 0.75,
            KnowledgeSource::PatchFile => 0.6,
            KnowledgeSource::KeyVault => 0.7,
            KnowledgeSource::SealLoop => 0.9,
            KnowledgeSource::HashCortxAgents => 0.88,
            KnowledgeSource::HashCortxSecurity => 0.92,
            KnowledgeSource::HashCortxSwarm => 0.85,
            KnowledgeSource::HashCortxFailover => 0.82,
            KnowledgeSource::HetuLuoshu => 0.88,
            KnowledgeSource::YijingBinary => 0.9,
            KnowledgeSource::FivePhasesGauge => 0.82,
            KnowledgeSource::ThreeCosmologies => 0.85,
            KnowledgeSource::HuainanziCalendar => 0.83,
            KnowledgeSource::ZhangHengSeismoscope => 0.8,
            KnowledgeSource::MawangduiAstronomy => 0.82,
            KnowledgeSource::ShaoYongCosmology => 0.86,
            KnowledgeSource::DayanNumber => 0.84,
            KnowledgeSource::SecurityAttacks => 0.95,
            KnowledgeSource::LiteParse => 0.88,
            KnowledgeSource::SmartSearch => 0.85,
            KnowledgeSource::AQBot => 0.82,
            KnowledgeSource::AionUi => 0.86,
            KnowledgeSource::CyberVerse => 0.78,
            KnowledgeSource::Hotpush => 0.75,
            KnowledgeSource::InfiniteCanvas => 0.72,
            KnowledgeSource::AutoDocxProofread => 0.7,
            KnowledgeSource::OpenSwe => 0.8,
            KnowledgeSource::PiMonolith => 0.85,
            KnowledgeSource::ClawCode => 0.92,
            KnowledgeSource::HermesAgent => 0.88,
            KnowledgeSource::Bernstein => 0.82,
            KnowledgeSource::Mastra => 0.78,
            KnowledgeSource::Omi => 0.75,
            KnowledgeSource::Crush => 0.7,
            KnowledgeSource::QwenCode => 0.85,
            KnowledgeSource::LlmWiki => 0.88,
            KnowledgeSource::DarwinSkill => 0.93,
            // vectors_group_b variants
            KnowledgeSource::SkillOpt => 0.88,
            KnowledgeSource::MuseAutoskill => 0.85,
            KnowledgeSource::FeynmanAgent => 0.9,
            KnowledgeSource::AwesomeArchitecture => 0.87,
            KnowledgeSource::VulnGym => 0.82,
            KnowledgeSource::ZepMemory => 0.85,
            KnowledgeSource::HindsightMemory => 0.88,
            KnowledgeSource::CogneeMemory => 0.82,
            KnowledgeSource::SageMemory => 0.84,
            KnowledgeSource::ApexMem => 0.83,
            KnowledgeSource::LangMem => 0.86,
            KnowledgeSource::LettaMemory => 0.87,
            KnowledgeSource::Maigret => 0.82,
            KnowledgeSource::TasteSkill => 0.88,
            KnowledgeSource::UnderstandAnything => 0.85,
            KnowledgeSource::CarbonCode => 0.8,
            KnowledgeSource::LlmArch => 0.85,
            KnowledgeSource::Spear => 0.95,
            KnowledgeSource::Sia => 0.92,
            KnowledgeSource::SkillsGate => 0.82,
            KnowledgeSource::AdamsLaw => 0.85,
        }
    }

    /// Returns sources sorted by access frequency — hot sources first
    pub fn sort_by_access<'a>(sources: &'a [KnowledgeSource], tracker: &super::SourceAccessTracker) -> Vec<&'a KnowledgeSource> {
        let mut sorted: Vec<&'a KnowledgeSource> = sources.iter().collect();
        sorted.sort_by(|a, b| {
            tracker.access_count(b).cmp(&tracker.access_count(a))
        });
        sorted
    }
}

impl super::KnowledgeProvider for KnowledgeSource {
    fn name(&self) -> &str {
        KnowledgeSource::name(self)
    }

    fn capability_vector(&self) -> CapabilityVector {
        KnowledgeSource::capability_vector(self)
    }

    fn source_weight(&self) -> f64 {
        KnowledgeSource::source_weight(self)
    }
}

#[cfg(test)]
mod provider_tests {
    use crate::core::knowledge::*;

    #[test]
    fn test_knowledge_provider_heroui() {
        let source = KnowledgeSource::HeroUI;
        assert!(!KnowledgeProvider::name(&source).is_empty());
        let cv = KnowledgeProvider::capability_vector(&source);
        assert!(cv.arr.iter().any(|&v| v > 0.0));
        assert!(KnowledgeProvider::source_weight(&source) > 0.0);
    }

    #[test]
    fn test_knowledge_provider_memos() {
        let source = KnowledgeSource::MemOS;
        assert!(KnowledgeProvider::name(&source).contains("MemOS"));
        let cv = KnowledgeProvider::capability_vector(&source);
        assert!(!cv.extension.is_empty());
        assert!(KnowledgeProvider::source_weight(&source) > 0.0);
    }
}
