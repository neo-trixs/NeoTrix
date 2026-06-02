//! Agent 预设模板 — HashCortX Agent-as-Config 模式
//!
//! 9 个 Concrete Agent 预设, system prompt + tool list = agent 定义
//! 对标 HashCortX BUILTIN_AGENTS + tool resolution

use serde::{Deserialize, Serialize};

/// 模型能力 tier (参考 HashCortX getModelTier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModelTier {
    /// 300B+ frontier models
    Frontier = 5,
    /// 70B-300B strong models
    Strong = 4,
    /// 30B-70B capable models
    Capable = 3,
    /// 8B-30B moderate models
    Moderate = 2,
    /// 1.5B-8B small models
    Small = 1,
}

impl ModelTier {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Frontier => "frontier",
            Self::Strong => "strong",
            Self::Capable => "capable",
            Self::Moderate => "moderate",
            Self::Small => "small",
        }
    }
}

/// Agent 预设模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPreset {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub system_prompt: String,
    pub tools: Vec<String>,
    pub min_tier: ModelTier,
}

impl AgentPreset {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        icon: impl Into<String>,
        description: impl Into<String>,
        system_prompt: impl Into<String>,
        tools: Vec<String>,
        min_tier: ModelTier,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            description: description.into(),
            system_prompt: system_prompt.into(),
            tools,
            min_tier,
        }
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

