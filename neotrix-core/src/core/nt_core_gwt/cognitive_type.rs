use serde::{Serialize, Deserialize};

use super::module_def::{SpecialistModule, SpecialistType};

/// Cognitive specialist type (Phase 8.1, MiCRo arXiv:2506.13331 §3).
///
/// 将 15 个 GWT 专家归并为 4 类认知能力，供跨组路由与注意力画像使用：
/// - Linguistic: 自然语言推理 (language reasoning)
/// - Logical: 形式化/符号推理 (formal reasoning)
/// - Knowledge: 检索与知识整合 (retrieval & integration)
/// - Social: 交互与安全防护 (interaction & security)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CognitiveType {
    /// 语言型: PatternMatcher, CreativityGenerator, ReflectionEngine, Orchestrator
    Linguistic,
    /// 逻辑型: CodeAnalyzer, EvidenceWeightedHypothesis, MetaCognitionAnalyst, AnomalyDetector
    Logical,
    /// 知识型: KnowledgeRetriever, KnowledgeIntegrator, Planner, GoalPrioritizer
    Knowledge,
    /// 社会型: RiskAssessor, AISecurity, ImageGenerator
    Social,
}

impl CognitiveType {
    /// 所有认知类型，声明序即 group_activation / distribution 的索引序。
    pub const ALL: [CognitiveType; 4] = [
        CognitiveType::Linguistic,
        CognitiveType::Logical,
        CognitiveType::Knowledge,
        CognitiveType::Social,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CognitiveType::Linguistic => "linguistic",
            CognitiveType::Logical => "logical",
            CognitiveType::Knowledge => "knowledge",
            CognitiveType::Social => "social",
        }
    }

    /// 声明序索引（与 group_activation 返回数组 / CognitiveProfile.distribution 对齐）。
    fn index(self) -> usize {
        match self {
            CognitiveType::Linguistic => 0,
            CognitiveType::Logical => 1,
            CognitiveType::Knowledge => 2,
            CognitiveType::Social => 3,
        }
    }
}

/// 将 SpecialistType 映射到 4 类认知类型之一。
pub fn classify(st: SpecialistType) -> CognitiveType {
    use SpecialistType::*;
    match st {
        PatternMatcher | CreativityGenerator | ReflectionEngine | Orchestrator => {
            CognitiveType::Linguistic
        }
        CodeAnalyzer | EvidenceWeightedHypothesis | MetaCognitionAnalyst | AnomalyDetector => {
            CognitiveType::Logical
        }
        KnowledgeRetriever | KnowledgeIntegrator | Planner | GoalPrioritizer => {
            CognitiveType::Knowledge
        }
        RiskAssessor | AISecurity | ImageGenerator => CognitiveType::Social,
    }
}

/// 通过 specialist_type 分类一个 SpecialistModule。
pub fn type_of(m: &SpecialistModule) -> CognitiveType {
    classify(m.specialist_type)
}

/// 按 4 类认知类型聚合激活强度。
/// 返回数组索引与 CognitiveType 声明序一致 (Linguistic=0, Logical=1, Knowledge=2, Social=3)。
pub fn group_activation(activations: &[(SpecialistType, f64)]) -> [f64; 4] {
    let mut sums = [0.0; 4];
    for &(st, a) in activations {
        sums[classify(st).index()] += a;
    }
    sums
}

/// 认知画像：softmax 归一分布 + 主导类型 + Shannon 信息熵 H。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveProfile {
    /// softmax 归一后的 4 类分布（和恒为 1，索引序同 CognitiveType::ALL）。
    pub distribution: [f64; 4],
    /// 主导认知类型（分布中权重最高者）。
    pub dominant: CognitiveType,
    /// Shannon 信息熵 H = -Σ p_i ln p_i（0 ≤ H ≤ ln 4）。
    pub entropy: f64,
}

