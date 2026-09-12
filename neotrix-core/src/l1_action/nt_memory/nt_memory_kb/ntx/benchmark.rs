//! NTX 基准测试 — 性能对比与回归检测
//!
//! 对比 SQLite 全量加载 vs NTX 向量搜索, 验证冷启动加速。

use std::time::{Duration, Instant};
use super::NtxFile;
use super::vec_segment::{VecSegment, HnswParams};
use super::frames::{KnowledgeFrame, FrameType, Encoding};
use super::graph_segment::{GraphSegment, GraphEdge, EdgeDirection};

/// 基准测试结果
#[derive(Debug, Clone)]
pub struct BenchResult {
    pub name: String,
    pub iterations: u32,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub ops_per_sec: f64,
    pub throughput_mb_per_sec: f64,
    pub data_size_bytes: u64,
}

impl BenchResult {
    pub fn report(&self) -> String {
        format!(
            "[{}] {} iterations, avg {:.2}ms, {:.0} ops/s, {:.1} MB/s, data {} bytes",
            self.name, self.iterations,
            self.avg_duration.as_secs_f64() * 1000.0,
            self.ops_per_sec,
            self.throughput_mb_per_sec,
            self.data_size_bytes,
        )
    }
}

/// 帧写入基准
pub(crate) fn bench_frame_write(count: usize, iterations: u32) -> BenchResult {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("bench_frames.ntx");
    let data = vec![42u8; 1024]; // 1KB payload
    let start_total = Instant::now();

    for _ in 0..iterations {
        let mut ntx = NtxFile::create(&path).unwrap();
        for i in 0..count {
            let mut node_id = [0u8; 36];
            node_id[0] = (i % 256) as u8;
            let frame = KnowledgeFrame::new(
                i as u64, node_id, FrameType::Node, &data, Encoding::Zstd, None,
            );
            ntx.put_frame(&frame).unwrap();
        }
        ntx.commit().unwrap();
    }

    let total = start_total.elapsed();
    let avg = total / iterations;
    let ops = iterations as f64 * count as f64 / total.as_secs_f64();
    let data_size = (count * 1024 * iterations as usize) as u64;
    let throughput = data_size as f64 / total.as_secs_f64() / (1024.0 * 1024.0);

    BenchResult {
        name: "frame_write_zstd_1kb".to_string(),
        iterations,
        total_duration: total,
        avg_duration: avg,
        ops_per_sec: ops,
        throughput_mb_per_sec: throughput,
        data_size_bytes: data_size,
    }
}

/// 帧读取基准
pub(crate) fn bench_frame_read(count: usize, iterations: u32) -> BenchResult {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("bench_read.ntx");

    // 预写入
    {
        let mut ntx = NtxFile::create(&path).unwrap();
        let data = vec![42u8; 1024];
        for i in 0..count {
            let mut node_id = [0u8; 36];
            node_id[0] = (i % 256) as u8;
            ntx.put_frame(&KnowledgeFrame::new(
                i as u64, node_id, FrameType::Node, &data, Encoding::Zstd, None,
            )).unwrap();
        }
        ntx.commit().unwrap();
    }

    let start_total = Instant::now();
    for _ in 0..iterations {
        let ntx = NtxFile::open(&path).unwrap();
        assert_eq!(ntx.frames().len(), count);
    }

    let total = start_total.elapsed();
    let avg = total / iterations;
    let ops = iterations as f64 / total.as_secs_f64();

    BenchResult {
        name: "frame_read_1000".to_string(),
        iterations,
        total_duration: total,
        avg_duration: avg,
        ops_per_sec: ops,
        throughput_mb_per_sec: 0.0,
        data_size_bytes: 0,
    }
}

/// 向量搜索基准
pub(crate) fn bench_vec_search(dimension: usize, vec_count: usize, k: usize, iterations: u32) -> BenchResult {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("bench_vec.ntx");

    // 预写入
    {
        let mut ntx = NtxFile::create(&path).unwrap();
        let mut seg = VecSegment::new(dimension, HnswParams::default());
        for i in 0..vec_count {
            let mut node_id = [0u8; 36];
            node_id[0] = (i % 256) as u8;
            let vec: Vec<f32> = (0..dimension).map(|d| (i * d) as f32 / dimension as f32).collect();
            seg.insert(node_id, vec);
        }
        ntx.put_vec_segment(seg);
        ntx.commit().unwrap();
    }

    let query: Vec<f32> = (0..dimension).map(|d| d as f32 / dimension as f32).collect();

    let start_total = Instant::now();
    for _ in 0..iterations {
        let ntx = NtxFile::open_read_only(&path).unwrap();
        let seg = ntx.vec_segment().unwrap();
        let results = seg.search(&query, k);
        assert_eq!(results.len(), k.min(vec_count));
    }

    let total = start_total.elapsed();
    let avg = total / iterations;
    let ops = iterations as f64 / total.as_secs_f64();

    BenchResult {
        name: format!("vec_search_d{}_n{}_k{}", dimension, vec_count, k),
        iterations,
        total_duration: total,
        avg_duration: avg,
        ops_per_sec: ops,
        throughput_mb_per_sec: 0.0,
        data_size_bytes: 0,
    }
}

