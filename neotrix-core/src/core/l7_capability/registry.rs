use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::core::nt_core_cap::CapabilityVector;

pub type CapabilityId = Uuid;

/// OpenMontage-style tier classification for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityTier {
    Core,
    Voice,
    Enhance,
    Generate,
    Source,
    Analyze,
    Publish,
}

impl CapabilityTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::Voice => "Voice",
            Self::Enhance => "Enhance",
            Self::Generate => "Generate",
            Self::Source => "Source",
            Self::Analyze => "Analyze",
            Self::Publish => "Publish",
        }
    }
}

/// Runtime environment for a capability (OpenMontage-inspired)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityRuntime {
    Local,
    LocalGpu,
    Api,
    Hybrid,
    Mcp,
}

impl CapabilityRuntime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::LocalGpu => "local_gpu",
            Self::Api => "api",
            Self::Hybrid => "hybrid",
            Self::Mcp => "mcp",
        }
    }
}

/// Stability level for a capability (OpenMontage-inspired)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityStability {
    Experimental,
    Beta,
    Production,
    Deprecated,
}

impl CapabilityStability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::Beta => "beta",
            Self::Production => "production",
            Self::Deprecated => "deprecated",
        }
    }

    pub fn confidence(&self) -> f64 {
        match self {
            Self::Experimental => 0.3,
            Self::Beta => 0.6,
            Self::Production => 0.95,
            Self::Deprecated => 0.1,
        }
    }
}

/// Fallback chain entry — if this capability fails, try the next
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackEntry {
    pub fallback_name: String,
    pub condition: FallbackCondition,
    pub degrade_gracefully: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FallbackCondition {
    OnError,
    OnTimeout,
    OnLowQuality { threshold: f64 },
    OnRateLimit,
    Always,
}

impl PartialEq for FallbackCondition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::OnError, Self::OnError) => true,
            (Self::OnTimeout, Self::OnTimeout) => true,
            (Self::OnLowQuality { threshold: a }, Self::OnLowQuality { threshold: b }) => {
                a.to_bits() == b.to_bits()
            }
            (Self::OnRateLimit, Self::OnRateLimit) => true,
            (Self::Always, Self::Always) => true,
            _ => false,
        }
    }
}

impl std::hash::Hash for FallbackCondition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        if let Self::OnLowQuality { threshold } = self {
            threshold.to_bits().hash(state);
        }
    }
}

impl FallbackCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OnError => "on_error",
            Self::OnTimeout => "on_timeout",
            Self::OnLowQuality { .. } => "on_low_quality",
            Self::OnRateLimit => "on_rate_limit",
            Self::Always => "always",
        }
    }
}

/// SkillForge-style domain category for capability matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCategory {
    Debugging,
    Testing,
    Security,
    Spreadsheet,
    Document,
    Presentation,
    Api,
    Database,
    CodeGeneration,
    Reasoning,
    KnowledgeQA,
    ToolUse,
    Safety,
    InstructionFollowing,
    MultiTurn,
    ImageGeneration,
    VideoGeneration,
    TextToSpeech,
    MusicGeneration,
    DataAnalysis,
    WebSearch,
    Communication,
    Planning,
    Creative,
    Design,
    General,
}

impl DomainCategory {
    pub fn all() -> &'static [DomainCategory; 26] {
        use DomainCategory::*;
        &[
            Debugging, Testing, Security, Spreadsheet, Document, Presentation,
            Api, Database, CodeGeneration, Reasoning, KnowledgeQA, ToolUse,
            Safety, InstructionFollowing, MultiTurn, ImageGeneration,
            VideoGeneration, TextToSpeech, MusicGeneration, DataAnalysis,
            WebSearch, Communication, Planning, Creative, Design, General,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        use DomainCategory::*;
        match self {
            Debugging => "debugging",
            Testing => "testing",
            Security => "security",
            Spreadsheet => "spreadsheet",
            Document => "document",
            Presentation => "presentation",
            Api => "api",
            Database => "database",
            CodeGeneration => "code_generation",
            Reasoning => "reasoning",
            KnowledgeQA => "knowledge_qa",
            ToolUse => "tool_use",
            Safety => "safety",
            InstructionFollowing => "instruction_following",
            MultiTurn => "multi_turn",
            ImageGeneration => "image_generation",
            VideoGeneration => "video_generation",
            TextToSpeech => "text_to_speech",
            MusicGeneration => "music_generation",
            DataAnalysis => "data_analysis",
            WebSearch => "web_search",
            Communication => "communication",
            Planning => "planning",
            Creative => "creative",
            Design => "design",
            General => "general",
        }
    }

