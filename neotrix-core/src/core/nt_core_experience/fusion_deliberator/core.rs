use super::types::{
    ChainConfig, CoreFringeAttention, CritiqueRound, DeliberationDepth, FusionDeliberator,
    FusionGraphNode, FusionStats, GateContext, HubNode, MessageMode, ParallelReasoningPanel,
    PoolMode, RebuttalRound, SynthesisMode,
};
use crate::core::nt_core_experience::vsa_judge::{JudgeAnalysis, PanelResult};
use crate::core::nt_core_hcube::QuantizedVSA;

// ─── Phase 37 — 4-Stage Deliberation ───

impl FusionDeliberator {
    pub fn deliberate_4stage(
        &mut self,
        query: &[u8],
        context: Option<&GateContext>,
        depth: DeliberationDepth,
    ) -> (Vec<u8>, JudgeAnalysis) {
        if let Some(ref gate) = self.gate {
            if let Some(ctx) = context {
                let benefit = gate.estimate_benefit(query, ctx);
                if benefit < self.min_benefit_threshold {
                    self.gate_skipped_benefit += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
                if !gate.should_deliberate(query, ctx) {
                    self.gate_blocks += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
            }
        }

        if let Some(ref mut cache) = self.cache {
            if let Some(entry) = cache.lookup(query) {
                self.cache_hits += 1;
                return (entry.synthesis.clone(), entry.analysis.clone());
            }
            self.cache_misses += 1;
        }

        // Stage 1: Generate
        let mut results = self.panel.run_panel(query);

        if depth == DeliberationDepth::Standard {
            let analysis = self.judge.analyze(query, &results);
            let synthesis = self.synthesize(&analysis, &results);
            self.deliberation_count += 1;
            if let Some(ref mut cache) = self.cache {
                cache.insert(query, synthesis.clone(), analysis.clone());
            }
            self.record_stats(&analysis, &results);
            return (synthesis, analysis);
        }

        // Stage 2: Cross-Critique
        let n = results.len();
        let mut critiques: Vec<CritiqueRound> = vec![];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let opponent_seed = (j as u64).wrapping_mul(7).wrapping_add(13);
                let dim = crate::core::nt_core_hcube::VSA_DIM;
                let opponent_perspective = QuantizedVSA::seeded_random(opponent_seed, dim);
                let critique =
                    QuantizedVSA::bind(&results[j].thought_vector, &opponent_perspective);
                let critique_neg = QuantizedVSA::bind(
                    &critique,
                    &QuantizedVSA::negate(&results[i].thought_vector),
                );
                critiques.push(CritiqueRound {
                    critiquing_chain: j,
                    target_chain: i,
                    critique_vector: QuantizedVSA::bundle(&[&critique, &critique_neg]),
                    critique_confidence: results[j].confidence * 0.5,
                });
            }
        }

        if depth == DeliberationDepth::Deep {
            for c in &critiques {
                results.push(PanelResult {
                    chain_id: n + c.critiquing_chain * n + c.target_chain,
                    thought_vector: c.critique_vector.clone(),
                    confidence: c.critique_confidence,
                    reasoning_label: format!(
                        "critique_{}_of_{}",
                        c.critiquing_chain, c.target_chain
                    ),
                    execution_time_ns: 0,
                });
            }
            let analysis = self.judge.analyze(query, &results);
            let synthesis = self.synthesize(&analysis, &results);
            self.deliberation_count += 1;
            if let Some(ref mut cache) = self.cache {
                cache.insert(query, synthesis.clone(), analysis.clone());
            }
            self.record_stats(&analysis, &results);
            return (synthesis, analysis);
        }

        let mut critique_results: Vec<PanelResult> = vec![];
        for c in &critiques {
            critique_results.push(PanelResult {
                chain_id: n + c.critiquing_chain * n + c.target_chain,
                thought_vector: c.critique_vector.clone(),
                confidence: c.critique_confidence,
                reasoning_label: format!("critique_{}_of_{}", c.critiquing_chain, c.target_chain),
                execution_time_ns: 0,
            });
        }

        // Stage 3: Rebuttal
        let mut rebuttals: Vec<RebuttalRound> = vec![];
        for chain_idx in 0..n {
            let chain_critiques: Vec<&CritiqueRound> = critiques
                .iter()
                .filter(|c| c.target_chain == chain_idx)
                .collect();
            if chain_critiques.is_empty() {
                continue;
            }
            let mut defense = results[chain_idx].thought_vector.clone();
            for cc in &chain_critiques {
                let rebuttal =
                    QuantizedVSA::bind(&defense, &QuantizedVSA::negate(&cc.critique_vector));
                let bundled = QuantizedVSA::bundle(&[&defense, &rebuttal]);
                defense = bundled;
            }
            rebuttals.push(RebuttalRound {
                chain_id: results[chain_idx].chain_id,
                rebuttal_vector: defense,
                rebuttal_confidence: results[chain_idx].confidence * 0.9,
            });
        }

        // Stage 4: Assemble all results and Judge
        for r in &rebuttals {
            results.push(PanelResult {
                chain_id: 2 * n + r.chain_id,
                thought_vector: r.rebuttal_vector.clone(),
                confidence: r.rebuttal_confidence,
                reasoning_label: format!("rebuttal_{}", r.chain_id),
                execution_time_ns: 0,
            });
        }

        let analysis = self.judge.analyze(query, &results);
        let synthesis = self.synthesize(&analysis, &results);
        self.deliberation_count += 1;
        if let Some(ref mut cache) = self.cache {
            cache.insert(query, synthesis.clone(), analysis.clone());
        }
        self.record_stats(&analysis, &results);
        (synthesis, analysis)
    }
}

// ─── P0.16 CoreFringeAttention — Graph Attention Pooling ───

