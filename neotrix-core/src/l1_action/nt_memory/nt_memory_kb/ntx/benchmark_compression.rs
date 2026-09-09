//! NTX 性能基准测试 — WAL 压缩和 mmap 加载性能对比

#[cfg(test)]
mod benchmarks {
    use super::super::*;
    use super::super::wal::{CompressionType, EmbeddedWal};
    use super::super::frames::{KnowledgeFrame, FrameType, Encoding};
    use super::super::format::NtxHeader;
    use super::super::vec_segment::{VecSegment, HnswParams};
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::{Seek, Write};
    use std::time::Instant;

    fn mk_frame(id: u64, data: &[u8]) -> KnowledgeFrame {
        let mut node_id = [0u8; 36];
        node_id[0] = id as u8;
        KnowledgeFrame::new(id, node_id, FrameType::Node, data, Encoding::Raw, None)
    }

    #[test]
    fn bench_wal_compression_zstd() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bench.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024 * 10; // 10MB
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::Zstd);

        // 生成可压缩数据
        let data = "benchmark data ".repeat(1000);
        let frame = mk_frame(1, data.as_bytes());

        // 预热
        for _ in 0..10 {
            wal.append(&mut file, &frame).unwrap();
        }

        // 基准测试
        let iterations = 1000;
        let start = Instant::now();
        for i in 0..iterations {
            let mut frame = mk_frame(i, data.as_bytes());
            wal.append(&mut file, &frame).unwrap();
        }
        let duration = start.elapsed();

        println!("WAL Zstd compression: {} iterations in {:?}", iterations, duration);
        println!("Average: {:?} per iteration", duration / iterations);
    }

    #[test]
    fn bench_wal_compression_lz4() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bench.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024 * 10; // 10MB
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::LZ4);

        // 生成可压缩数据
        let data = "benchmark data ".repeat(1000);
        let frame = mk_frame(1, data.as_bytes());

        // 预热
        for _ in 0..10 {
            wal.append(&mut file, &frame).unwrap();
        }

        // 基准测试
        let iterations = 1000;
        let start = Instant::now();
        for i in 0..iterations {
            let mut frame = mk_frame(i, data.as_bytes());
            wal.append(&mut file, &frame).unwrap();
        }
        let duration = start.elapsed();

        println!("WAL LZ4 compression: {} iterations in {:?}", iterations, duration);
        println!("Average: {:?} per iteration", duration / iterations);
    }

    #[test]
    fn bench_wal_no_compression() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bench.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024 * 10; // 10MB
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::None);

        // 生成可压缩数据
        let data = "benchmark data ".repeat(1000);
        let frame = mk_frame(1, data.as_bytes());

        // 预热
        for _ in 0..10 {
            wal.append(&mut file, &frame).unwrap();
        }

        // 基准测试
        let iterations = 1000;
        let start = Instant::now();
        for i in 0..iterations {
            let mut frame = mk_frame(i, data.as_bytes());
            wal.append(&mut file, &frame).unwrap();
        }
        let duration = start.elapsed();

        println!("WAL no compression: {} iterations in {:?}", iterations, duration);
        println!("Average: {:?} per iteration", duration / iterations);
    }

    #[test]
    fn bench_vec_segment_load() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bench.vec");

        // 创建测试数据
        let mut seg = VecSegment::new(128, HnswParams::default());
        for i in 0..10000 {
            let mut node_id = [0u8; 36];
            node_id[0] = i as u8;
            node_id[1] = (i >> 8) as u8;
            let vec: Vec<f32> = (0..128).map(|j| (i * 128 + j) as f32).collect();
            seg.insert(node_id, vec);
        }

        // 写入文件
        let mut file = File::create(&path).unwrap();
        seg.write_to(&mut file).unwrap();
        drop(file);

        // 基准测试: 传统加载
        let start = Instant::now();
        for _ in 0..10 {
            let mut file = File::open(&path).unwrap();
            let loaded = VecSegment::read_from(&mut file).unwrap();
        }
        let traditional_duration = start.elapsed();

        // 基准测试: from_file 加载
        let start = Instant::now();
        for _ in 0..10 {
            let loaded = VecSegment::from_file(&path).unwrap();
        }
        let from_file_duration = start.elapsed();

        println!("Vec segment load (10K vectors, 128 dim):");
        println!("  Traditional: {:?}", traditional_duration / 10);
        println!("  from_file: {:?}", from_file_duration / 10);
        println!("  Speedup: {:.2}x", traditional_duration.as_secs_f64() / from_file_duration.as_secs_f64());
    }

    #[test]
    fn bench_wal_recovery() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bench.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024 * 10; // 10MB
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::Zstd);

        // 写入数据
        let data = "recovery benchmark ".repeat(1000);
        for i in 0..1000 {
            let frame = mk_frame(i, data.as_bytes());
            wal.append(&mut file, &frame).unwrap();
        }

        // 基准测试: 恢复
        let start = Instant::now();
        let entries = wal.recover(&mut file).unwrap();
        let duration = start.elapsed();

        println!("WAL recovery (1000 entries, Zstd): {:?}", duration);
        println!("Recovered {} entries", entries.len());
    }

    #[test]
    fn bench_compression_ratio() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bench.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024 * 10; // 10MB
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        // 测试不同压缩算法的压缩率
        let data = "compression ratio test ".repeat(10000);
        let frame = mk_frame(1, data.as_bytes());
        let original_size = frame.payload.len() as f64;

        // Zstd
        let mut wal_zstd = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::Zstd);
        wal_zstd.append(&mut file, &frame).unwrap();
        let entries_zstd = wal_zstd.recover(&mut file).unwrap();
        let zstd_size = entries_zstd[0].payload.len() as f64;

        // LZ4
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();
        let mut wal_lz4 = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::LZ4);
        wal_lz4.append(&mut file, &frame).unwrap();
        let entries_lz4 = wal_lz4.recover(&mut file).unwrap();
        let lz4_size = entries_lz4[0].payload.len() as f64;

        println!("Compression ratio comparison:");
        println!("  Original size: {} bytes", original_size);
        println!("  Zstd: {} bytes ({:.2}x compression)", zstd_size, original_size / zstd_size);
        println!("  LZ4: {} bytes ({:.2}x compression)", lz4_size, original_size / lz4_size);
    }
}
