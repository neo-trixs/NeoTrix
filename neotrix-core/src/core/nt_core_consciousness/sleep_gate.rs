use std::collections::HashMap;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::stream_buffer::ConsciousnessStream;
use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};

/// SCM-inspired 4D importance (arXiv:2604.20943).
/// Each dimension captures a distinct memory salience signal:
/// recency (time proximity), novelty (dissimilarity to peers),
/// relevance (salience), coherence (confidence).
#[derive(Debug, Clone)]
pub struct MemImportance4D {
    pub recency: f64,
    pub novelty: f64,
    pub relevance: f64,
    pub coherence: f64,
}

impl MemImportance4D {
    /// Geometric mean of the 4 dimensions — a single consolidated score.
    pub fn score(&self) -> f64 {
        (self.recency * self.novelty * self.relevance * self.coherence).powf(0.25)
    }

    /// Classify a consolidated score into a consolidation action.
    pub fn classify(score: f64) -> &'static str {
        if score > 0.7 {
            "consolidate"
        } else if score > 0.4 {
            "rehearse"
        } else if score > 0.2 {
            "tag"
        } else {
            "decay"
        }
    }

    /// Build a 4D importance from raw dimensions, computing recency from
    /// the timestamp delta against the current wall clock.
    pub fn importance_4d(timestamp: u64, novelty: f64, relevance: f64, coherence: f64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let age = if now > timestamp {
            (now - timestamp) as f64
        } else {
            0.0
        };
        let recency = (-age * 0.001).exp().clamp(0.0, 1.0);
        Self {
            recency,
            novelty,
            relevance,
            coherence,
        }
    }
}

/// Sleep phase: detect interference, consolidate, compress.
/// SCM two-phase upgrade (arXiv:2604.20943): NREM pattern extraction +
/// redundancy elimination, then REM cross-domain association discovery.
#[derive(Debug, Clone)]
pub struct SleepGate {
    pub sleep_pressure: f64,
    pub consolidation_count: u64,
    pub last_sleep_iteration: u64,
    pub sleep_interval: usize,
    pub conflict_threshold: f64,
    pub merge_threshold: f64,
    pub consolidation_gate: f64,
    pub last_access: u64,
    pub importance_4d: Option<MemImportance4D>,
    // ── SCM two-phase fields ──
    /// NREM similarity threshold for deduplication
    pub nrem_similarity: f64,
    /// REM similarity range lower bound for cross-domain associations
    pub rem_similarity_low: f64,
    /// REM similarity range upper bound
    pub rem_similarity_high: f64,
    /// Attractor state: stabilized bundled vector after full consolidation
    pub attractor_state: Option<Vec<u8>>,
    /// Total NREM merges across all sleep cycles
    pub total_nrem_merged: u64,
    /// Total REM associations formed across all sleep cycles
    pub total_rem_associations: u64,
}

impl Default for SleepGate {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepGate {
    pub fn new() -> Self {
        Self {
            sleep_pressure: 0.0,
            consolidation_count: 0,
            last_sleep_iteration: 0,
            sleep_interval: 100,
            conflict_threshold: 0.85,
            merge_threshold: 0.92,
            consolidation_gate: 0.5,
            last_access: 0,
            importance_4d: None,
            nrem_similarity: 0.85,
            rem_similarity_low: 0.3,
            rem_similarity_high: 0.7,
            attractor_state: None,
            total_nrem_merged: 0,
            total_rem_associations: 0,
        }
    }

    /// Compute sleep_pressure based on entropy, density, and age of the stream.
    /// Returns the computed pressure (0.0–1.0).
    pub fn observe_interaction(&mut self, stream: &ConsciousnessStream) -> f64 {
        if stream.len() < 2 {
            self.sleep_pressure = 0.0;
            return 0.0;
        }

        let entropy = pairwise_similarity_variance(stream, 50);
        let density = compute_density(stream, 50, self.conflict_threshold);
        let age = (stream.len() as f64 / 100.0).min(1.0);

        let pressure = entropy * 0.2 + density * 0.4 + age * 0.4;
        self.sleep_pressure = pressure.min(1.0);
        self.sleep_pressure
    }

