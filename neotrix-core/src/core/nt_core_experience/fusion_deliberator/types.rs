use crate::core::nt_core_experience::vsa_judge::{JudgeAnalysis, PanelResult, VSAJudge};
use crate::core::nt_core_hcube::QuantizedVSA;
use std::time::Instant;

// ─── Chain Config ───

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: usize,
    pub vsa_seed: u64,
    pub similarity_threshold: f64,
    pub coherence_bias: f64,
    pub exploration_rate: f64,
    pub label: String,
}

impl ChainConfig {
    pub fn new(chain_id: usize, label: &str) -> Self {
        Self {
            chain_id,
            vsa_seed: chain_id as u64 * 6364136223846793005,
            similarity_threshold: 0.5 + (chain_id as f64 * 0.08).min(0.4),
            coherence_bias: 0.3 + (chain_id as f64 * 0.06).min(0.3),
            exploration_rate: (chain_id as f64 * 0.05).min(0.5),
            label: label.to_string(),
        }
    }

    pub fn diverse_pool(n: usize) -> Vec<Self> {
        let labels = [
            "analytical",
            "creative",
            "conservative",
            "exploratory",
            "balanced",
            "critical",
            "synthetic",
            "intuitive",
        ];
        (0..n)
            .map(|i| {
                let mut c = Self::new(i, labels.get(i).unwrap_or(&"adaptive"));
                c.vsa_seed = (i as u64 + 1) * 6364136223846793005;
                c
            })
            .collect()
    }
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::new(0, "default")
    }
}

// ─── Deliberation Gate ───

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum DeliberatorTaskType {
    Research,
    Creative,
    Analytical,
    Simple,
    Reflexive,
    Unknown,
}

impl DeliberatorTaskType {
    pub fn estimate_complexity(&self) -> f64 {
        match self {
            DeliberatorTaskType::Research => 0.8,
            DeliberatorTaskType::Creative => 0.7,
            DeliberatorTaskType::Analytical => 0.9,
            DeliberatorTaskType::Simple => 0.2,
            DeliberatorTaskType::Reflexive => 0.1,
            DeliberatorTaskType::Unknown => 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GateContext {
    pub cognitive_load: f64,
    pub cycle: u64,
    pub recent_deliberation_count: u64,
    pub task_type: DeliberatorTaskType,
    pub query_entropy: f64,
}

#[derive(Debug, Clone)]
pub struct DeliberationGate {
    pub complexity_threshold: f64,
    pub uncertainty_threshold: f64,
    pub max_deliberations_per_window: u64,
    pub deliberation_window: u64,
}

impl Default for DeliberationGate {
    fn default() -> Self {
        Self {
            complexity_threshold: 0.5,
            uncertainty_threshold: 0.4,
            max_deliberations_per_window: 5,
            deliberation_window: 100,
        }
    }
}

impl DeliberationGate {
    pub fn new(complexity_threshold: f64, uncertainty_threshold: f64) -> Self {
        Self {
            complexity_threshold,
            uncertainty_threshold,
            max_deliberations_per_window: 5,
            deliberation_window: 100,
        }
    }

    pub fn should_deliberate(&self, _query: &[u8], context: &GateContext) -> bool {
        if context.cognitive_load > 0.8 {
            return false;
        }
        if context.recent_deliberation_count >= self.max_deliberations_per_window {
            return false;
        }
        let complexity = context.task_type.estimate_complexity();
        if complexity < self.complexity_threshold
            && context.query_entropy < self.uncertainty_threshold
        {
            return false;
        }
        true
    }

    pub fn estimate_benefit(&self, _query: &[u8], context: &GateContext) -> f64 {
        let complexity = context.task_type.estimate_complexity();
        let uncertainty = context.query_entropy;
        (complexity * 0.6 + uncertainty * 0.4).clamp(0.0, 1.0)
    }

    pub fn estimate_cost(&self, panel_size: usize) -> f64 {
        panel_size as f64 * 0.15
    }
}

// ─── Deliberation Cache ───

#[derive(Debug, Clone)]
pub struct CachedDeliberation {
    pub query_hash: u64,
    pub synthesis: Vec<u8>,
    pub analysis: JudgeAnalysis,
    pub timestamp: std::time::Instant,
    pub hit_count: u64,
}

#[derive(Debug, Clone)]
pub struct DeliberationCache {
    entries: Vec<CachedDeliberation>,
    capacity: usize,
    similarity_threshold: f64,
}

impl DeliberationCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
            similarity_threshold: 0.92,
        }
    }

