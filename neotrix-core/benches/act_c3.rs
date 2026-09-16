// C3 benchmark 基线 — NT-ACT (行动执行者) 能力节点
//
// All benchmarks disabled: ToolSpec, ActionCache, DiskGuard were removed.
// This file is kept as a placeholder for future NT-ACT benchmarks.

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_placeholder(_c: &mut Criterion) {
    // Placeholder — real benchmarks to be re-added when NT-ACT types are re-stabilized.
}

criterion_group!(act_c3, bench_placeholder);
criterion_main!(act_c3);
