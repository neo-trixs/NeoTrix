use super::core::FIELD_NAMES;
use super::self_iterating::ReasoningBrain;
use std::path::PathBuf;

/// 一个缺口驱动的种子建议
#[derive(Debug, Clone)]
pub struct GapSeed {
    pub url: String,
    pub source: GapSource,
    pub priority: f64,
}

/// 缺口来源
#[derive(Debug, Clone, PartialEq)]
pub enum GapSource {
    CapabilityWeakness,
    HypercubeSparsity,
}

/// 缺口分析器 — 统一汇聚能力向量缺口 + 超立方体稀疏信号，生成探索/爬取种子
pub struct GapAnalyzer {
    _work_dir: PathBuf,
}

impl GapAnalyzer {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            _work_dir: work_dir,
        }
    }

    /// capability gaps only (no hypercube gaps)
    pub fn analyze_capability(&self, brain: &ReasoningBrain) -> Vec<GapSeed> {
        self.capability_gaps(brain)
    }

    /// 汇聚两种缺口信号，返回按优先级降序排列的种子
    #[cfg(any())]
    pub fn analyze(&self, brain: &ReasoningBrain) -> Vec<GapSeed> {
        let mut seeds = Vec::new();
        seeds.extend(self.capability_gaps(brain));
        if let Some(r) = router {
            seeds.extend(self.hypercube_gaps(r));
        }
        seeds.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        seeds
    }

    /// 从能力向量弱维度生成种子 (capability arr < 0.3)
    pub fn capability_gaps(&self, brain: &ReasoningBrain) -> Vec<GapSeed> {
        let mut seeds = Vec::new();
        let arr = brain.capability.arr();
        let weak: Vec<(usize, f64)> = arr
            .iter()
            .enumerate()
            .filter(|(_, &v)| v < 0.3)
            .map(|(i, &v)| (i, v))
            .collect();

        for (idx, score) in &weak {
            let name = FIELD_NAMES.get(*idx).copied().unwrap_or("unknown");
            let urls = match name {
                "inference_depth" | "analysis" => vec![
                    "https://en.wikipedia.org/wiki/Reasoning",
                    "https://en.wikipedia.org/wiki/Critical_thinking",
                    "https://en.wikipedia.org/wiki/Problem_solving",
                ],
                "synthesis" | "creativity" => vec![
                    "https://en.wikipedia.org/wiki/Creativity",
                    "https://en.wikipedia.org/wiki/Innovation",
                    "https://en.wikipedia.org/wiki/Design_thinking",
                ],
                "domain_specificity" => vec![
                    "https://en.wikipedia.org/wiki/Expert",
                    "https://en.wikipedia.org/wiki/Specialization",
                ],
                "experimental" => vec![
                    "https://en.wikipedia.org/wiki/Scientific_method",
                    "https://en.wikipedia.org/wiki/Experiment",
                ],
                _ => continue,
            };
            let priority = 1.0 - score;
            for url in urls {
                seeds.push(GapSeed {
                    url: url.to_string(),
                    source: GapSource::CapabilityWeakness,
                    priority,
                });
            }
        }
        seeds
    }

    /// 从超立方体稀疏维度生成种子 (sparsity_score > 0.7)
    #[cfg(any())]
    fn hypercube_gaps(&self) -> Vec<GapSeed> {
        let topics = router.sparse_topics();
        topics
            .into_iter()
            .map(|t| {
                let url = format!("https://en.wikipedia.org/wiki/{}", t.name());
                GapSeed {
                    url,
                    source: GapSource::HypercubeSparsity,
                    priority: 0.8,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_cap::CapabilityVector;
    use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;

    fn make_strong_cap() -> CapabilityVector {
        CapabilityVector::from_array(&vec![0.9; 23]).expect("value should be ok in test")
    }

    fn make_weak_cap() -> CapabilityVector {
        CapabilityVector::from_array(&vec![0.0; 23]).expect("value should be ok in test")
    }

    #[test]
    fn test_capability_gaps_empty_when_strong() {
        let mut brain = ReasoningBrain::new();
        brain.capability = make_strong_cap();
        let analyzer = GapAnalyzer::new(PathBuf::from("."));
        let seeds = analyzer.capability_gaps(&brain);
        assert!(seeds.is_empty(), "capability全部0.9应该没有缺口");
    }

    #[test]
    fn test_capability_gaps_finds_weak_dims() {
        let mut brain = ReasoningBrain::new();
        brain.capability = make_weak_cap();
        let analyzer = GapAnalyzer::new(PathBuf::from("."));
        let seeds = analyzer.capability_gaps(&brain);
        assert!(!seeds.is_empty(), "capability全部0.0应该有缺口种子");
        for seed in &seeds {
            assert_eq!(seed.source, GapSource::CapabilityWeakness);
            assert!(seed.url.starts_with("https://"));
            assert!(seed.priority > 0.0);
        }
    }

    #[test]
    fn test_analyze_capability_only_no_router() {
        let mut brain = ReasoningBrain::new();
        brain.capability =
            CapabilityVector::from_array(&vec![0.5; 23]).expect("value should be ok in test");
        // 使 inference_depth 弱
        if let Some(idx) = FIELD_NAMES.iter().position(|&n| n == "inference_depth") {
            brain.capability.arr[idx] = 0.1;
        }
        let analyzer = GapAnalyzer::new(PathBuf::from("."));
        let seeds = analyzer.capability_gaps(&brain);
        assert!(!seeds.is_empty(), "inference_depth弱应该有种子");
        assert!(seeds
            .iter()
            .all(|s| s.source == GapSource::CapabilityWeakness));
    }

    #[test]
    fn test_priority_sorting() {
        let mut brain = ReasoningBrain::new();
        brain.capability = make_weak_cap();
        let analyzer = GapAnalyzer::new(PathBuf::from("."));
        let seeds = analyzer.capability_gaps(&brain);
        let priorities: Vec<f64> = seeds.iter().map(|s| s.priority).collect();
        let mut sorted = priorities.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).expect("value should be ok in test"));
        assert_eq!(priorities, sorted, "种子应按优先级降序排列");
    }
}