    pub fn synonyms(&self) -> &[&str] {
        use DomainCategory::*;
        match self {
            Debugging => &["debug", "bug", "fix", "error", "issue"],
            Testing => &["test", "spec", "assertion", "coverage"],
            Security => &["secure", "vulnerability", "cve", "auth", "owasp"],
            Spreadsheet => &["excel", "csv", "sheet", "tabular"],
            Document => &["doc", "word", "markdown", "pdf"],
            Presentation => &["slides", "ppt", "deck", "presentation"],
            Api => &["rest", "endpoint", "http", "graphql", "mcp"],
            Database => &["sql", "query", "db", "nosql", "redis"],
            CodeGeneration => &["code", "programming", "script", "function"],
            Reasoning => &["reason", "logic", "inference", "think"],
            KnowledgeQA => &["knowledge", "qa", "fact", "question"],
            ToolUse => &["tool", "mcp", "function", "plugin"],
            Safety => &["safe", "harm", "align", "ethics", "jailbreak"],
            InstructionFollowing => &["instruction", "follow", "compliance"],
            MultiTurn => &["chat", "dialog", "multi_turn", "conversation"],
            ImageGeneration => &["image", "picture", "photo", "illustration", "art"],
            VideoGeneration => &["video", "movie", "clip", "animation"],
            TextToSpeech => &["tts", "speech", "voice", "audio"],
            MusicGeneration => &["music", "song", "melody", "sound"],
            DataAnalysis => &["analysis", "analytics", "stats", "visualization"],
            WebSearch => &["search", "web", "browse", "fetch", "crawl"],
            Communication => &["email", "slack", "message", "notify", "social"],
            Planning => &["plan", "strategy", "roadmap", "schedule"],
            Creative => &["creative", "write", "story", "content"],
            Design => &["design", "ui", "ux", "layout", "style"],
            General => &["general", "default", "fallback"],
        }
    }
}

