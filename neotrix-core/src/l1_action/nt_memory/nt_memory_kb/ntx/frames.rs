//! NTX 知识帧 — 追加只读的不可变知识单元
//!
//! 每条知识 = 一个 KnowledgeFrame, 支持 raw/zstd/lz4 压缩。

use std::io::{Read, Write};
use log::warn;
use serde::{Serialize, Deserialize};
use super::format::{crc32, Compression};

/// 帧类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FrameType {
    Node  = 0x00,  // KnowledgeNode (JSON)
    Edge  = 0x01,  // KnowledgeEdge (JSON)
    Kv    = 0x02,  // key-value 对
    Embed = 0x03,  // 向量 BLOB
}

/// 编码方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Encoding {
    Raw  = 0,
    Zstd = 1,
    Lz4  = 2,
}

impl From<Compression> for Encoding {
    fn from(c: Compression) -> Self {
        match c {
            Compression::None => Encoding::Raw,
            Compression::Zstd => Encoding::Zstd,
            Compression::Lz4  => Encoding::Lz4,
        }
    }
}

/// NTX 知识帧
#[derive(Debug, Clone)]
pub struct KnowledgeFrame {
    pub frame_id: u64,
    pub node_id: [u8; 36],        // UUID bytes
    pub frame_type: FrameType,
    pub encoding: Encoding,
    pub payload: Vec<u8>,         // 压缩后的数据
    pub uncompressed_len: u32,    // 原始长度
    pub checksum: u32,            // CRC32 of uncompressed payload
    pub timestamp: u64,           // Unix 秒
    pub tags: Option<String>,     // JSON 标签 (可选)
}

impl KnowledgeFrame {
    /// 创建新帧
    pub fn new(
        frame_id: u64,
        node_id: [u8; 36],
        frame_type: FrameType,
        data: &[u8],
        encoding: Encoding,
        tags: Option<String>,
    ) -> Self {
        let checksum = crc32(data);
        let uncompressed_len = data.len() as u32;
        let payload = match encoding {
            Encoding::Raw  => data.to_vec(),
            Encoding::Zstd => compress_zstd(data, 3),
            Encoding::Lz4  => compress_lz4(data),
        };
        Self {
            frame_id,
            node_id,
            frame_type,
            encoding,
            payload,
            uncompressed_len,
            checksum,
            timestamp: now_seconds(),
            tags,
        }
    }

    /// 解压 payload
    pub fn decompress(&self) -> Result<Vec<u8>, FrameError> {
        match self.encoding {
            Encoding::Raw  => Ok(self.payload.clone()),
            Encoding::Zstd => decompress_zstd(&self.payload, self.uncompressed_len as usize),
            Encoding::Lz4  => decompress_lz4(&self.payload, self.uncompressed_len as usize),
        }
    }

    /// 验证帧完整性
    pub fn verify(&self) -> bool {
        let data = match self.decompress() {
            Ok(d) => d,
            Err(_) => return false,
        };
        crc32(&data) == self.checksum
    }

    /// 帧序列化为字节 (磁盘格式, 写入时验证校验和)
    pub fn encode_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(256 + self.payload.len());
        buf.extend_from_slice(&self.frame_id.to_le_bytes());
        buf.extend_from_slice(&self.node_id);
        buf.push(self.frame_type as u8);
        buf.push(self.encoding as u8);
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf.extend_from_slice(&self.uncompressed_len.to_le_bytes());
        // 验证校验和一致性
        let computed = crc32(&self.payload);
        if self.checksum != 0 && self.checksum != computed {
            warn!("[NTX] 帧 {} 校验和不匹配: 存储={}, 计算={}", self.frame_id, self.checksum, computed);
        }
        buf.extend_from_slice(&self.checksum.to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        let tags_bytes = self.tags.as_deref().unwrap_or("").as_bytes();
        buf.extend_from_slice(&(tags_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(tags_bytes);
        buf
    }

    /// 从字节反序列化
    pub fn decode_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 65 { // 最小帧头: 8+36+1+1+4+0+4+4+8+2+0
            return Err(FrameError::Truncated);
        }
        let mut pos = 0;

        let frame_id = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
        pos += 8;

        let mut node_id = [0u8; 36];
        node_id.copy_from_slice(&data[pos..pos+36]);
        pos += 36;

        let frame_type = match data[pos] {
            0x00 => FrameType::Node,
            0x01 => FrameType::Edge,
            0x02 => FrameType::Kv,
            0x03 => FrameType::Embed,
            _ => return Err(FrameError::InvalidFrameType),
        };
        pos += 1;

        let encoding = match data[pos] {
            0x00 => Encoding::Raw,
            0x01 => Encoding::Zstd,
            0x02 => Encoding::Lz4,
            _ => return Err(FrameError::InvalidEncoding),
        };
        pos += 1;

        let payload_len = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
        pos += 4;

        if data.len() < pos + payload_len + 16 {
            return Err(FrameError::Truncated);
        }

        let payload = data[pos..pos+payload_len].to_vec();
        pos += payload_len;

        let uncompressed_len = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
        pos += 4;

        let checksum = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
        pos += 4;

        let timestamp = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
        pos += 8;

        let tags_len = u16::from_le_bytes(data[pos..pos+2].try_into().unwrap()) as usize;
        pos += 2;

        let tags = if tags_len > 0 {
            Some(String::from_utf8_lossy(&data[pos..pos+tags_len]).to_string())
        } else {
            None
        };

        Ok(Self {
            frame_id,
            node_id,
            frame_type,
            encoding,
            payload,
            uncompressed_len,
            checksum,
            timestamp,
            tags,
        })
    }
}

// ============================================================
// 压缩工具
// ============================================================

fn compress_zstd(data: &[u8], level: i32) -> Vec<u8> {
    zstd::encode_all(data, level).unwrap_or_else(|_| data.to_vec())
}

fn decompress_zstd(data: &[u8], expected_len: usize) -> Result<Vec<u8>, FrameError> {
    zstd::decode_all(data).map_err(|_| FrameError::DecompressFailed).and_then(|v| {
        if v.len() != expected_len {
            Err(FrameError::SizeMismatch { expected: expected_len, actual: v.len() })
        } else {
            Ok(v)
        }
    })
}

fn compress_lz4(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

fn decompress_lz4(data: &[u8], expected_len: usize) -> Result<Vec<u8>, FrameError> {
    lz4_flex::decompress_size_prepended(data).map_err(|_| FrameError::DecompressFailed).and_then(|v| {
        if v.len() != expected_len {
            Err(FrameError::SizeMismatch { expected: expected_len, actual: v.len() })
        } else {
            Ok(v)
        }
    })
}

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================
// 错误类型
// ============================================================

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("帧数据截断")]
    Truncated,
    #[error("无效帧类型")]
    InvalidFrameType,
    #[error("无效编码")]
    InvalidEncoding,
    #[error("解压失败")]
    DecompressFailed,
    #[error("无效压缩类型")]
    InvalidCompression,
    #[error("大小不匹配: expected {expected}, got {actual}")]
    SizeMismatch { expected: usize, actual: usize },
}

