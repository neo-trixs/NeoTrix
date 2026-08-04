//! nt_absorb_mapper — 吸收→能力树映射器 (Absorb-to-Capability Mapper)
//!
//! 忠实移植自 `scripts/absorb_to_capability.py`。
//! 把 `batch_%` 吸收节点 (repository/paper/article) 映射到 36 原子能力 + 7 域
//! BranchKind + 5 本源 (source core), 写入 `nodes.metadata` 的
//! `absorbed_capability` / `knowledge_source` 字段 (Cycle 121 / 161i)。
//!
//! 36 原子能力 (Cycle 121) × 9 层:
//!   PERCEIVE   : retrieve/search/observe/receive
//!   UNDERSTAND : detect/classify/measure/predict/compare/discover
//!   REASON     : plan/decompose/critique/explain
//!   MODEL      : state/transition/attribute/ground/simulate
//!   SYNTHESIZE : generate/transform/integrate
//!   EXECUTE    : execute/mutate/send
//!   VERIFY     : verify/checkpoint/rollback/constrain/audit
//!   REMEMBER   : persist/recall
//!   COORDINATE : delegate/synchronize/invoke/inquire
//!
//! 7 域: NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-SHIELD, NT-IO

use regex::Regex;
use rusqlite::{Connection, params};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// 7 域 → 主属能力 (36 原子能力 Cycle 121)
pub fn branch_capabilities() -> BTreeMap<&'static str, &'static [&'static str]> {
    let mut m = BTreeMap::new();
    m.insert("NT-CORE", &["detect", "classify", "measure", "predict", "compare", "discover",
                          "plan", "decompose", "critique", "explain"][..]);
    m.insert("NT-MIND", &["generate", "transform", "integrate", "plan", "decompose"][..]);
    m.insert("NT-MEMORY", &["state", "transition", "attribute", "ground", "simulate",
                            "persist", "recall"][..]);
    m.insert("NT-WORLD", &["retrieve", "search", "observe", "receive"][..]);
    m.insert("NT-ACT", &["execute", "mutate", "send"][..]);
    m.insert("NT-SHIELD", &["verify", "checkpoint", "rollback", "constrain", "audit"][..]);
    m.insert("NT-IO", &["delegate", "synchronize", "invoke", "inquire"][..]);
    m
}

/// 5 道之本源 (Cycle 161i): 每节点溯源到一个本源 + 演化路径。
pub fn source_cores() -> &'static [(&'static str, &'static str, &'static [&'static str], &'static str)] {
    use std::sync::OnceLock;
    static CORES: OnceLock<Vec<(&str, &str, &[&str], &str)>> = OnceLock::new();
    CORES.get_or_init(|| {
        vec![
            ("E8", "NT-CORE", &["symmetr", "mathemat", "algebra", "geometry", "theorem", "axiom",
                                "formal", "topolog", "calculus", "equation", "fractal", "invariant",
                                "funct", "statistical mechan", "proof", "quantum", "thermodynam",
                                "entrop", "relativit", "hamiltonian", "particle", "differential",
                                "topolog", "set theor", "number theor", "homolog", "manifold",
                                "tensor", "optimiz", "algorith", "complexity theor"][..],
                "一切形式/结构/规律之源"),
            ("VSA", "NT-MEMORY", &["memor", "semant", "represent", "vector", "embed", "symbol",
                                   "meaning", "concept", "knowledge base", "encod", "hypercub",
                                   "recall", "retrieve", "latent", "holographic", "state space",
                                   "distributed represent", "kb", "embedding", "knowledge graph",
                                   "hyperdimension", "ontolog", "semantic memory", "associative memor",
                                   "content-addressable", "episodic memor", "working memor",
                                   "dual-coding", "vector symbolic", "holographic represent"][..],
                "一切概念/记忆/表示之源"),
            ("GWT", "NT-CORE", &["conscious", "consciousness", "percept", "aware", "cognition",
                                 "cognitiv", "global workspace", "integrat inform", "mind", "sentient",
                                 "binding", "focus", "thalamus", "metacognit", "introspect",
                                 "self-aware", "neurosci", "mental", "emotion", "brain",
                                 "neural activ", "cognitive architecture", "phenomenolog", "qualia",
                                 "self model", "cognitive model", "working memor", "cognitive science",
                                 "subjective experienc", "sense of self", "perception",
                                 "attention mechanism", "gwt", "workspace theor",
                                 "conscious experienc"][..],
                "一切意识/感知/认知之源"),
            ("ConsciousnessTree", "NT-MIND", &["absorb", "distill", "crystalliz", "evolve",
                                               "self-improv", "learn", "adapt", "internaliz",
                                               "feedback", "growth", "self-heal", "recursion",
                                               "reflect", "experience", "pattern recognit",
                                               "intuition", "pruning", "meta-learn", "self-organiz",
                                               "self-evolv", "autonom", "curriculum"][..],
                "一切元认知/吸收/演化之源"),
            ("Reality", "NT-WORLD", &["world", "world model", "agent", "act", "action", "interact",
                                      "environ", "sensor", "control", "tool", "execute", "robot",
                                      "simulat", "perceiv", "explore", "harvest", "crawl", "embodied",
                                      "real world", "physical", "device", "hardware", "deploy",
                                      "operate", "drone"][..],
                "一切世界/感知/行动之源"),
        ]
    })
}

