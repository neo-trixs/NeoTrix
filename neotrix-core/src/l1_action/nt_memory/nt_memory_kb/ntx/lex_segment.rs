//! NTX Lex Segment — 嵌入式 FTS5 数据库快照
//!
//! 解决痛点 #2: FTS5 内容复制 2× → 嵌入 FTS5 数据库文件
//! 将 SQLite FTS5 数据库完整嵌入 NTX 单文件。

use std::io::{Read, Write};
use super::format::sha256;

/// Lex Segment magic
pub const LEX_MAGIC: &[u8; 4] = b"NTLX";

/// Lex 段: 存储 FTS5 数据库的原始字节
pub struct LexSegment {
    db_bytes: Vec<u8>,
}

impl LexSegment {
    /// 从 FTS5 数据库文件创建
    pub fn from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let db_bytes = std::fs::read(path)?;
        Ok(Self { db_bytes })
    }

    /// 从字节创建
    pub fn from_bytes(db_bytes: Vec<u8>) -> Self {
        Self { db_bytes }
    }

    /// 获取数据库字节
    pub fn db_bytes(&self) -> &[u8] {
        &self.db_bytes
    }

    /// 数据库大小
    pub fn size(&self) -> usize {
        self.db_bytes.len()
    }

    /// 写入段 (magic + checksum + data)
    pub fn write_to(&self, writer: &mut impl Write) -> std::io::Result<u64> {
        let start = writer.stream_position()?;

        // Magic
        writer.write_all(LEX_MAGIC)?;

        // 数据长度
        writer.write_all(&(self.db_bytes.len() as u64).to_le_bytes())?;

        // SHA-256 校验和
        let checksum = sha256(&self.db_bytes);
        writer.write_all(&checksum)?;

        // 数据
        writer.write_all(&self.db_bytes)?;

        Ok(start)
    }

    /// 从文件读取段
    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        // Magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != *LEX_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid Lex magic: expected {:?}, got {:?}", LEX_MAGIC, magic),
            ));
        }

        // 数据长度
        let mut len_buf = [0u8; 8];
        reader.read_exact(&mut len_buf)?;
        let data_len = u64::from_le_bytes(len_buf) as usize;

        // SHA-256 校验和
        let mut expected_checksum = [0u8; 32];
        reader.read_exact(&mut expected_checksum)?;

        // 数据
        let mut db_bytes = vec![0u8; data_len];
        reader.read_exact(&mut db_bytes)?;

        // 验证校验和
        let actual_checksum = sha256(&db_bytes);
        if actual_checksum != expected_checksum {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Lex segment checksum mismatch",
            ));
        }

        Ok(Self { db_bytes })
    }

    /// 验证段完整性
    pub fn verify(&self, expected_checksum: &[u8; 32]) -> bool {
        sha256(&self.db_bytes) == *expected_checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_roundtrip() {
        let data = b"CREATE VIRTUAL TABLE test USING fts5(content);";
        let seg = LexSegment::from_bytes(data.to_vec());

        let mut buf = std::io::Cursor::new(Vec::new());
        seg.write_to(&mut buf).unwrap();

        buf.set_position(0);
        let restored = LexSegment::read_from(&mut buf).unwrap();
        assert_eq!(restored.db_bytes(), data);
    }

    #[test]
    fn test_lex_checksum_verify() {
        let seg = LexSegment::from_bytes(b"test data".to_vec());
        let checksum = sha256(b"test data");
        assert!(seg.verify(&checksum));

        let bad_checksum = [0u8; 32];
        assert!(!seg.verify(&bad_checksum));
    }
}
