use std::collections::HashMap;

// ── Agent Genotype ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AgentGenotype {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub generation: u32,
    pub survived_rounds: u32,
    pub fitness: f64,
    pub traits: HashMap<String, f64>,
}

impl AgentGenotype {
    pub fn new(id: &str, traits: HashMap<String, f64>) -> Self {
        Self {
            id: id.to_string(),
            parent_ids: Vec::new(),
            generation: 0,
            survived_rounds: 0,
            fitness: 0.0,
            traits,
        }
    }

    pub fn mutate(&self, rng: &mut impl FnMut() -> f64, rate: f64, sigma: f64) -> Self {
        let mut traits = self.traits.clone();
        for v in traits.values_mut() {
            if rng() < rate {
                *v += (rng() - 0.5) * 2.0 * sigma;
                *v = v.clamp(0.0, 1.0);
            }
        }
        Self {
            id: format!("{}-m{}", self.id, (rng() * 1_000_000.0) as u64),
            parent_ids: vec![self.id.clone()],
            generation: self.generation + 1,
            survived_rounds: 0,
            fitness: 0.0,
            traits,
        }
    }

    pub fn crossover(&self, other: &Self, rng: &mut impl FnMut() -> f64, rate: f64) -> Self {
        let mut traits = self.traits.clone();
        for (k, v) in traits.iter_mut() {
            if rng() < rate {
                if let Some(other_v) = other.traits.get(k) {
                    *v = if rng() < 0.5 { *other_v } else { *v };
                }
            }
        }
        Self {
            id: format!("c{}-{}-{}", (rng() * 1_000_000.0) as u64, self.id, other.id),
            parent_ids: vec![self.id.clone(), other.id.clone()],
            generation: self.generation.max(other.generation) + 1,
            survived_rounds: 0,
            fitness: 0.0,
            traits,
        }
    }
}

// ── Adversarial Arena ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ArenaConfig {
    pub population_size: usize,
    pub tournament_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub mutation_sigma: f64,
    pub elite_count: usize,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            population_size: 20,
            tournament_size: 3,
            mutation_rate: 0.2,
            crossover_rate: 0.3,
            mutation_sigma: 0.1,
            elite_count: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub round: u64,
    pub task: String,
    pub agent_id: String,
    pub opponent_id: String,
    pub agent_score: f64,
    pub opponent_score: f64,
    pub agent_won: bool,
}

#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub generation: u32,
    pub population_size: usize,
    pub matches: Vec<MatchResult>,
    pub top_fitness: f64,
    pub avg_fitness: f64,
    pub diversity: f64,
}

// ── Gödel Agent Self-Reference ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GödelAgent {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub generation: u32,
    pub fitness: f64,
    pub code: String,
    pub traits: HashMap<String, f64>,
    pub archive_entries: Vec<String>,
    pub base_strategy: String,
    pub weight_deltas: HashMap<String, f64>,
    pub harness_version: u32,
}

// ── Harness+Weights Dual Update ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HarnessWeightsUpdate {
    pub delta_updates: Vec<(String, f64)>,
    pub is_harness_update: bool,
    pub new_base_strategy: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct DualUpdateConfig {
    pub weight_lr: f64,
    pub harness_update_prob: f64,
    pub max_weight_sum: f64,
}

impl Default for DualUpdateConfig {
    fn default() -> Self {
        Self {
            weight_lr: 0.1,
            harness_update_prob: 0.05,
            max_weight_sum: 2.0,
        }
    }
}

impl DualUpdateConfig {
    pub fn new(weight_lr: f64, harness_update_prob: f64, max_weight_sum: f64) -> Self {
        Self {
            weight_lr: weight_lr.max(0.0).min(1.0),
            harness_update_prob: harness_update_prob.max(0.0).min(1.0),
            max_weight_sum: max_weight_sum.max(0.1),
        }
    }
}

