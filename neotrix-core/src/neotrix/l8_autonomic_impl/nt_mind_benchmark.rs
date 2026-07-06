use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Instant;
use crate::core::CapabilityVector;
use crate::neotrix::nt_world_model::TaskType;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::ReasoningBrain;
use crate::core::nt_core_knowledge::KnowledgeSource;
use crate::core::nt_core_bank::ReasoningMemory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub category: String,
    pub score: f64,
    pub max_score: f64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub results: Vec<BenchmarkResult>,
    pub overall_score: f64,
    pub timestamp: String,
    pub iteration: u64,
}

pub struct BenchmarkSuite;

impl BenchmarkSuite {
    pub fn run_all(cap: &CapabilityVector) -> BenchmarkReport {
        let mut results = vec![
            BenchmarkResult {
                name: "general_intelligence".into(),
                category: "core".into(),
                score: cap.arr().iter().sum::<f64>() / cap.arr().len() as f64,
                max_score: 1.0,
                metadata: None,
            },
            BenchmarkResult {
                name: "quality_gates".into(),
                category: "core".into(),
                score: cap.quality_gates(),
                max_score: 1.0,
                metadata: None,
            },
            BenchmarkResult {
                name: "extension_diversity".into(),
                category: "knowledge".into(),
                score: (cap.extension.len() as f64).min(22.0) / 22.0,
                max_score: 1.0,
                metadata: None,
            },
        ];

        for (name, tt) in &[
            ("design", TaskType::Design),
            ("code_analysis", TaskType::CodeAnalysis),
            ("code_review", TaskType::CodeReview),
            ("nt_shield", TaskType::Security),
        ] {
            let score = crate::neotrix::nt_mind::core::PerformanceEvaluator::evaluate(tt, cap);
            results.push(BenchmarkResult {
                name: format!("task_{}", name),
                category: "task".into(),
                score,
                max_score: 1.0,
                metadata: None,
            });
        }

        let overall = results.iter().map(|r| r.score / r.max_score).sum::<f64>() / results.len() as f64;

        BenchmarkReport {
            overall_score: overall,
            timestamp: chrono::Utc::now().to_rfc3339(),
            results,
            iteration: 0,
        }
    }

    pub fn run_category(cap: &CapabilityVector, category: &str) -> Vec<BenchmarkResult> {
        let report = Self::run_all(cap);
        report.results.into_iter().filter(|r| r.category == category).collect()
    }

    pub fn run_all_extended(cap: &CapabilityVector, bank: &mut ReasoningBank) -> BenchmarkReport {
        let mut base = Self::run_all(cap);

        let knowledge = Self::run_knowledge_benchmarks(cap);
        let memory = Self::run_memory_benchmarks(bank);
        let convergence = Self::run_convergence_benchmarks(cap);

        base.results.extend(knowledge);
        base.results.extend(memory);
        base.results.extend(convergence);

        let overall = base.results.iter().map(|r| r.score / r.max_score).sum::<f64>() / base.results.len() as f64;
        base.overall_score = overall;
        base
    }

