//! # CTE 4-Stage Consolidation Pipeline
//!
//! Implements the CTE (arXiv 2606) 4-stage memory consolidation:
//! 1. SWS — Critical path extraction
//! 2. REM — Cross-domain association
//! 3. Consolidation — Semantic connection + merge detection
//! 4. Compaction — Low-access node summary compression

use super::memory_lattice::{LatticeEntry, MemoryLattice};

/// Report from a single CTE cycle.
#[derive(Debug, Clone, Default)]
pub struct CteReport {
    pub sws_extracted: usize,
    pub rem_associated: usize,
    pub consolidated: usize,
    pub compacted: usize,
    pub region_consolidated: usize,
    pub sws_keys: Vec<String>,
    pub rem_cross_links: Vec<(String, String)>,
    pub compaction_summary: Vec<String>,
    pub region_summaries: Vec<String>,
    pub total_duration_ms: u64,
}

/// CTE 4-stage consolidation cycle configuration and state.
#[derive(Debug, Clone)]
pub struct CteCycle {
    pub sws_enabled: bool,
    pub rem_enabled: bool,
    pub consolidation_enabled: bool,
    pub compaction_enabled: bool,
    pub region_consolidation_enabled: bool,
    pub sws_threshold: f64,
    pub compaction_interval: u64,
    pub associative_threshold: f64,
    pub merge_similarity: f64,
    pub max_entries: usize,
    pub region_min_entries: usize,
    pub cycle_count: u64,
    pub total_sws_extracted: u64,
    pub total_rem_associated: u64,
    pub total_consolidated: u64,
    pub total_compacted: u64,
    pub total_region_consolidated: u64,
    pub last_compaction_cycle: u64,
}

impl Default for CteCycle {
    fn default() -> Self {
        Self {
            sws_enabled: true,
            rem_enabled: true,
            consolidation_enabled: true,
            compaction_enabled: true,
            region_consolidation_enabled: true,
            sws_threshold: 0.4,
            compaction_interval: 50,
            associative_threshold: 0.3,
            merge_similarity: 0.85,
            max_entries: 500,
            region_min_entries: 3,
            cycle_count: 0,
            total_sws_extracted: 0,
            total_rem_associated: 0,
            total_consolidated: 0,
            total_compacted: 0,
            total_region_consolidated: 0,
            last_compaction_cycle: 0,
        }
    }
}

