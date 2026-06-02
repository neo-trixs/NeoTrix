use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use neotrix::CapabilityVector;
use neotrix::core::hypercube::{VSAEngine, VsaBackend};
use neotrix::neotrix::jepa_world_model::{JepaEncoder, JepaPredictor, JepaWorldModel};
use neotrix::neotrix::nt_mind_evolution_loop::EvolutionLoop;

// ============================================================
// Helpers
// ============================================================

fn make_jepa_model(latent_dim: usize) -> JepaWorldModel {
    let input_dim = 64;
    let hidden_dim = latent_dim * 2;
    let mut model = JepaWorldModel::new(input_dim);
    model.latent_dim = latent_dim;
    model.predictor = JepaPredictor::new(latent_dim, hidden_dim);
    model.context_encoder = JepaEncoder::new(input_dim, latent_dim);
    model.target_encoder = JepaEncoder::new(input_dim, latent_dim);
    model.td_target_critic = (0..latent_dim).map(|_| rand::random::<f64>() * 0.1).collect();
    model
}

fn random_latent(dim: usize) -> Vec<f64> {
    (0..dim).map(|_| (rand::random::<f64>() - 0.5) * 2.0).collect()
}

fn random_vsa_vec(dim: usize) -> Vec<f64> {
    (0..dim).map(|i| (i as f64).sin()).collect()
}

fn random_capability() -> CapabilityVector {
    CapabilityVector::from_values(
        0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.9,
        0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.9, 0.8, 0.7, 0.6, 0.5,
    )
}

// ============================================================
// 1. JEPA World Model Prediction
// ============================================================

fn bench_jepa_predict_next_latent(c: &mut Criterion) {
    let dims = [64usize, 128, 256];

    let mut group = c.benchmark_group("jepa_predict_next_latent");
    for &dim in &dims {
        let model = make_jepa_model(dim);
        let latent = random_latent(dim);
        group.throughput(Throughput::Elements(dim as u64));
        group.bench_with_input(format!("dim_{}", dim), &(model, latent), |b, (m, z)| {
            b.iter(|| m.predict_next_latent(black_box(z)))
        });
    }
    group.finish();
}

// ============================================================
// 2. TD-JEPA N-Step
// ============================================================

fn bench_jepa_td_predict_n(c: &mut Criterion) {
    let horizons = [1usize, 5, 10];
    let model = make_jepa_model(64);
    let obs: Vec<f64> = (0..64).map(|_| rand::random::<f64>()).collect();

    let mut group = c.benchmark_group("jepa_td_predict_n");
    for &h in &horizons {
        group.throughput(Throughput::Elements(h as u64));
        group.bench_with_input(format!("horizon_{}", h), &(model.clone(), h, &obs), |b, (m, h, o)| {
            b.iter(|| m.td_predict_n(black_box(*h), black_box(&[]), black_box(o)))
        });
    }
    group.finish();
}

// ============================================================
// 3. Hypercube VSA Operations
// ============================================================

fn bench_vsa_operations(c: &mut Criterion) {
    let engine = VSAEngine::new(4096);
    let a = random_vsa_vec(4096);
    let b = random_vsa_vec(4096);

    let mut group = c.benchmark_group("vsa_4096");
    group.throughput(Throughput::Elements(4096));

    group.bench_function("bind", |bench| {
        bench.iter(|| engine.bind(black_box(&a), black_box(&b)))
    });

    group.bench_function("bundle", |bench| {
        bench.iter(|| engine.bundle(black_box(&[&a, &b])))
    });

    group.bench_function("cosine_similarity", |bench| {
        bench.iter(|| engine.similarity(black_box(&a), black_box(&b)))
    });

    group.finish();
}

// ============================================================
// 4. CapabilityVector Operations
// ============================================================

fn bench_capability_vector_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("capability_vector");

    group.bench_function("normalize", |bench| {
        let mut cv = random_capability();
        bench.iter(|| cv.normalize())
    });

    group.bench_function("cosine_similarity", |bench| {
        let cv1 = random_capability();
        let cv2 = random_capability();
        bench.iter(|| cv1.similarity(black_box(&cv2)))
    });

    group.bench_function("update_from_other", |bench| {
        let mut cv1 = random_capability();
        let cv2 = random_capability();
        bench.iter(|| cv1.update_from_other(black_box(&cv2), black_box(0.1)))
    });

    group.finish();
}

// ============================================================
// 5. Evolution Loop Iteration
// ============================================================

fn bench_evolution_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("evolution_loop");

    group.bench_function("run_cycle", |bench| {
        let mut evo = EvolutionLoop::new();
        bench.iter(|| {
            let _report = evo.run_cycle(black_box(None), black_box(None));
            // Reset after each iteration to avoid accumulated state
            evo = EvolutionLoop::new();
        })
    });

    group.finish();
}

// ============================================================
// Criterion Groups
// ============================================================

criterion_group!(jepa, bench_jepa_predict_next_latent, bench_jepa_td_predict_n);
criterion_group!(vsa, bench_vsa_operations);
criterion_group!(capability, bench_capability_vector_ops);
criterion_group!(evolution, bench_evolution_loop);

criterion_main!(jepa, vsa, capability, evolution);
