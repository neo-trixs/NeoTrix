//! NeoTrix Performance Benchmarks — Key System Latency Profiling
//!
//! Benchmarks 6 critical paths with 1000 iterations each,
//! reporting min/max/avg/p99 latencies via `std::time::Instant`.
//!
//! NOTE: Some target modules (ExperimentTree, ActionFusion, ContextCompactor,
//! SkillState, TieredStore) exist as files but aren't compiled into the crate
//! (not declared in parent mod.rs). This benchmark uses:
//! - Compiled modules: search_fts, ExperimentRegistry, ExperimentDesigner
//! - Inline implementations: LRU hot tier, bounded context, action fusion patterns
//!
//! Run: `cargo bench --bench neotrix_benchmarks`

use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use lru::LruCache;
use neotrix::l1_action::nt_memory::nt_memory_kb::nt_memory_search::search_fts;
use neotrix::l5_cognition::nt_mind::nt_mind::evolution::experiment::{
    ABTestDesign, ExperimentDesigner, ExperimentRegistry, ExperimentResult, Hypothesis,
};
use rusqlite::Connection;

const ITERS: usize = 1000;

// ═══════════════════════════════════════════════════════════════════════════════
// Benchmark Result Struct
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct BenchResult {
    name: String,
    iterations: usize,
    min_us: u128,
    max_us: u128,
    avg_us: f64,
    p99_us: u128,
    total_us: u128,
}

impl BenchResult {
    fn report(&self) {
        println!(
            "{:<45} iters={:>5}  min={:>8.1}us  avg={:>8.1}us  max={:>8.1}us  p99={:>8.1}us  total={:.2}ms",
            self.name,
            self.iterations,
            self.min_us as f64,
            self.avg_us,
            self.max_us as f64,
            self.p99_us as f64,
            self.total_us as f64 / 1000.0,
        );
    }
}

fn compute_result(name: &str, timings: &mut Vec<u128>) -> BenchResult {
    timings.sort_unstable();
    let min = timings[0];
    let max = timings[timings.len() - 1];
    let total: u128 = timings.iter().sum();
    let avg = total as f64 / timings.len() as f64;
    let p99_idx = ((timings.len() as f64) * 0.99) as usize;
    let p99 = timings[p99_idx.min(timings.len() - 1)];
    BenchResult {
        name: name.to_string(),
        iterations: timings.len(),
        min_us: min,
        max_us: max,
        avg_us: avg,
        p99_us: p99,
        total_us: total,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. KB Search Performance — FTS5 query latency (compiled module)
// ═══════════════════════════════════════════════════════════════════════════════

fn setup_kb_search_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        "CREATE TABLE nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL DEFAULT 'concept',
            title TEXT NOT NULL DEFAULT '',
            summary TEXT NOT NULL DEFAULT '',
            content TEXT,
            url TEXT,
            domain TEXT,
            language TEXT,
            confidence REAL DEFAULT 0.5,
            importance REAL DEFAULT 0.5,
            created_at INTEGER DEFAULT 0,
            updated_at INTEGER DEFAULT 0,
            access_count INTEGER DEFAULT 0,
            metadata TEXT
        );
        CREATE VIRTUAL TABLE nodes_fts USING fts5(title, summary, content, domain);
        ",
    )
    .expect("create tables");

    let mut stmt = conn
        .prepare("INSERT INTO nodes (id, title, summary, content) VALUES (?1, ?2, ?3, ?4)")
        .expect("prepare insert");
    let mut fts_stmt = conn
        .prepare("INSERT INTO nodes_fts (rowid, title, summary, content) VALUES (last_insert_rowid(), ?1, ?2, ?3)")
        .expect("prepare fts insert");

    for i in 0..500 {
        let title = format!("Knowledge Article {} about Rust performance", i);
        let summary = format!("Summary of article {} covering systems programming topics", i);
        let content = format!(
            "Detailed content for article {}. This covers advanced Rust patterns including \
             lifetimes, borrow checker, async runtime, and memory safety guarantees. \
             The article discusses how NeoTrix leverages these patterns for its \
             consciousness architecture and reasoning engine.",
            i
        );
        stmt.execute(rusqlite::params![format!("n{}", i), title, summary, content])
            .expect("insert node");
        fts_stmt
            .execute(rusqlite::params![title, summary, content])
            .expect("insert fts");
    }

    drop(stmt);
    drop(fts_stmt);

    conn
}

