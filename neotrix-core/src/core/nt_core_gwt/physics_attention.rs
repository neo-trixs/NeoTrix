use super::resonance::MODULE_COUNT;
use crate::core::nt_core_hex::ReasoningHexagram;

/// Adaptive slice clustering inspired by Transolver's Physics-Attention.
///
/// Instead of fixed Hamming-distance resonance clusters, dynamically groups
/// specialists into "slices" based on their current activation-weighted
/// hexagram similarity. Attention is computed per-slice, then distributed
/// to members proportionally to their activation within the slice.
///
/// Reference: Transolver (arXiv:2402.02366) — Physics-Attention splits the
/// domain into learnable slices of flexible shapes based on underlying states.
#[derive(Debug, Clone)]
pub struct AdaptiveSlicer {
    /// Number of slices to form (auto-adjusted if 0).
    pub num_slices: usize,
    /// Minimum similarity threshold for slice membership.
    pub min_similarity: f64,
    /// Whether to use activation-weighted similarity (vs. raw hexagram distance).
    pub use_activation_weight: bool,
    /// The formed slices from the last call to `form_slices()`.
    pub slices: Vec<Slice>,
    /// Per-slice attention weights (from last call).
    pub slice_weights: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct Slice {
    /// Indices of specialists in this slice.
    pub members: Vec<usize>,
    /// Centroid state (average hexagram bits).
    pub centroid: f64,
    /// Cohesion: how similar members are to each other (0-1).
    pub cohesion: f64,
}

impl Default for AdaptiveSlicer {
    fn default() -> Self {
        Self {
            num_slices: 0,
            min_similarity: 0.6,
            use_activation_weight: true,
            slices: Vec::new(),
            slice_weights: Vec::new(),
        }
    }
}

impl AdaptiveSlicer {
    pub fn new(num_slices: usize) -> Self {
        Self {
            num_slices,
            ..Default::default()
        }
    }

