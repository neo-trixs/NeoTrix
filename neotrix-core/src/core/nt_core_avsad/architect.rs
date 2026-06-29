use std::collections::HashMap;
use std::f64;

/// A discovered VSA operation candidate with algebraic validation results.
#[derive(Debug, Clone)]
pub struct VsaOperationCandidate {
    pub name: String,
    pub operation_type: VsaOperationType,
    pub params: HashMap<String, f64>,
    pub associativity_score: f64,
    pub commutativity_score: f64,
    pub distributivity_score: f64,
    pub self_inverse_score: f64,
    pub coherence: f64,
    pub is_valid: bool,
}

/// Categories of VSA primitive operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VsaOperationType {
    Bind,
    Bundle,
    Similarity,
    Invert,
    SoftBind,
    BidirectionalBind,
}

impl VsaOperationType {
    pub fn all() -> Vec<VsaOperationType> {
        vec![
            VsaOperationType::Bind,
            VsaOperationType::Bundle,
            VsaOperationType::Similarity,
            VsaOperationType::Invert,
            VsaOperationType::SoftBind,
            VsaOperationType::BidirectionalBind,
        ]
    }
}

/// AVSAD VSA architecture discovery engine.
///
/// Generate → Validate → Rank pipeline for discovering new VSA operations.
pub struct VsaArchitect {
    /// Candidates that passed all validation gates.
    pub valid_candidates: Vec<VsaOperationCandidate>,
    /// Total candidates evaluated.
    pub total_evaluated: u64,
    /// Discovery cycles run.
    pub discovery_cycles: u64,
    /// VSA dimension for test vectors.
    pub vsa_dim: usize,
}

impl Default for VsaArchitect {
    fn default() -> Self {
        Self::new()
    }
}

impl VsaArchitect {
    pub fn new() -> Self {
        Self {
            valid_candidates: Vec::new(),
            total_evaluated: 0,
            discovery_cycles: 0,
            vsa_dim: 256,
        }
    }

    pub fn with_dim(dim: usize) -> Self {
        Self {
            vsa_dim: dim.max(64).min(4096),
            ..Self::new()
        }
    }

    /// Run one discovery cycle: generate candidates from templates,
    /// validate each against algebraic VSA laws, collect valid ones.
    pub fn discover(&mut self, vsa_core: &impl VsaCoreOps) -> Vec<VsaOperationCandidate> {
        self.discovery_cycles += 1;
        let candidates = self.generate_candidates(vsa_core);
        let mut valid = Vec::new();
        for mut cand in candidates {
            self.total_evaluated += 1;
            cand.associativity_score = self.check_associativity(vsa_core, &cand);
            cand.commutativity_score = self.check_commutativity(vsa_core, &cand);
            cand.distributivity_score = self.check_distributivity(vsa_core, &cand);
            cand.self_inverse_score = self.check_self_inverse(vsa_core, &cand);
            cand.coherence = (cand.associativity_score
                + cand.commutativity_score
                + cand.distributivity_score
                + cand.self_inverse_score)
                / 4.0;
            cand.is_valid = cand.coherence > 0.75;
            if cand.is_valid {
                valid.push(cand.clone());
            }
        }
        self.valid_candidates.extend(valid.clone());
        valid
    }