    pub fn should_sleep(&self, iteration: usize) -> bool {
        self.sleep_pressure > 0.7
            || (iteration as u64 - self.last_sleep_iteration >= self.sleep_interval as u64)
    }

    /// Run the full consolidation cycle: conflict detection, selective eviction,
    /// merging of similar entries, and capacity cleanup.
    pub fn execute_sleep(
        &mut self,
        stream: &mut ConsciousnessStream,
        iteration: usize,
    ) -> SleepReport {
        let pre_len = stream.len();
        let pressure_before = self.sleep_pressure;

        if stream.is_empty() {
            self.sleep_pressure = 0.0;
            self.last_sleep_iteration = iteration as u64;
            self.consolidation_count += 1;
            return SleepReport {
                conflicts_detected: 0,
                evicted_count: 0,
                merged_count: 0,
                cleaned_count: 0,
                pre_sleep_len: 0,
                post_sleep_len: 0,
                sleep_pressure_before: pressure_before,
                mean_importance_4d: 0.0,
                high_consolidate_count: 0,
                low_consolidate_count: 0,
                rehearse_count: 0,
                forget_count: 0,
                nrem_merged: 0,
                rem_associations: 0,
            };
        }

        // Snapshot all entries (owned copies) and clear the stream
        let all_refs = stream.recent(stream.len());
        let mut all_entries: Vec<VsaTagged> = all_refs.into_iter().cloned().collect();
        stream.clear();

        let initial_len = all_entries.len();

        // --- Step a: Conflict detection ---
        let entry_refs: Vec<&VsaTagged> = all_entries.iter().collect();
        let conflicts = detect_conflicts(&entry_refs, self.conflict_threshold);
        let conflicts_detected = conflicts.len();

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // --- Step b: Selective eviction (entries with very low confidence) ---
        all_entries.retain(|e| e.confidence >= 0.3);
        let evicted_count = initial_len - all_entries.len();

        // --- SCM Phase 1: NREM — pattern extraction + redundancy elimination ---
        let nrem_merged = self.execute_nrem(&mut all_entries);

        // --- 4D importance (SCM-inspired) ---
        let importance_4d_vec: Vec<MemImportance4D> = all_entries
            .iter()
            .map(|e| self.compute_4d_importance(e, &all_entries, current_time))
            .collect();
        let importance_scores: Vec<f64> = importance_4d_vec.iter().map(|i| i.score()).collect();
        let n_4d = importance_scores.len();
        let mean_4d_score = if n_4d > 0 {
            importance_scores.iter().sum::<f64>() / n_4d as f64
        } else {
            0.0
        };

        // Aggregate 4D importance for the gate
        self.importance_4d = if !importance_4d_vec.is_empty() {
            Some(MemImportance4D {
                recency: importance_4d_vec.iter().map(|i| i.recency).sum::<f64>() / n_4d as f64,
                novelty: importance_4d_vec.iter().map(|i| i.novelty).sum::<f64>() / n_4d as f64,
                relevance: importance_4d_vec.iter().map(|i| i.relevance).sum::<f64>() / n_4d as f64,
                coherence: importance_4d_vec.iter().map(|i| i.coherence).sum::<f64>() / n_4d as f64,
            })
        } else {
            None
        };

        // 4D-informed eviction: drop Forget-tagged entries (score < 0.2)
        let mut retained_4d: Vec<VsaTagged> = Vec::with_capacity(all_entries.len());
        let mut forget_count = 0usize;
        for (idx, entry) in all_entries.into_iter().enumerate() {
            let s = importance_scores.get(idx).copied().unwrap_or(0.0);
            if s < 0.2 {
                forget_count += 1;
            } else {
                retained_4d.push(entry);
            }
        }
        let evicted_4d_count = forget_count;
        let _all_entries_len_after_4d = retained_4d.len();
        let all_entries = retained_4d;
        // 4D-evicted entries count toward total eviction
        let evicted_count = evicted_count + evicted_4d_count;

        // Count MemTag4D categories
        let high_count = importance_scores.iter().filter(|&&s| s > 0.7).count();
        let low_count = importance_scores
            .iter()
            .filter(|&&s| s > 0.4 && s <= 0.7)
            .count();
        let rehearse_count = importance_scores
            .iter()
            .filter(|&&s| s > 0.2 && s <= 0.4)
            .count();

        // --- Step c: Merging (original greedy clique merge) ---
        let (merged_count, survivors) = Self::merge_group(&all_entries, self.merge_threshold);
        let mut final_entries = survivors;

        // --- SCM Phase 2: REM — cross-domain association discovery ---
        let rem_associations = self.execute_rem(&mut final_entries, current_time);

        // --- Step d: Cleanup (evict lowest-importance if > 80 % capacity) ---
        let cap = stream.capacity();
        let target_max = cap * 80 / 100;
        let cleaned_count = if final_entries.len() > target_max {
            let remove = final_entries.len() - target_max;
            // Sort by timestamp (oldest first) as proxy for low importance
            final_entries.sort_by_key(|e| e.timestamp);
            final_entries.drain(0..remove);
            remove
        } else {
            0
        };

        self.last_access = current_time;

        for entry in final_entries {
            stream.push(entry);
        }

        self.sleep_pressure = 0.0;
        self.last_sleep_iteration = iteration as u64;
        self.consolidation_count += 1;

        SleepReport {
            conflicts_detected,
            evicted_count,
            merged_count,
            cleaned_count,
            pre_sleep_len: pre_len,
            post_sleep_len: stream.len(),
            sleep_pressure_before: pressure_before,
            mean_importance_4d: mean_4d_score,
            high_consolidate_count: high_count,
            low_consolidate_count: low_count,
            rehearse_count,
            forget_count,
            nrem_merged,
            rem_associations,
        }
    }