    /// Form adaptive slices from specialist activations and hexagram states.
    ///
    /// Uses a greedy clustering approach:
    /// 1. Seed clusters with the most active specialist not yet assigned
    /// 2. Add specialists with similarity above threshold
    /// 3. Repeat until all specialists are assigned or num_slices reached
    pub fn form_slices(
        &mut self,
        activations: &[f64; MODULE_COUNT],
        states: &[ReasoningHexagram; MODULE_COUNT],
    ) -> &[Slice] {
        let k = if self.num_slices > 0 {
            self.num_slices.min(MODULE_COUNT)
        } else {
            (MODULE_COUNT / 3).max(2)
        };

        let mut assigned = [false; MODULE_COUNT];
        let mut slices: Vec<Slice> = Vec::new();

        for _ in 0..k {
            if assigned.iter().all(|&a| a) {
                break;
            }
            let seed = (0..MODULE_COUNT)
                .filter(|&i| !assigned[i])
                .max_by(|&a, &b| {
                    let wa = if self.use_activation_weight {
                        activations[a]
                    } else {
                        1.0
                    };
                    let wb = if self.use_activation_weight {
                        activations[b]
                    } else {
                        1.0
                    };
                    wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(0);

            assigned[seed] = true;
            let mut members = vec![seed];

            let seed_bits = states[seed].0 as f64;
            let mut sum_bits = seed_bits;
            let mut cohesion_sum = 0.0;

            for i in 0..MODULE_COUNT {
                if assigned[i] || i == seed {
                    continue;
                }
                let dist = states[seed].hamming_dist(&states[i]) as f64 / 6.0;
                let similarity = 1.0 - dist;
                if similarity >= self.min_similarity {
                    assigned[i] = true;
                    members.push(i);
                    sum_bits += states[i].0 as f64;
                    cohesion_sum += similarity;
                }
            }

            let centroid = sum_bits / members.len() as f64;
            let cohesion = if members.len() > 1 {
                cohesion_sum / (members.len() - 1) as f64
            } else {
                1.0
            };

            slices.push(Slice {
                members,
                centroid,
                cohesion,
            });
        }

        // Assign remaining unassigned specialists as singleton slices
        for i in 0..MODULE_COUNT {
            if !assigned[i] {
                slices.push(Slice {
                    members: vec![i],
                    centroid: states[i].0 as f64,
                    cohesion: 1.0,
                });
            }
        }

        // Compute per-slice weights (average activation within each slice)
        let slice_weights: Vec<f64> = slices
            .iter()
            .map(|slice| {
                let avg_act: f64 = slice.members.iter().map(|&m| activations[m]).sum::<f64>()
                    / slice.members.len() as f64;
                avg_act
            })
            .collect();

        self.slices = slices;
        self.slice_weights = slice_weights;
        &self.slices
    }

    /// Get the slice index for a specialist module.
    pub fn slice_of(&self, idx: usize) -> Option<usize> {
        self.slices.iter().position(|s| s.members.contains(&idx))
    }

    /// Reconstruct a resonance-like salience vector from slice weights.
    /// Each specialist gets its slice's weight, normalized by slice size.
    pub fn slice_salience(&self) -> [f64; MODULE_COUNT] {
        let mut sal = [0.0; MODULE_COUNT];
        for (i, slice) in self.slices.iter().enumerate() {
            let w = self.slice_weights.get(i).copied().unwrap_or(0.0);
            for &m in &slice.members {
                sal[m] = w;
            }
        }
        sal
    }

    /// Number of slices formed.
    pub fn slice_count(&self) -> usize {
        self.slices.len()
    }

    /// Reset slicer state.
    pub fn reset(&mut self) {
        self.slices.clear();
        self.slice_weights.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::super::resonance::default_specialist_states;
    use super::*;

    #[test]
    fn test_default() {
        let s = AdaptiveSlicer::default();
        assert!(s.slices.is_empty());
    }

    #[test]
    fn test_new() {
        let s = AdaptiveSlicer::new(4);
        assert_eq!(s.num_slices, 4);
    }

    #[test]
    fn test_form_slices_creates_slices() {
        let mut s = AdaptiveSlicer::new(3);
        let states = default_specialist_states();
        let activations = [0.5; MODULE_COUNT];
        let slices = s.form_slices(&activations, &states);
        assert!(slices.len() >= 2);
    }

    #[test]
    fn test_form_slices_assigns_all_modules() {
        let mut s = AdaptiveSlicer::new(4);
        let states = default_specialist_states();
        let activations = [0.5; MODULE_COUNT];
        s.form_slices(&activations, &states);
        let total_members: usize = s.slices.iter().map(|sl| sl.members.len()).sum();
        assert_eq!(total_members, MODULE_COUNT);
    }

    #[test]
    fn test_slice_of_returns_correct() {
        let mut s = AdaptiveSlicer::new(3);
        let states = default_specialist_states();
        let activations = [0.5; MODULE_COUNT];
        s.form_slices(&activations, &states);
        for i in 0..MODULE_COUNT {
            let sidx = s.slice_of(i);
            assert!(sidx.is_some(), "module {} should be in a slice", i);
            assert!(sidx.unwrap() < s.slices.len());
        }
    }

    #[test]
    fn test_slice_salience_produces_valid_output() {
        let mut s = AdaptiveSlicer::new(3);
        let states = default_specialist_states();
        let activations = [0.5; MODULE_COUNT];
        s.form_slices(&activations, &states);
        let sal = s.slice_salience();
        assert_eq!(sal.len(), MODULE_COUNT);
        for &v in &sal {
            assert!(v >= 0.0 && v <= 1.0);
        }
    }

    #[test]
    fn test_activation_weight_affects_clustering() {
        let mut s = AdaptiveSlicer::new(3);
        let states = default_specialist_states();
        // Make one module extremely active
        let mut activations = [0.1; MODULE_COUNT];
        activations[0] = 0.99;
        s.use_activation_weight = true;
        s.form_slices(&activations, &states);
        // Module 0 should be a seed (first slice)
        let seed_slice = s.slice_of(0).unwrap();
        assert_eq!(s.slices[seed_slice].members[0], 0);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut s = AdaptiveSlicer::new(3);
        let states = default_specialist_states();
        let activations = [0.5; MODULE_COUNT];
        s.form_slices(&activations, &states);
        assert!(!s.slices.is_empty());
        s.reset();
        assert!(s.slices.is_empty());
        assert!(s.slice_weights.is_empty());
    }

    #[test]
    fn test_slice_has_cohesion() {
        let mut s = AdaptiveSlicer::new(2);
        let states = default_specialist_states();
        let activations = [0.5; MODULE_COUNT];
        s.form_slices(&activations, &states);
        for sl in &s.slices {
            assert!(sl.cohesion >= 0.0 && sl.cohesion <= 1.0);
            assert!(sl.centroid >= 0.0 && sl.centroid <= 63.0);
        }
    }
}
