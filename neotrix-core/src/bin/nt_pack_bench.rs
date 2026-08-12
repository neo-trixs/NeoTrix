//! NT-Pack 性能基准 (手写 std::time::Instant, C3 成熟度)
//!
//! 覆盖: 编码/解码吞吐 + 压缩率 (E5 无压缩 vs E5+zstd)。
//! 用法: cargo run --release -p neotrix --bin nt_pack_bench [N=10000]
//! 输出: 每项 ns/op + 吞吐 (MB/s) + B/记录。

use std::time::Instant;
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::{GeoPoint, PackDecoder, PackEncoder};

fn sample_points(n: usize) -> Vec<GeoPoint> {
    (0..n)
        .map(|i| {
            let lat = -60.0 + (i as f64 * 0.7).sin() * 60.0;
            let lng = -180.0 + (i as f64 * 1.3).cos() * 180.0;
            GeoPoint {
                node_id: format!("geo:airport:{:04X}", i),
                lat,
                lng,
                country: ["US", "CN", "JP", "DE", "BR"][i % 5].into(),
                region: format!("R-{}", i % 50),
                city: format!("City{}", i % 1000),
                tags: "机场,small_airport".into(),
                source: "ourairports".into(),
            }
        })
        .collect()
}

fn timed<F: FnMut() -> O, O>(label: &str, iters: usize, mut f: F) -> f64 {
    let t0 = Instant::now();
    for _ in 0..iters {
        f();
    }
    let elapsed = t0.elapsed().as_nanos() as f64 / iters as f64;
    println!("  {:<28} {:>9.0} ns/op", label, elapsed);
    elapsed
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    let pts = sample_points(n);
    let iters = 20usize.max(1_000_000 / n);

    println!("NT-Pack 基准: {} 条记录, each op = 编码/解码整个集合", n);
    println!("---");

    let enc_e5 = PackEncoder::new(5, false);
    let enc_e5z = PackEncoder::new(5, true);

    // 编码
    timed("encode E5 (no zstd)", iters, || {
        let _b = enc_e5.encode(&pts);
    });
    timed("encode E5+zstd", iters, || {
        let _b = enc_e5z.encode(&pts);
    });

    let pack_e5 = enc_e5.encode(&pts);
    let pack_e5z = enc_e5z.encode(&pts);

    // 解码
    timed("decode E5 (no zstd)", iters, || {
        let (_, out) = PackDecoder::decode(&pack_e5).unwrap();
        std::hint::black_box(out.len());
    });
    timed("decode E5+zstd", iters, || {
        let (_, out) = PackDecoder::decode(&pack_e5z).unwrap();
        std::hint::black_box(out.len());
    });

    // 压缩率
    println!("---");
    println!("  压缩率 (B/记录):");
    println!("    E5 无压缩: {:.1} B/记录", pack_e5.len() as f64 / n as f64);
    println!("    E5+zstd:   {:.1} B/记录", pack_e5z.len() as f64 / n as f64);
    let json_bytes = serde_json::to_string(&pts).unwrap().len();
    println!("    JSON:      {:.1} B/记录", json_bytes as f64 / n as f64);
    println!("    zstd vs JSON: {:.1}x", json_bytes as f64 / pack_e5z.len() as f64);
}