fn bench_kb_search() -> BenchResult {
    let conn = setup_kb_search_db();
    let queries = [
        "Rust performance",
        "consciousness architecture",
        "memory safety",
        "borrow checker",
        "async runtime",
    ];

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let query = queries[i % queries.len()];
        let start = Instant::now();
        let _results = search_fts(&conn, query, 10).expect("search_fts");
        timings.push(start.elapsed().as_micros());
    }
    compute_result("KB Search (FTS5)", &mut timings)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Dispatch Routing — capability tag matching & task dispatch
//    (dispatch_internal_capability is private; benchmarks the public routing pattern)
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_dispatch_routing() -> BenchResult {
    let tags = [
        "xlsx_consolidation",
        "file_extract",
        "universal_model",
        "file_enhance",
        "code_review",
        "data_merge",
        "content_extraction",
        "unknown_tag",
    ];

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let tag = tags[i % tags.len()];
        let start = Instant::now();

        // Simulate dispatch routing: tag matching + handler selection
        let _handler = match tag {
            "xlsx_consolidation" | "data_merge" => "xlsx_handler",
            "file_extract" | "content_extraction" => "file_extractor",
            "universal_model" => "model_router",
            "file_enhance" => "file_enhancer",
            _ => "fallback",
        };

        timings.push(start.elapsed().as_micros());
    }
    compute_result("Dispatch Routing (tag match)", &mut timings)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Tiered Store — hot/warm access patterns (LRU + SQLite)
//    (TieredStore not compiled; benchmarks constituent tier technologies)
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_tiered_store_hot() -> BenchResult {
    let mut hot: LruCache<String, Vec<u8>> =
        LruCache::new(NonZeroUsize::new(1000).unwrap());
    for i in 0..500 {
        hot.put(format!("key_{}", i), vec![i as u8; 128]);
    }

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let key = format!("key_{}", i % 500);
        let start = Instant::now();
        let _ = hot.get(&key);
        timings.push(start.elapsed().as_micros());
    }
    compute_result("Tiered Store (hot tier LRU)", &mut timings)
}

fn bench_tiered_store_warm() -> BenchResult {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        "CREATE TABLE tiered_warm (
            key TEXT PRIMARY KEY,
            data BLOB NOT NULL,
            access_count INTEGER NOT NULL DEFAULT 1,
            last_accessed INTEGER NOT NULL
        );",
    )
    .expect("create warm table");

    for i in 0..2000 {
        conn.execute(
            "INSERT INTO tiered_warm (key, data, access_count, last_accessed) VALUES (?1, ?2, 1, 0)",
            rusqlite::params![format!("key_{}", i), vec![i as u8; 128]],
        )
        .expect("insert warm");
    }

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let key = format!("key_{}", i % 2000);
        let start = Instant::now();
        let _result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT data FROM tiered_warm WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .ok();
        timings.push(start.elapsed().as_micros());
    }
    compute_result("Tiered Store (warm tier SQLite)", &mut timings)
}

fn bench_tiered_store_full_flow() -> BenchResult {
    let mut hot: LruCache<String, Vec<u8>> =
        LruCache::new(NonZeroUsize::new(500).unwrap());
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        "CREATE TABLE tiered_warm (
            key TEXT PRIMARY KEY,
            data BLOB NOT NULL,
            access_count INTEGER NOT NULL DEFAULT 1,
            last_accessed INTEGER NOT NULL
        );",
    )
    .expect("create warm table");

    // Pre-populate: 200 hot, 1800 warm
    for i in 0..2000 {
        let data = vec![i as u8; 128];
        if i < 200 {
            hot.put(format!("key_{}", i), data.clone());
        }
        conn.execute(
            "INSERT INTO tiered_warm (key, data, access_count, last_accessed) VALUES (?1, ?2, 1, 0)",
            rusqlite::params![format!("key_{}", i), data],
        )
        .expect("insert");
    }

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let key = format!("key_{}", i % 2000);
        let start = Instant::now();
        // Tiered get: hot -> warm
        let _data = hot.get(&key).cloned().unwrap_or_else(|| {
            conn.query_row(
                "SELECT data FROM tiered_warm WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .ok()
            .unwrap_or_default()
        });
        timings.push(start.elapsed().as_micros());
    }
    compute_result("Tiered Store (hot+warm flow)", &mut timings)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. SKILL.state — bounded context serialization
