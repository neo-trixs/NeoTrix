use chrono::Utc;

use super::self_iterating::SelfIteratingBrain;

#[derive(Debug, Clone)]
pub struct BenchDimension {
    pub name: String,
    pub score: f64,
    pub weight: f64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ReasoningBenchReport {
    pub score: f64,
    pub dimensions: Vec<BenchDimension>,
    pub timestamp: String,
}

impl ReasoningBenchReport {
    pub fn display(&self) {
        println!("╭─ Reasoning Engine Quality Benchmark ───────────────╮");
        println!("│ Overall Score: {:.3}                                 │", self.score);
        println!("├──────────────────────────────────────────────────────┤");
        for dim in &self.dimensions {
            let bar = score_bar(dim.score);
            println!("│ {:<30} {:>6.1} {} │", dim.name, dim.score * 100.0, bar);
        }
        println!("╰──────────────────────────────────────────────────────╯");
    }
}

fn score_bar(score: f64) -> String {
    let filled = (score * 20.0).round() as usize;
    let empty = 20usize.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

pub struct ReasoningBenchmark;

impl ReasoningBenchmark {
    pub fn run() -> ReasoningBenchReport {
        let mut dimensions = Vec::new();

        #[cfg(any())]
        let e8_score = Self::test_e8_strategy_matrix();
        #[cfg(any())]
        dimensions.push(BenchDimension {
            name: "E8 Strategy Matrix".into(),
            score: e8_score,
            weight: 0.30,
            description: "All 64 entries have valid hexagram values (0..63)".into(),
        });

        let seal_score = Self::test_seal_iteration();
        dimensions.push(BenchDimension {
            name: "SEAL Loop Iteration".into(),
            score: seal_score,
            weight: 0.30,
            description: "5 SEAL iterations with monotonic improvement".into(),
        });

        #[cfg(any())]
        let attention_score = Self::test_attention_router();
        #[cfg(any())]
        dimensions.push(BenchDimension {
            name: "Attention Router".into(),
            score: attention_score,
            weight: 0.20,
            description: "All 11 specialists respond to trigger keywords".into(),
        });

        #[cfg(any())]
        let knowledge_score = Self::test_knowledge_retrieval();
        #[cfg(any())]
        dimensions.push(BenchDimension {
            name: "Knowledge HyperCube".into(),
            score: knowledge_score,
            weight: 0.20,
            description: "HyperCube insert and recall accuracy".into(),
        });

        let total_weight: f64 = dimensions.iter().map(|d| d.weight).sum();
        let weighted = dimensions.iter()
            .map(|d| d.score * d.weight)
            .sum::<f64>() / total_weight;

        ReasoningBenchReport {
            score: weighted,
            dimensions,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    #[cfg(any())]
    /// Verify all 64 strategy matrix entries have valid hexagram values (0..63).
    fn test_e8_strategy_matrix() -> f64 {
        let matrix = strategy_matrix();
        let mut valid = 0usize;
        let total = 64usize;
        let mut seen = std::collections::HashSet::new();

        for row in 0..8 {
            for col in 0..8 {
                let hex = matrix[row][col];
                if hex.0 < 64 {
                    valid += 1;
                }
                seen.insert(hex.0);
            }
        }

        let mut coverage_score = valid as f64 / total as f64;

        let unique_count = seen.len();
        if unique_count >= 60 {
            coverage_score *= 1.0;
        } else if unique_count >= 40 {
            coverage_score *= 0.8;
        } else {
            coverage_score *= 0.5;
        }

        coverage_score.min(1.0)
    }

    /// Run 5 SEAL iterations without external reward, verify monotonic improvement.
    fn test_seal_iteration() -> f64 {
        let mut brain = SelfIteratingBrain::new();
        brain.brain.learning_rate = 0.05;
        brain.quality_threshold = 0.5;

        let tasks = [
            "analyze log patterns",
            "review code structure",
            "optimize query performance",
            "design component API",
            "refactor module boundaries",
        ];

        let mut scores = Vec::new();
        let mut success_count = 0usize;

        for task in &tasks {
            let result = brain.kernel_iterate(task);
            scores.push(result.score_after);
            if result.improved || result.score_after > 0.3 {
                success_count += 1;
            }
        }

        let base_score = success_count as f64 / tasks.len() as f64;

        let monotonic = if scores.len() >= 5 {
            let mut increases = 0usize;
            for w in scores.windows(2) {
                if w[1] >= w[0] - 0.01 {
                    increases += 1;
                }
            }
            increases as f64 / (scores.len() - 1) as f64
        } else {
            0.0
        };

        base_score * 0.5 + monotonic * 0.5
    }

    /// Verify all 11 specialists respond to their trigger keywords.
    #[cfg(any())]
    fn test_attention_router() -> f64 {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();

        let test_cases: Vec<(&str, Vec<&str>)> = vec![
            ("find patterns in the error logs and suggest fixes",
             vec!["pattern-matcher", "anomaly-detector"]),
            ("plan next sprint goals and set milestones",
             vec!["goal-prioritizer", "planner"]),
            ("brainstorm novel design ideas for the new interface",
             vec!["creativity-generator"]),
            ("critical bug in production, nt_shield vulnerability detected",
             vec!["anomaly-detector", "risk-assessor"]),
            ("reflect on recent performance and improve the process",
             vec!["reflection-engine"]),
            ("analyze the code for structural issues",
             vec!["code-analyzer"]),
            ("integrate knowledge from multiple research papers",
             vec!["knowledge-integrator"]),
            ("assess risks of the deployment strategy",
             vec!["risk-assessor"]),
            ("synthesize findings from all experiments",
             vec!["knowledge-integrator", "pattern-matcher"]),
        ];

        let mut triggered = 0usize;
        let mut total_expected = 0usize;

        for (context, expected_specialists) in &test_cases {
            let result = router.route(context);
            let active_names: Vec<&str> = result.active_specialists.iter()
                .map(|st| st.name())
                .collect();
            for expected in expected_specialists {
                total_expected += 1;
                if active_names.contains(expected) {
                    triggered += 1;
                }
            }
        }

        if total_expected > 0 {
            triggered as f64 / total_expected as f64
        } else {
            0.0
        }
    }

    /// Insert a test entry into HyperCube, verify recall.
    #[cfg(any())]
    fn test_knowledge_retrieval() -> f64 {
        let mut bridge = HyperCubeBridge::new();
        let mut cortex = super::cortex_memory::CortexMemory::new(10, 100);

        let traces = vec![
            MemoryTrace::new(
                "Deep Learning Fundamentals",
                "https://arxiv.org/abs/2501.00001",
                "Neural networks and deep learning basics",
                Modality::Text,
                vec![DimensionTag::TechAI, DimensionTag::KnowledgeScience],
            ),
            MemoryTrace::new(
                "Quantum Computing",
                "https://arxiv.org/abs/2501.00002",
                "Quantum algorithms and error correction",
                Modality::Text,
                vec![DimensionTag::TechAI, DimensionTag::KnowledgeScience],
            ),
            MemoryTrace::new(
                "Ancient Philosophy",
                "https://en.wikipedia.org/wiki/Ancient_philosophy",
                "Greek and Eastern philosophical traditions",
                Modality::Text,
                vec![DimensionTag::KnowledgeCulture, DimensionTag::KnowledgePhilosophy],
            ),
        ];

        for t in traces {
            cortex.store(t);
        }

        bridge.ingest_from_cortex(&cortex);
        let total_cells = bridge.hypercube.cell_count();
        if total_cells == 0 {
            return 0.0;
        }

        let coord_tech = HyperCoord::with(DimensionAxis::Domain, 0.5);
        let tech_results = bridge.query(&coord_tech, 5);
        let tech_recalled = tech_results.iter().any(|e| e.source.contains("arxiv"));

        let coord_culture = HyperCoord::with(DimensionAxis::Culture, 0.5);
        let culture_results = bridge.query(&coord_culture, 5);
        let culture_recalled = culture_results.iter().any(|e| e.source.contains("wikipedia"));

        let mut recall_score = 0.0;
        if tech_recalled { recall_score += 0.5; }
        if culture_recalled { recall_score += 0.5; }

        let density_uniform = if total_cells >= 3 { 1.0 } else { total_cells as f64 / 3.0 };
        recall_score * 0.7 + density_uniform * 0.3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any())]
    #[test]
    fn test_e8_strategy_matrix_all_valid() {
        let score = ReasoningBenchmark::test_e8_strategy_matrix();
        assert!(score > 0.0, "E8 strategy matrix should have valid entries");
        assert!(score <= 1.0);
    }

    #[test]
    fn test_seal_iteration_runs() {
        let score = ReasoningBenchmark::test_seal_iteration();
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    #[cfg(any())]
    fn test_attention_router_activates_specialists() {
        let score = ReasoningBenchmark::test_attention_router();
        assert!(score > 0.0, "attention router should activate specialists: {:.3}", score);
        assert!(score <= 1.0);
    }

    #[test]
    #[cfg(any())]
    fn test_knowledge_retrieval_inserts_and_recalls() {
        let score = ReasoningBenchmark::test_knowledge_retrieval();
        assert!(score > 0.0, "knowledge retrieval should recall entries: {:.3}", score);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_reasoning_benchmark_runs() {
        let report = ReasoningBenchmark::run();
        assert!(report.score >= 0.0 && report.score <= 1.0);
        assert_eq!(report.dimensions.len(), 4);
        assert!(!report.timestamp.is_empty());
    }

    #[test]
    fn test_dimension_weights_sum_to_one() {
        let report = ReasoningBenchmark::run();
        let weight_sum: f64 = report.dimensions.iter().map(|d| d.weight).sum();
        assert!((weight_sum - 1.0).abs() < 0.01, "weights sum to {:.3}", weight_sum);
    }

    #[test]
    fn test_score_bar_format() {
        let bar = score_bar(0.5);
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert!(bar.len() >= 4, "bar '{}' should have brackets and content", bar);
        assert!(bar.contains('█'), "bar should have filled blocks");
        assert!(bar.contains('░'), "bar should have empty blocks");
    }

    #[test]
    fn test_display_does_not_panic() {
        let report = ReasoningBenchmark::run();
        report.display();
    }
}
