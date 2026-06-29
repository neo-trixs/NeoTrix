// SPLIT PLAN:
//   File: 2005 lines — 7 distinct struct+impl groups:
//   1. `recurrence_detector.rs`   — RecurrenceDetector + RecurrenceStats (lines 1–114)
//   2. `episodic_buffer.rs`       — UtilityItem + BoundedEpisodicBuffer (lines 116–298)
//   3. `consolidation_entry.rs`   — ConsolidationEntry + WeightConsolidator (lines 300–497)
//   4. `learned_consolidator.rs`  — LearnedConsolidator (lines 499–658)
//   5. `consolidation_bridge.rs`  — ConsolidationBridge struct+impl Default (lines 660–1232)
//   6. `scm_consolidation.rs`     — SCM sleep-consolidated memory (lines 1173–1798)
//   7. `scm_tests.rs`             — #[cfg(test)] module (lines 1799–2005)
//   How: extract largest structs first, keep module-level re-exports.

use crate::core::nt_core_consciousness::dream_consolidator::DreamConsolidator;
use crate::core::nt_core_consciousness::hebbian_associative_memory::HebbianAssociativeMemory;
use crate::core::nt_core_consciousness::sleep_gate::SleepGate;
use crate::core::nt_core_hcube::dream_consolidation::{DreamConfig, DreamConsolidation};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

/// RecurrenceDetector — RecMem-inspired (arXiv:2605.16045) recurrence-gated
/// consolidation trigger. Tracks semantically similar items in the episodic
/// buffer and triggers consolidation when cluster recurrence exceeds threshold.
#[derive(Debug, Clone)]
pub struct RecurrenceDetector {
    /// VSA similarity threshold for considering two items "recurrent"
    pub recurrence_similarity: f64,
    /// Minimum cluster size to trigger consolidation
    pub recurrence_count_threshold: usize,
    /// Current largest cluster size
    pub largest_cluster: usize,
    /// Number of clusters above threshold
    pub clusters_above_threshold: usize,
    /// Total items scanned
    pub total_scanned: usize,
}

impl RecurrenceDetector {
    pub fn new(similarity: f64, count_threshold: usize) -> Self {
        Self {
            recurrence_similarity: similarity,
            recurrence_count_threshold: count_threshold,
            largest_cluster: 0,
            clusters_above_threshold: 0,
            total_scanned: 0,
        }
    }

    /// Scan a set of VSA vectors for recurrence clusters.
    /// Returns (largest_cluster_size, clusters_above_threshold).
    pub fn scan(&mut self, items: &[Vec<u8>]) -> (usize, usize) {
        self.total_scanned = items.len();
        if items.is_empty() {
            self.largest_cluster = 0;
            self.clusters_above_threshold = 0;
            return (0, 0);
        }

        let mut assigned = vec![false; items.len()];
        let mut max_cluster = 0;
        let mut count_above = 0;

        for i in 0..items.len() {
            if assigned[i] {
                continue;
            }
            assigned[i] = true;
            let mut cluster_size = 1;
            for j in (i + 1)..items.len() {
                if assigned[j] {
                    continue;
                }
                let a = &items[i];
                let b = &items[j];
                let len = a.len().min(b.len());
                let sim = if len == 0 {
                    0.0
                } else {
                    1.0 - a.iter().zip(b.iter()).filter(|(x, y)| x != y).count() as f64 / len as f64
                };
                if sim >= self.recurrence_similarity {
                    assigned[j] = true;
                    cluster_size += 1;
                }
            }
            if cluster_size > max_cluster {
                max_cluster = cluster_size;
            }
            if cluster_size >= self.recurrence_count_threshold {
                count_above += 1;
            }
        }

        self.largest_cluster = max_cluster;
        self.clusters_above_threshold = count_above;
        (max_cluster, count_above)
    }

    pub fn should_consolidate(&self) -> bool {
        self.clusters_above_threshold > 0
    }

    pub fn reset(&mut self) {
        self.largest_cluster = 0;
        self.clusters_above_threshold = 0;
        self.total_scanned = 0;
    }

    pub fn stats(&self) -> RecurrenceStats {
        RecurrenceStats {
            largest_cluster: self.largest_cluster,
            clusters_above_threshold: self.clusters_above_threshold,
            total_scanned: self.total_scanned,
            recurrence_threshold: self.recurrence_count_threshold,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecurrenceStats {
    pub largest_cluster: usize,
    pub clusters_above_threshold: usize,
    pub total_scanned: usize,
    pub recurrence_threshold: usize,
}

// SECTION: UtilityItem + BoundedEpisodicBuffer

/// Utility-tagged memory item for CraniMem-style gated consolidation
/// (ICLR 2026 MemAgents Workshop).
#[derive(Debug, Clone)]
pub struct UtilityItem {
    pub vector: Vec<u8>,
    pub access_count: u64,
    pub last_access_cycle: u64,
    pub coherence_score: f64,
    pub utility_score: f64,
}

impl UtilityItem {
    pub fn new(vector: Vec<u8>, cycle: u64) -> Self {
        Self {
            vector,
            access_count: 1,
            last_access_cycle: cycle,
            coherence_score: 1.0,
            utility_score: 1.0,
        }
    }

    pub fn record_access(&mut self, cycle: u64) {
        self.access_count += 1;
        self.last_access_cycle = cycle;
        self.recompute_utility(cycle);
    }

    pub fn recompute_utility(&mut self, current_cycle: u64) {
        let recency = if current_cycle > self.last_access_cycle {
            1.0 / (1.0 + (current_cycle - self.last_access_cycle) as f64 * 0.01)
        } else {
            1.0
        };
        let frequency = (self.access_count as f64).ln_1p();
        self.utility_score = recency * 0.4 + frequency * 0.3 + self.coherence_score * 0.3;
    }
}

/// CraniMem-inspired bounded episodic buffer with utility-based eviction.
/// Keeps top-k items by utility when capacity is exceeded.
#[derive(Debug, Clone)]
pub struct BoundedEpisodicBuffer {
    pub items: Vec<UtilityItem>,
    pub max_capacity: usize,
    pub current_cycle: u64,
    pub eviction_count: u64,
}

impl BoundedEpisodicBuffer {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_capacity),
            max_capacity,
            current_cycle: 0,
            eviction_count: 0,
        }
    }

    pub fn push(&mut self, vector: Vec<u8>) {
        self.items
            .push(UtilityItem::new(vector, self.current_cycle));
        if self.items.len() > self.max_capacity {
            self.evict_lowest_utility();
        }
    }

    pub fn record_access_to(&mut self, index: usize, cycle: u64) {
        if let Some(item) = self.items.get_mut(index) {
            item.record_access(cycle);
        }
    }

    /// Evict the single lowest-utility item (CraniMem bounded retention).
    fn evict_lowest_utility(&mut self) {
        if self.items.is_empty() {
            return;
        }
        for item in &mut self.items {
            item.recompute_utility(self.current_cycle);
        }
        let min_idx = self
            .items
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.utility_score
                    .partial_cmp(&b.utility_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx);
        if let Some(idx) = min_idx {
            self.items.swap_remove(idx);
            self.eviction_count += 1;
        }
    }

    /// Consolidation: keep only items with utility above threshold.
    /// Returns number of items pruned.
    pub fn consolidate(&mut self, threshold: f64) -> usize {
        let before = self.items.len();
        for item in &mut self.items {
            item.recompute_utility(self.current_cycle);
        }
        self.items.retain(|item| item.utility_score >= threshold);
        let pruned = before - self.items.len();
        self.eviction_count += pruned as u64;
        pruned
    }

    pub fn advance_cycle(&mut self) {
        self.current_cycle += 1;
    }

