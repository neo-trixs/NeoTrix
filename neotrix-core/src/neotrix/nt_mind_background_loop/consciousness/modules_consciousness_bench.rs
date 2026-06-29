use super::ConsciousnessIntegration;
use crate::neotrix::nt_mind_benchmark::BenchmarkSuite;

/// Benchmark history bounded at 100 entries to prevent unbounded growth.
const MAX_BENCH_HISTORY: usize = 100;

impl ConsciousnessIntegration {
    /// Run consciousness benchmarks every 50 cycles.
    ///
    /// Extracts real-time metrics from CI subsystems (meta-cognition loop,
    /// specious present, composite loss, etc.) and feeds them into
    /// `BenchmarkSuite::run_consciousness_benchmarks()`.
    ///
    /// Results are logged and kept in a bounded circular buffer for trend analysis.
    pub fn handle_consciousness_bench_tick(&mut self) -> String {
        if self.cycle == 0 || self.cycle % 50 != 0 {
            return "bench:idle".into();
        }

        // Extract metrics from CI subsystems
        let phi = self.meta_cognition_loop.auto_phi();
        let meta_acc = self.meta_cognition_loop.current_meta_accuracy();

        // GNW proxy: text buffer saturation + working memory load
        let text_buf_coverage = (self.text_buffer.len() as f64 / 50.0).min(1.0);
        let wm_utilization = self.working_memory.load();

        // DRT recursion depth from attractor state + meta-accuracy
        let drt_depth = (self.attractor_state.len() / 16).min(16);
        let drt_max = 16usize;

        // Coherence from specious present (self-world boundary clarity)
        let coherence = self.specious_present.average_coherence();

        // Cycle throughput
        let cycle_count = self.cycle;

        // Run the benchmarks
        let results = BenchmarkSuite::run_consciousness_benchmarks(
            phi,
            text_buf_coverage,
            wm_utilization,
            drt_depth,
            drt_max,
            meta_acc,
            coherence,
            0,
            cycle_count,
        );

        // Log results
        for r in &results {
            log::info!(
                "BENCH: {} score={:.4} max={:.4}",
                r.name,
                r.score,
                r.max_score
            );
        }

        // Bounded history (ring buffer)
        let history = self.bench_history.get_or_insert_with(Vec::new);
        history.extend(results);
        if history.len() > MAX_BENCH_HISTORY {
            let excess = history.len() - MAX_BENCH_HISTORY;
            let _ = history.drain(0..excess);
        }

        format!(
            "bench:phi={:.3}_gnw={:.3}_drt={:.3}_meta={:.3}_coh={:.3}",
            phi,
            text_buf_coverage * wm_utilization,
            meta_acc,
            meta_acc,
            coherence,
        )
    }

    /// Return the most recent benchmark history as a JSON-like summary string.
    pub fn handle_consciousness_bench_history(&mut self) -> String {
        match self.bench_history {
            Some(ref h) if !h.is_empty() => {
                let n = h.len();
                let avg: f64 = h.iter().map(|r| r.score / r.max_score).sum::<f64>() / n as f64;
                format!("bench_history:n={}_avg={:.4}", n, avg)
            }
            _ => "bench_history:empty".to_string(),
        }
    }
}