pub fn graph_attention_pool(results: &[PanelResult], tau: f64) -> Vec<u8> {
    if results.is_empty() {
        return vec![0u8; crate::core::nt_core_hcube::VSA_DIM];
    }
    let n = results.len();
    let mut nodes: Vec<HubNode> = results.iter().map(HubNode::from_panel_result).collect();

    let mut attn = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        let mut row_sum = 0.0;
        for j in 0..n {
            let sim = QuantizedVSA::cosine(&nodes[i].thought_vector, &nodes[j].thought_vector);
            let raw = (sim / tau.max(0.01)).exp();
            attn[i][j] = raw;
            row_sum += raw;
        }
        if row_sum > 0.0 {
            for j in 0..n {
                attn[i][j] /= row_sum;
            }
        }
    }

    for j in 0..n {
        let inbound: usize = (0..n).filter(|&i| attn[i][j] > 0.2).count();
        let outbound: usize = (0..n).filter(|&j2| attn[j][j2] > 0.2).count();
        nodes[j].inbound_degree = inbound;
        nodes[j].outbound_balance = outbound as isize - inbound as isize;
    }
    let max_inbound = nodes
        .iter()
        .map(|n| n.inbound_degree)
        .max()
        .unwrap_or(1)
        .max(1);
    for node in &mut nodes {
        node.is_hub = node.inbound_degree as f64 / max_inbound as f64 > 0.6;
    }

    let mut pooled = vec![0i64; crate::core::nt_core_hcube::VSA_DIM];
    for i in 0..n {
        let node_weight = if nodes[i].is_hub { 1.5 } else { 1.0 } * nodes[i].confidence;
        for j in 0..n {
            if i == j {
                continue;
            }
            let w = attn[i][j] * node_weight;
            for k in 0..pooled.len().min(nodes[j].thought_vector.len()) {
                if nodes[j].thought_vector[k] > 0 {
                    pooled[k] += (w * 256.0) as i64;
                } else {
                    pooled[k] -= (w * 256.0) as i64;
                }
            }
        }
    }
    let result: Vec<u8> = pooled.iter().map(|&v| if v > 0 { 1 } else { 0 }).collect();
    if result.len() != crate::core::nt_core_hcube::VSA_DIM {
        return results
            .iter()
            .next()
            .map(|r| r.thought_vector.clone())
            .unwrap_or_else(|| vec![0u8; crate::core::nt_core_hcube::VSA_DIM]);
    }
    result
}

pub fn detect_hubs(results: &[PanelResult]) -> (Vec<usize>, Vec<usize>) {
    let n = results.len();
    if n < 3 {
        return (vec![], (0..n).collect());
    }
    let mut hubs = Vec::new();
    let mut spokes = Vec::new();
    for i in 0..n {
        let sims: Vec<f64> = (0..n)
            .filter(|&j| i != j)
            .map(|j| QuantizedVSA::cosine(&results[i].thought_vector, &results[j].thought_vector))
            .collect();
        let mean = sims.iter().sum::<f64>() / sims.len() as f64;
        let variance = sims.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / sims.len() as f64;
        let z = if variance > 0.0 {
            (sims.iter().sum::<f64>() / sims.len() as f64 - mean) / variance.sqrt()
        } else {
            0.0
        };
        if z > 1.5 {
            hubs.push(i);
        } else {
            spokes.push(i);
        }
    }
    (hubs, spokes)
}

// ─── Core Deliberation Methods ───

impl FusionDeliberator {
    pub fn deliberate(
        &mut self,
        query: &[u8],
        context: Option<&GateContext>,
    ) -> (Vec<u8>, JudgeAnalysis) {
        self.deliberate_with_external(query, context, &[])
    }

