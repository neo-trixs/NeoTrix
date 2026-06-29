use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neotrix::core::nt_core_hcube::kroneker_cleanup::KronekerCodebook;
use neotrix::core::nt_core_hcube::linear_code::LinearCodeConfig;
use neotrix::core::nt_core_hcube::linear_code::LinearCodeVSA;
use neotrix::core::nt_core_hcube::resonator_decoder::ResonatorDecoder;
use neotrix::core::nt_core_hcube::sparse_vsa::SparseBinaryVSA;
use neotrix::core::nt_core_hcube::QuantizedVSA;
use neotrix::core::nt_core_hcube::{hamming_distance_packed, pack_binary, VSA_DIM};

// ============================================================
// Helpers
// ============================================================

fn random_binary_vec() -> Vec<u8> {
    QuantizedVSA::random_binary()
}

fn random_seeded_vec(seed: u64) -> Vec<u8> {
    QuantizedVSA::seeded_random(seed, VSA_DIM)
}

fn make_codebook(seed: u64, size: usize) -> Vec<Vec<u8>> {
    (0..size)
        .map(|i| QuantizedVSA::seeded_random(seed + i as u64 * 100, 1024))
        .collect()
}

fn make_labels(prefix: &str, size: usize) -> Vec<String> {
    (0..size).map(|i| format!("{}_{}", prefix, i)).collect()
}

