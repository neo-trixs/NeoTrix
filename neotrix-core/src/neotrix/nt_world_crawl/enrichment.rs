use super::classifier::ContentClassifier;
use super::config::CrawlStrategy;
use super::config::CrawlTopic;
use super::fetcher::FetcherPool;
use super::frontier::{extract_domain, extract_links, DualQueueFrontier, UrlEntry};
use super::mapper::KnowledgeMapper;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;
use crate::neotrix::nt_world_scrape::ScraperConfig;
use std::time::Instant;

// Default domain→crawl topic mapping for 23 capability dimensions
const FIELD_NAMES_DOMAIN: &[&str; 23] = &[
    "compound_composition",
    "tailwind",
    "accessibility",
    "react_aria",
    "ai_native_states",
    "semantic_layer",
    "verification",
    "quality_gates",
    "video_rendering",
    "html_composition",
    "secret_detection",
    "nt_shield_audit",
    "vulnerability_knowledge",
    "anti_detection",
    "web_scraping",
    "react_lint",
    "health_scoring",
    "vector_design_canvas",
    "mcp_design_tools",
    "agent_trading",
    "signal_sync",
    "esp32_firmware",
    "quadruped_kinematics",
];

fn dimension_to_crawl_topics(_dim: &str) -> Vec<CrawlTopic> {
    vec![
        CrawlTopic::ScienceAndTechnology,
        CrawlTopic::NewsAndMedia,
        CrawlTopic::General,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrichmentDepth {
    Shallow,
    Moderate,
    Deep,
}

#[derive(Debug, Clone)]
pub struct EnrichmentConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub batch_size: usize,
    pub weak_threshold: f64,
    pub depth: EnrichmentDepth,
    pub max_seeds_per_cycle: usize,
}

impl Default for EnrichmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300,
            batch_size: 3,
            weak_threshold: 0.3,
            depth: EnrichmentDepth::Shallow,
            max_seeds_per_cycle: 10,
        }
    }
}

pub struct BrainEnrichmentPipeline {
    config: EnrichmentConfig,
    cycle: u64,
    fetcher: FetcherPool,
    classifier: ContentClassifier,
    mapper: KnowledgeMapper,
    frontier: DualQueueFrontier,
}

impl BrainEnrichmentPipeline {
    pub fn new(config: EnrichmentConfig) -> Self {
        Self {
            config,
            cycle: 0,
            fetcher: FetcherPool::new(
                &ScraperConfig {
                    headless: true,
                    block_images: true,
                    timeout_secs: 15,
                    max_retries: 2,
                    use_tiny_profile: true,
                    ..ScraperConfig::default()
                },
                CrawlStrategy::Balanced,
            ),
            classifier: ContentClassifier::new(),
            mapper: KnowledgeMapper::new(),
            frontier: DualQueueFrontier::new(50),
        }
    }

    pub fn run_cycle(
        &mut self,
        brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
    ) -> EnrichmentReport {
        let start = Instant::now();
        self.cycle += 1;
        let gap_topics = self.analyze_gaps(brain);
        let gaps_found = gap_topics.len();
        let seeds = self.gaps_to_seeds(&gap_topics, brain);
        let seeds_generated = seeds.len();
        for seed in &seeds {
            let domain = extract_domain(&seed.url);
            self.frontier.push(UrlEntry {
                url: seed.url.clone(),
                domain,
                depth: 0,
                priority: 3,
                topic: Some(seed.topic.clone()),
            });
        }
        let batch_size = self.config.batch_size.min(seeds.len().max(1));
        let mut urls_crawled = 0usize;
        let mut urls_absorbed = 0usize;
        for _ in 0..batch_size {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let entry = match self.frontier.pop(now, 200) {
                Some(e) => e,
                None => break,
            };
            urls_crawled += 1;
            let result = self.fetcher.fetch(&entry.url);
            if result.error.is_some() || !result.is_success() {
                continue;
            }
            let text = match &result.text {
                Some(t) => t,
                None => continue,
            };
            let classified = self.classifier.classify(&entry.url, text);
            let mapped = self.mapper.map(&classified);
            self.mapper.apply_to_brain(&mapped, brain, bank);
            urls_absorbed += 1;
            if let Some(body) = &result.body {
                let new_links = extract_links(body, &result.url);
                for link in new_links {
                    let domain = extract_domain(&link);
                    self.frontier.push(UrlEntry {
                        url: link,
                        domain,
                        depth: 0,
                        priority: 1,
                        topic: None,
                    });
                }
            }
        }
        let duration_ms = start.elapsed().as_millis() as u64;
        let message = if urls_absorbed > 0 {
            format!(
                "enriched {} dimensions, absorbed {} URLs",
                gaps_found, urls_absorbed
            )
        } else if gaps_found > 0 {
            format!("{} gaps found but no URLs absorbed", gaps_found)
        } else {
            "no weak dimensions — brain well-enriched".into()
        };
        EnrichmentReport {
            cycle: self.cycle,
            gaps_found,
            seeds_generated,
            urls_crawled,
            urls_absorbed,
            duration_ms,
            message,
        }
    }

