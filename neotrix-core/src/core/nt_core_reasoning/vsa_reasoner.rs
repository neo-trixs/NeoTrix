use crate::core::nt_core_hcube::multi_head_resonator::{AggregationMode, MultiHeadResonator};
use crate::core::nt_core_hcube::vsa_multi_model::{ModelFactory, SupportedModel, VsaModel};
use crate::core::nt_core_hcube::vsa_vector::VsaVector;
use crate::core::nt_core_hcube::QuantizedVSA;

use super::vsa_blackboard::{ExpertType, Hypothesis, VsaBlackboard};

use nt_lang::tensor_graph::{self, compute_forward, optimize, GraphNode, TensorGraph};

#[derive(Debug, Clone)]
pub struct ReasonerConfig {
    pub dimension: usize,
    pub analogical_threshold: f64,
    pub causal_entropy_threshold: f64,
    pub multi_hop_max_depth: usize,
    pub contradiction_threshold: f64,
    pub max_blackboard_hypotheses: usize,
    pub vsa_model_type: SupportedModel,
}

impl Default for ReasonerConfig {
    fn default() -> Self {
        Self {
            dimension: 4096,
            analogical_threshold: 0.6,
            causal_entropy_threshold: 0.4,
            multi_hop_max_depth: 5,
            contradiction_threshold: 0.85,
            max_blackboard_hypotheses: 256,
            vsa_model_type: SupportedModel::Map,
        }
    }
}

#[derive(Debug)]
pub struct VsaReasoner {
    pub blackboard: VsaBlackboard,
    pub pattern_matcher: PatternMatcher,
    pub last_analogical_matches: Vec<(String, f64)>,
    pub vsa_model: Option<Box<dyn VsaModel>>,
    config: ReasonerConfig,
    /// Exploration temperature for divergent reasoning (1.0 = greedy, >1.0 = more alternatives)
    pub exploration_temperature: f64,
    /// P0.4: Multi-Head Resonator for VSA bundle decomposition (4 parallel heads)
    pub multi_head_resonator: Option<MultiHeadResonator>,
}

impl Clone for VsaReasoner {
    fn clone(&self) -> Self {
        Self {
            blackboard: self.blackboard.clone(),
            pattern_matcher: self.pattern_matcher.clone(),
            last_analogical_matches: self.last_analogical_matches.clone(),
            vsa_model: Some(ModelFactory::create(
                self.config.vsa_model_type,
                self.config.dimension,
            )),
            config: self.config.clone(),
            exploration_temperature: self.exploration_temperature,
            multi_head_resonator: self.multi_head_resonator.clone(),
        }
    }
}

impl VsaReasoner {
    pub fn new(config: ReasonerConfig) -> Self {
        let blackboard = VsaBlackboard::new(config.max_blackboard_hypotheses);
        let pattern_matcher = PatternMatcher::new(config.analogical_threshold);
        let dim = config.dimension;
        let vsa_model = Some(ModelFactory::create(config.vsa_model_type, dim));
        Self {
            blackboard,
            pattern_matcher,
            last_analogical_matches: Vec::new(),
            vsa_model,
            config,
            exploration_temperature: 1.0,
            multi_head_resonator: None,
        }
    }

    pub fn config(&self) -> &ReasonerConfig {
        &self.config
    }

    /// P0.4: Attach a Multi-Head Resonator for VSA bundle decomposition
    pub fn with_resonator(mut self, resonator: MultiHeadResonator) -> Self {
        self.multi_head_resonator = Some(resonator);
        self
    }