/// 本源兜底启发式: 无关键词命中时按标题线索词分源。
fn fallback_hints() -> &'static [(&'static [&'static str], &'static str)] {
    use std::sync::OnceLock;
    static HINTS: OnceLock<Vec<(&[&str], &str)>> = OnceLock::new();
    HINTS.get_or_init(|| {
        vec![
            (&["math", "phys", "theor", "scien", "logic", "philosoph", "quantum", "chem", "astron",
               "relativ", "biolog", "geolog", "crystal", "equat", "axiom", "proof", "formal"][..], "E8"),
            (&["memor", "semantic", "represent", "concept", "knowledge", "intellig", "language",
               "symbol", "embed", "vector", "database", "graph", "word", "text"][..], "VSA"),
            (&["conscious", "mind", "brain", "cogni", "percept", "psych", "emotion", "aware", "neuro",
               "attention", "mental", "dream"][..], "GWT"),
            (&["learn", "evolv", "adapt", "growth", "self", "reflect", "experienc", "feedback",
               "develop", "train", "improv"][..], "ConsciousnessTree"),
            (&["world", "action", "agent", "society", "polit", "econom", "hist", "culture", "art",
               "war", "power", "soci", "commun", "technolog", "engineer", "industr", "market", "law",
               "govern", "earth", "space", "human", "life"][..], "Reality"),
        ]
    })
}