    pub fn deliberate_with_external(
        &mut self,
        query: &[u8],
        context: Option<&GateContext>,
        external_results: &[PanelResult],
    ) -> (Vec<u8>, JudgeAnalysis) {
        if let Some(ref gate) = self.gate {
            if let Some(ctx) = context {
                let benefit = gate.estimate_benefit(query, ctx);
                if benefit < self.min_benefit_threshold {
                    self.gate_skipped_benefit += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
                if !gate.should_deliberate(query, ctx) {
                    self.gate_blocks += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
            }
        }

        if let Some(ref mut cache) = self.cache {
            if let Some(entry) = cache.lookup(query) {
                self.cache_hits += 1;
                return (entry.synthesis.clone(), entry.analysis.clone());
            }
            self.cache_misses += 1;
        }

        let mut results = self.panel.run_panel(query);
        if !external_results.is_empty() {
            results.extend_from_slice(external_results);
        }
        let analysis = self.judge.analyze(query, &results);
        let synthesis = self.synthesize(&analysis, &results);

        self.update_chain_histories(&synthesis, &results);
        self.deliberation_count += 1;

        if let Some(ref mut cache) = self.cache {
            cache.insert(query, synthesis.clone(), analysis.clone());
        }

        self.record_stats(&analysis, &results);
        (synthesis, analysis)
    }

    fn update_chain_histories(&mut self, synthesis: &[u8], results: &[PanelResult]) {
        for r in results {
            let sim = QuantizedVSA::cosine(synthesis, &r.thought_vector);
            if let Some(h) = self
                .chain_histories
                .iter_mut()
                .find(|h| h.chain_id == r.chain_id)
            {
                h.record_accuracy(sim);
            }
        }
    }

    pub fn chain_weights(&self) -> Vec<(usize, f64)> {
        self.chain_histories
            .iter()
            .map(|h| (h.chain_id, h.total_weight))
            .collect()
    }

    pub fn deliberate_hierarchical(&mut self, query: &[u8], depth: u8) -> (Vec<u8>, JudgeAnalysis) {
        let depth = depth.min(self.max_depth).max(1);
        let mut current_query = query.to_vec();

        for _level in 0..depth {
            let results = self.panel.run_panel(&current_query);
            let analysis = self.judge.analyze(&current_query, &results);
            let synthesis = self.synthesize(&analysis, &results);

            let sim_to_query = QuantizedVSA::cosine(&synthesis, query);
            if sim_to_query > 0.85 || analysis.has_strong_consensus() {
                self.hierarchical_count += 1;
                return (synthesis, analysis);
            }
            current_query = synthesis;
        }

        self.hierarchical_count += 1;
        let results = self.panel.run_panel(&current_query);
        let analysis = self.judge.analyze(&current_query, &results);
        let synthesis = self.synthesize(&analysis, &results);
        (synthesis, analysis)
    }

    pub fn deliberate_with_disagreement(
        &mut self,
        query: &[u8],
        _context: Option<&GateContext>,
    ) -> (Vec<u8>, JudgeAnalysis) {
        let results = self.panel.run_panel(query);
        let mut counter_thoughts = Vec::with_capacity(results.len());

        for i in 0..results.len() {
            for j in 0..results.len() {
                if i == j {
                    continue;
                }
                let opponent_perspective = QuantizedVSA::bind(
                    &results[j].thought_vector,
                    &QuantizedVSA::seeded_random(j as u64 * 3 + 7, query.len().min(4096)),
                );
                let critique =
                    QuantizedVSA::bind(&results[i].thought_vector, &opponent_perspective);
                counter_thoughts.push(PanelResult {
                    chain_id: results.len() + i * results.len() + j,
                    thought_vector: critique,
                    confidence: results[i].confidence * 0.6,
                    reasoning_label: format!("critique_{}_of_{}", i, j),
                    execution_time_ns: 0,
                });
            }
        }

        self.deliberation_count += 1;
        let all_results: Vec<PanelResult> = results.into_iter().chain(counter_thoughts).collect();
        let analysis = self.judge.analyze(query, &all_results);
        let synthesis = self.synthesize(&analysis, &all_results);
        self.record_stats(&analysis, &all_results);
        (synthesis, analysis)
    }

    pub fn self_fusion(&mut self, query: &[u8], n: usize) -> (Vec<u8>, JudgeAnalysis) {
        let n = n.max(2).min(16);
        let mut panel = ParallelReasoningPanel::with_chains(self.panel.chains.clone());
        let mut all_results = vec![];
        for i in 0..n {
            let single_chain = vec![ChainConfig::new(i, &format!("self_{}", i))];
            panel.chains = single_chain;
            let results = panel.run_panel(query);
            all_results.extend(results);
        }
        panel.chains = self.panel.chains.clone();
        let analysis = self.judge.analyze(query, &all_results);
        let synthesis = self.synthesize(&analysis, &all_results);
        self.deliberation_count += 1;
        (synthesis, analysis)
    }

    fn synthesize(&self, analysis: &JudgeAnalysis, results: &[PanelResult]) -> Vec<u8> {
        if self.graph_mode && results.len() >= 3 {
            return graph_attention_pool(results, 0.5);
        }
        match self.synthesis_mode {
            SynthesisMode::ConsensusWeighted => {
                self.judge.synthesis_by_consensus(analysis, results)
            }
            SynthesisMode::ConsensusPlusNovelty => {
                self.judge.synthesis_by_consensus(analysis, results)
            }
            SynthesisMode::StructuredEnsemble => {
                let mut weighted: Vec<&[u8]> = vec![];
                if let Some(ref consensus) = analysis.consensus {
                    for &id in &consensus.member_ids {
                        if let Some(r) = results.iter().find(|r| r.chain_id == id) {
                            weighted.push(r.thought_vector.as_slice());
                        }
                    }
                }
                let remaining: Vec<&[u8]> = results
                    .iter()
                    .filter(|r| {
                        if let Some(ref consensus) = analysis.consensus {
                            !consensus.member_ids.contains(&r.chain_id)
                        } else {
                            true
                        }
                    })
                    .map(|r| r.thought_vector.as_slice())
                    .collect();
                weighted.extend(remaining);
                if weighted.is_empty() {
                    let all: Vec<&[u8]> = results
                        .iter()
                        .map(|r| r.thought_vector.as_slice())
                        .collect();
                    QuantizedVSA::bundle(&all)
                } else {
                    QuantizedVSA::bundle(&weighted)
                }
            }
        }
    }

    fn record_stats(&mut self, analysis: &JudgeAnalysis, results: &[PanelResult]) {
        self.running_confidence_sum += analysis.overall_confidence;
        self.running_panel_sum += results.len();
        self.running_samples += 1;
    }

    pub fn stats(&self) -> FusionStats {
        FusionStats {
            total_deliberations: self.deliberation_count,
            total_hierarchical: self.hierarchical_count,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            gate_blocks: self.gate_blocks,
            gate_skipped_benefit: self.gate_skipped_benefit,
            avg_panel_size: if self.running_samples > 0 {
                self.running_panel_sum as f64 / self.running_samples as f64
            } else {
                0.0
            },
            avg_confidence: if self.running_samples > 0 {
                self.running_confidence_sum / self.running_samples as f64
            } else {
                0.0
            },
            deep_deliberations: self.deep_deliberations,
            full_deliberations: self.full_deliberations,
        }
    }

    pub fn deliberate_graph(
        &mut self,
        query: &[u8],
        context: Option<&GateContext>,
    ) -> (Vec<u8>, JudgeAnalysis) {
        if let Some(ref gate) = self.gate {
            if let Some(ctx) = context {
                let benefit = gate.estimate_benefit(query, ctx);
                if benefit < self.min_benefit_threshold {
                    self.gate_skipped_benefit += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
                if !gate.should_deliberate(query, ctx) {
                    self.gate_blocks += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
            }
        }

        if let Some(ref mut cache) = self.cache {
            if let Some(entry) = cache.lookup(query) {
                self.cache_hits += 1;
                return (entry.synthesis.clone(), entry.analysis.clone());
            }
            self.cache_misses += 1;
        }

        let results = self.panel.run_panel(query);
        let (_hubs, _spokes) = detect_hubs(&results);
        let synthesis = graph_attention_pool(&results, 0.5);
        let analysis = self.judge.analyze(query, &results);

        self.deliberation_count += 1;
        if let Some(ref mut cache) = self.cache {
            cache.insert(query, synthesis.clone(), analysis.clone());
        }
        self.record_stats(&analysis, &results);
        (synthesis, analysis)
    }

    pub fn set_graph_mode(&mut self, enabled: bool) {
        self.graph_mode = enabled;
    }
    pub fn set_pool_mode(&mut self, mode: PoolMode) {
        self.pool_mode = mode;
    }

    pub fn deliberate_with_graph(
        &mut self,
        query: &[u8],
        ctx: Option<&GateContext>,
        _depth: DeliberationDepth,
    ) -> (Vec<u8>, JudgeAnalysis) {
        if let Some(ref gate) = self.gate {
            if let Some(context) = ctx {
                let benefit = gate.estimate_benefit(query, context);
                if benefit < self.min_benefit_threshold {
                    self.gate_skipped_benefit += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
                if !gate.should_deliberate(query, context) {
                    self.gate_blocks += 1;
                    let empty = vec![];
                    let analysis = self.judge.analyze(query, &empty);
                    return (query.to_vec(), analysis);
                }
            }
        }

        if let Some(ref mut cache) = self.cache {
            if let Some(entry) = cache.lookup(query) {
                self.cache_hits += 1;
                return (entry.synthesis.clone(), entry.analysis.clone());
            }
            self.cache_misses += 1;
        }

        let results = self.panel.run_panel(query);
        let attractor = results
            .first()
            .map(|r| r.thought_vector.clone())
            .unwrap_or_default();
        let mut nodes: Vec<FusionGraphNode> = results
            .iter()
            .map(|r| FusionGraphNode::from_panel_result(r, &attractor))
            .collect();

        CoreFringeAttention::detect_hubs(&mut nodes, 1.5);
        let attn = CoreFringeAttention::compute_attention(&nodes, 0.5);
        let updated = CoreFringeAttention::message_passing(&nodes, &attn, MessageMode::HubToSpoke);
        let synthesis = CoreFringeAttention::graph_pool(&nodes, &updated, self.pool_mode);

        let analysis = self.judge.analyze(query, &results);
        self.deliberation_count += 1;
        if let Some(ref mut cache) = self.cache {
            cache.insert(query, synthesis.clone(), analysis.clone());
        }
        self.record_stats(&analysis, &results);
        (synthesis, analysis)
    }

    pub fn reset_stats(&mut self) {
        self.deliberation_count = 0;
        self.hierarchical_count = 0;
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.gate_blocks = 0;
        self.gate_skipped_benefit = 0;
        self.running_confidence_sum = 0.0;
        self.running_panel_sum = 0;
        self.running_samples = 0;
        self.deep_deliberations = 0;
        self.full_deliberations = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;
    use crate::core::nt_core_experience::vsa_judge::{DeliberationOutcome, VSAJudge};
    use crate::core::nt_core_hcube::QuantizedVSA;

    #[test]
    fn test_panel_default_creation() {
        let panel = ParallelReasoningPanel::default();
        assert_eq!(panel.chain_count(), 4);
    }

    #[test]
    fn test_panel_n_chains() {
        let panel = ParallelReasoningPanel::with_n_chains(6);
        assert_eq!(panel.chain_count(), 6);
    }

    #[test]
    fn test_panel_clamped_chains() {
        let panel = ParallelReasoningPanel::with_n_chains(1);
        assert_eq!(panel.chain_count(), 2);
        let panel = ParallelReasoningPanel::with_n_chains(10);
        assert_eq!(panel.chain_count(), 8);
    }

    #[test]
    fn test_panel_run_produces_results() {
        let panel = ParallelReasoningPanel::default();
        let query = QuantizedVSA::random_binary();
        let results = panel.run_panel(&query);
        assert_eq!(results.len(), 4);
        for r in &results {
            assert_eq!(r.thought_vector.len(), 4096);
            assert!(r.confidence >= 0.0 && r.confidence <= 1.0);
        }
    }

    #[test]
    fn test_deliberator_basic_flow() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate(&query, None);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors > 0);
        assert!(analysis.overall_confidence >= 0.0);
    }

    #[test]
    fn test_deliberator_hierarchical() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate_hierarchical(&query, 2);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors > 0);
        assert!(dl.stats().total_hierarchical >= 1);
    }

    #[test]
    fn test_deliberator_self_fusion() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.self_fusion(&query, 4);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors >= 4);
        assert!(dl.stats().total_deliberations >= 1);
    }

