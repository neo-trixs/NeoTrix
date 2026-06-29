//! # SpectrumSignal — Diversity Generation + Signal Amplification Pipeline (G396)
//!
//! Implements the SPECTRUM→SIGNAL dual-phase inference pattern:
//! - SPECTRUM: Generate N diverse candidate interpretations (exploration)
//! - SIGNAL: Amplify high-quality candidates via resonance/evaluation (exploitation)
//!
//! Before this module, NeoTrix used single-pass reasoning — one inference, one output.
//! This pipeline introduces the fundamental architectural principle of
//! "explore diversity first, then converge via signal amplification."
//!
//! Inspired by VibeThinker-3B's CLR (Consistent Latent Reasoning) diversity
//! and BRDFusion's dual-path constraint+generative architecture.

/// Phase of the SPECTRUM→SIGNAL pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelinePhase {
    Spectrum, // Diversity generation
    Signal,   // Quality amplification
    Merged,   // Combined output
}

/// A single candidate in the spectrum.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: usize,
    pub label: String,
    pub vector: Vec<u8>,
    pub diversity_score: f64,
    pub quality_score: f64,
    pub confidence: f64,
}

impl Candidate {
    pub fn new(id: usize, label: String, vector: Vec<u8>) -> Self {
        Self {
            id,
            label,
            vector,
            diversity_score: 0.0,
            quality_score: 0.0,
            confidence: 0.5,
        }
    }
}

/// Configuration for the spectrum→signal pipeline.
#[derive(Debug, Clone)]
pub struct SpectrumConfig {
    /// Number of diverse candidates to generate (Spectrum phase)
    pub n_candidates: usize,
    /// Similarity threshold for diversity (0.0 = identical, 1.0 = orthogonal)
    pub diversity_threshold: f64,
    /// Quality threshold for signal amplification
    pub quality_threshold: f64,
    /// Whether to use pairwise diversity filtering
    pub pairwise_diversity: bool,
    /// Maximum refinement rounds during signal phase
    pub signal_rounds: usize,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            n_candidates: 5,
            diversity_threshold: 0.6,
            quality_threshold: 0.5,
            pairwise_diversity: true,
            signal_rounds: 2,
        }
    }
}

/// # SpectrumSignal
///
/// The diversity→signal inference pipeline. Generates N diverse candidates
/// (SPECTRUM), then evaluates and amplifies the best ones (SIGNAL), and
/// finally merges into a single output.
///
/// ## Architecture-Level Insight
///
/// Before this module, every reasoning step was a single deterministic pass.
/// This creates a blind spot: the system can't explore alternative
/// interpretations before committing to one. The SpectrumSignal pipeline
/// adds the fundamental "first explore, then exploit" architectural pattern.
#[derive(Debug)]
pub struct SpectrumSignal {
    config: SpectrumConfig,
    candidates: Vec<Candidate>,
    phase: PipelinePhase,
}

impl SpectrumSignal {
    pub fn new(config: SpectrumConfig) -> Self {
        Self {
            config,
            candidates: Vec::new(),
            phase: PipelinePhase::Spectrum,
        }
    }