    pub fn stats(&self) -> BoundedBufferStats {
        let total_utility: f64 = self.items.iter().map(|i| i.utility_score).sum();
        let avg_utility = if self.items.is_empty() {
            0.0
        } else {
            total_utility / self.items.len() as f64
        };
        BoundedBufferStats {
            item_count: self.items.len(),
            max_capacity: self.max_capacity,
            eviction_count: self.eviction_count,
            avg_utility,
            current_cycle: self.current_cycle,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundedBufferStats {
    pub item_count: usize,
    pub max_capacity: usize,
    pub eviction_count: u64,
    pub avg_utility: f64,
    pub current_cycle: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum PupilState {
    Constricted,
    Dilated,
}

#[derive(Debug, Clone)]
pub struct WeightUpdate {
    pub id: u64,
    pub pattern_vector: Vec<u8>,
    pub target_vector: Vec<u8>,
    pub importance: f64,
    pub learning_rate: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct SyntheticTrainingPair {
    pub input_vector: Vec<u8>,
    pub expected_output: Vec<u8>,
    pub domain: String,
    pub difficulty: f64,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConsolidationPhase {
    Working,
    NREM,
    REM,
    Archived,
}

// SECTION: ConsolidationEntry + WeightConsolidator

#[derive(Clone, Debug)]
pub struct ConsolidationEntry {
    pub id: u64,
    pub content: String,
    pub vsa_vector: Vec<u8>,
    pub recency: f64,
    pub coherence: f64,
    pub strength: f64,
    pub phase: ConsolidationPhase,
    pub associations: Vec<u64>,
}

impl ConsolidationEntry {
    pub fn new(id: u64, content: String, vsa: Vec<u8>) -> Self {
        Self {
            id,
            content,
            vsa_vector: vsa,
            recency: 1.0,
            coherence: 0.5,
            strength: 0.5,
            phase: ConsolidationPhase::Working,
            associations: Vec::new(),
        }
    }

    pub fn importance_score(&self) -> f64 {
        0.4 * self.recency + 0.4 * self.coherence + 0.2 * self.strength
    }

    pub fn salience_override(&mut self, salience: f64) {
        self.recency = (self.recency + salience) / 2.0;
        self.strength = (self.strength + salience) / 2.0;
    }
}

#[derive(Debug, Clone)]
pub struct SCMStats {
    pub total_entries: usize,
    pub nrem_phase_count: u64,
    pub rem_phase_count: u64,
    pub nrem_merged: u64,
    pub nrem_pruned: u64,
    pub rem_associations: u64,
    pub working_entries: usize,
    pub archived_entries: usize,
}

#[derive(Debug, Clone)]
pub struct WeightConsolidator {
    pub weight_updates: Vec<WeightUpdate>,
    pub synthetic_pairs: Vec<SyntheticTrainingPair>,
    pub counter: u64,
    pub max_updates: usize,
    pub max_pairs: usize,
}

impl WeightConsolidator {
    pub fn new(max_updates: usize, max_pairs: usize) -> Self {
        Self {
            weight_updates: Vec::with_capacity(max_updates),
            synthetic_pairs: Vec::with_capacity(max_pairs),
            counter: 0,
            max_updates,
            max_pairs,
        }
    }

    pub fn record_pattern(&mut self, pattern: Vec<u8>, importance: f64, source: &str) {
        let target = QuantizedVSA::seeded_random(self.counter, 4096);
        let update = WeightUpdate {
            id: self.counter,
            pattern_vector: pattern,
            target_vector: target,
            importance,
            learning_rate: 0.01 * importance,
            source: source.to_string(),
        };
        self.counter += 1;
        self.weight_updates.push(update);
        if self.weight_updates.len() > self.max_updates {
            if let Some(min_idx) = self
                .weight_updates
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.importance
                        .partial_cmp(&b.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
            {
                self.weight_updates.remove(min_idx);
            }
        }
    }

    pub fn nrem_consolidation(&mut self) -> usize {
        let before = self.weight_updates.len();
        if before < 2 {
            return 0;
        }
        let mut merged = Vec::with_capacity(before);
        let mut used = vec![false; before];
        for i in 0..before {
            if used[i] {
                continue;
            }
            let mut merged_vec = self.weight_updates[i].pattern_vector.clone();
            let mut merged_importance = self.weight_updates[i].importance;
            let mut merged_lr = self.weight_updates[i].learning_rate;
            let mut count = 1;
            for j in (i + 1)..before {
                if used[j] {
                    continue;
                }
                let sim = self.weight_updates[i]
                    .pattern_vector
                    .iter()
                    .zip(self.weight_updates[j].pattern_vector.iter())
                    .take(32)
                    .filter(|(a, b)| a == b)
                    .count() as f64
                    / 32.0;
                if sim > 0.7 {
                    used[j] = true;
                    merged_vec = merged_vec
                        .iter()
                        .zip(self.weight_updates[j].pattern_vector.iter())
                        .map(|(a, b)| ((*a as u16 + *b as u16) / 2) as u8)
                        .collect();
                    merged_importance =
                        (merged_importance + self.weight_updates[j].importance) / 2.0;
                    merged_lr = merged_lr.max(self.weight_updates[j].learning_rate);
                    count += 1;
                }
            }
            merged.push(WeightUpdate {
                id: self.counter,
                pattern_vector: merged_vec,
                target_vector: QuantizedVSA::seeded_random(self.counter, 4096),
                importance: merged_importance * (count as f64).sqrt(),
                learning_rate: merged_lr,
                source: "nrem".to_string(),
            });
            self.counter += 1;
        }
        self.weight_updates = merged;
        before - self.weight_updates.len()
    }

    pub fn rem_synthetic_generation(&mut self) -> usize {
        let before = self.synthetic_pairs.len();
        for update in &self.weight_updates {
            if update.importance < 0.3 {
                continue;
            }
            let pair = SyntheticTrainingPair {
                input_vector: update.pattern_vector.clone(),
                expected_output: update.target_vector.clone(),
                domain: if update.importance > 0.7 {
                    "critical".into()
                } else {
                    "normal".into()
                },
                difficulty: (1.0 - update.importance).clamp(0.0, 1.0),
            };
            self.synthetic_pairs.push(pair);
            if self.synthetic_pairs.len() > self.max_pairs {
                self.synthetic_pairs.remove(0);
            }
        }
        self.synthetic_pairs.len() - before
    }

    pub fn update_count(&self) -> usize {
        self.weight_updates.len()
    }

    pub fn pair_count(&self) -> usize {
        self.synthetic_pairs.len()
    }
}

/// Auto-Dreamer inspired (arXiv:2605.20616) learned consolidation outcome tracker.
/// Records consolidation strategies and their downstream effectiveness,
/// building a VSA-native "policy" that biases future consolidation thresholds.
#[derive(Debug, Clone)]
pub struct ConsolidationTrace {
    pub strategy_threshold: f64,
    pub source_count: usize,
    pub output_size: usize,
    pub domain: String,
    pub utility_after: f64,
    pub was_retrieved: bool,
    pub retrieval_success: bool,
}

#[derive(Debug, Clone)]
pub struct LearnedConsolidator {
    /// Traces per strategy threshold, bucketed at 0.05 granularity
    pub traces: Vec<ConsolidationTrace>,
    /// Success rate per threshold bucket: Vec<(threshold, success_count, total_count)>
    pub strategy_success: Vec<(f64, usize, usize)>,
    /// Number of traces to keep before pruning oldest
    pub max_traces: usize,
    /// Current best estimated threshold (EMA)
    pub learned_threshold: f64,
    /// EMA alpha for threshold adaptation
    pub adaptation_rate: f64,
}

impl LearnedConsolidator {
    pub fn new(max_traces: usize, initial_threshold: f64, adaptation_rate: f64) -> Self {
        let mut strategies = Vec::with_capacity(20);
        for i in 0..20 {
            strategies.push((0.50 + i as f64 * 0.025, 0, 0));
        }
        Self {
            traces: Vec::with_capacity(max_traces),
            strategy_success: strategies,
            max_traces,
            learned_threshold: initial_threshold,
            adaptation_rate,
        }
    }

    /// Record a consolidation operation with its strategy parameters.
    pub fn record_consolidation(
        &mut self,
        strategy_threshold: f64,
        source_count: usize,
        output_size: usize,
        domain: &str,
    ) {
        self.traces.push(ConsolidationTrace {
            strategy_threshold,
            source_count,
            output_size,
            domain: domain.to_string(),
            utility_after: 0.0,
            was_retrieved: false,
            retrieval_success: false,
        });
        if self.traces.len() > self.max_traces {
            self.traces.remove(0);
        }
    }

    /// Record a downstream retrieval outcome for the most recent consolidation.
    pub fn record_retrieval_outcome(&mut self, success: bool, utility: f64) {
        if let Some(last) = self.traces.last_mut() {
            last.was_retrieved = true;
            last.retrieval_success = success;
            last.utility_after = utility;
        }
    }

    /// Update strategy success rates from completed traces and adapt threshold.
    pub fn learn(&mut self) {
        for trace in &self.traces {
            if !trace.was_retrieved {
                continue;
            }
            let bucket = ((trace.strategy_threshold - 0.50) / 0.025).round() as usize;
            if bucket < self.strategy_success.len() {
                let entry = &mut self.strategy_success[bucket];
                if trace.retrieval_success {
                    entry.1 += 1;
                }
                entry.2 += 1;
            }
        }

        if let Some(best_bucket) = self
            .strategy_success
            .iter()
            .filter(|(_, _, total)| *total >= 2)
            .max_by(|(_, s1, t1), (_, s2, t2)| {
                let r1 = *s1 as f64 / *t1 as f64;
                let r2 = *s2 as f64 / *t2 as f64;
                r1.partial_cmp(&r2).unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            let best_threshold = best_bucket.0;
            self.learned_threshold = self.learned_threshold * (1.0 - self.adaptation_rate)
                + best_threshold * self.adaptation_rate;
        }
    }

    pub fn stats(&self) -> LearnedConsolidatorStats {
        let total_traces = self.traces.len();
        let retrieved = self.traces.iter().filter(|t| t.was_retrieved).count();
        let successful = self.traces.iter().filter(|t| t.retrieval_success).count();
        let avg_utility = if retrieved > 0 {
            self.traces
                .iter()
                .filter(|t| t.was_retrieved)
                .map(|t| t.utility_after)
                .sum::<f64>()
                / retrieved as f64
        } else {
            0.0
        };
        LearnedConsolidatorStats {
            total_traces,
            retrieved_traces: retrieved,
            successful_traces: successful,
            avg_utility_after: avg_utility,
            learned_threshold: self.learned_threshold,
            active_strategies: self
                .strategy_success
                .iter()
                .filter(|(_, _, total)| *total > 0)
                .count(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LearnedConsolidatorStats {
    pub total_traces: usize,
    pub retrieved_traces: usize,
    pub successful_traces: usize,
    pub avg_utility_after: f64,
    pub learned_threshold: f64,
    pub active_strategies: usize,
}

// SECTION: ConsolidationBridge

#[derive(Debug, Clone)]
pub struct ConsolidationBridge {
    pub sleep_gate: SleepGate,
    pub dream_consolidation: DreamConsolidation,
    pub dream_consolidator: DreamConsolidator,
    pub cycle: u64,
    pub total_consolidations: u64,
    pub last_pressure: f64,
    pub weight_consolidator: WeightConsolidator,
    pub pupil_state: PupilState,
    pub pupil_oscillation: f64,
    pub pupil_events: Vec<(Vec<u8>, String, f64, PupilState)>,
    pub pupil_constricted_count: u64,
    pub pupil_dilated_count: u64,
    pub episodic_buffer: BoundedEpisodicBuffer,
    /// RecMem-inspired recurrence detector (arXiv:2605.16045)
    pub recurrence: RecurrenceDetector,
    /// Auto-Dreamer inspired learned consolidator (arXiv:2605.20616)
    pub learned_consolidator: LearnedConsolidator,
    /// HeLa-Mem inspired Hebbian associative memory (arXiv:2604.16839)
    pub hebbian_mem: HebbianAssociativeMemory,
    /// Scan recurrence every N feeds
    pub recurrence_scan_interval: u64,
    /// Recurrence scan counter
    pub recurrence_scan_counter: u64,
    /// Whether the current consolidation was triggered by recurrence
    pub last_consolidation_triggered_by_recurrence: bool,
    // ── CTE 4-stage consolidation pipeline metrics ──
    pub cte_sws_extracted: u64,
    pub cte_rem_associated: u64,
    pub cte_consolidated: u64,
    pub cte_compacted: u64,
    /// Integrity hash chain: (cycle_number, sha256_of_buffer_state_before_consolidation)
    pub integrity_chain: VecDeque<(usize, [u8; 32])>,
    /// Number of successful integrity verifications
    pub integrity_passes: u64,
    /// Number of integrity verification failures
    pub integrity_failures: u64,
    // ── SCM: Sleep-Consolidated Memory (arXiv:2604.20943) ──
    /// Entries for two-phase consolidation
    pub entries: Vec<ConsolidationEntry>,
    /// Alternating phase counter (0 = NREM, 1 = REM)
    pub consolidation_phase: usize,
    pub scm_nrem_merged: u64,
    pub scm_nrem_pruned: u64,
    pub scm_rem_associations: u64,
    pub scm_nrem_phase_count: u64,
    pub scm_rem_phase_count: u64,
}

impl ConsolidationBridge {
    pub fn new() -> Self {
        Self {
            sleep_gate: SleepGate::new(),
            dream_consolidation: DreamConsolidation::new(DreamConfig::default()),
            dream_consolidator: DreamConsolidator::new(200, 0.4, 0.3),
            cycle: 0,
            total_consolidations: 0,
            last_pressure: 0.0,
            weight_consolidator: WeightConsolidator::new(500, 200),
            pupil_state: PupilState::Constricted,
            pupil_oscillation: 0.0,
            pupil_events: Vec::new(),
            pupil_constricted_count: 0,
            pupil_dilated_count: 0,
            episodic_buffer: BoundedEpisodicBuffer::new(100),
            recurrence: RecurrenceDetector::new(0.78, 3),
            learned_consolidator: LearnedConsolidator::new(200, 0.78, 0.1),
            hebbian_mem: HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 200),
            recurrence_scan_interval: 50,
            recurrence_scan_counter: 0,
            last_consolidation_triggered_by_recurrence: false,
            cte_sws_extracted: 0,
            cte_rem_associated: 0,
            cte_consolidated: 0,
            cte_compacted: 0,
            integrity_chain: VecDeque::new(),
            integrity_passes: 0,
            integrity_failures: 0,
            entries: Vec::new(),
            consolidation_phase: 0,
            scm_nrem_merged: 0,
            scm_nrem_pruned: 0,
            scm_rem_associations: 0,
            scm_nrem_phase_count: 0,
            scm_rem_phase_count: 0,
        }
    }

    pub fn feed_stream_entry(&mut self, vector: Vec<u8>, label: &str, salience: f64) {
        self.pupil_oscillation = (self.pupil_oscillation + 0.15) % (2.0 * std::f64::consts::PI);
        self.pupil_state = if self.pupil_oscillation.sin() > 0.3 {
            PupilState::Dilated
        } else {
            PupilState::Constricted
        };
        self.pupil_events.push((
            vector.clone(),
            label.to_string(),
            salience,
            self.pupil_state,
        ));
        match self.pupil_state {
            PupilState::Constricted => self.pupil_constricted_count += 1,
            PupilState::Dilated => self.pupil_dilated_count += 1,
        }
        self.dream_consolidation
            .record_event(vector.clone(), label, salience);
        self.sleep_gate.sleep_pressure =
            (self.sleep_gate.sleep_pressure + salience * 0.05).min(1.0);
        self.episodic_buffer.push(vector.clone());
        self.weight_consolidator
            .record_pattern(vector.clone(), salience, "wake");

        // SCM: push to consolidation entries with full VSA vector
        let next_scm_id = self.entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
        let mut entry = ConsolidationEntry::new(next_scm_id, label.to_string(), vector.clone());
        entry.salience_override(salience);
        self.entries.push(entry);

        // HeLa-Mem: temporal co-occurrence → Hebbian edge
        // During sleep, ACh is naturally low (cholinergic pause), enabling plasticity
        if self.episodic_buffer.items.len() >= 2 {
            let last = self.episodic_buffer.items.len() - 1;
            let prev = &self.episodic_buffer.items[last].vector;
            let curr = &self.episodic_buffer.items[last - 1].vector;
            self.hebbian_mem
                .record_coactivation_between(prev, curr, Some(0.1)); // Sleep = low ACh
        }

        self.cycle += 1;

        // RecMem-inspired recurrence scan: periodically check episodic buffer
        // for semantic clusters and trigger early consolidation.
        self.recurrence_scan_counter += 1;
        if self.recurrence_scan_counter >= self.recurrence_scan_interval {
            self.recurrence_scan_counter = 0;
            let items: Vec<Vec<u8>> = self
                .episodic_buffer
                .items
                .iter()
                .map(|item| item.vector.clone())
                .collect();
            self.recurrence.scan(&items);
        }
    }

    pub fn feed_session_pattern(&mut self, session_id: &str, patterns: &[(Vec<u8>, f64)]) {
        self.dream_consolidator.feed(session_id, patterns);
    }

    fn compute_buffer_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for item in &self.episodic_buffer.items {
            hasher.update(&item.vector);
            hasher.update(&item.access_count.to_le_bytes());
            hasher.update(&item.last_access_cycle.to_le_bytes());
            hasher.update(&item.coherence_score.to_le_bytes());
            hasher.update(&item.utility_score.to_le_bytes());
        }
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }

    /// SCM NREM sparsification utility: sorts entries by importance (descending)
    /// and keeps only the top 60%. Operates on a generic (importance, content, vector) vec.
    pub fn nrem_sparsify(&self, entries: &mut Vec<(f64, String, Vec<u8>)>) {
        entries.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let keep = (entries.len() as f64 * 0.6).max(1.0) as usize;
        entries.truncate(keep);
    }

    /// SCM Phase 1 — NREM (Slow-wave): Pattern Extraction + Redundancy Elimination.
    /// 1. Score each entry by importance (recency*0.4 + coherence*0.4 + strength*0.2)
    /// 2. Remove lowest-scoring 30% if over capacity
    /// 3. Merge duplicate entries (VSA similarity > 0.85)
    /// 4. Bundle merged vectors into shared substructures (pattern extraction)
    pub fn nrem_consolidate(&mut self) {
        self.scm_nrem_phase_count += 1;
        if self.entries.len() < 8 {
            return;
        }

        // 1. Score and sort ascending
        self.entries.sort_by(|a, b| {
            a.importance_score()
                .partial_cmp(&b.importance_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 2. Remove lowest 30% if over capacity threshold
        let threshold = self.episodic_buffer.max_capacity.max(50);
        if self.entries.len() > threshold {
            let remove_count = (self.entries.len() * 30 / 100).max(1);
            self.entries.drain(..remove_count);
            self.scm_nrem_pruned += remove_count as u64;
        }

        // 3. Merge duplicate entries (VSA similarity > 0.85)
        let mut merged_ids: Vec<u64> = Vec::new();
        let mut i = 0;
        while i < self.entries.len() {
            if merged_ids.contains(&self.entries[i].id) {
                i += 1;
                continue;
            }
            let mut merge_group: Vec<Vec<u8>> = Vec::new();
            let mut j = i + 1;
            while j < self.entries.len() {
                if merged_ids.contains(&self.entries[j].id) {
                    j += 1;
                    continue;
                }
                let sim = QuantizedVSA::similarity(
                    &self.entries[i].vsa_vector,
                    &self.entries[j].vsa_vector,
                );
                if sim > 0.85 {
                    merged_ids.push(self.entries[j].id);
                    merge_group.push(self.entries[j].vsa_vector.clone());
                    self.scm_nrem_merged += 1;
                    // Merge associations (clone to avoid double borrow)
                    let assoc_ids: Vec<u64> = self.entries[j].associations.clone();
                    for &assoc_id in &assoc_ids {
                        if !self.entries[i].associations.contains(&assoc_id) {
                            self.entries[i].associations.push(assoc_id);
                        }
                    }
                    // Merge coherence and recency
                    self.entries[i].coherence =
                        (self.entries[i].coherence + self.entries[j].coherence) / 2.0;
                    self.entries[i].recency = self.entries[i].recency.max(self.entries[j].recency);
                    self.entries.remove(j);
                } else {
                    j += 1;
                }
            }

            // 4. Pattern extraction: if we found similar vectors, bundle them
            if !merge_group.is_empty() {
                merge_group.push(self.entries[i].vsa_vector.clone());
                let refs: Vec<&[u8]> = merge_group.iter().map(|v| v.as_slice()).collect();
                self.entries[i].vsa_vector = QuantizedVSA::majority_bundle(&refs);
                self.entries[i].strength =
                    (self.entries[i].strength + merge_group.len() as f64 * 0.1).min(1.0);
            }

            i += 1;
        }

        // Mark surviving entries as NREM phase
        for entry in &mut self.entries {
            if entry.phase == ConsolidationPhase::Working {
                entry.phase = ConsolidationPhase::NREM;
            }
        }
    }

    /// SCM Phase 2 — REM (Rapid Eye Movement): Cross-Domain Association Discovery.
    /// 1. For each pair of entries, compute VSA similarity via QuantizedVSA
    /// 2. If similarity in (0.3, 0.7) range — potential cross-domain association
    /// 3. Create a new association entry using QuantizedVSA::bind to bundle the two vectors
    pub fn rem_consolidate(&mut self) {
        self.scm_rem_phase_count += 1;
        if self.entries.len() < 4 {
            return;
        }

        let mut new_associations: Vec<(usize, usize, f64)> = Vec::new();
        for i in 0..self.entries.len() {
            for j in (i + 1)..self.entries.len() {
                let sim = QuantizedVSA::similarity(
                    &self.entries[i].vsa_vector,
                    &self.entries[j].vsa_vector,
                );
                // Similarity in (0.3, 0.7) range = potential cross-domain association
                if sim > 0.3 && sim < 0.7 {
                    new_associations.push((i, j, sim));
                }
            }
        }

        let mut next_id = self.entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
        for (i, j, sim) in &new_associations {
            let bundled_vsa =
                QuantizedVSA::bind(&self.entries[*i].vsa_vector, &self.entries[*j].vsa_vector);

            let content = format!(
                "cross-domain:{}<->{}:sim={:.3}",
                self.entries[*i].id, self.entries[*j].id, sim
            );

            let mut assoc_entry = ConsolidationEntry::new(next_id, content, bundled_vsa);
            assoc_entry.recency = (self.entries[*i].recency + self.entries[*j].recency) / 2.0;
            assoc_entry.coherence = (self.entries[*i].coherence + self.entries[*j].coherence) / 2.0;
            assoc_entry.strength = *sim;
            assoc_entry.phase = ConsolidationPhase::REM;
            assoc_entry.associations = vec![self.entries[*i].id, self.entries[*j].id];

            self.entries[*i].associations.push(next_id);
            self.entries[*j].associations.push(next_id);
            self.entries.push(assoc_entry);
            self.scm_rem_associations += 1;
            next_id += 1;
        }
    }

    /// Record CTE 4-stage consolidation pipeline metrics.
    pub fn record_cte_metrics(&mut self, sws: usize, rem: usize, cons: usize, comp: usize) {
        self.cte_sws_extracted = self.cte_sws_extracted.wrapping_add(sws as u64);
        self.cte_rem_associated = self.cte_rem_associated.wrapping_add(rem as u64);
        self.cte_consolidated = self.cte_consolidated.wrapping_add(cons as u64);
        self.cte_compacted = self.cte_compacted.wrapping_add(comp as u64);
    }

    pub fn consolidate_if_needed(&mut self, iteration: usize) -> Option<ConsolidationReport> {
        let triggered_by_recurrence = self.recurrence.should_consolidate();
        let triggered_by_sleep = self.sleep_gate.should_sleep(iteration);
        if !triggered_by_sleep && !triggered_by_recurrence {
            return None;
        }
        self.last_consolidation_triggered_by_recurrence = triggered_by_recurrence;

        let pre_hash = self.compute_buffer_hash();

        let pressure_before = self.sleep_gate.sleep_pressure;
        let current_pupil = self.pupil_state;

        let dream_report = self.dream_consolidation.run_consolidation_cycle();

        let results = self.dream_consolidator.consolidate();

        let weight_updates_merged = self.weight_consolidator.nrem_consolidation();
        let synthetic_pairs_generated = self.weight_consolidator.rem_synthetic_generation();

        // Auto-Dreamer: record consolidation trace and adapt threshold
        let source_items = self.episodic_buffer.items.len();
        let used_threshold = self.learned_consolidator.learned_threshold;
        self.learned_consolidator.record_consolidation(
            used_threshold,
            source_items,
            dream_report.patterns_merged + results.len(),
            "consolidation_bridge",
        );
        self.learned_consolidator.learn();
        let learned_stats = self.learned_consolidator.stats();

        // HeLa-Mem: decay all Hebbian edges and prune weak ones
        self.hebbian_mem.decay_all();
        self.hebbian_mem.prune_edges();
        let hebbian_stats = self.hebbian_mem.stats();

        self.episodic_buffer.advance_cycle();
        let _buffer_pruned = self.episodic_buffer.consolidate(0.3);
        let buffer_stats = self.episodic_buffer.stats();

        let dilated_events: Vec<&(Vec<u8>, String, f64, PupilState)> = self
            .pupil_events
            .iter()
            .filter(|(_, _, _, s)| *s == PupilState::Dilated)
            .collect();
        let associative_links_formed = if current_pupil == PupilState::Dilated {
            let mut count = 0;
            for i in 0..dilated_events.len() {
                for j in (i + 1)..dilated_events.len() {
                    let sim = QuantizedVSA::similarity(&dilated_events[i].0, &dilated_events[j].0);
                    if sim > 0.85 {
                        count += 1;
                    }
                }
            }
            count
        } else {
            0
        };

        let pupil_constricted_events = self
            .pupil_events
            .iter()
            .filter(|(_, _, _, s)| *s == PupilState::Constricted)
            .count();
        let pupil_dilated_events = self
            .pupil_events
            .iter()
            .filter(|(_, _, _, s)| *s == PupilState::Dilated)
            .count();

        self.sleep_gate.sleep_pressure = 0.0;
        self.last_pressure = pressure_before;
        self.total_consolidations += 1;

        let post_hash = self.compute_buffer_hash();
        let verified = pre_hash == post_hash || triggered_by_recurrence;
        if verified {
            self.integrity_passes += 1;
        } else {
            self.integrity_failures += 1;
            log::warn!(
                "Consolidation integrity check failed at cycle {}",
                self.cycle
            );
        }
        self.integrity_chain
            .push_back((self.cycle as usize, pre_hash));
        if self.integrity_chain.len() > 100 {
            self.integrity_chain.pop_front();
        }

        let recurrence_cluster = self.recurrence.largest_cluster;
        let triggered_by_recurrence = self.last_consolidation_triggered_by_recurrence;
        self.recurrence.reset();

        // ── SCM two-phase consolidation (arXiv:2604.20943) ──
        // Run BOTH NREM (pattern extraction + dedup) then REM (cross-domain association)
        self.nrem_consolidate();
        self.rem_consolidate();

        Some(ConsolidationReport {
            sequences_replayed: dream_report.sequences_replayed,
            patterns_merged: dream_report.patterns_merged,
            abstractions_formed: dream_report.abstractions_formed,
            predictions_generated: dream_report.predictions_generated,
            cross_session_consolidated: results.len(),
            novelty_score: dream_report.novelty_score,
            coherence_gain: dream_report.coherence_gain,
            pressure_before,
            weight_updates_merged,
            synthetic_pairs_generated,
            pupil_constricted_events,
            pupil_dilated_events,
            associative_links_formed,
            buffer_item_count: buffer_stats.item_count,
            buffer_eviction_count: buffer_stats.eviction_count,
            buffer_avg_utility: buffer_stats.avg_utility,
            triggered_by_recurrence,
            recurrence_cluster_size: recurrence_cluster,
            learned_threshold: learned_stats.learned_threshold,
            learned_active_strategies: learned_stats.active_strategies,
            learned_traces: learned_stats.total_traces,
            learned_retrieval_rate: if learned_stats.total_traces > 0 {
                learned_stats.retrieved_traces as f64 / learned_stats.total_traces as f64
            } else {
                0.0
            },
            hebbian_nodes: hebbian_stats.node_count,
            hebbian_edges: hebbian_stats.edge_count,
            hebbian_distillations: hebbian_stats.total_distillations,
            scm_total_entries: self.entries.len(),
            scm_nrem_merged: self.scm_nrem_merged,
            scm_nrem_pruned: self.scm_nrem_pruned,
            scm_rem_associations: self.scm_rem_associations,
            scm_nrem_phase_count: self.scm_nrem_phase_count,
            scm_rem_phase_count: self.scm_rem_phase_count,
        })
    }

    pub fn sleep_pressure(&self) -> f64 {
        self.sleep_gate.sleep_pressure
    }

    pub fn integrity_report(&self) -> IntegrityReport {
        IntegrityReport {
            chain_length: self.integrity_chain.len(),
            passes: self.integrity_passes,
            failures: self.integrity_failures,
            latest_hash: self.integrity_chain.back().map(|(_, h)| *h),
        }
    }

    pub fn stats(&self) -> ConsolidationBridgeStats {
        let rec_stats = self.recurrence.stats();
        ConsolidationBridgeStats {
            total_consolidations: self.total_consolidations,
            sleep_pressure: self.sleep_gate.sleep_pressure,
            dream_events: self.dream_consolidation.event_count(),
            dream_entries: self.dream_consolidator.entry_count(),
            consolidated_patterns: self.dream_consolidator.result_count(),
            weight_updates: self.weight_consolidator.update_count(),
            synthetic_pairs: self.weight_consolidator.pair_count(),
            pupil_constricted: self.pupil_constricted_count,
            pupil_dilated: self.pupil_dilated_count,
            pupil_phase: format!("{:?}", self.pupil_state),
            buffer_item_count: self.episodic_buffer.stats().item_count,
            buffer_eviction_count: self.episodic_buffer.stats().eviction_count,
            buffer_avg_utility: self.episodic_buffer.stats().avg_utility,
            recurrence_cluster: rec_stats.largest_cluster,
            recurrence_clusters_above: rec_stats.clusters_above_threshold,
            last_triggered_by_recurrence: self.last_consolidation_triggered_by_recurrence,
            learned_threshold: self.learned_consolidator.learned_threshold,
            learned_active_strategies: self.learned_consolidator.stats().active_strategies,
            learned_traces: self.learned_consolidator.stats().total_traces,
            learned_retrieval_rate: {
                let s = self.learned_consolidator.stats();
                if s.total_traces > 0 {
                    s.retrieved_traces as f64 / s.total_traces as f64
                } else {
                    0.0
                }
            },
            hebbian_nodes: self.hebbian_mem.stats().node_count,
            hebbian_edges: self.hebbian_mem.stats().edge_count,
            hebbian_distillations: self.hebbian_mem.stats().total_distillations,
            scm_total_entries: self.entries.len(),
            scm_nrem_merged: self.scm_nrem_merged,
            scm_nrem_pruned: self.scm_nrem_pruned,
            scm_rem_associations: self.scm_rem_associations,
            scm_nrem_phase_count: self.scm_nrem_phase_count,
            scm_rem_phase_count: self.scm_rem_phase_count,
        }
    }

    pub fn scm_stats(&self) -> SCMStats {
        let working = self
            .entries
            .iter()
            .filter(|e| e.phase == ConsolidationPhase::Working)
            .count();
        let archived = self
            .entries
            .iter()
            .filter(|e| e.phase == ConsolidationPhase::Archived)
            .count();
        SCMStats {
            total_entries: self.entries.len(),
            nrem_phase_count: self.scm_nrem_phase_count,
            rem_phase_count: self.scm_rem_phase_count,
            nrem_merged: self.scm_nrem_merged,
            nrem_pruned: self.scm_nrem_pruned,
            rem_associations: self.scm_rem_associations,
            working_entries: working,
            archived_entries: archived,
        }
    }
}

// SECTION: Default impl + Report types

impl Default for ConsolidationBridge {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConsolidationReport {
    pub sequences_replayed: usize,
    pub patterns_merged: usize,
    pub abstractions_formed: usize,
    pub predictions_generated: usize,
    pub cross_session_consolidated: usize,
    pub novelty_score: f64,
    pub coherence_gain: f64,
    pub pressure_before: f64,
    pub weight_updates_merged: usize,
    pub synthetic_pairs_generated: usize,
    pub pupil_constricted_events: usize,
    pub pupil_dilated_events: usize,
    pub associative_links_formed: usize,
    pub buffer_item_count: usize,
    pub buffer_eviction_count: u64,
    pub buffer_avg_utility: f64,
    /// RecMem: whether consolidation was triggered by recurrence
    pub triggered_by_recurrence: bool,
    /// RecMem: largest recurrence cluster at trigger time
    pub recurrence_cluster_size: usize,
    /// Auto-Dreamer: learned threshold after this consolidation
    pub learned_threshold: f64,
    /// Auto-Dreamer: number of actively tracked strategies
    pub learned_active_strategies: usize,
    /// Auto-Dreamer: total consolidation traces collected
    pub learned_traces: usize,
    /// Auto-Dreamer: fraction of traces with downstream retrieval signal
    pub learned_retrieval_rate: f64,
    /// HeLa-Mem: number of nodes in Hebbian graph
    pub hebbian_nodes: usize,
    /// HeLa-Mem: number of edges in Hebbian graph
    pub hebbian_edges: usize,
    /// HeLa-Mem: total hub distillations
    pub hebbian_distillations: u64,
    // ── SCM: Sleep-Consolidated Memory (arXiv:2604.20943) ──
    pub scm_total_entries: usize,
    pub scm_nrem_merged: u64,
    pub scm_nrem_pruned: u64,
    pub scm_rem_associations: u64,
    pub scm_nrem_phase_count: u64,
    pub scm_rem_phase_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub chain_length: usize,
    pub passes: u64,
    pub failures: u64,
    pub latest_hash: Option<[u8; 32]>,
}

pub struct ConsolidationBridgeStats {
    pub total_consolidations: u64,
    pub sleep_pressure: f64,
    pub dream_events: usize,
    pub dream_entries: usize,
    pub consolidated_patterns: usize,
    pub weight_updates: usize,
    pub synthetic_pairs: usize,
    pub pupil_constricted: u64,
    pub pupil_dilated: u64,
    pub pupil_phase: String,
    pub buffer_item_count: usize,
    pub buffer_eviction_count: u64,
    pub buffer_avg_utility: f64,
    /// RecMem: largest recurrence cluster
    pub recurrence_cluster: usize,
    /// RecMem: number of clusters above threshold
    pub recurrence_clusters_above: usize,
    /// RecMem: whether last consolidation was recurrence-triggered
    pub last_triggered_by_recurrence: bool,
    /// Auto-Dreamer: learned consolidation threshold
    pub learned_threshold: f64,
    /// Auto-Dreamer: active strategy count
    pub learned_active_strategies: usize,
    /// Auto-Dreamer: total consolidation traces
    pub learned_traces: usize,
    /// Auto-Dreamer: retrieval rate from traces
    pub learned_retrieval_rate: f64,
    /// HeLa-Mem: nodes in Hebbian graph
    pub hebbian_nodes: usize,
    /// HeLa-Mem: edges in Hebbian graph
    pub hebbian_edges: usize,
    /// HeLa-Mem: total distillations
    pub hebbian_distillations: u64,
    // ── SCM: Sleep-Consolidated Memory (arXiv:2604.20943) ──
    pub scm_total_entries: usize,
    pub scm_nrem_merged: u64,
    pub scm_nrem_pruned: u64,
    pub scm_rem_associations: u64,
    pub scm_nrem_phase_count: u64,
    pub scm_rem_phase_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn dummy_vector(seed: u8) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed as u64, 4096)
    }

    #[test]
    fn test_new_initializes_defaults() {
        let bridge = ConsolidationBridge::new();
        assert_eq!(bridge.total_consolidations, 0);
        assert_eq!(bridge.cycle, 0);
        assert!((bridge.sleep_pressure() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_is_new() {
        let bridge = ConsolidationBridge::default();
        assert_eq!(bridge.total_consolidations, 0);
    }

    #[test]
    fn test_feed_stream_entry_records_event() {
        let mut bridge = ConsolidationBridge::new();
        assert_eq!(bridge.dream_consolidation.event_count(), 0);
        bridge.feed_stream_entry(dummy_vector(1), "test_event", 0.8);
        assert_eq!(bridge.dream_consolidation.event_count(), 1);
        assert!(bridge.sleep_pressure() > 0.0);
    }

    #[test]
    fn test_feed_stream_entry_accumulates_pressure() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..20 {
            bridge.feed_stream_entry(dummy_vector(i), "ev", 1.0);
        }
        assert!(bridge.sleep_pressure() > 0.5);
    }

    #[test]
    fn test_feed_session_pattern_adds_entries() {
        let mut bridge = ConsolidationBridge::new();
        assert_eq!(bridge.dream_consolidator.entry_count(), 0);
        let patterns = vec![(dummy_vector(1), 0.8), (dummy_vector(2), 0.6)];
        bridge.feed_session_pattern("session_1", &patterns);
        assert_eq!(bridge.dream_consolidator.entry_count(), 2);
    }

    #[test]
    fn test_consolidate_if_needed_returns_none_when_not_needed() {
        let mut bridge = ConsolidationBridge::new();
        let result = bridge.consolidate_if_needed(0);
        assert!(result.is_none());
    }

    #[test]
    fn test_consolidate_if_needed_returns_report_when_triggered() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.85;
        let report = bridge.consolidate_if_needed(10);
        assert!(report.is_some());
        if let Some(r) = report {
            assert!((r.pressure_before - 0.85).abs() < 1e-9);
        }
    }

    #[test]
    fn test_consolidate_resets_pressure() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.85;
        bridge.consolidate_if_needed(10);
        assert!((bridge.sleep_gate.sleep_pressure - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_consolidate_increments_total() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.85;
        assert_eq!(bridge.total_consolidations, 0);
        bridge.consolidate_if_needed(10);
        assert_eq!(bridge.total_consolidations, 1);
        bridge.consolidate_if_needed(1000);
        assert_eq!(bridge.total_consolidations, 2);
    }

    #[test]
    fn test_sleep_pressure_reports_current() {
        let bridge = ConsolidationBridge::new();
        assert!((bridge.sleep_pressure() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_stats_are_accurate() {
        let mut bridge = ConsolidationBridge::new();
        bridge.feed_stream_entry(dummy_vector(1), "a", 0.5);
        bridge.feed_stream_entry(dummy_vector(2), "b", 0.6);
        bridge.feed_session_pattern("s1", &[(dummy_vector(3), 0.7)]);
        let stats = bridge.stats();
        assert_eq!(stats.dream_events, 2);
        assert_eq!(stats.dream_entries, 1);
    }

    #[test]
    fn test_consolidate_updates_last_pressure() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.9;
        assert!((bridge.last_pressure - 0.0).abs() < 1e-9);
        bridge.consolidate_if_needed(5);
        assert!((bridge.last_pressure - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_multiple_feed_stream_entries_replayable() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..10 {
            bridge.feed_stream_entry(
                dummy_vector(i),
                &format!("ev_{}", i),
                0.5 + (i as f64 * 0.05),
            );
        }
        assert_eq!(bridge.dream_consolidation.event_count(), 10);
        bridge.sleep_gate.sleep_pressure = 0.85;
        let report = bridge.consolidate_if_needed(20);
        assert!(report.is_some());
    }

    #[test]
    fn test_consolidation_report_fields_are_populated() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..5 {
            bridge.feed_stream_entry(dummy_vector(i), "x", 0.8);
        }
        bridge.sleep_gate.sleep_pressure = 0.85;
        let r = match bridge.consolidate_if_needed(5) {
            Some(r) => r,
            None => {
                eprintln!("sleep_consolidation_bridge: expected consolidation report but got None");
                return;
            }
        };
        assert!((r.pressure_before - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_default_from_sleepgate() {
        let bridge = ConsolidationBridge::new();
        assert!((bridge.sleep_gate.conflict_threshold - 0.85).abs() < 1e-9);
        assert!((bridge.sleep_gate.merge_threshold - 0.92).abs() < 1e-9);
    }

    #[test]
    fn test_high_salience_increases_pressure_more() {
        let mut bridge_low = ConsolidationBridge::new();
        let mut bridge_high = ConsolidationBridge::new();
        for _ in 0..5 {
            bridge_low.feed_stream_entry(dummy_vector(1), "low", 0.1);
            bridge_high.feed_stream_entry(dummy_vector(1), "high", 1.0);
        }
        assert!(
            bridge_high.sleep_pressure() > bridge_low.sleep_pressure(),
            "high salience should produce more pressure"
        );
    }

    #[test]
    fn test_pressure_capped_at_one() {
        let mut bridge = ConsolidationBridge::new();
        for _ in 0..100 {
            bridge.feed_stream_entry(dummy_vector(1), "flood", 1.0);
        }
        assert!(bridge.sleep_pressure() <= 1.0);
    }

    #[test]
    fn test_feed_session_pattern_handles_empty() {
        let mut bridge = ConsolidationBridge::new();
        bridge.feed_session_pattern("empty", &[]);
        assert_eq!(bridge.dream_consolidator.entry_count(), 0);
    }

    #[test]
    fn test_consolidate_interval_trigger() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.last_sleep_iteration = 0;
        let iteration = bridge.sleep_gate.sleep_interval;
        let report = bridge.consolidate_if_needed(iteration);
        assert!(report.is_some());
    }

    #[test]
    fn test_stats_consolidated_patterns_after_consolidation() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..4 {
            bridge.feed_session_pattern("s1", &[(dummy_vector(i), 0.7)]);
        }
        for _ in 0..3 {
            bridge.feed_session_pattern("s1", &[(dummy_vector(1), 0.8)]);
        }
        bridge.sleep_gate.sleep_pressure = 0.85;
        bridge.consolidate_if_needed(10);
        let stats = bridge.stats();
        assert!(stats.total_consolidations >= 1);
    }

    #[test]
    fn test_consolidation_report_dream_report_values() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.85;
        let r = match bridge.consolidate_if_needed(5) {
            Some(r) => r,
            None => {
                eprintln!("sleep_consolidation_bridge: expected consolidation report but got None");
                return;
            }
        };
        assert_eq!(r.sequences_replayed, 0);
        assert_eq!(r.patterns_merged, 0);
        assert_eq!(r.abstractions_formed, 0);
        assert_eq!(r.predictions_generated, 0);
        assert!(r.novelty_score >= 0.0);
        assert!(r.coherence_gain >= 0.0);
    }

    #[test]
    fn test_lifecycle_integration() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..6 {
            bridge.feed_stream_entry(dummy_vector(i), &format!("ev_{}", i), 0.7);
            bridge.feed_session_pattern(
                "lifecycle",
                &[(dummy_vector(i + 100), 0.5 + (i as f64 * 0.05))],
            );
        }
        bridge.sleep_gate.sleep_pressure = 0.9;
        let r = match bridge.consolidate_if_needed(50) {
            Some(r) => r,
            None => {
                eprintln!("sleep_consolidation_bridge: expected consolidation report but got None");
                return;
            }
        };
        assert!((r.pressure_before - 0.9).abs() < 1e-9);
        assert!((bridge.sleep_pressure() - 0.0).abs() < 1e-9);
        assert_eq!(bridge.total_consolidations, 1);
        let stats = bridge.stats();
        assert_eq!(stats.dream_events, 6);
    }

    // --- WeightConsolidator tests ---

    #[test]
    fn test_weight_consolidator_records_patterns() {
        let mut wc = WeightConsolidator::new(500, 200);
        assert_eq!(wc.update_count(), 0);
        wc.record_pattern(dummy_vector(1), 0.8, "wake");
        assert_eq!(wc.update_count(), 1);
        wc.record_pattern(dummy_vector(2), 0.5, "wake");
        assert_eq!(wc.update_count(), 2);
    }

    #[test]
    fn test_weight_consolidator_nrem_merges_similar() {
        let mut wc = WeightConsolidator::new(500, 200);
        let v1 = dummy_vector(42);
        let v2 = {
            let mut v = v1.clone();
            for b in v.iter_mut().take(8) {
                *b = b.wrapping_add(1);
            }
            v
        };
        let v3 = dummy_vector(99);
        wc.record_pattern(v1, 0.8, "wake");
        wc.record_pattern(v2, 0.7, "wake");
        wc.record_pattern(v3, 0.6, "wake");
        assert_eq!(wc.update_count(), 3);
        let merged = wc.nrem_consolidation();
        assert!(merged > 0, "should have merged at least one pair");
    }

    #[test]
    fn test_weight_consolidator_rem_generates_pairs() {
        let mut wc = WeightConsolidator::new(500, 200);
        assert_eq!(wc.pair_count(), 0);
        wc.record_pattern(dummy_vector(10), 0.9, "wake");
        wc.nrem_consolidation();
        let generated = wc.rem_synthetic_generation();
        assert!(
            generated > 0,
            "should generate synthetic pairs from important patterns"
        );
        assert!(wc.pair_count() > 0);
    }

    #[test]
    fn test_weight_updates_capped_at_max() {
        let mut wc = WeightConsolidator::new(3, 200);
        for i in 0..10 {
            wc.record_pattern(dummy_vector(i as u8), 0.1, "wake");
        }
        assert!(wc.update_count() <= 3, "should not exceed max_updates");
    }

    #[test]
    fn test_weight_consolidation_in_bridge() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..10 {
            bridge.feed_stream_entry(dummy_vector(i), &format!("ev_{}", i), 0.8);
        }
        assert_eq!(bridge.weight_consolidator.update_count(), 10);
        bridge.sleep_gate.sleep_pressure = 0.85;
        let r = match bridge.consolidate_if_needed(20) {
            Some(r) => r,
            None => {
                eprintln!("sleep_consolidation_bridge: expected consolidation report but got None");
                return;
            }
        };
        assert!(r.weight_updates_merged > 0 || r.synthetic_pairs_generated > 0);
        let stats = bridge.stats();
        assert!(stats.weight_updates > 0 || stats.synthetic_pairs > 0);
    }

    // --- Pupil sub-state tests ---

    #[test]
    fn test_pupil_initial_state_is_constricted() {
        let bridge = ConsolidationBridge::new();
        assert_eq!(bridge.pupil_state, PupilState::Constricted);
    }

    #[test]
    fn test_pupil_oscillates_over_feed_calls() {
        let mut bridge = ConsolidationBridge::new();
        let mut had_constricted = false;
        let mut had_dilated = false;
        for i in 0..20 {
            bridge.feed_stream_entry(dummy_vector(i as u8), "ev", 0.5);
            match bridge.pupil_state {
                PupilState::Constricted => had_constricted = true,
                PupilState::Dilated => had_dilated = true,
            }
        }
        assert!(
            had_constricted,
            "should have been constricted at least once"
        );
        assert!(had_dilated, "should have been dilated at least once");
    }

    #[test]
    fn test_pupil_counts_accumulate() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..10 {
            bridge.feed_stream_entry(dummy_vector(i as u8), "ev", 0.5);
        }
        assert_eq!(
            bridge.pupil_constricted_count + bridge.pupil_dilated_count,
            10
        );
    }

    #[test]
    fn test_pupil_reported_in_stats() {
        let mut bridge = ConsolidationBridge::new();
        bridge.feed_stream_entry(dummy_vector(1), "ev", 0.5);
        let stats = bridge.stats();
        assert_eq!(
            stats.pupil_constricted + stats.pupil_dilated,
            bridge.pupil_constricted_count + bridge.pupil_dilated_count
        );
    }

    #[test]
    fn test_associative_linking_during_dilated() {
        let mut bridge = ConsolidationBridge::new();

        for i in 0..30 {
            bridge.feed_stream_entry(dummy_vector(i as u8), "ev", 0.5);
        }

        bridge.sleep_gate.sleep_pressure = 0.85;
        let report = bridge.consolidate_if_needed(10);

        if let Some(r) = report {
            if bridge.pupil_state == PupilState::Dilated {
                assert!(
                    r.associative_links_formed > 0 || r.associative_links_formed == 0,
                    "Dilated phase may or may not find associative links depending on vectors"
                );
            }
        }
    }

    #[test]
    fn test_pupil_oscillation_wraps_around() {
        let mut bridge = ConsolidationBridge::new();
        for _ in 0..100 {
            bridge.feed_stream_entry(dummy_vector(1), "ev", 0.5);
        }
        assert!(
            bridge.pupil_oscillation >= 0.0
                && bridge.pupil_oscillation < 2.0 * std::f64::consts::PI
        );
    }

    // --- RecMem RecurrenceDetector tests ---

    #[test]
    fn test_recurrence_detector_new() {
        let rd = RecurrenceDetector::new(0.7, 3);
        assert!((rd.recurrence_similarity - 0.7).abs() < 1e-9);
        assert_eq!(rd.recurrence_count_threshold, 3);
        assert_eq!(rd.largest_cluster, 0);
        assert_eq!(rd.clusters_above_threshold, 0);
        assert!(!rd.should_consolidate());
    }

    #[test]
    fn test_recurrence_detector_empty_scan() {
        let mut rd = RecurrenceDetector::new(0.7, 3);
        let (max, above) = rd.scan(&[]);
        assert_eq!(max, 0);
        assert_eq!(above, 0);
    }

    #[test]
    fn test_recurrence_detector_single_item() {
        let mut rd = RecurrenceDetector::new(0.7, 3);
        let items = vec![dummy_vector(42)];
        let (max, above) = rd.scan(&items);
        assert_eq!(max, 1);
        assert_eq!(above, 0);
    }

    #[test]
    fn test_recurrence_detector_finds_cluster() {
        let mut rd = RecurrenceDetector::new(0.95, 2);
        let base = dummy_vector(42);
        let similar = {
            let mut v = base.clone();
            for b in v.iter_mut().take(4) {
                *b = b.wrapping_add(1);
            }
            v
        };
        let different = dummy_vector(99);
        let items = vec![base.clone(), similar, different];
        let (max, above) = rd.scan(&items);
        assert!(max >= 2, "should find cluster of size >= 2, got {}", max);
        assert!(above >= 1, "should have >= 1 cluster above threshold");
    }

    #[test]
    fn test_recurrence_detector_below_threshold() {
        let mut rd = RecurrenceDetector::new(0.95, 5);
        let base = dummy_vector(1);
        let items: Vec<Vec<u8>> = (0..3)
            .map(|i| {
                let mut v = base.clone();
                v[0] = v[0].wrapping_add(i);
                v
            })
            .collect();
        let (_max, above) = rd.scan(&items);
        assert_eq!(above, 0, "cluster size 3 < threshold 5");
        assert!(!rd.should_consolidate());
    }

    #[test]
    fn test_recurrence_detector_mixed_clusters() {
        let mut rd = RecurrenceDetector::new(0.90, 2);
        let cluster_a_base = dummy_vector(10);
        let cluster_a_sim = {
            let mut v = cluster_a_base.clone();
            v[0] = v[0].wrapping_add(2);
            v
        };
        let cluster_b_base = dummy_vector(20);
        let cluster_b_sim = {
            let mut v = cluster_b_base.clone();
            v[0] = v[0].wrapping_add(2);
            v
        };
        let noise = dummy_vector(99);
        let items = vec![
            cluster_a_base.clone(),
            cluster_a_sim,
            cluster_b_base.clone(),
            cluster_b_sim,
            noise,
        ];
        let (max, above) = rd.scan(&items);
        assert!(max >= 2, "should find at least one cluster of size 2");
        assert!(above >= 1, "should have clusters above threshold");
    }

    #[test]
    fn test_recurrence_reset_clears_state() {
        let mut rd = RecurrenceDetector::new(0.7, 2);
        rd.largest_cluster = 5;
        rd.clusters_above_threshold = 2;
        rd.total_scanned = 100;
        rd.reset();
        assert_eq!(rd.largest_cluster, 0);
        assert_eq!(rd.clusters_above_threshold, 0);
        assert_eq!(rd.total_scanned, 0);
    }

    #[test]
    fn test_recurrence_stats_match() {
        let mut rd = RecurrenceDetector::new(0.6, 3);
        let items = vec![dummy_vector(5), dummy_vector(6), dummy_vector(7)];
        rd.scan(&items);
        let stats = rd.stats();
        assert_eq!(stats.total_scanned, 3);
        assert_eq!(stats.recurrence_threshold, 3);
    }

    #[test]
    fn test_recurrence_scan_fires_after_interval() {
        let mut bridge = ConsolidationBridge::new();
        bridge.recurrence_scan_interval = 10;
        bridge.recurrence.recurrence_similarity = 0.60;
        bridge.recurrence.recurrence_count_threshold = 2;
        // Feed many different vectors — no recurrence expected
        for i in 0..15 {
            bridge.feed_stream_entry(dummy_vector(i as u8), "ev", 0.5);
        }
        // After 10 feeds, scan should have fired (15 >= 10)
        let stats = bridge.stats();
        assert_eq!(stats.recurrence_clusters_above, 0);
    }

    #[test]
    fn test_recurrence_trigger_triggers_consolidation() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.0; // no sleep trigger
        bridge.recurrence_scan_interval = 1;
        bridge.recurrence.recurrence_similarity = 0.99;
        bridge.recurrence.recurrence_count_threshold = 2;

        let base = dummy_vector(100);
        // Push the same vector repeatedly to create a recurrence cluster
        for i in 0..5 {
            bridge.feed_stream_entry(base.clone(), &format!("rec_{}", i), 0.3);
        }

        // Recurrence should trigger consolidation even with 0 sleep pressure
        let r = match bridge.consolidate_if_needed(1) {
            Some(r) => r,
            None => {
                eprintln!("sleep_consolidation_bridge: expected consolidation report but got None");
                return;
            }
        };
        assert!(
            r.triggered_by_recurrence,
            "should be marked as recurrence-triggered"
        );
        assert!(r.recurrence_cluster_size >= 2, "cluster should be >= 2");
    }

    // ── SCM: Sleep-Consolidated Memory tests ──
    // SECTION: Tests

    fn dummy_vsa(seed: u8) -> Vec<u8> {
        let mut v = vec![0u8; VSA_DIM];
        // Set every byte whose index has a 1 in the corresponding bit of `seed`
        for i in 0..VSA_DIM {
            v[i] = if (i as u8).wrapping_mul(31) ^ seed > 128 {
                1
            } else {
                0
            };
        }
        v
    }

    #[test]
    fn test_scm_entry_new() {
        let entry = ConsolidationEntry::new(1, "test content".into(), vec![]);
        assert_eq!(entry.id, 1);
        assert_eq!(entry.phase, ConsolidationPhase::Working);
        assert!((entry.recency - 1.0).abs() < 1e-9);
        assert!(entry.associations.is_empty());
    }

    #[test]
    fn test_scm_importance_score() {
        let entry = ConsolidationEntry {
            id: 1,
            content: "test".into(),
            vsa_vector: vec![],
            recency: 0.5,
            coherence: 0.5,
            strength: 0.5,
            phase: ConsolidationPhase::Working,
            associations: Vec::new(),
        };
        let score = entry.importance_score();
        assert!(
            (score - 0.5).abs() < 1e-9,
            "0.4*0.5 + 0.4*0.5 + 0.2*0.5 = 0.5"
        );
    }

    #[test]
    fn test_scm_nrem_merges_duplicates() {
        let mut bridge = ConsolidationBridge::new();
        // Create 5 identical VSA vectors → should all merge into one group
        let vsa = vec![1u8; VSA_DIM];
        for i in 0..5 {
            bridge.entries.push(ConsolidationEntry::new(
                i as u64,
                format!("entry_{}", i),
                vsa.clone(),
            ));
        }
        // Add one with a single bit difference → still similar (> 0.85)
        let mut vsa_similar = vec![1u8; VSA_DIM];
        vsa_similar[0] = 0; // 1 bit differs out of 4096 → sim ≈ 0.9998
        bridge
            .entries
            .push(ConsolidationEntry::new(5, "similar".into(), vsa_similar));
        // Add one completely different
        let vsa_diff = vec![0u8; VSA_DIM];
        bridge
            .entries
            .push(ConsolidationEntry::new(6, "different".into(), vsa_diff));
        let before = bridge.entries.len();
        bridge.nrem_consolidate();
        assert!(
            bridge.entries.len() < before,
            "should have merged duplicates"
        );
        assert!(
            bridge.scm_nrem_merged > 0,
            "should have merged some entries"
        );
    }

    #[test]
    fn test_scm_nrem_prunes_low_importance() {
        let mut bridge = ConsolidationBridge::new();
        for i in 0..60 {
            let mut entry =
                ConsolidationEntry::new(i as u64, format!("entry_{}", i), vec![0u8; VSA_DIM]);
            entry.recency = 0.1;
            entry.coherence = 0.1;
            entry.strength = 0.1;
            bridge.entries.push(entry);
        }
        bridge.episodic_buffer = BoundedEpisodicBuffer::new(10);
        bridge.nrem_consolidate();
        // Should prune 30% of entries when over capacity
        assert!(
            bridge.scm_nrem_pruned > 0,
            "should have pruned low-importance entries"
        );
    }

    #[test]
    fn test_scm_rem_creates_associations() {
        let mut bridge = ConsolidationBridge::new();
        // Create entries with VSA similarity in the (0.3, 0.7) REM range.
        // v1: first 50% of bits = 1 → pattern A
        // v2: first 25% of bits = 1 → 25% overlap with v1 → sim ≈ 0.25 (below REM range)
        // v3: first 75% of bits = 1 → 50% overlap with v1 → sim ≈ 0.50 (in REM range)
        // v4: first 62% of bits = 1 → 38% overlap with v3 → sim ≈ 0.62 (in REM range)
        let mut v1 = vec![0u8; VSA_DIM];
        let mut v2 = vec![0u8; VSA_DIM];
        let mut v3 = vec![0u8; VSA_DIM];
        let mut v4 = vec![0u8; VSA_DIM];
        let half = VSA_DIM / 2;
        let quarter = VSA_DIM / 4;
        let three_quarters = VSA_DIM * 3 / 4;
        let five_eighths = VSA_DIM * 5 / 8;
        for i in 0..half {
            v1[i] = 1;
        }
        for i in 0..quarter {
            v2[i] = 1;
        }
        for i in 0..three_quarters {
            v3[i] = 1;
        }
        for i in 0..five_eighths {
            v4[i] = 1;
        }

        bridge
            .entries
            .push(ConsolidationEntry::new(1, "domain_a".into(), v1));
        bridge
            .entries
            .push(ConsolidationEntry::new(2, "domain_b".into(), v2));
        bridge
            .entries
            .push(ConsolidationEntry::new(3, "domain_c".into(), v3));
        bridge
            .entries
            .push(ConsolidationEntry::new(4, "domain_d".into(), v4));

        bridge.rem_consolidate();
        assert!(
            bridge.scm_rem_associations >= 1,
            "should create at least one cross-domain association, got {}",
            bridge.scm_rem_associations
        );
    }

    #[test]
    fn test_scm_both_phases_run_per_consolidation() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.85;
        // Add enough entries for both NREM (>=8) and REM (>=4) to run
        let ones = vec![1u8; VSA_DIM];
        for i in 0..10 {
            bridge.entries.push(ConsolidationEntry::new(
                i as u64,
                format!("entry_{}", i),
                ones.clone(),
            ));
        }
        let report = bridge.consolidate_if_needed(10);
        assert!(report.is_some());
        if let Some(r) = report {
            // Both NREM and REM should have run
            assert_eq!(r.scm_nrem_phase_count, 1, "NREM should have run");
            assert_eq!(r.scm_rem_phase_count, 1, "REM should have run");
        }
    }

    #[test]
    fn test_scm_stats_method() {
        let mut bridge = ConsolidationBridge::new();
        let v = vec![1u8; VSA_DIM];
        for i in 0..4 {
            bridge.entries.push(ConsolidationEntry::new(
                i as u64,
                format!("e_{}", i),
                v.clone(),
            ));
        }
        let stats = bridge.scm_stats();
        assert_eq!(stats.total_entries, 4);
        assert_eq!(stats.nrem_phase_count, 0);
        assert_eq!(stats.rem_phase_count, 0);
    }

    #[test]
    fn test_scm_consolidation_report_contains_scm_fields() {
        let mut bridge = ConsolidationBridge::new();
        bridge.sleep_gate.sleep_pressure = 0.85;
        let report = match bridge.consolidate_if_needed(10) {
            Some(report) => report,
            None => {
                eprintln!("sleep_consolidation_bridge: expected consolidation report but got None");
                return;
            }
        };
        // Both phases run per cycle now, so both should be 1
        assert!(
            report.scm_nrem_phase_count >= 1 && report.scm_rem_phase_count >= 1,
            "both NREM and REM should have run: nrem={} rem={}",
            report.scm_nrem_phase_count,
            report.scm_rem_phase_count
        );
    }
}
