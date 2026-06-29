#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::VecDeque;

/// Continuous learning loop — synthetic data flywheel
pub struct ContinuousLearning {
    pub experience_buffer: VecDeque<ExperienceEntry>,
    pub max_buffer: usize,
    pub skill_map: HashMap<String, SkillStats>,
    pub total_learned: u64,
}

/// A learning experience
#[derive(Debug, Clone)]
pub struct ExperienceEntry {
    pub task: String,
    pub success: bool,
    pub pattern: Vec<u8>,
    pub timestamp: u64,
}

/// Skill learning statistics
#[derive(Debug, Clone)]
pub struct SkillStats {
    pub attempts: u64,
    pub successes: u64,
    pub avg_duration_ms: f64,
}

impl ContinuousLearning {
    pub fn new() -> Self {
        ContinuousLearning {
            experience_buffer: VecDeque::with_capacity(256),
            max_buffer: 10000,
            skill_map: HashMap::new(),
            total_learned: 0,
        }
    }

    pub fn record_experience(&mut self, task: &str, success: bool, pattern: Vec<u8>) {
        if self.experience_buffer.len() >= self.max_buffer {
            self.experience_buffer.pop_front();
        }
        self.experience_buffer.push_back(ExperienceEntry {
            task: task.into(),
            success,
            pattern,
            timestamp: now_secs(),
        });
        let stats = self.skill_map.entry(task.into()).or_insert(SkillStats {
            attempts: 0,
            successes: 0,
            avg_duration_ms: 0.0,
        });
        stats.attempts += 1;
        if success {
            stats.successes += 1;
        }
        self.total_learned += 1;
    }

    pub fn success_rate(&self, task: &str) -> f64 {
        self.skill_map.get(task).map_or(0.0, |s| {
            if s.attempts == 0 {
                0.0
            } else {
                s.successes as f64 / s.attempts as f64
            }
        })
    }