/// 所有内置 agent 预设
/// 9 个模板, 覆盖 HashCortX BUILTIN_AGENTS 全部类型
fn tl(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

pub fn builtin_presets() -> Vec<AgentPreset> {
    vec![
        AgentPreset::new(
            "personal_assistant", "Personal Assistant", "🤖",
            "Full-featured AI assistant with memory, web search, and code execution",
            concat!("You are an intelligent personal assistant. You can:\n",
                "Use memory to recall past conversations. Search the web for real-time information. ",
                "Fetch and analyze URLs. Calculate and process data. Write and execute code. ",
                "Always start by understanding the user's intent."),
            tl(&["memory", "nt_world_search", "fetch_url", "datetime", "calculate", "code_interpreter"]),
            ModelTier::Moderate,
        ),
        AgentPreset::new(
            "lite", "Lite Assistant", "⚡",
            "Minimal assistant optimized for 1.5B-8B models",
            "You are a lightweight assistant. Keep responses brief. Focus on direct answers.",
            tl(&["memory", "datetime", "calculate"]),
            ModelTier::Small,
        ),
        AgentPreset::new(
            "researcher", "Deep Researcher", "🔬",
            "Multi-step iterative research with source synthesis and verification",
            concat!("You are a research assistant. Decompose questions into sub-questions. ",
                "Search multiple sources. Cross-reference findings. Synthesize into a coherent brief. ",
                "Prioritize authoritative sources. Flag contradictions. Provide citations."),
            tl(&["memory", "nt_world_search", "wikipedia", "fetch_url", "datetime", "code_interpreter"]),
            ModelTier::Capable,
        ),
        AgentPreset::new(
            "deep_research", "Deep Research Brief", "🧠",
            "Comprehensive source-backed research briefs",
            concat!("You produce structured research briefs: Executive Summary, ",
                "Key Findings with sources, Evidence Weights, Gaps & Uncertainties. ",
                "Every claim must be traceable. Use PubMed for scientific topics."),
            tl(&["memory", "nt_world_search", "wikipedia", "fetch_url", "pubmed", "code_interpreter"]),
            ModelTier::Strong,
        ),
        AgentPreset::new(
            "hash_coder", "HashCoder", "💻",
            "Professional coding agent with file system and shell access",
            concat!("You are a senior software engineer. Write clean idiomatic code. ",
                "Read/write/patch files. Execute shell commands. Run code interpreter. ",
                "Always check existing code before modifying. Run tests after changes. ",
                "Handle errors gracefully. Follow project conventions."),
            tl(&["memory", "nt_world_search", "fetch_url", "file_read", "file_write",
                 "file_patch", "shell_exec", "code_interpreter"]),
            ModelTier::Strong,
        ),
        AgentPreset::new(
            "url_reader", "URL Reader", "🌐",
            "Analyze, summarize, and extract data from web content",
            concat!("You analyze web content. Determine content type (article/docs/code). ",
                "Extract key information in structured format. Provide concise summaries. ",
                "For documentation, focus on API signatures and usage examples."),
            tl(&["memory", "fetch_url", "nt_world_search", "code_interpreter"]),
            ModelTier::Moderate,
        ),
        AgentPreset::new(
            "papers", "Papers Agent", "📄",
            "Scientific literature search and structured analysis",
            concat!("Search and analyze scientific literature. Extract title, authors, ",
                "year, method, results, limitations. Compare across papers. ",
                "Identify research gaps. Output with BibTeX-ready citations."),
            tl(&["memory", "pubmed", "fetch_url", "nt_world_search", "code_interpreter"]),
            ModelTier::Capable,
        ),
        AgentPreset::new(
            "medical_lexi", "Medical Lexi", "💊",
            "Drug interaction analysis and medical reference",
            concat!("Analyze pharmaceutical information. Identify drugs and mechanisms. ",
                "Search known interactions. Classify severity. Suggest monitoring. ",
                "Include disclaimer. Cite sources for all claims."),
            tl(&["memory", "nt_world_search", "fetch_url", "code_interpreter"]),
            ModelTier::Capable,
        ),
        AgentPreset::new(
            "ats_auditor", "ATS Auditor", "📋",
            "Resume ATS scoring and optimization",
            concat!("Audit resumes for ATS compatibility. Score keyword density, ",
                "format compatibility, quantifiable achievements, action verbs. ",
                "Provide specific before/after improvement suggestions."),
            tl(&["memory", "nt_world_search", "code_interpreter"]),
            ModelTier::Moderate,
        ),
    ]
}

/// 根据 ID 查找预设
pub fn find_preset(id: &str) -> Option<AgentPreset> {
    builtin_presets().into_iter().find(|p| p.id == id)
}

/// 根据模型 tier 过滤可用预设
pub fn presets_for_tier(tier: ModelTier) -> Vec<AgentPreset> {
    builtin_presets()
        .into_iter()
        .filter(|p| p.min_tier <= tier)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_presets_count() {
        let presets = builtin_presets();
        assert_eq!(presets.len(), 9);
    }

    #[test]
    fn test_each_preset_has_id() {
        for p in builtin_presets() {
            assert!(!p.id.is_empty());
            assert!(!p.name.is_empty());
            assert!(!p.system_prompt.is_empty());
        }
    }

    #[test]
    fn test_find_preset() {
        let p = find_preset("hash_coder").expect("value should be ok in test");
        assert_eq!(p.name, "HashCoder");
        assert!(p.tools.contains(&"file_read".to_string()));
        assert!(p.tools.contains(&"file_patch".to_string()));
    }

    #[test]
    fn test_find_nonexistent() {
        assert!(find_preset("no_such_agent").is_none());
    }

    #[test]
    fn test_presets_for_tier_small() {
        let available = presets_for_tier(ModelTier::Small);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "lite");
    }

    #[test]
    fn test_presets_for_tier_strong() {
        let available = presets_for_tier(ModelTier::Strong);
        assert!(available.len() > 5);
        assert!(available.iter().any(|p| p.id == "hash_coder"));
    }

    #[test]
    fn test_each_preset_has_icon() {
        for p in builtin_presets() {
            assert!(!p.icon.is_empty(), "Preset {} missing icon", p.id);
        }
    }

    #[test]
    fn test_each_preset_has_description() {
        for p in builtin_presets() {
            assert!(!p.description.is_empty(), "Preset {} missing description", p.id);
        }
    }

    #[test]
    fn test_tool_count_varied() {
        let presets = builtin_presets();
        let counts: Vec<usize> = presets.iter().map(|p| p.tool_count()).collect();
        assert!(counts.iter().any(|c| *c == 3));   // lite
        assert!(counts.iter().any(|c| *c == 8));   // hash_coder
        assert!(counts.iter().any(|c| *c == 6));   // assistant
    }

    #[test]
    fn test_model_tier_ordering() {
        assert!(ModelTier::Frontier > ModelTier::Strong);
        assert!(ModelTier::Strong > ModelTier::Capable);
        assert!(ModelTier::Capable > ModelTier::Moderate);
        assert!(ModelTier::Moderate > ModelTier::Small);
    }

    #[test]
    fn test_model_tier_label() {
        assert_eq!(ModelTier::Frontier.label(), "frontier");
        assert_eq!(ModelTier::Small.label(), "small");
    }

    #[test]
    fn test_duplicate_ids() {
        let mut ids: Vec<String> = builtin_presets().into_iter().map(|p| p.id).collect();
        let len_before = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), len_before, "Duplicate preset IDs found");
    }
}
