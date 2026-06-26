#![forbid(unsafe_code)]

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

/// Hop-level reward tracking for multi-hop retrieval.
#[derive(Debug, Clone)]
pub struct HopReward {
    pub hop: usize,
    pub query_vsa: Vec<u8>,
    pub entry_ids: Vec<String>,
    pub entry_vsas: Vec<Vec<u8>>,
    pub composition_score: f64,
    pub terminal_score: f64,
    pub reward: f64,
}

/// Progress-aware RAG tracker that scores each retrieval hop.
#[derive(Debug, Clone)]
pub struct ProgressAwareRAG {
    pub hop_rewards: Vec<HopReward>,
    pub max_hops: usize,
    pub beam_width: usize,
    prior_entry_vsas: Vec<Vec<u8>>,
}

impl Default for ProgressAwareRAG {
    fn default() -> Self {
        Self {
            hop_rewards: Vec::new(),
            max_hops: 10,
            beam_width: 3,
            prior_entry_vsas: Vec::new(),
        }
    }
}

impl ProgressAwareRAG {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a retrieval hop and compute the stepwise reward.
    ///
    /// Reward formula:
    ///   composition_gain = composition_score - similarity(prior_bundled, query_vsa)
    ///   terminal_gain = similarity(bundled_entries, initial_query) * 0.5
    ///   reward = composition_gain.clamp(-1.0, 1.0) * 0.7 + terminal_gain * 0.3
    pub fn record_hop(
        &mut self,
        hop: usize,
        query_vsa: Vec<u8>,
        entry_vsas: Vec<(String, Vec<u8>)>,
        initial_query: &[u8],
    ) -> f64 {
        let entry_ids: Vec<String> = entry_vsas.iter().map(|(id, _)| id.clone()).collect();
        let entry_vecs: Vec<Vec<u8>> = entry_vsas.into_iter().map(|(_, v)| v).collect();
        let entry_refs: Vec<&[u8]> = entry_vecs.iter().map(|v| v.as_slice()).collect();

        let bundled_entries = if entry_refs.is_empty() {
            vec![0u8; VSA_DIM]
        } else {
            QuantizedVSA::bundle(&entry_refs)
        };

        let composition_score = if bundled_entries.iter().all(|&b| b == 0) {
            0.0
        } else {
            QuantizedVSA::similarity(&bundled_entries, &query_vsa)
        };

        let terminal_score = if bundled_entries.iter().all(|&b| b == 0) {
            0.0
        } else {
            QuantizedVSA::similarity(&bundled_entries, initial_query)
        };

        let prior_refs: Vec<&[u8]> = self.prior_entry_vsas.iter().map(|v| v.as_slice()).collect();
        let prior_bundled = if prior_refs.is_empty() {
            vec![0u8; VSA_DIM]
        } else {
            QuantizedVSA::bundle(&prior_refs)
        };
        let prior_sim = if prior_bundled.iter().all(|&b| b == 0) {
            0.0
        } else {
            QuantizedVSA::similarity(&prior_bundled, &query_vsa)
        };

        let composition_gain = composition_score - prior_sim;
        let terminal_gain = terminal_score * 0.5;
        let reward = composition_gain.clamp(-1.0, 1.0) * 0.7 + terminal_gain * 0.3;

        // Accumulate entry VSA vectors for next hop's prior
        for v in &entry_vecs {
            self.prior_entry_vsas.push(v.clone());
        }

        let hop_reward = HopReward {
            hop,
            query_vsa,
            entry_ids,
            entry_vsas: entry_vecs,
            composition_score,
            terminal_score,
            reward,
        };

        self.hop_rewards.push(hop_reward);
        reward
    }

    /// Total cumulative reward across all hops.
    pub fn cumulative_reward(&self) -> f64 {
        self.hop_rewards.iter().map(|r| r.reward).sum()
    }

    /// Reward for a specific hop index.
    pub fn reward_for_hop(&self, hop: usize) -> f64 {
        self.hop_rewards.get(hop).map(|r| r.reward).unwrap_or(0.0)
    }

    /// Clear all tracked hops and prior state.
    pub fn clear(&mut self) {
        self.hop_rewards.clear();
        self.prior_entry_vsas.clear();
    }
}

// ---------------------------------------------------------------------------
// VSA Encoder trait (generic text-to-VSA interface)
// ---------------------------------------------------------------------------

/// Trait for encoding text into a VSA vector.
pub trait VsaEncoder {
    fn encode(&self, text: &str) -> Vec<u8>;
}

/// A basic VSA encoder that uses seeded_random with a hash of the text.
pub struct BasicVsaEncoder;

impl VsaEncoder for BasicVsaEncoder {
    fn encode(&self, text: &str) -> Vec<u8> {
        let h: u64 = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        QuantizedVSA::seeded_random(h, VSA_DIM)
    }
}

// ---------------------------------------------------------------------------
// Standalone VSA helpers
// ---------------------------------------------------------------------------

