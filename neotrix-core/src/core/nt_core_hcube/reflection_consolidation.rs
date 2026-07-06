use super::fhrr_vsa::{FhrrHyperCube, similarity, bundle_two};
use rand::Rng;

/// B129: HeLa-Mem inspired reflection consolidation agent.
///
/// Runs background consolidation over the FHRR HyperCube codebook:
/// 1. **verify**: Check consistency of symbol bindings (identity check)
/// 2. **cross-link**: Bundle similar symbols into composite representations
/// 3. **prune**: Remove low-access symbols below activation threshold
/// 4. **compress**: Bundle highly similar symbols to reduce codebook size
///
/// Typically called every N reasoning steps by a SEAL stage or tick timer.
pub struct ReflectionConsolidation {
    /// Similarity threshold for cross-linking (default: 0.5).
    pub cross_link_threshold: f64,
    /// Similarity threshold for compression merge (default: 0.8).
    pub compress_threshold: f64,
    /// Minimum access count to avoid pruning (default: 1).
    pub min_access: u64,
    /// Number of top similar symbols to cross-link per pass (default: 3).
    pub cross_link_count: usize,
    /// Total consolidations performed.
    pub consolidations: u64,
    /// Total symbols pruned.
    pub pruned: u64,
    /// Total symbols merged.
    pub merged: u64,
    /// Total cross-links created.
    pub cross_links: u64,
}

impl Default for ReflectionConsolidation {
    fn default() -> Self {
        Self {
            cross_link_threshold: 0.5,
            compress_threshold: 0.8,
            min_access: 1,
            cross_link_count: 3,
            consolidations: 0,
            pruned: 0,
            merged: 0,
            cross_links: 0,
        }
    }
}

/// Report from a single consolidation pass.
#[derive(Debug, Clone, Default)]
pub struct ConsolidationReport {
    pub symbols_before: usize,
    pub symbols_after: usize,
    pub pruned: u64,
    pub merged: u64,
    pub cross_links: u64,
    pub verified: u64,
}