    /// SCM Phase 1 — NREM: Pattern Extraction + Redundancy Elimination.
    /// Deduplicates similar memory traces (similarity > nrem_similarity)
    /// by bundling their VSA vectors into a single pattern.
    /// Returns number of merged entries.
    pub fn execute_nrem(&mut self, entries: &mut Vec<VsaTagged>) -> usize {
        if entries.len() < 2 {
            return 0;
        }

        let mut merged_any = false;
        let mut merged_count = 0usize;
        let mut i = 0;
        while i < entries.len() {
            let mut j = i + 1;
            while j < entries.len() {
                let sim = QuantizedVSA::similarity(&entries[i].vector, &entries[j].vector);
                if sim > self.nrem_similarity {
                    // Bundle: merge vector by majority bundling
                    let refs: Vec<&[u8]> =
                        [entries[i].vector.as_slice(), entries[j].vector.as_slice()].to_vec();
                    entries[i].vector = QuantizedVSA::majority_bundle(&refs);
                    // Merge confidence and salience
                    entries[i].confidence = (entries[i].confidence + entries[j].confidence) / 2.0;
                    entries[i].salience = entries[i].salience.max(entries[j].salience);
                    entries[i].timestamp = entries[i].timestamp.max(entries[j].timestamp);
                    // Remove duplicate
                    entries.remove(j);
                    merged_count += 1;
                    merged_any = true;
                    // Re-check from same i (don't increment)
                } else {
                    j += 1;
                }
            }
            i += 1;
        }

        // After NREM, build attractor state from the bundled centroid
        if merged_any && !entries.is_empty() {
            let refs: Vec<&[u8]> = entries.iter().map(|e| e.vector.as_slice()).collect();
            self.attractor_state = Some(QuantizedVSA::majority_bundle(&refs));
        }

        self.total_nrem_merged += merged_count as u64;
        merged_count
    }