/// 本源溯源: 返回 (source_core, primary_domain, trace_keywords) 或 None。
/// 互斥判定: 取最高关键词命中数; 相同取列表序靠前者 (确定性)。
pub fn map_source_core(title: &str, content: &str, node_type: &str) -> Option<(&'static str, &'static str, Vec<&'static str>)> {
    let blob = format!("{} {}", title, &content.chars().take(2000).collect::<String>());

    // paper 载体默认溯源 E8, 除非内容强命中其它本源
    let (prior_core, prior_margin): (Option<&str>, f64) = match node_type {
        "paper" => (Some("E8"), 0.35),
        _ => (None, 0.0),
    };

    let mut best: Option<(&'static str, &'static str)> = None;
    let mut best_score = 0usize;
    let mut best_kws: Vec<&'static str> = Vec::new();

    for (name, domain, kws, _def) in source_cores() {
        let name: &'static str = name;
        let domain: &'static str = domain;
        let mut hits: Vec<(&'static str, usize)> = Vec::new();
        for kw in kws.iter().copied() {
            let re = Regex::new(&format!(r"(?i){}", regex::escape(kw)));
            let count = match re {
                Ok(r) => r.find_iter(&blob).count(),
                Err(_) => 0,
            };
            hits.push((kw, count));
        }
        let mut score: usize = hits.iter().map(|(_, h)| h).sum();
        if prior_core == Some(name) && score > 0 {
            score += ((score as f64 * prior_margin) as usize).max(1) + 2;
        }
        if score > best_score {
            best_score = score;
            best = Some((name, domain));
            let mut sorted = hits.clone();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            best_kws = sorted.iter().filter(|(_, h)| *h > 0).map(|(k, _)| *k).take(3).collect();
        }
    }

    if let Some((core, domain)) = best {
        if best_score > 0 {
            return Some((core, domain, best_kws));
        }
    }
    None
}

/// 兜底: 标题线索词分源。repository→Reality, paper→E8, 其余按线索词。
/// 无任何线索词 → Reality (世界知识大本营, 行动之源)。
fn fallback_source(title: &str, node_type: &str) -> (&'static str, &'static str, &'static str) {
    if node_type == "repository" {
        return ("Reality", "NT-WORLD", "tool");
    }
    if node_type == "paper" {
        return ("E8", "NT-CORE", "theory");
    }
    let blob = title.to_ascii_lowercase();
    for (kws, core) in fallback_hints() {
        if let Some(k) = kws.iter().find(|k| blob.contains(**k)) {
            let domain = source_cores()
                .iter()
                .find(|(c, _, _, _)| *c == *core)
                .map(|(_, d, _, _)| *d)
                .unwrap_or("NT-WORLD");
            return (core, domain, k);
        }
    }
    ("Reality", "NT-WORLD", "world")
}

/// 代码库已有节点名 → 能力树 (NT- 域) — 为 repository 节点提供确定性映射
pub fn known_repos() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("mattpocock/skills", "NT-CORE", "plan"),
        ("anthropics/skills", "NT-CORE", "plan"),
        ("google/skills", "NT-CORE", "plan"),
        ("claude-code", "NT-ACT", "execute"),
        ("openai/codex", "NT-ACT", "execute"),
        ("tauri", "NT-IO", "invoke"),
        ("camoufox", "NT-SHIELD", "constrain"),
        ("firecrawl", "NT-WORLD", "retrieve"),
        ("crawl4ai", "NT-WORLD", "retrieve"),
        ("OpenHands", "NT-ACT", "execute"),
        ("AutoAgent", "NT-ACT", "execute"),
        ("khoj", "NT-MEMORY", "recall"),
        ("librechat", "NT-IO", "synchronize"),
        ("langfuse", "NT-SHIELD", "verify"),
        ("flowise", "NT-MIND", "integrate"),
        ("markitdown", "NT-MIND", "transform"),
        ("maigret", "NT-WORLD", "observe"),
        ("MediaCrawler", "NT-WORLD", "retrieve"),
        ("mem0", "NT-MEMORY", "recall"),
        ("LightMem", "NT-MEMORY", "recall"),
        ("SimpleMem", "NT-MEMORY", "recall"),
        ("croc", "NT-ACT", "send"),
        ("kimi", "NT-MIND", "generate"),
        ("DeepSeek-V3", "NT-MIND", "generate"),
        ("UI-TARS", "NT-WORLD", "observe"),
        ("OmniParser", "NT-WORLD", "observe"),
        ("docling", "NT-MIND", "transform"),
        ("mineru", "NT-MIND", "transform"),
        ("GFPGAN", "NT-MIND", "transform"),
        ("exo", "NT-IO", "inquire"),
        ("ollama", "NT-IO", "inquire"),
        ("OpenManus", "NT-ACT", "execute"),
        ("Fabric", "NT-MIND", "integrate"),
        ("fabric", "NT-MIND", "integrate"),
        ("awesome-llm-apps", "NT-MIND", "integrate"),
        ("OpenResearcher", "NT-CORE", "explain"),
        ("FinanceDatabase", "NT-CORE", "predict"),
        ("supavec", "NT-IO", "invoke"),
        ("context7", "NT-MEMORY", "recall"),
        ("unstract", "NT-MEMORY", "recall"),
        ("kotaemon", "NT-MEMORY", "recall"),
        ("Serena", "NT-ACT", "execute"),
        ("jan", "NT-IO", "inquire"),
        ("chatbox", "NT-IO", "synchronize"),
        ("copilotkit", "NT-MIND", "generate"),
        ("Remotion", "NT-MIND", "generate"),
        ("hyperframes", "NT-MIND", "generate"),
        ("remotion", "NT-MIND", "generate"),
        ("livetalking", "NT-MIND", "generate"),
        ("pipecat", "NT-IO", "synchronize"),
        ("meshflow", "NT-MIND", "generate"),
        ("MeshFlow", "NT-MIND", "generate"),
        ("kroko", "NT-MIND", "generate"),
        ("graphify", "NT-MEMORY", "search"),
        ("Graphify", "NT-MEMORY", "search"),
        ("khoj-ai", "NT-MEMORY", "recall"),
        ("nmap", "NT-SHIELD", "audit"),
        ("sqlmap", "NT-SHIELD", "audit"),
        ("zaproxy", "NT-SHIELD", "audit"),
        ("grype", "NT-SHIELD", "audit"),
        ("trivy", "NT-SHIELD", "audit"),
        ("lynis", "NT-SHIELD", "audit"),
        ("wpscan", "NT-SHIELD", "audit"),
        ("impacket", "NT-SHIELD", "audit"),
        ("BloodHound", "NT-SHIELD", "audit"),
        ("Empire", "NT-SHIELD", "audit"),
        ("Amass", "NT-SHIELD", "audit"),
        ("phasar", "NT-SHIELD", "audit"),
        ("scorecard", "NT-SHIELD", "audit"),
        ("cosign", "NT-SHIELD", "verify"),
        ("sigstore", "NT-SHIELD", "verify"),
        ("opa", "NT-SHIELD", "constrain"),
        ("secureCodeBox", "NT-SHIELD", "verify"),
        ("constitutional-ai", "NT-SHIELD", "constrain"),
        ("llm-guard", "NT-SHIELD", "constrain"),
        ("guardrails", "NT-SHIELD", "constrain"),
        ("guidance", "NT-SHIELD", "constrain"),
        ("keycloak", "NT-SHIELD", "constrain"),
        ("casbin", "NT-SHIELD", "constrain"),
        ("certbot", "NT-SHIELD", "verify"),
        ("letsencrypt", "NT-SHIELD", "constrain"),
        ("vault", "NT-SHIELD", "constrain"),
        ("sops", "NT-SHIELD", "constrain"),
        ("snyk", "NT-SHIELD", "audit"),
        ("DependencyCheck", "NT-SHIELD", "audit"),
        ("cdxgen", "NT-SHIELD", "audit"),
        ("spdx-sbom-generator", "NT-SHIELD", "audit"),
        ("purl-spec", "NT-SHIELD", "constrain"),
        ("in-toto", "NT-SHIELD", "verify"),
        ("dependabot", "NT-SHIELD", "audit"),
        ("flare-vm", "NT-SHIELD", "audit"),
        ("sigma", "NT-SHIELD", "audit"),
        ("detection-rules", "NT-SHIELD", "audit"),
        ("attack_range", "NT-SHIELD", "audit"),
        ("evals", "NT-SHIELD", "verify"),
        ("kafka", "NT-IO", "synchronize"),
        ("nats-server", "NT-IO", "synchronize"),
        ("rabbitmq-server", "NT-IO", "synchronize"),
        ("mosquitto", "NT-IO", "synchronize"),
        ("emqx", "NT-IO", "synchronize"),
        ("nsq", "NT-IO", "synchronize"),
        ("libzmq", "NT-IO", "synchronize"),
        ("redis", "NT-IO", "invoke"),
        ("temporal", "NT-IO", "synchronize"),
        ("prefect", "NT-IO", "synchronize"),
        ("dagster", "NT-IO", "synchronize"),
        ("airflow", "NT-IO", "synchronize"),
        ("tree-of-thoughts", "NT-CORE", "critique"),
        ("human-eval", "NT-CORE", "measure"),
        ("peft", "NT-MIND", "transform"),
        ("LoRA", "NT-MIND", "transform"),
        ("OpenInstruct", "NT-MIND", "generate"),
        ("FastChat", "NT-MIND", "generate"),
        ("optuna", "NT-MIND", "transform"),
        ("BayesianOptimization", "NT-MIND", "transform"),
        ("keras-tuner", "NT-MIND", "transform"),
        ("talos", "NT-MIND", "transform"),
        ("AutoML", "NT-MIND", "integrate"),
        ("meta-dataset", "NT-MIND", "integrate"),
        ("MAML-Pytorch", "NT-MIND", "transform"),
        ("learn2learn", "NT-MIND", "transform"),
        ("pytorch-meta", "NT-MIND", "transform"),
        ("awesome-meta-learning", "NT-MIND", "integrate"),
        ("brian2", "NT-MEMORY", "simulate"),
        ("BrainPy", "NT-MEMORY", "simulate"),
        ("PyNN", "NT-MEMORY", "simulate"),
        ("BluePy", "NT-MEMORY", "simulate"),
        ("OpenWorm", "NT-MEMORY", "simulate"),
        ("neuropixels", "NT-CORE", "measure"),
        ("deap", "NT-MIND", "integrate"),
        ("jenetics", "NT-MIND", "integrate"),
        ("pagmo2", "NT-MIND", "integrate"),
        ("Platypus", "NT-MIND", "integrate"),
        ("cmaes", "NT-MIND", "transform"),
        ("opencog", "NT-MEMORY", "simulate"),
        ("bids-validator", "NT-CORE", "verify"),
        // Cycle 206 批 (2026-08-04 吸收 47 源) — 专家判定 (sub-agent 四字段表)
        ("kostja94/marketing-skills", "NT-IO", "invoke"),
        ("CyberStrike", "NT-SHIELD", "constrain"),
        ("zhaoxuya520/reverse-skill", "NT-SHIELD", "audit"),
        ("uditgoenka/autoresearch", "NT-MIND", "integrate"),
        ("averygan/reclip", "NT-WORLD", "retrieve"),
        ("github/spec-kit", "NT-CORE", "plan"),
        ("Kritt-ai/open-kritt", "NT-SHIELD", "audit"),
        ("yc-software/qm", "NT-ACT", "execute"),
        ("alibaba/open-code-review", "NT-CORE", "critique"),
        ("affaan-m/ECC", "NT-MIND", "integrate"),
        ("AgentSwarms-fyi/agentswarms", "NT-ACT", "delegate"),
        ("trycompai/crm", "NT-ACT", "execute"),
        ("xai-org/grok-build", "NT-ACT", "execute"),
        ("jakubkrehel/skills", "NT-IO", "invoke"),
        ("jakubkrehel/oklch-skill", "NT-IO", "invoke"),
        ("jakubkrehel/make-interfaces-feel-better", "NT-IO", "invoke"),
        ("robert-mcdermott/ai-knowledge-graph", "NT-MEMORY", "search"),
        ("vxcontrol/pentagi", "NT-SHIELD", "audit"),
        ("toeverything/AFFiNE", "NT-MEMORY", "persist"),
        ("huangruiteng/loopx", "NT-ACT", "execute"),
        ("google/magika", "NT-SHIELD", "verify"),
        ("phibrowser", "NT-WORLD", "observe"),
        ("lightpanda-io/browser", "NT-WORLD", "retrieve"),
        ("firecrawl/pdf-inspector", "NT-MEMORY", "recall"),
        ("aigclink/geolook", "NT-MIND", "integrate"),
        ("diegosouzapw/OmniRoute", "NT-IO", "inquire"),
        ("stablyai/orca", "NT-ACT", "delegate"),
        ("citrolabs/ego-lite", "NT-WORLD", "retrieve"),
        ("CoreBunch/Instatic", "NT-SHIELD", "constrain"),
        ("Lordog/dive-into-llms", "NT-MEMORY", "recall"),
        ("anthropics/claude-cookbooks", "NT-CORE", "plan"),
        ("NanoNets/Graft", "NT-MEMORY", "search"),
        ("ever-co/ever-gauzy", "NT-ACT", "execute"),
        ("superdesigndev/superdesign", "NT-IO", "invoke"),
        ("skalesapp/skales", "NT-ACT", "execute"),
        ("claraverse-space/ClaraVerse", "NT-MEMORY", "recall"),
        ("FareedKhan-dev/kimi-k3-in-c", "NT-MIND", "generate"),
        ("LasCC/HackTools", "NT-SHIELD", "audit"),
        ("taranis-ai/taranis-ai", "NT-WORLD", "observe"),
        ("ruvnet/ruflo", "NT-SHIELD", "constrain"),
        ("projectdiscovery/nuclei", "NT-SHIELD", "audit"),
        ("whiteguo233/OpenBiliClaw", "NT-WORLD", "observe"),
    ]
}

