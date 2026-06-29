// REVIVED Evo 4
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CueTask {
    pub id: u64,
    pub description: String,
    pub target_domain: String,
    pub information_gain: f64,
    pub difficulty: f64,
    pub state_seed: Vec<u8>,
    pub completed: bool,
    pub attempts: u32,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct CueConfig {
    pub enabled: bool,
    pub max_active_tasks: usize,
    pub difficulty_decay: f64,
    pub ig_threshold: f64,
    pub novelty_bonus: f64,
    pub max_history_length: usize,
}

impl Default for CueConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active_tasks: 10,
            difficulty_decay: 0.9,
            ig_threshold: 0.15,
            novelty_bonus: 0.3,
            max_history_length: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CueStats {
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub coverage_ratio: f64,
    pub bottlenecks_found: usize,
    pub avg_information_gain: f64,
    pub total_tasks_created: u64,
}

pub struct CueEngine {
    config: CueConfig,
    active_tasks: Vec<CueTask>,
    completed_tasks: Vec<CueTask>,
    visited_state_hashes: VecDeque<u64>,
    task_id_counter: u64,
    pub cycle_count: u64,
}

impl CueEngine {
    pub fn new(config: CueConfig) -> Self {
        Self {
            config,
            active_tasks: Vec::new(),
            completed_tasks: Vec::new(),
            visited_state_hashes: VecDeque::new(),
            task_id_counter: 0,
            cycle_count: 0,
        }
    }

    pub fn record_state_visit(&mut self, hash: u64) {
        self.visited_state_hashes.push_back(hash);
        if self.visited_state_hashes.len() > self.config.max_history_length {
            self.visited_state_hashes.pop_front();
        }
    }

    pub fn compute_coverage(&self) -> f64 {
        let len = self.visited_state_hashes.len();
        if len == 0 {
            return 0.0;
        }
        let mut unique: Vec<u64> = self.visited_state_hashes.iter().copied().collect();
        unique.sort();
        unique.dedup();
        unique.len() as f64 / len as f64
    }

    pub fn detect_bottlenecks(&self) -> Vec<(u64, u64, f64)> {
        if self.visited_state_hashes.len() < 10 {
            return Vec::new();
        }
        let mut freq: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
        for h in &self.visited_state_hashes {
            *freq.entry(*h).or_insert(0) += 1;
        }
        let counts: Vec<usize> = freq.values().copied().collect();
        let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
        let variance = counts
            .iter()
            .map(|c| (*c as f64 - mean).powi(2))
            .sum::<f64>()
            / counts.len() as f64;
        let stddev = variance.sqrt();
        let threshold = mean - 0.5 * stddev;

        let rare: Vec<u64> = freq
            .iter()
            .filter(|(_, c)| **c as f64 <= threshold)
            .map(|(h, _)| *h)
            .collect();
        let frequent: Vec<u64> = freq
            .iter()
            .filter(|(_, c)| **c as f64 > threshold)
            .map(|(h, _)| *h)
            .collect();

        if rare.is_empty() || frequent.is_empty() {
            return Vec::new();
        }

        let mut bottlenecks = Vec::new();
        for r in &rare {
            let mut best_sim = 0.0f64;
            let mut best_f = 0u64;
            for f in &frequent {
                let sim = hamming_sim_u64(*r, *f);
                if sim > best_sim {
                    best_sim = sim;
                    best_f = *f;
                }
            }
            let sparsity = (1.0 - best_sim).clamp(0.0, 1.0);
            if sparsity > 0.3 {
                bottlenecks.push((best_f, *r, sparsity));
            }
        }
        bottlenecks
    }

    pub fn generate_tasks(&mut self) -> Vec<CueTask> {
        let mut tasks = Vec::new();
        let bottlenecks = self.detect_bottlenecks();
        for (freq_h, rare_h, sparsity) in &bottlenecks {
            let ig = sparsity * (1.0 + self.config.novelty_bonus);
            if ig < self.config.ig_threshold {
                continue;
            }
            let desc = format!(
                "Explore transitions from state {:x} to {:x}",
                freq_h, rare_h
            );
            if self.active_tasks.iter().any(|t| t.description == desc)
                || self.completed_tasks.iter().any(|t| t.description == desc)
            {
                continue;
            }
            self.task_id_counter += 1;
            tasks.push(CueTask {
                id: self.task_id_counter,
                description: desc,
                target_domain: "exploration".to_string(),
                information_gain: ig,
                difficulty: sparsity.clamp(0.0, 1.0),
                state_seed: vec![(rare_h & 0xFF) as u8; 16],
                completed: false,
                attempts: 0,
                success_rate: 0.0,
            });
        }

        let coverage = self.compute_coverage();
        if coverage < 0.3
            && !self
                .active_tasks
                .iter()
                .any(|t| t.description == "Discover new state patterns")
            && !self
                .completed_tasks
                .iter()
                .any(|t| t.description == "Discover new state patterns")
        {
            let ig = (1.0 - coverage) * 0.5;
            if ig >= self.config.ig_threshold {
                self.task_id_counter += 1;
                tasks.push(CueTask {
                    id: self.task_id_counter,
                    description: "Discover new state patterns".to_string(),
                    target_domain: "exploration".to_string(),
                    information_gain: ig,
                    difficulty: (1.0 - coverage).clamp(0.0, 1.0),
                    state_seed: vec![0u8; 16],
                    completed: false,
                    attempts: 0,
                    success_rate: 0.0,
                });
            }
        }

        for task in &tasks {
            if self.active_tasks.len() >= self.config.max_active_tasks {
                break;
            }
            self.active_tasks.push(task.clone());
        }
        tasks
    }

    pub fn synthesize_curriculum(&self) -> Vec<CueTask> {
        let mut sorted = self.active_tasks.clone();
        sorted.sort_by(|a, b| {
            b.information_gain
                .partial_cmp(&a.information_gain)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(self.config.max_active_tasks);
        sorted
    }

    pub fn record_task_outcome(&mut self, id: u64, success: bool) {
        if let Some(pos) = self.active_tasks.iter().position(|t| t.id == id) {
            let task = &mut self.active_tasks[pos];
            task.attempts += 1;
            let n = task.attempts as f64;
            task.success_rate =
                (task.success_rate * (n - 1.0) + if success { 1.0 } else { 0.0 }) / n;
            if task.success_rate >= 0.8 {
                let mut completed = self.active_tasks.remove(pos);
                completed.completed = true;
                self.completed_tasks.push(completed);
            }
        }
    }

    pub fn cycle(&mut self) {
        self.cycle_count += 1;
    }

    pub fn stats(&self) -> CueStats {
        let total_ig: f64 = self.active_tasks.iter().map(|t| t.information_gain).sum();
        let avg_ig = if !self.active_tasks.is_empty() {
            total_ig / self.active_tasks.len() as f64
        } else {
            0.0
        };
        CueStats {
            active_tasks: self.active_tasks.len(),
            completed_tasks: self.completed_tasks.len(),
            coverage_ratio: self.compute_coverage(),
            bottlenecks_found: self
                .active_tasks
                .iter()
                .filter(|t| t.target_domain == "exploration")
                .count(),
            avg_information_gain: avg_ig,
            total_tasks_created: self.task_id_counter,
        }
    }
}

fn hamming_sim_u64(a: u64, b: u64) -> f64 {
    let diff = (a ^ b).count_ones();
    1.0 - diff as f64 / 64.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let cfg = CueConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.max_active_tasks, 10);
        assert!((cfg.difficulty_decay - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_empty_engine_safe_defaults() {
        let engine = CueEngine::new(CueConfig::default());
        let stats = engine.stats();
        assert_eq!(stats.active_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
        assert!((stats.coverage_ratio - 0.0).abs() < 1e-6);
        assert_eq!(stats.total_tasks_created, 0);
    }

    #[test]
    fn test_coverage_all_unique() {
        let mut engine = CueEngine::new(CueConfig::default());
        for i in 0..10 {
            engine.record_state_visit(i);
        }
        let cov = engine.compute_coverage();
        assert!(
            (cov - 1.0).abs() < 1e-6,
            "all unique -> coverage 1.0, got {}",
            cov
        );
    }

    #[test]
    fn test_coverage_all_same() {
        let mut engine = CueEngine::new(CueConfig::default());
        for _ in 0..10 {
            engine.record_state_visit(42);
        }
        let cov = engine.compute_coverage();
        assert!(
            (cov - 0.1).abs() < 1e-6,
            "all same -> coverage 0.1, got {}",
            cov
        );
    }

    #[test]
    fn test_detect_bottleneck_insufficient_data() {
        let engine = CueEngine::new(CueConfig::default());
        assert!(engine.detect_bottlenecks().is_empty());
    }

    #[test]
    fn test_detect_bottleneck_pattern() {
        let mut engine = CueEngine::new(CueConfig::default());
        for _ in 0..20 {
            engine.record_state_visit(100);
        }
        for i in 0..5 {
            engine.record_state_visit(200 + i);
        }
        let bottlenecks = engine.detect_bottlenecks();
        assert!(
            !bottlenecks.is_empty(),
            "should detect bottlenecks with imbalanced visits"
        );
    }

    #[test]
    fn test_generate_tasks_from_coverage_gap() {
        let mut engine = CueEngine::new(CueConfig {
            ig_threshold: 0.05,
            ..Default::default()
        });
        for _ in 0..20 {
            engine.record_state_visit(42);
        }
        let tasks = engine.generate_tasks();
        assert!(
            tasks
                .iter()
                .any(|t| t.description == "Discover new state patterns"),
            "should create coverage gap task when coverage < 0.3"
        );
    }

    #[test]
    fn test_task_deduplication() {
        let mut engine = CueEngine::new(CueConfig {
            ig_threshold: 0.01,
            novelty_bonus: 0.0,
            ..Default::default()
        });
        for _ in 0..20 {
            engine.record_state_visit(100);
        }
        for _ in 0..3 {
            engine.record_state_visit(999);
        }

        engine.generate_tasks();
        let count_before = engine.active_tasks.len();
        engine.generate_tasks();
        assert_eq!(
            engine.active_tasks.len(),
            count_before,
            "should not create duplicate tasks"
        );
    }

    #[test]
    fn test_record_outcome_completes_task() {
        let mut engine = CueEngine::new(CueConfig {
            ig_threshold: 0.01,
            ..Default::default()
        });
        for _ in 0..20 {
            engine.record_state_visit(100);
        }
        for _ in 0..3 {
            engine.record_state_visit(999);
        }
        engine.generate_tasks();

        if let Some(task) = engine.active_tasks.first() {
            let id = task.id;
            for _ in 0..5 {
                engine.record_task_outcome(id, true);
            }
            assert!(
                engine.active_tasks.is_empty() || engine.active_tasks.iter().all(|t| t.id != id),
                "task should be removed from active after 5 successes"
            );
            assert!(
                engine.completed_tasks.iter().any(|t| t.id == id),
                "task should be in completed"
            );
        }
    }

    #[test]
    fn test_synthesize_curriculum_orders_by_ig() {
        let mut engine = CueEngine::new(CueConfig {
            max_active_tasks: 5,
            ..Default::default()
        });

        for i in 0..5 {
            engine.task_id_counter += 1;
            engine.active_tasks.push(CueTask {
                id: engine.task_id_counter,
                description: format!("task_{}", i),
                target_domain: "test".to_string(),
                information_gain: i as f64 * 0.2,
                difficulty: 0.5,
                state_seed: vec![],
                completed: false,
                attempts: 0,
                success_rate: 0.0,
            });
        }
        let curriculum = engine.synthesize_curriculum();
        assert_eq!(curriculum.len(), 5);
        for w in curriculum.windows(2) {
            assert!(
                w[0].information_gain >= w[1].information_gain,
                "tasks should be sorted by IG descending"
            );
        }
    }

    #[test]
    fn test_bottleneck_detection_rare_vs_frequent() {
        let mut engine = CueEngine::new(CueConfig::default());
        for _ in 0..30 {
            engine.record_state_visit(1);
        }
        for _ in 0..30 {
            engine.record_state_visit(2);
        }
        for _ in 0..3 {
            engine.record_state_visit(99);
        }
        for _ in 0..2 {
            engine.record_state_visit(100);
        }
        let bottlenecks = engine.detect_bottlenecks();
        assert!(
            bottlenecks.len() >= 1,
            "should find at least one bottleneck pair"
        );
        for (freq, rare, sparsity) in &bottlenecks {
            assert!(
                *freq == 1 || *freq == 2,
                "frequent state should be one of the common ones"
            );
            assert!(
                *rare == 99 || *rare == 100,
                "rare state should be one of the uncommon ones"
            );
            assert!(*sparsity > 0.0, "sparsity should be positive");
        }
    }

    #[test]
    fn test_stats_aggregation() {
        let mut engine = CueEngine::new(CueConfig::default());
        engine.task_id_counter = 5;
        engine.active_tasks.push(CueTask {
            id: 1,
            description: "a".into(),
            target_domain: "test".into(),
            information_gain: 0.5,
            difficulty: 0.5,
            state_seed: vec![],
            completed: false,
            attempts: 2,
            success_rate: 0.5,
        });
        let stats = engine.stats();
        assert_eq!(stats.active_tasks, 1);
        assert_eq!(stats.total_tasks_created, 5);
        assert!((stats.avg_information_gain - 0.5).abs() < 1e-6);
    }
}
