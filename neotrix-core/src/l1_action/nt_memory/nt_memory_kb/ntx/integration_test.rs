//! NTX 集成测试 — 验证 WAL 压缩和 mmap 加载功能

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::super::wal::{CompressionType, EmbeddedWal};
    use super::super::frames::{KnowledgeFrame, FrameType, Encoding};
    use super::super::format::NtxHeader;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::{Seek, Write};

    fn mk_frame(id: u64, data: &[u8]) -> KnowledgeFrame {
        let mut node_id = [0u8; 36];
        node_id[0] = id as u8;
        KnowledgeFrame::new(id, node_id, FrameType::Node, data, Encoding::Raw, None)
    }

    #[test]
    fn test_wal_compression_integration() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("test.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024;
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        // 测试 Zstd 压缩
        let mut wal_zstd = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::Zstd);

        let data = "compressible data ".repeat(1000);
        let frame1 = mk_frame(1, data.as_bytes());
        wal_zstd.append(&mut file, &frame1).unwrap();

        // 测试 LZ4 压缩
        let mut wal_lz4 = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::LZ4);

        let frame2 = mk_frame(2, data.as_bytes());
        wal_lz4.append(&mut file, &frame2).unwrap();

        // 恢复并验证
        let entries = wal_zstd.recover(&mut file).unwrap();
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        assert_eq!(entry.compression, CompressionType::Zstd);
        assert!(entry.uncompressed_size > 0);
        assert!(entry.payload.len() < entry.uncompressed_size as usize);
        
        // 验证解压正确
        let decompressed = entry.decompress_payload().unwrap();
        assert_eq!(decompressed.len(), entry.uncompressed_size as usize);
    }

    #[test]
    fn test_wal_compression_with_large_data() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("test.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024;
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::Zstd);

        // 写入多个大帧
        for i in 0..10 {
            let data = format!("frame {} data {}", i, "x".repeat(1000));
            let frame = mk_frame(i, data.as_bytes());
            wal.append(&mut file, &frame).unwrap();
        }

        // 恢复并验证
        let entries = wal.recover(&mut file).unwrap();
        assert_eq!(entries.len(), 10);
        
        // 验证所有条目都被正确压缩和解压
        for entry in &entries {
            assert_eq!(entry.compression, CompressionType::Zstd);
            assert!(entry.uncompressed_size > 0);
            let decompressed = entry.decompress_payload().unwrap();
            assert_eq!(decompressed.len(), entry.uncompressed_size as usize);
        }
    }

    #[test]
    fn test_wal_compression_checkpoint() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("test.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024;
        header.write_to(&mut file).unwrap();
        file.seek(std::io::SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header)
            .unwrap()
            .with_compression(CompressionType::Zstd);

        // 写入数据直到 WAL 满
        for i in 0..20 {
            let data = format!("frame {} data {}", i, "x".repeat(100));
            let frame = mk_frame(i, data.as_bytes());
            wal.append(&mut file, &frame).unwrap();
        }

        // 检查点
        assert!(wal.needs_checkpoint());
        let entries = wal.checkpoint(&mut file).unwrap();
        assert_eq!(entries.len(), 20);
        assert!(!wal.needs_checkpoint());
        assert_eq!(wal.stats().pending_bytes, 0);

        // 验证检查点后的 WAL 可以继续写入
        let frame = mk_frame(21, b"new data after checkpoint");
        wal.append(&mut file, &frame).unwrap();
        
        let entries = wal.recover(&mut file).unwrap();
        assert_eq!(entries.len(), 1);
    }
}