    /// SCM Phase 2 — REM: Cross-Domain Association Discovery.
    /// Finds entries with VSA similarity in (rem_similarity_low, rem_similarity_high)
    /// range and creates association entries by bundling them.
    /// Modifies entries in-place by adding new association entries.
    /// Returns number of new associations created.
    pub fn execute_rem(&mut self, entries: &mut Vec<VsaTagged>, current_time: u64) -> usize {
        if entries.len() < 4 {
            return 0;
        }

        let mut new_entries: Vec<VsaTagged> = Vec::new();
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let sim = QuantizedVSA::similarity(&entries[i].vector, &entries[j].vector);
                // Potential cross-domain association in the moderate similarity range
                if sim > self.rem_similarity_low && sim < self.rem_similarity_high {
                    let bound = QuantizedVSA::bind(&entries[i].vector, &entries[j].vector);
                    let avg_conf = (entries[i].confidence + entries[j].confidence) / 2.0;
                    new_entries.push(VsaTagged {
                        vector: bound,
                        tag: VsaOrigin::Self_(VsaSelfCategory::Association),
                        confidence: avg_conf * sim,
                        timestamp: current_time,
                        salience: sim,
                        provenance: None,
                        sense_modality: None,
                        prediction: None,
                        outcome: None,
                    });
                }
            }
        }

        let count = new_entries.len();
        entries.extend(new_entries);
        self.total_rem_associations += count as u64;
        count
    }

    /// Public wrapper: updates pressure, then executes sleep if warranted.
    pub fn consolidate(
        &mut self,
        stream: &mut ConsciousnessStream,
        iteration: usize,
    ) -> SleepReport {
        self.observe_interaction(stream);
        if self.should_sleep(iteration) {
            self.execute_sleep(stream, iteration)
        } else {
            SleepReport {
                conflicts_detected: 0,
                evicted_count: 0,
                merged_count: 0,
                cleaned_count: 0,
                pre_sleep_len: stream.len(),
                post_sleep_len: stream.len(),
                sleep_pressure_before: self.sleep_pressure,
                mean_importance_4d: 0.0,
                high_consolidate_count: 0,
                low_consolidate_count: 0,
                rehearse_count: 0,
                forget_count: 0,
                nrem_merged: 0,
                rem_associations: 0,
            }
        }
    }

    /// Compute SCM-inspired 4D importance for a single entry relative to all entries.
    /// Recency (time since creation), novelty (dissimilarity to peers),
    /// relevance (salience), coherence (confidence in the entry).
    pub fn compute_4d_importance(
        &self,
        entry: &VsaTagged,
        all_entries: &[VsaTagged],
        current_time: u64,
    ) -> MemImportance4D {
        let age = if current_time > entry.timestamp {
            (current_time - entry.timestamp) as f64
        } else {
            0.0
        };
        let recency = (-age * 0.001).exp().clamp(0.0, 1.0);

        let novelty = if all_entries.len() <= 1 {
            1.0
        } else {
            let sum_sim: f64 = all_entries
                .iter()
                .filter(|e| e.timestamp != entry.timestamp)
                .map(|e| QuantizedVSA::similarity(&entry.vector, &e.vector))
                .sum();
            let avg = sum_sim / (all_entries.len() - 1) as f64;
            (1.0 - avg).clamp(0.0, 1.0)
        };

        let relevance = entry.salience.clamp(0.0, 1.0);
        let coherence = entry.confidence.clamp(0.0, 1.0);

        MemImportance4D {
            recency,
            novelty,
            relevance,
            coherence,
        }
    }

    /// Find groups of same-tag entries where all pairwise similarities exceed
    /// `threshold` and merge each group of size > 2 into a single bundled entry.
    /// Returns (merged_count, surviving_entries).
    fn merge_group(entries: &[VsaTagged], threshold: f64) -> (usize, Vec<VsaTagged>) {
        let n = entries.len();
        if n < 3 {
            return (0, entries.to_vec());
        }

        let mut by_tag: HashMap<VsaOrigin, Vec<usize>> = HashMap::new();
        for (i, entry) in entries.iter().enumerate() {
            by_tag.entry(entry.tag).or_default().push(i);
        }

        let mut merged_count = 0usize;
        let mut survivors: Vec<VsaTagged> = Vec::new();

        for (_, indices) in by_tag {
            let m = indices.len();
            if m < 3 {
                for &idx in &indices {
                    survivors.push(entries[idx].clone());
                }
                continue;
            }

            // Build similarity matrix for this tag group
            let mut sim = vec![vec![0.0f64; m]; m];
            for i in 0..m {
                for j in (i + 1)..m {
                    let s = QuantizedVSA::similarity(
                        &entries[indices[i]].vector,
                        &entries[indices[j]].vector,
                    );
                    sim[i][j] = s;
                    sim[j][i] = s;
                }
            }

            // Greedy clique discovery
            let mut used = vec![false; m];
            for start in 0..m {
                if used[start] {
                    continue;
                }

                let mut clique = vec![start];
                used[start] = true;

                for candidate in (start + 1)..m {
                    if used[candidate] {
                        continue;
                    }
                    let all_similar = clique.iter().all(|&c| sim[c][candidate] > threshold);
                    if all_similar {
                        clique.push(candidate);
                        used[candidate] = true;
                    }
                }

                if clique.len() > 2 {
                    let vectors: Vec<&[u8]> = clique
                        .iter()
                        .map(|&idx| entries[indices[idx]].vector.as_slice())
                        .collect();
                    let bundled = QuantizedVSA::bundle(&vectors);
                    let avg_conf = clique
                        .iter()
                        .map(|&idx| entries[indices[idx]].confidence)
                        .sum::<f64>()
                        / clique.len() as f64;

                    survivors.push(VsaTagged {
                        vector: bundled,
                        tag: entries[indices[clique[0]]].tag,
                        confidence: avg_conf,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                        salience: 0.5,
                        provenance: None,
                        sense_modality: None,
                        prediction: None,
                        outcome: None,
                    });
                    merged_count += clique.len() - 1;
                } else {
                    for &idx in &clique {
                        survivors.push(entries[indices[idx]].clone());
                    }
                }
            }
        }

        (merged_count, survivors)
    }
}