    pub fn config(&self) -> &SpectrumConfig {
        &self.config
    }
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }
    pub fn phase(&self) -> PipelinePhase {
        self.phase
    }

    /// SPECTRUM phase: Generate N diverse candidate interpretations.
    ///
    /// Takes a base VSA vector and generates `n_candidates` diverse variants
    /// by mutating along orthogonal hypervector dimensions.
    pub fn generate_spectrum(&mut self, base: &[u8], task_context: &str) -> &[Candidate] {
        self.phase = PipelinePhase::Spectrum;
        self.candidates.clear();

        for i in 0..self.config.n_candidates {
            // Generate a diverse candidate by injecting variability
            // proportional to the diversity threshold
            let mut vec = base.to_vec();
            let offset = (i * 7) % 256;
            for j in 0..vec.len().min(32) {
                let variation = (offset.wrapping_add(j * 13) % 256) as u8;
                let strength = (self.config.diversity_threshold * 255.0) as u8;
                vec[j] = vec[j].wrapping_add(variation.wrapping_mul(strength) / 255);
            }

            let mut candidate = Candidate::new(i, format!("candidate_{}", i), vec);
            candidate.diversity_score = self.config.diversity_threshold
                * (1.0 - (i as f64 / self.config.n_candidates as f64) * 0.3);

            // Label encodes the task context with candidate-specific nuance
            candidate.label = format!("{}_{}", task_context, i);

            self.candidates.push(candidate);
        }

        // Pairwise diversity filtering if enabled
        if self.config.pairwise_diversity && self.candidates.len() > 1 {
            self.filter_by_pairwise_diversity();
        }

        &self.candidates
    }

    /// SIGNAL phase: Evaluate and amplify the best candidates.
    ///
    /// Scores each candidate on quality, keeps those above threshold,
    /// and runs `signal_rounds` refinement iterations.
    pub fn amplify_signal(&mut self) -> Vec<&Candidate> {
        self.phase = PipelinePhase::Signal;

        // Score each candidate
        let scores: Vec<f64> = self
            .candidates
            .iter()
            .map(|c| evaluate_quality(c))
            .collect();
        for (c, score) in self.candidates.iter_mut().zip(scores) {
            c.quality_score = score;
        }

        // Filter by quality threshold
        let threshold = self.config.quality_threshold;
        self.candidates.retain(|c| c.quality_score >= threshold);

        // Amplify: run iterative refinement on survivors
        let signal_rounds = self.config.signal_rounds;
        for round in 0..signal_rounds {
            let scores: Vec<f64> = self
                .candidates
                .iter()
                .map(|c| evaluate_quality(c))
                .collect();
            for (c, score) in self.candidates.iter_mut().zip(scores) {
                let boost = 1.0 + (0.1 * (round + 1) as f64);
                c.confidence = (c.confidence * boost).min(1.0);
                c.quality_score = score;
            }
        }

        // Sort by quality descending
        self.candidates.sort_by(|a, b| {
            b.quality_score
                .partial_cmp(&a.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return references to survivors
        self.candidates.iter().collect()
    }

    /// MERGE phase: Combine the best candidates into one output.
    pub fn merge(&mut self) -> Option<Candidate> {
        self.phase = PipelinePhase::Merged;
        if self.candidates.is_empty() {
            return None;
        }

        // Winner-take-all: highest quality candidate
        let best = self.candidates[0].clone();
        Some(best)
    }

    /// Run the full SPECTRUM → SIGNAL → MERGE pipeline.
    pub fn run_pipeline(&mut self, base: &[u8], task_context: &str) -> Option<Candidate> {
        self.generate_spectrum(base, task_context);
        self.amplify_signal();
        self.merge()
    }

    /// Remove candidates that are too similar to each other (pairwise).
    fn filter_by_pairwise_diversity(&mut self) {
        let mut keep: Vec<usize> = Vec::new();
        for i in 0..self.candidates.len() {
            let mut should_keep = true;
            for j in 0..keep.len() {
                let sim = self.hamming_similarity(
                    &self.candidates[i].vector,
                    &self.candidates[keep[j]].vector,
                );
                if sim > 0.8 {
                    should_keep = false;
                    break;
                }
            }
            if should_keep {
                keep.push(i);
            }
        }
        self.candidates = keep
            .into_iter()
            .map(|i| self.candidates[i].clone())
            .collect();
    }

    fn hamming_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let n = a.len().min(b.len());
        let same = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        same as f64 / n as f64
    }
}

/// Evaluate quality of a candidate (simplified — uses vector entropy + confidence).
fn evaluate_quality(candidate: &Candidate) -> f64 {
    let entropy: f64 = candidate
        .vector
        .iter()
        .take(32)
        .map(|&b| {
            let p = b as f64 / 255.0;
            if p < 0.01 || p > 0.99 {
                0.0
            } else {
                -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
            }
        })
        .sum::<f64>()
        / 32.0;

    let diversity_component = candidate.diversity_score * 0.3;
    let entropy_component = (entropy / 8.0) * 0.3;
    let confidence_component = candidate.confidence * 0.4;

    (diversity_component + entropy_component + confidence_component).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectrum_generation() {
        let config = SpectrumConfig::default();
        let mut ss = SpectrumSignal::new(config);
        let base = vec![128u8; 64];
        let candidates = ss.generate_spectrum(&base, "test_task");
        assert_eq!(candidates.len(), 5);
        assert!(candidates.iter().all(|c| c.diversity_score > 0.0));
    }

    #[test]
    fn test_signal_amplification() {
        let mut ss = SpectrumSignal::new(SpectrumConfig::default());
        ss.generate_spectrum(&vec![128u8; 64], "test");
        let amplified = ss.amplify_signal();
        assert!(!amplified.is_empty());
        for c in amplified {
            assert!(c.quality_score >= 0.4);
        }
    }

    #[test]
    fn test_full_pipeline() {
        let mut ss = SpectrumSignal::new(SpectrumConfig::default());
        let result = ss.run_pipeline(&vec![128u8; 64], "test_pipeline");
        assert!(result.is_some());
        assert_eq!(ss.phase(), PipelinePhase::Merged);
    }

    #[test]
    fn test_pairwise_diversity() {
        let mut ss = SpectrumSignal::new(SpectrumConfig {
            n_candidates: 10,
            pairwise_diversity: true,
            ..Default::default()
        });
        let candidates = ss.generate_spectrum(&vec![128u8; 64], "diversity");
        assert!(candidates.len() <= 10);
        assert!(candidates.len() >= 1);
    }

    #[test]
    fn test_default_config() {
        let config = SpectrumConfig::default();
        assert_eq!(config.n_candidates, 5);
        assert!((config.diversity_threshold - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_merge_empty() {
        let mut ss = SpectrumSignal::new(SpectrumConfig::default());
        let result = ss.merge();
        assert!(result.is_none());
    }
}
