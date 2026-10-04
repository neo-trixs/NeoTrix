//! Domain mapper — map knowledge nodes to NeoTrix 8 domains + 5 source cores.
//!
//! Extracted from absorb_to_capability.py's keyword rules, KNOWN_REPOS,
//! SOURCE_CORES, and BRANCH_CAPABILITIES. This is the essential mapping
//! logic, not a verbatim copy.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

use super::source_adapter::KnowledgeInput;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    NtCore,
    NtMeta,
    NtMind,
    NtMemory,
    NtWorld,
    NtAct,
    NtShield,
    NtIo,
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::NtCore => "NT-CORE",
            Domain::NtMeta => "NT-META",
            Domain::NtMind => "NT-MIND",
            Domain::NtMemory => "NT-MEMORY",
            Domain::NtWorld => "NT-WORLD",
            Domain::NtAct => "NT-ACT",
            Domain::NtShield => "NT-SHIELD",
            Domain::NtIo => "NT-IO",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "NT-CORE" => Some(Domain::NtCore),
            "NT-META" => Some(Domain::NtMeta),
            "NT-MIND" => Some(Domain::NtMind),
            "NT-MEMORY" => Some(Domain::NtMemory),
            "NT-WORLD" => Some(Domain::NtWorld),
            "NT-ACT" => Some(Domain::NtAct),
            "NT-SHIELD" => Some(Domain::NtShield),
            "NT-IO" => Some(Domain::NtIo),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceCore {
    E8,
    Vsa,
    Gwt,
    ConsciousnessTree,
    MetaCognition,
    Reality,
}

