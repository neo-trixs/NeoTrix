// C3 benchmark 基线 — NT-MEMORY (知识守护者) 能力节点 (C2→C3 晋升证据)
//
// 覆盖:
// - nt_memory_embed::cosine_similarity    — 高维向量相似度吞吐
// - nt_memory_embed::local_embed_texts    — 批量局部嵌入 (无外部 API)
// - nt_memory_graph_cache::GraphCache     — 图缓存插入 + neighbors/adjacent_ids
// - nt_memory_graph_cache::weighted_shortest_path — 加权最短路径 (Dijkstra)
//
// 目的: 建立 NT-MEMORY 性能基线 (C3 = benchmark 基线建立 + 无回归),
// 对比历史 (cargo bench --bench memory_c3)。

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_embed::{
    cosine_similarity, local_embed_texts,
};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_graph_cache::{
    weighted_shortest_path, GraphCache,
};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::{KnowledgeEdge, RelationType};

fn bench_cosine(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_cosine");
    for &dim in &[64usize, 256, 512] {
        let va: Vec<f32> = (0..dim).map(|i| (i as f32) / dim as f32).collect();
        let vb: Vec<f32> = (0..dim).map(|i| 1.0 - (i as f32) / dim as f32).collect();
        group.bench_function(format!("cosine_dim{dim}"), |b| {
            b.iter(|| black_box(cosine_similarity(&va, &vb)));
        });
    }
    group.finish();
}

fn bench_local_embed(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_local_embed");
    for &(n, dim) in &[(4usize, 32usize), (16, 64), (64, 64)] {
        let owned: Vec<String> = (0..n)
            .map(|i| format!("knowledge node {i}: hybrid VSA HyperCube embedding baseline"))
            .collect();
        let texts: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        group.bench_function(format!("embed_{n}x{dim}"), |b| {
            b.iter_batched(
                || texts.clone(),
                |t| {
                    let v = local_embed_texts(&t, dim);
                    black_box(v.len());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// 构造 n×n 网格图: 每节点向右/向下连边 (Dijkstra 典型稠密短路径结构)。
fn make_grid_cache(n: usize) -> GraphCache {
    let mut cache = GraphCache::empty();
    for r in 0..n {
        for c in 0..n {
            let id = format!("n{r}_{c}");
            for (dr, dc) in [(0, 1), (1, 0)] {
                let (nr, nc) = (r + dr, c + dc);
                if nr < n && nc < n {
                    cache.insert_edge(KnowledgeEdge {
                        id: format!("{id}->n{nr}_{nc}"),
                        source_id: id.clone(),
                        target_id: format!("n{nr}_{nc}"),
                        relation_type: RelationType::DependsOn,
                        weight: 1.0,
                        description: None,
                        created_at: 0,
                        metadata: None,
                    });
                }
            }
        }
    }
    cache
}

fn bench_graph_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_graph_cache");
    for &n in &[10usize, 20, 40] {
        let cache = make_grid_cache(n);
        let node_count = n * n;
        group.bench_function(format!("neighbors_{node_count}nodes"), |b| {
            b.iter(|| {
                let mut acc = 0usize;
                for r in 0..n {
                    for c in 0..n {
                        acc += cache.neighbors(&format!("n{r}_{c}")).len();
                    }
                }
                black_box(acc);
            });
        });
        group.bench_function(format!("wsp_{node_count}nodes"), |b| {
            b.iter(|| {
                black_box(weighted_shortest_path(
                    &cache,
                    "n0_0",
                    &format!("n{}_{}", n - 1, n - 1),
                ))
            });
        });
    }
    group.finish();
}

criterion_group!(memory_c3, bench_cosine, bench_local_embed, bench_graph_cache);
criterion_main!(memory_c3);