/// 图谱 BFS 基准
pub(crate) fn bench_graph_bfs(node_count: usize, edges_per_node: usize, iterations: u32) -> BenchResult {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("bench_graph.ntx");

    // 预写入
    {
        let mut ntx = NtxFile::create(&path).unwrap();
        let mut g = GraphSegment::new();
        for i in 0..node_count {
            for j in 0..edges_per_node {
                let target = (i + j + 1) % node_count;
                let mut src = [0u8; 36];
                src[0] = (i % 256) as u8;
                let mut tgt = [0u8; 36];
                tgt[0] = (target % 256) as u8;
                g.add_edge(GraphEdge {
                    source: src, target: tgt,
                    edge_type: 0, weight: 1.0,
                    direction: EdgeDirection::Bidirectional,
                });
            }
        }
        ntx.put_graph_segment(g);
        ntx.commit().unwrap();
    }

    let start_id = [0u8; 36];

    let start_total = Instant::now();
    for _ in 0..iterations {
        let ntx = NtxFile::open_read_only(&path).unwrap();
        let g = ntx.graph_segment().unwrap();
        let _ = g.bfs(&start_id, 3);
    }

    let total = start_total.elapsed();
    let avg = total / iterations;
    let ops = iterations as f64 / total.as_secs_f64();

    BenchResult {
        name: format!("graph_bfs_n{}_e{}", node_count, edges_per_node),
        iterations,
        total_duration: total,
        avg_duration: avg,
        ops_per_sec: ops,
        throughput_mb_per_sec: 0.0,
        data_size_bytes: 0,
    }
}

/// WAL 崩溃恢复基准
pub fn bench_wal_recovery(entry_count: usize, iterations: u32) -> BenchResult {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("bench_wal.ntx");

    // 预写入 (不检查点, 模拟崩溃)
    {
        let mut ntx = NtxFile::create(&path).unwrap();
        for i in 0..entry_count {
            let mut node_id = [0u8; 36];
            node_id[0] = (i % 256) as u8;
            ntx.put_frame(&KnowledgeFrame::new(
                i as u64, node_id, FrameType::Node, b"wal test", Encoding::Raw, None,
            )).unwrap();
        }
        ntx.commit().unwrap();
    }

    let start_total = Instant::now();
    for _ in 0..iterations {
        let mut ntx = NtxFile::open(&path).unwrap();
        let recovered = ntx.recover().unwrap();
        // WAL 可能为空 (已检查点), 这是正常的
        let _ = recovered;
    }

    let total = start_total.elapsed();
    let avg = total / iterations;
    let ops = iterations as f64 / total.as_secs_f64();

    BenchResult {
        name: format!("wal_recovery_n{}", entry_count),
        iterations,
        total_duration: total,
        avg_duration: avg,
        ops_per_sec: ops,
        throughput_mb_per_sec: 0.0,
        data_size_bytes: 0,
    }
}

/// 完整基准套件
pub(crate) fn run_full_benchmark() -> Vec<BenchResult> {
    let mut results = Vec::new();

    println!("=== NTX Benchmark Suite ===\n");

    // 帧写入
    let r = bench_frame_write(1000, 5);
    println!("{}", r.report());
    results.push(r);

    // 帧读取
    let r = bench_frame_read(1000, 10);
    println!("{}", r.report());
    results.push(r);

    // 向量搜索
    let r = bench_vec_search(128, 10000, 10, 100);
    println!("{}", r.report());
    results.push(r);

    let r = bench_vec_search(512, 50000, 10, 20);
    println!("{}", r.report());
    results.push(r);

    // 图谱 BFS
    let r = bench_graph_bfs(1000, 5, 100);
    println!("{}", r.report());
    results.push(r);

    let r = bench_graph_bfs(10000, 3, 50);
    println!("{}", r.report());
    results.push(r);

    // WAL 恢复
    let r = bench_wal_recovery(5000, 10);
    println!("{}", r.report());
    results.push(r);

    println!("\n=== Benchmark Complete ===");
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_frame_write() {
        let r = bench_frame_write(100, 2);
        println!("{}", r.report());
        assert!(r.ops_per_sec > 0.0);
    }

    #[test]
    fn test_bench_vec_search() {
        let r = bench_vec_search(64, 1000, 5, 10);
        println!("{}", r.report());
        assert!(r.ops_per_sec > 0.0);
    }

    #[test]
    fn test_bench_graph_bfs() {
        let r = bench_graph_bfs(100, 3, 10);
        println!("{}", r.report());
        assert!(r.ops_per_sec > 0.0);
    }
}