pub fn capability_id_from_name(name: &str) -> CapabilityId {
    let hash = {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(name.as_bytes());
        hasher.finish()
    };
    let bytes = hash.to_le_bytes();
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes[..8].copy_from_slice(&bytes);
    uuid_bytes[8..].copy_from_slice(&bytes);
    Uuid::from_bytes(uuid_bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityKind {
    Perceptual,
    Cognitive,
    Mnemonic,
    Physical,
    Social,
    Metacognitive,
    Shield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaturityLevel {
    Primitive,
    Candidate,
    Reviewed,
    Validated,
    GroundTruth,
    Transcendent,
}

impl MaturityLevel {
    pub fn promote(&self) -> Option<Self> {
        match self {
            Self::Primitive => Some(Self::Candidate),
            Self::Candidate => Some(Self::Reviewed),
            Self::Reviewed => Some(Self::Validated),
            Self::Validated => Some(Self::GroundTruth),
            Self::GroundTruth => Some(Self::Transcendent),
            Self::Transcendent => None,
        }
    }

    pub fn confidence(&self) -> f64 {
        match self {
            Self::Primitive => 0.1,
            Self::Candidate => 0.25,
            Self::Reviewed => 0.5,
            Self::Validated => 0.75,
            Self::GroundTruth => 0.9,
            Self::Transcendent => 0.99,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Primitive => "Primitive",
            Self::Candidate => "Candidate",
            Self::Reviewed => "Reviewed",
            Self::Validated => "Validated",
            Self::GroundTruth => "GroundTruth",
            Self::Transcendent => "Transcendent",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSlot {
    pub name: String,
    pub kind: SlotKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotKind {
    Task,
    Context,
    Goal,
    Attention,
    Memory,
    Perception,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCost {
    pub estimated_tokens: u64,
    pub estimated_ms: u64,
    pub estimated_memory_kb: u64,
}

impl Default for CapabilityCost {
    fn default() -> Self {
        Self { estimated_tokens: 1000, estimated_ms: 500, estimated_memory_kb: 64 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStats {
    pub call_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_latency_ms: f64,
    pub avg_prm_score: f64,
    pub diversity_score: f64,
    pub last_called: Option<i64>,
    pub total_tokens: u64,
    pub success_rate: f64,
}

impl Default for CapabilityStats {
    fn default() -> Self {
        Self {
            call_count: 0, success_count: 0, failure_count: 0,
            avg_latency_ms: 0.0, avg_prm_score: 0.0, diversity_score: 0.0,
            last_called: None, total_tokens: 0, success_rate: 0.0,
        }
    }
}

impl CapabilityStats {
    pub fn record_call(&mut self, success: bool, latency_ms: f64, prm_score: f64) {
        self.call_count += 1;
        if success { self.success_count += 1; } else { self.failure_count += 1; }
        self.avg_latency_ms = (self.avg_latency_ms * (self.call_count as f64 - 1.0) + latency_ms) / self.call_count as f64;
        self.avg_prm_score = (self.avg_prm_score * (self.call_count as f64 - 1.0) + prm_score) / self.call_count as f64;
        self.success_rate = self.success_count as f64 / self.call_count as f64;
        self.last_called = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub name: String,
    pub tags: Vec<String>,
    pub kind: CapabilityKind,
    pub maturity: MaturityLevel,
    pub vector: CapabilityVector,
    pub e8_triggers: Vec<u8>,
    pub context_requirements: Vec<ContextSlot>,
    pub cost: CapabilityCost,
    pub stats: CapabilityStats,
    pub version: String,
    pub layer: u8,
    // OpenMontage-inspired fields
    pub tier: CapabilityTier,
    pub runtime: CapabilityRuntime,
    pub stability: CapabilityStability,
    pub fallback_chain: Vec<FallbackEntry>,
    pub provider: Option<String>,
    // SkillForge-inspired domain matching
    pub domain: DomainCategory,
    pub input_schema: Option<String>,
    pub output_schema: Option<String>,
    // Resource profile
    pub resource_cpu: f64,
    pub resource_ram_mb: f64,
    pub resource_vram_mb: f64,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub struct CapabilityRegistry {
    capabilities: Vec<Capability>,
    tag_index: std::collections::HashMap<String, Vec<CapabilityId>>,
}

impl Default for CapabilityRegistry { fn default() -> Self { Self::new() } }

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self { capabilities: Vec::new(), tag_index: std::collections::HashMap::new() }
    }

    pub fn register(&mut self, cap: Capability) {
        for tag in &cap.tags {
            self.tag_index.entry(tag.clone()).or_default().push(cap.id);
        }
        self.capabilities.push(cap);
    }

    pub fn get(&self, id: &CapabilityId) -> Option<&Capability> {
        self.capabilities.iter().find(|c| &c.id == id)
    }

    pub fn get_mut(&mut self, id: &CapabilityId) -> Option<&mut Capability> {
        self.capabilities.iter_mut().find(|c| &c.id == id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.name == name)
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&Capability> {
        self.tag_index.get(tag).map(|ids| ids.iter().filter_map(|id| self.get(id)).collect()).unwrap_or_default()
    }

    pub fn find_by_kind(&self, kind: CapabilityKind) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.kind == kind).collect()
    }

    pub fn find_by_layer(&self, layer: u8) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.layer == layer).collect()
    }

    pub fn search_by_e8_state(&self, hexagram: u8) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.e8_triggers.contains(&hexagram)).collect()
    }

    pub fn all(&self) -> &[Capability] { &self.capabilities }
    pub fn count(&self) -> usize { self.capabilities.len() }

    /// 桥接 nt_core_model_skills 模型能力注册表 → L7 能力网。
    /// 把模型能力 (vision/context_window/provider) 注册为 Cognitive 类 capability,
    /// 使能力网可感知模型选择能力 (模型能力查询桥接, 消除 model_skills 死代码)。
    pub fn register_model_skills(&mut self) -> usize {
        use crate::core::nt_core_model_skills::REGISTRY;
        let mut registered = 0;
        for cap in REGISTRY.list_models() {
            let id = capability_id_from_name(&format!("model_{}", cap.model_name));
            if self.get(&id).is_some() {
                continue;
            }
            let mut tags = vec!["model".to_string(), cap.provider.clone()];
            if cap.supports_vision {
                tags.push("vision".to_string());
            }
            self.register(Capability {
                id,
                name: cap.model_name.clone(),
                tags,
                kind: CapabilityKind::Cognitive,
                maturity: MaturityLevel::Reviewed,
                vector: CapabilityVector::default(),
                e8_triggers: vec![],
                context_requirements: vec![],
                cost: CapabilityCost::default(),
                stats: CapabilityStats::default(),
                version: "0.1.0".to_string(),
                layer: 7,
                tier: CapabilityTier::Core,
                runtime: CapabilityRuntime::Local,
                stability: CapabilityStability::Production,
                fallback_chain: vec![],
                provider: Some(cap.provider.clone()),
                domain: DomainCategory::General,
                input_schema: None,
                output_schema: None,
                resource_cpu: 1.0,
                resource_ram_mb: 64.0,
                resource_vram_mb: 0.0,
                dependencies: vec![],
            });
            registered += 1;
        }
        registered
    }

    pub fn calculate_similarity(&self, a: &CapabilityId, b: &CapabilityId) -> f64 {
        match (self.get(a), self.get(b)) {
            (Some(ca), Some(cb)) => ca.vector.similarity(&cb.vector),
            _ => 0.0,
        }
    }

    pub fn remove(&mut self, id: &CapabilityId) -> Option<Capability> {
        if let Some(pos) = self.capabilities.iter().position(|c| &c.id == id) {
            let cap = self.capabilities.remove(pos);
            for tag in &cap.tags {
                if let Some(ids) = self.tag_index.get_mut(tag) { ids.retain(|i| i != id); }
            }
            Some(cap)
        } else { None }
    }

    // ── Domain-based search (SkillForge-inspired) ──

    /// Find capabilities by domain category
    pub fn find_by_domain(&self, domain: DomainCategory) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.domain == domain).collect()
    }

    /// Multi-factor domain matching: match by domain + kind + tags with confidence
    pub fn match_by_domain(
        &self, domain: DomainCategory, tags: &[String], kind: Option<CapabilityKind>
    ) -> Vec<(f64, &Capability)> {
        let mut scored: Vec<(f64, &Capability)> = self.capabilities.iter()
            .map(|c| {
                let mut score = 0.0;
                // Exact domain match: +0.5
                if c.domain == domain { score += 0.5; }
                // Domain synonym overlap: +0.2
                let domain_syns = domain.synonyms();
                let match_count = c.tags.iter()
                    .filter(|t| domain_syns.contains(&t.as_str())).count();
                score += match_count as f64 * 0.1;
                // Tag overlap: +0.15 per matching tag
                let tag_overlap = tags.iter().filter(|t| c.tags.contains(t)).count();
                score += tag_overlap as f64 * 0.15;
                // Kind match: +0.2
                if let Some(k) = kind { if c.kind == k { score += 0.2; } }
                // Maturity bonus: +0.05 per level
                score += (c.maturity as u8) as f64 * 0.05;
                // Stability penalty for experimental
                if c.stability == CapabilityStability::Experimental { score *= 0.8; }
                if c.stability == CapabilityStability::Deprecated { score *= 0.5; }
                (score.min(1.0), c)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    // ── Fallback chain resolution ──

    /// Resolve fallback chain: find the next viable fallback for a capability
    pub fn resolve_fallback(&self, cap_id: &CapabilityId) -> Option<&Capability> {
        let cap = self.get(cap_id)?;
        for entry in &cap.fallback_chain {
            if let Some(fallback) = self.find_by_name(&entry.fallback_name) {
                if fallback.stability == CapabilityStability::Deprecated { continue; }
                return Some(fallback);
            }
        }
        None
    }

    /// Get all capabilities with a specific runtime
    pub fn find_by_runtime(&self, runtime: CapabilityRuntime) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.runtime == runtime).collect()
    }

    /// Get all capabilities by tier
    pub fn find_by_tier(&self, tier: CapabilityTier) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.tier == tier).collect()
    }

    /// Get all capabilities with stability >= given level
    pub fn find_stable(&self, min_stability: CapabilityStability) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| {
            let order = |s: CapabilityStability| -> u8 {
                match s {
                    CapabilityStability::Deprecated => 0,
                    CapabilityStability::Experimental => 1,
                    CapabilityStability::Beta => 2,
                    CapabilityStability::Production => 3,
                }
            };
            order(c.stability) >= order(min_stability)
        }).collect()
    }

    // ── Selector-style routing (OpenMontage-inspired) ──

    /// Route by domain and kind — returns sorted (score, cap) pairs
    pub fn route_by_capability(&self, target: &str, domain: DomainCategory) -> Vec<(f64, &Capability)> {
        let tags = vec![target.to_string()];
        self.match_by_domain(domain, &tags, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cap(name: &str, kind: CapabilityKind, layer: u8) -> Capability {
        Capability {
            id: capability_id_from_name(name),
            name: name.to_string(),
            tags: vec!["test".to_string()],
            kind,
            maturity: MaturityLevel::Primitive,
            vector: CapabilityVector::default(),
            e8_triggers: vec![0x01],
            context_requirements: vec![],
            cost: CapabilityCost::default(),
            stats: CapabilityStats::default(),
            version: "0.1.0".to_string(),
            layer,
            tier: CapabilityTier::Core,
            runtime: CapabilityRuntime::Local,
            stability: CapabilityStability::Production,
            fallback_chain: vec![],
            provider: None,
            domain: DomainCategory::General,
            input_schema: None,
            output_schema: None,
            resource_cpu: 1.0,
            resource_ram_mb: 64.0,
            resource_vram_mb: 0.0,
            dependencies: vec![],
        }
    }

    #[test] fn test_register_and_get() {
        let mut reg = CapabilityRegistry::new();
        let cap = test_cap("test", CapabilityKind::Cognitive, 4);
        let id = cap.id; reg.register(cap);
        assert!(reg.get(&id).is_some()); assert_eq!(reg.count(), 1);
    }

    #[test] fn test_find_by_name() {
        let mut reg = CapabilityRegistry::new();
        reg.register(test_cap("finder", CapabilityKind::Physical, 1));
        assert!(reg.find_by_name("finder").is_some());
    }

    #[test] fn test_find_by_tag() {
        let mut reg = CapabilityRegistry::new();
        let mut cap = test_cap("t", CapabilityKind::Social, 2);
        cap.tags = vec!["alpha".into(), "beta".into()];
        reg.register(cap);
        assert_eq!(reg.find_by_tag("alpha").len(), 1);
        assert_eq!(reg.find_by_tag("gamma").len(), 0);
    }

    #[test] fn test_find_by_kind() {
        let mut reg = CapabilityRegistry::new();
        reg.register(test_cap("c1", CapabilityKind::Cognitive, 4));
        reg.register(test_cap("c2", CapabilityKind::Cognitive, 4));
        reg.register(test_cap("p1", CapabilityKind::Physical, 1));
        assert_eq!(reg.find_by_kind(CapabilityKind::Cognitive).len(), 2);
        assert_eq!(reg.find_by_kind(CapabilityKind::Physical).len(), 1);
    }

    #[test] fn test_search_by_e8_state() {
        let mut reg = CapabilityRegistry::new();
        let mut cap = test_cap("e8cap", CapabilityKind::Cognitive, 4);
        cap.e8_triggers = vec![0x42];
        reg.register(cap);
        assert_eq!(reg.search_by_e8_state(0x42).len(), 1);
    }

    #[test] fn test_remove() {
        let mut reg = CapabilityRegistry::new();
        reg.register(test_cap("r", CapabilityKind::Shield, 1));
        let id = capability_id_from_name("r");
        assert_eq!(reg.count(), 1);
        assert!(reg.remove(&id).is_some());
        assert_eq!(reg.count(), 0);
    }

    #[test] fn test_maturity_promotion() {
        assert_eq!(MaturityLevel::Primitive.promote(), Some(MaturityLevel::Candidate));
        assert_eq!(MaturityLevel::Transcendent.promote(), None);
    }

    #[test] fn test_domain_category_all() {
        let all = DomainCategory::all();
        assert_eq!(all.len(), 26);
    }

    #[test] fn test_domain_category_str() {
        assert_eq!(DomainCategory::Debugging.as_str(), "debugging");
        assert_eq!(DomainCategory::ImageGeneration.as_str(), "image_generation");
    }

    #[test] fn test_domain_synonyms() {
        let syns = DomainCategory::ImageGeneration.synonyms();
        assert!(syns.contains(&"image"));
        assert!(syns.contains(&"art"));
        let dbg = DomainCategory::Debugging.synonyms();
        assert!(dbg.contains(&"bug"));
    }

    #[test] fn test_capability_tier_str() {
        assert_eq!(CapabilityTier::Core.as_str(), "Core");
        assert_eq!(CapabilityTier::Generate.as_str(), "Generate");
    }

    #[test] fn test_capability_runtime_str() {
        assert_eq!(CapabilityRuntime::Mcp.as_str(), "mcp");
        assert_eq!(CapabilityRuntime::LocalGpu.as_str(), "local_gpu");
    }

    #[test] fn test_capability_stability_confidence() {
        assert!((CapabilityStability::Experimental.confidence() - 0.3).abs() < 0.01);
        assert!((CapabilityStability::Production.confidence() - 0.95).abs() < 0.01);
    }

    #[test] fn test_fallback_condition_str() {
        assert_eq!(FallbackCondition::OnError.as_str(), "on_error");
        assert_eq!(FallbackCondition::OnRateLimit.as_str(), "on_rate_limit");
    }

    #[test] fn test_find_by_domain() {
        let mut reg = CapabilityRegistry::new();
        let mut cap = test_cap("img_gen", CapabilityKind::Physical, 1);
        cap.domain = DomainCategory::ImageGeneration;
        cap.runtime = CapabilityRuntime::Api;
        cap.stability = CapabilityStability::Beta;
        cap.tier = CapabilityTier::Generate;
        let _id = cap.id;
        reg.register(cap);
        assert_eq!(reg.find_by_domain(DomainCategory::ImageGeneration).len(), 1);
        assert_eq!(reg.find_by_domain(DomainCategory::VideoGeneration).len(), 0);
        assert_eq!(reg.find_by_runtime(CapabilityRuntime::Api).len(), 1);
        assert_eq!(reg.find_by_tier(CapabilityTier::Generate).len(), 1);
        let stable = reg.find_stable(CapabilityStability::Beta);
        assert_eq!(stable.len(), 1);
        let prod = reg.find_stable(CapabilityStability::Production);
        assert_eq!(prod.len(), 0);
    }

    #[test] fn test_domain_matching_scoring() {
        let mut reg = CapabilityRegistry::new();
        let mut cap = test_cap("debug1", CapabilityKind::Cognitive, 4);
        cap.domain = DomainCategory::Debugging;
        cap.tags = vec!["bug".into(), "fix".into()];
        cap.maturity = MaturityLevel::Validated;
        cap.stability = CapabilityStability::Production;
        reg.register(cap);
        let results = reg.match_by_domain(DomainCategory::Debugging, &["fix".into()], Some(CapabilityKind::Cognitive));
        assert!(!results.is_empty());
        assert!(results[0].0 > 0.5);
    }

    #[test] fn test_fallback_chain_resolution() {
        let mut reg = CapabilityRegistry::new();
        let mut primary = test_cap("primary", CapabilityKind::Physical, 1);
        primary.domain = DomainCategory::ImageGeneration;
        primary.fallback_chain = vec![FallbackEntry {
            fallback_name: "backup".into(),
            condition: FallbackCondition::OnError,
            degrade_gracefully: true,
        }];
        let backup = test_cap("backup", CapabilityKind::Physical, 1);
        let pid = primary.id;
        let bid = backup.id;
        reg.register(primary);
        reg.register(backup);
        assert!(reg.resolve_fallback(&pid).is_some());
        assert!(reg.resolve_fallback(&bid).is_none());
    }

    #[test] fn test_capability_stats_record_call() {
        let mut s = CapabilityStats::default();
        s.record_call(true, 100.0, 0.85);
        assert_eq!(s.call_count, 1);
        assert!((s.success_rate - 1.0).abs() < 0.001);
    }

    #[test] fn test_id_deterministic() {
        assert_eq!(capability_id_from_name("a"), capability_id_from_name("a"));
        assert_ne!(capability_id_from_name("a"), capability_id_from_name("b"));
    }

    #[test]
    fn test_register_model_skills_bridges_capabilities() {
        // 桥接 nt_core_model_skills → L7 能力网: 模型能力注册为 Cognitive capability
        let mut reg = CapabilityRegistry::new();
        let registered = reg.register_model_skills();
        assert!(registered > 0, "应注册至少一个模型能力");
        assert_eq!(reg.count(), registered);
        // 幂等: 再次注册不重复
        let second = reg.register_model_skills();
        assert_eq!(second, 0, "重复注册应幂等");
        // 模型能力可被 tag 检索 (model / provider / vision)
        let model_caps = reg.find_by_tag("model");
        assert!(!model_caps.is_empty());
        // vision 模型带 vision tag
        let vision_caps = reg.find_by_tag("vision");
        assert!(vision_caps.iter().any(|c| c.tags.contains(&"vision".to_string())));
    }
}
