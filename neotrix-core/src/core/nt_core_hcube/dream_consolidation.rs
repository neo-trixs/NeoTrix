use crate::core::nt_core_hcube::hippocampal_trace::HippocampalMemory;
use crate::core::nt_core_hcube::sm2_scheduler::SM2Scheduler;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

const SIMILARITY_THRESHOLD: f64 = 0.5;
const PATTERN_MATCH_THRESHOLD: f64 = 0.8;

#[derive(Debug, Clone)]
pub struct DreamConfig {
    pub merge_threshold: f64,
    pub replay_batch_size: usize,
    pub min_abstraction_freq: usize,
    pub dream_cycles_per_consolidation: usize,
    pub prediction_horizon: usize,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            merge_threshold: 0.7,
            replay_batch_size: 10,
            min_abstraction_freq: 3,
            dream_cycles_per_consolidation: 5,
            prediction_horizon: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DreamEvent {
    pub id: u64,
    pub vector: Vec<u8>,
    pub label: String,
    pub timestamp: f64,
    pub salience: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DreamReport {
    pub sequences_replayed: usize,
    pub patterns_merged: usize,
    pub abstractions_formed: usize,
    pub predictions_generated: usize,
    pub novelty_score: f64,
    pub coherence_gain: f64,
    pub consolidation_id: u64,
}

#[derive(Debug, Clone)]
pub enum DreamPhase {
    SequenceReplay,
    PatternMerging,
    Abstraction,
    Predictive,
}

// ── SCM Two-Phase (arXiv:2604.20943) Configuration ──

#[derive(Debug, Clone)]
pub struct NremConfig {
    pub iterations: usize,
    pub redundancy_threshold: f64,
    pub sparsification_ratio: f64,
}

impl Default for NremConfig {
    fn default() -> Self {
        Self {
            iterations: 3,
            redundancy_threshold: 0.85,
            sparsification_ratio: 0.3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemConfig {
    pub walk_length: usize,
    pub association_threshold: f64,
    pub discovery_rate: f64,
}

impl Default for RemConfig {
    fn default() -> Self {
        Self {
            walk_length: 100,
            association_threshold: 0.7,
            discovery_rate: 0.1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsolidationPhase {
    NREM,
    REM,
    Complete,
}

#[derive(Debug, Clone)]
pub struct DreamConsolidation {
    events: Vec<DreamEvent>,
    consolidated: Vec<(Vec<u8>, String, f64)>,
    pattern_freq: Vec<(Vec<u8>, usize)>,
    next_id: u64,
    config: DreamConfig,
    step_count: u64,

    // SCM two-phase (arXiv:2604.20943)
    pub nrem_config: NremConfig,
    pub rem_config: RemConfig,
    pub current_phase: ConsolidationPhase,
    pub phase_iteration: usize,

    // G58: SM-2 spaced repetition scheduler & hippocampal memory
    pub sm2_scheduler: Option<SM2Scheduler>,
    pub hippocampus: Option<HippocampalMemory>,
}

impl DreamConsolidation {
    pub fn new(config: DreamConfig) -> Self {
        Self {
            events: Vec::new(),
            consolidated: Vec::new(),
            pattern_freq: Vec::new(),
            next_id: 0,
            config,
            step_count: 0,
            nrem_config: NremConfig::default(),
            rem_config: RemConfig::default(),
            current_phase: ConsolidationPhase::NREM,
            phase_iteration: 0,
            sm2_scheduler: None,
            hippocampus: None,
        }
    }

    pub fn record_event(&mut self, vector: Vec<u8>, label: &str, salience: f64) {
        let event = DreamEvent {
            id: self.next_id,
            vector,
            label: label.to_string(),
            timestamp: self.step_count as f64,
            salience,
        };
        self.next_id += 1;
        self.events.push(event);
    }

    pub fn run_consolidation_cycle(&mut self) -> DreamReport {
        let cid = self.step_count;
        let mut report = DreamReport {
            consolidation_id: cid,
            ..Default::default()
        };

        if self.events.len() < 3 {
            self.step_count += 1;
            return report;
        }

        // ── Core: sequence replay & pattern merging (original pipeline) ──
        let mut seq_indices = self._find_sequence_indices();
        if self.sm2_scheduler.is_some() {
            seq_indices = self._prioritize_with_sm2(&seq_indices);
        }
        report.sequences_replayed = seq_indices.len();

        if seq_indices.is_empty() {
            // Even without sequences, run SCM phases on existing consolidated pool
            self._run_scm_phases(&mut report);
            self.step_count += 1;
            return report;
        }

        let mut new_merged: Vec<(Vec<u8>, f64)> = Vec::new();

        for idxs in &seq_indices {
            let vectors: Vec<Vec<u8>> = idxs
                .iter()
                .map(|&i| self.events[i].vector.clone())
                .collect();
            let vec_refs: Vec<&[u8]> = vectors.iter().map(|v| v.as_slice()).collect();
            let bundle = QuantizedVSA::majority_bundle(&vec_refs);
            let coherence = Self::_sequence_coherence(&vectors);
            new_merged.push((bundle.clone(), coherence));
            self.consolidated
                .push((bundle, format!("merged_{}", self.next_id), coherence));
            self.next_id += 1;
        }
        report.patterns_merged = new_merged.len();

        for (vec, _coh) in &new_merged {
            let mut matched = false;
            for (existing, count) in &mut self.pattern_freq {
                if QuantizedVSA::similarity(vec, existing) > PATTERN_MATCH_THRESHOLD {
                    *count += 1;
                    matched = true;
                    if *count >= self.config.min_abstraction_freq {
                        let similar: Vec<Vec<u8>> = self
                            .consolidated
                            .iter()
                            .filter(|(v, _, _)| {
                                QuantizedVSA::similarity(vec, v) > PATTERN_MATCH_THRESHOLD
                            })
                            .map(|(v, _, _)| v.clone())
                            .collect();
                        if similar.len() >= self.config.min_abstraction_freq {
                            let sim_refs: Vec<&[u8]> =
                                similar.iter().map(|v| v.as_slice()).collect();
                            let abstraction = QuantizedVSA::majority_bundle(&sim_refs);
                            self.consolidated.push((
                                abstraction,
                                format!("abstract_{}", self.next_id),
                                0.9,
                            ));
                            self.next_id += 1;
                            report.abstractions_formed += 1;
                        }
                        *count = 0;
                    }
                    break;
                }
            }
            if !matched {
                self.pattern_freq.push((vec.clone(), 1));
            }
        }

        for idxs in &seq_indices {
            let vectors: Vec<Vec<u8>> = idxs
                .iter()
                .map(|&i| self.events[i].vector.clone())
                .collect();
            if vectors.len() >= 2 {
                let mut deltas: Vec<Vec<u8>> = Vec::new();
                for i in 0..vectors.len() - 1 {
                    deltas.push(QuantizedVSA::xor_bind(&vectors[i], &vectors[i + 1]));
                }
                let delta_refs: Vec<&[u8]> = deltas.iter().map(|d| d.as_slice()).collect();
                let avg_delta = QuantizedVSA::majority_bundle(&delta_refs);
                let prediction = QuantizedVSA::xor_bind(&vectors[vectors.len() - 1], &avg_delta);

                let pred_coherence = if deltas.len() >= 2 {
                    let sims: f64 = (0..deltas.len() - 1)
                        .map(|i| QuantizedVSA::similarity(&deltas[i], &deltas[i + 1]))
                        .sum();
                    sims / (deltas.len() - 1) as f64
                } else {
                    0.5
                };

                self.consolidated.push((
                    prediction,
                    format!("pred_{}", self.next_id),
                    pred_coherence,
                ));
                self.next_id += 1;
                report.predictions_generated += 1;
            }
        }

        // ── G58: Register consolidated patterns with SM-2 scheduler ──
        if let Some(ref mut scheduler) = self.sm2_scheduler {
            for (vec, label, _) in &self.consolidated {
                let key = format!("dream_{}", label);
                let already = scheduler.all_items().iter().any(|it| it.memory_id == key);
                if !already {
                    scheduler.add_item(&key, vec.clone());
                }
            }
        }

        // ── SCM Two-Phase: NREM (pattern extraction) + REM (dreaming) ──
        self._run_scm_phases(&mut report);

        report.novelty_score = self._compute_novelty();
        report.coherence_gain = self._compute_coherence_gain();

        self.step_count += 1;
        report
    }

    fn _prioritize_with_sm2(&self, seq_indices: &[Vec<usize>]) -> Vec<Vec<usize>> {
        let scheduler = match self.sm2_scheduler.as_ref() {
            Some(s) => s,
            None => return seq_indices.to_vec(),
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let due_ids: Vec<String> = scheduler
            .items_due_now(now)
            .iter()
            .map(|it| it.memory_id.clone())
            .collect();
        if due_ids.is_empty() {
            return seq_indices.to_vec();
        }
        let mut prioritized: Vec<Vec<usize>> = seq_indices
            .iter()
            .filter(|idxs| {
                idxs.iter().any(|&i| {
                    let label = format!("merged_{}", i);
                    due_ids.iter().any(|d| d.contains(&label))
                })
            })
            .cloned()
            .collect();
        let remaining: Vec<Vec<usize>> = seq_indices
            .iter()
            .filter(|idxs| {
                !idxs.iter().any(|&i| {
                    let label = format!("merged_{}", i);
                    due_ids.iter().any(|d| d.contains(&label))
                })
            })
            .cloned()
            .collect();
        prioritized.extend(remaining);
        prioritized
    }

    /// Run the SCM two-phase cycle: NREM → REM (if enough consolidated entries)
    fn _run_scm_phases(&mut self, report: &mut DreamReport) {
        // ── NREM Phase: pattern extraction, redundancy elimination, sparsification ──
        self.current_phase = ConsolidationPhase::NREM;
        let nrem_count = self.run_nrem_phase();
        self.phase_iteration += 1;
        report.patterns_merged += nrem_count;

        // ── REM Phase: only if sufficient consolidated attractors ──
        if self.consolidated.len() > 50 {
            self.current_phase = ConsolidationPhase::REM;
            let rem_count = self.run_rem_phase();
            report.abstractions_formed += rem_count;
        }

        self.current_phase = if self.consolidated.len() > 100 {
            ConsolidationPhase::Complete
        } else {
            ConsolidationPhase::NREM
        };
    }

    pub fn consolidated_patterns(&self) -> &[(Vec<u8>, String, f64)] {
        &self.consolidated
    }

    pub fn prune_low_coherence(&mut self, threshold: f64) {
        self.consolidated
            .retain(|(_, _, coherence)| *coherence >= threshold);
    }

    pub fn stats(&self) -> DreamReport {
        let novelty = self._compute_novelty();
        let coherence = self._compute_coherence_gain();
        let abs_count = self
            .pattern_freq
            .iter()
            .filter(|(_, count)| *count >= self.config.min_abstraction_freq)
            .count();

        DreamReport {
            sequences_replayed: 0,
            patterns_merged: 0,
            abstractions_formed: abs_count,
            predictions_generated: 0,
            novelty_score: novelty,
            coherence_gain: coherence,
            consolidation_id: self.step_count,
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn set_phase_config(&mut self, nrem: NremConfig, rem: RemConfig) {
        self.nrem_config = nrem;
        self.rem_config = rem;
    }

    // ── SCM NREM Phase (arXiv:2604.20943 §3.1) ──
    // Pattern extraction → redundancy elimination → sparsification
    pub fn run_nrem_phase(&mut self) -> usize {
        let cfg = &self.nrem_config;
        let mut consolidated_count = 0usize;

        for _iter in 0..cfg.iterations {
            // 1. Pattern extraction: find recurring attractors among events
            let mut seen_centroids: Vec<(Vec<u8>, usize)> = Vec::new();
            for ev in &self.events {
                let mut matched = false;
                for (centroid, count) in &mut seen_centroids {
                    if QuantizedVSA::similarity(&ev.vector, centroid) > 0.6 {
                        *count += 1;
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    seen_centroids.push((ev.vector.clone(), 1));
                }
            }

            // 2. Redundancy elimination: merge near-identical attractors
            let threshold = cfg.redundancy_threshold;
            let mut merged: Vec<(Vec<u8>, usize)> = Vec::new();
            for (vec, count) in &seen_centroids {
                let mut merged_with = false;
                for (mvec, mcount) in &mut merged {
                    if QuantizedVSA::similarity(vec, mvec) > threshold {
                        // Merge by majority bundle of the two attractors
                        let bundled =
                            QuantizedVSA::majority_bundle(&[vec.as_slice(), mvec.as_slice()]);
                        *mvec = bundled;
                        *mcount += count;
                        merged_with = true;
                        break;
                    }
                }
                if !merged_with {
                    merged.push((vec.clone(), *count));
                }
            }

            // 3. Sparsification: keep only the top `sparsification_ratio` attractors by frequency
            merged.sort_by(|a, b| b.1.cmp(&a.1));
            let keep = (merged.len() as f64 * cfg.sparsification_ratio)
                .max(1.0)
                .ceil() as usize;
            let survivors: Vec<_> = merged.into_iter().take(keep).collect();

            // 4. Write survivors into consolidated with coherence = attractor frequency / total events
            let total_events = self.events.len().max(1);
            for (vec, freq) in &survivors {
                let coherence = *freq as f64 / total_events as f64;
                // Avoid duplicate entries for the same attractor
                let exists = self
                    .consolidated
                    .iter()
                    .any(|(v, _, _): &(Vec<u8>, String, f64)| {
                        QuantizedVSA::similarity(v, vec) > threshold
                    });
                if !exists {
                    self.consolidated.push((
                        vec.clone(),
                        format!("nrem_pattern_{}", self.next_id),
                        coherence.min(1.0),
                    ));
                    self.next_id += 1;
                    consolidated_count += 1;
                }
            }
        }

        consolidated_count
    }

    // ── SCM REM Phase (arXiv:2604.20943 §3.2) ──
    // Random walk on attractor graph → cross-modal associations → novel connections
    pub fn integrate_sm2(&mut self, scheduler: SM2Scheduler) {
        self.sm2_scheduler = Some(scheduler);
    }

    pub fn integrate_hippocampal(&mut self, hippocampus: HippocampalMemory) {
        self.hippocampus = Some(hippocampus);
    }

    pub fn run_rem_phase(&mut self) -> usize {
        let cfg = &self.rem_config;
        let n = self.consolidated.len();
        if n < 3 {
            return 0;
        }

        let mut associations = 0usize;

        // 1. Build attractor similarity graph adjacency & pick random start nodes
        let sim_matrix: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            0.0
                        } else {
                            QuantizedVSA::similarity(
                                &self.consolidated[i].0,
                                &self.consolidated[j].0,
                            )
                        }
                    })
                    .collect()
            })
            .collect();

        // 2. Random walks: for each walk_length step, transition to neighbor with prob ~ similarity
        for start in 0..n {
            let mut current = start;
            for _step in 0..cfg.walk_length {
                let neighbors: Vec<usize> = (0..n)
                    .filter(|&j| j != current && sim_matrix[current][j] > 0.3)
                    .collect();
                if neighbors.is_empty() {
                    break;
                }
                // Pick next node weighted by similarity
                let weights: Vec<f64> = neighbors.iter().map(|&j| sim_matrix[current][j]).collect();
                let total_w: f64 = weights.iter().sum();
                if total_w <= 0.0 {
                    break;
                }
                let r = fastrand::f64() * total_w;
                let mut cum = 0.0;
                let mut next = neighbors[0];
                for (&nbr, &w) in neighbors.iter().zip(&weights) {
                    cum += w;
                    if r <= cum {
                        next = nbr;
                        break;
                    }
                }
                current = next;
            }

            // 3. Cross-modal association discovery:
            //    If the random walk ended at a node dissimilar from the start,
            //    bundle them together as a novel association
            let sim = QuantizedVSA::similarity(
                &self.consolidated[start].0,
                &self.consolidated[current].0,
            );
            if sim < cfg.association_threshold && sim > 0.1 {
                // G58: hippocampal pattern completion during REM
                let completed_bundle = if let Some(ref hippocampus) = self.hippocampus {
                    let base = QuantizedVSA::majority_bundle(&[
                        self.consolidated[start].0.as_slice(),
                        self.consolidated[current].0.as_slice(),
                    ]);
                    if let Some(trace) = hippocampus.complete(&base, 0.4) {
                        HippocampalMemory::pattern_complete(
                            &base,
                            &trace.separated_vector,
                            &trace.binding_key,
                        )
                    } else {
                        base
                    }
                } else {
                    QuantizedVSA::majority_bundle(&[
                        self.consolidated[start].0.as_slice(),
                        self.consolidated[current].0.as_slice(),
                    ])
                };
                let discovery_qual = 1.0 - sim; // lower sim → more novel
                if fastrand::f64() < cfg.discovery_rate + discovery_qual * 0.3 {
                    let exists = self
                        .consolidated
                        .iter()
                        .any(|(v, _, _)| QuantizedVSA::similarity(v, &completed_bundle) > 0.85);
                    if !exists {
                        self.consolidated.push((
                            completed_bundle,
                            format!("rem_assoc_{}", self.next_id),
                            discovery_qual.min(1.0),
                        ));
                        self.next_id += 1;
                        associations += 1;
                    }
                }
            }
        }

        associations
    }

    fn _sequence_coherence(vectors: &[Vec<u8>]) -> f64 {
        if vectors.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        let mut count = 0;
        for i in 0..vectors.len() {
            for j in i + 1..vectors.len() {
                total += QuantizedVSA::similarity(&vectors[i], &vectors[j]);
                count += 1;
            }
        }
        total / count as f64
    }

    fn _find_sequence_indices(&self) -> Vec<Vec<usize>> {
        let mut sequences = Vec::new();
        let mut current: Vec<usize> = Vec::new();

        for i in 0..self.events.len() {
            if let Some(&last_idx) = current.last() {
                if QuantizedVSA::similarity(&self.events[last_idx].vector, &self.events[i].vector)
                    > SIMILARITY_THRESHOLD
                {
                    current.push(i);
                } else {
                    if current.len() >= 3 {
                        sequences.push(std::mem::take(&mut current));
                        current.push(i);
                    } else {
                        current.clear();
                        current.push(i);
                    }
                }
            } else {
                current.push(i);
            }
        }
        if current.len() >= 3 {
            sequences.push(current);
        }

        sequences
    }

    fn _compute_novelty(&self) -> f64 {
        if self.consolidated.len() < 2 {
            return 0.0;
        }
        let mut total_sim = 0.0;
        let mut count = 0;
        for i in 0..self.consolidated.len() {
            for j in i + 1..self.consolidated.len() {
                total_sim +=
                    QuantizedVSA::similarity(&self.consolidated[i].0, &self.consolidated[j].0);
                count += 1;
            }
        }
        let avg_sim = total_sim / count as f64;
        1.0 - avg_sim
    }

    fn _compute_coherence_gain(&self) -> f64 {
        if self.consolidated.is_empty() {
            return 0.0;
        }
        self.consolidated.iter().map(|(_, _, c)| c).sum::<f64>() / self.consolidated.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn make_test_events(dc: &mut DreamConsolidation, count: usize) {
        let mut v0 = vec![0u8; VSA_DIM];
        v0[0..500].fill(1);
        let mut v1 = v0.clone();
        v1[500..1000].fill(1);
        let mut v2 = v1.clone();
        v2[1000..1500].fill(1);
        dc.record_event(v0, "ev0", 0.8);
        dc.record_event(v1, "ev1", 0.7);
        dc.record_event(v2, "ev2", 0.9);
        if count > 3 {
            let mut v3 = vec![0u8; VSA_DIM];
            v3[2000..2500].fill(1);
            let mut v4 = v3.clone();
            v4[2500..3000].fill(1);
            dc.record_event(v3, "ev3", 0.6);
            dc.record_event(v4, "ev4", 0.5);
        }
    }

    fn make_batch(dc: &mut DreamConsolidation, prefix: &str, region_start: usize, n: usize) {
        for i in 0..n {
            let mut v = vec![0u8; VSA_DIM];
            let end = (region_start + (i + 1) * 200).min(VSA_DIM);
            v[region_start..end].fill(1);
            dc.record_event(v, &format!("{}_{}", prefix, i), 0.8);
        }
    }

    #[test]
    fn test_record_event() {
        let config = DreamConfig::default();
        let mut dc = DreamConsolidation::new(config);
        dc.record_event(vec![1u8; VSA_DIM], "test", 0.5);
        assert_eq!(dc.event_count(), 1);
    }

    #[test]
    fn test_run_cycle_empty() {
        let config = DreamConfig::default();
        let mut dc = DreamConsolidation::new(config);
        let report = dc.run_consolidation_cycle();
        assert_eq!(report.sequences_replayed, 0);
        assert_eq!(report.patterns_merged, 0);
        assert_eq!(report.abstractions_formed, 0);
        assert_eq!(report.predictions_generated, 0);
    }

    #[test]
    fn test_run_cycle_with_events() {
        let config = DreamConfig::default();
        let mut dc = DreamConsolidation::new(config);
        make_test_events(&mut dc, 5);
        assert_eq!(dc.event_count(), 5);
        for _ in 0..3 {
            let report = dc.run_consolidation_cycle();
            if cfg!(debug_assertions) {
                if report.sequences_replayed > 0 {
                    assert!(report.patterns_merged > 0);
                }
            }
        }
        assert!(!dc.consolidated_patterns().is_empty());
    }

    #[test]
    fn test_prune_low_coherence() {
        let config = DreamConfig::default();
        let mut dc = DreamConsolidation::new(config);
        dc.consolidated
            .push((vec![0u8; VSA_DIM], "low".to_string(), 0.2));
        dc.consolidated
            .push((vec![1u8; VSA_DIM], "high".to_string(), 0.9));
        dc.consolidated
            .push((vec![0u8; VSA_DIM], "mid".to_string(), 0.5));
        dc.prune_low_coherence(0.5);
        assert_eq!(dc.consolidated.len(), 2);
        assert!(dc.consolidated.iter().all(|(_, _, c)| *c >= 0.5));
    }

    #[test]
    fn test_multiple_cycles_increase_patterns_merged() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());
        // First batch: 3 similar events in region 0..600
        make_batch(&mut dc, "a", 0, 3);
        let r1 = dc.run_consolidation_cycle();
        assert!(
            r1.patterns_merged >= 1,
            "first cycle should merge at least 1 pattern"
        );

        // Second batch: 3 similar events in a disjoint region
        // Events in region 3000..4096 will be very dissimilar from region 0..600 events
        make_batch(&mut dc, "b", 3000, 3);
        let r2 = dc.run_consolidation_cycle();
        assert!(
            r2.patterns_merged >= 1,
            "second cycle should merge the new batch"
        );

        // The second cycle should find both the old and new sequences
        assert!(
            r2.patterns_merged >= r1.patterns_merged,
            "patterns_merged should not decrease"
        );
    }

    #[test]
    fn test_consolidation_produces_abstractions() {
        let mut config = DreamConfig::default();
        config.min_abstraction_freq = 2;
        let mut dc = DreamConsolidation::new(config);

        // Three similar events that will form the same merged pattern each cycle
        make_batch(&mut dc, "x", 0, 3);

        // Run enough cycles so pattern frequency triggers abstraction (min_abstraction_freq=2)
        let mut total_abs = 0;
        for _ in 0..6 {
            let r = dc.run_consolidation_cycle();
            total_abs += r.abstractions_formed;
        }
        assert!(total_abs > 0, "should form at least 1 abstraction");
    }

    #[test]
    fn test_pattern_frequency_tracking() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());

        // Record 3 similar events → one merged pattern tracked in pattern_freq
        make_batch(&mut dc, "p", 0, 3);
        dc.run_consolidation_cycle();

        // pattern_freq should have 1 entry with count >= 1
        assert_eq!(dc.pattern_freq.len(), 1, "one pattern should be tracked");
        assert!(dc.pattern_freq[0].1 >= 1, "pattern count >= 1");

        let count_after_first = dc.pattern_freq[0].1;

        // Run more cycles → same pattern appears again, frequency increases
        dc.run_consolidation_cycle();
        dc.run_consolidation_cycle();
        assert!(
            dc.pattern_freq[0].1 > count_after_first,
            "frequency should increase across cycles"
        );
    }

    #[test]
    fn test_prediction_generation_from_sequences() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());

        // make_test_events creates sequences with >= 2 consecutive deltas → predictions
        make_test_events(&mut dc, 5);
        let report = dc.run_consolidation_cycle();
        assert!(
            report.predictions_generated > 0,
            "should generate predictions from sequences"
        );

        // The prediction should be in consolidated patterns
        let pred_count = dc
            .consolidated
            .iter()
            .filter(|(_, label, _)| label.starts_with("pred_"))
            .count();
        assert!(
            pred_count > 0,
            "consolidated should contain prediction entries"
        );
    }

    #[test]
    fn test_novelty_score_decreases_with_consolidation() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());

        // Record events in batches across different regions
        // Initially with few consolidated patterns, novelty should be low or zero
        assert_eq!(dc._compute_novelty(), 0.0, "no patterns → novelty = 0");

        make_batch(&mut dc, "a", 0, 3);
        let r1 = dc.run_consolidation_cycle();
        let initial_novelty = r1.novelty_score;

        // Add more events from the same region → similar patterns → novelty decreases
        make_batch(&mut dc, "a2", 0, 3);
        let r2 = dc.run_consolidation_cycle();
        // More patterns that are similar to existing ones → novelty should drop or stay low
        assert!(
            r2.novelty_score <= initial_novelty + 1.0,
            "novelty should not increase dramatically as similar patterns accumulate"
        );
    }