    pub fn run_knowledge_benchmarks(cap: &CapabilityVector) -> Vec<BenchmarkResult> {
        use KnowledgeSource::*;
        let sources = vec![
            HeroUI, BaseUI, ArcUI, CortexUI, AgenticDS, DesignPhilosophy,
            Hyperframes, Betterleaks, YaoWebsecurity, Botasaurus, ReactDoctor,
            OpenPencil, AiTrader, SesameRobot, EverOS, MattPocockSkills,
            NestedLearning, AutonomousGoal, AwesomeDesignSkills,
            DeepSeekTui, Codebuff, OpenClaude, Cairn, Orca, RedRun,
            AutonomousSpeedrunning, Synesis, MemOS, Reflexio, Mem0,
            Mnemosyne, OriMnemos, OPSD,
        ];

        let non_zero = sources.iter().filter(|s: &&KnowledgeSource| {
            s.capability_vector().arr().iter().any(|&v| v > 0.0)
        }).count();
        let coverage = non_zero as f64 / sources.len() as f64;

        let total: f64 = cap.arr().iter().sum();
        let entropy = if total > 0.0 {
            -cap.arr().iter().filter(|&&v| v > 0.0).map(|&v| {
                let p = v / total;
                p * p.log2()
            }).sum::<f64>()
        } else {
            0.0
        };
        let max_entropy = (cap.arr().len() as f64).log2();
        let diversity = if max_entropy > 0.0 { (entropy / max_entropy).min(1.0) } else { 0.0 };

        let richness = (cap.extension.len() as f64 / 50.0).min(1.0);

        vec![
            BenchmarkResult {
                name: "knowledge_coverage".into(),
                category: "knowledge".into(),
                score: coverage,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("total_sources".into(), sources.len().to_string()),
                    ("non_zero_sources".into(), non_zero.to_string()),
                ])),
            },
            BenchmarkResult {
                name: "knowledge_diversity".into(),
                category: "knowledge".into(),
                score: diversity,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("entropy".into(), format!("{:.4}", entropy)),
                    ("max_entropy".into(), format!("{:.4}", max_entropy)),
                ])),
            },
            BenchmarkResult {
                name: "extension_richness".into(),
                category: "knowledge".into(),
                score: richness,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("extension_count".into(), cap.extension.len().to_string()),
                ])),
            },
        ]
    }

    pub fn run_memory_benchmarks(bank: &mut ReasoningBank) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        let memory_retention_result = {
            let count_before = bank.stats().total_memories;
            let test_memories: Vec<ReasoningMemory> = (0..5).map(|i| {
                ReasoningMemory::new(
                    &format!("benchmark_retention_test_{}", i),
                    TaskType::General,
                    &[],
                    0.8,
                )
            }).collect();

            for mem in &test_memories {
                bank.store(mem.clone());
            }

            let recalled = test_memories.iter().filter(|m| {
                let retrieved = bank.retrieve_relevant(&m.task_description, None, 10);
                retrieved.iter().any(|r| r.id == m.id)
            }).count();
            let retention = if test_memories.is_empty() { 0.0 } else { recalled as f64 / test_memories.len() as f64 };

            BenchmarkResult {
                name: "memory_retention".into(),
                category: "memory".into(),
                score: retention,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("stored".into(), test_memories.len().to_string()),
                    ("recalled".into(), recalled.to_string()),
                    ("count_before".into(), count_before.to_string()),
                ])),
            }
        };
        results.push(memory_retention_result);

        let memory_capacity_result = {
            let fill_count = 200usize;
            let mut panic_occurred = false;
            let _before_stats = bank.stats().total_memories;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for i in 0..fill_count {
                    let mem = ReasoningMemory::new(
                        &format!("benchmark_capacity_fill_{}", i),
                        TaskType::General,
                        &[],
                        0.1,
                    );
                    bank.store(mem);
                }
                let _ = bank.stats();
            }));
            if result.is_err() {
                panic_occurred = true;
            }

            BenchmarkResult {
                name: "memory_capacity".into(),
                category: "memory".into(),
                score: if panic_occurred { 0.0 } else { 1.0 },
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("panic".into(), panic_occurred.to_string()),
                    ("fill_attempted".into(), fill_count.to_string()),
                    ("total_after".into(), bank.stats().total_memories.to_string()),
                ])),
            }
        };
        results.push(memory_capacity_result);

        let memory_retrieval_speed_result = {
            let start = Instant::now();
            let trials = 5;
            for _ in 0..trials {
                let _ = bank.retrieve_relevant("benchmark speed test query", None, 5);
            }
            let elapsed = start.elapsed();
            let avg = elapsed / trials as u32;
            let speed_score = if avg.as_micros() < 1000 {
                1.0
            } else if avg.as_micros() < 5000 {
                0.8
            } else if avg.as_micros() < 10000 {
                0.5
            } else {
                0.2
            };

            BenchmarkResult {
                name: "memory_retrieval_speed".into(),
                category: "memory".into(),
                score: speed_score,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("avg_micros".into(), avg.as_micros().to_string()),
                    ("trials".into(), trials.to_string()),
                ])),
            }
        };
        results.push(memory_retrieval_speed_result);

        results
    }

    pub fn run_convergence_benchmarks(cap: &CapabilityVector) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        let vector_stability_result = {
            let mut brain = ReasoningBrain::new();
            brain.capability = cap.clone();

            let absorb_sources = [KnowledgeSource::HeroUI,
                KnowledgeSource::BaseUI,
                KnowledgeSource::ArcUI,
                KnowledgeSource::CortexUI,
                KnowledgeSource::AgenticDS];

            let mut snapshots: Vec<Vec<f64>> = Vec::new();
            for i in 0..10 {
                brain.absorb(absorb_sources[i % absorb_sources.len()]);
                snapshots.push(brain.capability.arr.clone());
            }

            let n = snapshots.len() as f64;
            let per_dim_variance: Vec<f64> = if n > 0.0 {
                (0..snapshots[0].len()).map(|dim| {
                    let mean: f64 = snapshots.iter().map(|s| s[dim]).sum::<f64>() / n;
                    snapshots.iter().map(|s| (s[dim] - mean).powi(2)).sum::<f64>() / n
                }).collect()
            } else {
                vec![0.0]
            };
            let avg_variance = if per_dim_variance.is_empty() {
                0.0
            } else {
                per_dim_variance.iter().sum::<f64>() / per_dim_variance.len() as f64
            };
            let stability = 1.0 / (1.0 + avg_variance * 100.0);

            BenchmarkResult {
                name: "vector_stability".into(),
                category: "convergence".into(),
                score: stability,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("avg_variance".into(), format!("{:.6}", avg_variance)),
                    ("iterations".into(), "10".into()),
                    ("dimensions".into(), per_dim_variance.len().to_string()),
                ])),
            }
        };
        results.push(vector_stability_result);

        let absorption_efficiency_result = {
            let mut brain = ReasoningBrain::new();
            brain.capability = cap.clone();

            let sources = [KnowledgeSource::DesignPhilosophy,
                KnowledgeSource::Hyperframes,
                KnowledgeSource::Betterleaks,
                KnowledgeSource::ReactDoctor,
                KnowledgeSource::OpenPencil];

            let start = Instant::now();
            for i in 0..10 {
                brain.absorb(sources[i % sources.len()]);
            }
            let elapsed = start.elapsed();
            let avg_us = elapsed.as_micros() as f64 / 10.0;
            let efficiency = if avg_us < 50.0 {
                1.0
            } else if avg_us < 200.0 {
                0.8
            } else if avg_us < 500.0 {
                0.5
            } else {
                0.2
            };

            BenchmarkResult {
                name: "absorption_efficiency".into(),
                category: "convergence".into(),
                score: efficiency,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("total_micros".into(), elapsed.as_micros().to_string()),
                    ("avg_micros".into(), format!("{:.1}", avg_us)),
                ])),
            }
        };
        results.push(absorption_efficiency_result);

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_cap() -> CapabilityVector {
        let mut cap = CapabilityVector::default();
        for v in cap.arr_mut().iter_mut() {
            *v = 0.5;
        }
        cap
    }

    #[test]
    fn test_benchmark_runs_without_panic() {
        let cap = CapabilityVector::default();
        let report = BenchmarkSuite::run_all(&cap);
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 1.0);
        assert!(!report.results.is_empty());
    }

    #[test]
    fn test_benchmark_category_filter() {
        let cap = CapabilityVector::default();
        let core = BenchmarkSuite::run_category(&cap, "core");
        assert!(!core.is_empty());
        let task = BenchmarkSuite::run_category(&cap, "task");
        assert_eq!(task.len(), 4);
    }

    #[test]
    fn test_knowledge_benchmarks() {
        let cap = make_test_cap();
        let results = BenchmarkSuite::run_knowledge_benchmarks(&cap);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.category, "knowledge");
            assert!(r.score >= 0.0 && r.score <= 1.0);
            assert!(r.metadata.is_some());
        }
        let coverage = results.iter().find(|r| r.name == "knowledge_coverage").expect("knowledge_coverage result should be present");
        assert!(coverage.score > 0.0);
        let diversity = results.iter().find(|r| r.name == "knowledge_diversity").expect("knowledge_diversity result should be present");
        assert!(diversity.score >= 0.0);
    }

    #[test]
    fn test_memory_retention() {
        let mut bank = ReasoningBank::new(100);
        let results = BenchmarkSuite::run_memory_benchmarks(&mut bank);
        let retention = results.iter().find(|r| r.name == "memory_retention").expect("memory_retention result should be present");
        assert_eq!(retention.category, "memory");
        assert!(retention.score >= 0.0 && retention.score <= 1.0);
    }

    #[test]
    fn test_memory_capacity() {
        let mut bank = ReasoningBank::new(100);
        let results = BenchmarkSuite::run_memory_benchmarks(&mut bank);
        let capacity = results.iter().find(|r| r.name == "memory_capacity").expect("memory_capacity result should be present");
        assert_eq!(capacity.max_score, 1.0);
        let meta = capacity.metadata.as_ref().expect("memory_capacity should have metadata");
        assert_eq!(meta.get("panic").expect("metadata should contain 'panic' key"), "false");
    }

    #[test]
    fn test_vector_stability() {
        let cap = make_test_cap();
        let results = BenchmarkSuite::run_convergence_benchmarks(&cap);
        let stability = results.iter().find(|r| r.name == "vector_stability").expect("vector_stability result should be present");
        assert_eq!(stability.category, "convergence");
        assert!(stability.score > 0.0 && stability.score <= 1.0);
    }

    #[test]
    fn test_benchmark_report_format() {
        let cap = make_test_cap();
        let mut bank = ReasoningBank::new(100);
        let report = BenchmarkSuite::run_all_extended(&cap, &mut bank);
        assert!(!report.results.is_empty());
        assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
        assert!(!report.timestamp.is_empty());
        let categories: std::collections::HashSet<&str> = report.results.iter().map(|r| r.category.as_str()).collect();
        assert!(categories.contains("core"));
        assert!(categories.contains("task"));
        assert!(categories.contains("knowledge"));
        assert!(categories.contains("memory"));
        assert!(categories.contains("convergence"));
    }

    #[test]
    fn test_backward_compatible_run_all() {
        let cap = make_test_cap();
        let report = BenchmarkSuite::run_all(&cap);
        assert_eq!(report.results.len(), 7);
        for r in &report.results {
            assert!(r.metadata.is_none());
        }
    }
}
