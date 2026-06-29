//! MANAR: Memory-Augmented Neural Attention with Resonators.
//! Linear-time attention using VSA concept bottleneck slots.
//! Inspired by MANAR (arXiv 2603.18676) — Token-free Reasoning via
//! Abstract Concept Bottleneck, adapted for VSA-based GWT.
//!
//! Replaces O(n²) heuristic salience with O(K) concept slot projection
//! where K=32..64 abstract concept slots form a VSA bottleneck.

use crate::core::nt_core_e8::shao_yong_sequence;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_hcube::vsa_vector::VsaVector;

/// MANAR attention configuration.
#[derive(Debug, Clone)]
pub struct ManarConfig {
    /// Number of abstract concept slots K (default: 32).
    pub num_concept_slots: usize,
    /// Minimum similarity to activate a slot (default: 0.7).
    pub similarity_threshold: f64,
    /// Slot salience decay per tick (default: 0.9).
    pub broadcast_decay: f64,
    /// Minimum entropy before diversity injection (default: 0.3).
    pub diversity_threshold: f64,
    /// Noise fraction for diversity injection (default: 0.05).
    pub injection_noise: f64,
}

impl Default for ManarConfig {
    fn default() -> Self {
        Self {
            num_concept_slots: 32,
            similarity_threshold: 0.7,
            broadcast_decay: 0.9,
            diversity_threshold: 0.3,
            injection_noise: 0.05,
        }
    }
}

/// A single abstract concept slot.
///
/// Each slot holds a VSA vector prototype, an accumulated salience,
/// and a bound_content bundle that accumulates winning proposals via
/// VSA binding (XOR).
#[derive(Debug, Clone)]
pub struct ConceptSlot {
    pub id: usize,
    pub vector: VsaVector,
    pub salience: f64,
    pub age: u64,
    pub bound_content: VsaVector,
}

/// MANAR attention engine — linear-time concept bottleneck attention.
///
/// Projects each VSA proposal onto K abstract concept slots, computes
/// bottleneck salience, and tracks slot memory via VSA binding.
pub struct ManarAttention {
    config: ManarConfig,
    slots: Vec<ConceptSlot>,
    cycle: u64,
    total_projected: u64,
}

impl ManarAttention {
    /// Initialize with K concept slots using deterministic random VSA vectors.
    pub fn new(config: ManarConfig) -> Self {
        let num = config.num_concept_slots;
        let mut slots = Vec::with_capacity(num);
        for i in 0..num {
            slots.push(ConceptSlot {
                id: i,
                vector: VsaVector::random(i as u64 ^ 0x4d41_4e41),
                salience: 0.0,
                age: 0,
                bound_content: VsaVector::new(),
            });
        }
        Self {
            config,
            slots,
            cycle: 0,
            total_projected: 0,
        }
    }

    /// Seed concept slots from E8 64 hexagram states.
    ///
    /// Each hexagram (0..63) seeds a slot's prototype vector via
    /// deterministic randomness. Slots beyond 64 retain their initial vectors.
    pub fn seed_from_e8(&mut self) {
        let hexagrams = shao_yong_sequence();
        let n = self.config.num_concept_slots.min(hexagrams.len());
        for i in 0..n {
            let hex = hexagrams[i];
            self.slots[i].vector = VsaVector::random(0xE8_00_00_00 | (hex.bits as u64));
        }
    }

    /// Project a VSA proposal onto all concept slots.
    ///
    /// Returns a vector of (slot_id, similarity) pairs sorted by slot order.
    /// Complexity: O(K) Hamming distance computations = O(K * VSA_DIM) — LINEAR.
    pub fn project(&self, proposal: &[u8]) -> Vec<(usize, f64)> {
        self.slots
            .iter()
            .map(|slot| {
                let sim = QuantizedVSA::similarity(proposal, slot.vector.as_bytes());
                (slot.id, sim)
            })
            .collect()
    }

    /// Compute concept-bottleneck salience for each proposal.
    ///
    /// For each proposal, finds the maximum similarity to any concept slot.
    /// The output length matches the input proposals length.
    pub fn attend(&mut self, proposals: &[Vec<u8>]) -> Vec<f64> {
        self.total_projected += proposals.len() as u64;
        proposals
            .iter()
            .map(|proposal| {
                self.project(proposal)
                    .into_iter()
                    .map(|(_, sim)| sim)
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or(0.0)
            })
            .collect()
    }

