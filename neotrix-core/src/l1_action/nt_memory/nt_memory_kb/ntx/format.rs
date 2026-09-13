//! NTX 文件格式定义 — Header / TOC / SegmentDescriptor
//!
//! 受 Memvid MV2 启发, 为 NeoTrix 设计的单文件知识存储格式。

use std::io::{Read, Seek, SeekFrom, Write};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// NTX 魔数
pub const NTX_MAGIC: &[u8; 4] = b"NTX\0";
/// TOC 魔数
pub const TOC_MAGIC: &[u8; 4] = b"NNTC";
/// 当前格式版本
pub const NTX_VERSION: u16 = 0x0100;
/// Header 大小 (4KB)
pub const HEADER_SIZE: usize = 4096;
/// WAL 最小大小
pub const WAL_MIN_SIZE: u64 = 1024 * 1024; // 1MB
/// WAL 最大大小
pub const WAL_MAX_SIZE: u64 = 64 * 1024 * 1024; // 64MB

// NTX 特性标志位图
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FeatureFlags: u8 {
        const LEX      = 0b0000_0001; // 全文索引
        const VEC      = 0b0000_0010; // 向量索引
        const GRAPH    = 0b0000_0100; // 知识图谱
        const TEMPORAL = 0b0000_1000; // 时间索引
    }
}

impl Default for FeatureFlags {
    fn default() -> Self { Self::empty() }
}

/// 压缩算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Compression {
    None = 0,
    Zstd = 1,
    Lz4 = 2,
}

impl Default for Compression {
    fn default() -> Self { Self::Zstd }
}

/// 段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SegmentType {
    Wal     = 0,
    Frames  = 1,
    Graph   = 2,
    Vec     = 3,
    Lex     = 4,
    Time    = 5,
}

/// 段描述符 (TOC 条目)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDescriptor {
    pub segment_type: SegmentType,
    pub offset: u64,
    pub length: u64,
    pub checksum: [u8; 32],
}

impl SegmentDescriptor {
    pub fn new(segment_type: SegmentType, offset: u64, length: u64, data: &[u8]) -> Self {
        let checksum = sha256(data);
        Self { segment_type, offset, length, checksum }
    }

    pub fn verify(&self, data: &[u8]) -> bool {
        sha256(data) == self.checksum
    }
}

/// NTX 文件头 (4096 bytes, 固定布局)
#[derive(Debug, Clone)]
#[repr(C)]
pub struct NtxHeader {
    pub magic: [u8; 4],               // "NTX\0"
    pub version: u16,                 // 格式版本
    pub feature_flags: u8,            // 特性位图
    pub compression: u8,              // 默认压缩算法
    pub footer_offset: u64,           // TOC 字节偏移
    pub wal_offset: u64,              // WAL 字节偏移 (恒定 4096)
    pub wal_size: u64,                // WAL 区域大小
    pub wal_checkpoint_pos: u64,      // 最后检查点序列号
    pub wal_sequence: u64,            // 当前 WAL 序列号
    pub frame_count: u64,             // 知识帧总数
    pub node_count: u64,              // 图谱节点总数
    pub edge_count: u64,              // 图谱边总数
    pub toc_checksum: [u8; 32],       // TOC SHA-256
    pub schema_hash: [u8; 32],        // SQLite schema 哈希
    pub _reserved: [u8; 3960],        // 零填充
}

impl Default for NtxHeader {
    fn default() -> Self {
        Self {
            magic: *NTX_MAGIC,
            version: NTX_VERSION,
            feature_flags: 0,
            compression: Compression::Zstd as u8,
            footer_offset: 0,
            wal_offset: HEADER_SIZE as u64,
            wal_size: WAL_MIN_SIZE,
            wal_checkpoint_pos: 0,
            wal_sequence: 0,
            frame_count: 0,
            node_count: 0,
            edge_count: 0,
            toc_checksum: [0u8; 32],
            schema_hash: [0u8; 32],
            _reserved: [0u8; 3960],
        }
    }
}