impl SourceCore {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceCore::E8 => "E8",
            SourceCore::Vsa => "VSA",
            SourceCore::Gwt => "GWT",
            SourceCore::ConsciousnessTree => "ConsciousnessTree",
            SourceCore::MetaCognition => "MetaCognition",
            SourceCore::Reality => "Reality",
        }
    }

    pub fn primary_domain(&self) -> Domain {
        match self {
            SourceCore::E8 => Domain::NtCore,
            SourceCore::Vsa => Domain::NtMemory,
            SourceCore::Gwt => Domain::NtCore,
            SourceCore::ConsciousnessTree => Domain::NtMind,
            SourceCore::MetaCognition => Domain::NtMeta,
            SourceCore::Reality => Domain::NtWorld,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingResult {
    pub domain: Domain,
    pub capability: String,
    pub source_core: SourceCore,
    pub evidence: String,
    pub trace_keywords: Vec<String>,
}

struct KeywordRule {
    pattern: Regex,
    domain: Domain,
    capability: String,
    title_weight: u32,
}

struct SourceCoreDef {
    name: SourceCore,
    keywords: Vec<&'static str>,
}

static KEYWORD_RULES: LazyLock<Vec<KeywordRule>> = LazyLock::new(|| {
    vec![
        kw_rule(
            r"(?i)crawl|scrap|fetcher|spider|browser|harvest|extract_web",
            Domain::NtWorld,
            "retrieve",
            3,
        ),
        kw_rule(
            r"(?i)search|retriev|index|semantic_search|rag|vector",
            Domain::NtWorld,
            "search",
            3,
        ),
        kw_rule(
            r"(?i)osint|recon|subdomain|whois|dns_lookup|port_scan",
            Domain::NtWorld,
            "observe",
            3,
        ),
        kw_rule(
            r"(?i)explor|discover|survey|overview|categoriz|taxonom",
            Domain::NtCore,
            "discover",
            3,
        ),
        kw_rule(
            r"(?i)analyz|understand|classif|detect|cluster|ner\b",
            Domain::NtCore,
            "detect",
            3,
        ),
        kw_rule(
            r"(?i)metric|measure|score|benchmark|eval|quantif|statistic",
            Domain::NtCore,
            "measure",
            3,
        ),
        kw_rule(
            r"(?i)predict|forecast|trend|market",
            Domain::NtCore,
            "predict",
            3,
        ),
        kw_rule(
            r"(?i)plan|roadmap|scheduler|task_plan|goal",
            Domain::NtCore,
            "plan",
            3,
        ),
        kw_rule(
            r"(?i)reason|logic|infer|deduc|inference|chain_of_thought|critique",
            Domain::NtCore,
            "critique",
            3,
        ),
        kw_rule(
            r"(?i)explain|interpret|insight|attribution|xai\b",
            Domain::NtCore,
            "explain",
            3,
        ),
        kw_rule(
            r"(?i)wikipedia|encyclopedia|wiki\b",
            Domain::NtCore,
            "explain",
            3,
        ),
        kw_rule(
            r"(?i)karma|buddha|shinto|religion|philosoph|ethics|zen|tao",
            Domain::NtCore,
            "explain",
            3,
        ),
        kw_rule(
            r"(?i)generate|llm|gpt|model|prompt|text_gen|image_gen",
            Domain::NtMind,
            "generate",
            3,
        ),
        kw_rule(
            r"(?i)transform|translat|convert|summariz|rewrite|polish",
            Domain::NtMind,
            "transform",
            3,
        ),
        kw_rule(
            r"(?i)integrat|orchestrat|pipeline|workflow|compose|plugin",
            Domain::NtMind,
            "integrate",
            3,
        ),
        kw_rule(
            r"(?i)memory|remember|recall|store|persist|knowledge_base|database",
            Domain::NtMemory,
            "recall",
            3,
        ),
        kw_rule(
            r"(?i)model|simulat|world_model|environment|state_machine",
            Domain::NtMemory,
            "simulate",
            3,
        ),
        kw_rule(
            r"(?i)execut|tool|action|automation|script|cli\b|command|shell",
            Domain::NtAct,
            "execute",
            3,
        ),
        kw_rule(
            r"(?i)\bsdk\b|\blibrary\b|rest api|api wrapper",
            Domain::NtAct,
            "send",
            3,
        ),
        kw_rule(
            r"(?i)\bmcp (server|client|protocol)\b",
            Domain::NtAct,
            "send",
            3,
        ),
        kw_rule(
            r"(?i)webhook|notification|messaging|telegram|slack|discord",
            Domain::NtAct,
            "send",
            3,
        ),
        kw_rule(
            r"(?i)security|vuln|audit|scan|pen_test|pentest|exploit|firewall|shield|malware",
            Domain::NtShield,
            "audit",
            3,
        ),
        kw_rule(
            r"(?i)verify|test|validate|check|quality|assert|lint",
            Domain::NtShield,
            "verify",
            3,
        ),
        kw_rule(
            r"(?i)\bui\b|\bux\b|interface|frontend|design|dashboard|visual|component",
            Domain::NtIo,
            "invoke",
            3,
        ),
        kw_rule(
            r"(?i)communicat|chat|message|socket|stream|real_time|notify",
            Domain::NtIo,
            "synchronize",
            3,
        ),
        kw_rule(
            r"(?i)agent|multi_agent|delegate|subagent|swarm|coordinator",
            Domain::NtIo,
            "delegate",
            3,
        ),
        kw_rule(
            r"(?i)provider|gateway|model_router|llm_api|auth|login|sso|oauth",
            Domain::NtIo,
            "inquire",
            3,
        ),
        kw_rule(
            r"(?i)meta[-_ ]cognit|introspect|self[-_ ]aware|self[-_ ]monitor|self[-_ ]reflect",
            Domain::NtMeta,
            "introspect",
            3,
        ),
        kw_rule(
            r"(?i)consolidat|memory[-_ ]merg|narrative[-_ ]memor|compress|active[-_ ]forget|dream|replay",
            Domain::NtMemory,
            "consolidate",
            3,
        ),
        kw_rule(
            r"(?i)broadcast|global[-_ ]workspace|attention[-_ ]rout|routing|attend",
            Domain::NtCore,
            "broadcast",
            3,
        ),
    ]
});

static SOURCE_CORES: LazyLock<Vec<SourceCoreDef>> = LazyLock::new(|| {
    vec![
        SourceCoreDef {
            name: SourceCore::E8,
            keywords: vec![
                "symmetr",
                "mathemat",
                "algebra",
                "geometry",
                "theorem",
                "axiom",
                "formal",
                "topolog",
                "calculus",
                "equation",
                "quantum",
                "thermodynam",
                "entrop",
                "algorith",
                "complexity",
            ],
        },
        SourceCoreDef {
            name: SourceCore::Vsa,
            keywords: vec![
                "memor",
                "semant",
                "represent",
                "vector",
                "embed",
                "symbol",
                "meaning",
                "concept",
                "encod",
                "hypercub",
                "recall",
                "latent",
                "holographic",
                "distributed represent",
                "knowledge graph",
                "hyperdimension",
            ],
        },
        SourceCoreDef {
            name: SourceCore::Gwt,
            keywords: vec![
                "conscious",
                "consciousness",
                "percept",
                "aware",
                "cognition",
                "cognitiv",
                "global workspace",
                "mind",
                "sentient",
                "binding",
                "focus",
                "thalamus",
                "metacognit",
                "neurosci",
            ],
        },
        SourceCoreDef {
            name: SourceCore::ConsciousnessTree,
            keywords: vec![
                "absorb",
                "distill",
                "crystalliz",
                "evolve",
                "self-improv",
                "learn",
                "adapt",
                "internaliz",
                "feedback",
                "growth",
                "self-heal",
                "recursion",
                "reflect",
                "experience",
                "meta-learn",
                "self-evolv",
            ],
        },
        SourceCoreDef {
            name: SourceCore::MetaCognition,
            keywords: vec![
                "meta-cognit",
                "introspect",
                "self-evolv",
                "self-improv",
                "recursiv",
                "autotelic",
                "calibrat",
                "monitor",
                "self-aware",
                "self-monitor",
                "reflection",
                "self-correction",
                "oversight",
                "reward-model",
                "rlhf",
            ],
        },
        SourceCoreDef {
            name: SourceCore::Reality,
            keywords: vec![
                "world", "agent", "act", "action", "interact", "environ", "sensor", "control",
                "tool", "execute", "robot", "simulat", "perceiv", "explore", "embodied",
                "physical", "device", "deploy", "operate",
            ],
        },
    ]
});

struct RepoMapping {
    domain: Domain,
    capability: &'static str,
}

static KNOWN_REPOS: LazyLock<HashMap<&'static str, RepoMapping>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    macro_rules! repo {
        ($k:expr, $d:expr, $c:expr) => {
            m.insert(
                $k,
                RepoMapping {
                    domain: $d,
                    capability: $c,
                },
            );
        };
    }

    repo!("ollama/ollama", Domain::NtIo, "inquire");
    repo!("langchain-ai/langchain", Domain::NtMind, "integrate");
    repo!("microsoft/autogen", Domain::NtIo, "delegate");
    repo!("Aider-AI/aider", Domain::NtAct, "execute");
    repo!("microsoft/markitdown", Domain::NtMind, "transform");
    repo!("huggingface/transformers", Domain::NtMind, "generate");
    repo!("ggerganov/llama.cpp", Domain::NtMind, "generate");
    repo!("run-llama/llama_index", Domain::NtMemory, "recall");
    repo!("mem0ai/mem0", Domain::NtMemory, "recall");
    repo!("browser-use/browser-use", Domain::NtWorld, "retrieve");
    repo!("crewAIInc/crewAI", Domain::NtIo, "delegate");
    repo!("geekan/MetaGPT", Domain::NtIo, "delegate");
    repo!("open-webui/open-webui", Domain::NtIo, "invoke");
    repo!("soxoj/maigret", Domain::NtWorld, "observe");
    repo!("firecrawl/firecrawl", Domain::NtWorld, "retrieve");
    repo!("n8n-io/n8n", Domain::NtAct, "delegate");
    repo!("lobehub/lobe-chat", Domain::NtIo, "invoke");
    repo!(
        "AUTOMATIC1111/stable-diffusion-webui",
        Domain::NtMind,
        "generate"
    );
    repo!("anthropics/claude-code", Domain::NtAct, "execute");
    repo!("openai/swarm", Domain::NtAct, "delegate");
    repo!("karpathy/autoresearch", Domain::NtMind, "integrate");
    repo!("microsoft/graphrag", Domain::NtMemory, "recall");
    repo!("infiniflow/ragflow", Domain::NtMemory, "recall");
    repo!("expo/expo", Domain::NtIo, "invoke");
    repo!("milvus-io/milvus", Domain::NtMemory, "recall");
    repo!("livekit/agents", Domain::NtIo, "delegate");
    repo!("google-research/timesfm", Domain::NtCore, "predict");
    repo!("coqui-ai/TTS", Domain::NtMind, "generate");
    repo!("unslothai/unsloth", Domain::NtMind, "transform");
    repo!("CodebuffAI/codebuff", Domain::NtAct, "execute");
    repo!("facebookresearch/map-anything", Domain::NtWorld, "observe");
    repo!("openclaw/openclaw", Domain::NtIo, "delegate");
    repo!("langgenius/dify", Domain::NtMind, "integrate");
    repo!("langflow-ai/langflow", Domain::NtMind, "integrate");
    repo!("toeverything/AFFiNE", Domain::NtMemory, "persist");
    repo!("google/magika", Domain::NtShield, "verify");
    repo!("projectdiscovery/nuclei", Domain::NtShield, "audit");
    repo!("nmap", Domain::NtShield, "audit");
    repo!("sqlmap", Domain::NtShield, "audit");
    repo!("trivy", Domain::NtShield, "audit");
    repo!("snyk", Domain::NtShield, "audit");
    repo!("keycloak", Domain::NtShield, "constrain");
    repo!("casbin", Domain::NtShield, "constrain");
    repo!("ollama", Domain::NtIo, "inquire");
    repo!("khoj", Domain::NtMemory, "recall");
    repo!("maigret", Domain::NtWorld, "observe");
    m
});

