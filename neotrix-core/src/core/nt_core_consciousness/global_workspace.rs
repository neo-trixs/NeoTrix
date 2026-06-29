use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct WorkspaceProposal {
    pub module_name: String,
    pub vector: Vec<u8>,
    pub salience: f64,
    pub timestamp: u64,
    pub context: String,
    /// Processor type for CTM-AI competition tracking
    pub processor_type: Option<String>,
    /// Diversity bonus: how different this proposal is from others (0-1)
    pub diversity_score: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessorStats {
    pub processor_id: String,
    pub proposals_submitted: u64,
    pub broadcasts_won: u64,
    pub last_broadcast_cycle: u64,
    pub avg_salience: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone)]
pub struct BroadcastContent {
    pub winner: String,
    pub vector: Vec<u8>,
    pub salience: f64,
    pub runner_up: Option<(String, f64)>,
    pub broadcast_cycle: u64,
    /// Processor types in competition pool for this round
    pub processor_count: usize,
    /// Diversity of competing proposals (entropy-based)
    pub competition_diversity: f64,
}

pub struct GlobalLatentWorkspace {
    pub slots: Vec<WorkspaceProposal>,
    pub broadcast_history: Vec<BroadcastContent>,
    pub max_slots: usize,
    pub min_salience_threshold: f64,
    pub broadcast_cooldown: u64,
    pub last_broadcast_cycle: u64,
    pub workspace_state: Vec<u8>,
    pub decay_rate: f64,
    cycle: u64,
    /// CTM-AI: processor registry for up-tree/down-tree competition
    pub processors: Vec<ProcessorStats>,
    /// Up-tree: diversity weight in salience computation
    pub diversity_weight: f64,
    /// Down-tree: how much broadcast influences workspace_state
    pub down_tree_influence: f64,
}

impl GlobalLatentWorkspace {
    pub fn new() -> Self {
        Self {
            slots: Vec::with_capacity(16),
            broadcast_history: Vec::with_capacity(32),
            max_slots: 16,
            min_salience_threshold: 0.3,
            broadcast_cooldown: 3,
            last_broadcast_cycle: 0,
            workspace_state: vec![0u8; VSA_DIM],
            decay_rate: 0.01,
            cycle: 0,
            processors: Vec::new(),
            diversity_weight: 0.3,
            down_tree_influence: 0.5,
        }
    }

    pub fn submit_proposal(&mut self, module: &str, vector: Vec<u8>, context: &str, cycle: u64) {
        let salience = self.compute_salience(&vector, context, module);
        if self.slots.len() >= self.max_slots {
            // Replace lowest salience slot
            let mut min_idx = 0;
            let mut min_sal = self.slots[0].salience;
            for (i, slot) in self.slots.iter().enumerate().skip(1) {
                if slot.salience < min_sal {
                    min_sal = slot.salience;
                    min_idx = i;
                }
            }
            if salience > min_sal {
                self.slots[min_idx] = WorkspaceProposal {
                    module_name: module.to_string(),
                    vector,
                    salience,
                    timestamp: cycle,
                    context: context.to_string(),
                    processor_type: None,
                    diversity_score: 0.0,
                };
            }
        } else {
            self.slots.push(WorkspaceProposal {
                module_name: module.to_string(),
                vector,
                salience,
                timestamp: cycle,
                context: context.to_string(),
                processor_type: None,
                diversity_score: 0.0,
            });
        }
    }

    pub fn compute_salience(&self, vector: &[u8], context: &str, module: &str) -> f64 {
        self.compute_salience_with_diversity(vector, context, module, 0.5)
    }

    /// CTM-AI up-tree: salience includes diversity bonus for processor competition
    fn compute_salience_with_diversity(
        &self,
        vector: &[u8],
        context: &str,
        module: &str,
        diversity_score: f64,
    ) -> f64 {
        let novelty = if self.workspace_state.iter().any(|&b| b != 0) {
            let sim = Self::vsa_similarity(vector, &self.workspace_state);
            1.0 - sim
        } else {
            0.5
        };

        let self_sim = {
            let own: Vec<&WorkspaceProposal> = self
                .slots
                .iter()
                .filter(|p| p.module_name == module)
                .collect();
            if own.is_empty() {
                0.5
            } else {
                let mean_sim: f64 = own
                    .iter()
                    .map(|p| Self::vsa_similarity(vector, &p.vector))
                    .sum::<f64>()
                    / own.len() as f64;
                mean_sim
            }
        };

        let max_ctx = 256.0;
        let ctx_richness = (context.len() as f64).min(max_ctx) / max_ctx;

        // CTM-AI up-tree: 4-component salience with diversity bonus
        0.35 * novelty
            + 0.25 * self_sim
            + 0.25 * ctx_richness
            + self.diversity_weight * diversity_score
    }