    #[test]
    fn test_deliberator_cache() {
        let mut dl = FusionDeliberator::default();
        dl.cache = Some(DeliberationCache::new(10));
        let query = QuantizedVSA::random_binary();
        let (_s1, _a1) = dl.deliberate(&query, None);
        let (_s2, _a2) = dl.deliberate(&query, None);
        assert_eq!(dl.stats().cache_hits, 1);
        assert_eq!(dl.stats().cache_misses, 1);
    }

    #[test]
    fn test_deliberator_gate_blocks_simple() {
        let mut dl = FusionDeliberator::default();
        dl.gate = Some(DeliberationGate::new(0.8, 0.8));
        let query = QuantizedVSA::random_binary();
        let context = GateContext {
            cognitive_load: 0.9,
            cycle: 0,
            recent_deliberation_count: 0,
            task_type: DeliberatorTaskType::Simple,
            query_entropy: 0.1,
        };
        let should = dl
            .gate
            .as_ref()
            .unwrap()
            .should_deliberate(&query, &context);
        assert!(!should);
    }

    #[test]
    fn test_deliberator_all_synthesis_modes() {
        let query = QuantizedVSA::random_binary();
        for mode in &[
            SynthesisMode::ConsensusWeighted,
            SynthesisMode::ConsensusPlusNovelty,
            SynthesisMode::StructuredEnsemble,
        ] {
            let mut dl = FusionDeliberator {
                synthesis_mode: *mode,
                ..Default::default()
            };
            let (synthesis, _) = dl.deliberate(&query, None);
            assert_eq!(synthesis.len(), 4096, "mode {:?} failed", mode);
        }
    }

