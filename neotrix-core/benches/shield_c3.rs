// C3 benchmark 基线 — NT-SHIELD (影卫) 能力节点 (C2→C3 晋升证据)
//
// 覆盖:
// - nt_shield_audit::SecurityAuditor      — run_static 全量清单审计吞吐 + calculate_score
// - nt_shield_audit::ReasoningTraceGuard  — strip_blocks 推理块剥离 + scan 泄露扫描 (P1)
// - nt_shield_comm::strip_internal        — 请求头内部字段剥离吞吐
//
// 目的: 建立 NT-SHIELD 性能基线 (C3 = benchmark 基线建立 + 无回归),
// 对比历史 (cargo bench --bench shield_c3)。

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use neotrix::neotrix::l1_body_impl::nt_shield_audit::{
    ReasoningTraceGuard, SecurityAuditor, TraceGuardMode,
};
use neotrix::neotrix::l1_body_impl::nt_shield_comm::strip_internal;

/// 构造 n 条含推理块/指纹/内部头的混合审计文本 (模拟真实请求行)。
fn make_trace_text(n: usize) -> String {
    (0..n)
        .map(|i| match i % 4 {
            0 => format!("msg-{i} <reasoning_cot>\ninternal plan {i}\n</reasoning_cot> tail"),
            1 => format!("msg-{i} <reasoning_encrypted> block-{i} </reasoning_encrypted> tail"),
            2 => "plain visible line with benign text".to_string(),
            _ => format!("msg-{i} <thinking>hidden {i}</thinking> visible"),
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// 构造 n 条内部请求头 (strip_internal 的典型输入形态: nt_ token / UUID / 本地路径)。
fn make_headers(n: usize) -> String {
    (0..n)
        .map(|i| {
            format!(
                "x-nt-request-id: req-{i}\nx-internal-token: tok-{i}\ntrace: nt_core_{i}\ncaller: neotrix-{i}\nuuid: {i:08x}-0000-0000-0000-{i:012x}\npath: /Users/neo/tmp/f-{i}\naccept: text/html"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn bench_audit_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("shield_audit");
    group.bench_function("run_static_full_checklist", |b| {
        b.iter(|| {
            let report = SecurityAuditor::run_static(black_box("bench-project"), "");
            black_box(report.total_checks);
        });
    });
    group.bench_function("calculate_score", |b| {
        let report = SecurityAuditor::run_static("bench-project", "");
        b.iter(|| black_box(SecurityAuditor::calculate_score(&report)));
    });
    group.finish();
}

fn bench_trace_guard(c: &mut Criterion) {
    let guard = ReasoningTraceGuard::with_mode(TraceGuardMode::Strip);
    let mut group = c.benchmark_group("shield_trace_guard");
    for &n in &[16usize, 64, 256] {
        let text = make_trace_text(n);
        group.bench_function(format!("strip_{n}blocks"), |b| {
            b.iter_batched(
                || text.clone(),
                |t| black_box(guard.strip_blocks(&t)),
                BatchSize::SmallInput,
            );
        });
        group.bench_function(format!("scan_{n}blocks"), |b| {
            b.iter_batched(
                || text.clone(),
                |t| black_box(guard.scan(&t).blocks_found),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_header_strip(c: &mut Criterion) {
    let mut group = c.benchmark_group("shield_comm");
    for &n in &[8usize, 32, 128] {
        let headers = make_headers(n);
        group.bench_function(format!("strip_internal_{n}reqs"), |b| {
            b.iter_batched(
                || headers.clone(),
                |h| black_box(strip_internal(&h)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    shield_c3,
    bench_audit_run,
    bench_trace_guard,
    bench_header_strip,
);
criterion_main!(shield_c3);
