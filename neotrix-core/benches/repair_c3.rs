// C3 benchmark 基线 — NT-REPAIR (自愈工程师) 能力节点 (C2→C3 晋升证据)
//
// 覆盖:
// - nt_repair_causal_trace::CausalChainWalker — walk 因果链步行吞吐
// - nt_repair_causal_trace::SourceAdjudicator — adjudicate 单一赢家源裁决
// - nt_repair_causal_trace::EvidenceGate      — evaluate 证据门禁 (with_default_rules)
//
// 目的: 建立 NT-REPAIR 性能基线 (C3 = benchmark 基线建立 + 无回归),
// 对比历史 (cargo bench --bench repair_c3)。

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use neotrix::neotrix::l8_autonomic_impl::nt_repair_causal_trace::{
    default_adjudicator, CausalChainWalker, CausalNode, EvidenceGate,
};

/// 构造 depth 深进程链: target 在叶端, 根为 "systemd" (命中高特异性检测器)。
fn make_chain(depth: usize) -> Vec<CausalNode> {
    let mut nodes: Vec<CausalNode> = (0..depth)
        .map(|i| {
            let name = if i == 0 { "systemd".to_string() } else { format!("proc-{i}") };
            CausalNode::new(format!("p{i}"), name).with_evidence(format!("evidence line {i}: /sbin/init"))
        })
        .collect();
    for i in (1..depth).rev() {
        let parent = nodes[i - 1].id.clone();
        nodes[i].parent = Some(parent);
    }
    nodes
}

fn bench_chain_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("repair_chain_walk");
    for &depth in &[16usize, 64, 256] {
        let chain = make_chain(depth);
        let target = chain.last().unwrap().clone();
        let nodes: std::collections::HashMap<String, CausalNode> =
            chain.iter().map(|n| (n.id.clone(), n.clone())).collect();
        group.bench_function(format!("walk_depth{depth}"), |b| {
            b.iter_batched(
                || target.clone(),
                |t| {
                    let walker = CausalChainWalker::new(512);
                    let resolved = walker.walk(&t, &mut |id: &str| nodes.get(id).cloned());
                    black_box(resolved.len());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_source_adjudication(c: &mut Criterion) {
    let adjudicator = default_adjudicator();
    let mut group = c.benchmark_group("repair_source_adjudication");
    for &depth in &[16usize, 64, 256] {
        let chain = make_chain(depth);
        group.bench_function(format!("adjudicate_depth{depth}"), |b| {
            b.iter(|| black_box(adjudicator.adjudicate(&chain).is_identified()));
        });
    }
    group.finish();
}

fn bench_evidence_gate(c: &mut Criterion) {
    let gate = EvidenceGate::with_default_rules();
    let mut group = c.benchmark_group("repair_evidence_gate");
    for &depth in &[16usize, 64, 256] {
        let chain = make_chain(depth);
        group.bench_function(format!("evaluate_depth{depth}"), |b| {
            b.iter(|| black_box(gate.evaluate(&chain).len()));
        });
        group.bench_function(format!("is_clean_depth{depth}"), |b| {
            b.iter(|| black_box(gate.is_clean(&chain)));
        });
    }
    group.finish();
}

criterion_group!(
    repair_c3,
    bench_chain_walk,
    bench_source_adjudication,
    bench_evidence_gate,
);
criterion_main!(repair_c3);
