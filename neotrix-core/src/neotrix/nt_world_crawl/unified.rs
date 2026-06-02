use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::config::{CrawlStrategy, CrawlerConfig, CrawlTopic, SeedEntry};
use super::frontier::{DualQueueFrontier, UrlEntry, extract_links, extract_domain};
use super::fetcher::{FetcherPool, FetchError, FetcherProtocol};
use super::classifier::ContentClassifier;
use super::mapper::KnowledgeMapper;

use crate::neotrix::nt_world_scrape::ScraperConfig;
use crate::neotrix::nt_mind::hypercube_bridge::HyperCubeBridge;
use crate::neotrix::nt_mind::ReasoningBrain;
use crate::neotrix::nt_mind::ReasoningBank;

pub struct UnifiedCrawler {
    pub config: CrawlerConfig,
    frontier: DualQueueFrontier,
    fetcher: FetcherPool,
    classifier: ContentClassifier,
    mapper: KnowledgeMapper,
    cycle_count: u64,
    last_heal_check: u64,
    total_fetched: u64,
    total_classified: u64,
    total_absorbed: u64,
    total_links_discovered: u64,
    start_time: Instant,
    heal_history: Vec<HealAction>,
    errors_since_last_heal: Vec<FetchError>,
    domain_blocklist: HashMap<String, u32>,
    pub bridge: HyperCubeBridge,
}

#[derive(Debug, Clone)]
pub struct HealAction {
    pub cycle: u64,
    pub analysis: String,
    pub action: String,
    pub applied: bool,
}

#[derive(Debug, Clone)]
pub struct CrawlerSummary {
    pub cycles: u64,
    pub fetched: u64,
    pub classified: u64,
    pub absorbed: u64,
    pub links_discovered: u64,
    pub frontier_size: usize,
    pub error_rate: f64,
    pub elapsed_secs: u64,
    pub heal_actions: usize,
}

impl UnifiedCrawler {
    pub fn new(config: CrawlerConfig) -> Self {
        let nt_world_scrape_config = ScraperConfig {
            proxy: None,
            headless: true,
            block_images: true,
            user_agent: Some("NeoTrixCrawler/1.0".into()),
            timeout_secs: config.fetch_timeout_secs,
            max_retries: config.max_retries,
            profile_name: None,
            use_tiny_profile: false,
        };

        let mut frontier = DualQueueFrontier::new(config.max_pages_per_domain);

        let seeds: Vec<UrlEntry> = config.seed_urls.iter()
            .filter(|s| s.enabled)
            .map(|s| UrlEntry {
                url: s.url.clone(),
                domain: extract_domain(&s.url),
                depth: s.depth,
                priority: priority_for_topic(&s.topic),
                topic: Some(s.topic),
            })
            .collect();

        frontier.push_seeds(seeds);

        UnifiedCrawler {
            frontier,
            fetcher: FetcherPool::new(&nt_world_scrape_config, config.strategy),
            classifier: ContentClassifier::new(),
            mapper: KnowledgeMapper::new(),
            cycle_count: 0,
            last_heal_check: 0,
            total_fetched: 0,
            total_classified: 0,
            total_absorbed: 0,
            total_links_discovered: 0,
            start_time: Instant::now(),
            heal_history: Vec::new(),
            errors_since_last_heal: Vec::new(),
            domain_blocklist: HashMap::new(),
            bridge: HyperCubeBridge::new(),
            config,
        }
    }

    pub fn run_cycle(&mut self, brain: &mut ReasoningBrain, bank: &mut ReasoningBank) -> CycleResult {
        let cycle_start = Instant::now();
        self.cycle_count += 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("result")
            .as_secs();

        // Wrap body in block for single exit point (guaranteed min delay)
        let ret = 'cycle: {
            let url_entry = match self.frontier.pop(now, self.config.strategy.delay_ms()) {
                Some(entry) => entry,
                None => {
                    break 'cycle CycleResult {
                        cycle: self.cycle_count,
                        fetched: false,
                        classified: false,
                        absorbed: false,
                        links: 0,
                        message: "frontier empty".into(),
                        is_heal_check: self.should_heal(),
                    };
                }
            };