    /// Update a concept slot by binding (XOR) a winning proposal's VSA vector
    /// into the slot's bound_content, and boosting the slot's salience.
    pub fn update_slots(&mut self, winner_idx: usize, proposal_vsa: &[u8]) {
        if winner_idx >= self.slots.len() {
            return;
        }
        let slot = &mut self.slots[winner_idx];
        let rebound = QuantizedVSA::xor_bind(slot.bound_content.as_bytes(), proposal_vsa);
        slot.bound_content = VsaVector::from_bytes(rebound).unwrap_or_default();
        let sim = QuantizedVSA::similarity(proposal_vsa, slot.vector.as_bytes());
        slot.salience = (slot.salience + sim).min(1.0);
        slot.age = self.cycle;
    }

    /// Select the broadcast winner from concept slot space.
    ///
    /// Returns `Some((slot_id, broadcast_content))` for the slot with
    /// highest accumulated salience above the similarity threshold,
    /// or `None` if no slot meets the threshold.
    pub fn select_broadcast(&self) -> Option<(usize, VsaVector)> {
        if self.slots.is_empty() {
            return None;
        }
        self.slots
            .iter()
            .max_by(|a, b| {
                a.salience
                    .partial_cmp(&b.salience)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|s| s.salience > self.config.similarity_threshold)
            .map(|s| (s.id, s.bound_content.clone()))
    }

    /// Inject diversity when slot salience entropy falls below threshold.
    ///
    /// Adds controlled noise to break deadlock when attention is too
    /// concentrated. Returns the number of slots modified.
    pub fn diversity_inject(&mut self) -> usize {
        let saliences: Vec<f64> = self.slots.iter().map(|s| s.salience).collect();
        let total: f64 = saliences.iter().sum();
        if total <= 0.0 {
            return 0;
        }
        let entropy: f64 = saliences
            .iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| {
                let p = v / total;
                -p * p.log2()
            })
            .sum();
        if entropy >= self.config.diversity_threshold {
            return 0;
        }
        let deficit = (self.config.diversity_threshold - entropy)
            / self.config.diversity_threshold.max(1e-10);
        let noise_strength =
            (deficit * self.config.injection_noise).min(self.config.injection_noise);
        let mut injected = 0;
        for slot in self.slots.iter_mut() {
            let noise = fastrand::f64() * 2.0 * noise_strength - noise_strength;
            let new_sal = (slot.salience + noise).clamp(0.0, 1.0);
            if (new_sal - slot.salience).abs() > 1e-10 {
                injected += 1;
            }
            slot.salience = new_sal;
        }
        injected
    }

    /// Age all slots: decay salience by broadcast_decay factor.
    ///
    /// Slots untouched for more than 10 cycles receive extra decay.
    pub fn tick(&mut self) {
        self.cycle += 1;
        for slot in self.slots.iter_mut() {
            slot.salience *= self.config.broadcast_decay;
            if self.cycle > slot.age + 10 {
                slot.salience *= 0.5;
            }
        }
    }

    /// VSA-native binding attention: bind+bundle instead of dot-product softmax.
    ///
    /// For each proposal, computes `xor_bind(proposal, slot.prototype)` for each
    /// active slot, then bundles all bound results via majority voting. Returns
    /// per-proposal salience measured by similarity between the bundled binding
    /// output and the original proposal — high when the proposal binds coherently
    /// across multiple concept slots.
    ///
    /// This implements the "Attention as Binding" paradigm (AAAI 2026,
    /// arXiv:2512.14709): VSA attention IS a binding operation, not a weighted sum.
    /// Complexity: O(K * VSA_DIM) per proposal = LINEAR, same as `attend`.
    pub fn forward_binding(&mut self, proposals: &[Vec<u8>]) -> Vec<f64> {
        self.total_projected += proposals.len() as u64;
        if proposals.is_empty() || self.slots.is_empty() {
            return vec![];
        }

        proposals
            .iter()
            .map(|proposal| {
                let bound_results: Vec<Vec<u8>> = self
                    .slots
                    .iter()
                    .map(|slot| QuantizedVSA::xor_bind(slot.vector.as_bytes(), proposal))
                    .collect();

                let bound_refs: Vec<&[u8]> = bound_results.iter().map(|v| v.as_slice()).collect();
                let bundle = QuantizedVSA::bundle(&bound_refs);

                QuantizedVSA::similarity(&bundle, proposal)
            })
            .collect()
    }