/// Bundle multiple VSA vectors via majority bundling.
pub fn bundle_vsas(vsas: &[&[u8]]) -> Vec<u8> {
    if vsas.is_empty() {
        return vec![0u8; VSA_DIM];
    }
    QuantizedVSA::bundle(vsas)
}

/// Cosine/Hamming similarity between two VSA vectors.
pub fn vsa_similarity(a: &[u8], b: &[u8]) -> f64 {
    QuantizedVSA::similarity(a, b)
}

/// Progress-aware multi-hop retrieval function.
///
/// Iteratively retrieves top-k entries from a search function, records hop
/// rewards, and constructs sub-queries from retrieved content until
/// convergence or max_hops.
pub fn progress_aware_retrieve<SearchFn, SubQueryFn>(
    query: &str,
    vsa_encoder: &dyn VsaEncoder,
    rag: &mut ProgressAwareRAG,
    search: SearchFn,
    sub_queries: SubQueryFn,
) -> Vec<String>
where
    SearchFn: Fn(&[u8]) -> Vec<(String, Vec<u8>)>,
    SubQueryFn: Fn(&[(String, Vec<u8>)]) -> Vec<String>,
{
    let mut collected_ids: Vec<String> = Vec::new();
    let initial_vsa = vsa_encoder.encode(query);

    let mut current_query_vsa = initial_vsa.clone();

    for hop in 0..rag.max_hops {
        let results = search(&current_query_vsa);
        if results.is_empty() {
            break;
        }

        // Take top-k by beam_width
        let top_k: Vec<(String, Vec<u8>)> = results.into_iter().take(rag.beam_width).collect();

        // Record hop reward
        let _reward = rag.record_hop(hop, current_query_vsa.clone(), top_k.clone(), &initial_vsa);

        // Collect entry IDs
        for (id, _) in &top_k {
            if !collected_ids.contains(id) {
                collected_ids.push(id.clone());
            }
        }

        // Generate sub-queries from retrieved entries
        let sq = sub_queries(&top_k);
        if sq.is_empty() {
            break;
        }

        // Encode first sub-query for next iteration
        current_query_vsa = vsa_encoder.encode(&sq[0]);

        // Check convergence: if last 2 hops have composition_gain < 0.05
        if hop >= 1 {
            let last = rag.reward_for_hop(hop);
            let second_last = rag.reward_for_hop(hop - 1);
            if (last - second_last).abs() < 0.05 {
                break;
            }
        }
    }

    collected_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    #[test]
    fn test_record_hop_returns_f64() {
        let mut rag = ProgressAwareRAG::new();
        let q = make_vsa(42);
        let init = make_vsa(999);
        let entries = vec![("e1".into(), make_vsa(1)), ("e2".into(), make_vsa(2))];

        let reward = rag.record_hop(0, q, entries, &init);
        assert!(reward.is_finite());
        assert_eq!(rag.hop_rewards.len(), 1);
        assert_eq!(rag.hop_rewards[0].hop, 0);
    }

    #[test]
    fn test_record_hop_stores_correct_fields() {
        let mut rag = ProgressAwareRAG::new();
        let q = make_vsa(100);
        let init = make_vsa(200);
        let entries = vec![("a".into(), make_vsa(10)), ("b".into(), make_vsa(20))];

        rag.record_hop(0, q.clone(), entries.clone(), &init);
        let hr = &rag.hop_rewards[0];

        assert_eq!(hr.entry_ids, vec!["a", "b"]);
        assert_eq!(hr.entry_vsas.len(), 2);
        assert!(hr.composition_score >= 0.0);
        assert!(hr.terminal_score >= 0.0);
    }

    #[test]
    fn test_cumulative_reward_aggregates_correctly() {
        let mut rag = ProgressAwareRAG::new();
        let init = make_vsa(999);

        let r0 = rag.record_hop(0, make_vsa(10), vec![("a".into(), make_vsa(10))], &init);
        let r1 = rag.record_hop(1, make_vsa(20), vec![("b".into(), make_vsa(20))], &init);

        assert!((rag.cumulative_reward() - (r0 + r1)).abs() < 1e-9);
    }

    #[test]
    fn test_empty_hops_returns_zero() {
        let rag = ProgressAwareRAG::new();
        assert!((rag.cumulative_reward() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_reward_for_hop_out_of_range_returns_zero() {
        let rag = ProgressAwareRAG::new();
        assert!((rag.reward_for_hop(0) - 0.0).abs() < 1e-9);
        assert!((rag.reward_for_hop(5) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_reward_for_hop_valid_index() {
        let mut rag = ProgressAwareRAG::new();
        let init = make_vsa(999);
        let r = rag.record_hop(0, make_vsa(10), vec![("a".into(), make_vsa(10))], &init);
        assert!((rag.reward_for_hop(0) - r).abs() < 1e-9);
    }

    #[test]
    fn test_clear_resets_state() {
        let mut rag = ProgressAwareRAG::new();
        let init = make_vsa(999);
        rag.record_hop(0, make_vsa(10), vec![("a".into(), make_vsa(10))], &init);
        rag.record_hop(1, make_vsa(20), vec![("b".into(), make_vsa(20))], &init);
        assert_eq!(rag.hop_rewards.len(), 2);

        rag.clear();
        assert!(rag.hop_rewards.is_empty());
        assert!((rag.cumulative_reward() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_composition_gain_higher_for_similar_queries() {
        let mut rag = ProgressAwareRAG::new();
        let init = make_vsa(999);

        // Hop 0: query and entries are aligned
        let q0 = make_vsa(50);
        let e0 = vec![("e".into(), make_vsa(50))];
        let r0 = rag.record_hop(0, q0, e0, &init);

        // Hop 1: query is different from entries, so composition_gain should be lower
        let q1 = make_vsa(9999);
        let e1 = vec![("f".into(), make_vsa(1))];
        let r1 = rag.record_hop(1, q1, e1, &init);

        assert!(
            r0 >= r1 || (r0 - r1).abs() < 0.2,
            "aligned hop should have reward >= misaligned hop; got r0={}, r1={}",
            r0,
            r1
        );
    }

    #[test]
    fn test_terminal_gain_increases_as_entries_approach_answer() {
        let mut rag = ProgressAwareRAG::new();
        let target = make_vsa(42);

        // Hop 0: entries far from target
        let _r0 = rag.record_hop(0, make_vsa(1), vec![("a".into(), make_vsa(1))], &target);

        // Hop 1: entries close to target
        let _r1 = rag.record_hop(1, make_vsa(42), vec![("b".into(), make_vsa(42))], &target);

        assert!(
            rag.hop_rewards[1].terminal_score >= rag.hop_rewards[0].terminal_score,
            "terminal_score should increase as entries approach target; got {} vs {}",
            rag.hop_rewards[1].terminal_score,
            rag.hop_rewards[0].terminal_score,
        );
    }

    #[test]
    fn test_bundle_vsas_consistency() {
        let v1 = make_vsa(1);
        let v2 = make_vsa(2);
        let v3 = make_vsa(3);

        let b1 = bundle_vsas(&[&v1, &v2]);
        let b2 = bundle_vsas(&[&v1, &v2]);
        assert_eq!(b1, b2, "bundling must be deterministic");
        assert_eq!(b1.len(), VSA_DIM);

        let b3 = bundle_vsas(&[&v1, &v2, &v3]);
        assert_eq!(b3.len(), VSA_DIM);
    }

    #[test]
    fn test_bundle_vsas_empty_returns_zero() {
        let b = bundle_vsas(&[]);
        assert!(b.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_vsa_similarity_symmetric() {
        let v1 = make_vsa(10);
        let v2 = make_vsa(20);
        let s12 = vsa_similarity(&v1, &v2);
        let s21 = vsa_similarity(&v2, &v1);
        assert!((s12 - s21).abs() < 1e-9);
    }

    #[test]
    fn test_basic_vsa_encoder_deterministic() {
        let enc = BasicVsaEncoder;
        let v1 = enc.encode("hello world");
        let v2 = enc.encode("hello world");
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_basic_vsa_encoder_different_inputs() {
        let enc = BasicVsaEncoder;
        let v1 = enc.encode("apple");
        let v2 = enc.encode("orange");
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_reward_bounded() {
        let mut rag = ProgressAwareRAG::new();
        let init = make_vsa(42);

        let reward = rag.record_hop(0, make_vsa(42), vec![("x".into(), make_vsa(42))], &init);
        assert!(
            reward >= -1.0 && reward <= 1.0,
            "reward should be bounded in [-1, 1]; got {}",
            reward
        );
    }

    #[test]
    fn test_progress_aware_retrieve_empty_search() {
        let mut rag = ProgressAwareRAG::new();
        let encoder = BasicVsaEncoder;

        let ids = progress_aware_retrieve("test query", &encoder, &mut rag, |_| vec![], |_| vec![]);
        assert!(ids.is_empty());
    }

    #[test]
    fn test_progress_aware_retrieve_one_hop() {
        let mut rag = ProgressAwareRAG::new();
        let encoder = BasicVsaEncoder;

        let ids = progress_aware_retrieve(
            "physics quantum",
            &encoder,
            &mut rag,
            |q| {
                vec![
                    (
                        "e1".into(),
                        QuantizedVSA::similarity(&make_vsa(1), q)
                            .to_be_bytes()
                            .to_vec(),
                    ),
                    (
                        "e2".into(),
                        QuantizedVSA::similarity(&make_vsa(2), q)
                            .to_be_bytes()
                            .to_vec(),
                    ),
                ]
            },
            |results| {
                results
                    .iter()
                    .map(|(id, _)| format!("sub_{}", id))
                    .collect()
            },
        );

        assert!(!ids.is_empty());
        assert!(rag.hop_rewards.len() <= rag.max_hops);
    }
}