/// 关键词 → (域, 能力) 规则表 (编译一次)。
fn keyword_rules() -> &'static Vec<(Regex, &'static str, &'static str)> {
    use std::sync::OnceLock;
    static RULES: OnceLock<Vec<(Regex, &'static str, &'static str)>> = OnceLock::new();
    RULES.get_or_init(|| {
        [
            (r"crawl|scrap|fetcher|spider|crawler|browser|harvest|extract_web", "NT-WORLD", "retrieve"),
            (r"search|retriev|index|semantic_search|rag|vector", "NT-WORLD", "search"),
            (r"osint|recon|reconnaissance|subdomain|whois|dns_lookup|port_scan", "NT-WORLD", "observe"),
            (r"explor|discover|survey|overview|invent|categoriz|taxonom|reconnaissance", "NT-CORE", "discover"),
            (r"analyz|understand|classif|detect|cluster|topic_model|ner\b|segment", "NT-CORE", "detect"),
            (r"metric|measure|score|benchmark|eval|quantif|statistic", "NT-CORE", "measure"),
            (r"predict|forecast|forecast|trend|market", "NT-CORE", "predict"),
            (r"plan|roadmap|scheduler|task_plan|goal", "NT-CORE", "plan"),
            (r"reason|logic|infer|deduc|inference|chain_of_thought|debate|critique", "NT-CORE", "critique"),
            (r"explain|interpret|insight|attribution|xai\b", "NT-CORE", "explain"),
            (r"wikipedia|wikip|encyclopedia|wiki\b", "NT-CORE", "explain"),
            (r"karma|buddha|shinto|jain|hindu|veda|sanskrit|islam|quran|religion|philosoph|ethics|theolog|sutta|dharma|zen|tao|confuci", "NT-CORE", "explain"),
            (r"google books|open library|archive\.org|gutenberg|libgen|ebook|textbook", "NT-MEMORY", "recall"),
            (r"generate|llm|gpt|model|prompt|text_gen|completion|image_gen|video_gen", "NT-MIND", "generate"),
            (r"transform|translat|convert|summariz|rewrite|polish", "NT-MIND", "transform"),
            (r"integrat|orchestrat|pipeline|workflow|compose|plugin", "NT-MIND", "integrate"),
            (r"memory|remember|recall|store|persist|knowledge_base|kb\b|database|db\b", "NT-MEMORY", "recall"),
            (r"model|simulat|world_model|environment|state_machine", "NT-MEMORY", "simulate"),
            (r"execut|tool|action|automation|script|cli\b|command|terminal|shell", "NT-ACT", "execute"),
            (r"\bsdk\b|\bclient library\b|\blibrary\b|rest api wrapper|api wrapper|sdk for", "NT-ACT", "send"),
            (r"\bmcp (server|client|protocol)\b|mcp-|/mcp\b", "NT-ACT", "send"),
            (r"\bwebhook|notification|messaging|push\b|telegram|slack|discord|wechat", "NT-ACT", "send"),
            (r"security|vuln|audit|scan|pen_test|pentest|exploit|firewall|shield|protect|secur|pwn|hack|breach|malware|ransom", "NT-SHIELD", "audit"),
            (r"verify|test|validate|check|quality|assert|lint", "NT-SHIELD", "verify"),
            (r"ui\b|ux\b|interface|frontend|design|dashboard|visual|component", "NT-IO", "invoke"),
            (r"communicat|chat|message|socket|stream|real_time|notify|webhook", "NT-IO", "synchronize"),
            (r"agent|multi_agent|delegate|subagent|swarm|coordinator|router", "NT-IO", "delegate"),
            (r"provider|gateway|model_router|llm_api|auth|login|sso|oauth", "NT-IO", "inquire"),
        ]
        .iter()
        .map(|(pat, br, cap)| (Regex::new(pat).expect("静态能力正则必须合法"), *br, *cap))
        .collect()
    })
}

