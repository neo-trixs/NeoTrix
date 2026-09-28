#![forbid(unsafe_code)]

//! MemoryCascade — orchestrator for the five-tier cascade pipeline.
//!
//! Flow: observe → attend → promote → distill
//!
//! 1. **SensoryBuffer** receives raw observations
//! 2. **WorkingMemory** gates items via GWT attention threshold
//! 3. **ShortTermStore** holds memories that survive working memory
//! 4. **EpisodicStore** compresses short-term memories into summaries
//! 5. **LongTermStore** distills rules from episodes with confidence tracking

use super::episodic::EpisodicStore;
use super::long_term::LongTermStore;
use super::sensory::SensoryBuffer;
use super::short_term::ShortTermStore;
use super::working::WorkingMemory;

/// Configuration for the full cascade pipeline.
#[derive(Debug, Clone)]
pub struct CascadeConfig {
    pub sensory_ttl_secs: u64,
    pub sensory_max: usize,
    pub working_max: usize,
    pub attention_threshold: f64,
    pub short_term_ttl_secs: u64,
    pub short_term_max: usize,
    pub episodic_max: usize,
    pub long_term_max: usize,
    pub confidence_decay_rate: f64,
    pub distill_threshold: f64,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            sensory_ttl_secs: 60,
            sensory_max: 100,
            working_max: 7,
            attention_threshold: 0.3,
            short_term_ttl_secs: 7 * 24 * 3600,
            short_term_max: 1000,
            episodic_max: 500,
            long_term_max: 10_000,
            confidence_decay_rate: 0.01,
            distill_threshold: 0.5,
        }
    }
}

/// Statistics for a single cascade tick.
#[derive(Debug, Clone, Default)]
pub struct CascadeTickStats {
    pub observations_ingested: usize,
    pub items_admitted_to_working: usize,
    pub items_promoted_to_short_term: usize,
    pub episodes_compressed: usize,
    pub rules_distilled: usize,
}

/// MemoryCascade: the five-tier orchestrator.
pub struct MemoryCascade {
    pub sensory: SensoryBuffer,
    pub working: WorkingMemory,
    pub short_term: ShortTermStore,
    pub episodic: EpisodicStore,
    pub long_term: LongTermStore,
    config: CascadeConfig,
}

impl MemoryCascade {
    pub fn new(config: CascadeConfig) -> Self {
        use std::time::Duration;

        Self {
            sensory: SensoryBuffer::new(
                Duration::from_secs(config.sensory_ttl_secs),
                config.sensory_max,
            ),
            working: WorkingMemory::new(config.working_max, config.attention_threshold),
            short_term: ShortTermStore::new(
                Duration::from_secs(config.short_term_ttl_secs),
                config.short_term_max,
            ),
            episodic: EpisodicStore::new(config.episodic_max),
            long_term: LongTermStore::new(
                config.long_term_max,
                config.confidence_decay_rate,
                Duration::from_secs(3600),
            ),
            config,
        }
    }

    pub fn default_pipeline() -> Self {
        Self::new(CascadeConfig::default())
    }

    /// Step 1: Ingest a raw observation into the sensory buffer.
    pub fn observe(&mut self, content: String, source: String) {
        self.sensory.observe(content, source);
    }

    /// Run the full cascade: attend → promote → distill.
    /// Returns statistics for this tick.
    pub fn tick(&mut self) -> CascadeTickStats {
        let mut stats = CascadeTickStats::default();

        // Drain sensory buffer into working memory
        let observations = self.sensory.drain_attended();
        stats.observations_ingested = observations.len();

        let mut surviving_ids: Vec<u64> = Vec::new();

        for obs in observations {
            let attention = self.compute_attention(&obs.content);
            let admitted = self
                .working
                .attend(obs.content.clone(), attention, obs.source.clone());
            if admitted {
                stats.items_admitted_to_working += 1;
            }
        }

        // Promote working memory items that cleared the attention gate to short-term
        let working_items = self.working.drain_all();
        for item in working_items {
            if item.attention_weight >= self.config.attention_threshold {
                let st_id = self.short_term.store(item.content, item.attention_weight);
                surviving_ids.push(st_id);
                stats.items_promoted_to_short_term += 1;
            }
        }

        // Compress short-term into episodic summaries
        let candidates = self
            .short_term
            .promote_candidates(self.config.distill_threshold);
        if candidates.len() >= 3 {
            let summary = self.compress_to_summary(&candidates);
            let source_ids: Vec<u64> = candidates.iter().map(|c| c.id).collect();
            let avg_importance: f64 = candidates.iter().map(|c| c.attention_weight).sum::<f64>()
                / candidates.len() as f64;

            self.episodic.store(
                summary,
                source_ids,
                avg_importance,
                vec!["auto_compressed".into()],
            );
            stats.episodes_compressed += 1;
        }

        // Distill high-importance episodes into rules
        let episodes = self.episodic.all_by_importance();
        for ep in episodes.iter().take(5) {
            if ep.importance >= self.config.distill_threshold && ep.summary.len() > 10 {
                self.long_term
                    .distill(ep.summary.clone(), ep.importance, ep.source_ids.clone());
                stats.rules_distilled += 1;
            }
        }

        stats
    }