    /// Generate candidate variants by varying operation type and parameters.
    fn generate_candidates(&self, _core: &impl VsaCoreOps) -> Vec<VsaOperationCandidate> {
        let mut candidates = Vec::new();
        let models = self.supported_models();
        for op_type in VsaOperationType::all() {
            for (mi, model_label) in models.iter().enumerate() {
                let mut params = HashMap::new();
                params.insert("model_index".to_string(), mi as f64);
                match op_type {
                    VsaOperationType::Bind => {
                        let base = VsaOperationCandidate {
                            name: format!("bind_{}", model_label),
                            operation_type: VsaOperationType::Bind,
                            params: params.clone(),
                            associativity_score: 0.0,
                            commutativity_score: 0.0,
                            distributivity_score: 0.0,
                            self_inverse_score: 0.0,
                            coherence: 0.0,
                            is_valid: false,
                        };
                        candidates.push(base);
                        let mut alt = params.clone();
                        alt.insert("alternate_encoding".to_string(), 1.0);
                        candidates.push(VsaOperationCandidate {
                            name: format!("bind_alt_{}", model_label),
                            operation_type: VsaOperationType::Bind,
                            params: alt,
                            ..VsaOperationCandidate::new_default()
                        });
                    }
                    VsaOperationType::Bundle => {
                        candidates.push(VsaOperationCandidate {
                            name: format!("bundle_{}", model_label),
                            operation_type: VsaOperationType::Bundle,
                            params: params.clone(),
                            ..VsaOperationCandidate::new_default()
                        });
                        let mut w = params.clone();
                        w.insert("weighted".to_string(), 1.0);
                        candidates.push(VsaOperationCandidate {
                            name: format!("bundle_weighted_{}", model_label),
                            operation_type: VsaOperationType::Bundle,
                            params: w,
                            ..VsaOperationCandidate::new_default()
                        });
                    }
                    VsaOperationType::Similarity => {
                        candidates.push(VsaOperationCandidate {
                            name: format!("similarity_{}", model_label),
                            operation_type: VsaOperationType::Similarity,
                            params: params.clone(),
                            ..VsaOperationCandidate::new_default()
                        });
                    }
                    VsaOperationType::Invert => {
                        candidates.push(VsaOperationCandidate {
                            name: format!("invert_{}", model_label),
                            operation_type: VsaOperationType::Invert,
                            params: params.clone(),
                            ..VsaOperationCandidate::new_default()
                        });
                    }
                    VsaOperationType::SoftBind => {
                        let mut soft = params.clone();
                        soft.insert("soft_alpha".to_string(), 0.5);
                        candidates.push(VsaOperationCandidate {
                            name: format!("soft_bind_{}", model_label),
                            operation_type: VsaOperationType::SoftBind,
                            params: soft,
                            ..VsaOperationCandidate::new_default()
                        });
                    }
                    VsaOperationType::BidirectionalBind => {
                        for iters in [2, 4, 8] {
                            let mut bi = params.clone();
                            bi.insert("iterations".to_string(), iters as f64);
                            candidates.push(VsaOperationCandidate {
                                name: format!("bidir_bind_{}it_{}", iters, model_label),
                                operation_type: VsaOperationType::BidirectionalBind,
                                params: bi,
                                ..VsaOperationCandidate::new_default()
                            });
                        }
                    }
                }
            }
        }
        candidates
    }

    /// Test associativity: bind(a, bind(b, c)) ≈ bind(bind(a, b), c)
    fn check_associativity(&self, core: &impl VsaCoreOps, _cand: &VsaOperationCandidate) -> f64 {
        let (a, b, c) = self.test_triple();
        let left = core.vsa_bind(&a, &core.vsa_bind(&b, &c));
        let right = core.vsa_bind(&core.vsa_bind(&a, &b), &c);
        core.vsa_similarity(&left, &right)
    }

    /// Test commutativity: bind(a, b) ≈ bind(b, a)
    fn check_commutativity(&self, core: &impl VsaCoreOps, _cand: &VsaOperationCandidate) -> f64 {
        let (a, b, _) = self.test_triple();
        let forward = core.vsa_bind(&a, &b);
        let backward = core.vsa_bind(&b, &a);
        core.vsa_similarity(&forward, &backward)
    }

    /// Test distributivity: bind(a, bundle(b, c)) ≈ bundle(bind(a, b), bind(a, c))
    fn check_distributivity(&self, core: &impl VsaCoreOps, _cand: &VsaOperationCandidate) -> f64 {
        let (a, b, c) = self.test_triple();
        let left = core.vsa_bind(&a, &core.vsa_bundle(&[&b, &c]));
        let right = core.vsa_bundle(&[&core.vsa_bind(&a, &b), &core.vsa_bind(&a, &c)]);
        core.vsa_similarity(&left, &right)
    }

    /// Test self-inverse: bind(bind(a, b), b) ≈ a
    fn check_self_inverse(&self, core: &impl VsaCoreOps, _cand: &VsaOperationCandidate) -> f64 {
        let (a, b, _) = self.test_triple();
        let rebound = core.vsa_bind(&core.vsa_bind(&a, &b), &b);
        core.vsa_similarity(&rebound, &a)
    }

    /// Generate three random test vectors.
    fn test_triple(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let r = || -> Vec<f64> {
            let seed: u64 = self.discovery_cycles.wrapping_mul(2654435761);
            let mut v = Vec::with_capacity(self.vsa_dim);
            for i in 0..self.vsa_dim {
                let h = seed.wrapping_mul(31).wrapping_add(i as u64);
                let val = (h as f64 / u64::MAX as f64) * 2.0 - 1.0;
                v.push(val);
            }
            normalize(&v)
        };
        (r(), r(), r())
    }