fn kw_rule(pattern: &str, domain: Domain, capability: &str, title_weight: u32) -> KeywordRule {
    KeywordRule {
        pattern: Regex::new(pattern).unwrap(),
        domain,
        capability: capability.to_string(),
        title_weight,
    }
}

pub struct DomainMapper;

impl DomainMapper {
    pub fn map(input: &KnowledgeInput) -> MappingResult {
        let blob = Self::build_blob(input);

        if let Some((br, cap, ev)) = Self::try_known_repo(input) {
            let sc = Self::guess_source_core(&blob);
            return MappingResult {
                domain: br,
                capability: cap,
                source_core: sc,
                evidence: ev,
                trace_keywords: vec![],
            };
        }

        if let Some((domain, cap, score)) = Self::keyword_match(&blob, &input.title) {
            let sc = Self::guess_source_core(&blob);
            return MappingResult {
                domain,
                capability: cap,
                source_core: sc,
                evidence: format!("keyword_hits:{}", score),
                trace_keywords: vec![],
            };
        }

        let fallback = Self::type_fallback(&input);
        let sc = Self::guess_source_core(&blob);
        MappingResult {
            domain: fallback.0,
            capability: fallback.1.to_string(),
            source_core: sc,
            evidence: format!("fallback:{}", fallback.2),
            trace_keywords: vec![],
        }
    }