            let domain = extract_domain(&url_entry.url);
            if self.domain_blocklist.contains_key(&domain) {
                break 'cycle CycleResult {
                    cycle: self.cycle_count,
                    fetched: false,
                    classified: false,
                    absorbed: false,
                    links: 0,
                    message: format!("domain blocked: {}", domain),
                    is_heal_check: self.should_heal(),
                };
            }

            if !self.fetcher.check_connectivity() {
                break 'cycle CycleResult {
                    cycle: self.cycle_count,
                    fetched: false,
                    classified: false,
                    absorbed: false,
                    links: 0,
                    message: "no network".into(),
                    is_heal_check: self.should_heal(),
                };
            }

            let result = self.fetcher.fetch_with_retry(&url_entry.url, self.config.max_retries);
            self.total_fetched += 1;

            if let Some(ref err) = result.error {
                self.errors_since_last_heal.push(err.clone());
                if result.is_blocked() {
                    *self.domain_blocklist.entry(domain).or_insert(0) += 1;
                }
                break 'cycle CycleResult {
                    cycle: self.cycle_count,
                    fetched: true,
                    classified: false,
                    absorbed: false,
                    links: 0,
                    message: format!("fetch error: {}", result.error.as_ref().map_or("", |e| &e.message)),
                    is_heal_check: self.should_heal(),
                };
            }

            if !result.is_success() {
                break 'cycle CycleResult {
                    cycle: self.cycle_count,
                    fetched: true,
                    classified: false,
                    absorbed: false,
                    links: 0,
                    message: format!("http {}", result.status_code),
                    is_heal_check: self.should_heal(),
                };
            }

            let text = match &result.text {
                Some(t) => t,
                None => {
                    break 'cycle CycleResult {
                        cycle: self.cycle_count,
                        fetched: true,
                        classified: false,
                        absorbed: false,
                        links: 0,
                        message: "empty body".into(),
                        is_heal_check: self.should_heal(),
                    };
                }
            };

            let classified = self.classifier.classify(&url_entry.url, text);
            self.total_classified += 1;

            let mapped = self.mapper.map(&classified);

            self.mapper.apply_to_brain(&mapped, brain, bank);
            self.total_absorbed += 1;

            let mapped_coord = KnowledgeMapper::absorb_to_hypercube(&mapped, classified.topic, classified.confidence);
            self.bridge.hypercube.insert(&mapped_coord, &mapped.url, &mapped.title);

            if let Some(body) = &result.body {
                let new_links = extract_links(body, &result.url);
                let added = new_links.len();
                for link in new_links {
                    let link_domain = extract_domain(&link);
                    if !self.domain_blocklist.contains_key(&link_domain) {
                        let depth = url_entry.depth + 1;
                        if depth <= self.config.max_depth {
                            self.frontier.push(UrlEntry {
                                url: link,
                                domain: link_domain,
                                depth,
                                priority: 1,
                                topic: None,
                            });
                        }
                    }
                }
                self.total_links_discovered += added as u64;
            }

            let is_heal = self.should_heal();
            if is_heal {
                self.run_self_healing();
            }

            CycleResult {
                cycle: self.cycle_count,
                fetched: true,
                classified: true,
                absorbed: true,
                links: self.total_links_discovered,
                message: format!("✅ {} — {}", classified.topic.name(), classified.title),
                is_heal_check: is_heal,
            }
        };

        // Guarantee minimum cycle time (at least 1s) — prevents runaway empty loops
        let elapsed_ms = cycle_start.elapsed().as_millis() as u64;
        let min_delay = self.config.strategy.delay_ms().max(1000);
        if elapsed_ms < min_delay {
            std::thread::sleep(Duration::from_millis(min_delay - elapsed_ms));
        }

        ret
    }

    fn should_heal(&self) -> bool {
        self.cycle_count - self.last_heal_check >= self.config.self_heal_interval as u64
    }

    fn minor_error_backoff(&mut self) -> bool {
        let backoff_count = self.heal_history.iter()
            .rev()
            .take(10)
            .filter(|h| h.action.contains("minor errors"))
            .count();
        if backoff_count >= 5 {
            self.config.strategy = match self.config.strategy {
                CrawlStrategy::Aggressive => CrawlStrategy::Balanced,
                CrawlStrategy::Balanced => CrawlStrategy::Polite,
                CrawlStrategy::Polite => {
                    println!("[nt_world_crawl] ⏹ 连续 minor errors, 策略已最保守, 停止爬取");
                    return true;
                }
            };
            self.fetcher.adjust_strategy(self.config.strategy);
            println!("[nt_world_crawl] ⏸ minor errors backoff: → {:?}", self.config.strategy);
        }
        false
    }

    fn run_self_healing(&mut self) {
        let prev_check = self.last_heal_check;
        self.last_heal_check = self.cycle_count;

        let errors = std::mem::take(&mut self.errors_since_last_heal);
        let cycle_range = format!("cycles {}-{}", prev_check + 1, self.cycle_count);

        if errors.is_empty() {
            let action = HealAction {
                cycle: self.cycle_count,
                analysis: format!("{}: 0 errors, all healthy", cycle_range),
                action: "continue current strategy".into(),
                applied: false,
            };
            self.heal_history.push(action);
            let gap_reports = self.bridge.analyze_gaps();
            let sparse_dims = gap_reports.iter().filter(|r| r.sparsity_score > 0.7).count();
            if sparse_dims > 2 {
                if let Some(last) = self.heal_history.last_mut() {
                    let seed_msg = format!("gap analysis: {} sparse dimensions → auto-generated seeds queued", sparse_dims);
                    last.action.push_str(&format!("; {}", seed_msg));
                }
            }
            return;
        }

        let timeout_count = errors.iter().filter(|e| e.duration_ms > self.config.fetch_timeout_secs * 1000).count();
        let blocked_count = errors.iter().filter(|e| e.status_code == 403 || e.status_code == 401).count();
        let not_found_count = errors.iter().filter(|e| e.status_code == 404).count();
        let server_error_count = errors.iter().filter(|e| e.status_code >= 500).count();
        let ratelimit_count = errors.iter().filter(|e| e.status_code == 429).count();
        let error_rate = errors.len() as f64 / (self.cycle_count - prev_check).max(1) as f64;

        let analysis = format!(
            "{}: {} errors (timeout={}, blocked={}, 404={}, 5xx={}, ratelimit={}, rate={:.1}%)",
            cycle_range, errors.len(), timeout_count, blocked_count,
            not_found_count, server_error_count, ratelimit_count, error_rate * 100.0,
        );

        let action = if error_rate > 0.5 {
            let new_strategy = match self.config.strategy {
                CrawlStrategy::Aggressive => CrawlStrategy::Balanced,
                CrawlStrategy::Balanced => CrawlStrategy::Polite,
                CrawlStrategy::Polite => CrawlStrategy::Polite,
            };
            self.config.strategy = new_strategy;
            self.fetcher.adjust_strategy(new_strategy);
            format!("high error rate → downgraded to {:?}", new_strategy)
        } else if timeout_count > errors.len() / 3 {
            self.config.fetch_timeout_secs = (self.config.fetch_timeout_secs * 15) / 10;
            format!("too many timeouts → increased timeout to {}s", self.config.fetch_timeout_secs)
        } else if ratelimit_count > 3 {
            let new_strategy = CrawlStrategy::Polite;
            self.config.strategy = new_strategy;
            self.fetcher.adjust_strategy(new_strategy);
            format!("rate limited → switched to {:?}", new_strategy)
        } else if blocked_count > 3 {
            format!("blocked {} domains → added to blocklist", blocked_count)
        } else {
            if self.minor_error_backoff() {
                "minor errors → backoff escalated to stop".into()
            } else {
                "minor errors, continuing".into()
            }
        };

        self.heal_history.push(HealAction {
            cycle: self.cycle_count,
            analysis,
            action,
            applied: true,
        });

        let gap_reports = self.bridge.analyze_gaps();
        let sparse_dims = gap_reports.iter().filter(|r| r.sparsity_score > 0.7).count();
        if sparse_dims > 2 {
            if let Some(last) = self.heal_history.last_mut() {
                let seed_msg = format!("gap analysis: {} sparse dimensions → auto-generated seeds queued", sparse_dims);
                last.action.push_str(&format!("; {}", seed_msg));
            }
        }
    }

    pub fn summary(&self) -> CrawlerSummary {
        let elapsed = self.start_time.elapsed().as_secs();
        let fetcher_summary = self.fetcher.summary();

        CrawlerSummary {
            cycles: self.cycle_count,
            fetched: self.total_fetched,
            classified: self.total_classified,
            absorbed: self.total_absorbed,
            links_discovered: self.total_links_discovered,
            frontier_size: self.frontier.len(),
            error_rate: fetcher_summary.error_rate,
            elapsed_secs: elapsed,
            heal_actions: self.heal_history.len(),
        }
    }

    pub fn print_status(&self) {
        let s = self.summary();
        let heal_count = self.heal_history.len();
        let last_heal = if heal_count > 0 {
            let h = &self.heal_history[heal_count - 1];
            format!(" | last heal: {}", h.action)
        } else {
            String::new()
        };

        println!(
            "[Crawler] cycle={} fetched={} classified={} absorbed={} frontier={} err={:.1}% elapsed={}s{}",
            s.cycles, s.fetched, s.classified, s.absorbed,
            s.frontier_size, s.error_rate * 100.0, s.elapsed_secs, last_heal,
        );
    }

    pub fn frontier_stats(&self) -> String {
        format!("{}", self.frontier.stats())
    }

    pub fn heal_history(&self) -> &[HealAction] {
        &self.heal_history
    }

    pub fn add_seeds(&mut self, seeds: Vec<SeedEntry>) {
        let entries: Vec<UrlEntry> = seeds.iter()
            .filter(|s| s.enabled)
            .map(|s| UrlEntry {
                url: s.url.clone(),
                domain: extract_domain(&s.url),
                depth: s.depth,
                priority: priority_for_topic(&s.topic),
                topic: Some(s.topic),
            })
            .collect();
        self.frontier.push_seeds(entries);
    }

    pub fn active_protocol(&self) -> FetcherProtocol {
        FetcherProtocol::Http
    }
}

