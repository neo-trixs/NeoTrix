use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentPatternCategory {
    StarterAgent,
    AdvancedAgent,
    MultiAgentTeam,
    VoiceAgent,
    RAGPipeline,
    AgentSkill,
    MCPAgent,
    AlwaysOnAgent,
    GenerativeUI,
    GameAgent,
    ResearchAgent,
    CodeAgent,
}

impl AgentPatternCategory {
    pub fn label(&self) -> &'static str {
        match self {
            AgentPatternCategory::StarterAgent => "Starter Agent",
            AgentPatternCategory::AdvancedAgent => "Advanced Agent",
            AgentPatternCategory::MultiAgentTeam => "Multi-Agent Team",
            AgentPatternCategory::VoiceAgent => "Voice Agent",
            AgentPatternCategory::RAGPipeline => "RAG Pipeline",
            AgentPatternCategory::AgentSkill => "Agent Skill",
            AgentPatternCategory::MCPAgent => "MCP Agent",
            AgentPatternCategory::AlwaysOnAgent => "Always-On Agent",
            AgentPatternCategory::GenerativeUI => "Generative UI",
            AgentPatternCategory::GameAgent => "Game Agent",
            AgentPatternCategory::ResearchAgent => "Research Agent",
            AgentPatternCategory::CodeAgent => "Code Agent",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPatternMetadata {
    pub name: String,
    pub description: String,
    pub category: AgentPatternCategory,
    pub complexity: f64,
    pub requires_vision: bool,
    pub requires_voice: bool,
    pub tags: Vec<String>,
}

pub struct AgentPatternRegistry {
    patterns: HashMap<String, AgentPatternMetadata>,
}

impl AgentPatternRegistry {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        for meta in BUILTIN_PATTERNS.iter() {
            patterns.insert(meta.name.clone(), meta.clone());
        }
        Self { patterns }
    }

    pub fn get(&self, name: &str) -> Option<&AgentPatternMetadata> {
        self.patterns.get(name)
    }

    pub fn list_all(&self) -> Vec<&AgentPatternMetadata> {
        self.patterns.values().collect()
    }

    pub fn register(&mut self, meta: AgentPatternMetadata) {
        self.patterns.insert(meta.name.clone(), meta);
    }

    pub fn find_by_category(&self, category: AgentPatternCategory) -> Vec<&AgentPatternMetadata> {
        self.patterns.values().filter(|p| p.category == category).collect()
    }

    pub fn categories(&self) -> Vec<AgentPatternCategory> {
        let mut cats: Vec<AgentPatternCategory> = self
            .patterns
            .values()
            .map(|p| p.category)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        cats.sort_by_key(|c| *c as u8);
        cats
    }
}

pub fn query_pattern(name: &str) -> Option<AgentPatternMetadata> {
    BUILTIN_PATTERNS.iter().find(|m| m.name == name).cloned()
}

pub fn list_patterns() -> Vec<AgentPatternMetadata> {
    BUILTIN_PATTERNS.clone()
}

impl Default for AgentPatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

static BUILTIN_PATTERNS: LazyLock<Vec<AgentPatternMetadata>> = LazyLock::new(|| vec![
    AgentPatternMetadata {
        name: "ai-travel-agent".to_string(),
        description: "Personalized day-by-day travel itineraries with API integrations".to_string(),
        category: AgentPatternCategory::StarterAgent,
        complexity: 0.3,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["travel".to_string(), "itinerary".to_string(), "api".to_string()],
    },
    AgentPatternMetadata {
        name: "ai-blog-to-podcast".to_string(),
        description: "Turn any blog URL into a narrated podcast episode".to_string(),
        category: AgentPatternCategory::StarterAgent,
        complexity: 0.25,
        requires_vision: false,
        requires_voice: true,
        tags: vec!["tts".to_string(), "blog".to_string(), "podcast".to_string()],
    },
    AgentPatternMetadata {
        name: "ai-data-analysis-agent".to_string(),
        description: "Ask questions of any CSV or Excel file in plain English".to_string(),
        category: AgentPatternCategory::StarterAgent,
        complexity: 0.35,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["data".to_string(), "csv".to_string(), "analysis".to_string()],
    },
    AgentPatternMetadata {
        name: "ai-medical-imaging-agent".to_string(),
        description: "Diagnostic analysis of X-rays and scans with Gemini multimodal".to_string(),
        category: AgentPatternCategory::AdvancedAgent,
        complexity: 0.7,
        requires_vision: true,
        requires_voice: false,
        tags: vec!["medical".to_string(), "vision".to_string(), "diagnostics".to_string()],
    },
    AgentPatternMetadata {
        name: "ai-meme-generator-browser".to_string(),
        description: "Makes memes by driving a real browser, not an image API".to_string(),
        category: AgentPatternCategory::AdvancedAgent,
        complexity: 0.55,
        requires_vision: true,
        requires_voice: false,
        tags: vec!["browser".to_string(), "meme".to_string(), "automation".to_string()],
    },
    AgentPatternMetadata {
        name: "ai-music-generator".to_string(),
        description: "Prompt in, MP3 track out with full music generation pipeline".to_string(),
        category: AgentPatternCategory::AdvancedAgent,
        complexity: 0.5,
        requires_vision: false,
        requires_voice: true,
        tags: vec!["music".to_string(), "audio".to_string(), "generation".to_string()],
    },
    AgentPatternMetadata {
        name: "ai-home-renovation-agent".to_string(),
        description: "Plan your home renovation with AI-driven design and budgeting".to_string(),
        category: AgentPatternCategory::AdvancedAgent,
        complexity: 0.6,
        requires_vision: true,
        requires_voice: false,
        tags: vec!["home".to_string(), "design".to_string(), "budgeting".to_string()],
    },
    AgentPatternMetadata {
        name: "multi-agent-research-team".to_string(),
        description: "Deploy a specialized research team with supervisor, writer, reviewer".to_string(),
        category: AgentPatternCategory::MultiAgentTeam,
        complexity: 0.85,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["research".to_string(), "multi-agent".to_string(), "team".to_string()],
    },
    AgentPatternMetadata {
        name: "voice-controlled-smart-home".to_string(),
        description: "Manage IoT devices through natural voice commands with real-time control".to_string(),
        category: AgentPatternCategory::VoiceAgent,
        complexity: 0.65,
        requires_vision: false,
        requires_voice: true,
        tags: vec!["voice".to_string(), "iot".to_string(), "smart-home".to_string()],
    },
    AgentPatternMetadata {
        name: "rag-knowledge-assistant".to_string(),
        description: "Query large document corpora with RAG-based retrieval and synthesis".to_string(),
        category: AgentPatternCategory::RAGPipeline,
        complexity: 0.5,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["rag".to_string(), "knowledge".to_string(), "search".to_string()],
    },
    AgentPatternMetadata {
        name: "customer-support-agent".to_string(),
        description: "Automated customer support with ticket classification and resolution".to_string(),
        category: AgentPatternCategory::AgentSkill,
        complexity: 0.4,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["support".to_string(), "ticket".to_string(), "automation".to_string()],
    },
    AgentPatternMetadata {
        name: "mcp-tool-server".to_string(),
        description: "Expose any API as an MCP tool for LLM consumption".to_string(),
        category: AgentPatternCategory::MCPAgent,
        complexity: 0.3,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["mcp".to_string(), "tool".to_string(), "server".to_string()],
    },
    AgentPatternMetadata {
        name: "always-on-monitor".to_string(),
        description: "Background monitor that watches for events and triggers alerts".to_string(),
        category: AgentPatternCategory::AlwaysOnAgent,
        complexity: 0.45,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["monitor".to_string(), "alert".to_string(), "background".to_string()],
    },
    AgentPatternMetadata {
        name: "generative-dashboard".to_string(),
        description: "AI-generated real-time dashboards from natural language queries".to_string(),
        category: AgentPatternCategory::GenerativeUI,
        complexity: 0.7,
        requires_vision: true,
        requires_voice: false,
        tags: vec!["dashboard".to_string(), "ui".to_string(), "generative".to_string()],
    },
    AgentPatternMetadata {
        name: "game-npc-agent".to_string(),
        description: "Intelligent NPC dialogue and behavior generation for games".to_string(),
        category: AgentPatternCategory::GameAgent,
        complexity: 0.75,
        requires_vision: false,
        requires_voice: true,
        tags: vec!["game".to_string(), "npc".to_string(), "dialogue".to_string()],
    },
    AgentPatternMetadata {
        name: "code-review-agent".to_string(),
        description: "Automated code review with style checks, security audit, and best practice recommendations".to_string(),
        category: AgentPatternCategory::CodeAgent,
        complexity: 0.5,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["code".to_string(), "review".to_string(), "security".to_string()],
    },
    AgentPatternMetadata {
        name: "deep-research-agent".to_string(),
        description: "Multi-source research with citation tracking, synthesis, and report generation".to_string(),
        category: AgentPatternCategory::ResearchAgent,
        complexity: 0.8,
        requires_vision: false,
        requires_voice: false,
        tags: vec!["research".to_string(), "synthesis".to_string(), "report".to_string()],
    },
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_contains_patterns() {
        let reg = AgentPatternRegistry::new();
        assert!(reg.list_all().len() >= 15);
    }

    #[test]
    fn test_query_known_pattern() {
        let result = query_pattern("ai-travel-agent");
        assert!(result.is_some());
        assert_eq!(result.unwrap().category, AgentPatternCategory::StarterAgent);
    }

    #[test]
    fn test_query_unknown_pattern() {
        let result = query_pattern("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_patterns() {
        let patterns = list_patterns();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_categories() {
        let cats = AgentPatternRegistry::new().categories();
        assert!(cats.len() >= 5);
    }
}