//    (SkillState not compiled; benchmarks the O(1) bounded-context pattern)
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_skill_state_context() -> BenchResult {
    // Simulate SkillState: bounded mutable state that doesn't grow with iterations
    struct SimulatedSkillState {
        goal: String,
        constraints: Vec<String>,
        current_phase: String,
        iteration: usize,
        total_tokens: usize,
        recent_errors: Vec<(String, String)>,
        observation: Option<(String, f64)>,
    }

    impl SimulatedSkillState {
        fn to_context(&self) -> String {
            let mut ctx = String::with_capacity(512);
            ctx.push_str(&format!("goal: {}\n", self.goal));
            ctx.push_str(&format!("phase: {}\n", self.current_phase));
            ctx.push_str(&format!("iteration: {}\n", self.iteration));
            ctx.push_str(&format!("tokens: {}\n", self.total_tokens));
            for (phase, msg) in self.recent_errors.iter().rev().take(5) {
                ctx.push_str(&format!("error: {} -- {}\n", phase, msg));
            }
            if let Some((phase, eff)) = &self.observation {
                ctx.push_str(&format!("obs: {} eff={:.3}\n", phase, eff));
            }
            ctx
        }

        fn prompt_growth(&self) -> usize {
            let spec_len = self.goal.len()
                + self.constraints.iter().map(|c| c.len()).sum::<usize>();
            let state_len = 64 + self.recent_errors.len() * 128;
            let obs_len = if self.observation.is_some() { 256 } else { 0 };
            spec_len + state_len + obs_len
        }
    }

    let mut state = SimulatedSkillState {
        goal: "Optimize SEAL pipeline convergence rate".to_string(),
        constraints: vec![
            "No regression in accuracy".to_string(),
            "Token budget < 50K".to_string(),
        ],
        current_phase: "checkpoint".to_string(),
        iteration: 0,
        total_tokens: 0,
        recent_errors: Vec::new(),
        observation: None,
    };

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        state.iteration = i;
        state.total_tokens += 100;
        if i % 100 == 0 {
            state
                .recent_errors
                .push(("process".into(), format!("err {}", i)));
            if state.recent_errors.len() > 5 {
                state.recent_errors.remove(0);
            }
        }
        state.observation = Some(("benchmark".into(), 0.5 + (i as f64) * 0.0001));

        let start = Instant::now();
        let _ctx = state.to_context();
        let _size = state.prompt_growth();
        timings.push(start.elapsed().as_micros());
    }
    compute_result("SKILL.state (to_context+growth)", &mut timings)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. SoL-Pi Efficiency — Action Fusion + Context Compactor patterns
//    (ActionFusion/ContextCompactor not compiled; benchmarks the algorithms)
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_action_fusion() -> BenchResult {
    // Simulate ActionFusion: batch N tool calls into a single coordinated action
    struct SimPendingAction {
        id: String,
        tool: String,
        priority: u8,
    }

    struct SimActionFusion {
        pending: Vec<SimPendingAction>,
        max_batch: usize,
        batch_id: u32,
    }

    impl SimActionFusion {
        fn new(max_batch: usize) -> Self {
            Self {
                pending: Vec::with_capacity(max_batch * 2),
                max_batch,
                batch_id: 0,
            }
        }

        fn add(&mut self, action: SimPendingAction) {
            if self.pending.len() < self.max_batch * 2 {
                self.pending.push(action);
            }
        }

        fn should_fuse(&self) -> bool {
            self.pending.len() >= self.max_batch
        }

        fn fuse(&mut self) -> Vec<SimPendingAction> {
            if self.pending.is_empty() {
                return Vec::new();
            }
            self.batch_id += 1;
            let batch_size = self.pending.len().min(self.max_batch);
            let mut batch: Vec<SimPendingAction> = self.pending.drain(..batch_size).collect();
            batch.sort_by(|a, b| b.priority.cmp(&a.priority));
            batch
        }
    }

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let mut fusion = SimActionFusion::new(8);
        let start = Instant::now();
        for j in 0..8 {
            fusion.add(SimPendingAction {
                id: format!("a{}_{}", i, j),
                tool: "read_file".into(),
                priority: (j % 4) as u8,
            });
        }
        if fusion.should_fuse() {
            let batch = fusion.fuse();
            let _summary = format!("Fused {} actions", batch.len());
        }
        timings.push(start.elapsed().as_micros());
    }
    compute_result("SoL-Pi ActionFusion (8 actions)", &mut timings)
}