    /// Recall across all tiers, returning matched content from each level.
    pub fn recall_all(&self, query: &str) -> CascadeRecallResult {
        let sensory_matches: Vec<String> = self
            .sensory
            .peek_latest()
            .map(|o| vec![o.content.clone()])
            .unwrap_or_default();

        let working_matches: Vec<String> = self
            .working
            .recall_all()
            .iter()
            .filter(|i| i.content.contains(query))
            .map(|i| i.content.clone())
            .collect();

        let short_term_matches: Vec<String> = self
            .short_term
            .all_by_recency()
            .iter()
            .filter(|e| e.content.contains(query))
            .map(|e| e.content.clone())
            .collect();

        let episodic_matches: Vec<String> = self
            .episodic
            .search_by_content(query)
            .iter()
            .map(|e| e.summary.clone())
            .collect();

        let long_term_matches: Vec<String> = self
            .long_term
            .search(query)
            .iter()
            .map(|r| r.rule.clone())
            .collect();

        CascadeRecallResult {
            sensory: sensory_matches,
            working: working_matches,
            short_term: short_term_matches,
            episodic: episodic_matches,
            long_term: long_term_matches,
        }
    }

    fn compute_attention(&self, content: &str) -> f64 {
        // Heuristic: longer content with keywords gets higher attention
        let length_score = (content.len() as f64 / 200.0).min(1.0);
        let keyword_bonus = if content.contains("error")
            || content.contains("critical")
            || content.contains("important")
        {
            0.3
        } else {
            0.0
        };
        (length_score + keyword_bonus).min(1.0)
    }

    fn compress_to_summary(&self, items: &[&super::short_term::ShortTermEntry]) -> String {
        let contents: Vec<&str> = items.iter().map(|i| i.content.as_str()).collect();
        format!(
            "Compressed episode: {} items — {}",
            contents.len(),
            contents.join("; ")
        )
    }
}

/// Result of a cross-tier recall query.
#[derive(Debug, Clone, Default)]
pub struct CascadeRecallResult {
    pub sensory: Vec<String>,
    pub working: Vec<String>,
    pub short_term: Vec<String>,
    pub episodic: Vec<String>,
    pub long_term: Vec<String>,
}

impl CascadeRecallResult {
    pub fn total_matches(&self) -> usize {
        self.sensory.len()
            + self.working.len()
            + self.short_term.len()
            + self.episodic.len()
            + self.long_term.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipeline_tick() {
        let mut cascade = MemoryCascade::default_pipeline();
        cascade.observe("important task: deploy the service".into(), "user".into());
        cascade.observe("error: connection refused".into(), "system".into());

        let stats = cascade.tick();
        assert_eq!(stats.observations_ingested, 2);
    }

    #[test]
    fn recall_finds_across_tiers() {
        let mut cascade = MemoryCascade::default_pipeline();
        cascade.observe("critical: fix the auth bug".into(), "user".into());
        cascade.tick();

        let result = cascade.recall_all("auth");
        assert!(result.total_matches() >= 1);
    }

    #[test]
    fn default_config_is_sane() {
        let cfg = CascadeConfig::default();
        assert_eq!(cfg.sensory_max, 100);
        assert_eq!(cfg.working_max, 7);
        assert_eq!(cfg.episodic_max, 500);
    }
}