impl NtxHeader {
    /// 从文件读取 Header
    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf = [0u8; HEADER_SIZE];
        reader.read_exact(&mut buf)?;
        Self::from_bytes(&buf)
    }

    /// 写入 Header 到文件
    pub fn write_to(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let buf = self.to_bytes();
        writer.write_all(&buf)?;
        Ok(())
    }

    /// 从字节解析
    pub fn from_bytes(buf: &[u8; HEADER_SIZE]) -> std::io::Result<Self> {
        if buf[0..4] != *NTX_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid NTX magic",
            ));
        }
        Ok(Self {
            magic: [buf[0], buf[1], buf[2], buf[3]],
            version: u16::from_le_bytes([buf[4], buf[5]]),
            feature_flags: buf[6],
            compression: buf[7],
            footer_offset: u64::from_le_bytes(buf[8..16].try_into().expect("8-byte slice for u64")),
            wal_offset: u64::from_le_bytes(buf[16..24].try_into().expect("8-byte slice for u64")),
            wal_size: u64::from_le_bytes(buf[24..32].try_into().expect("8-byte slice for u64")),
            wal_checkpoint_pos: u64::from_le_bytes(buf[32..40].try_into().expect("8-byte slice for u64")),
            wal_sequence: u64::from_le_bytes(buf[40..48].try_into().expect("8-byte slice for u64")),
            frame_count: u64::from_le_bytes(buf[48..56].try_into().expect("8-byte slice for u64")),
            node_count: u64::from_le_bytes(buf[56..64].try_into().expect("8-byte slice for u64")),
            edge_count: u64::from_le_bytes(buf[64..72].try_into().expect("8-byte slice for u64")),
            toc_checksum: buf[72..104].try_into().expect("32-byte slice for checksum"),
            schema_hash: buf[104..136].try_into().expect("32-byte slice for hash"),
            _reserved: [0u8; 3960],
        })
    }

    /// 序列化为字节
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6] = self.feature_flags;
        buf[7] = self.compression;
        buf[8..16].copy_from_slice(&self.footer_offset.to_le_bytes());
        buf[16..24].copy_from_slice(&self.wal_offset.to_le_bytes());
        buf[24..32].copy_from_slice(&self.wal_size.to_le_bytes());
        buf[32..40].copy_from_slice(&self.wal_checkpoint_pos.to_le_bytes());
        buf[40..48].copy_from_slice(&self.wal_sequence.to_le_bytes());
        buf[48..56].copy_from_slice(&self.frame_count.to_le_bytes());
        buf[56..64].copy_from_slice(&self.node_count.to_le_bytes());
        buf[64..72].copy_from_slice(&self.edge_count.to_le_bytes());
        buf[72..104].copy_from_slice(&self.toc_checksum);
        buf[104..136].copy_from_slice(&self.schema_hash);
        buf
    }

    /// 更新特性标志
    pub fn set_feature(&mut self, flag: FeatureFlags, on: bool) {
        if on {
            self.feature_flags |= flag.bits();
        } else {
            self.feature_flags &= !flag.bits();
        }
    }

    /// 检查特性标志
    pub fn has_feature(&self, flag: FeatureFlags) -> bool {
        self.feature_flags & flag.bits() != 0
    }
}

/// NTX Table of Contents (文件尾部)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtxToc {
    pub magic: [u8; 4],          // "NNTC"
    pub version: u16,
    pub segments: Vec<SegmentDescriptor>,
}

impl Default for NtxToc {
    fn default() -> Self {
        Self {
            magic: *TOC_MAGIC,
            version: NTX_VERSION,
            segments: Vec::new(),
        }
    }
}

