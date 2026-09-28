use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, black_box};
use neotrix::l2_perception::nt_core_knowledge::{
    KnowledgeSource, KSActivationEngine, SourceAccessTracker, TaskType,
};
use neotrix::l2_perception::nt_core_vector_store::{
    BruteForceVectorStore, IndexConfig, VectorRecord, VectorStore,
};
use neotrix::l5_cognition::nt_core::capability::types::CapabilityVector;
use std::collections::HashMap;

fn bench_memory_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_store");

    for num_entries in [10, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("tracker_record_access", num_entries),
            &num_entries,
            |b, &n| {
                b.iter_batched(
                    || {
                        let tracker = SourceAccessTracker::new(3);
                        let sources = KnowledgeSource::all();
                        (tracker, sources, n)
                    },
                    |(mut tracker, sources, n)| {
                        for i in 0..n {
                            let src = sources[i % sources.len()];
                            tracker.record_access(black_box(&src));
                        }
                        black_box(&tracker);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_memory_recall(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_recall");

    for num_sources in [10, 50] {
        group.bench_with_input(
            BenchmarkId::new("activation_engine_select", num_sources),
            &num_sources,
            |b, &n| {
                b.iter_batched(
                    || {
                        let mut engine = KSActivationEngine::new();
                        let sources = KnowledgeSource::all();
                        let query = CapabilityVector::from_array(&[0.5; 23])
                            .expect("valid value in benchmark");
                        // Pre-populate with hits to make some sources "mature"
                        for i in 0..n.min(sources.len()) {
                            let src = sources[i];
                            engine.record_hit(src);
                        }
                        (engine, query)
                    },
                    |(engine, query)| {
                        let result = engine.select(black_box(TaskType::General), black_box(&query));
                        black_box(result);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_entity_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_extraction");

    group.bench_function("knowledge_source_from_name", |b| {
        let names: Vec<String> = KnowledgeSource::all()
            .iter()
            .map(|s| format!("{:?}", s))
            .collect();
        b.iter(|| {
            for name in &names {
                black_box(KnowledgeSource::from_name(black_box(name)));
            }
        });
    });

    group.bench_function("capability_vector_lookup", |b| {
        let sources = KnowledgeSource::all();
        b.iter(|| {
            for src in &sources {
                black_box(src.capability_vector());
            }
        });
    });

    group.finish();
}

fn bench_consolidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("consolidation");

    group.bench_function("tracker_prune_cold", |b| {
        b.iter_batched(
            || {
                let mut tracker = SourceAccessTracker::new(3);
                let sources = KnowledgeSource::all();
                // Access first half heavily, leave second half cold
                for src in &sources[..sources.len() / 2] {
                    for _ in 0..5 {
                        tracker.record_access(src);
                    }
                }
                tracker
            },
            |tracker| {
                black_box(tracker.prune_cold(black_box(3)));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("tracker_sort_by_access", |b| {
        b.iter_batched(
            || {
                let mut tracker = SourceAccessTracker::new(1);
                let sources = KnowledgeSource::all();
                for (i, src) in sources.iter().enumerate() {
                    for _ in 0..i % 10 {
                        tracker.record_access(src);
                    }
                }
                let targets: Vec<KnowledgeSource> = sources.into_iter().take(30).collect();
                (tracker, targets)
            },
            |(tracker, targets)| {
                black_box(KnowledgeSource::sort_by_access(black_box(&targets), &tracker));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("lifecycle_report", |b| {
        b.iter_batched(
            || {
                let mut engine = KSActivationEngine::new();
                let sources = KnowledgeSource::all();
                for src in &sources[..20] {
                    for _ in 0..5 {
                        engine.record_hit(*src);
                    }
                }
                engine
            },
            |engine| {
                black_box(engine.lifecycle_report());
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_vector_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_search");

    for num_vectors in [100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("ivf_store_search", num_vectors),
            &num_vectors,
            |b, &n| {
                b.iter_batched(
                    || {
                        let config = IndexConfig {
                            num_partitions: 4,
                            ..IndexConfig::default()
                        };
                        let mut store = IvfVectorStore::new(config);
                        use rand::{Rng, SeedableRng};
                        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
                        for i in 0..n {
                            let v: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
                            store
                                .insert(VectorRecord::new(format!("vec_{}", i), v))
                                .unwrap();
                        }
                        let query: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
                        (store, query)
                    },
                    |(store, query)| {
                        black_box(store.search(black_box(&query), 10));
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.bench_function("bruteforce_store_search_1000", |b| {
        b.iter_batched(
            || {
                let config = IndexConfig::default();
                let mut store = BruteForceVectorStore::new(config);
                use rand::{Rng, SeedableRng};
                let mut rng = rand::rngs::StdRng::seed_from_u64(42);
                for i in 0..1000 {
                    let v: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
                    store
                        .insert(VectorRecord::new(format!("vec_{}", i), v))
                        .unwrap();
                }
                let query: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
                (store, query)
            },
            |(store, query)| {
                black_box(store.search(black_box(&query), 10));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("ivf_search_with_filter", |b| {
        b.iter_batched(
            || {
                let config = IndexConfig {
                    num_partitions: 4,
                    ..IndexConfig::default()
                };
                let mut store = IvfVectorStore::new(config);
                use rand::{Rng, SeedableRng};
                let mut rng = rand::rngs::StdRng::seed_from_u64(42);
                for i in 0..500 {
                    let v: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
                    let mut meta = HashMap::new();
                    meta.insert(
                        "domain".to_string(),
                        if i % 2 == 0 {
                            "science".to_string()
                        } else {
                            "art".to_string()
                        },
                    );
                    store
                        .insert(VectorRecord::new(format!("vec_{}", i), v).with_metadata(meta))
                        .unwrap();
                }
                let query: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
                let mut filter = HashMap::new();
                filter.insert("domain".to_string(), "science".to_string());
                (store, query, filter)
            },
            |(store, query, filter)| {
                black_box(store.search_with_filter(black_box(&query), 10, &filter));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

use neotrix::l2_perception::nt_core_vector_store::IvfVectorStore;

criterion_group!(
    benches,
    bench_memory_store,
    bench_memory_recall,
    bench_entity_extraction,
    bench_consolidation,
    bench_vector_search,
);
criterion_main!(benches);