    #[test]
    fn test_deliberator_stats() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        dl.deliberate(&query, None);
        dl.deliberate(&query, None);
        let stats = dl.stats();
        assert!(stats.total_deliberations >= 2);
        assert!(stats.avg_panel_size > 0.0);
        assert!(stats.avg_confidence >= 0.0);
    }

    #[test]
    fn test_deliberator_reset_stats() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        dl.deliberate(&query, None);
        dl.reset_stats();
        let stats = dl.stats();
        assert_eq!(stats.total_deliberations, 0);
        assert_eq!(stats.avg_panel_size, 0.0);
    }

    #[test]
    fn test_gate_estimate_benefit() {
        let gate = DeliberationGate::default();
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.3,
            cycle: 0,
            recent_deliberation_count: 0,
            task_type: DeliberatorTaskType::Research,
            query_entropy: 0.7,
        };
        let benefit = gate.estimate_benefit(&query, &ctx);
        assert!(benefit > 0.5);
    }

    #[test]
    fn test_gate_blocks_when_overloaded() {
        let gate = DeliberationGate::default();
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.9,
            cycle: 0,
            recent_deliberation_count: 10,
            task_type: DeliberatorTaskType::Research,
            query_entropy: 0.8,
        };
        assert!(!gate.should_deliberate(&query, &ctx));
    }

    #[test]
    fn test_cache_insert_and_lookup() {
        let mut cache = DeliberationCache::new(10);
        let query = QuantizedVSA::random_binary();
        let analysis = VSAJudge::default().analyze(&query, &[]);
        assert!(cache.lookup(&query).is_none());
        cache.insert(&query, query.clone(), analysis);
        assert!(cache.lookup(&query).is_some());
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = DeliberationCache::new(2);
        let empty_analysis = VSAJudge::default().analyze(&[0u8; 4096], &[]);
        for i in 0..5 {
            let v = QuantizedVSA::seeded_random(i, 4096);
            cache.insert(&v, v.clone(), empty_analysis.clone());
        }
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = DeliberationCache::new(10);
        let empty_analysis = VSAJudge::default().analyze(&[0u8; 4096], &[]);
        for i in 0..3 {
            let v = QuantizedVSA::seeded_random(i, 4096);
            cache.insert(&v, v.clone(), empty_analysis.clone());
        }
        assert_eq!(cache.len(), 3);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_chain_config_diverse_pool() {
        let chains = ChainConfig::diverse_pool(5);
        assert_eq!(chains.len(), 5);
        let seeds: std::collections::HashSet<u64> = chains.iter().map(|c| c.vsa_seed).collect();
        assert_eq!(seeds.len(), 5);
    }

    #[test]
    fn test_diverse_chain_strategies_produce_different_outputs() {
        let panel = ParallelReasoningPanel::with_chains(ChainConfig::diverse_pool(8));
        let query = QuantizedVSA::random_binary();
        let results = panel.run_panel(&query);
        let mut unique = std::collections::HashSet::new();
        for r in &results {
            unique.insert(r.thought_vector.clone());
        }
        assert!(
            unique.len() >= 6,
            "expected >=6 unique outputs from 8 diverse chains, got {}",
            unique.len()
        );
    }

    #[test]
    fn test_analytical_chain_stays_close_to_query() {
        let analytical = ChainConfig::new(0, "analytical");
        let panel = ParallelReasoningPanel::with_chains(vec![analytical]);
        let query = QuantizedVSA::random_binary();
        let results = panel.run_panel(&query);
        assert_eq!(results.len(), 1);
        let sim = QuantizedVSA::cosine(&results[0].thought_vector, &query);
        assert!(
            sim > 0.0,
            "analytical chain should have non-zero similarity to query"
        );
    }

    #[test]
    fn test_critical_chain_produces_divergent_output() {
        let critical = ChainConfig::new(0, "critical");
        let panel = ParallelReasoningPanel::with_chains(vec![critical]);
        let query = QuantizedVSA::random_binary();
        let results = panel.run_panel(&query);
        assert_eq!(results.len(), 1);
        let sim = QuantizedVSA::cosine(&results[0].thought_vector, &query);
        assert!(sim > 0.0, "critical chain should still have some structure");
    }

    #[test]
    fn test_chain_confidence_bounded() {
        let panel = ParallelReasoningPanel::default();
        let query = QuantizedVSA::random_binary();
        let results = panel.run_panel(&query);
        for r in &results {
            assert!(
                r.confidence >= 0.0 && r.confidence <= 1.0,
                "chain {} confidence {:.3} out of bounds",
                r.chain_id,
                r.confidence
            );
        }
    }

    #[test]
    fn test_cache_uses_query_hash_for_lookup() {
        let mut cache = DeliberationCache::new(10);
        let q1 = QuantizedVSA::seeded_random(100, 4096);
        let q2 = QuantizedVSA::seeded_random(200, 4096);
        let analysis = VSAJudge::default().analyze(&q1, &[]);
        cache.insert(&q1, q1.clone(), analysis);
        assert!(cache.lookup(&q2).is_none());
    }

    #[test]
    fn test_full_fusion_pipeline_end_to_end() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate(&query, None);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors >= 4);
        let outcome = analysis.recommended_outcome();
        match outcome {
            DeliberationOutcome::Recommendation => {
                assert!(analysis.has_strong_consensus());
            }
            DeliberationOutcome::Alternatives => {
                assert!(analysis.has_critical_contradictions());
            }
            DeliberationOutcome::Investigate => {
                assert!(analysis.overall_confidence < 0.5);
            }
            _ => {}
        }
        assert!(QuantizedVSA::hamming_distance(&synthesis, &query) as f64 / 4096.0 > 0.0);
    }

    #[test]
    fn test_gate_blocks_full_deliberation_under_load() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.9,
            cycle: 0,
            recent_deliberation_count: 10,
            task_type: DeliberatorTaskType::Analytical,
            query_entropy: 0.5,
        };
        let (synthesis, analysis) = dl.deliberate(&query, Some(&ctx));
        assert_eq!(
            analysis.n_contributors, 0,
            "gate should skip panel, got {} contributors",
            analysis.n_contributors
        );
        assert!(!synthesis.is_empty());
        assert!(dl.stats().gate_blocks >= 1);
    }

    #[test]
    fn test_deliberate_with_disagreement() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate_with_disagreement(&query, None);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors > 0);
        assert!(dl.stats().total_deliberations >= 1);
    }

    #[test]
    fn test_disagreement_produces_more_contradictions_than_plain() {
        let query = QuantizedVSA::random_binary();
        let mut plain = FusionDeliberator::default();
        let (_, plain_analysis) = plain.deliberate(&query, None);
        let mut forced = FusionDeliberator::default();
        let (_, forced_analysis) = forced.deliberate_with_disagreement(&query, None);
        assert!(
            forced_analysis.contradictions.len() >= plain_analysis.contradictions.len(),
            "forced disagreement should surface at least as many contradictions (plain: {}, forced: {})",
            plain_analysis.contradictions.len(),
            forced_analysis.contradictions.len()
        );
    }

    // ─── Phase 37 — 4-Stage Pipeline Tests ───

    #[test]
    fn test_deliberate_4stage_standard_depth() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate_4stage(&query, None, DeliberationDepth::Standard);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors >= 4);
        assert!(analysis.overall_confidence >= 0.0);
    }

    #[test]
    fn test_deliberate_4stage_deep_depth() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate_4stage(&query, None, DeliberationDepth::Deep);
        assert_eq!(synthesis.len(), 4096);
        let expected_min_contributors = 4 + (4 * 4 - 4);
        assert!(
            analysis.n_contributors >= expected_min_contributors,
            "Deep 4-stage should have >= {} contributors, got {}",
            expected_min_contributors,
            analysis.n_contributors
        );
    }

    #[test]
    fn test_deliberate_4stage_full_depth() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate_4stage(&query, None, DeliberationDepth::Full);
        assert_eq!(synthesis.len(), 4096);
        let n = 4;
        let critiques = n * n - n;
        let rebuttals = n;
        let expected_min = n + critiques + rebuttals;
        assert!(
            analysis.n_contributors >= expected_min,
            "Full 4-stage should have >= {} contributors, got {}",
            expected_min,
            analysis.n_contributors
        );
    }

    #[test]
    fn test_4stage_deep_produces_more_contradictions_than_standard() {
        let query = QuantizedVSA::random_binary();
        let mut std = FusionDeliberator::default();
        let (_, std_analysis) = std.deliberate_4stage(&query, None, DeliberationDepth::Standard);
        let mut deep = FusionDeliberator::default();
        let (_, deep_analysis) = deep.deliberate_4stage(&query, None, DeliberationDepth::Deep);
        assert!(deep_analysis.contradictions.len() >= std_analysis.contradictions.len(), "Deep 4-stage should surface at least as many contradictions as standard (std: {}, deep: {})", std_analysis.contradictions.len(), deep_analysis.contradictions.len());
    }

    #[test]
    fn test_4stage_gate_blocks_under_load() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.9,
            cycle: 0,
            recent_deliberation_count: 10,
            task_type: DeliberatorTaskType::Simple,
            query_entropy: 0.1,
        };
        let (synthesis, analysis) =
            dl.deliberate_4stage(&query, Some(&ctx), DeliberationDepth::Full);
        assert_eq!(
            analysis.n_contributors, 0,
            "gate should block under load, got {} contributors",
            analysis.n_contributors
        );
        assert!(!synthesis.is_empty());
    }

    // ─── Phase 38 — Historical Chain Weighting Tests ───

    #[test]
    fn test_chain_history_initial_weight() {
        let h = ChainHistory::new(0, "test");
        assert!((h.total_weight - 1.0).abs() < 1e-6);
        assert!((h.historical_accuracy() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_chain_history_records_accuracy() {
        let mut h = ChainHistory::new(0, "test");
        h.record_accuracy(0.8);
        h.record_accuracy(0.6);
        let avg = h.historical_accuracy();
        assert!((avg - 0.7).abs() < 1e-6);
        assert!(h.total_weight > 0.8);
        assert!(h.total_weight < 1.0);
    }

    #[test]
    fn test_chain_weights_available_after_deliberation() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        dl.deliberate(&query, None);
        let weights = dl.chain_weights();
        assert!(!weights.is_empty());
        for (id, w) in &weights {
            assert!(*w > 0.0, "chain {} weight should be positive", id);
        }
    }

    // ─── Phase 39 — Selective Deliberation Tests ───

    #[test]
    fn test_gate_skipped_when_benefit_too_low() {
        let mut dl = FusionDeliberator::default();
        dl.min_benefit_threshold = 0.9;
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.3,
            cycle: 0,
            recent_deliberation_count: 0,
            task_type: DeliberatorTaskType::Simple,
            query_entropy: 0.2,
        };
        let (_, analysis) = dl.deliberate(&query, Some(&ctx));
        assert_eq!(
            analysis.n_contributors, 0,
            "gate should skip low-benefit query, got {} contributors",
            analysis.n_contributors
        );
        assert!(
            dl.stats().gate_skipped_benefit >= 1,
            "should record benefit skip"
        );
    }

    #[test]
    fn test_normal_benefit_allows_deliberation() {
        let mut dl = FusionDeliberator::default();
        dl.min_benefit_threshold = 0.1;
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.3,
            cycle: 0,
            recent_deliberation_count: 0,
            task_type: DeliberatorTaskType::Analytical,
            query_entropy: 0.7,
        };
        let (_, analysis) = dl.deliberate(&query, Some(&ctx));
        assert!(
            analysis.n_contributors > 0,
            "normal benefit should allow deliberation"
        );
    }

    // ─── P0.16 — CoreFringeAttention Tests ───

    #[test]
    fn test_hub_node_from_panel_result() {
        let pr = PanelResult {
            chain_id: 0,
            thought_vector: vec![1u8; 4096],
            confidence: 0.8,
            reasoning_label: "analytical".into(),
            execution_time_ns: 100,
        };
        let hn = HubNode::from_panel_result(&pr);
        assert_eq!(hn.chain_id, 0);
        assert_eq!(hn.thought_vector.len(), 4096);
        assert_eq!(hn.label, "analytical");
    }

    #[test]
    fn test_graph_attention_pool_produces_vsa() {
        let query = vec![1u8; 4096];
        let panel = ParallelReasoningPanel::default();
        let results = panel.run_panel(&query);
        let pooled = graph_attention_pool(&results, 0.5);
        assert_eq!(pooled.len(), 4096);
    }

    #[test]
    fn test_graph_attention_pool_empty_results() {
        let pooled = graph_attention_pool(&[], 0.5);
        assert_eq!(pooled.len(), 4096);
        assert!(pooled.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_detect_hubs_identifies_structure() {
        let mut results = Vec::new();
        for i in 0..5 {
            let v = QuantizedVSA::seeded_random(i, 4096);
            results.push(PanelResult {
                chain_id: i as usize,
                thought_vector: v,
                confidence: 0.8,
                reasoning_label: format!("chain_{}", i),
                execution_time_ns: 0,
            });
        }
        let (hubs, spokes) = detect_hubs(&results);
        assert!(hubs.is_empty() || !spokes.is_empty());
    }

    #[test]
    fn test_deliberate_graph_mode_produces_vsa_output() {
        let mut dl = FusionDeliberator::default();
        dl.graph_mode = true;
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) = dl.deliberate_graph(&query, None);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors >= 4);
    }

    #[test]
    fn test_deliberate_graph_gate_blocks_under_load() {
        let mut dl = FusionDeliberator::default();
        dl.graph_mode = true;
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.9,
            cycle: 0,
            recent_deliberation_count: 10,
            task_type: DeliberatorTaskType::Simple,
            query_entropy: 0.1,
        };
        let (_, analysis) = dl.deliberate_graph(&query, Some(&ctx));
        assert_eq!(analysis.n_contributors, 0, "gate should block under load");
    }

    #[test]
    fn test_synthesize_in_graph_mode() {
        let dl = FusionDeliberator {
            graph_mode: true,
            ..Default::default()
        };
        let query = QuantizedVSA::random_binary();
        let results = dl.panel.run_panel(&query);
        let analysis = dl.judge.analyze(&query, &results);
        let synthesis = dl.synthesize(&analysis, &results);
        assert_eq!(synthesis.len(), 4096);
    }

    #[test]
    fn test_synthesize_graph_mode_skipped_for_few_results() {
        let dl = FusionDeliberator {
            graph_mode: true,
            ..Default::default()
        };
        let analysis = dl.judge.analyze(&[0u8; 4096], &[]);
        let synthesis = dl.synthesize(&analysis, &[]);
        assert_eq!(synthesis.len(), 4096);
    }

    #[test]
    fn test_chain_history_constructors_consistent() {
        let h1 = ChainHistory::new(1, "alpha");
        let h2 = ChainHistory::new(1, "alpha");
        assert_eq!(h1.chain_id, h2.chain_id);
        assert_eq!(h1.label, h2.label);
        assert!((h1.total_weight - h2.total_weight).abs() < 1e-10);
    }

    #[test]
    fn test_fusion_graph_node_creation() {
        let pr = PanelResult {
            chain_id: 0,
            thought_vector: vec![1u8; 4096],
            confidence: 0.8,
            reasoning_label: "analytical".into(),
            execution_time_ns: 100,
        };
        let attractor = vec![0u8; 4096];
        let node = FusionGraphNode::from_panel_result(&pr, &attractor);
        assert_eq!(node.thought_vector.len(), 4096);
        assert_eq!(node.attractor_state.len(), 4096);
        assert_eq!(node.chain_label, "analytical");
        assert!((node.confidence - 0.8).abs() < 1e-6);
        assert!((node.hubness_score - 0.0).abs() < 1e-6);
        assert!(!node.is_hub);
    }

    #[test]
    fn test_compute_attention_matrix_sums_to_one() {
        let mut nodes = vec![];
        for i in 0..4 {
            let v = QuantizedVSA::seeded_random(i, 4096);
            nodes.push(FusionGraphNode {
                thought_vector: v,
                confidence: 0.8,
                chain_label: format!("c{}", i),
                attractor_state: vec![0u8; 4096],
                hubness_score: 0.0,
                is_hub: false,
            });
        }
        let attn = CoreFringeAttention::compute_attention(&nodes, 0.5);
        assert_eq!(attn.len(), 4);
        for row in &attn {
            assert_eq!(row.len(), 4);
            let sum: f64 = row.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-6,
                "row sum should be 1.0, got {}",
                sum
            );
        }
    }

    #[test]
    fn test_compute_attention_empty_nodes() {
        let attn = CoreFringeAttention::compute_attention(&[], 0.5);
        assert!(attn.is_empty());
    }

    #[test]
    fn test_message_passing_hub_to_spoke_produces_vectors() {
        let nodes: Vec<FusionGraphNode> = (0..4)
            .map(|i| FusionGraphNode {
                thought_vector: QuantizedVSA::seeded_random(i, 4096),
                confidence: 0.5 + i as f64 * 0.1,
                chain_label: format!("c{}", i),
                attractor_state: vec![0u8; 4096],
                hubness_score: 0.0,
                is_hub: i == 0 || i == 1,
            })
            .collect();
        let attn = CoreFringeAttention::compute_attention(&nodes, 0.5);
        let updated = CoreFringeAttention::message_passing(&nodes, &attn, MessageMode::HubToSpoke);
        assert_eq!(updated.len(), 4);
        for v in &updated {
            assert_eq!(v.len(), 4096);
        }
    }

    #[test]
    fn test_message_passing_full_graph_all_pairs() {
        let nodes: Vec<FusionGraphNode> = (0..3)
            .map(|i| FusionGraphNode {
                thought_vector: QuantizedVSA::seeded_random(i, 4096),
                confidence: 0.7,
                chain_label: format!("c{}", i),
                attractor_state: vec![0u8; 4096],
                hubness_score: 0.0,
                is_hub: false,
            })
            .collect();
        let attn = CoreFringeAttention::compute_attention(&nodes, 0.5);
        let updated = CoreFringeAttention::message_passing(&nodes, &attn, MessageMode::FullGraph);
        assert_eq!(updated.len(), 3);
        for v in &updated {
            assert_eq!(v.len(), 4096);
        }
    }

    #[test]
    fn test_detect_hubs_marks_nodes_with_finite_scores() {
        let mut nodes: Vec<FusionGraphNode> = (0..5)
            .map(|i| FusionGraphNode {
                thought_vector: QuantizedVSA::seeded_random(i, 4096),
                confidence: 0.7,
                chain_label: format!("c{}", i),
                attractor_state: vec![0u8; 4096],
                hubness_score: 0.0,
                is_hub: false,
            })
            .collect();
        CoreFringeAttention::detect_hubs(&mut nodes, 1.5);
        for node in &nodes {
            assert!(node.hubness_score.is_finite());
        }
    }

    #[test]
    fn test_detect_hubs_small_set_never_hub() {
        let mut nodes: Vec<FusionGraphNode> = (0..2)
            .map(|i| FusionGraphNode {
                thought_vector: QuantizedVSA::seeded_random(i, 4096),
                confidence: 0.7,
                chain_label: format!("c{}", i),
                attractor_state: vec![0u8; 4096],
                hubness_score: 0.0,
                is_hub: false,
            })
            .collect();
        CoreFringeAttention::detect_hubs(&mut nodes, 1.5);
        for node in &nodes {
            assert!(!node.is_hub, "n<3 should never mark as hub");
        }
    }

    #[test]
    fn test_graph_pool_max_mode() {
        let v1 = vec![1u8, 0, 1, 0];
        let v2 = vec![0u8, 1, 0, 1];
        let pooled = CoreFringeAttention::graph_pool(&[], &[v1, v2], PoolMode::Max);
        assert_eq!(pooled, vec![1, 1, 1, 1]);
    }

    #[test]
    fn test_graph_pool_mean_mode() {
        let v1 = vec![1u8, 1, 0, 0];
        let v2 = vec![1u8, 0, 1, 0];
        let pooled = CoreFringeAttention::graph_pool(&[], &[v1, v2], PoolMode::Mean);
        assert_eq!(pooled, vec![1, 0, 0, 0]);
    }

    #[test]
    fn test_graph_pool_empty() {
        let pooled = CoreFringeAttention::graph_pool(&[], &[], PoolMode::Mean);
        assert_eq!(pooled.len(), 4096);
        assert!(pooled.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_deliberate_with_graph_produces_vsa() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let (synthesis, analysis) =
            dl.deliberate_with_graph(&query, None, DeliberationDepth::Standard);
        assert_eq!(synthesis.len(), 4096);
        assert!(analysis.n_contributors >= 4);
        assert!(dl.stats().total_deliberations >= 1);
    }

    #[test]
    fn test_set_graph_mode_and_pool_mode() {
        let mut dl = FusionDeliberator::default();
        dl.set_graph_mode(true);
        assert!(dl.graph_mode);
        dl.set_pool_mode(PoolMode::Max);
        assert_eq!(dl.pool_mode, PoolMode::Max);
        dl.set_pool_mode(PoolMode::Mean);
        assert_eq!(dl.pool_mode, PoolMode::Mean);
        dl.set_graph_mode(false);
        assert!(!dl.graph_mode);
    }

    #[test]
    fn test_deliberate_with_graph_gate_blocks_under_load() {
        let mut dl = FusionDeliberator::default();
        let query = QuantizedVSA::random_binary();
        let ctx = GateContext {
            cognitive_load: 0.9,
            cycle: 0,
            recent_deliberation_count: 10,
            task_type: DeliberatorTaskType::Simple,
            query_entropy: 0.1,
        };
        let (_, analysis) =
            dl.deliberate_with_graph(&query, Some(&ctx), DeliberationDepth::Standard);
        assert_eq!(analysis.n_contributors, 0, "gate should block under load");
    }
}
