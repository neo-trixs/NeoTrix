use chrono::Utc;
use std::time::Instant;

use super::super::knowledge_engine::{KnowledgeEntry, KnowledgeSourceType};
use super::super::memory::ReasoningBank;
use super::super::self_iterating::ReasoningBrain;
use super::super::web_miner::{WebMinedKnowledge, WebSourceType};
use super::*;

impl ExplorationPipeline {
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
                    if self.processed.contains(&url) || retries >= self.max_retries {
                        continue;
                    }
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
            match UnifiedKnowledgeSourceType::detect(url) {
                UnifiedKnowledgeSourceType::GitHub => {
                    gh_urls.push(url.clone());
                }
                _ => {
                    web_urls.push(url.clone());
                }
            }
        }

        let mut total_mined = 0usize;
        let mut total_reward = 0.0f64;

        // Phase 1: Web mining with LRU cache
        if !web_urls.is_empty() {
            // Split URLs into cache hits (valid TTL) and misses
            let mut misses: Vec<String> = Vec::new();
            for url in &web_urls {
                let ttl = self.cache_ttl_for_url(url);
                let hit = self.explore_cache.get(url).and_then(|cached| {
                    if cached.fetched_at.elapsed() < ttl {
                        Some(cached.result.clone())
                    } else {
                        None
                    }
                });
                if let Some(result) = hit {
                    self.web_miner.mined_history.push(result);
                    total_mined += 1;
                    self.cache_hits += 1;
                    continue;
                }
                self.cache_misses += 1;
                misses.push(url.clone());
            }

            if !misses.is_empty() {
                let miss_refs: Vec<&str> = misses.iter().map(|s| s.as_str()).collect();
                let result = self.web_miner.mine_all(&miss_refs, brain, bank);
                total_mined += result.success_count;
                total_reward += result.total_reward;

                if result.success_count < misses.len() {
                    for url in &misses {
                        let mined_urls: Vec<&str> = self
                            .web_miner
                            .mined_history
                            .iter()
                            .map(|k| k.source_url.as_str())
                            .collect();
                        if !mined_urls.contains(&url.as_str()) {
                            *self.failed.entry(url.clone()).or_insert(0) += 1;
                            self.processed.remove(url);
                        }
                    }
                }
            }

            // Cache all newly mined results (since before_web)
            for kn in &self.web_miner.mined_history[before_web..] {
                self.explore_cache.put(
                    kn.source_url.clone(),
                    CachedExploreResult {
                        result: kn.clone(),
                        fetched_at: Instant::now(),
                    },
                );
            }

            // Feed KE from all new entries (cache hits + fresh mines)
            for kn in &self.web_miner.mined_history[before_web..] {
                let src = match kn.source_type {
                    WebSourceType::Wikipedia => KnowledgeSourceType::Wikipedia,
                    WebSourceType::ArXiv => KnowledgeSourceType::ArXiv,
                    WebSourceType::GitHub => KnowledgeSourceType::GitHub,
                    WebSourceType::KnowledgeBase => KnowledgeSourceType::KnowledgeBase,
                    WebSourceType::GenericUrl => KnowledgeSourceType::WebPage,
                };
                let entry = KnowledgeEntry::new(
                    &kn.title,
                    &kn.summary,
                    src,
                    &format!("exploration:{}", domain.name()),
                )
                .with_tags(vec![domain.name().into(), kn.source_type.name().into()])
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
                    &kn.source_name,
                    &kn.insights.join("; "),
                    KnowledgeSourceType::GitHub,
                    &kn.source_url,
                )
                .with_tags(vec![domain.name().into(), "github".into()])
                .with_importance(0.7 + kn.confidence * 0.2);
                self.knowledge_engine.add_entry(entry);
            }
        }

        // Phase 3: Auto-discovery from mined content
        let new_mined: Vec<WebMinedKnowledge> = self.web_miner.mined_history[before_web..].to_vec();
        let discovery_count = self.discover_from_content(&new_mined, domain);

        let ke_added = self.knowledge_engine.stats().total_entries - before_ke;
        self.knowledge_engine
            .set_persist_path(self.work_dir.join("knowledge_engine.json"));
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
            details.push(format!(
                "[goals] +{} exploration goals from weak dimensions",
                goals_added
            ));
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
            if !self.should_crawl(domain) {
                continue;
            }
            let enqueued = self.enqueue_domain(domain);
            if enqueued == 0 && !self.has_pending_for(domain) {
                continue;
            }

            domains_processed.push(domain);
            let (mined, reward, ke, discovered) = self.process_domain(domain, brain, bank);
            if mined > 0 || ke > 0 || discovered > 0 {
                total_mined += mined;
                total_reward += reward;
                total_ke += ke;
                total_discoveries += discovered;
                let mut parts = Vec::new();
                if mined > 0 {
                    parts.push(format!("+{}mined", mined));
                }
                if ke > 0 {
                    parts.push(format!("+{}ke", ke));
                }
                if discovered > 0 {
                    parts.push(format!("+{}discovered", discovered));
                }
                details.push(format!(
                    "[{}] {} reward={:.3}",
                    domain.name(),
                    parts.join(" "),
                    reward
                ));
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
                let domain = match UnifiedKnowledgeSourceType::detect(url) {
                    UnifiedKnowledgeSourceType::GitHub => ExploreDomain::GitHub,
                    UnifiedKnowledgeSourceType::ArXiv => ExploreDomain::Papers,
                    UnifiedKnowledgeSourceType::Wikipedia => ExploreDomain::Wiki,
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
            details.push(format!(
                "[retry] re-queued {} failed URLs for retry",
                retried
            ));
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
}