    /// P0.4: Decompose a VSA bundle using multi-head resonator (if available)
    /// Returns decomposed factors with confidence scores.
    pub fn decompose_bundle(&self, bundle: &[u8]) -> Vec<(String, f64)> {
        if let Some(ref resonator) = self.multi_head_resonator {
            resonator
                .decode_aggregated(bundle)
                .into_iter()
                .map(|(label, _, conf)| (label, conf))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn analogical_reason(&mut self, source: &[u8], target: &[u8], context: &[u8]) -> u64 {
        let relation = QuantizedVSA::xor_bind(source, target);
        let analogy = QuantizedVSA::xor_bind(context, &relation);
        let source_target_sim = QuantizedVSA::similarity(source, target);
        let confidence = (1.0 - source_target_sim).clamp(0.0, 1.0);
        let analogy_f64: Vec<f64> = analogy.iter().map(|&b| b as f64).collect();
        let pattern_name = format!("analogical_{}", self.pattern_matcher.codebook_size());
        self.pattern_matcher
            .register_from_text(&pattern_name, "analogical inference");
        self.last_analogical_matches = self.pattern_matcher.analogical_search(&analogy_f64);
        // When exploration_temperature is high, post perturbed alternatives for divergent
        // analogical reasoning
        if self.exploration_temperature > 1.1 {
            let n_alternatives = ((self.exploration_temperature - 1.0) * 5.0).ceil() as usize;
            let n = n_alternatives.min(5);
            for i in 0..n {
                let perturbation = (i as f64 + 1.0) * 0.1;
                let alt: Vec<u8> = analogy
                    .iter()
                    .map(|&b| {
                        let f = b as f64;
                        let perturbed = f + perturbation
                            * (if (f * 100.0) as i64 % 2 == 0 {
                                1.0
                            } else {
                                -1.0
                            });
                        (perturbed.round().clamp(0.0, 255.0)) as u8
                    })
                    .collect();
                let alt_confidence = confidence * (1.0 - perturbation * 0.3);
                self.blackboard.post_hypothesis(
                    alt,
                    alt_confidence,
                    ExpertType::Analogical,
                    vec![],
                );
            }
        }
        self.blackboard
            .post_hypothesis(analogy, confidence, ExpertType::Analogical, vec![])
    }

    pub fn causal_reason(&mut self, premises: &[Vec<u8>], goal: &[u8]) -> u64 {
        if premises.is_empty() {
            return self
                .blackboard
                .post_hypothesis(goal.to_vec(), 0.0, ExpertType::Causal, vec![]);
        }
        let mut trajectory = premises[0].clone();
        for premise in premises.iter().skip(1) {
            trajectory = QuantizedVSA::xor_bind(&trajectory, premise);
        }
        let confidence = QuantizedVSA::similarity(&trajectory, goal);
        self.blackboard
            .post_hypothesis(trajectory, confidence, ExpertType::Causal, vec![])
    }

    pub fn multi_hop_reason(&mut self, query: &[u8], knowledge_base: &[Vec<u8>]) -> u64 {
        let mut current = query.to_vec();
        let mut evidence_ids = Vec::new();
        let max_depth = self.config.multi_hop_max_depth;

        for hop in 0..max_depth {
            if knowledge_base.is_empty() {
                break;
            }
            // Select next hop: greedy argmax (temp=1.0) or softmax sampling (temp>1.0)
            let best_idx = if self.exploration_temperature > 1.05 {
                // Softmax sampling with temperature
                let sims: Vec<f64> = knowledge_base
                    .iter()
                    .map(|v| QuantizedVSA::similarity(&current, v))
                    .collect();
                let max_sim = sims.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let exps: Vec<f64> = sims
                    .iter()
                    .map(|s| ((s - max_sim) / self.exploration_temperature).exp())
                    .collect();
                let sum_exp: f64 = exps.iter().sum();
                let probs: Vec<f64> = if sum_exp > 1e-30 {
                    exps.iter().map(|e| e / sum_exp).collect()
                } else {
                    vec![1.0 / knowledge_base.len() as f64; knowledge_base.len()]
                };
                // Sample from probability distribution
                let r: f64 = rand::random::<f64>();
                let mut cumulative = 0.0;
                let mut idx = 0;
                for (i, p) in probs.iter().enumerate() {
                    cumulative += p;
                    if r <= cumulative {
                        idx = i;
                        break;
                    }
                }
                idx
            } else {
                knowledge_base
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| {
                        let sa = QuantizedVSA::similarity(&current, a);
                        let sb = QuantizedVSA::similarity(&current, b);
                        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            };

            let best_vec = &knowledge_base[best_idx];
            let sim = QuantizedVSA::similarity(&current, best_vec);
            if sim > self.config.analogical_threshold {
                break;
            }
            let next = QuantizedVSA::xor_bind(&current, best_vec);
            let confidence = 0.9 * (0.8f64.powi(hop as i32 + 1));
            let inter_id = self.blackboard.post_hypothesis(
                next.clone(),
                confidence,
                ExpertType::MultiHop,
                evidence_ids.clone(),
            );
            evidence_ids.push(inter_id);
            current = next;
        }

        let final_confidence = 0.9 * (0.8f64.powi(max_depth as i32));
        let current_f64: Vec<f64> = current.iter().map(|&b| b as f64).collect();
        self.pattern_matcher
            .register_pattern("multi_hop_conclusion", current_f64);
        self.blackboard.post_hypothesis(
            current,
            final_confidence,
            ExpertType::MultiHop,
            evidence_ids,
        )
    }

    pub fn detect_contradictions(&mut self, hypotheses: &[Vec<u8>]) -> Vec<(usize, usize, f64)> {
        let mut contradictions = Vec::new();
        for i in 0..hypotheses.len() {
            for j in (i + 1)..hypotheses.len() {
                let sim = QuantizedVSA::similarity(&hypotheses[i], &hypotheses[j]);
                if sim > self.config.contradiction_threshold {
                    contradictions.push((i, j, sim));
                    let conflict_vec = QuantizedVSA::xor_bind(&hypotheses[i], &hypotheses[j]);
                    let divergence = 1.0 - sim;
                    self.blackboard.post_hypothesis(
                        conflict_vec,
                        divergence,
                        ExpertType::Contradiction,
                        vec![],
                    );
                }
            }
        }
        contradictions
    }

    pub fn synthesize(&mut self) -> u64 {
        let hypotheses = self.blackboard.hypotheses.clone();
        if hypotheses.is_empty() {
            let empty = vec![0u8; self.config.dimension];
            return self
                .blackboard
                .post_hypothesis(empty, 0.0, ExpertType::Synthesis, vec![]);
        }

        let mut analogical: Vec<&Hypothesis> = hypotheses
            .iter()
            .filter(|h| h.expert == ExpertType::Analogical && !h.is_contradicted)
            .collect();
        let mut causal: Vec<&Hypothesis> = hypotheses
            .iter()
            .filter(|h| h.expert == ExpertType::Causal && !h.is_contradicted)
            .collect();
        let mut multihop: Vec<&Hypothesis> = hypotheses
            .iter()
            .filter(|h| h.expert == ExpertType::MultiHop && !h.is_contradicted)
            .collect();

        let sort_desc = |a: &&Hypothesis, b: &&Hypothesis| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        };
        analogical.sort_by(sort_desc);
        causal.sort_by(sort_desc);
        multihop.sort_by(sort_desc);

        let top_a = analogical.first();
        let top_c = causal.first();
        let top_m = multihop.first();

        let mut votes: Vec<&[u8]> = Vec::new();
        let mut weights: Vec<f64> = Vec::new();

        if let Some(h) = top_a {
            votes.push(h.content.as_slice());
            weights.push(h.confidence);
        }
        if let Some(h) = top_c {
            votes.push(h.content.as_slice());
            weights.push(h.confidence);
        }
        if let Some(h) = top_m {
            votes.push(h.content.as_slice());
            weights.push(h.confidence);
        }

        if votes.is_empty() {
            let empty = vec![0u8; self.config.dimension];
            return self
                .blackboard
                .post_hypothesis(empty, 0.0, ExpertType::Synthesis, vec![]);
        }

        let weight_factor = |w: f64| ((w * 10.0).round() as usize).max(1);
        let mut weighted_votes: Vec<&[u8]> = Vec::new();
        for (i, v) in votes.iter().enumerate() {
            let count = weight_factor(weights[i]);
            for _ in 0..count {
                weighted_votes.push(v);
            }
        }

        let synthesis = QuantizedVSA::bundle(&weighted_votes);
        let avg_confidence = weights.iter().sum::<f64>() / weights.len() as f64;

        let evidence_ids: Vec<u64> = [top_a, top_c, top_m]
            .iter()
            .filter_map(|&h| h.map(|h| h.id))
            .collect();

        self.blackboard.post_hypothesis(
            synthesis,
            avg_confidence,
            ExpertType::Synthesis,
            evidence_ids,
        )
    }

    /// Gradient-based pattern adaptation using tensor_graph's backpropagation engine.
    /// Builds a differentiable computation graph where the pattern is a trainable
    /// `ConstVector`, optimizes cosine similarity to positives while minimizing
    /// similarity to negatives, then returns the optimized pattern.
    ///
    /// The graph structure:
    ///   param(ConstVector) ← pattern
    ///     ├─ CosineSimilarity(param, ConstVector(pos_i))  ∀ i ∈ positives
    ///     └─ CosineSimilarity(param, ConstVector(neg_j))  ∀ j ∈ negatives
    ///   loss = Sub(avg_neg_sim, avg_pos_sim)  [minimize = maximize pos, minimize neg]
    ///   output = loss
    ///
    /// Falls back to a simple weighted-average heuristic when tensor_graph returns
    /// an error (e.g., dimension mismatch).
    pub fn tensor_adapt(
        &self,
        pattern: &[f64],
        positives: &[Vec<f64>],
        negatives: &[Vec<f64>],
        lr: f64,
        steps: usize,
    ) -> Vec<f64> {
        let dim = pattern.len();
        if dim == 0 {
            return pattern.to_vec();
        }

        let mut graph = TensorGraph::new();
        let param = graph.add_node(GraphNode::ConstVector(pattern.to_vec()));

        let n_pos = positives.len() as f64;
        let n_neg = negatives.len() as f64;

        let mut pos_sims = Vec::new();
        for pos in positives {
            let pos_node = graph.add_node(GraphNode::ConstVector(pos.clone()));
            pos_sims.push(graph.add_node(GraphNode::CosineSimilarity(param, pos_node)));
        }

        let mut neg_sims = Vec::new();
        for neg in negatives {
            let neg_node = graph.add_node(GraphNode::ConstVector(neg.clone()));
            neg_sims.push(graph.add_node(GraphNode::CosineSimilarity(param, neg_node)));
        }

        let avg_pos = if !pos_sims.is_empty() {
            let weighted: Vec<(tensor_graph::NodeId, f64)> =
                pos_sims.iter().map(|&id| (id, 1.0 / n_pos)).collect();
            graph.add_node(GraphNode::WeightedSum(weighted))
        } else {
            graph.add_node(GraphNode::ConstScalar(0.0))
        };

        let avg_neg = if !neg_sims.is_empty() {
            let weighted: Vec<(tensor_graph::NodeId, f64)> =
                neg_sims.iter().map(|&id| (id, 1.0 / n_neg)).collect();
            graph.add_node(GraphNode::WeightedSum(weighted))
        } else {
            graph.add_node(GraphNode::ConstScalar(0.0))
        };

        // loss = -(avg_pos - avg_neg) = avg_neg - avg_pos
        // minimize this => maximize pos similarity, minimize neg similarity
        let loss = graph.add_node(GraphNode::Sub(avg_neg, avg_pos));
        graph.output = loss;

        // Run tensor_graph optimization with identity loss (graph output is the loss scalar)
        if let Err(_e) = optimize(&mut graph, dim.max(1), lr, steps, |out| out[0]) {
            // Fallback: simple weighted average heuristic
            let mut adapted = pattern.to_vec();
            if !positives.is_empty() {
                for (i, val) in adapted.iter_mut().enumerate() {
                    let pos_contrib: f64 = positives.iter().map(|p| p[i]).sum::<f64>() / n_pos;
                    *val = *val * 0.5 + pos_contrib * 0.5;
                }
            }
            if !negatives.is_empty() {
                for (i, val) in adapted.iter_mut().enumerate() {
                    let neg_contrib: f64 = negatives.iter().map(|n| n[i]).sum::<f64>() / n_neg;
                    *val = *val * 0.7 - neg_contrib * 0.3;
                }
            }
            return adapted;
        }

        // Read back the optimized parameter
        match compute_forward(&graph, dim.max(1)) {
            Ok((all_vals, _)) => all_vals
                .get(param)
                .cloned()
                .unwrap_or_else(|| pattern.to_vec()),
            Err(_) => pattern.to_vec(),
        }
    }

    pub fn pattern_match_report(&self) -> String {
        let mut report = format!(
            "Pattern Matcher Report ({} patterns, {} recent matches):\n",
            self.pattern_matcher.codebook_size(),
            self.last_analogical_matches.len()
        );
        for (name, sim) in &self.last_analogical_matches {
            report.push_str(&format!("  {}: similarity={:.4}\n", name, sim));
        }
        report
    }

    /// Bind two vectors using the active VSA model.
    pub fn vsa_bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        match &self.vsa_model {
            Some(model) => model.bind(a, b),
            None => a.iter().zip(b).map(|(x, y)| x * y).collect(),
        }
    }

    /// Bundle multiple vectors using the active VSA model.
    pub fn vsa_bundle(&self, vecs: &[&[f64]]) -> Vec<f64> {
        match &self.vsa_model {
            Some(model) => model.bundle(vecs),
            None => {
                let dim = vecs[0].len();
                (0..dim)
                    .map(|i| vecs.iter().map(|v| v[i]).sum::<f64>() / vecs.len() as f64)
                    .collect()
            }
        }
    }

    /// Compute similarity using the active VSA model.
    pub fn vsa_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        match &self.vsa_model {
            Some(model) => model.similarity(a, b),
            None => {
                let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
                let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
                let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    dot / (norm_a * norm_b)
                }
            }
        }
    }

    /// Invert a vector using the active VSA model.
    pub fn vsa_invert(&self, v: &[f64]) -> Vec<f64> {
        match &self.vsa_model {
            Some(model) => model.invert(v),
            None => v.iter().map(|x| -x).collect(),
        }
    }

    /// Soft/weighted binding: weighted superposition of items (attention-style).
    /// Each item is (vector, weight). Returns weighted average.
    pub fn vsa_soft_bind(&self, items: &[(&[f64], f64)]) -> Vec<f64> {
        match &self.vsa_model {
            Some(model) => model.soft_bind(items),
            None => {
                if items.is_empty() {
                    return vec![];
                }
                let dim = items[0].0.len();
                let total_weight: f64 = items.iter().map(|(_, w)| w).sum();
                if total_weight == 0.0 {
                    return items[0].0.to_vec();
                }
                (0..dim)
                    .map(|i| items.iter().map(|(v, w)| v[i] * w).sum::<f64>() / total_weight)
                    .collect()
            }
        }
    }

    /// Bidirectional binding with iterative refinement.
    /// Alternates bind(a,·) and bind(b,·) for `iterations` rounds.
    pub fn vsa_bidirectional_bind(&self, a: &[f64], b: &[f64], iterations: usize) -> Vec<f64> {
        match &self.vsa_model {
            Some(model) => model.bidirectional_bind(a, b, iterations),
            None => {
                let mut bound = self.vsa_bind(a, b);
                for _ in 0..iterations.saturating_sub(1) {
                    bound = self.vsa_bind(&bound, &self.vsa_bind(a, &bound));
                    bound = self.vsa_bind(&bound, &self.vsa_bind(b, &bound));
                }
                bound
            }
        }
    }

    /// Orthogonal Subspace Carving bind — projects filler onto role's null space.
    /// Reduces cross-talk noise in high-superposition bundling (arXiv 2606.11391).
    pub fn vsa_osc_bind(&self, filler: &[f64], role: &[f64]) -> Vec<f64> {
        match &self.vsa_model {
            Some(model) => model.osc_bind(filler, role),
            None => {
                let dot: f64 = filler.iter().zip(role.iter()).map(|(a, b)| a * b).sum();
                let role_norm_sq: f64 = role.iter().map(|x| x * x).sum();
                if role_norm_sq < 1e-30 {
                    return filler.to_vec();
                }
                let projection_scale = dot / role_norm_sq;
                filler
                    .iter()
                    .zip(role.iter())
                    .map(|(f, r)| f - projection_scale * r)
                    .collect()
            }
        }
    }

    /// Builder: set VSA model type (replaces the current model).
    pub fn with_vsa_model(mut self, model_type: SupportedModel) -> Self {
        self.vsa_model = Some(ModelFactory::create(model_type, self.config.dimension));
        self
    }

    pub fn benchmark(&mut self) -> BenchmarkReport {
        let source = QuantizedVSA::seeded_random(100, self.config.dimension);
        let target = QuantizedVSA::seeded_random(200, self.config.dimension);
        let context = QuantizedVSA::seeded_random(300, self.config.dimension);
        let goal = QuantizedVSA::seeded_random(400, self.config.dimension);
        let query = QuantizedVSA::seeded_random(500, self.config.dimension);

        let premises = vec![
            QuantizedVSA::seeded_random(601, self.config.dimension),
            QuantizedVSA::seeded_random(602, self.config.dimension),
            QuantizedVSA::seeded_random(603, self.config.dimension),
        ];

        let kb: Vec<Vec<u8>> = (0..20)
            .map(|i| QuantizedVSA::seeded_random(700 + i as u64, self.config.dimension))
            .collect();

        let start = std::time::Instant::now();
        let analogy_id = self.analogical_reason(&source, &target, &context);
        let analogy_time = start.elapsed().as_nanos() as u64;
        let analogy_accuracy = self
            .blackboard
            .get_hypothesis(analogy_id)
            .map_or(0.0, |h| h.confidence);

        let start = std::time::Instant::now();
        let causal_id = self.causal_reason(&premises, &goal);
        let causal_time = start.elapsed().as_nanos() as u64;
        let causal_accuracy = self
            .blackboard
            .get_hypothesis(causal_id)
            .map_or(0.0, |h| h.confidence);

        let start = std::time::Instant::now();
        let multihop_id = self.multi_hop_reason(&query, &kb);
        let multihop_time = start.elapsed().as_nanos() as u64;
        let multihop_accuracy = self
            .blackboard
            .get_hypothesis(multihop_id)
            .map_or(0.0, |h| h.confidence);

        let test_hypotheses = vec![
            QuantizedVSA::seeded_random(800, self.config.dimension),
            QuantizedVSA::seeded_random(801, self.config.dimension),
            QuantizedVSA::seeded_random(800, self.config.dimension),
        ];
        let start = std::time::Instant::now();
        let contradictions = self.detect_contradictions(&test_hypotheses);
        let contradiction_time = start.elapsed().as_nanos() as u64;

        let start = std::time::Instant::now();
        let synth_id = self.synthesize();
        let synth_time = start.elapsed().as_nanos() as u64;
        let synthesis_accuracy = self
            .blackboard
            .get_hypothesis(synth_id)
            .map_or(0.0, |h| h.confidence);

        BenchmarkReport {
            analogical_accuracy: analogy_accuracy,
            analogical_time_ns: analogy_time,
            causal_accuracy,
            causal_time_ns: causal_time,
            multi_hop_accuracy: multihop_accuracy,
            multi_hop_time_ns: multihop_time,
            contradiction_count: contradictions.len(),
            contradiction_time_ns: contradiction_time,
            synthesis_accuracy,
            synthesis_time_ns: synth_time,
            n_hypotheses: self.blackboard.hypotheses.len(),
            dimension: self.config.dimension,
        }
    }

    pub fn reason_cycle(&mut self, input: &[u8], context: &[Vec<u8>]) -> Option<Hypothesis> {
        self.blackboard.clear();

        let source = input;
        let target = context.first().map_or(input, |c| c.as_slice());
        let context_vsa = if context.len() > 1 {
            &context[1]
        } else {
            input
        };

        self.analogical_reason(source, target, context_vsa);

        if context.len() >= 2 {
            self.causal_reason(context, input);
        } else {
            let premises = vec![input.to_vec()];
            self.causal_reason(&premises, input);
        }

        self.multi_hop_reason(input, context);

        let all_contents: Vec<Vec<u8>> = self
            .blackboard
            .hypotheses
            .iter()
            .map(|h| h.content.clone())
            .collect();
        self.detect_contradictions(&all_contents);
        self.blackboard.resolve_conflicts();
        self.synthesize();

        self.blackboard.best_hypothesis().cloned()
    }
}

