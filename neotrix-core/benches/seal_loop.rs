//! SEAL 循环和 ReasoningBank 基准测试

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use neotrix::neotrix::nt_mind::self_edit::SelfEdit;
use neotrix::neotrix::nt_mind::{
    core::{CapabilityVector, KnowledgeSource},
    memory::{ReasoningBank, ReasoningMemory},
    self_iterating::SelfIteratingBrain,
};
use std::time::Duration;

/// 生成测试用的 SelfEdit
fn make_test_edit(_task: &str) -> SelfEdit {
    SelfEdit {
        task_type: neotrix::neotrix::nt_expert_routing::TaskType::Design,
        target_dimensions: vec!["typography".to_string(), "color".to_string()],
        adjustment_magnitude: 0.1,
        tool_calls: vec![],
        config_overrides: std::collections::HashMap::new(),
    }
}

/// 填充 ReasoningBank 并测试检索性能
fn benchmark_reasoning_bank_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("reasoning_bank_retrieval");
    group.measurement_time(Duration::from_secs(5));

    for &mem_count in &[10usize, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("retrieve_relevant", mem_count),
            &mem_count,
            |b, &mem_count| {
                let mut bank = ReasoningBank::new(mem_count * 2);

                // 填充记忆
                for i in 0..mem_count {
                    let mut memory = ReasoningMemory::from_self_edit(
                        &format!("设计任务 {}", i),
                        neotrix::neotrix::nt_expert_routing::TaskType::Design,
                        &make_test_edit(&format!("task {}", i)),
                        if i % 2 == 0 { 0.8 } else { 0.3 },
                    );
                    memory.embedding = Some(vec![i as f64 / mem_count as f64; 23]);
                    bank.store(memory);
                }

                b.iter(|| black_box(bank.retrieve_relevant("设计", None, 5)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("retrieve_by_embedding", mem_count),
            &mem_count,
            |b, &mem_count| {
                let mut bank = ReasoningBank::new(mem_count * 2);

                // 填充记忆（带 embedding）
                for i in 0..mem_count {
                    let mut memory = ReasoningMemory::from_self_edit(
                        &format!("任务 {}", i),
                        neotrix::neotrix::nt_expert_routing::TaskType::Design,
                        &make_test_edit(&format!("task {}", i)),
                        0.5,
                    );
                    // 创建不同的 embedding
                    let emb: Vec<f64> = (0..23)
                        .map(|j| ((i * 7 + j * 3) as f64).sin().abs())
                        .collect();
                    memory.embedding = Some(emb);
                    bank.store(memory);
                }

                let query_emb: Vec<f64> = (0..23).map(|j| ((j * 5) as f64).sin().abs()).collect();

                b.iter(|| black_box(bank.retrieve_relevant_by_embedding(&query_emb, None, 5)))
            },
        );
    }

    group.finish();
}

fn benchmark_seal_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("seal_loop");
    group.measurement_time(Duration::from_secs(10));

    // 基准测试：单次 SEAL 循环
    group.bench_function("run_seal_loop_single", |b| {
        let mut brain = SelfIteratingBrain::new();
        brain.brain.learning_rate = 0.05;
        brain.reasoning_bank = ReasoningBank::new(100);

        // 预填充一些记忆
        for i in 0..10 {
            let memory = ReasoningMemory::from_self_edit(
                &format!("历史任务 {}", i),
                neotrix::neotrix::nt_expert_routing::TaskType::Design,
                &make_test_edit(&format!("task {}", i)),
                0.7,
            );
            brain.reasoning_bank.store(memory);
        }

        b.iter(|| {
            let mut brain = SelfIteratingBrain::new();
            brain.brain.learning_rate = 0.05;
            // 克隆 ReasoningBank（性能测试中包含克隆开销）
            black_box(brain.run_seal_loop("设计一个响应式 UI 界面", None, None))
        })
    });

    // 基准测试：批量 SEAL 循环
    group.bench_function("run_seal_loop_batch_3", |b| {
        let tasks: Vec<(String, Option<Vec<f64>>, Option<f64>)> = vec![
            ("设计 React 组件".to_string(), None, None),
            ("分析代码性能".to_string(), None, None),
            ("优化数据库查询".to_string(), None, None),
        ];

        b.iter(|| {
            let mut brain = SelfIteratingBrain::new();
            brain.brain.learning_rate = 0.05;
            black_box(brain.run_seal_loop_batch(&tasks))
        })
    });

    // 基准测试：带有 embedding 的 SEAL 循环
    group.bench_function("run_seal_loop_with_embedding", |b| {
        let embedding: Vec<f64> = (0..23).map(|i| (i as f64 * 0.1).sin().abs()).collect();

        b.iter(|| {
            let mut brain = SelfIteratingBrain::new();
            brain.brain.learning_rate = 0.05;
            black_box(brain.run_seal_loop("设计 UI", Some(embedding.clone()), None))
        })
    });

    group.finish();
}

fn benchmark_capability_vector_similarity_seal(c: &mut Criterion) {
    let mut group = c.benchmark_group("capability_similarity");

    // 测试不同数量知识源的相似度计算
    let sources = [
        KnowledgeSource::HeroUI,
        KnowledgeSource::BaseUI,
        KnowledgeSource::ArcUI,
        KnowledgeSource::CortexUI,
        KnowledgeSource::AgenticDS,
        KnowledgeSource::DesignPhilosophy,
    ];

    group.bench_function("similarity_knowledge_sources", |b| {
        let cv = CapabilityVector::default();
        let source_vecs: Vec<CapabilityVector> = sources
            .iter()
            .map(|s: &KnowledgeSource| s.capability_vector())
            .collect();

        b.iter(|| {
            for sv in &source_vecs {
                black_box(cv.similarity(sv));
            }
        })
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(Duration::from_secs(2));

    targets =
        benchmark_reasoning_bank_retrieval,
        benchmark_seal_loop,
        benchmark_capability_vector_similarity_seal
}

criterion_main!(benches);