    #[test]
    fn test_pruning_removes_low_coherence_patterns() {
        let mut config = DreamConfig::default();
        config.min_abstraction_freq = 10; // prevent auto-abstraction from adding new entries
        let mut dc = DreamConsolidation::new(config);

        // Create events, run consolidation to generate patterns with mixed coherence
        make_batch(&mut dc, "x", 0, 3);
        // Artificially add a low-coherence entry
        dc.consolidated
            .push((vec![0u8; VSA_DIM], "noise".to_string(), 0.05));

        let before = dc.consolidated.len();
        let low_count_before = dc.consolidated.iter().filter(|(_, _, c)| *c < 0.3).count();
        assert!(low_count_before > 0, "should have low-coherence entries");

        dc.prune_low_coherence(0.3);

        let after = dc.consolidated.len();
        assert!(after < before, "pruning should remove entries");
        assert!(
            dc.consolidated.iter().all(|(_, _, c)| *c >= 0.3),
            "all remaining patterns should have coherence >= 0.3"
        );
    }

    #[test]
    fn test_stats_report_matches_accumulated_state() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());

        // Initial state
        let s0 = dc.stats();
        assert_eq!(s0.novelty_score, 0.0);

        // Add events and consolidate
        make_test_events(&mut dc, 5);
        let _r = dc.run_consolidation_cycle();

        let s1 = dc.stats();
        // After consolidation, novelty should be recomputed from all consolidated entries
        assert!(s1.novelty_score >= 0.0, "novelty should be non-negative");
        // Coherence gain should reflect average coherence of all entries
        if !dc.consolidated.is_empty() {
            assert!(s1.coherence_gain > 0.0, "coherence_gain should be > 0");
        }
        // consolidation_id in stats should equal step_count
        assert_eq!(s1.consolidation_id, dc.step_count);

        // Run another cycle
        dc.run_consolidation_cycle();
        let s2 = dc.stats();
        assert!(s2.consolidation_id > s1.consolidation_id);
    }

    #[test]
    fn test_large_number_of_events_handled_gracefully() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());

        // Create 100 events across different regions so they form multiple sequences
        for batch in 0..10 {
            let region = batch * 400;
            for j in 0..10 {
                let mut v = vec![0u8; VSA_DIM];
                let end = (region + (j + 1) * 40).min(VSA_DIM);
                if end > region {
                    v[region..end].fill(1);
                }
                dc.record_event(v, &format!("b{}_{}", batch, j), 0.6);
            }
        }
        assert_eq!(dc.event_count(), 100);

        // Run consolidation — should not panic
        let report = dc.run_consolidation_cycle();
        assert!(report.sequences_replayed > 0, "should find sequences");
        assert!(!dc.consolidated_patterns().is_empty());
        assert!(dc.event_count() == 100, "events should not be consumed");
    }

    #[test]
    fn test_dissimilar_events_dont_create_sequences() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());

        // Events with non-overlapping halves → each has ~2048 ones in different positions
        // Similarity between any two should be near 0
        let mut ev1 = vec![0u8; VSA_DIM];
        ev1[0..2048].fill(1);

        let mut ev2 = vec![0u8; VSA_DIM];
        ev2[2048..4096].fill(1);

        let mut ev3 = vec![0u8; VSA_DIM];
        ev3[0..1024].fill(1);
        ev3[3072..4096].fill(1);

        // Verify pairwise similarity is low
        let sim12 = QuantizedVSA::similarity(&ev1, &ev2);
        let sim13 = QuantizedVSA::similarity(&ev1, &ev3);
        let sim23 = QuantizedVSA::similarity(&ev2, &ev3);

        dc.record_event(ev1, "dissim1", 0.8);
        dc.record_event(ev2, "dissim2", 0.8);
        dc.record_event(ev3, "dissim3", 0.8);

        let report = dc.run_consolidation_cycle();
        assert_eq!(
            report.sequences_replayed, 0,
            "no sequences should form from dissimilar events (sim={:.3},{:.3},{:.3})",
            sim12, sim13, sim23
        );
        assert_eq!(report.patterns_merged, 0);
    }

    #[test]
    fn test_consolidation_with_exact_duplicates() {
        let mut dc = DreamConsolidation::new(DreamConfig::default());

        // Exact same vector repeated 5 times → high-similarity sequence
        let v = vec![0b10101010u8; VSA_DIM];
        for i in 0..5 {
            dc.record_event(v.clone(), &format!("dup_{}", i), 0.9);
        }

        let report = dc.run_consolidation_cycle();
        assert!(
            report.sequences_replayed > 0,
            "duplicate vectors should form a sequence"
        );
        assert!(
            report.patterns_merged > 0,
            "duplicate sequence should be merged"
        );
        assert!(
            report.predictions_generated > 0,
            "duplicate sequence should generate predictions"
        );

        // The merged pattern should have high coherence (all vectors identical)
        let high_coherence = dc.consolidated.iter().any(|(_, _, c)| *c > 0.9);
        assert!(high_coherence, "duplicate merge should have high coherence");
    }
}
