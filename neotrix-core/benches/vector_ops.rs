//! 向量操作基准测试
//! 测试 CapabilityVector 和 signal::ops 的性能

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neotrix::core::CapabilityVector;
use neotrix::neotrix::nt_core_signal::ops;

/// 生成随机向量（指定维度）
fn random_vector(dim: usize) -> Vec<f64> {
    (0..dim).map(|i| (i as f64 * 0.1).sin().abs().max(0.01)).collect()
}

/// 生成随机 CapabilityVector
fn random_capability_vector() -> CapabilityVector {
    let v = random_vector(23);
    let mut cv = CapabilityVector::default();
    cv.set_typography(v[0]);
    cv.set_grid(v[1]);
    cv.set_color(v[2]);
    cv.set_whitespace(v[3]);
    cv.set_data_viz(v[4]);
    cv.set_emotion(v[5]);
    cv.set_minimalism(v[6]);
    cv.set_experimental(v[7]);
    cv.set_inference_depth(v[8]);
    cv.set_creativity(v[9]);
    cv.set_analysis(v[10]);
    cv.set_synthesis(v[11]);
    cv.set_domain_specificity(v[12]);
    cv.set_accessibility(v[13]);
    cv.set_compound_composition(v[14]);
    cv.set_tailwind_proficiency(v[15]);
    cv.set_react_aria_usage(v[16]);
    cv.set_bem_naming(v[17]);
    cv.set_figma_integration(v[18]);
    cv.set_ai_native_states(v[19]);
    cv.set_semantic_layer(v[20]);
    cv.set_quality_gates(v[21]);
    cv.set_verification(v[22]);
    cv
}

fn benchmark_capability_vector_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("capability_vector");

    // 基准测试：to_array()
    group.bench_function("to_array_23d", |b| {
        let cv = random_capability_vector();
        b.iter(|| black_box(cv.to_array()))
    });

    // 基准测试：to_vector()
    group.bench_function("to_vector_23d", |b| {
        let cv = random_capability_vector();
        b.iter(|| black_box(cv.to_vector()))
    });

    // 基准测试：similarity() - 23维
    group.bench_function("similarity_23d", |b| {
        let cv1 = random_capability_vector();
        let cv2 = random_capability_vector();
        b.iter(|| black_box(cv1.similarity(&cv2)))
    });

    // 基准测试：normalize()
    group.bench_function("normalize_23d", |b| {
        b.iter(|| {
            let mut cv = random_capability_vector();
            cv.normalize();
            black_box(cv.clone())
        })
    });

    // 基准测试：from_array() + to_array()
    group.bench_function("from_array_23d", |b| {
        let arr = random_vector(23);
        b.iter(|| black_box(CapabilityVector::from_array(&arr).expect("invalid array for capability vector")))
    });

    group.finish();

    // 测试不同维度的 Vec 向量操作
    let mut group = c.benchmark_group("vec_ops");

    for dim in [22, 100, 1000].iter() {
        let dim = *dim;
        let v1 = random_vector(dim);
        let v2 = random_vector(dim);

        // 点积
        group.bench_function(&format!("dot_product_{}d", dim), |b| {
            b.iter(|| black_box(v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum::<f64>()))
        });

        // L2 范数
        group.bench_function(&format!("l2_norm_{}d", dim), |b| {
            b.iter(|| {
                let sum: f64 = v1.iter().map(|x| x * x).sum();
                black_box(sum.sqrt())
            })
        });

        // 余弦相似度 (使用 signal::ops)
        let vec1 = random_vector(dim);
        let vec2 = random_vector(dim);
        group.bench_function(&format!("cosine_similarity_{}d", dim), |b| {
            b.iter(|| black_box(ops::cosine_similarity(&vec1, &vec2)))
        });
    }

    group.finish();

    // 测试 signal::ops 函数
    let mut group = c.benchmark_group("signal_ops");

    let vec_22 = random_vector(22);
    let vec_100 = random_vector(100);
    let vec_1000 = random_vector(1000);

    for (name, vec) in [("22d", vec_22), ("100d", vec_100), ("1000d", vec_1000)] {
        let v = vec.clone();

        group.bench_function(&format!("softmax_{}", name), |b| {
            b.iter(|| black_box(ops::softmax(&v)))
        });

        group.bench_function(&format!("relu_{}", name), |b| {
            b.iter(|| black_box(ops::relu(&v)))
        });

        group.bench_function(&format!("sigmoid_{}", name), |b| {
            b.iter(|| black_box(ops::sigmoid(&v)))
        });

        group.bench_function(&format!("normalize_{}", name), |b| {
            b.iter(|| black_box(ops::normalize(&v)))
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_capability_vector_ops);
criterion_main!(benches);
