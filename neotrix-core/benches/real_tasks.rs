//! 真实场景基准测试
//!
//! 测量实际任务吞吐量：知识来源枚举、能力向量计算、相似度匹配

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neotrix::core::{KnowledgeSource, CapabilityVector};

/// 遍历所有 40 个知识来源并测量名称获取
fn benchmark_knowledge_source_names(c: &mut Criterion) {
    c.bench_function("knowledge_source_names", |b| {
        let sources = KnowledgeSource::all();
        b.iter(|| {
            for source in black_box(&sources) {
                black_box(source.name());
            }
        })
    });
}

/// 测量所有知识来源的 source_weight 计算
fn benchmark_source_weights(c: &mut Criterion) {
    c.bench_function("knowledge_source_weights", |b| {
        let sources = KnowledgeSource::all();
        b.iter(|| {
            for source in black_box(&sources) {
                black_box(source.source_weight());
            }
        })
    });
}

/// 测量所有知识来源的能力向量计算
fn benchmark_capability_vectors(c: &mut Criterion) {
    c.bench_function("capability_vectors_all_sources", |b| {
        let sources = KnowledgeSource::all();
        b.iter(|| {
            for source in black_box(&sources) {
                let cv: CapabilityVector = source.capability_vector();
                black_box(cv);
            }
        })
    });
}

/// 测量所有 sources 与默认向量的相似度
fn benchmark_source_similarity(c: &mut Criterion) {
    c.bench_function("source_similarity_all", |b| {
        let cv = CapabilityVector::default();
        let sources = KnowledgeSource::all();
        b.iter(|| {
            for source in black_box(&sources) {
                let sv = source.capability_vector();
                let sim = cv.similarity(&sv);
                black_box(sim);
            }
        })
    });
}

criterion_group! {
    name = real_tasks;
    config = Criterion::default().sample_size(30);
    targets =
        benchmark_knowledge_source_names,
        benchmark_source_weights,
        benchmark_capability_vectors,
        benchmark_source_similarity,
}

criterion_main!(real_tasks);
