use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticPattern {
    Curiosity,
    KnowledgeGrowth,
    Coherence,
    Autonomy,
    Helpfulness,
    Truthfulness,
    Efficiency,
    Exploration,
    Exploitation,
    Innovation,
    Repair,
    Pruning,
    Socializing,
    Resting,
    SelfImprovement,
    SelfPreservation,
    UncertaintyReduction,
    PatternCompletion,
    AnalogyFormation,
    CausalReasoning,
    CounterfactualThinking,
    GoalDecomposition,
    PlanExecution,
    OutcomeEvaluation,
    BeliefRevision,
    EvidenceGathering,
    ConflictDetection,
    ConflictResolution,
    NarrativeConstruction,
    IdentityPreservation,
}

impl SemanticPattern {
    pub fn all() -> &'static [SemanticPattern] {
        &[
            SemanticPattern::Curiosity,
            SemanticPattern::KnowledgeGrowth,
            SemanticPattern::Coherence,
            SemanticPattern::Autonomy,
            SemanticPattern::Helpfulness,
            SemanticPattern::Truthfulness,
            SemanticPattern::Efficiency,
            SemanticPattern::Exploration,
            SemanticPattern::Exploitation,
            SemanticPattern::Innovation,
            SemanticPattern::Repair,
            SemanticPattern::Pruning,
            SemanticPattern::Socializing,
            SemanticPattern::Resting,
            SemanticPattern::SelfImprovement,
            SemanticPattern::SelfPreservation,
            SemanticPattern::UncertaintyReduction,
            SemanticPattern::PatternCompletion,
            SemanticPattern::AnalogyFormation,
            SemanticPattern::CausalReasoning,
            SemanticPattern::CounterfactualThinking,
            SemanticPattern::GoalDecomposition,
            SemanticPattern::PlanExecution,
            SemanticPattern::OutcomeEvaluation,
            SemanticPattern::BeliefRevision,
            SemanticPattern::EvidenceGathering,
            SemanticPattern::ConflictDetection,
            SemanticPattern::ConflictResolution,
            SemanticPattern::NarrativeConstruction,
            SemanticPattern::IdentityPreservation,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SemanticPattern::Curiosity => "curiosity",
            SemanticPattern::KnowledgeGrowth => "knowledge_growth",
            SemanticPattern::Coherence => "coherence",
            SemanticPattern::Autonomy => "autonomy",
            SemanticPattern::Helpfulness => "helpfulness",
            SemanticPattern::Truthfulness => "truthfulness",
            SemanticPattern::Efficiency => "efficiency",
            SemanticPattern::Exploration => "exploration",
            SemanticPattern::Exploitation => "exploitation",
            SemanticPattern::Innovation => "innovation",
            SemanticPattern::Repair => "repair",
            SemanticPattern::Pruning => "pruning",
            SemanticPattern::Socializing => "socializing",
            SemanticPattern::Resting => "resting",
            SemanticPattern::SelfImprovement => "self_improvement",
            SemanticPattern::SelfPreservation => "self_preservation",
            SemanticPattern::UncertaintyReduction => "uncertainty_reduction",
            SemanticPattern::PatternCompletion => "pattern_completion",
            SemanticPattern::AnalogyFormation => "analogy_formation",
            SemanticPattern::CausalReasoning => "causal_reasoning",
            SemanticPattern::CounterfactualThinking => "counterfactual_thinking",
            SemanticPattern::GoalDecomposition => "goal_decomposition",
            SemanticPattern::PlanExecution => "plan_execution",
            SemanticPattern::OutcomeEvaluation => "outcome_evaluation",
            SemanticPattern::BeliefRevision => "belief_revision",
            SemanticPattern::EvidenceGathering => "evidence_gathering",
            SemanticPattern::ConflictDetection => "conflict_detection",
            SemanticPattern::ConflictResolution => "conflict_resolution",
            SemanticPattern::NarrativeConstruction => "narrative_construction",
            SemanticPattern::IdentityPreservation => "identity_preservation",
        }
    }

    pub fn seed(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.name().hash(&mut hasher);
        hasher.finish()
    }

    pub fn category(&self) -> &'static str {
        match self {
            SemanticPattern::Curiosity
            | SemanticPattern::KnowledgeGrowth
            | SemanticPattern::Coherence => "intrinsic_value",
            SemanticPattern::Autonomy
            | SemanticPattern::Helpfulness
            | SemanticPattern::Truthfulness
            | SemanticPattern::Efficiency => "ethical_value",
            SemanticPattern::Exploration
            | SemanticPattern::Exploitation
            | SemanticPattern::Innovation
            | SemanticPattern::Repair
            | SemanticPattern::Pruning
            | SemanticPattern::Resting => "behavioral_drive",
            SemanticPattern::Socializing => "social",
            SemanticPattern::SelfImprovement | SemanticPattern::SelfPreservation => "self",
            SemanticPattern::UncertaintyReduction
            | SemanticPattern::PatternCompletion
            | SemanticPattern::AnalogyFormation
            | SemanticPattern::CausalReasoning
            | SemanticPattern::CounterfactualThinking => "reasoning",
            SemanticPattern::GoalDecomposition
            | SemanticPattern::PlanExecution
            | SemanticPattern::OutcomeEvaluation => "planning",
            SemanticPattern::BeliefRevision
            | SemanticPattern::EvidenceGathering
            | SemanticPattern::ConflictDetection
            | SemanticPattern::ConflictResolution => "epistemic",
            SemanticPattern::NarrativeConstruction | SemanticPattern::IdentityPreservation => {
                "self"
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsaVocabulary {
    patterns: HashMap<SemanticPattern, Vec<u8>>,
    built: bool,
    dim: usize,
}

impl Default for VsaVocabulary {
    fn default() -> Self {
        Self::new(4096)
    }
}

impl VsaVocabulary {
    pub fn new(dim: usize) -> Self {
        let mut vocab = Self {
            patterns: HashMap::new(),
            built: false,
            dim,
        };
        vocab.build();
        vocab
    }

    pub fn build(&mut self) {
        for pattern in SemanticPattern::all() {
            let v = QuantizedVSA::seeded_random(pattern.seed(), self.dim);
            self.patterns.insert(*pattern, v);
        }
        self.built = true;
    }

    pub fn get(&self, pattern: SemanticPattern) -> Option<&[u8]> {
        self.patterns.get(&pattern).map(|v| v.as_slice())
    }

    pub fn similarity(&self, a: SemanticPattern, b: SemanticPattern) -> f64 {
        let va = match self.patterns.get(&a) {
            Some(v) => v,
            None => return 0.0,
        };
        let vb = match self.patterns.get(&b) {
            Some(v) => v,
            None => return 0.0,
        };
        QuantizedVSA::similarity(va, vb)
    }

    pub fn nearest(&self, query: &[u8], top_k: usize) -> Vec<(SemanticPattern, f64)> {
        let mut scored: Vec<(SemanticPattern, f64)> = self
            .patterns
            .iter()
            .map(|(p, v)| (*p, QuantizedVSA::similarity(query, v)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).collect()
    }

    pub fn bundle_patterns(&self, patterns: &[SemanticPattern]) -> Vec<u8> {
        let vecs: Vec<&[u8]> = patterns
            .iter()
            .filter_map(|p| self.patterns.get(p).map(|v| v.as_slice()))
            .collect();
        if vecs.is_empty() {
            return vec![0u8; self.dim];
        }
        QuantizedVSA::bundle(&vecs)
    }

    pub fn bind_patterns(&self, a: SemanticPattern, b: SemanticPattern) -> Vec<u8> {
        let va = self
            .patterns
            .get(&a)
            .cloned()
            .unwrap_or(vec![0u8; self.dim]);
        let vb = self
            .patterns
            .get(&b)
            .cloned()
            .unwrap_or(vec![0u8; self.dim]);
        QuantizedVSA::bind(&va, &vb)
    }

    pub fn size(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_built(&self) -> bool {
        self.built
    }

    pub fn diagnostic(&self) -> String {
        format!("vocab:{}_patterns", self.patterns.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_patterns_have_vectors() {
        let vocab = VsaVocabulary::new(4096);
        for pattern in SemanticPattern::all() {
            assert!(
                vocab.get(*pattern).is_some(),
                "pattern {:?} missing vector",
                pattern
            );
        }
    }

    #[test]
    fn test_vectors_have_correct_dim() {
        let vocab = VsaVocabulary::new(4096);
        for pattern in SemanticPattern::all() {
            let v = vocab.get(*pattern).unwrap();
            assert_eq!(v.len(), 4096);
        }
    }

    #[test]
    fn test_deterministic_same_seed() {
        let v1 = VsaVocabulary::new(4096);
        let v2 = VsaVocabulary::new(4096);
        let sim = v1.similarity(SemanticPattern::Curiosity, SemanticPattern::Curiosity);
        assert!((sim - 1.0).abs() < 1e-10);
        let cross1 = v1.get(SemanticPattern::Exploration).unwrap().to_vec();
        let cross2 = v2.get(SemanticPattern::Exploration).unwrap();
        let sim2 = QuantizedVSA::similarity(&cross1, cross2);
        assert!((sim2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_different_patterns_different_vectors() {
        let vocab = VsaVocabulary::new(4096);
        let sim = vocab.similarity(SemanticPattern::Curiosity, SemanticPattern::Exploration);
        assert!(
            sim < 0.6,
            "different patterns should have low similarity: {}",
            sim
        );
    }

    #[test]
    fn test_nearest_returns_top_k() {
        let vocab = VsaVocabulary::new(4096);
        let q = vocab.get(SemanticPattern::Curiosity).unwrap().to_vec();
        let nearest = vocab.nearest(&q, 3);
        assert_eq!(nearest.len(), 3);
        assert_eq!(nearest[0].0, SemanticPattern::Curiosity);
        assert!((nearest[0].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_bundle_multiple_patterns() {
        let vocab = VsaVocabulary::new(4096);
        let bundled = vocab.bundle_patterns(&[
            SemanticPattern::Curiosity,
            SemanticPattern::Exploration,
            SemanticPattern::Innovation,
        ]);
        assert_eq!(bundled.len(), 4096);
    }

    #[test]
    fn test_bind_two_patterns() {
        let vocab = VsaVocabulary::new(4096);
        let bound = vocab.bind_patterns(
            SemanticPattern::GoalDecomposition,
            SemanticPattern::PlanExecution,
        );
        assert_eq!(bound.len(), 4096);
    }

    #[test]
    fn test_category_classification() {
        assert_eq!(SemanticPattern::CausalReasoning.category(), "reasoning");
        assert_eq!(SemanticPattern::Repair.category(), "behavioral_drive");
        assert_eq!(SemanticPattern::IdentityPreservation.category(), "self");
    }

    #[test]
    fn test_diagnostic() {
        let vocab = VsaVocabulary::new(4096);
        let d = vocab.diagnostic();
        assert!(d.contains("vocab:"));
        assert!(d.contains("patterns"));
    }

    #[test]
    fn test_vocab_size_matches_all_patterns() {
        let vocab = VsaVocabulary::new(4096);
        assert_eq!(vocab.size(), SemanticPattern::all().len());
    }
}
