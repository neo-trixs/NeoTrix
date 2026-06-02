use crate::core::CapabilityVector;
use crate::neotrix::benchmark::{BenchmarkSuite, BenchmarkReport};
use crate::core::nt_core_bank::ReasoningBank;
use std::path::PathBuf;

pub struct ServerProxy;

impl ServerProxy {
    pub fn status() -> serde_json::Value {
        let cap = Self::load_brain();
        let path = Self::snap_path();
        let knowledge_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        serde_json::json!({
            "brain_dims": cap.arr.iter().filter(|&&v| v > 0.0).count(),
            "brain_extension": cap.extension.len(),
            "total_dims": cap.arr.len(),
            "knowledge_store_bytes": knowledge_size,
            "knowledge_store_mb": format!("{:.2}", knowledge_size as f64 / 1024.0 / 1024.0),
        })
    }

    pub fn benchmark() -> String {
        let cap = Self::load_brain();
        let mut bank = ReasoningBank::new(100);
        let report = BenchmarkSuite::run_all_extended(&cap, &mut bank);
        Self::format_report(&report)
    }

    pub fn benchmark_category(category: &str) -> String {
        let cap = Self::load_brain();
        let results = BenchmarkSuite::run_category(&cap, category);
        let report = BenchmarkReport {
            results,
            overall_score: 0.0,
            timestamp: String::new(),
            iteration: 0,
        };
        Self::format_report(&report)
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

    fn snap_path() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".neotrix/knowledge_v2.snap")
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
