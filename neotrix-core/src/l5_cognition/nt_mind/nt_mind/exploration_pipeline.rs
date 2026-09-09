use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::self_iterating::ReasoningBrain;
use super::memory::ReasoningBank;
use super::web_miner::{WebKnowledgeMiner, WebSourceType};
use super::knowledge_miner::KnowledgeMiner;
use super::knowledge_engine::{KnowledgeEngine, KnowledgeEntry, SourceType};

/// 统一来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnifiedSourceType {
    Wikipedia,
    ArXiv,
    GitHub,
    GenericWeb,
    KnowledgeBase,
    SeedDomain,
}

impl UnifiedSourceType {
    pub fn detect(url: &str) -> Self {
        let lower = url.to_lowercase();
        if lower.contains("wikipedia.org") || lower.contains("wikidata.org") {
            UnifiedSourceType::Wikipedia
        } else if lower.contains("arxiv.org") || lower.contains("semanticscholar.org") {
            UnifiedSourceType::ArXiv
        } else if lower.contains("github.com") {
            UnifiedSourceType::GitHub
        } else {
            UnifiedSourceType::GenericWeb
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExploreDomain {
    Parapsychology,
    Theology,
    EsotericStudies,
    Wiki,
    Papers,
    GitHub,
    General,
    Consciousness,
    RustML,
    Security,
    MathPhysics,
}

impl ExploreDomain {
    pub fn name(&self) -> &'static str {
        match self {
            ExploreDomain::Parapsychology => "parapsychology",
            ExploreDomain::Theology => "theology",
            ExploreDomain::EsotericStudies => "esoteric-studies",
            ExploreDomain::Wiki => "wiki",
            ExploreDomain::Papers => "papers",
            ExploreDomain::GitHub => "github",
            ExploreDomain::General => "general",
            ExploreDomain::Consciousness => "consciousness",
            ExploreDomain::RustML => "rust-ml",
            ExploreDomain::Security => "nt_shield",
            ExploreDomain::MathPhysics => "math-physics",
        }
    }
}

pub use super::exploration_seeds::seed_urls_by_domain;

#[derive(Debug, Clone)]
pub struct ExploreRoundResult {
    pub domains_processed: Vec<ExploreDomain>,
    pub total_mined: usize,
    pub total_absorbed: usize,
    pub total_reward: f64,
    pub ke_entries_added: usize,
    pub new_discoveries: usize,
    pub goals_generated: usize,
    pub details: Vec<String>,
}

/// 统一探索管道 — 单入口处理所有外部知识吸收, 自动重试+发现+目标构建
pub struct ExplorationPipeline {
    pub work_dir: PathBuf,
    pub web_miner: WebKnowledgeMiner,
    pub knowledge_miner: KnowledgeMiner,
    pub knowledge_engine: KnowledgeEngine,
    pub seed_queue: VecDeque<(ExploreDomain, Vec<String>)>,
    pub processed: HashSet<String>,
    pub failed: HashMap<String, u32>,
    pub max_retries: u32,
    pub round_count: u64,
    pub domain_interval: HashMap<ExploreDomain, u64>,
    pub last_crawl: HashMap<ExploreDomain, i64>,
    /// 自动发现的 URL 缓存（来自已抓取内容的 Wikipedia 链接）
    pub auto_discovered: HashSet<String>,
}

impl ExplorationPipeline {
    pub fn new(work_dir: PathBuf) -> Self {
        let web_miner = WebKnowledgeMiner::new(work_dir.clone());
        let knowledge_miner = KnowledgeMiner::new(work_dir.clone());
        let ke_path = work_dir.join("knowledge_engine.json");
        let knowledge_engine = KnowledgeEngine::load_from(&ke_path);
        let mut domain_interval = HashMap::new();
        domain_interval.insert(ExploreDomain::Parapsychology, 3600);
        domain_interval.insert(ExploreDomain::Theology, 3600);
        domain_interval.insert(ExploreDomain::EsotericStudies, 3600);
        domain_interval.insert(ExploreDomain::Wiki, 3600);
        domain_interval.insert(ExploreDomain::Papers, 7200);
        domain_interval.insert(ExploreDomain::GitHub, 21600);
        domain_interval.insert(ExploreDomain::General, 36000);
        domain_interval.insert(ExploreDomain::Consciousness, 3600);
        domain_interval.insert(ExploreDomain::RustML, 7200);
        domain_interval.insert(ExploreDomain::Security, 7200);
        domain_interval.insert(ExploreDomain::MathPhysics, 3600);

        Self {
            work_dir,
            web_miner,
            knowledge_miner,
            knowledge_engine,
            seed_queue: VecDeque::new(),
            processed: HashSet::new(),
            failed: HashMap::new(),
            max_retries: 3,
            round_count: 0,
            domain_interval,
            last_crawl: HashMap::new(),
            auto_discovered: HashSet::new(),
        }
    }

    /// 统一入口：接受任意 URL/来源，分类并入队
    pub fn ingest(&mut self, url: &str, domain: Option<ExploreDomain>) {
        let effective_domain = domain.unwrap_or_else(|| {
            let src = UnifiedSourceType::detect(url);
            match src {
                UnifiedSourceType::GitHub => ExploreDomain::GitHub,
                UnifiedSourceType::ArXiv => ExploreDomain::Papers,
                UnifiedSourceType::Wikipedia => ExploreDomain::Wiki,
                _ => ExploreDomain::General,
            }
        });
        if !self.processed.contains(url) && !self.auto_discovered.contains(url) {
            self.seed_queue.push_back((effective_domain, vec![url.to_string()]));
        }
    }

    fn should_crawl(&self, domain: ExploreDomain) -> bool {
        let now = Utc::now().timestamp();
        let last = self.last_crawl.get(&domain).copied().unwrap_or(0);
        let interval = self.domain_interval.get(&domain).copied().unwrap_or(3600);
        (now - last) >= interval as i64
    }

    /// 入队一个域的种子 URL
    pub fn enqueue_domain(&mut self, domain: ExploreDomain) -> usize {
        let urls = seed_urls_by_domain(domain);
        let fresh: Vec<String> = urls.into_iter()
            .filter(|u| !self.processed.contains(u) && !self.auto_discovered.contains(u))
            .collect();
        let count = fresh.len();
        if count > 0 {
            self.seed_queue.push_back((domain, fresh));
        }
        count
    }

    /// 从已抓取的 Wikipedia 内容中提取新链接，自动发现相关页面
    fn discover_from_content(&mut self, mined: &[super::web_miner::WebMinedKnowledge], domain: ExploreDomain) -> usize {
        let mut discovered = 0usize;
        for kn in mined {
            let text = &kn.summary;
            let lower = text.to_lowercase();
            let domain_terms: &[&str] = match domain {
                ExploreDomain::Parapsychology => &[
                    "psi", "esp", "telepathy", "precognition", "psychokinesis", "medium",
                    "haunting", "poltergeist", "clairvoyance", "intuition", "anomaly",
                    "supernatural", "quantum consciousness", "orchestrated reduction",
                ],
                ExploreDomain::Theology => &[
                    "theology", "religion", "god", "divine", "sacred", "faith", "prayer",
                    "worship", "scripture", "revelation", "salvation", "grace",
                    "bible", "quran", "torah", "vedas", "sutra",
                ],
                ExploreDomain::EsotericStudies => &[
                    "occult", "mystery", "hermetic", "alchemical", "astral", "chakra",
                    "morphic", "akashic", "subtle body", "etheric", "initiation",
                    "correspondence", "synchronicity", "archetype",
                ],
                ExploreDomain::Consciousness => &[
                    "consciousness", "qualia", "phenomenology", "awareness", "sentience",
                    "integrated information", "phi", "global workspace", "binding",
                    "attention", "metacognition", "self-awareness", "theory of mind",
                    "predictive processing", "active inference", "free energy",
                    "neural correlate", "hard problem", "panpsychism", "iit",
                ],
                ExploreDomain::RustML => &[
                    "rust", "machine learning", "neural", "tensor", "deep learning",
                    "transformer", "reinforcement", "differentiable", "gradient",
                    "candle", "burn", "dfdx", "linfa", "tract",
                ],
                ExploreDomain::Security => &[
                    "nt_shield", "vulnerability", "exploit", "penetration", "injection",
                    "xss", "csrf", "buffer", "overflow", "mitigation", "owasp",
                    "cve", "zero-day", "authentication", "authorization",
                ],
                ExploreDomain::MathPhysics => &[
                    "category", "topology", "group theory", "representation", "gauge",
                    "symmetry", "entropy", "information", "complexity", "fractal",
                    "chaos", "quantum", "field theory", "renormalization", "gravity",
                ],
                _ => &["philosophy", "theory", "concept", "history", "research"],
            };
            let mut matched = false;
            for term in domain_terms {
                if lower.contains(term) {
                    matched = true;
                    break;
                }
            }
            if !matched { continue; }

            let title_lower = kn.title.to_lowercase().replace(' ', "_");
            let candidate = format!("https://en.wikipedia.org/wiki/{}", title_lower);
            if !self.processed.contains(&candidate) && !self.auto_discovered.contains(&candidate) && candidate != kn.source_url {
                self.auto_discovered.insert(candidate.clone());
                self.seed_queue.push_back((domain, vec![candidate]));
                discovered += 1;
                if discovered >= 10 { break; }
            }
        }
        discovered
    }

    /// 自动构建探索目标：根据能力缺口生成新的探索任务
    fn auto_generate_goals(&mut self, brain: &ReasoningBrain) -> usize {
        let mut goals = 0usize;
        let cap = &brain.capability;
        let arr = cap.arr();
        let weak_dims: Vec<(usize, f64)> = arr.iter().enumerate()
            .filter(|(_, &v)| v < 0.3)
            .map(|(i, &v)| (i, v))
            .collect();

        for (idx, _) in &weak_dims {
            let name = super::core::FIELD_NAMES.get(*idx).copied().unwrap_or("unknown");
            let new_urls: &[&str] = match name {
                "inference_depth" | "analysis" => &[
                    "https://en.wikipedia.org/wiki/Reasoning",
                    "https://en.wikipedia.org/wiki/Critical_thinking",
                    "https://en.wikipedia.org/wiki/Problem_solving",
                ],
                "synthesis" | "creativity" => &[
                    "https://en.wikipedia.org/wiki/Creativity",
                    "https://en.wikipedia.org/wiki/Innovation",
                    "https://en.wikipedia.org/wiki/Design_thinking",
                ],
                "domain_specificity" => &[
                    "https://en.wikipedia.org/wiki/Expert",
                    "https://en.wikipedia.org/wiki/Specialization",
                ],
                "experimental" => &[
                    "https://en.wikipedia.org/wiki/Scientific_method",
                    "https://en.wikipedia.org/wiki/Experiment",
                ],
                _ => continue,
            };
            for url in new_urls {
                let url_str = url.to_string();
                if !self.processed.contains(&url_str) && !self.auto_discovered.contains(&url_str) {
                    self.seed_queue.push_back((ExploreDomain::General, vec![url_str.clone()]));
                    self.auto_discovered.insert(url_str);
                    goals += 1;
                }
            }
            if goals >= 6 { break; }
        }
        goals
    }

    /// 处理一个域的所有待抓取 URL（含自动重试失败项）
    fn process_domain(
        &mut self,
        domain: ExploreDomain,
        brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
    ) -> (usize, f64, usize, usize) {
        let mut domain_urls: Vec<String> = Vec::new();
        let mut remaining: VecDeque<(ExploreDomain, Vec<String>)> = VecDeque::new();
        while let Some((d, urls)) = self.seed_queue.pop_front() {
            if d == domain {
                for url in urls {
                    let retries = self.failed.get(&url).copied().unwrap_or(0);
                    if self.processed.contains(&url) || retries >= self.max_retries { continue; }
                    self.processed.insert(url.clone());
                    domain_urls.push(url);
                }
            } else {
                remaining.push_back((d, urls));
            }
        }
        self.seed_queue = remaining;

        if domain_urls.is_empty() {
            return (0, 0.0, 0, 0);
        }

        let before_web = self.web_miner.mined_history.len();
        let before_ke = self.knowledge_engine.stats().total_entries;

        let mut web_urls: Vec<String> = Vec::new();
        let mut gh_urls: Vec<String> = Vec::new();
        for url in &domain_urls {
            match UnifiedSourceType::detect(url) {
                UnifiedSourceType::GitHub => { gh_urls.push(url.clone()); }
                _ => { web_urls.push(url.clone()); }
            }
        }

        let mut total_mined = 0usize;
        let mut total_reward = 0.0f64;

        // Phase 1: Web mining
        if !web_urls.is_empty() {
            let web_refs: Vec<&str> = web_urls.iter().map(|s| s.as_str()).collect();
            let result = self.web_miner.mine_all(&web_refs, brain, bank);
            total_mined += result.success_count;
            total_reward += result.total_reward;

            if result.success_count < web_urls.len() {
                for url in &web_urls {
                    let mined_urls: Vec<&str> = self.web_miner.mined_history.iter()
                        .map(|k| k.source_url.as_str()).collect();
                    if !mined_urls.contains(&url.as_str()) {
                        *self.failed.entry(url.clone()).or_insert(0) += 1;
                        self.processed.remove(url);
                    }
                }
            }

            for kn in &self.web_miner.mined_history[before_web..] {
                let src = match kn.source_type {
                    WebSourceType::Wikipedia => SourceType::Wikipedia,
                    WebSourceType::ArXiv => SourceType::ArXiv,
                    WebSourceType::GitHub => SourceType::GitHub,
                    WebSourceType::KnowledgeBase => SourceType::KnowledgeBase,
                    WebSourceType::GenericUrl => SourceType::WebPage,
                };
                let entry = KnowledgeEntry::new(
                    &kn.title, &kn.summary, src,
                    &format!("exploration:{}", domain.name()),
                ).with_tags(vec![domain.name().into(), kn.source_type.name().into()])
                 .with_importance(0.7 + kn.confidence * 0.2);
                self.knowledge_engine.add_entry(entry);
            }
        }

        // Phase 2: GitHub mining
        if !gh_urls.is_empty() {
            for url in &gh_urls {
                self.knowledge_miner.enqueue(url);
            }
            let gh_before_count = self.knowledge_miner.mined_sources.len();
            let gh_result = self.knowledge_miner.mine_round(brain, bank);
            let newly_mined = self.knowledge_miner.mined_sources.len() - gh_before_count;
            total_mined += newly_mined;

            if newly_mined < gh_urls.len() {
                for url in &gh_urls {
                    if !self.knowledge_miner.mined_sources.contains_key(url) {
                        *self.failed.entry(url.clone()).or_insert(0) += 1;
                        self.processed.remove(url);
                    }
                }
            }

            for kn in &gh_result.sources {
                let entry = KnowledgeEntry::new(
                    &kn.source_name, &kn.insights.join("; "), SourceType::GitHub,
                    &kn.source_url,
                ).with_tags(vec![domain.name().into(), "github".into()])
                 .with_importance(0.7 + kn.confidence * 0.2);
                self.knowledge_engine.add_entry(entry);
            }
        }

        // Phase 3: Auto-discovery from mined content
        let new_mined: Vec<super::web_miner::WebMinedKnowledge> = self.web_miner.mined_history[before_web..].to_vec();
        let discovery_count = self.discover_from_content(&new_mined, domain);

        let ke_added = self.knowledge_engine.stats().total_entries - before_ke;
        self.knowledge_engine.set_persist_path(self.work_dir.join("knowledge_engine.json"));
        let _ = self.knowledge_engine.compact();
        self.last_crawl.insert(domain, Utc::now().timestamp());

        (total_mined, total_reward, ke_added, discovery_count)
    }

    /// 单轮完整处理：检查所有域 -> 抓取 -> 自动发现 -> 目标构建
    pub fn run_round(
        &mut self,
        brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
    ) -> ExploreRoundResult {
        self.round_count += 1;
        let mut details = Vec::new();
        let mut total_mined = 0usize;
        let mut total_reward = 0.0f64;
        let mut total_ke = 0usize;
        let mut total_discoveries = 0usize;
        let mut domains_processed = Vec::new();

        // Phase A: Auto-generate goals from capability gaps
        let goals_added = self.auto_generate_goals(brain);
        if goals_added > 0 {
            details.push(format!("[goals] +{} exploration goals from weak dimensions", goals_added));
        }

        // Phase B: Process each domain
        let domains = vec![
            ExploreDomain::Consciousness,
            ExploreDomain::MathPhysics,
            ExploreDomain::Parapsychology,
            ExploreDomain::Theology,
            ExploreDomain::EsotericStudies,
            ExploreDomain::Wiki,
            ExploreDomain::RustML,
            ExploreDomain::Security,
            ExploreDomain::Papers,
            ExploreDomain::GitHub,
            ExploreDomain::General,
        ];

        for domain in domains {
            if !self.should_crawl(domain) { continue; }
            let enqueued = self.enqueue_domain(domain);
            if enqueued == 0 && !self.has_pending_for(domain) { continue; }

            domains_processed.push(domain);
            let (mined, reward, ke, discovered) = self.process_domain(domain, brain, bank);
            if mined > 0 || ke > 0 || discovered > 0 {
                total_mined += mined;
                total_reward += reward;
                total_ke += ke;
                total_discoveries += discovered;
                let mut parts = Vec::new();
                if mined > 0 { parts.push(format!("+{}mined", mined)); }
                if ke > 0 { parts.push(format!("+{}ke", ke)); }
                if discovered > 0 { parts.push(format!("+{}discovered", discovered)); }
                details.push(format!("[{}] {} reward={:.3}", domain.name(), parts.join(" "), reward));
            }
        }

        // Phase C: Retry failed URLs whose cooldown has passed
        let mut retried = 0usize;
        let now = Utc::now().timestamp();
        self.failed.retain(|url, retries| {
            let cooldown = 3600u64 * (*retries as u64);
            let last_attempt = self.processed.contains(url);
            if last_attempt && *retries < self.max_retries && now as u64 > cooldown {
                self.processed.remove(url);
                let domain = match UnifiedSourceType::detect(url) {
                    UnifiedSourceType::GitHub => ExploreDomain::GitHub,
                    UnifiedSourceType::ArXiv => ExploreDomain::Papers,
                    UnifiedSourceType::Wikipedia => ExploreDomain::Wiki,
                    _ => ExploreDomain::General,
                };
                self.seed_queue.push_back((domain, vec![url.clone()]));
                retried += 1;
                false
            } else {
                true
            }
        });
        if retried > 0 {
            details.push(format!("[retry] re-queued {} failed URLs for retry", retried));
        }

        if details.is_empty() {
            details.push("all domains up-to-date, sleeping until next cycle".into());
        }

        ExploreRoundResult {
            domains_processed,
            total_mined,
            total_absorbed: total_mined,
            total_reward,
            ke_entries_added: total_ke,
            new_discoveries: total_discoveries,
            goals_generated: goals_added,
            details,
        }
    }

    fn has_pending_for(&self, domain: ExploreDomain) -> bool {
        self.seed_queue.iter().any(|(d, _)| *d == domain)
    }

    /// 外部 URL 注入：从 SelfEvolver/用户输入等接收新 URL
    pub fn ingest_url(&mut self, url: &str) {
        self.ingest(url, None);
    }

    /// 统计
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            rounds: self.round_count,
            web_mined: self.web_miner.mined_history.len(),
            gh_mined: self.knowledge_miner.mined_sources.len(),
            ke_entries: self.knowledge_engine.stats().total_entries,
            queued: self.seed_queue.len(),
            processed: self.processed.len(),
            failed: self.failed.len(),
            auto_discovered: self.auto_discovered.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub rounds: u64,
    pub web_mined: usize,
    pub gh_mined: usize,
    pub ke_entries: usize,
    pub queued: usize,
    pub processed: usize,
    pub failed: usize,
    pub auto_discovered: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_source_type_detect_wikipedia() {
        assert_eq!(
            UnifiedSourceType::detect("https://en.wikipedia.org/wiki/Rust"),
            UnifiedSourceType::Wikipedia
        );
        assert_eq!(
            UnifiedSourceType::detect("https://wikidata.org/wiki/Q123"),
            UnifiedSourceType::Wikipedia
        );
    }

    #[test]
    fn test_unified_source_type_detect_arxiv() {
        assert_eq!(
            UnifiedSourceType::detect("https://arxiv.org/abs/2303.08774"),
            UnifiedSourceType::ArXiv
        );
        assert_eq!(
            UnifiedSourceType::detect("https://semanticscholar.org/paper/123"),
            UnifiedSourceType::ArXiv
        );
    }

    #[test]
    fn test_unified_source_type_detect_github() {
        assert_eq!(
            UnifiedSourceType::detect("https://github.com/rust-lang/rust"),
            UnifiedSourceType::GitHub
        );
        assert_eq!(
            UnifiedSourceType::detect("https://github.com/serde-rs/serde"),
            UnifiedSourceType::GitHub
        );
    }

    #[test]
    fn test_unified_source_type_detect_generic() {
        assert_eq!(
            UnifiedSourceType::detect("https://example.com"),
            UnifiedSourceType::GenericWeb
        );
        assert_eq!(
            UnifiedSourceType::detect("https://some-other-site.org/page"),
            UnifiedSourceType::GenericWeb
        );
    }

    #[test]
    fn test_unified_source_type_detect_unknown() {
        assert_eq!(
            UnifiedSourceType::detect(""),
            UnifiedSourceType::GenericWeb
        );
        assert_eq!(
            UnifiedSourceType::detect("not-a-url"),
            UnifiedSourceType::GenericWeb
        );
    }

    #[test]
    fn test_explore_domain_names() {
        assert_eq!(ExploreDomain::Parapsychology.name(), "parapsychology");
        assert_eq!(ExploreDomain::Theology.name(), "theology");
        assert_eq!(ExploreDomain::EsotericStudies.name(), "esoteric-studies");
        assert_eq!(ExploreDomain::Wiki.name(), "wiki");
        assert_eq!(ExploreDomain::Papers.name(), "papers");
        assert_eq!(ExploreDomain::GitHub.name(), "github");
        assert_eq!(ExploreDomain::General.name(), "general");
        assert_eq!(ExploreDomain::Consciousness.name(), "consciousness");
        assert_eq!(ExploreDomain::RustML.name(), "rust-ml");
        assert_eq!(ExploreDomain::Security.name(), "nt_shield");
        assert_eq!(ExploreDomain::MathPhysics.name(), "math-physics");
    }

    #[test]
    fn test_explore_domain_equality() {
        assert_eq!(ExploreDomain::Wiki, ExploreDomain::Wiki);
        assert_ne!(ExploreDomain::Wiki, ExploreDomain::Papers);
        assert_ne!(ExploreDomain::Parapsychology, ExploreDomain::Theology);
        assert_ne!(ExploreDomain::Consciousness, ExploreDomain::Security);
    }

    #[test]
    fn test_seed_urls_by_domain_general_is_empty() {
        let urls = seed_urls_by_domain(ExploreDomain::General);
        assert!(urls.is_empty());
    }

    #[test]
    fn test_seed_urls_by_domain_papers_not_empty() {
        let urls = seed_urls_by_domain(ExploreDomain::Papers);
        assert!(!urls.is_empty());
        assert!(urls.iter().all(|u| u.contains("arxiv.org")));
    }

    #[test]
    fn test_seed_urls_by_domain_consciousness_not_empty() {
        let urls = seed_urls_by_domain(ExploreDomain::Consciousness);
        assert!(!urls.is_empty());
        assert!(urls.iter().all(|u| u.contains("wikipedia.org")));
    }
}