    pub fn broadcast(&mut self, cycle: u64) -> Option<BroadcastContent> {
        if self.slots.is_empty() {
            return None;
        }
        if cycle - self.last_broadcast_cycle < self.broadcast_cooldown {
            return None;
        }

        let mut best_idx = 0;
        let mut best_sal = self.slots[0].salience;
        let mut second_best: Option<(usize, f64)> = None;
        for (i, slot) in self.slots.iter().enumerate().skip(1) {
            if slot.salience > best_sal {
                second_best = Some((best_idx, best_sal));
                best_sal = slot.salience;
                best_idx = i;
            } else if second_best.is_none() || slot.salience > second_best.map_or(0.0, |(_, s)| s) {
                second_best = Some((i, slot.salience));
            }
        }

        if best_sal < self.min_salience_threshold {
            return None;
        }

        let winner = self.slots[best_idx].clone();

        // Track processor wins (CTM-AI down-tree)
        if let Some(ref pt) = winner.processor_type {
            if let Some(p) = self
                .processors
                .iter_mut()
                .find(|p| p.processor_id.as_str() == pt)
            {
                p.broadcasts_won += 1;
                p.last_broadcast_cycle = cycle;
                p.win_rate = p.broadcasts_won as f64 / p.proposals_submitted.max(1) as f64;
                p.avg_salience = (p.avg_salience * (p.proposals_submitted - 1) as f64
                    + winner.salience)
                    / p.proposals_submitted.max(1) as f64;
            }
        }

        // VSA bundle winner into workspace_state
        let refs: Vec<&[u8]> = vec![&winner.vector, &self.workspace_state];
        self.workspace_state =
            crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::bundle(&refs);

        self.last_broadcast_cycle = cycle;
        self.cycle = cycle;

        let runner_up = second_best.map(|(i, s)| (self.slots[i].module_name.clone(), s));

        let processor_count = self
            .slots
            .iter()
            .filter_map(|s| s.processor_type.as_ref())
            .collect::<HashSet<_>>()
            .len();
        let competition_diversity = self.processor_diversity_score();

        let bc = BroadcastContent {
            winner: winner.module_name,
            vector: winner.vector,
            salience: winner.salience,
            runner_up,
            broadcast_cycle: cycle,
            processor_count,
            competition_diversity,
        };

        self.broadcast_history.push(bc.clone());
        if self.broadcast_history.len() > 32 {
            self.broadcast_history.remove(0);
        }

        self.slots.clear();

        Some(bc)
    }

    /// CTM-AI down-tree: distribute broadcast influence to processors
    pub fn down_tree_broadcast(&mut self, content: &BroadcastContent) {
        for processor in &mut self.processors {
            let received = Self::vsa_similarity(&content.vector, &self.workspace_state);
            processor.avg_salience = processor.avg_salience * (1.0 - self.down_tree_influence)
                + received * self.down_tree_influence;
        }
    }

    /// Entropy-based diversity score across processor types in current slots
    pub fn processor_diversity_score(&self) -> f64 {
        if self.slots.is_empty() {
            return 0.0;
        }
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for slot in &self.slots {
            let key = slot
                .processor_type
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            *type_counts.entry(key).or_insert(0) += 1;
        }
        let total = self.slots.len() as f64;
        let entropy: f64 = type_counts
            .values()
            .map(|&c| {
                let p = c as f64 / total;
                -p * p.log2()
            })
            .sum();
        // Normalize to [0, 1]: max entropy = log2(n_types)
        let n_types = type_counts.len() as f64;
        if n_types <= 1.0 {
            return 0.0;
        }
        (entropy / n_types.log2()).clamp(0.0, 1.0)
    }

    pub fn reset_processors(&mut self) {
        self.processors.clear();
    }

    pub fn proposal_count(&self) -> usize {
        self.slots.len()
    }

    pub fn decay(&mut self) {
        for b in self.workspace_state.iter_mut() {
            if fastrand::f64() < self.decay_rate {
                *b ^= 1;
            }
        }
    }

    pub fn vsa_similarity(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len()).min(VSA_DIM);
        if len == 0 {
            return 0.0;
        }
        let diff: u32 = a
            .iter()
            .zip(b.iter())
            .take(len)
            .map(|(x, y)| ((x ^ y) & 1) as u32)
            .sum();
        1.0 - (diff as f64 / len as f64)
    }

    pub fn check_convergence(current: &[u8], previous: &[u8], threshold: f64) -> bool {
        Self::vsa_similarity(current, previous) > threshold
    }
}

impl Default for GlobalLatentWorkspace {
    fn default() -> Self {
        Self::new()
    }
}
