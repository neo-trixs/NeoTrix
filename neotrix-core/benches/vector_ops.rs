use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neotrix::core::CapabilityVector;
use neotrix::neotrix::nt_core_signal::ops;

fn random_vector(dim: usize) -> Vec<f64> {
    (0..dim)
        .map(|i| (i as f64 * 0.1).sin().abs().max(0.01))
        .collect()
}

fn benchmark_signal_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_ops");
    for dim in [22, 100, 1000].iter() {
        let dim = *dim;
        let v1 = random_vector(dim);
        let v2 = random_vector(dim);
        group.bench_function(&format!("dot_product_{}d", dim), |b| {
            b.iter(|| {
                let sum: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
                black_box(sum)
            })
        });
        group.bench_function(&format!("cosine_similarity_{}d", dim), |b| {
            b.iter(|| black_box(ops::cosine_similarity(&v1, &v2)))
        });
    }
    group.finish();
    let mut group = c.benchmark_group("activation");
    for dim in [22, 100, 1000].iter() {
        let v = random_vector(*dim);
        group.bench_function(&format!("softmax_{}d", dim), |b| {
            b.iter(|| black_box(ops::softmax(&v)))
        });
        group.bench_function(&format!("relu_{}d", dim), |b| {
            b.iter(|| black_box(ops::relu(&v)))
        });
        group.bench_function(&format!("sigmoid_{}d", dim), |b| {
            b.iter(|| black_box(ops::sigmoid(&v)))
        });
        group.bench_function(&format!("normalize_{}d", dim), |b| {
            b.iter(|| black_box(ops::normalize(&v)))
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_signal_ops);
criterion_main!(benches);