fn xor_bind(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

fn majority_bundle(vectors: &[&[u8]]) -> Vec<u8> {
    if vectors.is_empty() {
        return Vec::new();
    }
    let n = vectors.len();
    let len = vectors[0].len();
    let total_bits = len * 8;
    let mut counts = vec![0i32; total_bits];
    for v in vectors {
        for bit in 0..total_bits.min(v.len() * 8) {
            if (v[bit / 8] >> (bit % 8)) & 1 == 1 {
                counts[bit] += 1;
            }
        }
    }
    let threshold = (n as i32) / 2;
    let mut result = vec![0u8; len];
    for bit in 0..total_bits {
        if counts[bit] > threshold {
            result[bit / 8] |= 1 << (bit % 8);
        }
    }
    result
}

// ============================================================
// Group 1: QuantizedVSA operations
// ============================================================

fn bench_quantized_vsa(c: &mut Criterion) {
    let a = random_binary_vec();
    let b = random_binary_vec();
    let packed_a = pack_binary(&a);
    let packed_b = pack_binary(&b);

    let mut group = c.benchmark_group("quantized_vsa_4096");

    group.bench_function("xor_bind", |bench| {
        bench.iter(|| QuantizedVSA::xor_bind(black_box(&a), black_box(&b)))
    });

    group.bench_function("bundle_2", |bench| {
        bench.iter(|| QuantizedVSA::bundle(black_box(&[&a, &b])))
    });

    {
        let c = random_binary_vec();
        let d = random_binary_vec();
        let e = random_binary_vec();
        let f = random_binary_vec();
        let g = random_binary_vec();
        group.bench_function("bundle_5", |bench| {
            bench.iter(|| QuantizedVSA::bundle(black_box(&[&a, &c, &d, &e, &f])))
        });
        group.bench_function("bundle_10", |bench| {
            let h = random_binary_vec();
            let i = random_binary_vec();
            let j = random_binary_vec();
            let k = random_binary_vec();
            let l = random_binary_vec();
            bench
                .iter(|| QuantizedVSA::bundle(black_box(&[&a, &c, &d, &e, &f, &g, &h, &i, &j, &k])))
        });
    }

    group.bench_function("hamming_distance", |bench| {
        bench.iter(|| QuantizedVSA::hamming_distance(black_box(&a), black_box(&b)))
    });

    group.bench_function("similarity", |bench| {
        bench.iter(|| QuantizedVSA::similarity(black_box(&a), black_box(&b)))
    });

    group.bench_function("cosine", |bench| {
        bench.iter(|| QuantizedVSA::cosine(black_box(&a), black_box(&b)))
    });

    group.bench_function("permute", |bench| {
        bench.iter(|| QuantizedVSA::permute(black_box(&a), black_box(127isize)))
    });

    group.bench_function("pack_binary", |bench| {
        bench.iter(|| pack_binary(black_box(&a)))
    });

    group.bench_function("hamming_distance_packed", |bench| {
        bench.iter(|| hamming_distance_packed(black_box(&packed_a), black_box(&packed_b)))
    });

    group.finish();
}

// ============================================================
// Group 2: SparseBinaryVSA operations (K=32)
// ============================================================

type SparseVSA = SparseBinaryVSA<4096, 32>;

fn bench_sparse_vsa(c: &mut Criterion) {
    let a = SparseVSA::random(42);
    let b = SparseVSA::random(123);
    let dense = a.to_dense();

    let mut group = c.benchmark_group("sparse_vsa_4096_k32");

    group.bench_function("bind", |bench| {
        bench.iter(|| SparseVSA::bind(black_box(&a), black_box(&b)))
    });

    {
        let c = SparseVSA::random(200);
        let d = SparseVSA::random(300);
        let e = SparseVSA::random(400);
        group.bench_function("bundle_5", |bench| {
            bench.iter(|| SparseVSA::bundle(black_box(&[&a, &b, &c, &d, &e])))
        });
    }

    group.bench_function("similarity", |bench| {
        bench.iter(|| SparseVSA::similarity(black_box(&a), black_box(&b)))
    });

    group.bench_function("permute", |bench| {
        bench.iter(|| SparseVSA::permute(black_box(&a), black_box(127isize)))
    });

    group.bench_function("to_dense", |bench| bench.iter(|| a.to_dense()));

    group.bench_function("from_dense", |bench| {
        bench.iter(|| SparseVSA::from_dense(black_box(&dense)))
    });

    group.finish();
}

// ============================================================
// Group 3: LinearCodeVSA operations
// ============================================================

fn bench_linear_code_vsa(c: &mut Criterion) {
    let vsa = LinearCodeVSA::new(LinearCodeConfig {
        dim: 256,
        code_rate: 0.25,
    });
    let k_bytes = (vsa.k() + 7) / 8;

    let msg = vec![0xABu8; k_bytes];
    let cw = vsa.encode(&msg);
    let cw2 = vsa.encode(&vec![0xCDu8; k_bytes]);
    let mut noisy = cw.clone();
    if !noisy.is_empty() {
        noisy[0] ^= 0x01;
    }

    let mut group = c.benchmark_group("linear_code_vsa_256");

    group.bench_function("encode", |bench| bench.iter(|| vsa.encode(black_box(&msg))));

    group.bench_function("decode", |bench| {
        bench.iter(|| vsa.decode(black_box(&noisy)))
    });

    group.bench_function("bind", |bench| {
        bench.iter(|| xor_bind(black_box(&cw), black_box(&cw2)))
    });

    {
        let cw3 = vsa.encode(&vec![0xEFu8; k_bytes]);
        let cw4 = vsa.encode(&vec![0x01u8; k_bytes]);
        let cw5 = vsa.encode(&vec![0x02u8; k_bytes]);
        group.bench_function("bundle_5", |bench| {
            bench.iter(|| majority_bundle(black_box(&[&cw, &cw2, &cw3, &cw4, &cw5])))
        });
    }

    group.bench_function("similarity", |bench| {
        bench.iter(|| vsa.similarity(black_box(&cw), black_box(&cw2)))
    });

    group.finish();
}

// ============================================================
// Group 4: KronekerCodebook cleanup
// ============================================================

fn bench_kroneker_cleanup(c: &mut Criterion) {
    let query = random_binary_vec();

    let mut group = c.benchmark_group("kroneker_cleanup");

    for &k in &[16usize, 64, 256] {
        let mut cb = KronekerCodebook::new(k);
        cb.add_seed(42);
        cb.add_seed(123);
        group.bench_with_input(
            format!("k={}", k),
            &(cb, query.as_slice()),
            |b, (codebook, q)| b.iter(|| codebook.cleanup(black_box(q), black_box(5))),
        );
    }

    group.finish();
}

// ============================================================
// Group 5: ResonatorDecoder
// ============================================================

fn bench_resonator_decoder(c: &mut Criterion) {
    let mut group = c.benchmark_group("resonator_decoder");

    // 2-factor, codebook size 16 each
    {
        let cb1 = make_codebook(1, 16);
        let cb2 = make_codebook(100, 16);
        let l1 = make_labels("a", 16);
        let l2 = make_labels("b", 16);
        let v1 = QuantizedVSA::seeded_random(2, 1024);
        let v2 = QuantizedVSA::seeded_random(150, 1024);
        let bundle = QuantizedVSA::bundle(&[&v1, &v2]);
        let decoder = ResonatorDecoder::new(vec![cb1, cb2], vec![l1, l2], 10);

        group.bench_function("decode_2factor_x16", |bench| {
            bench.iter(|| decoder.decode(black_box(&bundle)))
        });
    }

    // 3-factor, codebook size 8 each
    {
        let cb = make_codebook(1, 8);
        let l = make_labels("f", 8);
        let v1 = QuantizedVSA::seeded_random(5, 1024);
        let v2 = QuantizedVSA::seeded_random(300, 1024);
        let v3 = QuantizedVSA::seeded_random(600, 1024);
        let bundle = QuantizedVSA::bundle(&[&v1, &v2, &v3]);
        let decoder = ResonatorDecoder::new(
            vec![cb.clone(), cb.clone(), cb],
            vec![l.clone(), l.clone(), l],
            10,
        );

        group.bench_function("decode_3factor_x8", |bench| {
            bench.iter(|| decoder.decode(black_box(&bundle)))
        });
    }

    group.finish();
}

// ============================================================
// Criterion Groups
// ============================================================

criterion_group!(
    benches,
    bench_quantized_vsa,
    bench_sparse_vsa,
    bench_linear_code_vsa,
    bench_kroneker_cleanup,
    bench_resonator_decoder,
);

criterion_main!(benches);