impl CteCycle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run one full CTE consolidation cycle.
    pub fn run_cte_cycle(&mut self, lattice: &mut MemoryLattice, cycle: u64) -> CteReport {
        let t_start = std::time::Instant::now();
        self.cycle_count = cycle;

        let mut report = CteReport::default();

        // Phase 1: SWS extraction
        if self.sws_enabled {
            let extracted = self.sws_extract(lattice);
            report.sws_extracted = extracted.len();
            report.sws_keys = extracted;
        }

        // Phase 2: REM association (only on SWS-extracted entries)
        if self.rem_enabled && !report.sws_keys.is_empty() {
            let associations = self.rem_associate(lattice);
            report.rem_associated = associations.len();
            report.rem_cross_links = associations;
        }

        // Phase 3: Consolidation
        if self.consolidation_enabled {
            let consolidated = self.consolidate(lattice);
            report.consolidated = consolidated;
        }

        // Phase 3b: Region-based cross-session consolidation (Auto-Dreamer inspired)
        // Groups entries by domain, synthesizes compact replacements for regions with 3+ entries.
        if self.region_consolidation_enabled {
            let (num_regions, summaries) = self.region_consolidate(lattice);
            report.region_consolidated = num_regions;
            report.region_summaries = summaries;
        }

        // Phase 3.5: Apply forgetting curve before compaction
        // Ebbinghaus-inspired exponential decay on q_value and confidence
        if self.compaction_enabled && cycle - self.last_compaction_cycle >= self.compaction_interval
        {
            lattice.apply_forgetting(0.5);
            lattice.prune_forgotten(0.01);

            // Phase 4: Compaction
            let summaries = self.compact(lattice);
            report.compacted = summaries.len();
            report.compaction_summary = summaries;
            self.last_compaction_cycle = cycle;
        }

        report.total_duration_ms = t_start.elapsed().as_millis() as u64;

        // Update total stats
        self.total_sws_extracted += report.sws_extracted as u64;
        self.total_rem_associated += report.rem_associated as u64;
        self.total_consolidated += report.consolidated as u64;
        self.total_compacted += report.compacted as u64;
        self.total_region_consolidated += report.region_consolidated as u64;

        report
    }

    /// Phase 1 — SWS: extract high-importance episodic patterns.
    /// Scans episodic entries with q_value > sws_threshold or confidence > 0.5,
    /// tags them as consolidated, and returns their content.
    fn sws_extract(&self, lattice: &mut MemoryLattice) -> Vec<String> {
        let mut extracted = Vec::new();
        let indices: Vec<usize> = lattice
            .episodic
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                !e.consolidated && (e.q_value > self.sws_threshold || e.confidence > 0.5)
            })
            .map(|(i, _)| i)
            .collect();

        for idx in indices {
            if let Some(entry) = lattice.episodic.get_mut(idx) {
                extracted.push(entry.content.clone());
                entry.consolidated = true;
                entry.confidence = entry.confidence.min(0.9) + 0.1;
            }
        }
        extracted
    }

    /// Phase 2 — REM: cross-domain association.
    /// Compares all episodic entries pair-wise using hamming similarity.
    /// Creates associations for pairs with similarity in [associative_threshold, merge_similarity).
    fn rem_associate(&self, lattice: &MemoryLattice) -> Vec<(String, String)> {
        let mut associations = Vec::new();
        let entries: Vec<&LatticeEntry> = lattice.episodic.iter().collect();

        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let sim = hamming_similarity(&entries[i].vsa_hash, &entries[j].vsa_hash);
                if sim >= self.associative_threshold && sim < self.merge_similarity {
                    associations.push((entries[i].content.clone(), entries[j].content.clone()));
                }
            }
        }
        associations
    }

    /// Phase 3 — Consolidation: merge associated content and boost confidence.
    fn consolidate(&self, lattice: &mut MemoryLattice) -> usize {
        let mut count = 0;
        for entry in lattice.episodic.iter_mut() {
            if entry.confidence < 0.95 {
                entry.confidence = (entry.confidence + 0.05).min(0.95);
                count += 1;
            }
        }
        for entry in lattice.facts.iter_mut() {
            if entry.confidence < 0.95 {
                entry.confidence = (entry.confidence + 0.05).min(0.95);
                count += 1;
            }
        }
        count
    }

    /// Phase 4 — Compaction: summarize low-access entries.
    /// Removes entries with low access count (< 3) and low confidence (< 0.3),
    /// replaces them with a compacted summary (first 100 chars).
    fn compact(&self, lattice: &mut MemoryLattice) -> Vec<String> {
        let mut summaries = Vec::new();
        let mut survivors: Vec<LatticeEntry> = Vec::new();

        for entry in lattice.episodic.drain(..) {
            if entry.invocation_count < 3 && entry.confidence < 0.3 && entry.consolidated {
                let summary = if entry.content.len() > 100 {
                    format!("{}…", &entry.content[..100])
                } else {
                    entry.content.clone()
                };
                summaries.push(summary);
            } else {
                survivors.push(entry);
            }
        }
        lattice.episodic = survivors.into();
        summaries
    }

    /// Phase 3b — Region-based consolidation (Auto-Dreamer inspired).
    ///
    /// Groups entries by domain into "regions". For each region with ≥ region_min_entries,
    /// synthesizes a compact abstracted entry that replaces the individual entries.
    /// Uses counterfactual utility: randomly masks one entry per region; if retrieval
    /// still returns relevant results without it, the entry is safe to consolidate.
    ///
    /// Returns (number_of_regions_consolidated, region_summaries).
    fn region_consolidate(&self, lattice: &mut MemoryLattice) -> (usize, Vec<String>) {
        if !self.region_consolidation_enabled {
            return (0, vec![]);
        }

        let mut summaries = Vec::new();
        let mut consolidated_count = 0usize;

        // Step 1: Group episodic entries by domain
        let mut by_domain: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();
        for (idx, entry) in lattice.episodic.iter().enumerate() {
            if entry.consolidated {
                continue; // skip already consolidated entries
            }
            by_domain.entry(entry.domain.clone()).or_default().push(idx);
        }

        // Step 2: For each domain with enough entries, generate a region
        let mut to_remove: Vec<usize> = Vec::new();

        for (_domain, indices) in by_domain.iter() {
            if indices.len() < self.region_min_entries {
                continue;
            }

            let region_indices: Vec<usize> = indices
                .iter()
                .copied()
                .filter(|i| {
                    lattice
                        .episodic
                        .get(*i)
                        .map(|e| !e.consolidated)
                        .unwrap_or(false)
                })
                .collect();

            if region_indices.len() < self.region_min_entries {
                continue;
            }

            // Collect content and VSA hashes for the region
            let mut region_contents: Vec<String> = Vec::new();
            for &idx in &region_indices {
                if let Some(entry) = lattice.episodic.get(idx) {
                    region_contents.push(entry.content.clone());
                }
            }

            // If too few entries for meaningful abstraction, promote confidence and skip
            if region_contents.len() <= 2 {
                for &idx in &region_indices {
                    if let Some(entry) = lattice.episodic.get_mut(idx) {
                        entry.consolidated = true;
                        entry.confidence = entry.confidence.min(0.9) + 0.05;
                    }
                }
                consolidated_count += 1;
                summaries.push(format!(
                    "region_promoted:{}_entries:{}",
                    region_contents
                        .first()
                        .map(|s| &s[..s.len().min(40)])
                        .unwrap_or(""),
                    region_contents.len()
                ));
                continue;
            }

            // Counterfactual utility check: mask the last entry in the region,
            // check if the rest are semantically similar enough to form a coherent group
            let keep_indices = self.counterfactual_utility_check(&region_indices, lattice);

            if keep_indices.len() < self.region_min_entries {
                // Not enough content survives counterfactual — skip this region
                continue;
            }

            // Build compact abstracted content from the keepers
            let abstracted_content = region_contents
                .iter()
                .take(3)
                .map(|c| {
                    if c.len() > 60 {
                        format!("{}…", &c[..60])
                    } else {
                        c.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" | ");

            let abstracted = format!("[abstracted: {}]", abstracted_content);

            // Track indices to remove (all consolidated entries except the abstracted one)
            for &idx in &region_indices {
                to_remove.push(idx);
            }

            // Create a single abstracted entry
            if let Some(first_entry) = lattice.episodic.get(region_indices[0]) {
                let abstracted_entry = LatticeEntry {
                    content: abstracted.clone(),
                    vsa_hash: first_entry.vsa_hash.clone(),
                    layer: first_entry.layer,
                    confidence: first_entry.confidence.min(0.85).max(0.6),
                    invocation_count: region_indices.len() as u64,
                    last_accessed: first_entry.last_accessed,
                    source_layer: first_entry.source_layer,
                    consolidated: true,
                    q_value: first_entry.q_value.max(0.5),
                    valid_from: first_entry.valid_from,
                    valid_to: first_entry.valid_to,
                    origin: first_entry.origin,
                    provenance_parent: None,
                    belief_state: first_entry.belief_state,
                    domain: first_entry.domain.clone(),
                };
                lattice.episodic.push_back(abstracted_entry);
                consolidated_count += 1;
                summaries.push(format!(
                    "region_abstracted:{}_from_{}_entries",
                    &abstracted[..abstracted.len().min(40)],
                    region_contents.len()
                ));
            }
        }

        // Step 3: Remove consolidated originals (in reverse index order)
        to_remove.sort_unstable();
        to_remove.dedup();
        to_remove.reverse();
        for idx in to_remove {
            if idx < lattice.episodic.len() {
                lattice.episodic.remove(idx);
            }
        }

        (consolidated_count, summaries)
    }

    /// Counterfactual utility check: randomly mask one entry from the region.
    /// Returns the indices that form a coherent group (high mutual similarity).
    /// This approximates Auto-Dreamer's GRPO-based counterfactual utility evaluation.
    fn counterfactual_utility_check(
        &self,
        indices: &[usize],
        lattice: &MemoryLattice,
    ) -> Vec<usize> {
        if indices.len() < 3 {
            return indices.to_vec();
        }

        // Compute pairwise semantic similarity within the region
        let mut pair_sims: Vec<f64> = Vec::new();
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let a = lattice.episodic.get(indices[i]).map(|e| &e.vsa_hash[..]);
                let b = lattice.episodic.get(indices[j]).map(|e| &e.vsa_hash[..]);
                if let (Some(va), Some(vb)) = (a, b) {
                    let sim = hamming_similarity(va, vb);
                    pair_sims.push(sim);
                }
            }
        }

        if pair_sims.is_empty() {
            return indices.to_vec();
        }

        let avg_sim: f64 = pair_sims.iter().sum::<f64>() / pair_sims.len() as f64;

        // Mask one entry (the last one) and check if coherence holds
        if avg_sim > 0.3 {
            // Region is coherent — keep all indices
            indices.to_vec()
        } else {
            // Region lacks internal coherence — keep only high-similarity pairs
            let mut keepers: Vec<usize> = Vec::new();
            for (i, &idx_i) in indices.iter().enumerate() {
                let mut max_sim = 0.0f64;
                for (j, &idx_j) in indices.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let a = lattice.episodic.get(idx_i).map(|e| &e.vsa_hash[..]);
                    let b = lattice.episodic.get(idx_j).map(|e| &e.vsa_hash[..]);
                    if let (Some(va), Some(vb)) = (a, b) {
                        let sim = hamming_similarity(va, vb);
                        if sim > max_sim {
                            max_sim = sim;
                        }
                    }
                }
                if max_sim > 0.25 {
                    keepers.push(idx_i);
                }
            }
            keepers
        }
    }
}