    fn analyze_gaps(&self, brain: &ReasoningBrain) -> Vec<GapTopic> {
        let mut gaps = Vec::new();
        let arr = brain.capability.arr();
        for (idx, &val) in arr.iter().enumerate() {
            if val < self.config.weak_threshold {
                let name = FIELD_NAMES_DOMAIN
                    .get(idx)
                    .copied()
                    .unwrap_or("general_knowledge");
                let domains = dimension_to_crawl_topics(name);
                for domain in domains {
                    gaps.push(GapTopic {
                        dim_index: idx,
                        dim_name: name.to_string(),
                        current_value: val,
                        topic: domain,
                        priority: 1.0 - val,
                    });
                }
            }
        }
        gaps.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        gaps.truncate(self.config.max_seeds_per_cycle);
        gaps
    }

    fn gaps_to_seeds(&self, gaps: &[GapTopic], _brain: &ReasoningBrain) -> Vec<GapSeed> {
        let mut seeds = Vec::new();
        for gap in gaps {
            let urls = match self.config.depth {
                EnrichmentDepth::Shallow => vec![format!(
                    "https://en.wikipedia.org/wiki/{}",
                    gap.dim_name.replace('_', "_")
                )],
                EnrichmentDepth::Moderate => vec![format!(
                    "https://news.ycombinator.com/item?id={}",
                    gap.dim_name
                )],
                EnrichmentDepth::Deep => {
                    vec![format!("https://arxiv.org/search/?query={}", gap.dim_name)]
                }
            };
            for url in urls {
                seeds.push(GapSeed {
                    url,
                    topic: gap.topic.clone(),
                    priority: gap.priority,
                    dim_name: gap.dim_name.clone(),
                });
            }
        }
        seeds
    }
}

#[derive(Debug, Clone)]
pub struct GapTopic {
    pub dim_index: usize,
    pub dim_name: String,
    pub current_value: f64,
    pub topic: CrawlTopic,
    pub priority: f64,
}

#[derive(Debug, Clone)]
pub struct GapSeed {
    pub url: String,
    pub topic: CrawlTopic,
    pub priority: f64,
    pub dim_name: String,
}

#[derive(Debug, Clone)]
pub struct EnrichmentReport {
    pub cycle: u64,
    pub gaps_found: usize,
    pub seeds_generated: usize,
    pub urls_crawled: usize,
    pub urls_absorbed: usize,
    pub duration_ms: u64,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;

    fn make_brain_with_weak_dim(dim: usize) -> ReasoningBrain {
        let mut brain = ReasoningBrain::default();
        brain.capability.arr_mut()[dim] = 0.1;
        brain
    }

    #[test]
    fn test_gap_detection_finds_weak_dimensions() {
        let brain = make_brain_with_weak_dim(0);
        let pipeline = BrainEnrichmentPipeline::new(EnrichmentConfig::default());
        let gaps = pipeline.analyze_gaps(&brain);
        assert!(!gaps.is_empty(), "should find at least one gap");
        assert!(gaps.iter().any(|g| g.current_value < 0.3));
    }

    #[test]
    fn test_no_gaps_when_all_strong() {
        let mut brain = ReasoningBrain::default();
        for v in brain.capability.arr_mut().iter_mut() {
            *v = 0.9;
        }
        let pipeline = BrainEnrichmentPipeline::new(EnrichmentConfig::default());
        let gaps = pipeline.analyze_gaps(&brain);
        assert!(gaps.is_empty(), "strong brain should have no gaps");
    }

    #[test]
    fn test_shallow_urls_generated() {
        let brain = make_brain_with_weak_dim(0);
        let pipeline = BrainEnrichmentPipeline::new(EnrichmentConfig {
            depth: EnrichmentDepth::Shallow,
            ..Default::default()
        });
        let gaps = pipeline.analyze_gaps(&brain);
        let seeds = pipeline.gaps_to_seeds(&gaps, &brain);
        assert!(!seeds.is_empty());
        assert!(seeds[0].url.starts_with("http"));
    }

    #[test]
    fn test_gap_priority_sorting() {
        let mut brain = ReasoningBrain::default();
        brain.capability.arr_mut()[0] = 0.1;
        brain.capability.arr_mut()[5] = 0.25;
        let pipeline = BrainEnrichmentPipeline::new(EnrichmentConfig::default());
        let gaps = pipeline.analyze_gaps(&brain);
        assert!(gaps.len() >= 1);
    }

    #[test]
    fn test_enrichment_config_defaults() {
        let cfg = EnrichmentConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.interval_secs, 300);
        assert!(cfg.batch_size > 0);
    }
}