    pub fn lookup(&self, query: &[u8]) -> Option<&CachedDeliberation> {
        let query_hash = self.compute_hash(query);
        for entry in &self.entries {
            if entry.query_hash == query_hash {
                let sim = QuantizedVSA::cosine(query, &entry.synthesis);
                if sim >= self.similarity_threshold {
                    return Some(entry);
                }
            }
        }
        None
    }

    pub fn lookup_mut(&mut self, query: &[u8]) -> Option<&mut CachedDeliberation> {
        let query_hash = self.compute_hash(query);
        let threshold = self.similarity_threshold;
        for entry in &mut self.entries {
            if entry.query_hash == query_hash {
                let sim = QuantizedVSA::cosine(query, &entry.synthesis);
                if sim >= threshold {
                    return Some(entry);
                }
            }
        }
        None
    }

    pub fn insert(&mut self, query: &[u8], synthesis: Vec<u8>, analysis: JudgeAnalysis) {
        if self.entries.len() >= self.capacity {
            self.evict_lru();
        }
        let hash = self.compute_hash(query);
        self.entries.push(CachedDeliberation {
            query_hash: hash,
            synthesis,
            analysis,
            timestamp: std::time::Instant::now(),
            hit_count: 0,
        });
    }

    pub fn record_hit(&mut self, query: &[u8]) {
        if let Some(entry) = self.lookup_mut(query) {
            entry.hit_count += 1;
            entry.timestamp = std::time::Instant::now();
        }
    }