fn bench_context_compactor() -> BenchResult {
    // Simulate ContextCompactor: priority-queue based token budget enforcement
    #[derive(Clone)]
    struct SimChunk {
        id: String,
        tokens: usize,
        priority: u32,
    }

    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let budget = 1000usize;
        let mut chunks: BTreeMap<(u32, usize), SimChunk> = BTreeMap::new();
        let mut total_tokens = 0usize;

        let start = Instant::now();
        for j in 0..20 {
            let chunk = SimChunk {
                id: format!("c{}_{}", i, j),
                tokens: 80,
                priority: (j % 10) as u32,
            };
            total_tokens += chunk.tokens;
            chunks.insert(
                (chunk.priority, j),
                chunk,
            );

            // Compact: remove lowest-priority chunks until within budget
            while total_tokens > budget {
                if let Some((key, removed)) = chunks.pop_first() {
                    total_tokens -= removed.tokens;
                } else {
                    break;
                }
            }
        }
        let _within = total_tokens <= budget;
        timings.push(start.elapsed().as_micros());
    }
    compute_result("SoL-Pi ContextCompactor (20 chunks)", &mut timings)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Experiment Tree — registry operations (compiled module)
//    Uses ExperimentRegistry + ExperimentDesigner from evolution::experiment
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_experiment_registry_add() -> BenchResult {
    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let mut registry = ExperimentRegistry::new();
        let start = Instant::now();
        let h = Hypothesis::new(
            &format!("H-{}", i),
            &format!("Hypothesis {} about optimization", i),
            vec!["metric_a".into(), "metric_b".into()],
            0.1 + (i as f64) * 0.001,
        );
        registry.register_hypothesis(h);
        let design = ExperimentDesigner::design_ab_test(&registry.hypotheses[&format!("H-{}", i)]);
        registry.register_design(design);
        timings.push(start.elapsed().as_micros());
    }
    compute_result("Experiment Registry (add+design)", &mut timings)
}

fn bench_experiment_analyze() -> BenchResult {
    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let control: Vec<f64> = (0..50).map(|j| 1.0 + (j as f64) * 0.01).collect();
        let treatment: Vec<f64> = (0..50).map(|j| 1.2 + (j as f64) * 0.01 + (i as f64) * 0.0001).collect();

        let start = Instant::now();
        let _result = ExperimentDesigner::analyze_results(&control, &treatment, &format!("H-{}", i));
        timings.push(start.elapsed().as_micros());
    }
    compute_result("Experiment analyze_results (n=100)", &mut timings)
}

fn bench_experiment_registry_full_cycle() -> BenchResult {
    let mut timings = Vec::with_capacity(ITERS);
    for i in 0..ITERS {
        let mut registry = ExperimentRegistry::new();
        let start = Instant::now();

        // Register hypothesis
        let h = Hypothesis::new(
            &format!("H-{}", i),
            &format!("Test hypothesis {}", i),
            vec!["latency".into()],
            0.3,
        );
        registry.register_hypothesis(h);

        // Design A/B test
        let design = ExperimentDesigner::design_ab_test(
            &registry.hypotheses[&format!("H-{}", i)],
        );
        registry.register_design(design);

        // Mark active
        registry.mark_active(&format!("H-{}", i));

        // Analyze results
        let control: Vec<f64> = (0..30).map(|j| 1.0 + (j as f64) * 0.01).collect();
        let treatment: Vec<f64> = (0..30).map(|j| 1.5 + (j as f64) * 0.01).collect();
        let result = ExperimentDesigner::analyze_results(&control, &treatment, &format!("H-{}", i));

        // Record result
        registry.record_result(result);

        timings.push(start.elapsed().as_micros());
    }
    compute_result("Experiment full cycle (register->design->analyze)", &mut timings)
}