    /// Get a human-readable summary of the current attention state.
    pub fn state_summary(&self) -> String {
        let n_active = self.slots.iter().filter(|s| s.salience > 0.1).count();
        let max_sal = self
            .slots
            .iter()
            .map(|s| s.salience)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        format!(
            "MANAR[cycle={}, slots={}, active={}, max_sal={:.3}, projected={}]",
            self.cycle,
            self.slots.len(),
            n_active,
            max_sal,
            self.total_projected,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_projection() {
        let config = ManarConfig::default();
        let attn = ManarAttention::new(config);
        let slot0_vec = attn.slots[0].vector.as_bytes().to_vec();
        let projections = attn.project(&slot0_vec);
        // Slot 0 should be the best match for its own vector
        let (top_id, top_sim) = projections
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        assert_eq!(
            *top_id, 0,
            "slot 0 should be most similar to its own vector"
        );
        assert!(
            (top_sim - 1.0).abs() < 1e-6,
            "self-similarity should be 1.0"
        );
    }

    #[test]
    fn test_attend_simple() {
        let config = ManarConfig::default();
        let mut attn = ManarAttention::new(config);
        let match_proposal = attn.slots[0].vector.as_bytes().to_vec();
        let random_proposal = VsaVector::<4096>::random(999).into_inner();
        let proposals = vec![match_proposal, random_proposal];
        let saliences = attn.attend(&proposals);
        assert_eq!(saliences.len(), 2);
        assert!(
            saliences[0] >= 0.99,
            "exact match should have near-1.0 salience"
        );
        assert!(
            saliences[1] < 0.99,
            "random vector should have lower salience"
        );
    }

    #[test]
    fn test_diversity_injection() {
        let config = ManarConfig {
            diversity_threshold: 0.5,
            injection_noise: 0.1,
            ..Default::default()
        };
        let mut attn = ManarAttention::new(config);
        for slot in attn.slots.iter_mut() {
            slot.salience = 0.5;
        }
        let injected = attn.diversity_inject();
        assert!(
            injected > 0,
            "diversity injection should trigger on low entropy"
        );
    }

    #[test]
    fn test_e8_seeding() {
        let config = ManarConfig::default();
        let mut attn = ManarAttention::new(config);
        attn.seed_from_e8();
        assert_eq!(attn.slots.len(), 32);
        for i in 1..attn.slots.len() {
            let sim = QuantizedVSA::similarity(
                attn.slots[0].vector.as_bytes(),
                attn.slots[i].vector.as_bytes(),
            );
            assert!(
                sim < 1.0,
                "slot {} should differ from slot 0 (sim={})",
                i,
                sim
            );
        }
    }

    #[test]
    fn test_memory_retention() {
        let config = ManarConfig::default();
        let mut attn = ManarAttention::new(config);
        let original_content = attn.slots[0].bound_content.clone();
        let proposal = VsaVector::<4096>::random(42).into_inner();
        attn.update_slots(0, &proposal);
        let updated_bits = attn.slots[0].bound_content.as_bytes().to_vec();
        let sim = QuantizedVSA::similarity(original_content.as_bytes(), &updated_bits);
        assert!(sim < 1.0, "bound content should change after update");
        assert!(
            attn.slots[0].salience > 0.0,
            "salience should increase after update"
        );
        assert_eq!(attn.slots[0].age, attn.cycle, "age should track cycle");
    }

    #[test]
    fn test_tick_decay() {
        let config = ManarConfig {
            broadcast_decay: 0.5,
            ..Default::default()
        };
        let mut attn = ManarAttention::new(config);
        attn.slots[0].salience = 1.0;
        attn.tick();
        assert!(
            attn.slots[0].salience < 1.0,
            "salience should decay after tick"
        );
        assert_eq!(attn.cycle, 1, "cycle should increment");
    }

    #[test]
    fn test_state_summary() {
        let config = ManarConfig::default();
        let attn = ManarAttention::new(config);
        let summary = attn.state_summary();
        assert!(
            summary.contains("MANAR["),
            "summary should start with MANAR["
        );
        assert!(summary.contains("slots=32"), "should report 32 slots");
    }
}
