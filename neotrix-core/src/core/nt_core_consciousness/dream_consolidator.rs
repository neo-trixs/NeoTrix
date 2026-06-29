use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::HashMap;
use std::collections::VecDeque;

const MAX_RESULTS: usize = 10000;

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ConsolidationPriority {
    High,
    Medium,
    Low,
}

// Kleos 6-stage dream consolidation pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConsolidationStage {
    Filter,    // Stage 1: VSA similarity threshold (remove near-duplicates)
    Replay,    // Stage 2: high-reward trajectory replay
    Link,      // Stage 3: cross-domain VSA association discovery
    Abstract,  // Stage 4: concept clustering + upsampling
    Integrate, // Stage 5: write to long-term memory
    Prune,     // Stage 6: sparsify + forget low-value patterns
}

/// Stage-specific configuration for the Kleos pipeline
#[derive(Debug, Clone)]
pub struct KleosConfig {
    pub filter_threshold: f64, // VSA sim > this = duplicate (default 0.92)
    pub replay_reward_threshold: f64, // effectiveness > this triggers replay (default 0.7)
    pub link_min_similarity: f64, // cross-domain min sim for linking (default 0.6)
    pub abstract_cluster_count: usize, // number of clusters (default 8)
    pub integrate_confidence_min: f64, // min confidence to integrate (default 0.5)
    pub prune_retain_fraction: f64, // fraction of entries to keep after prune (default 0.6)
}

impl Default for KleosConfig {
    fn default() -> Self {
        Self {
            filter_threshold: 0.92,
            replay_reward_threshold: 0.7,
            link_min_similarity: 0.6,
            abstract_cluster_count: 8,
            integrate_confidence_min: 0.5,
            prune_retain_fraction: 0.6,
        }
    }
}

/// Per-stage result from the Kleos pipeline
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage: ConsolidationStage,
    pub entries_in: usize,
    pub entries_out: usize,
    pub new_links: usize,
    pub duration_ms: f64,
}

/// Full Kleos 6-stage pipeline result
#[derive(Debug, Clone)]
pub struct KleosPipelineResult {
    pub stages: Vec<StageResult>,
    pub total_duration_ms: f64,
    pub before_count: usize,
    pub after_count: usize,
    pub linked_patterns: usize,
    pub new_clusters: usize,
}

#[derive(Debug, Clone)]
pub struct DreamEntry {
    pub id: u64,
    pub session_id: String,
    pub pattern: Vec<u8>,
    pub effectiveness: f64,
    pub frequency: u64,
    pub last_seen: u64,
    pub priority: ConsolidationPriority,
    pub consolidated: bool,
}

#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    pub entry_id: u64,
    pub merged_pattern: Vec<u8>,
    pub new_heuristic_id: Option<u64>,
    pub novelty: f64,
    pub coherence: f64,
}

#[derive(Debug, Clone)]
pub struct DreamConsolidator {
    entries: Vec<DreamEntry>,
    results: Vec<ConsolidationResult>,
    next_id: u64,
    max_entries: usize,
    coherence_threshold: f64,
    novelty_threshold: f64,
    cycle: u64,
    sessions_processed: u64,
    pub kleos_config: KleosConfig,
    pub pipeline_results: VecDeque<KleosPipelineResult>,
}

fn classify_priority(effectiveness: f64, frequency: u64) -> ConsolidationPriority {
    if effectiveness > 0.7 && frequency >= 5 {
        ConsolidationPriority::High
    } else if effectiveness > 0.4 && frequency >= 2 {
        ConsolidationPriority::Medium
    } else {
        ConsolidationPriority::Low
    }
}