    pub fn weak_areas(&self, threshold: f64) -> Vec<String> {
        self.skill_map
            .iter()
            .filter(|(_, s)| {
                s.attempts >= 3 && (s.successes as f64 / s.attempts as f64) < threshold
            })
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn strong_areas(&self, threshold: f64) -> Vec<String> {
        self.skill_map
            .iter()
            .filter(|(_, s)| s.attempts >= 3 && s.successes as f64 / s.attempts as f64 >= threshold)
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn recent_experiences(&self, n: usize) -> Vec<&ExperienceEntry> {
        let n = n.min(self.experience_buffer.len());
        self.experience_buffer
            .iter()
            .skip(self.experience_buffer.len() - n)
            .collect()
    }

    pub fn generate_synthetic(&self, base_task: &str, count: usize) -> Vec<ExperienceEntry> {
        let mut synthetic = Vec::new();
        if let Some(_stats) = self.skill_map.get(base_task) {
            for i in 0..count {
                synthetic.push(ExperienceEntry {
                    task: format!("{}/synth/{}", base_task, i),
                    success: rand::random::<f64>() < 0.8,
                    pattern: vec![0; 8],
                    timestamp: now_secs(),
                });
            }
        }
        synthetic
    }

    pub fn report(&self) -> String {
        let weak = self.weak_areas(0.5).len();
        let strong = self.strong_areas(0.8).len();
        format!(
            "ContinuousLearning: {} experiences, {} skills ({} weak, {} strong), total_learned={}",
            self.experience_buffer.len(),
            self.skill_map.len(),
            weak,
            strong,
            self.total_learned,
        )
    }
}

/// A synthetic training sample generated from identified knowledge gaps
#[derive(Debug, Clone)]
pub struct SyntheticSample {
    pub id: u64,
    pub input: Vec<f64>,
    pub expected_output: Vec<f64>,
    pub difficulty: f64,
    pub source: String,
    pub quality_score: f64,
}

/// Strategy governing how synthetic samples are generated and ordered
#[derive(Debug, Clone, PartialEq)]
pub enum FlywheelStrategy {
    /// Easy tasks first, progress to hard
    Curriculum,
    /// Focus generation on known weaknesses
    Adversarial,
    /// Cover a broad distribution of the input space
    Diverse,
    /// Dynamically switch between strategies based on performance
    Adaptive,
}

/// Synthetic data flywheel engine — generates, tracks, and evaluates training data
#[derive(Debug, Clone)]
pub struct DataFlywheel {
    pub samples: Vec<SyntheticSample>,
    pub next_id: u64,
    pub max_samples: usize,
    pub generation_strategy: FlywheelStrategy,
}

impl DataFlywheel {
    pub fn new(strategy: FlywheelStrategy, max_samples: usize) -> Self {
        DataFlywheel {
            samples: Vec::with_capacity(max_samples.min(64)),
            next_id: 1,
            max_samples,
            generation_strategy: strategy,
        }
    }

    pub fn generate_from_gaps(
        &mut self,
        gaps: &[(&str, f64)],
        base_vector: &[f64],
        count: usize,
    ) -> usize {
        let _base_len = base_vector.len();
        let mut generated = 0;
        for (gap_name, confidence) in gaps {
            if generated >= count {
                break;
            }
            let per_gap = ((count - generated) as f64
                / (gaps.len() - generated.min(gaps.len())) as f64)
                .ceil() as usize;
            for _i in 0..per_gap {
                if self.samples.len() >= self.max_samples {
                    break;
                }
                let noise: Vec<f64> = base_vector
                    .iter()
                    .map(|v| {
                        let n: f64 = rand::random();
                        v + (n - 0.5) * (1.0 - confidence) * 2.0
                    })
                    .collect();
                let target: Vec<f64> = gap_name.bytes().map(|b| b as f64 / 255.0).collect();
                let difficulty = 1.0 - confidence;
                let sample = SyntheticSample {
                    id: self.next_id,
                    input: noise,
                    expected_output: target,
                    difficulty,
                    source: gap_name.to_string(),
                    quality_score: confidence * 0.8 + 0.2,
                };
                self.samples.push(sample);
                self.next_id += 1;
                generated += 1;
            }
        }
        generated
    }

    pub fn curriculum_order(&self) -> Vec<&SyntheticSample> {
        let mut sorted: Vec<&SyntheticSample> = self.samples.iter().collect();
        sorted.sort_by(|a, b| {
            a.difficulty
                .partial_cmp(&b.difficulty)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    pub fn adversarial_samples(&self, weak_areas: &[f64], count: usize) -> Vec<SyntheticSample> {
        if weak_areas.is_empty() || count == 0 {
            return Vec::new();
        }
        let weakness = weak_areas.iter().sum::<f64>() / weak_areas.len() as f64;
        let mut generated = Vec::with_capacity(count);
        for i in 0..count {
            let difficulty = 0.5 + weakness * 0.5;
            let noise: Vec<f64> = (0..weak_areas.len())
                .map(|_| {
                    let n: f64 = rand::random();
                    n * difficulty * 2.0 - difficulty
                })
                .collect();
            let target: Vec<f64> = (0..weak_areas.len())
                .map(|j| {
                    let n: f64 = rand::random();
                    weak_areas[j] + (n - 0.5) * (1.0 - weak_areas[j])
                })
                .collect();
            generated.push(SyntheticSample {
                id: self.next_id + i as u64,
                input: noise,
                expected_output: target,
                difficulty,
                source: format!("adversarial/weak/{}", i),
                quality_score: (1.0 - weakness) * 0.7 + 0.3,
            });
        }
        generated
    }

    pub fn sample_by_source(&self, source: &str) -> Vec<&SyntheticSample> {
        self.samples.iter().filter(|s| s.source == source).collect()
    }

    pub fn quality_distribution(&self) -> (f64, f64) {
        let n = self.samples.len();
        if n == 0 {
            return (0.0, 0.0);
        }
        let mean: f64 = self.samples.iter().map(|s| s.quality_score).sum::<f64>() / n as f64;
        let variance: f64 = self
            .samples
            .iter()
            .map(|s| (s.quality_score - mean).powi(2))
            .sum::<f64>()
            / n as f64;
        (mean, variance.sqrt())
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn estimate_improvement(&self) -> f64 {
        let n = self.samples.len();
        if n < 2 {
            return 0.0;
        }
        let mean_quality: f64 =
            self.samples.iter().map(|s| s.quality_score).sum::<f64>() / n as f64;
        let mean_difficulty: f64 =
            self.samples.iter().map(|s| s.difficulty).sum::<f64>() / n as f64;
        let total_score: f64 = self
            .samples
            .iter()
            .map(|s| s.quality_score * (1.0 - s.difficulty))
            .sum();
        let effective = total_score / n as f64;
        (effective - mean_quality * (1.0 - mean_difficulty))
            .abs()
            .min(1.0)
    }

    pub fn merge_flywheels(&mut self, other: DataFlywheel) {
        for sample in other.samples {
            if self.samples.len() >= self.max_samples {
                break;
            }
            let mut merged = sample;
            merged.id = self.next_id;
            self.next_id += 1;
            self.samples.push(merged);
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_success_rate() {
        let mut cl = ContinuousLearning::new();
        cl.record_experience("test", true, vec![1]);
        cl.record_experience("test", true, vec![2]);
        cl.record_experience("test", false, vec![3]);
        assert!((cl.success_rate("test") - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_weak_areas() {
        let mut cl = ContinuousLearning::new();
        cl.record_experience("hard", false, vec![]);
        cl.record_experience("hard", false, vec![]);
        cl.record_experience("hard", false, vec![]);
        cl.record_experience("easy", true, vec![]);
        cl.record_experience("easy", true, vec![]);
        cl.record_experience("easy", true, vec![]);
        let weak = cl.weak_areas(0.5);
        assert!(weak.contains(&"hard".to_string()));
        assert!(!weak.contains(&"easy".to_string()));
    }

    #[test]
    fn test_strong_areas() {
        let mut cl = ContinuousLearning::new();
        cl.record_experience("easy", true, vec![]);
        cl.record_experience("easy", true, vec![]);
        cl.record_experience("easy", true, vec![]);
        let strong = cl.strong_areas(0.8);
        assert!(strong.contains(&"easy".to_string()));
    }

    #[test]
    fn test_recent_experiences() {
        let mut cl = ContinuousLearning::new();
        for i in 0..10 {
            cl.record_experience(&format!("t{}", i), true, vec![]);
        }
        assert_eq!(cl.recent_experiences(3).len(), 3);
    }

    #[test]
    fn test_generate_synthetic() {
        let mut cl = ContinuousLearning::new();
        cl.record_experience("base", true, vec![]);
        cl.record_experience("base", true, vec![]);
        let syn = cl.generate_synthetic("base", 5);
        assert_eq!(syn.len(), 5);
    }

    #[test]
    fn test_report() {
        let cl = ContinuousLearning::new();
        let r = cl.report();
        assert!(r.contains("ContinuousLearning"));
    }

    #[test]
    fn test_total_learned() {
        let mut cl = ContinuousLearning::new();
        cl.record_experience("a", true, vec![]);
        cl.record_experience("b", true, vec![]);
        assert_eq!(cl.total_learned, 2);
    }

    #[test]
    fn test_synthetic_sample_new() {
        let s = SyntheticSample {
            id: 42,
            input: vec![0.1, 0.2, 0.3],
            expected_output: vec![0.5, 0.6],
            difficulty: 0.7,
            source: "math".into(),
            quality_score: 0.85,
        };
        assert_eq!(s.id, 42);
        assert!((s.difficulty - 0.7).abs() < 0.001);
        assert_eq!(s.source, "math");
    }

    #[test]
    fn test_generate_from_gaps() {
        let mut fw = DataFlywheel::new(FlywheelStrategy::Curriculum, 100);
        let base = vec![0.5; 8];
        let gaps = &[("algebra", 0.3), ("geometry", 0.6)];
        let count = fw.generate_from_gaps(gaps, &base, 10);
        assert_eq!(count, 10);
        assert_eq!(fw.sample_count(), 10);
        assert!(fw.samples.iter().all(|s| !s.input.is_empty()));
    }

    #[test]
    fn test_curriculum_order_ascending() {
        let mut fw = DataFlywheel::new(FlywheelStrategy::Curriculum, 50);
        let base = vec![0.5; 4];
        let gaps = &[("topic", 0.5)];
        fw.generate_from_gaps(gaps, &base, 5);
        let ordered = fw.curriculum_order();
        for w in ordered.windows(2) {
            assert!(w[0].difficulty <= w[1].difficulty + 0.001);
        }
        assert_eq!(ordered.len(), 5);
    }

    #[test]
    fn test_adversarial_samples() {
        let fw = DataFlywheel::new(FlywheelStrategy::Adversarial, 50);
        let weak = vec![0.2, 0.3, 0.4];
        let adv = fw.adversarial_samples(&weak, 5);
        assert_eq!(adv.len(), 5);
        assert!(adv.iter().all(|s| s.difficulty > 0.5));
    }

    #[test]
    fn test_sample_by_source() {
        let mut fw = DataFlywheel::new(FlywheelStrategy::Diverse, 50);
        let base = vec![0.5; 4];
        fw.generate_from_gaps(&[("physics", 0.6)], &base, 3);
        let matched = fw.sample_by_source("physics");
        assert_eq!(matched.len(), 3);
        let no_match = fw.sample_by_source("nonexistent");
        assert!(no_match.is_empty());
    }

    #[test]
    fn test_quality_distribution() {
        let mut fw = DataFlywheel::new(FlywheelStrategy::Curriculum, 50);
        let base = vec![0.5; 4];
        fw.generate_from_gaps(&[("test", 0.8)], &base, 10);
        let (mean, std) = fw.quality_distribution();
        assert!(mean > 0.0);
        assert!(std >= 0.0);
    }

    #[test]
    fn test_sample_count() {
        let fw = DataFlywheel::new(FlywheelStrategy::Adaptive, 50);
        assert_eq!(fw.sample_count(), 0);
    }

    #[test]
    fn test_estimate_improvement() {
        let mut fw = DataFlywheel::new(FlywheelStrategy::Adaptive, 50);
        let base = vec![0.5; 4];
        fw.generate_from_gaps(&[("improve_me", 0.8)], &base, 5);
        let improvement = fw.estimate_improvement();
        assert!(improvement >= 0.0);
        assert!(improvement <= 1.0);
    }

    #[test]
    fn test_merge_flywheels() {
        let mut fw1 = DataFlywheel::new(FlywheelStrategy::Curriculum, 100);
        let base = vec![0.5; 4];
        fw1.generate_from_gaps(&[("src_a", 0.7)], &base, 3);
        let mut fw2 = DataFlywheel::new(FlywheelStrategy::Diverse, 100);
        fw2.generate_from_gaps(&[("src_b", 0.5)], &base, 2);
        let pre = fw1.sample_count();
        fw1.merge_flywheels(fw2);
        assert_eq!(fw1.sample_count(), pre + 2);
    }

    #[test]
    fn test_flywheel_strategy_defaults() {
        let curriculum = DataFlywheel::new(FlywheelStrategy::Curriculum, 10);
        assert_eq!(curriculum.generation_strategy, FlywheelStrategy::Curriculum);
        assert_eq!(curriculum.max_samples, 10);
        assert_eq!(curriculum.sample_count(), 0);

        let adversarial = DataFlywheel::new(FlywheelStrategy::Adversarial, 20);
        assert_eq!(
            adversarial.generation_strategy,
            FlywheelStrategy::Adversarial
        );

        let diverse = DataFlywheel::new(FlywheelStrategy::Diverse, 30);
        assert_eq!(diverse.generation_strategy, FlywheelStrategy::Diverse);

        let adaptive = DataFlywheel::new(FlywheelStrategy::Adaptive, 40);
        assert_eq!(adaptive.generation_strategy, FlywheelStrategy::Adaptive);
    }
}
