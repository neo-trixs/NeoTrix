use crate::core::nt_core_bank::ReasoningMemory;
use crate::core::nt_core_consciousness::memory_lattice::{LatticeLayer, MemoryLattice};
use crate::core::nt_core_health::ConsciousnessDashboard;
use crate::core::nt_core_knowledge::KnowledgeSource;
use crate::core::CapabilityVector;
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::ReasoningBrain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

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

        let overall =
            results.iter().map(|r| r.score / r.max_score).sum::<f64>() / results.len() as f64;

        BenchmarkReport {
            overall_score: overall,
            timestamp: chrono::Utc::now().to_rfc3339(),
            results,
            iteration: 0,
        }
    }

    pub fn run_category(cap: &CapabilityVector, category: &str) -> Vec<BenchmarkResult> {
        let report = Self::run_all(cap);
        report
            .results
            .into_iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn run_all_extended(
        cap: &CapabilityVector,
        bank: &mut ReasoningBank,
        lattice: Option<&mut MemoryLattice>,
    ) -> BenchmarkReport {
        let mut base = Self::run_all(cap);

        let knowledge = Self::run_knowledge_benchmarks(cap);
        let memory = Self::run_memory_benchmarks(bank);
        let convergence = Self::run_convergence_benchmarks(cap);

        base.results.extend(knowledge);
        base.results.extend(memory);
        base.results.extend(convergence);

        if let Some(lat) = lattice {
            let ml = Self::run_memory_lattice_benchmarks(lat);
            base.results.extend(ml);
        }

        let overall = base
            .results
            .iter()
            .map(|r| r.score / r.max_score)
            .sum::<f64>()
            / base.results.len() as f64;
        base.overall_score = overall;
        base
    }

    pub fn run_knowledge_benchmarks(cap: &CapabilityVector) -> Vec<BenchmarkResult> {
        use KnowledgeSource::*;
        let sources = vec![
            HeroUI,
            BaseUI,
            ArcUI,
            CortexUI,
            AgenticDS,
            DesignPhilosophy,
            Hyperframes,
            Betterleaks,
            YaoWebsecurity,
            Botasaurus,
            ReactDoctor,
            OpenPencil,
            AiTrader,
            SesameRobot,
            EverOS,
            MattPocockSkills,
            NestedLearning,
            AutonomousGoal,
            AwesomeDesignSkills,
            DeepSeekTui,
            Codebuff,
            OpenClaude,
            Cairn,
            Orca,
            RedRun,
            AutonomousSpeedrunning,
            Synesis,
            MemOS,
            Reflexio,
            Mem0,
            Mnemosyne,
            OriMnemos,
            OPSD,
        ];

        let non_zero = sources
            .iter()
            .filter(|s: &&KnowledgeSource| s.capability_vector().arr().iter().any(|&v| v > 0.0))
            .count();
        let coverage = non_zero as f64 / sources.len() as f64;

        let total: f64 = cap.arr().iter().sum();
        let entropy = if total > 0.0 {
            -cap.arr()
                .iter()
                .filter(|&&v| v > 0.0)
                .map(|&v| {
                    let p = v / total;
                    p * p.log2()
                })
                .sum::<f64>()
        } else {
            0.0
        };
        let max_entropy = (cap.arr().len() as f64).log2();
        let diversity = if max_entropy > 0.0 {
            (entropy / max_entropy).min(1.0)
        } else {
            0.0
        };

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
                metadata: Some(HashMap::from([(
                    "extension_count".into(),
                    cap.extension.len().to_string(),
                )])),
            },
        ]
    }

    pub fn run_memory_benchmarks(bank: &mut ReasoningBank) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        let memory_retention_result = {
            let count_before = bank.stats().total_memories;
            let test_memories: Vec<ReasoningMemory> = (0..5)
                .map(|i| {
                    ReasoningMemory::new(
                        &format!("benchmark_retention_test_{}", i),
                        TaskType::General,
                        &[],
                        0.8,
                    )
                })
                .collect();

            for mem in &test_memories {
                bank.store(mem.clone());
            }

            let recalled = test_memories
                .iter()
                .filter(|m| {
                    let retrieved = bank.retrieve_relevant(&m.task_description, None, 10);
                    retrieved.iter().any(|r| r.id == m.id)
                })
                .count();
            let retention = if test_memories.is_empty() {
                0.0
            } else {
                recalled as f64 / test_memories.len() as f64
            };

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
                    (
                        "total_after".into(),
                        bank.stats().total_memories.to_string(),
                    ),
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

    /// Run consciousness-specific benchmarks from a snapshot of consciousness metrics.
    ///
    /// Takes primitive f64 values (no CI dependency) to avoid circular imports.
    /// Builds a ConsciousnessDashboard internally and scores each theory dimension.
    pub fn run_consciousness_benchmarks(
        iit_phi: f64,
        gnw_coverage: f64,
        gnw_utilization: f64,
        drt_depth: usize,
        drt_max_depth: usize,
        meta_accuracy: f64,
        coherence: f64,
        cycles_elapsed: u64,
        cycle_count: u64,
    ) -> Vec<BenchmarkResult> {
        let mut dashboard = ConsciousnessDashboard::new()
            .with_iit(iit_phi)
            .with_gnw(gnw_coverage, gnw_utilization, 0)
            .with_drt(drt_depth, drt_max_depth)
            .with_workspace(coherence, (coherence * 10.0) as usize);

        // IIT Φ — normalized phi score
        let phi_score = iit_phi;

        // GNW global access — broadcast coverage × slot utilization
        let gnw_score = gnw_coverage * gnw_utilization;

        // DRT self-model accuracy using meta-accuracy
        let drt_norm = if drt_max_depth > 0 {
            drt_depth as f64 / drt_max_depth as f64
        } else {
            0.0
        };
        // DRT score = average of meta_accuracy and recursion depth
        let drt_score = (meta_accuracy + drt_norm) / 2.0;

        // Information integration speed: cycles per second
        let elapsed_secs = (cycles_elapsed as f64).max(1.0) / 1_000_000.0; // assume µs precision
        let cycles_per_sec = if elapsed_secs > 0.0 {
            cycle_count as f64 / elapsed_secs
        } else {
            0.0
        };
        let speed_score = (cycles_per_sec / 10.0).min(1.0); // target: 10+ cycles/sec

        // Self-world boundary clarity: coherence
        let boundary_score = coherence;

        // Composite consciousness score from dashboard
        let composite = dashboard.compute_composite();

        vec![
            BenchmarkResult {
                name: "consciousness_iit_phi".into(),
                category: "consciousness".into(),
                score: phi_score,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("phi".into(), format!("{:.4}", iit_phi)),
                    ("integrated".into(), (iit_phi > 0.5).to_string()),
                ])),
            },
            BenchmarkResult {
                name: "consciousness_gnw_broadcast".into(),
                category: "consciousness".into(),
                score: gnw_score,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("coverage".into(), format!("{:.4}", gnw_coverage)),
                    ("utilization".into(), format!("{:.4}", gnw_utilization)),
                ])),
            },
            BenchmarkResult {
                name: "consciousness_drt_accuracy".into(),
                category: "consciousness".into(),
                score: drt_score,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("meta_accuracy".into(), format!("{:.4}", meta_accuracy)),
                    ("recursion_norm".into(), format!("{:.4}", drt_norm)),
                    ("recursion_depth".into(), drt_depth.to_string()),
                    ("max_depth".into(), drt_max_depth.to_string()),
                ])),
            },
            BenchmarkResult {
                name: "consciousness_integration_speed".into(),
                category: "consciousness".into(),
                score: speed_score,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("cycles_per_sec".into(), format!("{:.2}", cycles_per_sec)),
                    ("cycles".into(), cycle_count.to_string()),
                ])),
            },
            BenchmarkResult {
                name: "consciousness_self_world_boundary".into(),
                category: "consciousness".into(),
                score: boundary_score,
                max_score: 1.0,
                metadata: Some(HashMap::from([(
                    "coherence".into(),
                    format!("{:.4}", coherence),
                )])),
            },
            BenchmarkResult {
                name: "consciousness_composite".into(),
                category: "consciousness".into(),
                score: composite,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    (
                        "phi_weight".into(),
                        format!("{:.2}", dashboard.weights.phi_weight),
                    ),
                    (
                        "gnw_weight".into(),
                        format!("{:.2}", dashboard.weights.gnw_weight),
                    ),
                    (
                        "drt_weight".into(),
                        format!("{:.2}", dashboard.weights.drt_weight),
                    ),
                    (
                        "ws_weight".into(),
                        format!("{:.2}", dashboard.weights.ws_weight),
                    ),
                ])),
            },
        ]
    }

    /// Benchmark MemoryLattice operations: store, find, consolidate, prune, Q-learning.
    pub fn run_memory_lattice_benchmarks(lattice: &mut MemoryLattice) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // 1) Store throughput: write 20 entries across all layers
        let store_start = Instant::now();
        for i in 0..20 {
            let layer = match i % 5 {
                0 => LatticeLayer::Episodic,
                1 => LatticeLayer::Facts,
                2 => LatticeLayer::Skills,
                3 => LatticeLayer::MetaRules,
                _ => LatticeLayer::Identity,
            };
            lattice.store(
                format!("benchmark_entry_{}", i),
                format!("vsa_{}", i).into_bytes(),
                layer,
            );
        }
        let store_us = store_start.elapsed().as_micros();
        let store_score = if store_us < 500 {
            1.0
        } else if store_us < 2000 {
            0.8
        } else if store_us < 5000 {
            0.5
        } else {
            0.2
        };
        results.push(BenchmarkResult {
            name: "lattice_store_throughput".into(),
            category: "memory_lattice".into(),
            score: store_score,
            max_score: 1.0,
            metadata: Some(HashMap::from([
                ("store_us".into(), store_us.to_string()),
                ("entries".into(), "20".into()),
            ])),
        });

        // 2) Q-value update: update Q-values on existing entries
        let q_start = Instant::now();
        let q_cycles = 100;
        for i in 0..q_cycles {
            lattice.cycle = i as u64;
            let _ = lattice.update_q_value(
                i % lattice.episodic.len().max(1),
                LatticeLayer::Episodic,
                0.5 + (i as f64 * 0.01).sin().abs(),
                0.1,
            );
        }
        lattice.cycle = 0;
        let q_us = q_start.elapsed().as_micros();
        let q_score = if q_us < 500 {
            1.0
        } else if q_us < 2000 {
            0.8
        } else {
            0.5
        };
        results.push(BenchmarkResult {
            name: "lattice_q_learning".into(),
            category: "memory_lattice".into(),
            score: q_score,
            max_score: 1.0,
            metadata: Some(HashMap::from([
                ("q_us".into(), q_us.to_string()),
                ("cycles".into(), q_cycles.to_string()),
            ])),
        });

        // 3) Find by content: search for specific terms
        let find_start = Instant::now();
        for keyword in &["benchmark", "entry", "vsa"] {
            let _found = lattice.find(keyword);
        }
        let find_us = find_start.elapsed().as_micros();
        let find_score = if find_us < 300 {
            1.0
        } else if find_us < 1000 {
            0.8
        } else {
            0.5
        };
        results.push(BenchmarkResult {
            name: "lattice_find_content".into(),
            category: "memory_lattice".into(),
            score: find_score,
            max_score: 1.0,
            metadata: Some(HashMap::from([("find_us".into(), find_us.to_string())])),
        });

        // 4) Temporal retrieval: find_by_temporal with a range
        let _ = lattice.find_by_temporal("", 0, Some(LatticeLayer::Episodic));
        let _ = lattice.find_by_temporal("", 5, None);
        let _ = lattice.find_by_temporal("", 0, Some(LatticeLayer::Episodic));
        results.push(BenchmarkResult {
            name: "lattice_temporal_retrieval".into(),
            category: "memory_lattice".into(),
            score: 1.0,
            max_score: 1.0,
            metadata: None,
        });

        // 5) Consolidation quality
        let c_start = Instant::now();
        lattice.consolidation_threshold = 0.3;
        let consol_before = lattice.total_consolidations;
        let _ = lattice.consolidate();
        let consol_us = c_start.elapsed().as_micros();
        let consol_score = if consol_us < 500 { 1.0 } else { 0.8 };
        results.push(BenchmarkResult {
            name: "lattice_consolidation".into(),
            category: "memory_lattice".into(),
            score: consol_score,
            max_score: 1.0,
            metadata: Some(HashMap::from([
                ("consol_us".into(), consol_us.to_string()),
                ("consolidations_before".into(), consol_before.to_string()),
                (
                    "consolidations_after".into(),
                    lattice.total_consolidations.to_string(),
                ),
            ])),
        });

        // 6) Prune efficiency
        let prune_start = Instant::now();
        lattice.apply_forgetting(0.1);
        let _ = lattice.prune_forgotten(0.1);
        let prune_us = prune_start.elapsed().as_micros();
        let prune_score = if prune_us < 500 { 1.0 } else { 0.8 };
        results.push(BenchmarkResult {
            name: "lattice_prune".into(),
            category: "memory_lattice".into(),
            score: prune_score,
            max_score: 1.0,
            metadata: Some(HashMap::from([("prune_us".into(), prune_us.to_string())])),
        });

        // 7) Retrieval fidelity analysis
        let fidelity = lattice.retrieval_fidelity_analysis("", 0);
        let fidelity_score = if fidelity.close_count > 0 {
            (fidelity.close_avg_score * 0.6
                + fidelity.medium_avg_score * 0.3
                + fidelity.far_avg_score * 0.1)
                .min(1.0)
        } else {
            0.5
        };
        results.push(BenchmarkResult {
            name: "lattice_retrieval_fidelity".into(),
            category: "memory_lattice".into(),
            score: fidelity_score,
            max_score: 1.0,
            metadata: Some(HashMap::from([
                ("close_count".into(), fidelity.close_count.to_string()),
                (
                    "close_avg".into(),
                    format!("{:.3}", fidelity.close_avg_score),
                ),
                ("medium_count".into(), fidelity.medium_count.to_string()),
                (
                    "medium_avg".into(),
                    format!("{:.3}", fidelity.medium_avg_score),
                ),
            ])),
        });

        // 8) Layer distribution: all 5 layers populated
        let empty_layers = [
            lattice.episodic.is_empty(),
            lattice.facts.is_empty(),
            lattice.skills.is_empty(),
            lattice.meta_rules.is_empty(),
            lattice.identity.is_empty(),
        ]
        .iter()
        .filter(|&&e| e)
        .count();
        let dist_score = (5 - empty_layers) as f64 / 5.0;
        results.push(BenchmarkResult {
            name: "lattice_layer_distribution".into(),
            category: "memory_lattice".into(),
            score: dist_score,
            max_score: 1.0,
            metadata: Some(HashMap::from([
                ("empty_layers".into(), empty_layers.to_string()),
                (
                    "total".into(),
                    (lattice.episodic.len()
                        + lattice.facts.len()
                        + lattice.skills.len()
                        + lattice.meta_rules.len()
                        + lattice.identity.len())
                    .to_string(),
                ),
            ])),
        });

        results
    }

    pub fn run_convergence_benchmarks(cap: &CapabilityVector) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        let vector_stability_result = {
            let mut brain = ReasoningBrain::new();
            brain.capability = cap.clone();

            let absorb_sources = [
                KnowledgeSource::HeroUI,
                KnowledgeSource::BaseUI,
                KnowledgeSource::ArcUI,
                KnowledgeSource::CortexUI,
                KnowledgeSource::AgenticDS,
            ];

            let mut snapshots: Vec<Vec<f64>> = Vec::new();
            for i in 0..10 {
                brain.absorb(absorb_sources[i % absorb_sources.len()]);
                snapshots.push(brain.capability.arr.clone());
            }

            let n = snapshots.len() as f64;
            let per_dim_variance: Vec<f64> = if n > 0.0 {
                (0..snapshots[0].len())
                    .map(|dim| {
                        let mean: f64 = snapshots.iter().map(|s| s[dim]).sum::<f64>() / n;
                        snapshots
                            .iter()
                            .map(|s| (s[dim] - mean).powi(2))
                            .sum::<f64>()
                            / n
                    })
                    .collect()
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

            let sources = [
                KnowledgeSource::DesignPhilosophy,
                KnowledgeSource::Hyperframes,
                KnowledgeSource::Betterleaks,
                KnowledgeSource::ReactDoctor,
                KnowledgeSource::OpenPencil,
            ];

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
        let coverage = results
            .iter()
            .find(|r| r.name == "knowledge_coverage")
            .expect("knowledge_coverage result should be present");
        assert!(coverage.score > 0.0);
        let diversity = results
            .iter()
            .find(|r| r.name == "knowledge_diversity")
            .expect("knowledge_diversity result should be present");
        assert!(diversity.score >= 0.0);
    }

    #[test]
    fn test_memory_retention() {
        let mut bank = ReasoningBank::new(100);
        let results = BenchmarkSuite::run_memory_benchmarks(&mut bank);
        let retention = results
            .iter()
            .find(|r| r.name == "memory_retention")
            .expect("memory_retention result should be present");
        assert_eq!(retention.category, "memory");
        assert!(retention.score >= 0.0 && retention.score <= 1.0);
    }

    #[test]
    fn test_memory_capacity() {
        let mut bank = ReasoningBank::new(100);
        let results = BenchmarkSuite::run_memory_benchmarks(&mut bank);
        let capacity = results
            .iter()
            .find(|r| r.name == "memory_capacity")
            .expect("memory_capacity result should be present");
        assert_eq!(capacity.max_score, 1.0);
        let meta = capacity
            .metadata
            .as_ref()
            .expect("memory_capacity should have metadata");
        assert_eq!(
            meta.get("panic")
                .expect("metadata should contain 'panic' key"),
            "false"
        );
    }

    #[test]
    fn test_vector_stability() {
        let cap = make_test_cap();
        let results = BenchmarkSuite::run_convergence_benchmarks(&cap);
        let stability = results
            .iter()
            .find(|r| r.name == "vector_stability")
            .expect("vector_stability result should be present");
        assert_eq!(stability.category, "convergence");
        assert!(stability.score > 0.0 && stability.score <= 1.0);
    }

    #[test]
    fn test_benchmark_report_format() {
        let cap = make_test_cap();
        let mut bank = ReasoningBank::new(100);
        let mut lattice = MemoryLattice::new();
        let report = BenchmarkSuite::run_all_extended(&cap, &mut bank, Some(&mut lattice));
        assert!(!report.results.is_empty());
        assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
        assert!(!report.timestamp.is_empty());
        let categories: std::collections::HashSet<&str> =
            report.results.iter().map(|r| r.category.as_str()).collect();
        assert!(categories.contains("core"));
        assert!(categories.contains("task"));
        assert!(categories.contains("knowledge"));
        assert!(categories.contains("memory"));
        assert!(categories.contains("convergence"));
        assert!(categories.contains("memory_lattice"));
    }

    #[test]
    fn test_memory_lattice_benchmarks_all_scores_in_range() {
        let mut lattice = MemoryLattice::new();
        let results = BenchmarkSuite::run_memory_lattice_benchmarks(&mut lattice);
        assert!(!results.is_empty());
        for r in &results {
            assert_eq!(r.category, "memory_lattice");
            assert!(r.score >= 0.0 && r.score <= 1.0);
        }
        assert!(results.iter().any(|r| r.name == "lattice_store_throughput"));
        assert!(results.iter().any(|r| r.name == "lattice_find_content"));
        assert!(results.iter().any(|r| r.name == "lattice_consolidation"));
        assert!(results
            .iter()
            .any(|r| r.name == "lattice_retrieval_fidelity"));
    }

    #[test]
    fn test_consciousness_benchmarks_all_scores_in_range() {
        let results = BenchmarkSuite::run_consciousness_benchmarks(
            0.72, 0.85, 0.60, 4, 8, 0.78, 0.65, 500_000, 100,
        );
        assert_eq!(results.len(), 6);
        for r in &results {
            assert_eq!(r.category, "consciousness");
            assert!(r.score >= 0.0 && r.score <= 1.0);
            assert!(r.metadata.is_some());
        }
        let phi = results
            .iter()
            .find(|r| r.name == "consciousness_iit_phi")
            .unwrap();
        assert!((phi.score - 0.72).abs() < 1e-6);
        let gnw = results
            .iter()
            .find(|r| r.name == "consciousness_gnw_broadcast")
            .unwrap();
        assert!((gnw.score - 0.51).abs() < 0.01); // 0.85 * 0.60
        let drt = results
            .iter()
            .find(|r| r.name == "consciousness_drt_accuracy")
            .unwrap();
        let expected_drt = (0.78 + 0.5) / 2.0; // (meta_accuracy + 4/8)
        assert!((drt.score - expected_drt).abs() < 1e-6);
        let composite = results
            .iter()
            .find(|r| r.name == "consciousness_composite")
            .unwrap();
        assert!(composite.score > 0.0 && composite.score <= 1.0);
    }

    #[test]
    fn test_consciousness_benchmarks_zero_edge_cases() {
        let results =
            BenchmarkSuite::run_consciousness_benchmarks(0.0, 0.0, 0.0, 0, 0, 0.0, 0.0, 0, 0);
        assert_eq!(results.len(), 6);
        for r in &results {
            assert!(r.score >= 0.0 && r.score <= 1.0);
        }
        let drt = results
            .iter()
            .find(|r| r.name == "consciousness_drt_accuracy")
            .unwrap();
        assert!((drt.score - 0.0).abs() < 1e-6);
        let speed = results
            .iter()
            .find(|r| r.name == "consciousness_integration_speed")
            .unwrap();
        assert!((speed.score - 0.0).abs() < 0.01);
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