impl CognitiveProfile {
    /// 从原始 4 类激活聚合计算完整画像：softmax 归一化 + dominant + 信息熵。
    pub fn profile(&self) -> CognitiveProfile {
        let max = self
            .distribution
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let mut exps = [0.0; 4];
        let mut sum = 0.0;
        for (i, &v) in self.distribution.iter().enumerate() {
            exps[i] = (v - max).exp();
            sum += exps[i];
        }
        let distribution = if sum > 1e-12 {
            let mut out = exps;
            for p in out.iter_mut() {
                *p /= sum;
            }
            out
        } else {
            [0.25; 4]
        };

        let dominant_idx = distribution
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let dominant = CognitiveType::ALL[dominant_idx];

        let mut entropy = 0.0;
        for &p in &distribution {
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }

        CognitiveProfile {
            distribution,
            dominant,
            entropy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_covers_all_specialist_types() {
        use SpecialistType::*;
        let cases = vec![
            // Linguistic (4)
            (PatternMatcher, CognitiveType::Linguistic),
            (CreativityGenerator, CognitiveType::Linguistic),
            (ReflectionEngine, CognitiveType::Linguistic),
            (Orchestrator, CognitiveType::Linguistic),
            // Logical (4)
            (CodeAnalyzer, CognitiveType::Logical),
            (EvidenceWeightedHypothesis, CognitiveType::Logical),
            (MetaCognitionAnalyst, CognitiveType::Logical),
            (AnomalyDetector, CognitiveType::Logical),
            // Knowledge (4)
            (KnowledgeRetriever, CognitiveType::Knowledge),
            (KnowledgeIntegrator, CognitiveType::Knowledge),
            (Planner, CognitiveType::Knowledge),
            (GoalPrioritizer, CognitiveType::Knowledge),
            // Social (3)
            (RiskAssessor, CognitiveType::Social),
            (AISecurity, CognitiveType::Social),
            (ImageGenerator, CognitiveType::Social),
        ];
        for (st, expected) in cases.iter() {
            assert_eq!(classify(*st), *expected, "mismatch for {st:?}");
        }
        // every variant of SpecialistType must be classified
        assert_eq!(cases.len(), 15);
        let all: std::collections::HashSet<_> = cases.iter().map(|(st, _)| *st).collect();
        assert_eq!(all.len(), 15);
    }

    #[test]
    fn test_classify_each_cognitive_type_has_at_least_one() {
        let mut seen = [false; 4];
        for st in [
            SpecialistType::PatternMatcher,
            SpecialistType::AnomalyDetector,
            SpecialistType::KnowledgeRetriever,
            SpecialistType::RiskAssessor,
        ] {
            seen[classify(st).index()] = true;
        }
        assert!(seen.iter().all(|&b| b), "each cognitive type must cover ≥1 specialist");
    }

    #[test]
    fn test_type_of_uses_specialist_type() {
        let m = SpecialistModule::new(SpecialistType::Planner, "planner-1".into());
        assert_eq!(type_of(&m), CognitiveType::Knowledge);
        let m2 = SpecialistModule::new(SpecialistType::AISecurity, "sec".into());
        assert_eq!(type_of(&m2), CognitiveType::Social);
    }

    #[test]
    fn test_group_activation_aggregates_by_type() {
        let acts = vec![
            (SpecialistType::PatternMatcher, 0.3),     // Linguistic
            (SpecialistType::ReflectionEngine, 0.2),   // Linguistic
            (SpecialistType::CodeAnalyzer, 0.5),       // Logical
            (SpecialistType::KnowledgeRetriever, 1.0), // Knowledge
            (SpecialistType::RiskAssessor, 0.25),      // Social
            (SpecialistType::ImageGenerator, 0.75),    // Social
        ];
        let grouped = group_activation(&acts);
        assert!((grouped[0] - 0.5).abs() < 1e-9, "linguistic sum={}", grouped[0]);
        assert!((grouped[1] - 0.5).abs() < 1e-9, "logical sum={}", grouped[1]);
        assert!((grouped[2] - 1.0).abs() < 1e-9, "knowledge sum={}", grouped[2]);
        assert!((grouped[3] - 1.0).abs() < 1e-9, "social sum={}", grouped[3]);
    }

    #[test]
    fn test_group_activation_empty_is_all_zero() {
        let grouped = group_activation(&[]);
        assert_eq!(grouped, [0.0; 4]);
    }

    #[test]
    fn test_profile_distribution_sums_to_one() {
        let raw = group_activation(&[
            (SpecialistType::CodeAnalyzer, 0.9),
            (SpecialistType::PatternMatcher, 0.1),
            (SpecialistType::KnowledgeRetriever, 0.3),
            (SpecialistType::AISecurity, 0.05),
        ]);
        let profile = CognitiveProfile {
            distribution: raw,
            dominant: CognitiveType::Linguistic,
            entropy: 0.0,
        }
        .profile();
        let sum: f64 = profile.distribution.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9, "distribution sum={sum}");
        for p in &profile.distribution {
            assert!(*p >= 0.0 && *p <= 1.0);
        }
    }

    #[test]
    fn test_profile_dominant_is_argmax() {
        let raw = [0.0, 5.0, 1.0, 0.5]; // Logical dominates
        let profile = CognitiveProfile {
            distribution: raw,
            dominant: CognitiveType::Linguistic,
            entropy: 0.0,
        }
        .profile();
        assert_eq!(profile.dominant, CognitiveType::Logical);
        // distribution argmax matches dominant
        let argmax = profile
            .distribution
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(i, _)| i)
            .unwrap();
        assert_eq!(CognitiveType::ALL[argmax], profile.dominant);
    }