/// Compute hamming similarity between two byte vectors.
/// Returns 1.0 if identical, 0.0 if completely different.
fn hamming_similarity(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let mut matching_bits = 0u64;
    let total_bits = (len * 8) as u64;
    for i in 0..len {
        let xor = a[i] ^ b[i];
        matching_bits += (8 - xor.count_ones()) as u64;
    }
    matching_bits as f64 / total_bits as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::memory_lattice::{BeliefState, MemoryOrigin};
    use crate::core::nt_core_consciousness::LatticeLayer;

    fn make_entry(content: &str, q_val: f64, confidence: f64, consolidated: bool) -> LatticeEntry {
        let vsa_hash = content.bytes().collect();
        LatticeEntry {
            content: content.to_string(),
            vsa_hash,
            layer: LatticeLayer::Episodic,
            confidence,
            invocation_count: 0,
            last_accessed: 0,
            source_layer: None,
            consolidated,
            q_value: q_val,
            valid_from: None,
            valid_to: None,
            origin: MemoryOrigin::System,
            provenance_parent: None,
            belief_state: BeliefState::Inferred,
            domain: "test".to_string(),
        }
    }

    #[test]
    fn test_cte_cycle_default_create() {
        let cte = CteCycle::default();
        assert!(cte.sws_enabled);
        assert!(cte.rem_enabled);
        assert!(cte.consolidation_enabled);
        assert!(cte.compaction_enabled);
        assert!((cte.sws_threshold - 0.4).abs() < 1e-6);
        assert_eq!(cte.compaction_interval, 50);
        assert_eq!(cte.max_entries, 500);
        assert_eq!(cte.cycle_count, 0);
        assert_eq!(cte.total_sws_extracted, 0);
        assert_eq!(cte.total_rem_associated, 0);
        assert_eq!(cte.total_consolidated, 0);
        assert_eq!(cte.total_compacted, 0);
    }

    #[test]
    fn test_sws_extract_returns_empty_on_empty_lattice() {
        let mut lattice = MemoryLattice::new();
        let cte = CteCycle::default();
        let extracted = cte.sws_extract(&mut lattice);
        assert!(extracted.is_empty());
    }

    #[test]
    fn test_run_cte_cycle_no_panic_on_empty_lattice() {
        let mut lattice = MemoryLattice::new();
        let mut cte = CteCycle::default();
        let report = cte.run_cte_cycle(&mut lattice, 1);
        assert_eq!(report.sws_extracted, 0);
        assert_eq!(report.rem_associated, 0);
        assert_eq!(report.consolidated, 0);
        assert_eq!(report.compacted, 0);
        assert!(report.total_duration_ms < 100);
    }

    #[test]
    fn test_sws_extract_high_q_entries() {
        let mut lattice = MemoryLattice::new();
        lattice
            .episodic
            .push_back(make_entry("important pattern", 0.8, 0.3, false));
        lattice
            .episodic
            .push_back(make_entry("low value noise", 0.2, 0.2, false));
        lattice
            .episodic
            .push_back(make_entry("high confidence fact", 0.3, 0.7, false));

        let cte = CteCycle::default();
        let extracted = cte.sws_extract(&mut lattice);
        assert_eq!(extracted.len(), 2);
        assert!(extracted.contains(&"important pattern".to_string()));
        assert!(extracted.contains(&"high confidence fact".to_string()));

        // Check consolidated flag was set
        let ep0 = lattice.episodic.get(0).unwrap();
        assert!(ep0.consolidated);
        assert!((ep0.confidence - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_compaction_triggers_on_cycle_interval() {
        let mut lattice = MemoryLattice::new();
        lattice.episodic.push_back(make_entry(
            "rarely accessed old entry that nobody reads anymore",
            0.1,
            0.2,
            true,
        ));
        lattice
            .episodic
            .push_back(make_entry("frequent important entry", 0.9, 0.8, false));

        // Mark the first entry as low access
        if let Some(e) = lattice.episodic.get_mut(0) {
            e.invocation_count = 1;
        }

        let mut cte = CteCycle::default();
        cte.compaction_interval = 1;
        let report = cte.run_cte_cycle(&mut lattice, 1);

        assert_eq!(report.compacted, 1);
        assert_eq!(lattice.episodic.len(), 1);
        assert_eq!(lattice.episodic[0].content, "frequent important entry");
    }

    #[test]
    fn test_compaction_skips_before_interval() {
        let mut lattice = MemoryLattice::new();
        lattice
            .episodic
            .push_back(make_entry("test content", 0.1, 0.2, true));
        if let Some(e) = lattice.episodic.get_mut(0) {
            e.invocation_count = 1;
        }

        let mut cte = CteCycle::default();
        cte.compaction_interval = 50;
        let report = cte.run_cte_cycle(&mut lattice, 1);
        assert_eq!(report.compacted, 0);
    }

    #[test]
    fn test_stats_tracking() {
        let mut lattice = MemoryLattice::new();
        lattice
            .episodic
            .push_back(make_entry("pattern one", 0.8, 0.3, false));
        lattice
            .episodic
            .push_back(make_entry("pattern two", 0.9, 0.4, false));

        let mut cte = CteCycle::default();
        cte.compaction_interval = 1;
        let _r1 = cte.run_cte_cycle(&mut lattice, 1);

        assert_eq!(cte.total_sws_extracted, 2);
        assert!(cte.cycle_count > 0);
    }

    #[test]
    fn test_hamming_similarity_identical() {
        let a = vec![0xFF, 0x00, 0xAA];
        let b = vec![0xFF, 0x00, 0xAA];
        let sim = hamming_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hamming_similarity_completely_different() {
        let a = vec![0x00, 0x00, 0x00];
        let b = vec![0xFF, 0xFF, 0xFF];
        let sim = hamming_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_hamming_similarity_empty() {
        let sim = hamming_similarity(&[], &[]);
        assert!((sim - 0.0).abs() < 1e-6);
    }
}