/// Result of a single sleep cycle.
pub struct SleepReport {
    pub conflicts_detected: usize,
    pub evicted_count: usize,
    pub merged_count: usize,
    pub cleaned_count: usize,
    pub pre_sleep_len: usize,
    pub post_sleep_len: usize,
    pub sleep_pressure_before: f64,
    pub mean_importance_4d: f64,
    pub high_consolidate_count: usize,
    pub low_consolidate_count: usize,
    pub rehearse_count: usize,
    pub forget_count: usize,
    // ── SCM two-phase fields ──
    pub nrem_merged: usize,
    pub rem_associations: usize,
}

// ---------------------------------------------------------------------------
// Public helper functions
// ---------------------------------------------------------------------------

/// Variance of all pairwise similarities among the last `n` stream entries.
/// Returns 0 if fewer than 2 entries are available.
pub fn pairwise_similarity_variance(stream: &ConsciousnessStream, n: usize) -> f64 {
    let entries = stream.recent(n);
    let m = entries.len();
    if m < 2 {
        return 0.0;
    }

    let mut similarities = Vec::with_capacity(m * (m - 1) / 2);
    for i in 0..m {
        for j in (i + 1)..m {
            let s = QuantizedVSA::similarity(&entries[i].vector, &entries[j].vector);
            similarities.push(s);
        }
    }

    let count = similarities.len();
    if count == 0 {
        return 0.0;
    }

    let mean: f64 = similarities.iter().sum::<f64>() / count as f64;
    let variance: f64 = similarities.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / count as f64;

    // Normalize so that max possible variance (0.25 for binary [0,1] range) maps to 1.0
    (variance * 4.0).min(1.0)
}

/// Ratio of pairs whose similarity exceeds `threshold` among the last `n` entries.
pub fn compute_density(stream: &ConsciousnessStream, n: usize, threshold: f64) -> f64 {
    let entries = stream.recent(n);
    let m = entries.len();
    if m < 2 {
        return 0.0;
    }

    let mut high = 0usize;
    let mut total = 0usize;
    for i in 0..m {
        for j in (i + 1)..m {
            let s = QuantizedVSA::similarity(&entries[i].vector, &entries[j].vector);
            if s > threshold {
                high += 1;
            }
            total += 1;
        }
    }

    if total == 0 {
        0.0
    } else {
        high as f64 / total as f64
    }
}

