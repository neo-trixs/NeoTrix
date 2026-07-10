#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceType {
    GitHub,
    ArXiv,
    Wikipedia,
    WebPage,
    Discovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumEntry {
    pub source_id: String,
    pub source_type: SourceType,
    pub difficulty: f64,
    pub success: bool,
    pub items_absorbed: usize,
    pub knowledge_gaps_filled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSummary {
    pub total_tasks: usize,
    pub success_rate: f64,
    pub avg_difficulty: f64,
    pub type_distribution: HashMap<SourceType, usize>,
    pub current_difficulty: f64,
    pub suggested_next_type: Option<SourceType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryStrategy {
    Random,
    GapFilling,
    DiversityMaximization,
    DifficultyProgression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDiscoverer {
    pub known_sources: Vec<String>,
    pub strategy: DiscoveryStrategy,
    pub next_id: usize,
}

impl SourceDiscoverer {
    pub fn new(strategy: DiscoveryStrategy) -> Self {
        Self {
            known_sources: Vec::new(),
            strategy,
            next_id: 0,
        }
    }

    pub fn discover(&self, knowledge_gaps: &[String]) -> Vec<String> {
        match self.strategy {
            DiscoveryStrategy::Random => {
                vec!["random-discovery-1".to_string(), "random-discovery-2".to_string()]
            }
            DiscoveryStrategy::GapFilling => {
                knowledge_gaps.iter().map(|gap| format!("gap-{}", gap)).collect()
            }
            DiscoveryStrategy::DiversityMaximization => {
                vec![
                    "github-explore".to_string(),
                    "arxiv-latest".to_string(),
                    "wikipedia-category".to_string(),
                    "web-general".to_string(),
                ]
            }
            DiscoveryStrategy::DifficultyProgression => {
                vec![
                    "easy-source".to_string(),
                    "medium-source".to_string(),
                    "hard-source".to_string(),
                ]
            }
        }
    }

    pub fn add_known_source(&mut self, source: String) {
        self.known_sources.push(source);
    }

    pub fn known_sources(&self) -> &[String] {
        &self.known_sources
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTypeBalancer {
    pub type_counts: HashMap<SourceType, usize>,
    pub target_ratios: HashMap<SourceType, f64>,
    pub min_per_type: usize,
}

impl TaskTypeBalancer {
    pub fn new(target_ratios: HashMap<SourceType, f64>, min_per_type: usize) -> Self {
        Self {
            type_counts: HashMap::new(),
            target_ratios,
            min_per_type,
        }
    }

    pub fn record(&mut self, source_type: SourceType) {
        *self.type_counts.entry(source_type).or_insert(0) += 1;
    }

    pub fn suggest_type(&self) -> Option<SourceType> {
        let total: usize = self.type_counts.values().sum();
        if total == 0 {
            return self.target_ratios.keys().next().copied();
        }

        let mut worst_type: Option<SourceType> = None;
        let mut worst_deficit = -1.0f64;

        for (&st, &target) in &self.target_ratios {
            let count = *self.type_counts.get(&st).unwrap_or(&0);
            let actual = count as f64 / total as f64;
            let ratio_diff = target - actual;

            if count < self.min_per_type || ratio_diff > 0.1 {
                if ratio_diff > worst_deficit {
                    worst_deficit = ratio_diff;
                    worst_type = Some(st);
                }
            }
        }

        worst_type
    }

    pub fn balance(&self) -> HashMap<SourceType, f64> {
        let total: usize = self.type_counts.values().sum();
        if total == 0 {
            return self
                .target_ratios
                .iter()
                .map(|(&k, _)| (k, 0.0))
                .collect();
        }
        self.type_counts
            .iter()
            .map(|(&k, &v)| (k, v as f64 / total as f64))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyAdjuster {
    pub current_difficulty: f64,
    pub performance_window: VecDeque<(f64, bool)>,
    pub adjustment_rate: f64,
    max_history: usize,
}

impl DifficultyAdjuster {
    pub fn new(initial_difficulty: f64, adjustment_rate: f64) -> Self {
        Self {
            current_difficulty: initial_difficulty,
            performance_window: VecDeque::with_capacity(20),
            adjustment_rate,
            max_history: 20,
        }
    }

    pub fn record_outcome(&mut self, difficulty: f64, success: bool) {
        if self.performance_window.len() >= self.max_history {
            self.performance_window.pop_front();
        }
        self.performance_window.push_back((difficulty, success));
    }

    pub fn current_difficulty(&self) -> f64 {
        self.current_difficulty
    }

    pub fn suggested_difficulty(&self) -> f64 {
        let window_len = self.performance_window.len();
        if window_len == 0 {
            return self.current_difficulty;
        }

        let successes: usize = self
            .performance_window
            .iter()
            .filter(|(_, s)| *s)
            .count();
        let rate = successes as f64 / window_len as f64;

        let adjustment = if rate > 0.7 {
            self.adjustment_rate
        } else if rate < 0.3 {
            -self.adjustment_rate
        } else {
            0.0
        };

        (self.current_difficulty + adjustment).max(0.1).min(1.0)
    }

    pub fn learning_curve(&self) -> Vec<(usize, f64)> {
        self.performance_window
            .iter()
            .enumerate()
            .map(|(i, (diff, success))| {
                let value = if *success { *diff } else { 0.0 };
                (i, value)
            })
            .collect()
    }
}

pub struct SelfCurriculumPipeline {
    pub task_history: VecDeque<CurriculumEntry>,
    pub task_type_balancer: TaskTypeBalancer,
    pub source_discoverer: SourceDiscoverer,
    pub difficulty_adjuster: DifficultyAdjuster,
    max_history: usize,
}

impl SelfCurriculumPipeline {
    pub fn new() -> Self {
        let mut target_ratios = HashMap::new();
        target_ratios.insert(SourceType::GitHub, 0.3);
        target_ratios.insert(SourceType::ArXiv, 0.2);
        target_ratios.insert(SourceType::Wikipedia, 0.2);
        target_ratios.insert(SourceType::WebPage, 0.2);
        target_ratios.insert(SourceType::Discovery, 0.1);

        Self {
            task_history: VecDeque::with_capacity(100),
            task_type_balancer: TaskTypeBalancer::new(target_ratios, 2),
            source_discoverer: SourceDiscoverer::new(DiscoveryStrategy::Random),
            difficulty_adjuster: DifficultyAdjuster::new(0.5, 0.1),
            max_history: 100,
        }
    }

    pub fn suggest_next_source(&self, knowledge_gaps: &[String]) -> Option<String> {
        let _suggested_type = self.task_type_balancer.suggest_type()?;
        let candidates = self.source_discoverer.discover(knowledge_gaps);
        candidates.into_iter().find(|_| true)
    }

    pub fn record_outcome(&mut self, entry: CurriculumEntry) {
        if self.task_history.len() >= self.max_history {
            self.task_history.pop_front();
        }
        let st = entry.source_type;
        let diff = entry.difficulty;
        let succ = entry.success;
        self.task_type_balancer.record(st);
        self.difficulty_adjuster.record_outcome(diff, succ);
        self.task_history.push_back(entry);
    }

    pub fn adjust_difficulty(&mut self) -> f64 {
        let suggested = self.difficulty_adjuster.suggested_difficulty();
        self.difficulty_adjuster.current_difficulty = suggested;
        suggested
    }

    pub fn diversity_score(&self) -> f64 {
        let actual = self.task_type_balancer.balance();
        let total: f64 = actual.values().sum();
        if total == 0.0 {
            return 0.0;
        }
        let mut diff_sum = 0.0;
        for (&st, &target) in &self.task_type_balancer.target_ratios {
            let actual_ratio = actual.get(&st).copied().unwrap_or(0.0);
            diff_sum += (actual_ratio - target).abs();
        }
        (1.0 - diff_sum / 2.0).max(0.0).min(1.0)
    }

    pub fn performance_summary(&self) -> CurriculumSummary {
        let total_tasks = self.task_history.len();
        let successes: usize = self
            .task_history
            .iter()
            .filter(|e| e.success)
            .count();
        let success_rate = if total_tasks == 0 {
            0.0
        } else {
            successes as f64 / total_tasks as f64
        };

        let total_difficulty: f64 = self.task_history.iter().map(|e| e.difficulty).sum();
        let avg_difficulty = if total_tasks == 0 {
            0.0
        } else {
            total_difficulty / total_tasks as f64
        };

        let mut type_distribution: HashMap<SourceType, usize> = HashMap::new();
        for entry in &self.task_history {
            *type_distribution.entry(entry.source_type).or_insert(0) += 1;
        }

        let suggested_next_type = self.task_type_balancer.suggest_type();

        CurriculumSummary {
            total_tasks,
            success_rate,
            avg_difficulty,
            type_distribution,
            current_difficulty: self.difficulty_adjuster.current_difficulty,
            suggested_next_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_discoverer_random() {
        let d = SourceDiscoverer::new(DiscoveryStrategy::Random);
        let sources = d.discover(&[]);
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"random-discovery-1".to_string()));
    }

    #[test]
    fn test_source_discoverer_gap_filling() {
        let d = SourceDiscoverer::new(DiscoveryStrategy::GapFilling);
        let gaps = vec!["rust".to_string(), "ml".to_string()];
        let sources = d.discover(&gaps);
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"gap-rust".to_string()));
        assert!(sources.contains(&"gap-ml".to_string()));
    }

    #[test]
    fn test_source_discoverer_diversity() {
        let d = SourceDiscoverer::new(DiscoveryStrategy::DiversityMaximization);
        let sources = d.discover(&[]);
        assert_eq!(sources.len(), 4);
    }

    #[test]
    fn test_source_discoverer_difficulty_progression() {
        let d = SourceDiscoverer::new(DiscoveryStrategy::DifficultyProgression);
        let sources = d.discover(&[]);
        assert_eq!(sources.len(), 3);
        assert_eq!(sources[0], "easy-source");
        assert_eq!(sources[2], "hard-source");
    }

    #[test]
    fn test_source_discoverer_add_known() {
        let mut d = SourceDiscoverer::new(DiscoveryStrategy::Random);
        d.add_known_source("test".to_string());
        assert_eq!(d.known_sources(), &["test"]);
    }

    #[test]
    fn test_task_type_balancer_record_and_suggest() {
        let mut ratios = HashMap::new();
        ratios.insert(SourceType::GitHub, 0.5);
        ratios.insert(SourceType::ArXiv, 0.5);
        let mut b = TaskTypeBalancer::new(ratios, 1);
        b.record(SourceType::GitHub);
        let suggested = b.suggest_type();
        assert_eq!(suggested, Some(SourceType::ArXiv));
    }

    #[test]
    fn test_task_type_balancer_balance() {
        let mut ratios = HashMap::new();
        ratios.insert(SourceType::GitHub, 1.0);
        let mut b = TaskTypeBalancer::new(ratios, 0);
        b.record(SourceType::GitHub);
        let bal = b.balance();
        assert!((bal[&SourceType::GitHub] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_difficulty_adjuster_increase_on_success() {
        let mut a = DifficultyAdjuster::new(0.5, 0.1);
        for _ in 0..15 {
            a.record_outcome(0.5, true);
        }
        let suggested = a.suggested_difficulty();
        assert!(suggested > 0.5);
    }

    #[test]
    fn test_difficulty_adjuster_decrease_on_failure() {
        let mut a = DifficultyAdjuster::new(0.5, 0.1);
        for _ in 0..15 {
            a.record_outcome(0.5, false);
        }
        let suggested = a.suggested_difficulty();
        assert!(suggested < 0.5);
    }

    #[test]
    fn test_difficulty_adjuster_clamp() {
        let mut a = DifficultyAdjuster::new(0.05, 0.5);
        for _ in 0..20 {
            a.record_outcome(0.05, false);
        }
        let suggested = a.suggested_difficulty();
        assert!(suggested >= 0.1);
    }

    #[test]
    fn test_learning_curve() {
        let mut a = DifficultyAdjuster::new(0.5, 0.1);
        a.record_outcome(0.5, true);
        a.record_outcome(0.6, false);
        let curve = a.learning_curve();
        assert_eq!(curve.len(), 2);
        assert!((curve[0].1 - 0.5).abs() < 1e-6);
        assert!((curve[1].1 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_diversity_score_uniform_is_one() {
        let mut ratios = HashMap::new();
        ratios.insert(SourceType::GitHub, 1.0);
        let balancer = TaskTypeBalancer::new(ratios, 0);
        let mut pipeline = SelfCurriculumPipeline {
            task_history: VecDeque::new(),
            task_type_balancer: balancer,
            source_discoverer: SourceDiscoverer::new(DiscoveryStrategy::Random),
            difficulty_adjuster: DifficultyAdjuster::new(0.5, 0.1),
            max_history: 100,
        };
        pipeline.record_outcome(CurriculumEntry {
            source_id: "test".to_string(),
            source_type: SourceType::GitHub,
            difficulty: 0.5,
            success: true,
            items_absorbed: 10,
            knowledge_gaps_filled: vec![],
        });
        let ds = pipeline.diversity_score();
        assert!((ds - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_self_curriculum_pipeline_suggest() {
        let pipeline = SelfCurriculumPipeline::new();
        let gaps = vec!["rust".to_string()];
        let suggestion = pipeline.suggest_next_source(&gaps);
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_performance_summary() {
        let mut pipeline = SelfCurriculumPipeline::new();
        pipeline.record_outcome(CurriculumEntry {
            source_id: "s1".to_string(),
            source_type: SourceType::GitHub,
            difficulty: 0.3,
            success: true,
            items_absorbed: 5,
            knowledge_gaps_filled: vec![],
        });
        pipeline.record_outcome(CurriculumEntry {
            source_id: "s2".to_string(),
            source_type: SourceType::ArXiv,
            difficulty: 0.7,
            success: false,
            items_absorbed: 0,
            knowledge_gaps_filled: vec![],
        });
        let summary = pipeline.performance_summary();
        assert_eq!(summary.total_tasks, 2);
        assert!((summary.success_rate - 0.5).abs() < 1e-6);
        assert!((summary.avg_difficulty - 0.5).abs() < 1e-6);
        assert!(summary.suggested_next_type.is_some());
    }

    #[test]
    fn test_adjust_difficulty() {
        let mut pipeline = SelfCurriculumPipeline::new();
        for _ in 0..15 {
            pipeline.record_outcome(CurriculumEntry {
                source_id: "s".to_string(),
                source_type: SourceType::GitHub,
                difficulty: 0.5,
                success: true,
                items_absorbed: 1,
                knowledge_gaps_filled: vec![],
            });
        }
        let adj = pipeline.adjust_difficulty();
        assert!(adj > 0.5);
    }

    #[test]
    fn test_diversity_score_zero_on_empty() {
        let pipeline = SelfCurriculumPipeline::new();
        let ds = pipeline.diversity_score();
        assert!((ds - 0.0).abs() < 1e-6);
    }
}