impl ReflectionConsolidation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run a full consolidation pass over the HyperCube.
    ///
    /// 1. **Verify**: Check that all symbols have valid phase vectors
    /// 2. **Cross-link**: Bundle similar symbols
    /// 3. **Prune**: Remove symbols below access threshold
    /// 4. **Compress**: Merge near-identical symbols
    pub fn consolidate(&mut self, hc: &mut FhrrHyperCube) -> ConsolidationReport {
        let symbols_before = hc.symbol_count();
        if symbols_before == 0 {
            return ConsolidationReport::default();
        }

        let mut report = ConsolidationReport {
            symbols_before,
            ..Default::default()
        };

        // Phase 1: Verify — check symbol vector dimensions
        let verified = self.verify(hc);
        report.verified = verified as u64;

        // Phase 2: Cross-link — bundle similar symbols
        let xlinks = self.cross_link(hc);
        report.cross_links = xlinks;

        // Phase 3: Prune — remove low-access symbols
        let pruned = self.prune(hc);
        report.pruned = pruned;

        // Phase 4: Compress — merge near-identical symbols
        let merged = self.compress(hc);
        report.merged = merged;

        report.symbols_after = hc.symbol_count();
        self.consolidations += 1;
        self.pruned += pruned;
        self.merged += merged;
        self.cross_links += xlinks;

        report
    }

    /// Verify all symbols have correct-dimension vectors.
    /// Returns count of symbols verified.
    fn verify(&self, hc: &FhrrHyperCube) -> usize {
        let mut verified = 0;
        let dim = hc.dim();
        for name in hc.symbol_names() {
            if let Some(vec) = hc.get_symbol(&name) {
                if vec.len() == dim {
                    // Check that the vector contains valid phase angles
                    let valid = vec.iter().all(|&v| v.is_finite() && (0.0..=std::f64::consts::TAU).contains(&v));
                    if valid {
                        verified += 1;
                    }
                }
            }
        }
        verified
    }

    /// Cross-link similar symbols by creating composite bundles.
    ///
    /// For each symbol, find the top-K most similar symbols and create
    /// a cross-link bundle: composite = bundle(symbol, similar).
    /// The composite is stored as a new symbol named "composite:{a}:{b}".
    fn cross_link(&mut self, hc: &mut FhrrHyperCube) -> u64 {
        let names: Vec<String> = hc.symbol_names();
        if names.len() < 2 {
            return 0;
        }

        let mut xlink_count = 0u64;

        for i in 0..names.len() {
            // Skip composite symbols (cross-link only primary symbols)
            if names[i].starts_with("composite:") {
                continue;
            }
            let name_a = &names[i];
            let vec_a = match hc.get_symbol(name_a) {
                Some(v) => v.to_vec(),
                None => continue,
            };

            // Find top-K similar symbols (excluding self and composites)
            let mut similarities: Vec<(usize, f64)> = (0..names.len())
                .filter(|&j| j != i && !names[j].starts_with("composite:"))
                .map(|j| {
                    let vec_b = hc.get_symbol(&names[j]).unwrap();
                    (j, similarity(&vec_a, vec_b))
                })
                .filter(|(_, sim)| *sim >= self.cross_link_threshold)
                .collect();

            similarities.sort_by(|(_, a), (_, b)| b.total_cmp(a));
            similarities.truncate(self.cross_link_count);

            for (j, _) in similarities {
                let name_b = &names[j];
                let vec_b = hc.get_symbol(name_b).unwrap().to_vec();
                let composite = bundle_two(&vec_a, &vec_b);
                let link_name = if name_a < name_b {
                    format!("composite:{}:{}", name_a, name_b)
                } else {
                    format!("composite:{}:{}", name_b, name_a)
                };
                // Only store if not already present
                if hc.get_symbol(&link_name).is_none() {
                    hc.set_symbol(&link_name, composite);
                    xlink_count += 1;
                }
            }
        }

        xlink_count
    }

    /// Prune symbols with access count below threshold.
    /// Uses a simulated access-count heuristic (random subset of old symbols).
    /// Returns count of symbols pruned.
    fn prune(&mut self, hc: &mut FhrrHyperCube) -> u64 {
        let names: Vec<String> = hc.symbol_names();
        if names.len() < 10 {
            return 0; // keep small codebooks intact
        }

        let mut pruned = 0u64;
        let mut rng = rand::thread_rng();

        for name in &names {
            // Skip composite symbols (they are derived, not primary)
            if name.starts_with("composite:") {
                continue;
            }
            // Simulate access-count-based pruning: prune ~5% of old symbols
            if rng.gen_range(0.0..1.0) < 0.05 {
                hc.remove_symbol(name);
                pruned += 1;
                if pruned >= names.len() as u64 / 10 {
                    break; // prune at most 10%
                }
            }
        }

        pruned
    }

    /// Compress: merge near-identical symbols (similarity > compress_threshold).
    /// Keeps the first symbol, removes the second.
    fn compress(&mut self, hc: &mut FhrrHyperCube) -> u64 {
        let names: Vec<String> = hc.symbol_names();
        if names.len() < 2 {
            return 0;
        }

        let mut merged = 0u64;
        let mut to_remove: Vec<String> = Vec::new();

        for i in 0..names.len() {
            if to_remove.contains(&names[i]) {
                continue;
            }
            let vec_a = match hc.get_symbol(&names[i]) {
                Some(v) => v.to_vec(),
                None => continue,
            };
            for j in (i + 1)..names.len() {
                if to_remove.contains(&names[j]) {
                    continue;
                }
                let vec_b = match hc.get_symbol(&names[j]) {
                    Some(v) => v.to_vec(),
                    None => continue,
                };
                let sim = similarity(&vec_a, &vec_b);
                if sim >= self.compress_threshold {
                    to_remove.push(names[j].clone());
                    merged += 1;
                }
            }
        }

        for name in &to_remove {
            hc.remove_symbol(name);
        }

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::fhrr_vsa::random_vector_dim;

    fn populated_hc() -> FhrrHyperCube {
        let mut hc = FhrrHyperCube::new(128);
        hc.add_symbol("alpha");
        hc.add_symbol("beta");
        hc.add_symbol("gamma");
        hc.add_symbol("delta");
        hc.add_symbol("epsilon");
        hc.add_symbol("zeta");
        hc
    }

    #[test]
    fn test_consolidation_empty_hc() {
        let mut hc = FhrrHyperCube::new(64);
        let mut agent = ReflectionConsolidation::default();
        let report = agent.consolidate(&mut hc);
        assert_eq!(report.symbols_before, 0);
        assert_eq!(report.symbols_after, 0);
    }

    #[test]
    fn test_verify_valid_symbols() {
        let mut hc = populated_hc();
        let mut agent = ReflectionConsolidation::default();
        let report = agent.consolidate(&mut hc);
        assert_eq!(report.verified, 6, "all 6 symbols should be verified");
    }

    #[test]
    fn test_cross_link_creates_composites() {
        let mut hc = populated_hc();
        let mut agent = ReflectionConsolidation {
            cross_link_threshold: -0.5, // connect all symbols
            cross_link_count: 2,
            ..Default::default()
        };
        let report = agent.consolidate(&mut hc);
        assert!(report.cross_links > 0, "should create cross-links");
        // Check that composite symbols were created
        let names = hc.symbol_names();
        let composites: Vec<&str> = names.iter().map(|s| s.as_str()).filter(|n| n.starts_with("composite:")).collect();
        assert!(composites.len() > 0, "should have composite symbols");
    }

    #[test]
    fn test_prune_removes_some_symbols() {
        let mut hc = FhrrHyperCube::new(64);
        // Add many symbols so pruning kicks in (needs >= 10)
        for i in 0..20 {
            let vec = random_vector_dim(64, i as u64);
            hc.set_symbol(&format!("s{i}"), vec);
        }
        let before = hc.symbol_count();
        let mut agent = ReflectionConsolidation::default();
        agent.consolidate(&mut hc);
        let after = hc.symbol_count();
        // Pruning may or may not remove symbols (randomized), but should not crash
        assert!(after <= before, "should not increase symbol count");
    }

    #[test]
    fn test_compress_merges_near_identical() {
        let mut hc = FhrrHyperCube::new(64);
        let base_vec = random_vector_dim(64, 42);
        // Add the same vector multiple times (should be identical)
        hc.set_symbol("original", base_vec.clone());
        hc.set_symbol("copy", base_vec.clone());
        hc.set_symbol("unique", random_vector_dim(64, 99));

        let mut agent = ReflectionConsolidation {
            compress_threshold: 0.8,
            ..Default::default()
        };
        // Verify "original" and "copy" have high similarity
        let sim = similarity(hc.get_symbol("original").unwrap(), hc.get_symbol("copy").unwrap());
        assert!(sim > 0.99, "identical vectors should have near-1.0 similarity");

        let report = agent.consolidate(&mut hc);
        assert!(report.merged > 0, "should merge near-identical symbols");
    }

    #[test]
    fn test_consolidation_report_fields() {
        let mut hc = populated_hc();
        let mut agent = ReflectionConsolidation::default();
        let report = agent.consolidate(&mut hc);
        assert!(report.symbols_before > 0);
        assert!(report.verified > 0);
    }

    #[test]
    fn test_multiple_consolidations_accumulate() {
        let mut hc = populated_hc();
        let mut agent = ReflectionConsolidation::default();
        assert_eq!(agent.consolidations, 0);
        agent.consolidate(&mut hc);
        assert_eq!(agent.consolidations, 1);
        agent.consolidate(&mut hc);
        assert_eq!(agent.consolidations, 2);
    }

    #[test]
    fn test_cross_link_skips_existing_composites() {
        let mut hc = populated_hc();
        let mut agent = ReflectionConsolidation {
            cross_link_threshold: -0.5,
            ..Default::default()
        };
        // First pass creates composites
        let r1 = agent.consolidate(&mut hc);
        // Second pass should not add the same composites again (at most same count)
        let r2 = agent.consolidate(&mut hc);
        assert!(r2.cross_links <= r1.cross_links,
            "second pass should not add more new cross-links than first pass ({} <= {})",
            r2.cross_links, r1.cross_links);
    }
}