// ============================================================
// 批量读写
// ============================================================

/// 写入多帧到 writer
pub fn write_frames(writer: &mut (impl Write + std::io::Seek), frames: &[KnowledgeFrame]) -> std::io::Result<u64> {
    let start_offset = writer.stream_position()?;
    for frame in frames {
        let bytes = frame.encode_bytes();
        let len = (bytes.len() as u32).to_le_bytes();
        writer.write_all(&len)?;
        writer.write_all(&bytes)?;
    }
    Ok(start_offset)
}

/// 从 reader 读取多帧
pub fn read_frames(reader: &mut impl Read, count: usize) -> Result<Vec<KnowledgeFrame>, FrameError> {
    let mut frames = Vec::with_capacity(count);
    for _ in 0..count {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).map_err(|_| FrameError::Truncated)?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).map_err(|_| FrameError::Truncated)?;
        frames.push(KnowledgeFrame::decode_bytes(&buf)?);
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_roundtrip_raw() {
        let node_id = [1u8; 36];
        let data = b"hello knowledge";
        let frame = KnowledgeFrame::new(0, node_id, FrameType::Node, data, Encoding::Raw, None);
        assert!(frame.verify());

        let bytes = frame.encode_bytes();
        let restored = KnowledgeFrame::decode_bytes(&bytes).unwrap();
        assert_eq!(restored.frame_id, 0);
        assert_eq!(restored.node_id, node_id);
        assert_eq!(restored.decompress().unwrap(), data);
    }

    #[test]
    fn test_frame_roundtrip_zstd() {
        let node_id = [2u8; 36];
        let data = "zstd compressed content ".repeat(100).into_bytes();
        let frame = KnowledgeFrame::new(1, node_id, FrameType::Node, &data, Encoding::Zstd, None);
        assert!(frame.verify());
        assert!(frame.payload.len() < data.len()); // 压缩有效

        let bytes = frame.encode_bytes();
        let restored = KnowledgeFrame::decode_bytes(&bytes).unwrap();
        assert_eq!(restored.decompress().unwrap(), data);
    }

    #[test]
    fn test_frame_with_tags() {
        let node_id = [3u8; 36];
        let tags = r#"{"domain":"ai","importance":"high"}"#;
        let frame = KnowledgeFrame::new(
            2, node_id, FrameType::Kv, b"key=value", Encoding::Raw, Some(tags.to_string()),
        );
        let bytes = frame.encode_bytes();
        let restored = KnowledgeFrame::decode_bytes(&bytes).unwrap();
        assert_eq!(restored.tags.as_deref(), Some(tags));
    }

    #[test]
    fn test_write_read_frames() {
        let frames: Vec<KnowledgeFrame> = (0..10)
            .map(|i| {
                let mut node_id = [0u8; 36];
                node_id[0] = i;
                KnowledgeFrame::new(i, node_id, FrameType::Node, b"test", Encoding::Raw, None)
            })
            .collect();

        let mut buf = std::io::Cursor::new(Vec::new());
        write_frames(&mut buf, &frames).unwrap();

        buf.set_position(0);
        let restored = read_frames(&mut buf, 10).unwrap();
        assert_eq!(restored.len(), 10);
        for (i, frame) in restored.iter().enumerate() {
            assert_eq!(frame.frame_id, i as u64);
        }
    }
}