/// Helper: cosine similarity on Vec<f64> for real-valued VSA vectors.
pub fn cosine_similarity_f64(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum();
    let nb: f64 = b.iter().map(|x| x * x).sum();
    if na > 1e-10 && nb > 1e-10 {
        dot / (na.sqrt() * nb.sqrt())
    } else {
        0.0
    }
}

/// Gradient-based pattern adaptation using tensor_graph-style optimization.
/// Maximizes cosine similarity to `positives` while minimizing to `negatives`.
/// Uses numerical gradients (finite differences) over real-valued VSA vectors.
pub fn adapt_pattern(
    pattern: &mut [f64],
    positives: &[Vec<f64>],
    negatives: &[Vec<f64>],
    lr: f64,
    steps: usize,
) {
    let eps = 1e-6;
    let n_pos = positives.len().max(1) as f64;
    let n_neg = negatives.len().max(1) as f64;
    for _ in 0..steps {
        let pos_sim: f64 = positives
            .iter()
            .map(|p| cosine_similarity_f64(pattern, p))
            .sum::<f64>()
            / n_pos;
        let neg_sim: f64 = negatives
            .iter()
            .map(|n| cosine_similarity_f64(pattern, n))
            .sum::<f64>()
            / n_neg;
        let loss = -(pos_sim - neg_sim);
        let mut grad = vec![0.0; pattern.len()];
        for i in 0..pattern.len() {
            let old = pattern[i];
            pattern[i] = old + eps;
            let pos_sim2: f64 = positives
                .iter()
                .map(|p| cosine_similarity_f64(pattern, p))
                .sum::<f64>()
                / n_pos;
            let neg_sim2: f64 = negatives
                .iter()
                .map(|n| cosine_similarity_f64(pattern, n))
                .sum::<f64>()
                / n_neg;
            let loss2 = -(pos_sim2 - neg_sim2);
            grad[i] = (loss2 - loss) / eps;
            pattern[i] = old;
        }
        for i in 0..pattern.len() {
            pattern[i] -= lr * grad[i];
        }
    }
}

