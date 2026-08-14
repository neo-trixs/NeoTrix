// C3 benchmark 基线 — 3 个 C2 能力节点 (能力树 C2→C3 晋升证据, R-P100)
//
// 覆盖:
// - nt_memory_visibility::visibility_gate — filter_visibility 三值裁定吞吐
// - nt_memory_provenance::decision_trail — record_with_index 落盘 + query_provenance 回查
// - nt_io_hotreload::revertible_effects  — RevertibleContext track/recover 事务回滚
//
// 目的: 建立性能基线 (C3 = benchmark 基线建立 + 无回归), 每次晋升需跑此基准
// 对比历史 (cargo bench --bench capability_c3)。

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_provenance::{self, ProvenanceRecord, ProvActivity};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::{KnowledgeNode, NodeType, SearchResult, SearchMatchType};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_visibility::{filter_visibility, VisibilityConfig};
use neotrix::core::nt_core_context::revertible::{RevertibleContext, add_effect};

/// 构造 n 条候选 SearchResult (混合风险/相关度, 覆盖三种裁定路径)。
fn make_results(n: usize) -> Vec<SearchResult> {
    (0..n)
        .map(|i| {
            let node = KnowledgeNode {
                id: format!("node-{i}"),
                node_type: NodeType::Insight,
                title: format!("title {i}"),
                summary: Some(format!("summary {i}")),
                content: None,
                url: None,
                domain: Some("NT-CORE".into()),
                language: "zh".into(),
                confidence: 0.9,
                importance: 0.5,
                created_at: 0,
                updated_at: 0,
                access_count: 0,
                metadata: None,
                temporal: None,
                supersedes: None,
                source_episode: None,
            };
            // i%10==0 → 高风险 (risk>0.8), i%3==0 → 低相关, 其余 Allow
            let risk = if i % 10 == 0 { 0.95 } else { 0.1 };
            let score = if i % 3 == 0 { 0.2 } else { 0.9 };
            SearchResult {
                node,
                score,
                matched_on: vec![SearchMatchType::Bm25],
                signals: Some([risk, 0.5, 0.0, 0.0]),
            }
        })
        .collect()
}

fn bench_visibility_gate(c: &mut Criterion) {
    let config = VisibilityConfig::default();
    let mut group = c.benchmark_group("visibility_gate");
    for &n in &[64usize, 256, 1024] {
        let results = make_results(n);
        group.bench_with_input(format!("filter_{n}"), &results, |b, r| {
            b.iter_batched(
                || r.clone(),
                |batch| filter_visibility(black_box(batch), &config),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_decision_trail(c: &mut Criterion) {
    let mut group = c.benchmark_group("decision_trail");
    // record_with_index: 内存 DB 落盘 + 索引维护
    let conn = open_memory_conn();
    group.bench_function("record_with_index", |b| {
        b.iter_batched(
            || {
                ProvenanceRecord::new(
                    "nt_memory_curation",
                    ProvActivity::Supersede,
                    "node-old",
                    "superseded",
                )
            },
            |rec| {
                let _ = black_box(nt_memory_provenance::record_with_index(&conn, rec));
            },
            BatchSize::SmallInput,
        );
    });
    // query_provenance: 先写 1000 条, 测回查吞吐
    for i in 0..1000 {
        let rec = ProvenanceRecord::new(
            "nt_memory_curation",
            ProvActivity::Supersede,
            format!("node-{i}"),
            "superseded",
        );
        let _ = nt_memory_provenance::record_with_index(&conn, rec);
    }
    group.bench_function("query_provenance_1000", |b| {
        b.iter(|| {
            let _ = black_box(nt_memory_provenance::query_provenance(&conn, Some("nt_memory_curation"), None, None));
        });
    });
    group.finish();
}

fn bench_revertible_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("revertible_effects");
    // track: 100 次 forward 推栈
    group.bench_function("track_100", |b| {
        b.iter_batched(
            || {
                let mut ctx = RevertibleContext::new(0i64);
                for _ in 0..100 {
                    ctx.track(add_effect(
                        "op",
                        |s: &mut i64| *s += 1,
                        |s: &mut i64| *s -= 1,
                    ));
                }
                ctx
            },
            |ctx| black_box(ctx.depth()),
            BatchSize::SmallInput,
        );
    });
    // recover: 100 条效果栈整体回滚
    group.bench_function("recover_100", |b| {
        b.iter(|| {
            let mut ctx = RevertibleContext::new(0i64);
            for _ in 0..100 {
                ctx.track(add_effect(
                    "op",
                    |s: &mut i64| *s += 1,
                    |s: &mut i64| *s -= 1,
                ));
            }
            ctx.recover();
            black_box(ctx.is_clean());
        });
    });
    group.finish();
}

/// 打开内存 SQLite 连接 (provenance 存储依赖 kv_store 结构)。
fn open_memory_conn() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().expect("open memory db");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS kv_store (
            namespace TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (namespace, key)
        );",
    )
    .expect("create kv_store");
    conn
}

criterion_group!(
    capability_c3,
    bench_visibility_gate,
    bench_decision_trail,
    bench_revertible_effects,
);
criterion_main!(capability_c3);