/// `'Karpathy AutoResearch'` 或 `'GitHub - owner/repo: desc'` → owner/repo
fn normalize_repo_title(title: &str) -> String {
    let t = title.strip_prefix("GitHub - ").unwrap_or(title);
    t.split(':').next().map(|s| s.trim()).unwrap_or(t.trim()).to_string()
}

/// 映射单节点 → (branch, capability, evidence)。
pub fn map_node(node_type: &str, title: &str, content: &str, url: &str) -> Option<(&'static str, &'static str, String)> {
    // 1. KNOWN_REPOS 确定性映射 (URL 判真优先, 任意 node_type)
    if !url.is_empty() && url.contains("github.com") {
        let url_low = url.to_ascii_lowercase();
        let last = url_low.trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string();
        // Pass 1: 完整 owner/repo key
        for (k, br, cap) in known_repos() {
            let kl = k.to_ascii_lowercase();
            if k.contains('/') && url_low.contains(&kl) {
                return Some((br, cap, format!("known_repo:{k}")));
            }
        }
        // Pass 2: 裸 key 须等于 URL 末段
        for (k, br, cap) in known_repos() {
            let kl = k.to_ascii_lowercase();
            if !k.contains('/') && kl == last {
                return Some((br, cap, format!("known_repo:{k}")));
            }
        }
    }
    if node_type == "repository" {
        let owner_repo = normalize_repo_title(title).to_ascii_lowercase();
        for (k, br, cap) in known_repos() {
            let kl = k.to_ascii_lowercase();
            let bare = kl.split('/').next_back().unwrap_or(&kl).to_string();
            if owner_repo.contains(&kl) || owner_repo.ends_with(&kl) || owner_repo == bare {
                return Some((br, cap, format!("known_repo:{k}")));
            }
        }
    }

    // 2. 关键词规则
    let title_lower = title.to_ascii_lowercase();
    let blob = format!("{} {}", title, &content.chars().take(1200).collect::<String>());
    let mut best: Option<(&'static str, &'static str)> = None;
    let mut best_hits = 0usize;

    for (re, br, cap) in keyword_rules() {
        let hits = re.find_iter(&blob).count();
        // title 命中权重 ×3 (title 比 README 正文更具判别力)
        let title_hits = re.find_iter(&title_lower).count();
        let score = title_hits * 3 + hits;
        if score > best_hits {
            best_hits = score;
            best = Some((br, cap));
        }
    }
    if let Some((br, cap)) = best {
        return Some((br, cap, format!("keyword_hits:{best_hits}")));
    }

    // 3. 本源感知兜底
    if node_type == "repository" {
        return Some(("NT-WORLD", "retrieve", "fallback:repo".to_string()));
    }
    if node_type == "paper" {
        return Some(("NT-CORE", "critique", "fallback:paper".to_string()));
    }

    // 4. node_type 语义默认
    let type_default: &[(&str, &str, &str)] = &[
        ("concept", "NT-MEMORY", "recall"),
        ("web", "NT-WORLD", "search"),
        ("external", "NT-WORLD", "retrieve"),
        ("skill", "NT-ACT", "execute"),
        ("doi", "NT-CORE", "critique"),
        ("arxiv", "NT-CORE", "critique"),
        ("wikipedia", "NT-MEMORY", "recall"),
        ("reference", "NT-MEMORY", "recall"),
        ("book", "NT-MEMORY", "recall"),
        ("guide", "NT-IO", "invoke"),
        ("method", "NT-MIND", "integrate"),
        ("framework", "NT-MIND", "integrate"),
        ("insight", "NT-CORE", "explain"),
        ("thinking_trace", "NT-MIND", "integrate"),
        ("theory", "NT-CORE", "critique"),
        ("person", "NT-CORE", "explain"),
        ("organization", "NT-CORE", "explain"),
        ("evolution_pattern", "NT-MIND", "integrate"),
        ("conversation_evolution", "NT-MIND", "integrate"),
        ("resource", "NT-MEMORY", "recall"),
        ("source", "NT-MEMORY", "recall"),
        ("image", "NT-IO", "invoke"),
        ("wiki_page", "NT-CORE", "explain"),
        ("algorithm", "NT-CORE", "measure"),
        ("note", "NT-MEMORY", "recall"),
        ("event_record", "NT-CORE", "measure"),
        ("detection_finding", "NT-CORE", "detect"),
        ("goal_result", "NT-CORE", "measure"),
        ("github", "NT-ACT", "execute"),
    ];
    if let Some((_t, br, cap)) = type_default.iter().find(|(t, _, _)| *t == node_type) {
        return Some((br, cap, format!("fallback:type:{node_type}")));
    }

    // 5. 本源映射兜底
    let core = map_source_core(title, content, node_type)
        .map(|(c, _, _)| c)
        .or_else(|| Some(fallback_source(title, node_type).0));
    let src_cap: &[(&str, &str, &str)] = &[
        ("E8", "NT-CORE", "measure"),
        ("VSA", "NT-MEMORY", "recall"),
        ("GWT", "NT-CORE", "critique"),
        ("ConsciousnessTree", "NT-MIND", "integrate"),
        ("Reality", "NT-MEMORY", "recall"),
    ];
    if let Some(core) = core {
        if let Some((_c, br, cap)) = src_cap.iter().find(|(c, _, _)| *c == core) {
            return Some((br, cap, format!("fallback:core:{core}")));
        }
    }
    Some(("NT-CORE", "discover", "fallback:core:unknown".to_string()))
}