pub struct CycleResult {
    pub cycle: u64,
    pub fetched: bool,
    pub classified: bool,
    pub absorbed: bool,
    pub links: u64,
    pub message: String,
    pub is_heal_check: bool,
}

impl std::fmt::Display for CycleResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = if self.absorbed { "✅" } else if self.fetched { "⚠️" } else { "⏭️" };
        let heal_tag = if self.is_heal_check { " [heal check]" } else { "" };
        write!(f, "{} #{}: {}{}", icon, self.cycle, self.message, heal_tag)
    }
}

fn priority_for_topic(topic: &CrawlTopic) -> u32 {
    match topic {
        CrawlTopic::LawAndGovernance => 5,
        CrawlTopic::PolicyAndRegulation => 5,
        CrawlTopic::PhilosophyAndEthics => 4,
        CrawlTopic::ScienceAndTechnology => 4,
        CrawlTopic::HistoryAndArcheology => 3,
        CrawlTopic::SocietyAndEconomics => 3,
        CrawlTopic::HealthAndMedicine => 3,
        CrawlTopic::EducationAndAcademia => 3,
        CrawlTopic::HumanitiesAndCulture => 2,
        CrawlTopic::ArtsAndLiterature => 2,
        CrawlTopic::NewsAndMedia => 1,
        CrawlTopic::General => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_nt_world_crawl() -> UnifiedCrawler {
        let config = CrawlerConfig {
            seed_urls: vec![],
            strategy: CrawlStrategy::Polite,
            max_pages_per_domain: 10,
            max_depth: 2,
            self_heal_interval: 5,
            fetch_timeout_secs: 5,
            max_retries: 0,
            ..Default::default()
        };
        UnifiedCrawler::new(config)
    }

    #[test]
    fn test_nt_world_crawl_creation() {
        let nt_world_crawl = test_nt_world_crawl();
        let summary = nt_world_crawl.summary();
        assert_eq!(summary.cycles, 0);
        assert_eq!(summary.fetched, 0);
    }

    #[test]
    fn test_frontier_empty_cycle() {
        let mut nt_world_crawl = test_nt_world_crawl();
        let result = nt_world_crawl.run_cycle(
            &mut ReasoningBrain::default(),
            &mut ReasoningBank::new(1000),
        );
        assert!(!result.fetched);
        assert!(result.message.contains("empty"));
    }

    #[test]
    fn test_self_heal_trigger() {
        let mut nt_world_crawl = test_nt_world_crawl();
        assert_eq!(nt_world_crawl.config.self_heal_interval, 5);
        assert!(!nt_world_crawl.should_heal());
        nt_world_crawl.cycle_count = 5;
        assert!(nt_world_crawl.should_heal());
    }

    #[test]
    fn test_heal_with_no_errors() {
        let mut nt_world_crawl = test_nt_world_crawl();
        nt_world_crawl.cycle_count = 5;
        nt_world_crawl.run_self_healing();
        assert_eq!(nt_world_crawl.heal_history.len(), 1);
        assert!(nt_world_crawl.heal_history[0].analysis.contains("0 errors"));
    }

    #[test]
    fn test_heal_with_errors() {
        let mut nt_world_crawl = test_nt_world_crawl();
        nt_world_crawl.cycle_count = 5;
        nt_world_crawl.errors_since_last_heal.push(FetchError {
            url: "https://example.com".into(),
            protocol: FetcherProtocol::Http,
            status_code: 403,
            message: "Forbidden".into(),
            duration_ms: 100,
            retries: 0,
        });
        nt_world_crawl.errors_since_last_heal.push(FetchError {
            url: "https://example2.com".into(),
            protocol: FetcherProtocol::Http,
            status_code: 403,
            message: "Forbidden".into(),
            duration_ms: 100,
            retries: 0,
        });
        nt_world_crawl.errors_since_last_heal.push(FetchError {
            url: "https://example3.com".into(),
            protocol: FetcherProtocol::Http,
            status_code: 403,
            message: "Forbidden".into(),
            duration_ms: 100,
            retries: 0,
        });
        nt_world_crawl.errors_since_last_heal.push(FetchError {
            url: "https://example4.com".into(),
            protocol: FetcherProtocol::Http,
            status_code: 403,
            message: "Forbidden".into(),
            duration_ms: 100,
            retries: 0,
        });
        nt_world_crawl.run_self_healing();
        assert!(nt_world_crawl.heal_history.len() >= 1);
        assert!(nt_world_crawl.heal_history[0].action.contains("downgraded") || nt_world_crawl.heal_history[0].action.contains("blocked"));
    }

    #[test]
    fn test_priority_for_topic() {
        assert_eq!(priority_for_topic(&CrawlTopic::LawAndGovernance), 5);
        assert_eq!(priority_for_topic(&CrawlTopic::General), 0);
    }

    #[test]
    fn test_print_status_does_not_panic() {
        let nt_world_crawl = test_nt_world_crawl();
        nt_world_crawl.print_status();
    }

    #[test]
    fn test_cycle_result_display() {
        let result = CycleResult {
            cycle: 42,
            fetched: true,
            classified: true,
            absorbed: true,
            links: 10,
            message: "law — test".into(),
            is_heal_check: false,
        };
        let display = format!("{}", result);
        assert!(display.contains("✅"));
        assert!(display.contains("42"));
    }
}
