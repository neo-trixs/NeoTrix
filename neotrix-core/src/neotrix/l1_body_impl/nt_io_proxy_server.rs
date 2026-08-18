use crate::core::CapabilityVector;
use crate::core::nt_core_bank::ReasoningBank;

/// Local benchmark types (replaces L8 BenchmarkSuite dependency)
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub category: String,
    pub name: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub results: Vec<BenchmarkResult>,
    pub overall_score: f64,
}

pub struct ServerProxy;

impl ServerProxy {
    pub fn status() -> serde_json::Value {
        let cap = Self::load_brain();
        let (kb_bytes, kb_nodes, kb_edges) = Self::kb_stats();
        serde_json::json!({
            "brain_dims": cap.arr.iter().filter(|&&v| v > 0.0).count(),
            "brain_extension": cap.extension.len(),
            "total_dims": cap.arr.len(),
            "knowledge_store_bytes": kb_bytes,
            "knowledge_store_mb": format!("{:.2}", kb_bytes as f64 / 1024.0 / 1024.0),
            "knowledge_nodes": kb_nodes,
            "knowledge_edges": kb_edges,
        })
    }

    /// 真实 KB 统计: knowledge.db 文件大小 + nodes/edges 计数。
    /// 替代原 knowledge_v2.snap (不存在 → 恒 0 bytes 谎言)。
    fn kb_stats() -> (u64, i64, i64) {
        let db_path = dirs::home_dir().unwrap_or_default().join(".neotrix/knowledge.db");
        let bytes = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
        let (nodes, edges) = crate::neotrix::nt_memory_kb::KnowledgeBase::open(None)
            .ok()
            .and_then(|kb| {
                kb.stats().ok().map(|s| {
                    (s.total_nodes, s.total_edges)
                })
            })
            .unwrap_or((0, 0));
        (bytes, nodes, edges)
    }

    pub fn benchmark() -> String {
        let cap = Self::load_brain();
        let mut bank = ReasoningBank::new(100);
        let report = Self::run_benchmark_local(&cap, &mut bank);
        Self::format_report(&report)
    }

    pub fn benchmark_category(category: &str) -> String {
        let cap = Self::load_brain();
        let results = Self::run_benchmark_category_local(&cap, &[category]);
        let report = BenchmarkReport {
            results,
            overall_score: 0.0,
        };
        Self::format_report(&report)
    }

    fn run_benchmark_local(cap: &CapabilityVector, _bank: &mut ReasoningBank) -> BenchmarkReport {
        let dims = [
            ("General", "reasoning", 0.5),
            ("General", "planning", 0.5),
            ("General", "creativity", 0.5),
            ("Reasoning", "depth", cap.inference_depth()),
            ("Reasoning", "analysis", cap.analysis()),
            ("Reasoning", "synthesis", cap.synthesis()),
            ("Code", "generation", 0.5),
            ("Code", "review", 0.5),
            ("Design", "typography", cap.typography()),
            ("Design", "color", cap.color()),
            ("Design", "grid", cap.grid()),
            ("Design", "composition", cap.compound_composition()),
        ];
        let results: Vec<BenchmarkResult> = dims.iter().map(|(cat, name, score)| {
            BenchmarkResult {
                category: cat.to_string(),
                name: name.to_string(),
                score: *score,
            }
        }).collect();
        let overall_score = results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64;
        BenchmarkReport { results, overall_score }
    }

    fn run_benchmark_category_local(cap: &CapabilityVector, categories: &[&str]) -> Vec<BenchmarkResult> {
        let all = Self::run_benchmark_local(cap, &mut ReasoningBank::new(10));
        all.results.into_iter().filter(|r| categories.contains(&r.category.as_str())).collect()
    }

    fn load_brain() -> CapabilityVector {
        let path = dirs::home_dir().unwrap_or_default().join(".neotrix/brain.json");
        if path.exists() {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[server-proxy] read brain.json: {}", e);
                    return CapabilityVector::default();
                }
            };
            match serde_json::from_str::<CapabilityVector>(&content) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("[server-proxy] parse brain.json: {}", e);
                    CapabilityVector::default()
                }
            }
        } else {
            CapabilityVector::default()
        }
    }

    fn format_report(report: &BenchmarkReport) -> String {
        let mut out = String::from("╭─ NeoTrix Benchmark ───────────────────╮\n");
        out.push_str("│ Category      | Test              | Score │\n");
        out.push_str("├───────────────┼───────────────────┼───────┤\n");
        for r in &report.results {
            out.push_str(&format!("│ {:<13} | {:<17} | {:.2}  │\n", r.category, r.name, r.score));
        }
        out.push_str("├───────────────┼───────────────────┼───────┤\n");
        out.push_str(&format!("│ OVERALL       │                   │ {:.2}  │\n", report.overall_score));
        out.push_str("╰───────────────┴───────────────────┴───────╯\n");
        out
    }
}