/// 映射结果条目 (Python `mapped[nid]` dict)。
#[derive(Debug, Clone)]
pub struct CapabilityMapping {
    pub branch: &'static str,
    pub capability: &'static str,
    pub evidence: String,
    pub node_type: String,
    pub title: String,
    pub url: String,
    pub source_core: Option<&'static str>,
    pub source_domain: Option<&'static str>,
    pub trace_keywords: Vec<&'static str>,
}

/// 覆盖率报告。
#[derive(Debug, Default)]
pub struct MappingReport {
    pub mapped: usize,
    pub total: usize,
    pub per_branch: BTreeMap<String, Vec<String>>,
    pub per_cap: BTreeMap<String, usize>,
    pub per_source: BTreeMap<String, usize>,
    pub unmapped: Vec<(String, String)>,
}

impl MappingReport {
    pub fn unknown_source(&self, total: usize) -> usize {
        let sum: usize = self.per_source.values().sum();
        total.saturating_sub(sum)
    }
}

/// 读取 batch_% 节点并映射, 返回映射表 (不写库)。
pub fn map_batch_nodes(conn: &Connection) -> rusqlite::Result<(Vec<(String, CapabilityMapping)>, MappingReport)> {
    map_nodes(conn, Some("batch_"), None, None)
}

/// 读取全部节点并映射 (全库本源溯源, 对应 scripts/absorb_full_kb.py)。
pub fn map_all_nodes(conn: &Connection) -> rusqlite::Result<(Vec<(String, CapabilityMapping)>, MappingReport)> {
    map_nodes(conn, None, None, None)
}

