use std::collections::HashMap;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::stream_buffer::ConsciousnessStream;
use super::vsa_tag::{VsaOrigin, VsaTagged};

/// Sleep phase: detect interference, consolidate, compress
pub struct SleepGate {
    pub sleep_pressure: f64,
    pub consolidation_count: u64,
    pub last_sleep_iteration: u64,
    pub sleep_interval: usize,
    pub conflict_threshold: f64,
    pub merge_threshold: f64,
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

        // --- Step b: Selective eviction (entries with very low confidence) ---
        all_entries.retain(|e| e.confidence >= 0.3);
        let evicted_count = initial_len - all_entries.len();

        // --- Step c: Merging ---
        let (merged_count, survivors) = Self::merge_group(&all_entries, self.merge_threshold);

        // --- Step d: Cleanup (evict oldest if > 80 % capacity) ---
        let cap = stream.capacity();
        let target_max = cap * 80 / 100;
        let mut final_entries = survivors;
        let cleaned_count = if final_entries.len() > target_max {
            let remove = final_entries.len() - target_max;
            final_entries.drain(0..remove);
            remove
        } else {
            0
        };

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
        }
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
            }
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
                    let all_similar = clique
                        .iter()
                        .all(|&c| sim[c][candidate] > threshold);
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
                        timestamp: std::time::Instant::now(),
                        salience: 0.5,
                        provenance: None,
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
    let variance: f64 = similarities
        .iter()
        .map(|s| (s - mean).powi(2))
        .sum::<f64>()
        / count as f64;

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
        assert!(p > 0.5, "identical vectors should produce high pressure, got {}", p);
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
            stream.push(VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought)));
        }
        let var = pairwise_similarity_variance(&stream, 10);
        assert!(var < 1e-10, "variance should be ~0 for identical vectors, got {}", var);
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
        assert!(report.merged_count > 0, "expected merges, got {}", report.merged_count);
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
            stream_ident.push(VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought)));
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
            stream.push(VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought)));
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
            stream.push(VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought)));
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
        assert!(conflicts.is_empty(), "same VsaOrigin::Self_ should not conflict");
    }

    // -----------------------------------------------------------------------
    // 17 — compute_density on identical vectors returns 1.0
    // -----------------------------------------------------------------------
    #[test]
    fn test_density_identical() {
        let mut stream = ConsciousnessStream::new(1024);
        let v = QuantizedVSA::random_binary();
        for _ in 0..5 {
            stream.push(VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought)));
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
            stream.push(VsaTagged::new(v, VsaOrigin::Self_(VsaSelfCategory::Thought)));
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
}