/// PatternMatcher: stores a codebook of real-valued VSA patterns and supports
/// analogical search (cosine similarity threshold + ranking).
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    codebook: Vec<(String, Vec<f64>)>,
    threshold: f64,
}

impl PatternMatcher {
    pub fn new(threshold: f64) -> Self {
        Self {
            codebook: Vec::new(),
            threshold,
        }
    }

    pub fn register_pattern(&mut self, name: &str, vsa_real: Vec<f64>) {
        self.codebook.push((name.to_string(), vsa_real));
    }

    pub fn register_from_text(&mut self, name: &str, text: &str) {
        let vsa = VsaVector::<4096>::from_text(text).to_f64_dense();
        self.codebook.push((name.to_string(), vsa));
    }

    pub fn analogical_search(&self, query_real: &[f64]) -> Vec<(String, f64)> {
        let mut results: Vec<(String, f64)> = self
            .codebook
            .iter()
            .map(|(name, vec)| {
                let sim = cosine_similarity_f64(query_real, vec);
                (name.clone(), sim)
            })
            .filter(|(_, sim)| *sim >= self.threshold)
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn codebook_size(&self) -> usize {
        self.codebook.len()
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub analogical_accuracy: f64,
    pub analogical_time_ns: u64,
    pub causal_accuracy: f64,
    pub causal_time_ns: u64,
    pub multi_hop_accuracy: f64,
    pub multi_hop_time_ns: u64,
    pub contradiction_count: usize,
    pub contradiction_time_ns: u64,
    pub synthesis_accuracy: f64,
    pub synthesis_time_ns: u64,
    pub n_hypotheses: usize,
    pub dimension: usize,
}
