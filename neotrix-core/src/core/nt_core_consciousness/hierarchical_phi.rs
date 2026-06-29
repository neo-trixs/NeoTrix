// REVIVED Task 1 — dead_code removed 2026-06-24

use super::iit_phi::{FactoredTPM, PhiCalculator};
use rand::Rng;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ScaledPhiResult {
    pub phi_micro: f64,
    pub phi_meso: f64,
    pub phi_macro: f64,
    pub phi_system: f64,
    pub phi_star: f64,
    pub main_complex: Vec<usize>,
    pub tick: u64,
}

impl ScaledPhiResult {
    pub fn new(
        phi_micro: f64,
        phi_meso: f64,
        phi_macro: f64,
        phi_system: f64,
        main_complex: Vec<usize>,
        tick: u64,
    ) -> Self {
        let vals = [phi_micro, phi_meso, phi_macro, phi_system];
        let phi_star = vals.iter().cloned().fold(0.0_f64, f64::max);
        ScaledPhiResult {
            phi_micro,
            phi_meso,
            phi_macro,
            phi_system,
            phi_star,
            main_complex,
            tick,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhiProfile {
    pub history: Vec<ScaledPhiResult>,
    pub max_len: usize,
}

impl PhiProfile {
    pub fn new(max_len: usize) -> Self {
        PhiProfile {
            history: Vec::with_capacity(max_len.min(1000)),
            max_len,
        }
    }

    pub fn push(&mut self, result: ScaledPhiResult) {
        if self.history.len() >= self.max_len {
            self.history.remove(0);
        }
        self.history.push(result);
    }

    pub fn phi_star_smoothed(&self, window: usize) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        let w = window.min(n);
        let sum: f64 = self.history.iter().rev().take(w).map(|r| r.phi_star).sum();
        sum / w as f64
    }

    pub fn phi_drop_detected(&self, threshold: f64) -> bool {
        let n = self.history.len();
        if n < 3 {
            return false;
        }
        let recent = self.history[n - 1].phi_star;
        let prev_avg = self
            .history
            .iter()
            .rev()
            .skip(1)
            .take(2)
            .map(|r| r.phi_star)
            .sum::<f64>()
            / 2.0;
        prev_avg - recent > threshold
    }

    pub fn trend(&self) -> f64 {
        let n = self.history.len();
        if n < 2 {
            return 0.0;
        }
        let indices: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let vals: Vec<f64> = self.history.iter().map(|r| r.phi_star).collect();
        let mean_i = indices.iter().sum::<f64>() / n as f64;
        let mean_v = vals.iter().sum::<f64>() / n as f64;
        let num: f64 = indices
            .iter()
            .zip(vals.iter())
            .map(|(&i, &v)| (i - mean_i) * (v - mean_v))
            .sum();
        let den: f64 = indices.iter().map(|&i| (i - mean_i).powi(2)).sum();
        if den.abs() < 1e-10 {
            0.0
        } else {
            num / den
        }
    }

    pub fn average(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        self.history.iter().map(|r| r.phi_star).sum::<f64>() / n as f64
    }

    pub fn phi_micro_average(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        self.history.iter().map(|r| r.phi_micro).sum::<f64>() / n as f64
    }

    pub fn phi_meso_average(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        self.history.iter().map(|r| r.phi_meso).sum::<f64>() / n as f64
    }

    pub fn phi_macro_average(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        self.history.iter().map(|r| r.phi_macro).sum::<f64>() / n as f64
    }

    pub fn phi_system_average(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        self.history.iter().map(|r| r.phi_system).sum::<f64>() / n as f64
    }

    pub fn phi_star_max(&self) -> f64 {
        self.history
            .iter()
            .map(|r| r.phi_star)
            .fold(0.0_f64, f64::max)
    }

    pub fn phi_star_min(&self) -> f64 {
        self.history
            .iter()
            .map(|r| r.phi_star)
            .fold(f64::INFINITY, f64::min)
    }
}

#[derive(Debug, Clone)]
pub struct HierarchicalPhi {
    pub tpm: FactoredTPM,
    n: usize,
    pub cluster_map: Vec<usize>,
}

impl HierarchicalPhi {
    pub fn new(tpm: FactoredTPM) -> Self {
        let n = tpm.n;
        let cluster_map: Vec<usize> = (0..n).collect();
        HierarchicalPhi {
            tpm,
            n,
            cluster_map,
        }
    }

    pub fn with_clusters(tpm: FactoredTPM, clusters: Vec<usize>) -> Self {
        let n = tpm.n;
        HierarchicalPhi {
            tpm,
            n,
            cluster_map: clusters,
        }
    }

    pub fn auto_build_hierarchy(&mut self) {
        let n = self.n;
        let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();

        loop {
            if clusters.len() <= 1 {
                break;
            }
            let mut best_merge = (0usize, 0usize);
            let mut best_score = -1.0_f64;

            for i in 0..clusters.len() {
                for j in (i + 1)..clusters.len() {
                    let score = self.cluster_affinity(&clusters[i], &clusters[j]);
                    if score > best_score {
                        best_score = score;
                        best_merge = (i, j);
                    }
                }
            }

            if best_score < 0.01 {
                break;
            }

            let merged: Vec<usize> = {
                let mut combined = clusters[best_merge.0].clone();
                combined.extend(&clusters[best_merge.1]);
                combined
            };

            let j = best_merge.1;
            let i = best_merge.0;
            if i < j {
                clusters.remove(j);
                clusters.remove(i);
            } else {
                clusters.remove(i);
                clusters.remove(j);
            }
            clusters.push(merged);
        }

        let mut new_map = vec![0usize; n];
        for (cid, members) in clusters.iter().enumerate() {
            for &m in members {
                new_map[m] = cid;
            }
        }
        self.cluster_map = new_map;
    }

    fn cluster_affinity(&self, a: &[usize], b: &[usize]) -> f64 {
        let mut total = 0.0;
        let mut count = 0;
        for &ai in a {
            for &bj in b {
                let connected = self.tpm.nodes[ai].parents.contains(&bj)
                    || self.tpm.nodes[bj].parents.contains(&ai);
                if connected {
                    total += 1.0;
                }
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            total / count as f64
        }
    }

    pub fn cluster_ids(&self) -> HashSet<usize> {
        self.cluster_map.iter().copied().collect()
    }

    pub fn num_clusters(&self) -> usize {
        self.cluster_ids().len()
    }

    pub fn cluster_members(&self) -> Vec<Vec<usize>> {
        let ids = self.cluster_ids();
        let mut members: Vec<Vec<usize>> = Vec::new();
        let mut sorted: Vec<usize> = ids.into_iter().collect();
        sorted.sort_unstable();
        for cid in sorted {
            let m: Vec<usize> = self
                .cluster_map
                .iter()
                .enumerate()
                .filter(|(_, &c)| c == cid)
                .map(|(i, _)| i)
                .collect();
            members.push(m);
        }
        members
    }

    pub fn compute_all_scales(&self, state: &[u8], tick: u64) -> ScaledPhiResult {
        let phi_micro = self.compute_micro_phi(state);
        let phi_meso = self.compute_meso_phi(state);
        let phi_macro = self.compute_macro_phi(state);
        let phi_system = self.compute_system_phi(state);
        let main_complex = self.find_main_complex(state);
        ScaledPhiResult::new(
            phi_micro,
            phi_meso,
            phi_macro,
            phi_system,
            main_complex,
            tick,
        )
    }

    pub fn compute_micro_phi(&self, state: &[u8]) -> f64 {
        let calc = PhiCalculator::new(self.tpm.clone());
        calc.compute_phi(state)
    }

    pub fn compute_meso_phi(&self, state: &[u8]) -> f64 {
        let members = self.cluster_members();
        let meso_clusters: Vec<Vec<usize>> = members
            .into_iter()
            .filter(|m| m.len() >= 2 && m.len() <= 4)
            .collect();
        if meso_clusters.is_empty() {
            return self.compute_micro_phi(state);
        }
        self.compute_clustered_phi(state, &meso_clusters)
    }

    pub fn compute_macro_phi(&self, state: &[u8]) -> f64 {
        let members = self.cluster_members();
        let macro_clusters: Vec<Vec<usize>> =
            members.into_iter().filter(|m| m.len() >= 5).collect();
        if macro_clusters.is_empty() {
            return self.compute_meso_phi(state);
        }
        self.compute_clustered_phi(state, &macro_clusters)
    }

    pub fn compute_system_phi(&self, state: &[u8]) -> f64 {
        let all: Vec<usize> = (0..self.n).collect();
        self.compute_clustered_phi(state, &[all])
    }

    pub fn compute_clustered_phi(&self, state: &[u8], clusters: &[Vec<usize>]) -> f64 {
        let k = clusters.len();
        if k <= 1 {
            return 0.0;
        }
        let cluster_states: Vec<u8> = clusters
            .iter()
            .map(|c| {
                let ones = c
                    .iter()
                    .filter(|&&i| state.get(i).copied().unwrap_or(0) != 0)
                    .count();
                if ones > c.len() / 2 {
                    1
                } else {
                    0
                }
            })
            .collect();

        let deps: Vec<Vec<usize>> = (0..k)
            .map(|i| {
                let mut parents: HashSet<usize> = HashSet::new();
                for &node_i in &clusters[i] {
                    for p in &self.tpm.nodes[node_i].parents {
                        for (j, c) in clusters.iter().enumerate() {
                            if j != i && c.contains(p) {
                                parents.insert(j);
                            }
                        }
                    }
                }
                let mut sorted: Vec<usize> = parents.into_iter().collect();
                sorted.sort_unstable();
                sorted
            })
            .collect();

        let n_cluster = k;
        if n_cluster <= 1 {
            return 0.0;
        }
        let mut nodes: Vec<crate::core::nt_core_consciousness::iit_phi::NodeCPT> =
            Vec::with_capacity(n_cluster);
        for i in 0..n_cluster {
            let parents = deps[i].clone();
            let mut cpt = crate::core::nt_core_consciousness::iit_phi::NodeCPT::new(parents);
            for idx in 0..cpt.prob_one.len() {
                cpt.prob_one[idx] = 0.5;
            }
            nodes.push(cpt);
        }
        let cluster_tpm = FactoredTPM {
            nodes,
            n: n_cluster,
        };
        let calc = PhiCalculator::new(cluster_tpm);
        calc.compute_phi(&cluster_states)
    }

    #[allow(non_snake_case)]
    pub fn compute_K_phi(&self, state: &[u8], k: usize) -> f64 {
        if k <= 1 || k > self.n {
            return 0.0;
        }
        let mut rng = rand::thread_rng();
        let mut assignments: Vec<usize> = Vec::with_capacity(self.n);
        for _ in 0..self.n {
            assignments.push(rng.gen_range(0..k) as usize);
        }
        if assignments.iter().collect::<HashSet<_>>().len() < k.min(self.n) {
            for i in 0..self.n {
                assignments[i] = i % k;
            }
        }
        let mut cluster_groups: Vec<Vec<usize>> = (0..k).map(|_| Vec::new()).collect();
        for (i, &cid) in assignments.iter().enumerate() {
            if cid < k {
                cluster_groups[cid].push(i);
            }
        }
        let mut clusters = cluster_groups;
        clusters.retain(|c| !c.is_empty());
        self.compute_clustered_phi(state, &clusters)
    }

    pub fn find_main_complex(&self, state: &[u8]) -> Vec<usize> {
        if self.n <= 8 {
            let calc = PhiCalculator::new(self.tpm.clone());
            let (partition, _) = calc.compute_mip(state);
            return partition;
        }
        (0..self.n).collect()
    }

    pub fn phi_star(&self, state: &[u8]) -> f64 {
        let result = self.compute_all_scales(state, 0);
        result.phi_star
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::iit_phi::FactoredTPM;

    fn assert_approx_eq(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "|{} - {}| < {}", a, b, eps);
    }

    #[test]
    fn test_profile_new_and_push() {
        let mut profile = PhiProfile::new(5);
        assert_eq!(profile.history.len(), 0);
        for i in 0..5 {
            profile.push(ScaledPhiResult::new(0.1, 0.2, 0.3, 0.4, vec![i], i as u64));
        }
        assert_eq!(profile.history.len(), 5);
        profile.push(ScaledPhiResult::new(0.5, 0.6, 0.7, 0.8, vec![], 5));
        assert_eq!(profile.history.len(), 5);
    }

    #[test]
    fn test_profile_average() {
        let mut profile = PhiProfile::new(10);
        for i in 0..4 {
            profile.push(ScaledPhiResult::new(0.0, 0.0, 0.0, 0.0, vec![], i));
        }
        assert_approx_eq(profile.average(), 0.0, 1e-6);
        profile.push(ScaledPhiResult::new(0.0, 0.0, 0.0, 1.0, vec![], 4));
        assert!(profile.average() > 0.0);
    }

    #[test]
    fn test_profile_trend() {
        let mut profile = PhiProfile::new(10);
        for i in 0..5 {
            profile.push(ScaledPhiResult::new(
                0.0,
                0.0,
                0.0,
                i as f64 * 0.1,
                vec![],
                i,
            ));
        }
        let t = profile.trend();
        assert!(t > 0.0, "increasing trend should be positive, got {}", t);
    }

    #[test]
    fn test_phi_drop_detected() {
        let mut profile = PhiProfile::new(10);
        for i in 0..5 {
            profile.push(ScaledPhiResult::new(0.5, 0.5, 0.5, 0.5, vec![], i));
        }
        assert!(!profile.phi_drop_detected(0.3));
        profile.push(ScaledPhiResult::new(0.05, 0.05, 0.05, 0.05, vec![], 5));
        assert!(profile.phi_drop_detected(0.3));
    }

    #[test]
    fn test_hierarchical_micro_phi_basic() {
        let tpm = FactoredTPM::chain(6);
        let hier = HierarchicalPhi::new(tpm);
        let state = [0u8; 6];
        let micro = hier.compute_micro_phi(&state);
        assert!(micro >= 0.0, "micro phi should be >= 0");
    }

    #[test]
    fn test_hierarchical_phi_star_is_max() {
        let tpm = FactoredTPM::fully_connected(8);
        let mut hier = HierarchicalPhi::new(tpm);
        hier.auto_build_hierarchy();
        let state = [0u8; 8];
        let result = hier.compute_all_scales(&state, 0);
        assert!(result.phi_star >= result.phi_micro - 1e-10);
        assert!(result.phi_star >= result.phi_meso - 1e-10);
        assert!(result.phi_star >= result.phi_macro - 1e-10);
        assert!(result.phi_star >= result.phi_system - 1e-10);
    }

    #[test]
    fn test_auto_build_hierarchy() {
        let tpm = FactoredTPM::chain(10);
        let mut hier = HierarchicalPhi::new(tpm);
        hier.auto_build_hierarchy();
        let n_clusters = hier.num_clusters();
        assert!(n_clusters >= 1, "should have at least 1 cluster");
        assert!(n_clusters <= 10, "should not exceed node count");
    }

    #[test]
    fn test_clustered_phi_on_disconnected() {
        let tpm = FactoredTPM::disconnected(6);
        let hier = HierarchicalPhi::new(tpm);
        let state = [0u8; 6];
        let clusters = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let phi = hier.compute_clustered_phi(&state, &clusters);
        assert_approx_eq(phi, 0.0, 0.15);
    }

    #[test]
    fn test_phi_smoothed_unchanged() {
        let mut profile = PhiProfile::new(10);
        for i in 0..5 {
            profile.push(ScaledPhiResult::new(0.1, 0.2, 0.3, 0.5, vec![], i));
        }
        let smoothed = profile.phi_star_smoothed(3);
        assert_approx_eq(smoothed, 0.5, 1e-10);
    }

    #[test]
    fn test_hierarchical_full_connected_highest() {
        let tpm_chain = FactoredTPM::chain(6);
        let tpm_full = FactoredTPM::fully_connected(6);
        let hier_chain = HierarchicalPhi::new(tpm_chain);
        let hier_full = HierarchicalPhi::new(tpm_full);
        let state = [0u8; 6];
        let chain_phi = hier_chain.phi_star(&state);
        let full_phi = hier_full.phi_star(&state);
        assert!(
            full_phi >= chain_phi - 0.1,
            "fully connected phi_star >= chain phi_star"
        );
    }

    #[test]
    fn test_profile_push_truncates() {
        let mut profile = PhiProfile::new(3);
        for i in 0..5 {
            profile.push(ScaledPhiResult::new(
                0.0,
                0.0,
                0.0,
                i as f64 * 0.1,
                vec![],
                i,
            ));
        }
        assert_eq!(profile.history.len(), 3);
        assert!(profile.history[0].phi_system >= 0.2);
    }

    #[test]
    fn test_hierarchical_cluster_map_after_build() {
        let tpm = FactoredTPM::chain(8);
        let mut hier = HierarchicalPhi::new(tpm);
        hier.auto_build_hierarchy();
        assert_eq!(hier.cluster_map.len(), 8);
        let unique: HashSet<usize> = hier.cluster_map.iter().copied().collect();
        assert!(unique.len() <= 8);
        assert!(unique.len() >= 1);
    }

    #[test]
    fn test_K_phi_decreases_with_larger_K() {
        let tpm = FactoredTPM::fully_connected(6);
        let hier = HierarchicalPhi::new(tpm);
        let state = [0u8; 6];
        let phi_k2 = hier.compute_K_phi(&state, 2);
        let phi_k4 = hier.compute_K_phi(&state, 4);
        assert!(phi_k2 >= phi_k4 - 0.2, "phi(K=2) should >= phi(K=4)");
    }

    #[test]
    fn test_scaled_result_phi_star() {
        let r = ScaledPhiResult::new(0.1, 0.3, 0.5, 0.4, vec![], 0);
        assert_approx_eq(r.phi_star, 0.5, 1e-10);
    }

    #[test]
    fn test_profile_empty_returns_zero() {
        let profile = PhiProfile::new(10);
        assert_eq!(profile.average(), 0.0);
        assert_eq!(profile.phi_star_smoothed(5), 0.0);
        assert_eq!(profile.trend(), 0.0);
        assert!(!profile.phi_drop_detected(0.5));
    }
}

// ── G206: Multi-Scale Phi (K=8) ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct MultiScaleConfig {
    pub k: usize,
    pub micro_nodes: usize,
    pub meso_nodes: usize,
    pub macro_nodes: usize,
}

impl Default for MultiScaleConfig {
    fn default() -> Self {
        MultiScaleConfig {
            k: 8,
            micro_nodes: 16,
            meso_nodes: 32,
            macro_nodes: 64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiScalePhi {
    pub hier: HierarchicalPhi,
    pub config: MultiScaleConfig,
    partition_cache: Option<Vec<Vec<usize>>>,
}

impl MultiScalePhi {
    pub fn new(config: MultiScaleConfig) -> Self {
        let n = config.macro_nodes;
        let tpm = FactoredTPM::chain(n);
        let hier = HierarchicalPhi::new(tpm);
        MultiScalePhi {
            hier,
            config,
            partition_cache: None,
        }
    }

    pub fn with_tpm(tpm: FactoredTPM, config: MultiScaleConfig) -> Self {
        let hier = HierarchicalPhi::new(tpm);
        MultiScalePhi {
            hier,
            config,
            partition_cache: None,
        }
    }

    fn node_affinity(tpm: &FactoredTPM, a: usize, b: usize) -> f64 {
        let connected = tpm.nodes[a].parents.contains(&b) || tpm.nodes[b].parents.contains(&a);
        if connected {
            1.0
        } else {
            0.0
        }
    }

    pub fn partition_k8(&self) -> Vec<Vec<usize>> {
        let n = self.hier.tpm.n;
        let k = self.config.k.min(n);
        let mut clusters: Vec<Vec<usize>> = (0..k).map(|_| Vec::new()).collect();

        let mut unassigned: Vec<usize> = (0..n).collect();
        unassigned.sort_by(|&a, &b| {
            let sum_a: f64 = (0..n)
                .map(|j| {
                    if j == a {
                        0.0
                    } else {
                        Self::node_affinity(&self.hier.tpm, a, j)
                    }
                })
                .sum();
            let sum_b: f64 = (0..n)
                .map(|j| {
                    if j == b {
                        0.0
                    } else {
                        Self::node_affinity(&self.hier.tpm, b, j)
                    }
                })
                .sum();
            sum_b
                .partial_cmp(&sum_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for &node in &unassigned {
            let mut best_cid = 0;
            let mut best_score = -1.0_f64;
            for cid in 0..k {
                let members = &clusters[cid];
                let avg = if members.is_empty() {
                    0.0
                } else {
                    members
                        .iter()
                        .map(|&m| Self::node_affinity(&self.hier.tpm, node, m))
                        .sum::<f64>()
                        / members.len() as f64
                };
                if avg > best_score {
                    best_score = avg;
                    best_cid = cid;
                }
            }
            clusters[best_cid].push(node);
        }

        clusters
    }

    pub fn compute_all_scales(&mut self) -> ScaledPhiResult {
        let partition = self.partition_k8();
        self.partition_cache = Some(partition);

        let phi_micro = self.scale_integrated_info(0);
        let phi_meso = self.scale_integrated_info(1);
        let phi_macro = self.scale_integrated_info(2);

        let state = vec![0u8; self.hier.tpm.n];
        let phi_system = self.hier.compute_system_phi(&state);
        let main_complex = self.hier.find_main_complex(&state);

        ScaledPhiResult::new(phi_micro, phi_meso, phi_macro, phi_system, main_complex, 0)
    }

    pub fn scale_integrated_info(&self, scale: usize) -> f64 {
        let partition = match &self.partition_cache {
            Some(p) => p.clone(),
            None => self.partition_k8(),
        };

        let n_nodes = match scale {
            0 => self.config.micro_nodes,
            1 => self.config.meso_nodes,
            2 => self.config.macro_nodes,
            _ => self.hier.tpm.n,
        };

        let n = self.hier.tpm.n.min(n_nodes);
        let state = vec![0u8; n];
        let constrained: Vec<Vec<usize>> = partition
            .into_iter()
            .map(|c| c.into_iter().filter(|&i| i < n).collect())
            .filter(|c: &Vec<usize>| !c.is_empty())
            .collect();

        if constrained.len() <= 1 {
            return 0.0;
        }

        self.hier.compute_clustered_phi(&state, &constrained)
    }
}

#[cfg(test)]
mod multi_scale_tests {
    use super::*;

    fn assert_approx_eq(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "|{} - {}| < {}", a, b, eps);
    }

    #[test]
    fn test_multi_scale_config_defaults() {
        let config = MultiScaleConfig::default();
        assert_eq!(config.k, 8);
        assert_eq!(config.micro_nodes, 16);
        assert_eq!(config.meso_nodes, 32);
        assert_eq!(config.macro_nodes, 64);
    }

    #[test]
    fn test_k8_partition_count() {
        let config = MultiScaleConfig::default();
        let msp = MultiScalePhi::new(config);
        let partition = msp.partition_k8();
        assert_eq!(partition.len(), 8);
        for (i, cluster) in partition.iter().enumerate() {
            assert!(!cluster.is_empty(), "Cluster {} should not be empty", i);
        }
    }

    #[test]
    fn test_compute_all_scales_valid() {
        let config = MultiScaleConfig {
            macro_nodes: 16,
            meso_nodes: 16,
            micro_nodes: 16,
            ..Default::default()
        };
        let mut msp = MultiScalePhi::new(config);
        let result = msp.compute_all_scales();
        assert!(result.phi_micro >= 0.0);
        assert!(result.phi_meso >= 0.0);
        assert!(result.phi_macro >= 0.0);
        assert!(result.phi_system >= 0.0);
        assert!(result.phi_star >= result.phi_micro - 1e-10);
        assert!(result.phi_star >= result.phi_meso - 1e-10);
        assert!(result.phi_star >= result.phi_macro - 1e-10);
    }

    #[test]
    fn test_scale_integrated_info_disconnected() {
        let tpm = FactoredTPM::disconnected(16);
        let config = MultiScaleConfig {
            micro_nodes: 16,
            meso_nodes: 16,
            macro_nodes: 16,
            ..Default::default()
        };
        let msp = MultiScalePhi::with_tpm(tpm, config);
        let phi = msp.scale_integrated_info(0);
        assert_approx_eq(phi, 0.0, 0.15);
    }

    #[test]
    fn test_micro_meso_macro_consistency() {
        let config = MultiScaleConfig {
            macro_nodes: 16,
            meso_nodes: 16,
            micro_nodes: 16,
            ..Default::default()
        };
        let mut msp = MultiScalePhi::new(config);
        let result = msp.compute_all_scales();
        assert!(result.phi_macro >= result.phi_micro - 0.2);
    }

    #[test]
    fn test_large_scale_64_nodes() {
        let config = MultiScaleConfig {
            k: 8,
            macro_nodes: 64,
            meso_nodes: 32,
            micro_nodes: 16,
        };
        let mut msp = MultiScalePhi::new(config);
        let result = msp.compute_all_scales();
        assert!(result.phi_star >= 0.0);
        assert_eq!(result.main_complex.len(), 64);
    }

    #[test]
    fn test_multi_scale_phi_new_valid() {
        let config = MultiScaleConfig {
            macro_nodes: 8,
            meso_nodes: 8,
            micro_nodes: 8,
            k: 4,
        };
        let msp = MultiScalePhi::new(config);
        assert_eq!(msp.hier.tpm.n, 8);
        assert_eq!(msp.config.k, 4);
    }

    #[test]
    fn test_scale_integrated_info_deterministic() {
        let config = MultiScaleConfig {
            macro_nodes: 16,
            meso_nodes: 16,
            micro_nodes: 16,
            ..Default::default()
        };
        let msp = MultiScalePhi::new(config);
        let p1 = msp.scale_integrated_info(0);
        let p2 = msp.scale_integrated_info(0);
        assert_approx_eq(p1, p2, 1e-10);
    }
}