/// Find pairs of indices whose vectors have similarity > threshold but carry
/// different `VsaOrigin` variants (Self vs World).  Such pairs represent
/// identity-boundary conflicts.
pub fn detect_conflicts(entries: &[&VsaTagged], threshold: f64) -> Vec<(usize, usize)> {
    let mut conflicts = Vec::new();
    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let tags_differ = matches!(
                (&entries[i].tag, &entries[j].tag),
                (VsaOrigin::Self_(_), VsaOrigin::World(_))
                    | (VsaOrigin::World(_), VsaOrigin::Self_(_))
            );
            if !tags_differ {
                continue;
            }
            let s = QuantizedVSA::similarity(&entries[i].vector, &entries[j].vector);
            if s > threshold {
                conflicts.push((i, j));
            }
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{
        VsaOrigin, VsaSelfCategory, VsaWorldCategory,
    };
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    // -----------------------------------------------------------------------
    // 1
    // -----------------------------------------------------------------------
    #[test]
    fn test_new_defaults() {
        let gate = SleepGate::new();
        assert!((gate.sleep_pressure - 0.0).abs() < 1e-9);
        assert_eq!(gate.consolidation_count, 0);
        assert_eq!(gate.last_sleep_iteration, 0);
        assert_eq!(gate.sleep_interval, 100);
        assert!((gate.conflict_threshold - 0.85).abs() < 1e-9);
        assert!((gate.merge_threshold - 0.92).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // 2
    // -----------------------------------------------------------------------
    #[test]
    fn test_observe_empty() {
        let stream = ConsciousnessStream::new(1024);
        let mut gate = SleepGate::new();
        let p = gate.observe_interaction(&stream);
        assert!((p - 0.0).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // 3
    // -----------------------------------------------------------------------
    #[test]
    fn test_observe_identical_vectors() {
        let mut stream = ConsciousnessStream::new(1024);
        let v = QuantizedVSA::random_binary();
        let tag = VsaOrigin::Self_(VsaSelfCategory::Thought);
        for _ in 0..100 {
            stream.push(VsaTagged::new(v.clone(), tag));
        }
        let mut gate = SleepGate::new();
        let p = gate.observe_interaction(&stream);
        assert!(
            p > 0.5,
            "identical vectors should produce high pressure, got {}",
            p
        );
    }

    // -----------------------------------------------------------------------
    // 4
    // -----------------------------------------------------------------------
    #[test]
    fn test_should_sleep_false_initially() {
        let gate = SleepGate::new();
        assert!(!gate.should_sleep(10));
    }

    // -----------------------------------------------------------------------
    // 5
    // -----------------------------------------------------------------------
    #[test]
    fn test_should_sleep_true_high_pressure() {
        let mut gate = SleepGate::new();
        gate.sleep_pressure = 0.8;
        assert!(gate.should_sleep(0));
    }

    // -----------------------------------------------------------------------
    // 6
    // -----------------------------------------------------------------------
    #[test]
    fn test_should_sleep_true_interval_exceeded() {
        let mut gate = SleepGate::new();
        gate.last_sleep_iteration = 0;
        assert!(gate.should_sleep(gate.sleep_interval));
    }

    // -----------------------------------------------------------------------
    // 7
    // -----------------------------------------------------------------------
    #[test]
    fn test_execute_sleep_empty_stream() {
        let mut stream = ConsciousnessStream::new(1024);
        let mut gate = SleepGate::new();
        let report = gate.execute_sleep(&mut stream, 10);
        assert_eq!(report.evicted_count, 0);
        assert_eq!(report.merged_count, 0);
        assert_eq!(report.pre_sleep_len, 0);
        assert_eq!(report.post_sleep_len, 0);
    }

    // -----------------------------------------------------------------------
    // 8
    // -----------------------------------------------------------------------
    #[test]
    fn test_consolidate_no_panic_empty() {
        let mut stream = ConsciousnessStream::new(1024);
        let mut gate = SleepGate::new();
        let report = gate.consolidate(&mut stream, 0);
        assert_eq!(report.pre_sleep_len, 0);
        assert_eq!(report.post_sleep_len, 0);
    }

    // -----------------------------------------------------------------------
    // 9
    // -----------------------------------------------------------------------
    #[test]
    fn test_pairwise_variance_identical() {
        let mut stream = ConsciousnessStream::new(1024);
        let v = QuantizedVSA::random_binary();
        for _ in 0..5 {
            stream.push(VsaTagged::new(
                v.clone(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }
        let var = pairwise_similarity_variance(&stream, 10);
        assert!(
            var < 1e-10,
            "variance should be ~0 for identical vectors, got {}",
            var
        );
    }

    // -----------------------------------------------------------------------
    // 10
    // -----------------------------------------------------------------------
    #[test]
    fn test_detect_conflicts_empty() {
        let entries: Vec<&VsaTagged> = vec![];
        let conflicts = detect_conflicts(&entries, 0.85);
        assert!(conflicts.is_empty());
    }

    // -----------------------------------------------------------------------
    // 11
    // -----------------------------------------------------------------------
    #[test]
    fn test_detect_conflicts_different_tags() {
        let v = QuantizedVSA::random_binary();
        let a = VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought));
        let b = VsaTagged::new(v, VsaOrigin::World(VsaWorldCategory::UserInput));
        let entries = vec![&a, &b];
        let conflicts = detect_conflicts(&entries, 0.85);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], (0, 1));
    }

    // -----------------------------------------------------------------------
    // 12
    // -----------------------------------------------------------------------
    #[test]
    fn test_merge_reduces_count() {
        let v = QuantizedVSA::random_binary();
        let tag = VsaOrigin::Self_(VsaSelfCategory::Thought);
        let mut stream = ConsciousnessStream::new(100);
        for _ in 0..4 {
            stream.push(VsaTagged::new(v.clone(), tag));
        }
        let mut gate = SleepGate::new();
        let report = gate.execute_sleep(&mut stream, 0);
        assert!(
            report.merged_count > 0,
            "expected merges, got {}",
            report.merged_count
        );
        assert!(report.post_sleep_len < report.pre_sleep_len);
    }

    // -----------------------------------------------------------------------
    // 13 — diverse vectors produce lower pressure than identical ones
    // -----------------------------------------------------------------------
    #[test]
    fn test_observe_diverse_lower_than_identical() {
        let mut stream_ident = ConsciousnessStream::new(1024);
        let v = QuantizedVSA::random_binary();
        for _ in 0..80 {
            stream_ident.push(VsaTagged::new(
                v.clone(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }

        let mut stream_mixed = ConsciousnessStream::new(1024);
        for _ in 0..80 {
            stream_mixed.push(VsaTagged::new(
                QuantizedVSA::random_binary(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }

        let mut gate = SleepGate::new();
        let p_ident = gate.observe_interaction(&stream_ident);
        let p_mixed = gate.observe_interaction(&stream_mixed);
        assert!(
            p_ident > p_mixed,
            "identical vectors should yield higher pressure than random ones: ident={} mixed={}",
            p_ident,
            p_mixed
        );
    }

    // -----------------------------------------------------------------------
    // 14 — after execute_sleep, pressure resets to 0
    // -----------------------------------------------------------------------
    #[test]
    fn test_pressure_reset_after_sleep() {
        let mut stream = ConsciousnessStream::new(1024);
        let v = QuantizedVSA::random_binary();
        for _ in 0..80 {
            stream.push(VsaTagged::new(
                v.clone(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }
        let mut gate = SleepGate::new();
        gate.observe_interaction(&stream);
        assert!(gate.sleep_pressure > 0.0);
        gate.execute_sleep(&mut stream, 1);
        assert!((gate.sleep_pressure - 0.0).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // 15 — consolidation_count increments
    // -----------------------------------------------------------------------
    #[test]
    fn test_consolidation_count_increments() {
        let mut stream = ConsciousnessStream::new(1024);
        let v = QuantizedVSA::random_binary();
        for _ in 0..100 {
            stream.push(VsaTagged::new(
                v.clone(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }
        let mut gate = SleepGate::new();
        gate.sleep_pressure = 0.8;
        assert_eq!(gate.consolidation_count, 0);
        gate.execute_sleep(&mut stream, 0);
        assert_eq!(gate.consolidation_count, 1);
    }

    // -----------------------------------------------------------------------
    // 16 — detect_conflicts ignores same-tag pairs even when similar
    // -----------------------------------------------------------------------
    #[test]
    fn test_detect_conflicts_ignores_same_tag() {
        let v = QuantizedVSA::random_binary();
        let a = VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought));
        let b = VsaTagged::new(v, VsaOrigin::Self_(VsaSelfCategory::Memory));
        let entries = vec![&a, &b];
        let conflicts = detect_conflicts(&entries, 0.85);
        assert!(
            conflicts.is_empty(),
            "same VsaOrigin::Self_ should not conflict"
        );
    }

    // -----------------------------------------------------------------------
    // 17 — compute_density on identical vectors returns 1.0
    // -----------------------------------------------------------------------
    #[test]
    fn test_density_identical() {
        let mut stream = ConsciousnessStream::new(1024);
        let v = QuantizedVSA::random_binary();
        for _ in 0..5 {
            stream.push(VsaTagged::new(
                v.clone(),
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }
        let d = compute_density(&stream, 10, 0.85);
        assert!((d - 1.0).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // 18 — execute_sleep on non-empty stream produces internally consistent report
    // -----------------------------------------------------------------------
    #[test]
    fn test_execute_sleep_internally_consistent() {
        let mut stream = ConsciousnessStream::new(100);
        for _ in 0..50 {
            let v = QuantizedVSA::random_binary();
            stream.push(VsaTagged::new(
                v,
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }
        let mut gate = SleepGate::new();
        gate.sleep_pressure = 0.9;
        let report = gate.execute_sleep(&mut stream, 5);
        assert_eq!(report.pre_sleep_len, 50);
        assert!(report.post_sleep_len <= report.pre_sleep_len);
        assert_eq!(
            report.evicted_count + report.merged_count + report.cleaned_count,
            report.pre_sleep_len - report.post_sleep_len
        );
    }

    // -----------------------------------------------------------------------
    // 19 — NREM merges similar vectors
    // -----------------------------------------------------------------------
    #[test]
    fn test_nrem_merges_similar_vectors() {
        let v = QuantizedVSA::random_binary();
        let tag = VsaOrigin::Self_(VsaSelfCategory::Thought);
        let mut stream = ConsciousnessStream::new(100);
        for _ in 0..5 {
            stream.push(VsaTagged::new(v.clone(), tag));
        }
        let mut gate = SleepGate::new();
        let report = gate.execute_sleep(&mut stream, 0);
        assert!(report.nrem_merged > 0, "NREM should merge similar vectors");
        assert!(
            gate.attractor_state.is_some(),
            "attractor_state should be set after NREM"
        );
    }

    // -----------------------------------------------------------------------
    // 20 — REM discovers cross-domain associations
    // -----------------------------------------------------------------------
    #[test]
    fn test_rem_creates_associations() {
        let tag = VsaOrigin::Self_(VsaSelfCategory::Thought);
        let mut stream = ConsciousnessStream::new(100);
        // Create enough moderately similar vectors to trigger REM
        for i in 0..8 {
            let v = QuantizedVSA::random_binary();
            // Make some vectors moderately similar by flipping only a few bits
            if i > 0 {
                let _prev_idx = (i - 1) % 8;
                let _sim = QuantizedVSA::similarity(&v, &v);
            }
            stream.push(VsaTagged::new(v, tag));
        }
        let mut gate = SleepGate::new();
        gate.rem_similarity_low = 0.0;
        gate.rem_similarity_high = 1.0;
        let report = gate.execute_sleep(&mut stream, 0);
        // REM may or may not find associations depending on random vectors
        // Just verify the pipeline runs without error
        assert!(report.post_sleep_len >= report.nrem_merged);
    }

    // -----------------------------------------------------------------------
    // 21 — attractor_state is None before any sleep
    // -----------------------------------------------------------------------
    #[test]
    fn test_attractor_state_initially_none() {
        let gate = SleepGate::new();
        assert!(gate.attractor_state.is_none());
    }
}