    fn evict_lru(&mut self) {
        if let Some(idx) = self
            .entries
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.timestamp)
            .map(|(i, _)| i)
        {
            self.entries.remove(idx);
        }
    }

    fn compute_hash(&self, v: &[u8]) -> u64 {
        v.iter()
            .fold(0u64, |h, &b| h.wrapping_mul(31).wrapping_add(b as u64))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for DeliberationCache {
    fn default() -> Self {
        Self::new(100)
    }
}

// ─── Parallel Reasoning Panel ───

#[derive(Debug, Clone)]
pub struct ParallelReasoningPanel {
    pub chains: Vec<ChainConfig>,
}

impl Default for ParallelReasoningPanel {
    fn default() -> Self {
        Self::with_chains(ChainConfig::diverse_pool(4))
    }
}

impl ParallelReasoningPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_chains(chains: Vec<ChainConfig>) -> Self {
        Self { chains }
    }

    pub fn with_n_chains(n: usize) -> Self {
        Self::with_chains(ChainConfig::diverse_pool(n.max(2).min(8)))
    }

    pub fn run_panel(&self, query: &[u8]) -> Vec<PanelResult> {
        self.chains
            .iter()
            .map(|config| {
                let start = Instant::now();
                let result = self.run_single_chain(query, config);
                let elapsed = start.elapsed().as_nanos() as u64;
                PanelResult {
                    chain_id: config.chain_id,
                    thought_vector: result.0,
                    confidence: result.1,
                    reasoning_label: config.label.clone(),
                    execution_time_ns: elapsed,
                }
            })
            .collect()
    }

    fn run_single_chain(&self, query: &[u8], config: &ChainConfig) -> (Vec<u8>, f64) {
        let dim = crate::core::nt_core_hcube::VSA_DIM;

        let perspective = QuantizedVSA::seeded_random(config.vsa_seed, dim);
        let alt_seed = QuantizedVSA::seeded_random(config.vsa_seed.wrapping_add(1), dim);
        let neg_seed = QuantizedVSA::seeded_random(config.vsa_seed.wrapping_add(2), dim);

        let thought = match config.label.as_str() {
            "analytical" => {
                let bound = QuantizedVSA::bind(query, &perspective);
                let re_bound = QuantizedVSA::bind(&bound, &alt_seed);
                QuantizedVSA::bundle(&[query, &re_bound])
            }
            "creative" => {
                let shift = (config.vsa_seed % 4096) as isize;
                let permuted = QuantizedVSA::permute(query, shift);
                let bound = QuantizedVSA::bind(&permuted, &perspective);
                let p2 = QuantizedVSA::permute(&bound, 7);
                QuantizedVSA::bundle(&[&bound, &p2])
            }
            "conservative" => {
                let bound = QuantizedVSA::bind(query, &perspective);
                let bundled = QuantizedVSA::bundle(&[query, &bound]);
                QuantizedVSA::bundle(&[query, &bundled])
            }
            "exploratory" => {
                let permuted = QuantizedVSA::permute(query, 1);
                let p2 = QuantizedVSA::permute(query, 3);
                let p3 = QuantizedVSA::permute(query, 7);
                QuantizedVSA::bundle(&[&permuted, &p2, &p3])
            }
            "critical" => {
                let negated = QuantizedVSA::negate(query);
                let bound = QuantizedVSA::bind(query, &perspective);
                QuantizedVSA::bundle(&[&negated, &bound])
            }
            "balanced" => {
                let bound = QuantizedVSA::bind(query, &perspective);
                let permuted = QuantizedVSA::permute(&bound, (config.vsa_seed % 4096) as isize);
                QuantizedVSA::bundle(&[&bound, &permuted])
            }
            "synthetic" => {
                let a = QuantizedVSA::bind(query, &perspective);
                let b = QuantizedVSA::bind(query, &alt_seed);
                let c = QuantizedVSA::bind(query, &neg_seed);
                QuantizedVSA::bundle(&[&a, &b, &c])
            }
            "intuitive" => {
                let bound = QuantizedVSA::bind(query, &perspective);
                let bundled = QuantizedVSA::bundle(&[query, &bound]);
                let permuted = QuantizedVSA::permute(&bundled, 1);
                QuantizedVSA::bundle(&[&bundled, &permuted])
            }
            _ => {
                let bound = QuantizedVSA::bind(query, &perspective);
                let permuted = QuantizedVSA::permute(query, (config.vsa_seed % 4096) as isize);
                QuantizedVSA::bundle(&[&bound, &permuted])
            }
        };

        let self_sim = QuantizedVSA::cosine(&thought, query);
        let noise = QuantizedVSA::seeded_random(config.vsa_seed.wrapping_add(99), dim);
        let noise_sim = QuantizedVSA::cosine(&thought, &noise);
        let confidence = (self_sim * 0.7 + (1.0 - noise_sim) * 0.3).clamp(0.0, 1.0);

        (thought, confidence)
    }

    pub fn chain_count(&self) -> usize {
        self.chains.len()
    }
}

// ─── Phase 37 — 4-Stage Deliberation types ───