    fn build_blob(input: &KnowledgeInput) -> String {
        let mut parts = vec![input.title.as_str()];
        if let Some(ref c) = input.content {
            let truncated = if c.len() > 1200 {
                &c[..1200]
            } else {
                c.as_str()
            };
            parts.push(truncated);
        }
        parts.join(" ")
    }

    fn try_known_repo(input: &KnowledgeInput) -> Option<(Domain, String, String)> {
        if let Some(ref url) = input.url {
            // ⭐ 2026-10-03：host 判定，不再裸 contains。
            // ⛔ 与 nt_absorb_mapper 同形：`contains("github.com")` 让
            //    `https://github.com@evil.net/<key>` 进入本分支，
            //    再叠加下面 `lower.contains(&key)` 的 KNOWN_REPOS 匹配
            //    ⇒ 可用构造 URL 冒充任意已知仓库并套用其 capability。
            if crate::l0_substrate::nt_core_platform::url_match::url_matches_domain(
                url,
                "github.com",
            ) {
                let lower = url.to_lowercase();
                for (key, mapping) in KNOWN_REPOS.iter() {
                    if lower.contains(&key.to_lowercase()) {
                        return Some((
                            mapping.domain.clone(),
                            mapping.capability.to_string(),
                            format!("known_repo:{}", key),
                        ));
                    }
                }
            }
        }
        if input.source_kind == super::source_adapter::SourceKind::GitHubRepo {
            let lr = input.title.to_lowercase();
            for (key, mapping) in KNOWN_REPOS.iter() {
                let kl = key.to_lowercase();
                if lr.contains(&kl) || lr.ends_with(&kl) {
                    return Some((
                        mapping.domain.clone(),
                        mapping.capability.to_string(),
                        format!("known_repo:{}", key),
                    ));
                }
            }
        }
        None
    }