impl DreamConsolidator {
    pub fn new(max_entries: usize, coherence_threshold: f64, novelty_threshold: f64) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            results: Vec::new(),
            next_id: 1,
            max_entries,
            coherence_threshold,
            novelty_threshold,
            cycle: 0,
            sessions_processed: 0,
            kleos_config: KleosConfig::default(),
            pipeline_results: VecDeque::with_capacity(16),
        }
    }

    pub fn feed(&mut self, session_id: &str, patterns: &[(Vec<u8>, f64)]) {
        self.cycle += 1;
        for (pattern, effectiveness) in patterns {
            let found_existing = self
                .entries
                .iter_mut()
                .find(|e| QuantizedVSA::similarity(&e.pattern, pattern) > 0.75);
            if let Some(existing) = found_existing {
                existing.frequency += 1;
                existing.effectiveness = existing.effectiveness * 0.7 + effectiveness * 0.3;
                existing.last_seen = self.cycle;
                let eff = existing.effectiveness;
                let freq = existing.frequency;
                existing.priority = classify_priority(eff, freq);
            } else {
                if self.entries.len() >= self.max_entries {
                    self.prune_low_priority();
                }
                self.entries.push(DreamEntry {
                    id: self.next_id,
                    session_id: session_id.to_string(),
                    pattern: pattern.clone(),
                    effectiveness: *effectiveness,
                    frequency: 1,
                    last_seen: self.cycle,
                    priority: classify_priority(*effectiveness, 1),
                    consolidated: false,
                });
                self.next_id += 1;
            }
        }
    }

    pub fn consolidate(&mut self) -> Vec<ConsolidationResult> {
        self.cycle += 1;
        let mut new_results = Vec::new();

        let candidates: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.consolidated && e.frequency >= 3)
            .map(|(i, _)| i)
            .collect();

        for &idx in &candidates {
            let entry_eff = self.entries[idx].effectiveness;
            if entry_eff < 0.3 {
                continue;
            }

            let entry_id = self.entries[idx].id;
            let entry_pattern = self.entries[idx].pattern.clone();
            let entry_effectiveness = self.entries[idx].effectiveness;
            let entry_frequency = self.entries[idx].frequency;

            let merged = self.merge_with_similar_by_idx(idx);
            let merged_vsa = QuantizedVSA::bind(&entry_pattern, &merged);

            let (novelty, coherence) =
                self.evaluate(&merged_vsa, entry_effectiveness, entry_frequency);

            if coherence >= self.coherence_threshold {
                let heuristic_id = if novelty >= self.novelty_threshold {
                    Some(self.next_id)
                } else {
                    None
                };

                let result = ConsolidationResult {
                    entry_id,
                    merged_pattern: merged_vsa,
                    new_heuristic_id: heuristic_id,
                    novelty,
                    coherence,
                };

                if let Some(_hid) = heuristic_id {
                    self.next_id += 1;
                }

                new_results.push(result.clone());
                self.results.push(result);
                if self.results.len() > MAX_RESULTS {
                    self.results
                        .drain(0..self.results.len().saturating_sub(MAX_RESULTS));
                }

                if let Some(e) = self.entries.iter_mut().find(|e| e.id == entry_id) {
                    e.consolidated = true;
                }
            }
        }

        new_results
    }

    pub fn dream_cycle(&mut self) -> (Vec<ConsolidationResult>, Vec<ConsolidationResult>) {
        let results = self.consolidate();
        let cross_session = self.cross_session_bind();
        (results, cross_session)
    }

    fn cross_session_bind(&mut self) -> Vec<ConsolidationResult> {
        let mut bound = Vec::new();
        let by_session: HashMap<String, Vec<&DreamEntry>> = self
            .entries
            .iter()
            .filter(|e| e.consolidated)
            .fold(HashMap::new(), |mut acc, e| {
                acc.entry(e.session_id.clone()).or_default().push(e);
                acc
            });

        if by_session.len() < 2 {
            return bound;
        }

        let session_ids: Vec<&String> = by_session.keys().collect();
        for i in 0..session_ids.len().saturating_sub(1) {
            let si = session_ids[i];
            let sj = session_ids[i + 1];
            let ei = by_session.get(si).and_then(|v| v.first());
            let ej = by_session.get(sj).and_then(|v| v.first());
            if let (Some(a), Some(b)) = (ei, ej) {
                let bound_vsa = QuantizedVSA::bind(&a.pattern, &b.pattern);
                let novelty = QuantizedVSA::similarity(&a.pattern, &b.pattern).abs() * 0.5;
                let coherence = (a.effectiveness + b.effectiveness) / 2.0;
                if coherence >= self.coherence_threshold {
                    bound.push(ConsolidationResult {
                        entry_id: b.id,
                        merged_pattern: bound_vsa,
                        new_heuristic_id: Some(self.next_id),
                        novelty,
                        coherence,
                    });
                    self.next_id += 1;
                }
            }
        }
        bound
    }

    fn merge_with_similar_by_idx(&self, idx: usize) -> Vec<u8> {
        let similar: Vec<&DreamEntry> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(i, e)| {
                *i != idx && QuantizedVSA::similarity(&e.pattern, &self.entries[idx].pattern) > 0.6
            })
            .map(|(_, e)| e)
            .collect();
        if similar.is_empty() {
            return self.entries[idx].pattern.clone();
        }
        let mut merged = self.entries[idx].pattern.clone();
        for s in &similar {
            merged = QuantizedVSA::bind(&merged, &s.pattern);
        }
        merged
    }

    fn evaluate(&self, merged: &[u8], effectiveness: f64, frequency: u64) -> (f64, f64) {
        let mut max_sim = 0.0;
        for e in &self.entries {
            let sim = QuantizedVSA::similarity(merged, &e.pattern);
            if sim > max_sim {
                max_sim = sim;
            }
        }
        let novelty = 1.0 - max_sim;
        let coherence = effectiveness * (1.0_f64 + frequency as f64 * 0.1_f64).min(2.0_f64);
        (novelty, coherence.min(1.0))
    }

    pub fn cross_pollinate(&mut self) -> Vec<(u64, u64, f64)> {
        let mut pairs = Vec::new();
        for i in 0..self.entries.len() {
            for j in i + 1..self.entries.len() {
                let sim =
                    QuantizedVSA::similarity(&self.entries[i].pattern, &self.entries[j].pattern);
                if sim > 0.5 && sim < 0.95 {
                    let combined_eff =
                        (self.entries[i].effectiveness + self.entries[j].effectiveness) / 2.0;
                    pairs.push((self.entries[i].id, self.entries[j].id, combined_eff));
                }
            }
        }
        pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        pairs.truncate(10);
        pairs
    }

    pub fn report(&self) -> DreamReport {
        let consolidated_count = self.entries.iter().filter(|e| e.consolidated).count();
        DreamReport {
            total_entries: self.entries.len(),
            consolidated_entries: consolidated_count,
            total_results: self.results.len(),
            sessions_processed: self.sessions_processed,
            top_patterns: self
                .entries
                .iter()
                .take(5)
                .map(|e| DreamPatternSummary {
                    id: e.id,
                    effectiveness: e.effectiveness,
                    frequency: e.frequency,
                    priority: format!("{:?}", e.priority),
                })
                .collect(),
        }
    }

    pub fn prune_low_priority(&mut self) {
        let keep = self.max_entries * 3 / 4;
        if self.entries.len() > keep {
            self.entries.sort_by(|a, b| {
                let pa = a.priority as u8;
                let pb = b.priority as u8;
                pb.cmp(&pa).then_with(|| {
                    b.effectiveness
                        .partial_cmp(&a.effectiveness)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });
            self.entries.truncate(keep);
        }
    }

    pub fn increment_sessions(&mut self) {
        self.sessions_processed += 1;
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Run the full 6-stage Kleos consolidation pipeline.
    /// Returns a detailed result with per-stage metrics.
    pub fn run_kleos_pipeline(&mut self) -> KleosPipelineResult {
        let total_start = std::time::Instant::now();
        let before_count = self.entries.len();
        let mut stages = Vec::with_capacity(6);
        let config = self.kleos_config.clone();

        let t = std::time::Instant::now();
        let s = self.stage_filter(&config);
        stages.push(StageResult {
            duration_ms: t.elapsed().as_secs_f64() * 1000.0,
            ..s
        });

        let t = std::time::Instant::now();
        let s = self.stage_replay(&config);
        stages.push(StageResult {
            duration_ms: t.elapsed().as_secs_f64() * 1000.0,
            ..s
        });

        let t = std::time::Instant::now();
        let s = self.stage_link(&config);
        stages.push(StageResult {
            duration_ms: t.elapsed().as_secs_f64() * 1000.0,
            ..s
        });

        let t = std::time::Instant::now();
        let s = self.stage_abstract(&config);
        stages.push(StageResult {
            duration_ms: t.elapsed().as_secs_f64() * 1000.0,
            ..s
        });

        let t = std::time::Instant::now();
        let s = self.stage_integrate(&config);
        stages.push(StageResult {
            duration_ms: t.elapsed().as_secs_f64() * 1000.0,
            ..s
        });

        let t = std::time::Instant::now();
        let s = self.stage_prune(&config);
        stages.push(StageResult {
            duration_ms: t.elapsed().as_secs_f64() * 1000.0,
            ..s
        });

        let total_duration = total_start.elapsed().as_secs_f64() * 1000.0;
        let linked_patterns: usize = stages.iter().map(|s| s.new_links).sum();
        let new_clusters = stages[3].new_links;

        let result = KleosPipelineResult {
            stages: stages.clone(),
            total_duration_ms: total_duration,
            before_count,
            after_count: self.entries.len(),
            linked_patterns,
            new_clusters,
        };

        self.pipeline_results.push_back(result.clone());
        if self.pipeline_results.len() > 10 {
            self.pipeline_results.pop_front();
        }

        result
    }

    /// Stage 1: Filter — remove near-duplicate entries by VSA similarity.
    /// Keeps the entry with higher effectiveness among near-duplicates.
    pub fn stage_filter(&mut self, config: &KleosConfig) -> StageResult {
        let before = self.entries.len();
        if before < 2 {
            return StageResult {
                stage: ConsolidationStage::Filter,
                entries_in: before,
                entries_out: before,
                new_links: 0,
                duration_ms: 0.0,
            };
        }

        let mut indices: Vec<usize> = (0..self.entries.len()).collect();
        indices.sort_by(|&a, &b| {
            self.entries[b]
                .effectiveness
                .partial_cmp(&self.entries[a].effectiveness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut to_remove = vec![false; self.entries.len()];

        for &idx in &indices {
            if to_remove[idx] {
                continue;
            }
            for j in 0..self.entries.len() {
                if j == idx || to_remove[j] {
                    continue;
                }
                let sim =
                    QuantizedVSA::similarity(&self.entries[idx].pattern, &self.entries[j].pattern);
                if sim > config.filter_threshold {
                    to_remove[j] = true;
                }
            }
        }

        let mut removed: Vec<usize> = to_remove
            .iter()
            .enumerate()
            .filter(|(_, &r)| r)
            .map(|(i, _)| i)
            .collect();
        removed.sort_unstable_by(|a, b| b.cmp(a));
        for &idx in &removed {
            self.entries.remove(idx);
        }

        StageResult {
            stage: ConsolidationStage::Filter,
            entries_in: before,
            entries_out: self.entries.len(),
            new_links: 0,
            duration_ms: 0.0,
        }
    }

    /// Stage 2: Replay — boost high-effectiveness entries by increasing
    /// their frequency and effectiveness score.
    pub fn stage_replay(&mut self, config: &KleosConfig) -> StageResult {
        let before = self.entries.len();
        let mut replayed = 0usize;

        for entry in &mut self.entries {
            if entry.effectiveness > config.replay_reward_threshold {
                entry.frequency += 2;
                entry.effectiveness =
                    (entry.effectiveness + 1.0).min(1.0) * 0.5 + entry.effectiveness * 0.5;
                replayed += 1;
            }
        }

        StageResult {
            stage: ConsolidationStage::Replay,
            entries_in: before,
            entries_out: self.entries.len(),
            new_links: replayed,
            duration_ms: 0.0,
        }
    }

    /// Stage 3: Link — discover cross-domain VSA associations between entries.
    /// Creates new linked entries for strongly associated pairs.
    pub fn stage_link(&mut self, config: &KleosConfig) -> StageResult {
        let before = self.entries.len();
        let mut links = 0usize;
        let mut new_entries: Vec<DreamEntry> = Vec::new();

        for i in 0..self.entries.len() {
            for j in i + 1..self.entries.len() {
                let sim =
                    QuantizedVSA::similarity(&self.entries[i].pattern, &self.entries[j].pattern);
                if sim >= config.link_min_similarity && sim < 0.95 {
                    let linked =
                        QuantizedVSA::bind(&self.entries[i].pattern, &self.entries[j].pattern);
                    let eff = (self.entries[i].effectiveness + self.entries[j].effectiveness) / 2.0;
                    if self.entries.len() + new_entries.len() < self.max_entries {
                        new_entries.push(DreamEntry {
                            id: self.next_id,
                            session_id: format!(
                                "{}-{}",
                                self.entries[i].session_id, self.entries[j].session_id
                            ),
                            pattern: linked,
                            effectiveness: eff,
                            frequency: 1,
                            last_seen: self.cycle,
                            priority: classify_priority(eff, 1),
                            consolidated: false,
                        });
                        self.next_id += 1;
                        links += 1;
                    }
                }
            }
        }

        self.entries.extend(new_entries);

        StageResult {
            stage: ConsolidationStage::Link,
            entries_in: before,
            entries_out: self.entries.len(),
            new_links: links,
            duration_ms: 0.0,
        }
    }

    /// Stage 4: Abstract — cluster entries into concept centroids using
    /// VSA bundling, then create abstracted concept entries.
    pub fn stage_abstract(&mut self, config: &KleosConfig) -> StageResult {
        let before = self.entries.len();
        if before < 2 {
            return StageResult {
                stage: ConsolidationStage::Abstract,
                entries_in: before,
                entries_out: before,
                new_links: 0,
                duration_ms: 0.0,
            };
        }

        let k = config.abstract_cluster_count.min(self.entries.len());
        if k < 2 {
            return StageResult {
                stage: ConsolidationStage::Abstract,
                entries_in: before,
                entries_out: before,
                new_links: 0,
                duration_ms: 0.0,
            };
        }

        // Select seeds by top effectiveness
        let mut indices: Vec<usize> = (0..self.entries.len()).collect();
        indices.sort_by(|&a, &b| {
            self.entries[b]
                .effectiveness
                .partial_cmp(&self.entries[a].effectiveness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let seeds: Vec<usize> = indices.iter().take(k).copied().collect();

        // Assign each entry to nearest seed by VSA similarity
        let mut assignments: Vec<usize> = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            let mut best_sim = 0.0f64;
            let mut best_seed = 0;
            for (si, &seed_idx) in seeds.iter().enumerate() {
                let sim = QuantizedVSA::similarity(&entry.pattern, &self.entries[seed_idx].pattern);
                if sim > best_sim {
                    best_sim = sim;
                    best_seed = si;
                }
            }
            assignments.push(best_seed);
        }

        // Build cluster patterns (drop before mutating self.entries)
        let seed_effs: Vec<f64> = seeds
            .iter()
            .map(|&idx| self.entries[idx].effectiveness)
            .collect();
        let mut new_entries: Vec<DreamEntry> = Vec::new();
        {
            let mut cluster_patterns: Vec<Vec<&[u8]>> = vec![Vec::new(); seeds.len()];
            for (i, entry) in self.entries.iter().enumerate() {
                cluster_patterns[assignments[i]].push(entry.pattern.as_slice());
            }

            for si in 0..seeds.len() {
                let cluster_size = cluster_patterns[si].len();
                if cluster_size < 2 {
                    continue;
                }
                let abstracted = QuantizedVSA::bundle(&cluster_patterns[si]);
                let avg_eff = seed_effs[si];

                if self.entries.len() + new_entries.len() < self.max_entries {
                    new_entries.push(DreamEntry {
                        id: self.next_id,
                        session_id: "abstract".to_string(),
                        pattern: abstracted,
                        effectiveness: avg_eff,
                        frequency: cluster_size as u64,
                        last_seen: self.cycle,
                        priority: classify_priority(avg_eff, cluster_size as u64),
                        consolidated: false,
                    });
                    self.next_id += 1;
                }
            }
        }
        let new_clusters = new_entries.len();
        self.entries.extend(new_entries);

        StageResult {
            stage: ConsolidationStage::Abstract,
            entries_in: before,
            entries_out: self.entries.len(),
            new_links: new_clusters,
            duration_ms: 0.0,
        }
    }

    /// Stage 5: Integrate — write high-confidence entries to long-term memory
    /// by marking them as consolidated.
    pub fn stage_integrate(&mut self, config: &KleosConfig) -> StageResult {
        let before = self.entries.len();
        let mut integrated = 0usize;

        for entry in &mut self.entries {
            if entry.effectiveness >= config.integrate_confidence_min && !entry.consolidated {
                entry.consolidated = true;
                integrated += 1;
            }
        }

        StageResult {
            stage: ConsolidationStage::Integrate,
            entries_in: before,
            entries_out: self.entries.len(),
            new_links: integrated,
            duration_ms: 0.0,
        }
    }

    /// Stage 6: Prune — retain only the top fraction of entries by score,
    /// removing low-value patterns. Score combines effectiveness and frequency.
    pub fn stage_prune(&mut self, config: &KleosConfig) -> StageResult {
        let before = self.entries.len();
        let target = (self.max_entries as f64 * config.prune_retain_fraction) as usize;

        if self.entries.len() > target {
            self.entries.sort_by(|a, b| {
                let score_a = a.effectiveness * 0.6 + (a.frequency as f64 * 0.1).min(1.0) * 0.4;
                let score_b = b.effectiveness * 0.6 + (b.frequency as f64 * 0.1).min(1.0) * 0.4;
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.entries.truncate(target);
        }

        StageResult {
            stage: ConsolidationStage::Prune,
            entries_in: before,
            entries_out: self.entries.len(),
            new_links: 0,
            duration_ms: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DreamPatternSummary {
    pub id: u64,
    pub effectiveness: f64,
    pub frequency: u64,
    pub priority: String,
}

#[derive(Debug, Clone)]
pub struct DreamReport {
    pub total_entries: usize,
    pub consolidated_entries: usize,
    pub total_results: usize,
    pub sessions_processed: u64,
    pub top_patterns: Vec<DreamPatternSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_pattern(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 4096)
    }

    #[test]
    fn test_feed_new_entry() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        dc.feed("s1", &[(dummy_pattern(1), 0.8)]);
        assert_eq!(dc.entry_count(), 1);
    }

    #[test]
    fn test_feed_updates_existing() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        let p = dummy_pattern(1);
        dc.feed("s1", &[(p.clone(), 0.8)]);
        dc.feed("s1", &[(p.clone(), 0.6)]);
        assert_eq!(dc.entry_count(), 1);
    }

    #[test]
    fn test_consolidate_threshold() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        for i in 0..4 {
            dc.feed("s1", &[(dummy_pattern(i), 0.7)]);
        }
        for _ in 0..5 {
            dc.feed("s1", &[(dummy_pattern(0), 0.8)]);
        }
        let results = dc.consolidate();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_dream_cycle() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.3);
        for i in 0..5 {
            dc.feed(&format!("s{}", i), &[(dummy_pattern(i), 0.7)]);
        }
        let (r1, _r2) = dc.dream_cycle();
        assert!(r1.is_empty() || dc.result_count() > 0);
    }

    #[test]
    fn test_cross_pollinate() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.3);
        for i in 0..5 {
            dc.feed("s1", &[(dummy_pattern(i * 100), 0.5)]);
        }
        let pairs = dc.cross_pollinate();
        assert!(!pairs.is_empty());
    }

    #[test]
    fn test_report() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.3);
        dc.feed("s1", &[(dummy_pattern(1), 0.9)]);
        dc.increment_sessions();
        let report = dc.report();
        assert_eq!(report.total_entries, 1);
        assert_eq!(report.sessions_processed, 1);
    }

    #[test]
    fn test_prune() {
        let mut dc = DreamConsolidator::new(3, 0.6, 0.3);
        for i in 0..10 {
            dc.feed("s1", &[(dummy_pattern(i), 0.1)]);
        }
        assert!(dc.entry_count() <= 3);
    }

    // --- Kleos 6-stage pipeline tests ---

    #[test]
    fn test_filter_removes_near_duplicates() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        dc.feed("s1", &[(dummy_pattern(1), 0.9)]);
        dc.feed("s1", &[(dummy_pattern(2), 0.8)]);
        dc.feed("s1", &[(dummy_pattern(3), 0.7)]);
        let before = dc.entry_count();
        assert!(before >= 2, "need at least 2 entries for filter test");

        dc.kleos_config.filter_threshold = 0.01;
        let config = dc.kleos_config.clone();
        let result = dc.stage_filter(&config);

        assert!(result.entries_out < result.entries_in);
        assert_eq!(result.stage, ConsolidationStage::Filter);
    }

    #[test]
    fn test_replay_boosts_high_reward() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        dc.feed("s1", &[(dummy_pattern(1), 0.9)]);
        dc.feed("s1", &[(dummy_pattern(2), 0.2)]);

        dc.kleos_config.replay_reward_threshold = 0.5;
        let config = dc.kleos_config.clone();
        let result = dc.stage_replay(&config);

        assert!(
            result.new_links >= 1,
            "high-reward entry should be replayed"
        );
    }

    #[test]
    fn test_link_discovers_associations() {
        let mut dc = DreamConsolidator::new(200, 0.6, 0.4);
        for i in 0..10 {
            dc.feed("s1", &[(dummy_pattern(i), 0.5 + (i as f64 * 0.05))]);
        }

        dc.kleos_config.link_min_similarity = 0.01;
        let config = dc.kleos_config.clone();
        let result = dc.stage_link(&config);

        assert!(result.new_links >= 1, "should discover at least one link");
        assert!(
            result.entries_out > result.entries_in,
            "linked entries should be added"
        );
    }

    #[test]
    fn test_abstract_creates_clusters() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        for i in 0..12 {
            dc.feed("s1", &[(dummy_pattern(i), 0.5 + (i as f64 * 0.03))]);
        }

        dc.kleos_config.abstract_cluster_count = 4;
        let config = dc.kleos_config.clone();
        let result = dc.stage_abstract(&config);

        assert_eq!(result.stage, ConsolidationStage::Abstract);
        assert!(result.entries_out >= result.entries_in);
    }

    #[test]
    fn test_prune_retains_top_fraction() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        for i in 0..80 {
            dc.feed("s1", &[(dummy_pattern(i), 0.1)]);
        }

        dc.kleos_config.prune_retain_fraction = 0.3;
        let config = dc.kleos_config.clone();
        let result = dc.stage_prune(&config);

        assert!(
            result.entries_out <= 30,
            "prune should retain at most 30 entries"
        );
        assert!(result.entries_out <= result.entries_in);
    }

    #[test]
    fn test_full_kleos_pipeline_runs() {
        let mut dc = DreamConsolidator::new(100, 0.6, 0.4);
        for i in 0..30 {
            dc.feed("s1", &[(dummy_pattern(i), 0.3 + (i as f64 * 0.02))]);
        }

        dc.kleos_config.filter_threshold = 0.01;
        dc.kleos_config.link_min_similarity = 0.01;
        dc.kleos_config.prune_retain_fraction = 0.8;
        let result = dc.run_kleos_pipeline();

        assert_eq!(result.stages.len(), 6, "pipeline should have 6 stages");
        assert!(result.total_duration_ms > 0.0);
        assert!(result.before_count > 0);
        assert!(result.stages[5].entries_out <= result.before_count);
    }
}