impl GödelAgent {
    fn regenerate_code(&mut self) {
        if self.weight_deltas.is_empty() {
            self.code = self.base_strategy.clone();
        } else {
            let mut annotations = String::new();
            let mut pairs: Vec<(&String, &f64)> = self.weight_deltas.iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(b.0));
            for (k, v) in &pairs {
                let sign = if **v >= 0.0 { '+' } else { '-' };
                annotations.push_str(&format!("// w[{}] {} {:.4}\n", k, sign, v.abs()));
            }
            self.code = format!(
                "{}\n// -- weight deltas --\n{}",
                self.base_strategy, annotations
            );
        }
    }

    pub fn apply_dual_update(&mut self, update: HarnessWeightsUpdate) {
        if update.is_harness_update {
            if let Some(new_strategy) = update.new_base_strategy {
                self.base_strategy = new_strategy;
            }
            self.harness_version += 1;
            self.weight_deltas.clear();
            self.regenerate_code();
        } else {
            for (key, mut delta) in update.delta_updates {
                delta = delta.clamp(-0.5, 0.5);
                let entry = self.weight_deltas.entry(key).or_insert(0.0);
                *entry = (*entry + delta).clamp(-0.5, 0.5);
            }
            let total: f64 = self.weight_deltas.values().map(|v| v.abs()).sum();
            let cap = 2.0;
            if total > cap {
                let scale = cap / total;
                for v in self.weight_deltas.values_mut() {
                    *v *= scale;
                }
            }
            self.regenerate_code();
        }
    }

    pub fn effective_trait(&self, key: &str) -> f64 {
        let base = self.traits.get(key).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let delta = self.weight_deltas.get(key).copied().unwrap_or(0.0);
        (base + delta).clamp(0.0, 1.0)
    }

    pub fn harness_update_cost(&self) -> f64 {
        1.0 / (1.0 + self.harness_version as f64)
    }

    pub fn suggest_updates(
        &self,
        task_performance: &[(String, f64)],
        config: &DualUpdateConfig,
    ) -> Vec<HarnessWeightsUpdate> {
        let mut updates = Vec::new();

        for (trait_name, perf) in task_performance {
            if self.traits.contains_key(trait_name) {
                let gap = 0.5 - perf;
                if gap > 0.0 {
                    let delta = (gap * config.weight_lr).min(0.5);
                    updates.push(HarnessWeightsUpdate {
                        delta_updates: vec![(trait_name.clone(), delta)],
                        is_harness_update: false,
                        new_base_strategy: None,
                        confidence: gap.min(1.0),
                    });
                }
            }
        }

        let avg_perf: f64 = if task_performance.is_empty() {
            0.0
        } else {
            task_performance.iter().map(|(_, p)| p).sum::<f64>() / task_performance.len() as f64
        };

        if !task_performance.is_empty() && avg_perf < config.harness_update_prob {
            updates.push(HarnessWeightsUpdate {
                delta_updates: Vec::new(),
                is_harness_update: true,
                new_base_strategy: Some(format!(
                    "// harness v{} — underperforming traits\nfn agent() -> f64 {{ negentropy() }}",
                    self.harness_version + 1,
                )),
                confidence: 1.0 - avg_perf,
            });
        }

        updates
    }
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub agent_id: String,
    pub fitness: f64,
    pub code_snapshot: String,
    pub generation: u32,
}

// ── SelfReferenceEngine ──────────────────────────────────────────────────
//
// Tracks how the consciousness reflects on its own reasoning processes.
// Maintains a temporal record of self-observation steps and metacognitive
// feedback loops.

/// A self-referential VSA embedding cell that links a query vector to its
/// own earlier activations, forming a causal loop in VSA space.
///
/// Simplified implementation: stores a 1D causal trace keyed by activation
/// step. In a full implementation this would be a VSA vector that binds
/// the current state to a delayed copy of itself.
#[derive(Debug, Clone)]
pub struct CausalSleeperCell {
    /// Step index (monotonic counter)
    pub step: u64,
    /// The VSA binding of current → delayed-self using embedded trace keys
    pub trace_key: Vec<u8>,
    /// Delayed copy of the previous step's trace key (causal link)
    pub delayed_self: Vec<u8>,
}

impl CausalSleeperCell {
    /// Create a new CausalSleeperCell at step 0 with an empty trace.
    pub fn new() -> Self {
        Self {
            step: 0,
            trace_key: vec![0u8; 64],
            delayed_self: vec![0u8; 64],
        }
    }