    fn keyword_match(blob: &str, title: &str) -> Option<(Domain, String, u32)> {
        let mut best: Option<(Domain, String, u32)> = None;
        for rule in KEYWORD_RULES.iter() {
            let title_hits = rule.pattern.find_iter(title).count() as u32;
            let blob_hits = rule.pattern.find_iter(blob).count() as u32;
            let score = title_hits * rule.title_weight + blob_hits;
            if score > 0 {
                if best.as_ref().map_or(true, |b| score > b.2) {
                    best = Some((rule.domain.clone(), rule.capability.clone(), score));
                }
            }
        }
        best
    }

    fn type_fallback(input: &KnowledgeInput) -> (Domain, &'static str, &'static str) {
        match input.source_kind {
            super::source_adapter::SourceKind::GitHubRepo => (Domain::NtWorld, "retrieve", "repo"),
            super::source_adapter::SourceKind::ArxivPaper => (Domain::NtCore, "critique", "paper"),
            super::source_adapter::SourceKind::GitHubTopic => (Domain::NtWorld, "search", "topic"),
            _ => (Domain::NtMemory, "recall", "default"),
        }
    }

    pub fn guess_source_core(blob: &str) -> SourceCore {
        let lower = blob.to_lowercase();
        let mut best = (SourceCore::Reality, 0usize);
        for def in SOURCE_CORES.iter() {
            let hits = def.keywords.iter().filter(|kw| lower.contains(*kw)).count();
            if hits > best.1 {
                best = (def.name.clone(), hits);
            }
        }
        best.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::seal::source_adapter::{SourceKind, SourceSpecific};

    fn make_input(title: &str, url: Option<&str>) -> KnowledgeInput {
        let mut meta = HashMap::new();
        let ss = SourceSpecific::Generic;
        KnowledgeInput {
            source_kind: SourceKind::Article,
            title: title.to_string(),
            summary: title.to_string(),
            content: None,
            url: url.map(|s| s.to_string()),
            domain: None,
            language: "en".into(),
            confidence: 1.0,
            importance: 0.5,
            metadata: meta,
            source_specific: ss,
        }
    }

    #[test]
    fn test_known_repo_mapping() {
        let input = make_input("ollama/ollama", Some("https://github.com/ollama/ollama"));
        let result = DomainMapper::map(&input);
        assert_eq!(result.domain, Domain::NtIo);
        assert_eq!(result.capability, "inquire");
    }

    #[test]
    fn test_keyword_mapping() {
        let input = make_input("security audit tool for containers", None);
        let result = DomainMapper::map(&input);
        assert_eq!(result.domain, Domain::NtShield);
    }

    #[test]
    fn test_source_core_detection() {
        let sc = DomainMapper::guess_source_core("quantum algorithm algebraic geometry");
        assert_eq!(sc, SourceCore::E8);
    }
}