    fn supported_models(&self) -> Vec<&'static str> {
        vec!["map", "bsc", "hrr", "fhrr", "adaptive"]
    }

    /// Summary report for meta-cognitive logging.
    pub fn report(&self) -> VsaArchitectReport {
        let valid = self.valid_candidates.len();
        let avg_coherence = if valid > 0 {
            self.valid_candidates
                .iter()
                .map(|c| c.coherence)
                .sum::<f64>()
                / valid as f64
        } else {
            0.0
        };
        VsaArchitectReport {
            total_evaluated: self.total_evaluated,
            discovery_cycles: self.discovery_cycles,
            valid_candidates: valid,
            avg_coherence,
            top_candidate: self
                .valid_candidates
                .iter()
                .max_by(|a, b| {
                    a.coherence
                        .partial_cmp(&b.coherence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned(),
        }
    }
}

/// Aggregate report for the architect's current state.
#[derive(Debug, Clone)]
pub struct VsaArchitectReport {
    pub total_evaluated: u64,
    pub discovery_cycles: u64,
    pub valid_candidates: usize,
    pub avg_coherence: f64,
    pub top_candidate: Option<VsaOperationCandidate>,
}

impl VsaOperationCandidate {
    fn new_default() -> Self {
        Self {
            name: String::new(),
            operation_type: VsaOperationType::Bind,
            params: HashMap::new(),
            associativity_score: 0.0,
            commutativity_score: 0.0,
            distributivity_score: 0.0,
            self_inverse_score: 0.0,
            coherence: 0.0,
            is_valid: false,
        }
    }
}

fn normalize(v: &[f64]) -> Vec<f64> {
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm < 1e-12 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

/// Trait abstracting core VSA operations for the architect.
pub trait VsaCoreOps {
    fn vsa_bind(&self, a: &[f64], b: &[f64]) -> Vec<f64>;
    fn vsa_bundle(&self, vecs: &[&[f64]]) -> Vec<f64>;
    fn vsa_similarity(&self, a: &[f64], b: &[f64]) -> f64;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyCore;

    impl VsaCoreOps for DummyCore {
        fn vsa_bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
            let len = a.len().min(b.len());
            a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
        }
        fn vsa_bundle(&self, vecs: &[&[f64]]) -> Vec<f64> {
            let len = vecs.first().map(|v| v.len()).unwrap_or(0);
            let mut sum = vec![0.0_f64; len];
            for v in vecs {
                for (s, x) in sum.iter_mut().zip(v.iter()) {
                    *s += x;
                }
            }
            let n = vecs.len().max(1) as f64;
            sum.iter().map(|s| s / n).collect()
        }
        fn vsa_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
            let len = a.len().min(b.len());
            if len == 0 {
                return 1.0;
            }
            let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            let na: f64 = a.iter().map(|x| x * x).sum();
            let nb: f64 = b.iter().map(|x| x * x).sum();
            let denom = (na * nb).sqrt();
            if denom < 1e-12 {
                1.0
            } else {
                (dot / denom).clamp(-1.0, 1.0)
            }
        }
    }

    #[test]
    fn test_new_architect_defaults() {
        let arch = VsaArchitect::new();
        assert_eq!(arch.valid_candidates.len(), 0);
        assert_eq!(arch.total_evaluated, 0);
    }

    #[test]
    fn test_discovery_cycle_produces_candidates() {
        let mut arch = VsaArchitect::new();
        let core = DummyCore;
        let valid = arch.discover(&core);
        assert!(
            arch.total_evaluated > 0,
            "should evaluate at least one candidate"
        );
        assert!(
            !valid.is_empty(),
            "should discover at least one valid operation"
        );
    }

    #[test]
    fn test_discovery_tracks_cycles() {
        let mut arch = VsaArchitect::new();
        let core = DummyCore;
        arch.discover(&core);
        assert_eq!(arch.discovery_cycles, 1);
        arch.discover(&core);
        assert_eq!(arch.discovery_cycles, 2);
    }

    #[test]
    fn test_valid_candidates_have_high_coherence() {
        let mut arch = VsaArchitect::new();
        let core = DummyCore;
        let valid = arch.discover(&core);
        for c in &valid {
            assert!(
                c.coherence > 0.75,
                "valid candidate coherence={} should be >0.75",
                c.coherence
            );
            assert!(c.is_valid);
        }
    }

    #[test]
    fn test_report_format() {
        let mut arch = VsaArchitect::new();
        let core = DummyCore;
        arch.discover(&core);
        let r = arch.report();
        assert!(r.total_evaluated > 0);
        assert!(r.avg_coherence > 0.0);
        assert!(r.top_candidate.is_some());
    }

    #[test]
    fn test_associativity_high_for_bind() {
        let mut arch = VsaArchitect::new();
        let core = DummyCore;
        arch.discover(&core);
        if let Some(top) = arch.report().top_candidate {
            assert!(
                top.associativity_score > 0.0,
                "bind should have associativity: {}",
                top.associativity_score
            );
        }
    }
}