/// 泛化节点映射: 可选 id 前缀 / node_type 白名单 / limit。
pub fn map_nodes(
    conn: &Connection,
    prefix: Option<&str>,
    types: Option<&[String]>,
    limit: Option<usize>,
) -> rusqlite::Result<(Vec<(String, CapabilityMapping)>, MappingReport)> {
    let pattern = prefix.map(|p| format!("{}%", p)).unwrap_or_else(|| "%".to_string());
    let mut sql = String::from(
        "SELECT id, node_type, title, content, url, metadata FROM nodes WHERE id LIKE ?1",
    );
    let mut binds: Vec<String> = vec![pattern];
    if let Some(ts) = types {
        if !ts.is_empty() {
            let placeholders = vec!["?"; ts.len()].join(",");
            sql.push_str(&format!(" AND node_type IN ({})", placeholders));
            binds.extend(ts.iter().cloned());
        }
    }
    if let Some(l) = limit {
        sql.push_str(" LIMIT ?");
        binds.push(l.to_string());
    }
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(binds.iter()), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    })?;

    let mut mapped: Vec<(String, CapabilityMapping)> = Vec::new();
    let mut report = MappingReport::default();
    let mut seen_total = 0usize;

    for row in rows {
        let (nid, node_type, title, content, url, meta_json) = row?;
        seen_total += 1;
        report.total = seen_total;
        let title = title.unwrap_or_default();
        let content = content.unwrap_or_default();
        let url = url.unwrap_or_default();

        // GitHub topics/description 补充本源判定
        let mut topics: Vec<String> = Vec::new();
        if let Some(mj) = meta_json {
            if let Ok(md) = serde_json::from_str::<Value>(&mj) {
                if let Some(t) = md.get("topics").and_then(Value::as_array) {
                    for v in t {
                        if let Some(s) = v.as_str() {
                            topics.push(s.to_string());
                        }
                    }
                }
                if let Some(d) = md.get("description").and_then(Value::as_str) {
                    topics.push(d.to_string());
                }
            }
        }
        let topic_blob = topics.join(" ");

        let res = map_node(&node_type, &title, &content, &url);
        let Some((branch, cap, ev)) = res else {
            report.unmapped.push((nid.clone(), title.clone()));
            continue;
        };

        // 本源溯源层
        let mut src = map_source_core(&title, &content, &node_type);
        if src.is_none() && !topic_blob.is_empty() {
            src = map_source_core(&topic_blob, "", &node_type);
        }
        let src = src.unwrap_or_else(|| {
            let (c, d, k) = fallback_source(&title, &node_type);
            (c, d, vec![k])
        });

        report.per_branch.entry(branch.to_string()).or_default().push(cap.to_string());
        *report.per_cap.entry(cap.to_string()).or_insert(0) += 1;
        *report.per_source.entry(src.0.to_string()).or_insert(0) += 1;

        mapped.push((
            nid,
            CapabilityMapping {
                branch,
                capability: cap,
                evidence: ev,
                node_type: node_type.clone(),
                title: title.chars().take(60).collect(),
                url,
                source_core: Some(src.0),
                source_domain: Some(src.1),
                trace_keywords: src.2,
            },
        ));
    }

    Ok((mapped, report))
}

/// 写库: 合并 `absorbed_capability` + `knowledge_source` 进 metadata (read-modify-write)。
pub fn apply_mappings(conn: &Connection, mappings: &[(String, CapabilityMapping)]) -> rusqlite::Result<usize> {
    use crate::neotrix::nt_memory_kb::nt_memory_store;
    let now = unix_now();
    for (nid, m) in mappings {
        let meta = read_metadata(conn, nid)?;
        let mut meta = meta;
        meta.insert(
            "absorbed_capability".to_string(),
            serde_json::json!({
                "branch": m.branch,
                "capability": m.capability,
                "evidence": m.evidence,
                "mapped_at": now,
            }),
        );
        if let Some(core) = m.source_core {
            meta.insert(
                "knowledge_source".to_string(),
                serde_json::json!({
                    "source_core": core,
                    "primary_domain": m.source_domain,
                    "trace_path": m.trace_keywords,
                    "mapped_at": now,
                }),
            );
        }
        nt_memory_store::update_node_metadata(conn, nid, &Value::Object(meta))?;
    }
    Ok(mappings.len())
}