    /// Advance one step: the current trace_key becomes delayed_self,
    /// and a new trace_key is derived via `new_trace = bind(current, delayed)`.
    pub fn tick(&mut self, current_trace: &[u8]) {
        self.step += 1;
        self.delayed_self = self.trace_key.clone();
        // Simple binding: XOR current with delayed self
        let bound: Vec<u8> = current_trace
            .iter()
            .zip(self.delayed_self.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        self.trace_key = bound;
    }

    /// The self-referential similarity: cosine between new trace and delayed self.
    /// High values indicate the agent is "recognizing" its own prior state.
    pub fn self_similarity(&self) -> f64 {
        if self.trace_key.len() != self.delayed_self.len() || self.trace_key.is_empty() {
            return 0.0;
        }
        let dot: u32 = self
            .trace_key
            .iter()
            .zip(self.delayed_self.iter())
            .map(|(a, b)| (a & b) as u32)
            .sum();
        let norm_a: u32 = self.trace_key.iter().map(|&x| x as u32).sum();
        let norm_b: u32 = self.delayed_self.iter().map(|&x| x as u32).sum();
        if norm_a == 0 || norm_b == 0 {
            return 0.0;
        }
        dot as f64 / ((norm_a as f64) * (norm_b as f64)).sqrt()
    }
}

impl Default for CausalSleeperCell {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks how the agent reflects on its own reasoning chain.
///
/// Maintains a ring buffer of self-observation steps. Each step records
/// the CausalSleeperCell state and a brief reflection label.
#[derive(Debug, Clone)]
pub struct SelfReferenceEngine {
    pub cells: Vec<CausalSleeperCell>,
    pub reflection_log: Vec<SelfReflectionEntry>,
    pub max_cells: usize,
    pub coherence: f64,
}

#[derive(Debug, Clone)]
pub struct SelfReflectionEntry {
    pub step: u64,
    pub label: String,
    pub self_sim: f64,
    pub trace_hash: u64,
}

impl SelfReferenceEngine {
    pub fn new(num_cells: usize) -> Self {
        Self {
            cells: (0..num_cells).map(|_| CausalSleeperCell::new()).collect(),
            reflection_log: Vec::with_capacity(100),
            max_cells: num_cells,
            coherence: 0.0,
        }
    }

    /// Advance all sleeper cells with fresh trace data and record a reflection.
    pub fn reflect(&mut self, label: &str, traces: &[Vec<u8>]) {
        for (i, cell) in self.cells.iter_mut().enumerate() {
            let trace = traces.get(i).cloned().unwrap_or_default();
            cell.tick(&trace);
        }
        let avg_self_sim: f64 = if self.cells.is_empty() {
            0.0
        } else {
            self.cells.iter().map(|c| c.self_similarity()).sum::<f64>() / self.cells.len() as f64
        };
        let trace_hash: u64 = traces
            .iter()
            .flat_map(|t| t.iter())
            .fold(0u64, |h, &b| h.wrapping_mul(31).wrapping_add(b as u64));

        if self.reflection_log.len() >= 100 {
            self.reflection_log.remove(0);
        }
        self.reflection_log.push(SelfReflectionEntry {
            step: self.cells.first().map(|c| c.step).unwrap_or(0),
            label: label.to_string(),
            self_sim: avg_self_sim,
            trace_hash,
        });

        // Coherence = exponential moving average of self-similarity
        self.coherence = self.coherence * 0.9 + avg_self_sim * 0.1;
    }

    pub fn report(&self) -> String {
        let recent: Vec<&SelfReflectionEntry> = self.reflection_log.iter().rev().take(5).collect();
        let recent_str: Vec<String> = recent
            .iter()
            .map(|e| format!("{}:sim={:.3}", e.label, e.self_sim))
            .collect();
        format!(
            "SelfRefEngine: cells={} coherence={:.3} log={} recent=[{}]",
            self.cells.len(),
            self.coherence,
            self.reflection_log.len(),
            recent_str.join(", ")
        )
    }
}

impl Default for SelfReferenceEngine {
    fn default() -> Self {
        Self::new(4)
    }
}