#[derive(Debug, Clone)]
pub struct CritiqueRound {
    pub critiquing_chain: usize,
    pub target_chain: usize,
    pub critique_vector: Vec<u8>,
    pub critique_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct RebuttalRound {
    pub chain_id: usize,
    pub rebuttal_vector: Vec<u8>,
    pub rebuttal_confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum DeliberationDepth {
    Standard,
    Deep,
    Full,
}

// ─── Synthesis Mode ───

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SynthesisMode {
    ConsensusWeighted,
    ConsensusPlusNovelty,
    StructuredEnsemble,
}

// ─── P0.16 CoreFringeAttention types ───

#[derive(Debug, Clone)]
pub struct HubNode {
    pub chain_id: usize,
    pub thought_vector: Vec<u8>,
    pub confidence: f64,
    pub inbound_degree: usize,
    pub outbound_balance: isize,
    pub is_hub: bool,
    pub label: String,
}

impl HubNode {
    pub fn from_panel_result(r: &PanelResult) -> Self {
        Self {
            chain_id: r.chain_id,
            thought_vector: r.thought_vector.clone(),
            confidence: r.confidence,
            inbound_degree: 0,
            outbound_balance: 0,
            is_hub: false,
            label: r.reasoning_label.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FusionGraphNode {
    pub thought_vector: Vec<u8>,
    pub confidence: f64,
    pub chain_label: String,
    pub attractor_state: Vec<u8>,
    pub hubness_score: f64,
    pub is_hub: bool,
}

impl FusionGraphNode {
    pub fn from_panel_result(r: &PanelResult, attractor: &[u8]) -> Self {
        Self {
            thought_vector: r.thought_vector.clone(),
            confidence: r.confidence,
            chain_label: r.reasoning_label.clone(),
            attractor_state: attractor.to_vec(),
            hubness_score: 0.0,
            is_hub: false,
        }
    }
}

// ─── Message Passing Modes ───

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum MessageMode {
    HubToSpoke,
    FullGraph,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum PoolMode {
    Max,
    Mean,
    AttentionWeighted,
}

// ─── CoreFringeAttention — Graph Attention Operator ───

#[derive(Debug, Clone)]
pub struct CoreFringeAttention;

impl CoreFringeAttention {
    pub fn compute_attention(nodes: &[FusionGraphNode], tau: f64) -> Vec<Vec<f64>> {
        let n = nodes.len();
        if n == 0 {
            return vec![];
        }
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
        attn
    }

    pub fn message_passing(
        nodes: &[FusionGraphNode],
        attention: &[Vec<f64>],
        mode: MessageMode,
    ) -> Vec<Vec<u8>> {
        let n = nodes.len();
        if n == 0 {
            return vec![];
        }
        let dim = nodes[0].thought_vector.len();
        let mut updated = vec![vec![0i64; dim]; n];

        match mode {
            MessageMode::HubToSpoke => {
                for i in 0..n {
                    for j in 0..n {
                        if i == j {
                            continue;
                        }
                        if nodes[i].is_hub && !nodes[j].is_hub {
                            let w = attention[i][j];
                            for k in 0..dim.min(nodes[j].thought_vector.len()) {
                                if nodes[i].thought_vector[k] > 0 {
                                    updated[j][k] += (w * 256.0) as i64;
                                } else {
                                    updated[j][k] -= (w * 256.0) as i64;
                                }
                            }
                        } else if !nodes[i].is_hub && nodes[j].is_hub {
                            let w = attention[i][j];
                            for k in 0..dim.min(nodes[j].thought_vector.len()) {
                                if nodes[i].thought_vector[k] > 0 {
                                    updated[j][k] += (w * 256.0) as i64;
                                } else {
                                    updated[j][k] -= (w * 256.0) as i64;
                                }
                            }
                        }
                    }
                }
            }
            MessageMode::FullGraph => {
                for i in 0..n {
                    for j in 0..n {
                        if i == j {
                            continue;
                        }
                        let w = attention[i][j];
                        for k in 0..dim.min(nodes[j].thought_vector.len()) {
                            if nodes[j].thought_vector[k] > 0 {
                                updated[i][k] += (w * 256.0) as i64;
                            } else {
                                updated[i][k] -= (w * 256.0) as i64;
                            }
                        }
                    }
                }
            }
        }

        updated
            .iter()
            .map(|row| row.iter().map(|&v| if v > 0 { 1 } else { 0 }).collect())
            .collect()
    }

    pub fn graph_pool(
        _nodes: &[FusionGraphNode],
        updated_vectors: &[Vec<u8>],
        mode: PoolMode,
    ) -> Vec<u8> {
        let n = updated_vectors.len();
        if n == 0 {
            return vec![0u8; crate::core::nt_core_hcube::VSA_DIM];
        }
        let dim = updated_vectors[0].len();

        match mode {
            PoolMode::Max => {
                let mut pooled = vec![0u8; dim];
                for v in updated_vectors {
                    for k in 0..dim.min(v.len()) {
                        if v[k] > pooled[k] {
                            pooled[k] = v[k];
                        }
                    }
                }
                pooled
            }
            PoolMode::Mean => {
                let mut sum = vec![0i64; dim];
                for v in updated_vectors {
                    for k in 0..dim.min(v.len()) {
                        sum[k] += v[k] as i64;
                    }
                }
                let nf = n as i64;
                sum.iter()
                    .map(|&s| {
                        let avg = s as f64 / nf as f64;
                        if avg > 0.5 {
                            1
                        } else {
                            0
                        }
                    })
                    .collect()
            }
            PoolMode::AttentionWeighted => {
                let mut weights = vec![0.0f64; n];
                for i in 0..n {
                    for j in 0..n {
                        if i != j {
                            let sim =
                                QuantizedVSA::cosine(&updated_vectors[i], &updated_vectors[j]);
                            weights[i] += sim;
                        }
                    }
                    weights[i] /= (n - 1).max(1) as f64;
                }
                let w_sum: f64 = weights.iter().sum();
                if w_sum == 0.0 {
                    return updated_vectors[0].clone();
                }
                let mut pooled = vec![0i64; dim];
                for (i, v) in updated_vectors.iter().enumerate() {
                    let w = weights[i] / w_sum;
                    for k in 0..dim.min(v.len()) {
                        if v[k] > 0 {
                            pooled[k] += (w * 256.0) as i64;
                        } else {
                            pooled[k] -= (w * 256.0) as i64;
                        }
                    }
                }
                pooled.iter().map(|&v| if v > 0 { 1 } else { 0 }).collect()
            }
        }
    }

    pub fn detect_hubs(nodes: &mut [FusionGraphNode], z_threshold: f64) {
        let n = nodes.len();
        if n < 3 {
            for node in nodes.iter_mut() {
                node.hubness_score = 0.0;
                node.is_hub = false;
            }
            return;
        }

        let attention = Self::compute_attention(nodes, 0.5);
        let mut in_degrees = vec![0.0f64; n];
        for j in 0..n {
            for i in 0..n {
                if i != j {
                    in_degrees[j] += attention[i][j];
                }
            }
        }

        let mean = in_degrees.iter().sum::<f64>() / n as f64;
        let variance = in_degrees.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / n as f64;
        let std = variance.sqrt();

        for (i, node) in nodes.iter_mut().enumerate() {
            node.hubness_score = if std > 0.0 {
                (in_degrees[i] - mean) / std
            } else {
                0.0
            };
            node.is_hub = node.hubness_score > z_threshold;
        }
    }
}

// ─── TRT Strategy & Rollout ───

#[derive(Clone, Debug)]
pub struct TrtStrategy {
    pub explore_rate: f64,
    pub temperature: f64,
    pub n_perspectives: usize,
    pub self_verify: bool,
    pub label: String,
}

impl Default for TrtStrategy {
    fn default() -> Self {
        Self {
            explore_rate: 0.3,
            temperature: 1.0,
            n_perspectives: 4,
            self_verify: true,
            label: "balanced".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrtRollout {
    pub idx: usize,
    pub query: Vec<u8>,
    pub strategy: TrtStrategy,
    pub thoughts: Vec<Vec<u8>>,
    pub self_verify_score: f64,
    pub accumulated_knowledge: String,
    pub accepted: bool,
}

// ─── Fusion Deliberator ───

#[derive(Debug, Clone)]
pub struct ChainHistory {
    pub chain_id: usize,
    pub accuracy_sum: f64,
    pub accuracy_samples: u64,
    pub total_weight: f64,
    pub label: String,
}

impl ChainHistory {
    pub fn new(chain_id: usize, label: &str) -> Self {
        Self {
            chain_id,
            accuracy_sum: 0.0,
            accuracy_samples: 0,
            total_weight: 1.0,
            label: label.to_string(),
        }
    }

    pub fn record_accuracy(&mut self, similarity_to_synthesis: f64) {
        self.accuracy_sum += similarity_to_synthesis.clamp(0.0, 1.0);
        self.accuracy_samples += 1;
        let raw = self.accuracy_sum / self.accuracy_samples.max(1) as f64;
        self.total_weight = 0.5 + raw * 0.5;
    }

    pub fn historical_accuracy(&self) -> f64 {
        if self.accuracy_samples == 0 {
            return 0.5;
        }
        self.accuracy_sum / self.accuracy_samples as f64
    }
}

#[derive(Debug, Clone)]
pub struct FusionStats {
    pub total_deliberations: u64,
    pub total_hierarchical: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub gate_blocks: u64,
    pub gate_skipped_benefit: u64,
    pub avg_panel_size: f64,
    pub avg_confidence: f64,
    pub deep_deliberations: u64,
    pub full_deliberations: u64,
}

#[derive(Debug, Clone)]
pub struct FusionDeliberator {
    pub panel: ParallelReasoningPanel,
    pub judge: VSAJudge,
    pub gate: Option<DeliberationGate>,
    pub cache: Option<DeliberationCache>,
    pub synthesis_mode: SynthesisMode,
    pub max_depth: u8,

    pub(crate) deliberation_count: u64,
    pub(crate) hierarchical_count: u64,
    pub(crate) cache_hits: u64,
    pub(crate) cache_misses: u64,
    pub(crate) gate_blocks: u64,
    pub(crate) gate_skipped_benefit: u64,
    pub(crate) running_confidence_sum: f64,
    pub(crate) running_panel_sum: usize,
    pub(crate) running_samples: u64,
    pub(crate) deep_deliberations: u64,
    pub(crate) full_deliberations: u64,
    pub(crate) chain_histories: Vec<ChainHistory>,
    pub(crate) min_benefit_threshold: f64,
    pub graph_mode: bool,
    pub pool_mode: PoolMode,
    pub(crate) trt_strategy: TrtStrategy,
    pub(crate) trt_history: Vec<TrtRollout>,
    pub(crate) _trt_max_iters: usize,
}

impl Default for FusionDeliberator {
    fn default() -> Self {
        let chains = ParallelReasoningPanel::default().chains.clone();
        let histories: Vec<ChainHistory> = chains
            .iter()
            .map(|c| ChainHistory::new(c.chain_id, &c.label))
            .collect();
        Self {
            panel: ParallelReasoningPanel::default(),
            judge: VSAJudge::default(),
            gate: Some(DeliberationGate::default()),
            cache: Some(DeliberationCache::default()),
            synthesis_mode: SynthesisMode::ConsensusPlusNovelty,
            max_depth: 2,
            deliberation_count: 0,
            hierarchical_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            gate_blocks: 0,
            gate_skipped_benefit: 0,
            running_confidence_sum: 0.0,
            running_panel_sum: 0,
            running_samples: 0,
            deep_deliberations: 0,
            full_deliberations: 0,
            chain_histories: histories,
            min_benefit_threshold: 0.15,
            graph_mode: false,
            pool_mode: PoolMode::AttentionWeighted,
            trt_strategy: TrtStrategy::default(),
            trt_history: Vec::new(),
            _trt_max_iters: 10,
        }
    }
}

impl FusionDeliberator {
    pub fn new(panel: ParallelReasoningPanel, judge: VSAJudge) -> Self {
        Self {
            panel,
            judge,
            ..Default::default()
        }
    }

    pub fn with_gate(mut self, gate: DeliberationGate) -> Self {
        self.gate = Some(gate);
        self
    }

    pub fn with_cache(mut self, capacity: usize) -> Self {
        self.cache = Some(DeliberationCache::new(capacity));
        self
    }

    pub fn with_synthesis_mode(mut self, mode: SynthesisMode) -> Self {
        self.synthesis_mode = mode;
        self
    }
}