impl NtxToc {
    /// 从文件尾部读取 TOC
    pub fn read_from(reader: &mut (impl Read + Seek)) -> std::io::Result<Self> {
        // 获取文件大小
        let file_size = reader.seek(SeekFrom::End(0))?;
        if file_size < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData, "File too small for TOC pointer",
            ));
        }

        // 读 Footer 偏移 (最后 8 字节)
        reader.seek(SeekFrom::End(-8))?;
        let mut offset_buf = [0u8; 8];
        reader.read_exact(&mut offset_buf)?;
        let footer_offset = u64::from_le_bytes(offset_buf);

        // 校验偏移范围
        if footer_offset >= file_size - 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("TOC offset {} out of range (file size {})", footer_offset, file_size),
            ));
        }

        // 读取 TOC
        reader.seek(SeekFrom::Start(footer_offset))?;
        let mut toc_len_buf = [0u8; 4];
        reader.read_exact(&mut toc_len_buf)?;
        let toc_len = u32::from_le_bytes(toc_len_buf) as usize;

        let mut toc_buf = vec![0u8; toc_len];
        reader.read_exact(&mut toc_buf)?;

        let toc: NtxToc = bincode::deserialize(&toc_buf).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("TOC deserialize: {e}"))
        })?;

        // 验证 TOC magic
        if toc.magic != *TOC_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid TOC magic: expected {:?}, got {:?}", TOC_MAGIC, toc.magic),
            ));
        }

        Ok(toc)
    }

    /// 写入 TOC 到文件
    pub fn write_to(&self, writer: &mut (impl Write + Seek)) -> std::io::Result<u64> {
        let toc_bytes = bincode::serialize(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("TOC serialize: {e}"))
        })?;

        let footer_offset = writer.seek(SeekFrom::End(0))?;
        let toc_len = (toc_bytes.len() as u32).to_le_bytes();
        writer.write_all(&toc_len)?;
        writer.write_all(&toc_bytes)?;
        writer.write_all(&footer_offset.to_le_bytes())?; // 最后 8 字节指向 TOC

        Ok(footer_offset)
    }

    /// 查找指定类型的段
    pub fn find_segment(&self, stype: SegmentType) -> Option<&SegmentDescriptor> {
        self.segments.iter().find(|s| s.segment_type == stype)
    }

    /// 添加段
    pub fn add_segment(&mut self, desc: SegmentDescriptor) {
        self.segments.push(desc);
    }

    /// 计算 TOC 校验和
    pub fn checksum(&self) -> [u8; 32] {
        let bytes = bincode::serialize(self).unwrap_or_default();
        sha256(&bytes)
    }
}

/// SHA-256 哈希工具
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// CRC32 校验工具
pub fn crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// UUID 字符串 → 36 字节 (去掉连字符, hex → bytes)
pub fn uuid_to_bytes(uuid: &str) -> [u8; 36] {
    let mut bytes = [0u8; 36];
    let clean: String = uuid.chars().filter(|c| c.is_alphanumeric()).collect();
    for (i, chunk) in clean.as_bytes().chunks(2).enumerate() {
        if i >= 36 { break; }
        if let Ok(b) = u8::from_str_radix(std::str::from_utf8(chunk).unwrap_or("0"), 16) {
            bytes[i] = b;
        }
    }
    bytes
}

/// 36 字节 → UUID 字符串 (标准 36 字符格式, 带连字符)
pub fn bytes_to_uuid(bytes: &[u8; 36]) -> String {
    // 标准 UUID 格式: 8-4-4-4-12
    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_header_roundtrip() {
        let header = NtxHeader::default();
        let bytes = header.to_bytes();
        let restored = NtxHeader::from_bytes(&bytes).unwrap();
        assert_eq!(restored.magic, *NTX_MAGIC);
        assert_eq!(restored.version, NTX_VERSION);
        assert_eq!(restored.wal_offset, HEADER_SIZE as u64);
    }

    #[test]
    fn test_feature_flags() {
        let mut header = NtxHeader::default();
        assert!(!header.has_feature(FeatureFlags::LEX));
        header.set_feature(FeatureFlags::LEX, true);
        assert!(header.has_feature(FeatureFlags::LEX));
        header.set_feature(FeatureFlags::LEX, false);
        assert!(!header.has_feature(FeatureFlags::LEX));
    }

    #[test]
    fn test_toc_roundtrip() {
        let mut toc = NtxToc::default();
        toc.add_segment(SegmentDescriptor::new(SegmentType::Frames, 4096, 1024, b"test"));
        toc.add_segment(SegmentDescriptor::new(SegmentType::Vec, 5120, 2048, b"vec"));

        let mut buf = Cursor::new(Vec::new());
        toc.write_to(&mut buf).unwrap();

        buf.set_position(0);
        let restored = NtxToc::read_from(&mut buf).unwrap();
        assert_eq!(restored.segments.len(), 2);
        assert_eq!(restored.segments[0].segment_type, SegmentType::Frames);
        assert_eq!(restored.segments[1].segment_type, SegmentType::Vec);
    }

    #[test]
    fn test_segment_descriptor_verify() {
        let desc = SegmentDescriptor::new(SegmentType::Frames, 0, 100, b"hello world");
        assert!(desc.verify(b"hello world"));
        assert!(!desc.verify(b"hello world!"));
    }
}