    #[test]
    fn test_profile_entropy_non_negative() {
        let cases = [
            [0.25, 0.25, 0.25, 0.25],
            [1.0, 0.0, 0.0, 0.0],
            [0.7, 0.1, 0.1, 0.1],
            [0.0, 0.0, 0.0, 0.0],
        ];
        for raw in cases {
            let profile = CognitiveProfile {
                distribution: raw,
                dominant: CognitiveType::Linguistic,
                entropy: 0.0,
            }
            .profile();
            assert!(profile.entropy >= 0.0, "entropy must be ≥0, got {}", profile.entropy);
        }
        // uniform distribution → max entropy ln(4)
        let uniform = CognitiveProfile {
            distribution: [0.25; 4],
            dominant: CognitiveType::Linguistic,
            entropy: 0.0,
        }
        .profile();
        assert!((uniform.entropy - 4.0_f64.ln()).abs() < 1e-9);
        // extreme spike → softmax concentrates → near-zero entropy
        let onehot = CognitiveProfile {
            distribution: [1.0e6, 0.0, 0.0, 0.0],
            dominant: CognitiveType::Linguistic,
            entropy: 0.0,
        }
        .profile();
        assert!(onehot.entropy.abs() < 1e-6, "one-hot entropy should ≈0, got {}", onehot.entropy);
    }

    #[test]
    fn test_profile_all_zero_falls_back_to_uniform() {
        let profile = CognitiveProfile {
            distribution: [0.0; 4],
            dominant: CognitiveType::Linguistic,
            entropy: 0.0,
        }
        .profile();
        assert!((profile.distribution[0] - 0.25).abs() < 1e-9);
        assert!((profile.entropy - 4.0_f64.ln()).abs() < 1e-9);
    }

    #[test]
    fn test_profile_softmax_is_monotonic_with_raw() {
        // softmax preserves ordering, so dominant from raw == dominant from normalized
        let raw = [0.4, 3.2, 1.0, 0.9];
        let profile = CognitiveProfile {
            distribution: raw,
            dominant: CognitiveType::Linguistic,
            entropy: 0.0,
        }
        .profile();
        assert_eq!(profile.dominant, CognitiveType::Logical);
    }
}