fn bench_sample_size_estimation() -> BenchResult {
    let mut timings = Vec::with_capacity(ITERS);
    let effect_sizes = [0.1, 0.2, 0.3, 0.5, 0.8, 1.0];
    for i in 0..ITERS {
        let effect = effect_sizes[i % effect_sizes.len()];
        let start = Instant::now();
        let _n = ExperimentDesigner::estimate_sample_size(effect, 0.05, 0.80);
        timings.push(start.elapsed().as_micros());
    }
    compute_result("Experiment estimate_sample_size", &mut timings)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════════

fn main() {
    println!("=====================================================================");
    println!("  NeoTrix Performance Benchmarks -- {} iterations each", ITERS);
    println!("=====================================================================");
    println!();

    let results = vec![
        bench_kb_search(),
        bench_dispatch_routing(),
        bench_tiered_store_hot(),
        bench_tiered_store_warm(),
        bench_tiered_store_full_flow(),
        bench_skill_state_context(),
        bench_action_fusion(),
        bench_context_compactor(),
        bench_experiment_registry_add(),
        bench_experiment_analyze(),
        bench_experiment_registry_full_cycle(),
        bench_sample_size_estimation(),
    ];

    println!(
        "{:<45} {:>5} {:>10} {:>10} {:>10} {:>10} {:>9}",
        "Benchmark", "Iters", "Min (us)", "Avg (us)", "Max (us)", "P99 (us)", "Total (ms)"
    );
    println!("{}", "-".repeat(100));
    for r in &results {
        r.report();
    }

    // Performance assertions
    println!();
    println!("-- Performance Assertions --");
    let mut all_pass = true;
    for r in &results {
        let threshold_us = match r.name.as_str() {
            n if n.contains("KB Search") => 10_000,       // 10ms
            n if n.contains("Dispatch") => 50,             // 50us
            n if n.contains("hot tier") => 10,             // 10us (LRU should be fast)
            n if n.contains("warm tier") => 500,           // 500us (SQLite)
            n if n.contains("hot+warm") => 500,            // 500us
            n if n.contains("SKILL.state") => 100,         // 100us
            n if n.contains("ActionFusion") => 200,        // 200us
            n if n.contains("ContextCompactor") => 200,    // 200us
            n if n.contains("add") => 100,                 // 100us
            n if n.contains("analyze") => 500,             // 500us
            n if n.contains("full cycle") => 500,          // 500us
            n if n.contains("sample_size") => 50,          // 50us
            _ => 10_000,
        };
        let pass = r.p99_us <= threshold_us;
        let status = if pass { "PASS" } else { "FAIL" };
        println!(
            "  {} {:<45} p99={:>8.1}us  threshold={:>6}us",
            status, r.name, r.p99_us, threshold_us
        );
        if !pass {
            all_pass = false;
        }
    }
    println!();
    if all_pass {
        println!("  All assertions PASSED.");
    } else {
        println!("  Some assertions FAILED -- review latency regression.");
        std::process::exit(1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// #[cfg(test)] -- unit-testable performance assertions
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ITERS: usize = 100;

    #[test]
    fn kb_search_completes_under_10ms() {
        let conn = setup_kb_search_db();
        let mut timings = Vec::with_capacity(TEST_ITERS);
        for i in 0..TEST_ITERS {
            let query = match i % 3 {
                0 => "Rust performance",
                1 => "memory safety",
                _ => "async runtime",
            };
            let start = Instant::now();
            let _results = search_fts(&conn, query, 10).expect("search");
            timings.push(start.elapsed().as_micros());
        }
        let result = compute_result("kb_search", &mut timings);
        assert!(
            result.p99_us < 10_000,
            "KB search p99={}us exceeds 10ms threshold",
            result.p99_us
        );
    }

    #[test]
    fn tiered_hot_hit_under_10us() {
        let mut hot: LruCache<String, Vec<u8>> =
            LruCache::new(NonZeroUsize::new(1000).unwrap());
        for i in 0..500 {
            hot.put(format!("key_{}", i), vec![i as u8; 128]);
        }

        let mut timings = Vec::with_capacity(TEST_ITERS);
        for i in 0..TEST_ITERS {
            let key = format!("key_{}", i % 500);
            let start = Instant::now();
            let _ = hot.get(&key);
            timings.push(start.elapsed().as_micros());
        }
        let result = compute_result("tiered_hot_hit", &mut timings);
        assert!(
            result.p99_us < 10,
            "Tiered hot hit p99={}us exceeds 10us threshold",
            result.p99_us
        );
    }

    #[test]
    fn skill_state_context_bounded_after_1000_iterations() {
        struct SimState {
            goal: String,
            iteration: usize,
            errors: Vec<(String, String)>,
        }
        impl SimState {
            fn to_context(&self) -> String {
                format!(
                    "goal: {}\niteration: {}\nerrors: {}",
                    self.goal,
                    self.iteration,
                    self.errors.len()
                )
            }
            fn prompt_growth(&self) -> usize {
                64 + self.errors.len() * 128
            }
        }

        let mut state = SimState {
            goal: "test".into(),
            iteration: 0,
            errors: Vec::new(),
        };

        for i in 0..1000 {
            state.iteration = i;
            state.errors.push(("phase".into(), format!("err {}", i)));
        }

        let size = state.prompt_growth();
        assert!(
            size < 8192,
            "SKILL.state context should be bounded, got {} bytes",
            size
        );

        let ctx = state.to_context();
        assert!(
            ctx.len() < 4096,
            "to_context output too large: {}",
            ctx.len()
        );
    }

    #[test]
    fn action_fusion_batch_respects_size() {
        struct SimPendingAction {
            id: String,
            priority: u8,
        }
        struct SimFusion {
            pending: Vec<SimPendingAction>,
            max_batch: usize,
        }
        impl SimFusion {
            fn new(max_batch: usize) -> Self {
                Self {
                    pending: Vec::new(),
                    max_batch,
                }
            }
            fn add(&mut self, a: SimPendingAction) {
                self.pending.push(a);
            }
            fn should_fuse(&self) -> bool {
                self.pending.len() >= self.max_batch
            }
            fn fuse(&mut self) -> Vec<SimPendingAction> {
                let n = self.pending.len().min(self.max_batch);
                self.pending.drain(..n).collect()
            }
        }

        let mut fusion = SimFusion::new(4);
        for i in 0..8 {
            fusion.add(SimPendingAction {
                id: format!("a{}", i),
                priority: i as u8,
            });
        }
        assert!(fusion.should_fuse());
        let batch = fusion.fuse();
        assert_eq!(batch.len(), 4);
        assert_eq!(fusion.pending.len(), 4);
    }

    #[test]
    fn experiment_registry_full_lifecycle() {
        let mut registry = ExperimentRegistry::new();

        let h = Hypothesis::new("H-001", "Test hypothesis", vec!["metric".into()], 0.5);
        registry.register_hypothesis(h);
        assert_eq!(registry.hypotheses.len(), 1);

        let design = ExperimentDesigner::design_ab_test(&registry.hypotheses["H-001"]);
        registry.register_design(design);
        assert_eq!(registry.designs.len(), 1);

        registry.mark_active("H-001");
        assert_eq!(registry.active.len(), 1);

        let result = ExperimentResult {
            hypothesis_id: "H-001".into(),
            p_value: 0.01,
            effect_size: 0.5,
            significant: true,
            control_mean: 1.0,
            treatment_mean: 1.5,
            sample_size: 100,
        };
        registry.record_result(result);
        assert!(registry.active.is_empty());
        assert!(registry.results.contains_key("H-001"));
    }
}