/// 从 KB nodes 的 `absorbed_capability` 元数据加载 `(branch_str, capability)` 对,
/// 供 ConsciousnessTree 能力网同步 (R-P79 闭环, Cycle 206)。
pub fn load_absorbed_capabilities(conn: &Connection) -> rusqlite::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT metadata FROM nodes WHERE metadata IS NOT NULL")?;
    let rows = stmt.query_map([], |row| row.get::<_, Option<String>>(0))?;
    let mut out: Vec<(String, String)> = Vec::new();
    for row in rows {
        let Some(mj) = row? else { continue };
        let Ok(md) = serde_json::from_str::<Value>(&mj) else { continue };
        let Some(ac) = md.get("absorbed_capability") else { continue };
        let (Some(branch), Some(cap)) = (
            ac.get("branch").and_then(Value::as_str),
            ac.get("capability").and_then(Value::as_str),
        ) else {
            continue;
        };
        out.push((branch.to_string(), cap.to_string()));
    }
    Ok(out)
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn read_metadata(conn: &Connection, nid: &str) -> rusqlite::Result<Map<String, Value>> {
    let cur = conn.query_row(
        "SELECT metadata FROM nodes WHERE id=?1",
        params![nid],
        |row| row.get::<_, Option<String>>(0),
    )?;
    Ok(match cur {
        Some(raw) => serde_json::from_str::<Map<String, Value>>(&raw).unwrap_or_default(),
        None => Map::new(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::neotrix::nt_memory_kb::nt_memory_schema;
    use std::collections::BTreeSet;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_map_repository_known_repo() {
        let (br, cap, ev) = map_node("repository", "GitHub - openai/codex: coding agent", "src", "").unwrap();
        assert_eq!(br, "NT-ACT");
        assert_eq!(cap, "execute");
        assert!(ev.starts_with("known_repo:"));
    }

    #[test]
    fn test_map_paper_default() {
        let (br, cap, _) = map_node("paper", "Attention Is All You Need", "neutral prose", "").unwrap();
        assert_eq!(br, "NT-CORE");
        assert_eq!(cap, "critique");
    }

    #[test]
    fn test_map_repository_fallback() {
        let (br, cap, ev) = map_node("repository", "Some Random Repo No Keywords", "no content", "").unwrap();
        assert_eq!(br, "NT-WORLD");
        assert_eq!(cap, "retrieve");
        assert_eq!(ev, "fallback:repo");
    }

    #[test]
    fn test_map_web_type_default() {
        let (br, cap, ev) = map_node("web", "title", "content", "").unwrap();
        assert_eq!(br, "NT-WORLD");
        assert_eq!(cap, "search");
        assert!(ev.starts_with("fallback:type:"));
    }

    #[test]
    fn test_map_source_core_e8() {
        let r = map_source_core("Quantum Field Theory", "algebra geometry theorem", "paper");
        assert!(r.is_some());
        assert_eq!(r.unwrap().0, "E8");
    }

    #[test]
    fn test_fallback_source_repo() {
        let (core, domain, kw) = fallback_source("whatever", "repository");
        assert_eq!(core, "Reality");
        assert_eq!(domain, "NT-WORLD");
        assert_eq!(kw, "tool");
    }

    #[test]
    fn test_normalize_repo_title() {
        assert_eq!(normalize_repo_title("GitHub - openai/codex: desc"), "openai/codex");
        assert_eq!(normalize_repo_title("Karpathy AutoResearch"), "Karpathy AutoResearch");
    }

    #[test]
    fn test_map_batch_nodes_and_apply() {
        let conn = test_db();
        let now = unix_now();
        let now_str = now.to_string();
        conn.execute(
            "INSERT INTO nodes(id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,access_count,metadata,data_tier,temporal,supersedes,source_episode,tier) VALUES(?1,'repository','GitHub - openai/codex: desc','s','c','https://github.com/openai/codex','github.com','en',1.0,0.7,?2,?3,0,'{}','cache',NULL,NULL,NULL,'warm')",
            params![format!("batch_{now_str}_abcdef12"), now, now],
        ).unwrap();

        let (mapped, report) = map_batch_nodes(&conn).unwrap();
        assert_eq!(mapped.len(), 1);
        assert_eq!(report.mapped, 0); // 在 apply 前 mapped 计数为 0, total=1
        assert_eq!(report.total, 1);
        assert_eq!(mapped[0].1.branch, "NT-ACT");

        apply_mappings(&conn, &mapped).unwrap();
        let meta: Option<String> = conn
            .query_row("SELECT metadata FROM nodes WHERE id LIKE 'batch_%'", [], |r| r.get(0))
            .unwrap();
        let v: Value = serde_json::from_str(&meta.unwrap()).unwrap();
        assert_eq!(v["absorbed_capability"]["capability"], "execute");
        assert_eq!(v["knowledge_source"]["source_core"], "Reality");
    }

    #[test]
    fn test_map_all_nodes_with_type_and_limit() {
        let conn = test_db();
        let now = unix_now();
        for (i, (nid, ntype)) in [
            ("u_aaa1", "repository"),
            ("u_aaa2", "paper"),
            ("u_aaa3", "article"),
        ]
        .iter()
        .enumerate()
        {
            conn.execute(
                "INSERT INTO nodes(id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,access_count,metadata,data_tier,temporal,supersedes,source_episode,tier) VALUES(?1,?2,?3,'s','c','https://github.com/openai/codex','github.com','en',1.0,0.7,?4,?5,0,'{}','cache',NULL,NULL,NULL,'warm')",
                params![format!("{}_{}", nid, now), ntype, format!("GitHub - openai/codex #{}", i), now, now],
            ).unwrap();
        }

        // 全库: 3 节点全部进入
        let (mapped_all, _) = map_all_nodes(&conn).unwrap();
        assert_eq!(mapped_all.len(), 3);

        // 类型白名单: 仅 repository
        let types = vec!["repository".to_string()];
        let (mapped_repo, _) = map_nodes(&conn, None, Some(&types), None).unwrap();
        assert_eq!(mapped_repo.len(), 1);
        assert_eq!(mapped_repo[0].1.node_type, "repository");

        // limit: 全库仅取 2
        let (mapped_lim, _) = map_nodes(&conn, None, None, Some(2)).unwrap();
        assert_eq!(mapped_lim.len(), 2);
    }

    #[test]
    fn test_load_absorbed_capabilities_roundtrip() {
        let conn = test_db();
        assert!(load_absorbed_capabilities(&conn).unwrap().is_empty());

        let now = unix_now();
        conn.execute(
            "INSERT INTO nodes(id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,access_count,metadata,data_tier,temporal,supersedes,source_episode,tier) VALUES(?1,'repository','GitHub - openai/codex: desc','s','c','https://github.com/openai/codex','github.com','en',1.0,0.7,?2,?3,0,'{}','cache',NULL,NULL,NULL,'warm')",
            params![format!("u_{now}"), now, now],
        ).unwrap();
        let (mapped, _) = map_batch_nodes(&conn).unwrap();
        assert!(mapped.is_empty());
        let meta = serde_json::json!({
            "absorbed_capability": {"branch": "NT-ACT", "capability": "execute", "evidence": "known_repo:openai/codex", "mapped_at": now},
            "knowledge_source": {"source_core": "Reality", "primary_domain": "NT-ACT", "mapped_at": now},
        });
        conn.execute("UPDATE nodes SET metadata = ?1", params![meta.to_string()]).unwrap();

        let pairs = load_absorbed_capabilities(&conn).unwrap();
        assert_eq!(pairs, vec![("NT-ACT".to_string(), "execute".to_string())]);
    }

    #[test]
    fn test_branch_capabilities_36() {
        let caps: BTreeSet<&str> = branch_capabilities()
            .values()
            .flat_map(|c| c.iter().copied())
            .collect();
        assert_eq!(caps.len(), 36);
    }
}
