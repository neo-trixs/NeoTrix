//! NTX 嵌入式 WAL — 崩溃恢复引擎
//!
//! WAL 不持有 File 句柄, 所有 I/O 通过 NtxFile 统一调度。

use std::io::{Read, Seek, SeekFrom, Write};
use std::fs::File;
use super::frames::{KnowledgeFrame, FrameError};

/// WAL 条目类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WalEntryType {
    Append  = 0x01,
    Update  = 0x02,
    Delete  = 0x03,
    Checkpoint = 0x04,
}

/// WAL 条目
#[derive(Debug, Clone)]
pub struct WalEntry {
    pub sequence: u64,
    pub entry_type: WalEntryType,
    pub payload: Vec<u8>,
    pub checksum: u32,
}

impl WalEntry {
    pub fn append(frame: &KnowledgeFrame, sequence: u64) -> Self {
        let payload = frame.encode_bytes();
        let checksum = super::format::crc32(&payload);
        Self { sequence, entry_type: WalEntryType::Append, payload, checksum }
    }

    pub fn tombstone(frame_id: u64, sequence: u64) -> Self {
        let payload = frame_id.to_le_bytes().to_vec();
        let checksum = super::format::crc32(&payload);
        Self { sequence, entry_type: WalEntryType::Delete, payload, checksum }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(21 + self.payload.len());
        buf.extend_from_slice(&self.sequence.to_le_bytes());
        buf.push(self.entry_type as u8);
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf.extend_from_slice(&self.checksum.to_le_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 17 {
            return Err(FrameError::Truncated);
        }
        let sequence = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let entry_type = match data[8] {
            0x01 => WalEntryType::Append,
            0x02 => WalEntryType::Update,
            0x03 => WalEntryType::Delete,
            0x04 => WalEntryType::Checkpoint,
            _ => return Err(FrameError::InvalidFrameType),
        };
        let payload_len = u32::from_le_bytes(data[9..13].try_into().unwrap()) as usize;
        if data.len() < 13 + payload_len + 4 {
            return Err(FrameError::Truncated);
        }
        let payload = data[13..13+payload_len].to_vec();
        let checksum = u32::from_le_bytes(data[13+payload_len..17+payload_len].try_into().unwrap());
        Ok(Self { sequence, entry_type, payload, checksum })
    }
}

/// WAL 统计
#[derive(Debug, Clone, Default)]
pub struct WalStats {
    pub pending_bytes: u64,
    pub total_entries: u64,
    pub sequence: u64,
    pub checkpoint_pos: u64,
}

/// 嵌入式 WAL 引擎 (不持有 File)
pub struct EmbeddedWal {
    wal_offset: u64,
    wal_size: u64,
    write_pos: u64,
    sequence: u64,
    checkpoint_pos: u64,
    entry_count: u64,
}

impl EmbeddedWal {
    /// 创建只读 WAL (无状态)
    pub fn new_read_only(header: &super::format::NtxHeader) -> Self {
        Self {
            wal_offset: header.wal_offset,
            wal_size: header.wal_size,
            write_pos: header.wal_offset + header.wal_size,
            sequence: 0,
            checkpoint_pos: 0,
            entry_count: 0,
        }
    }
}

impl EmbeddedWal {
    /// 扫描 WAL 确定状态 (从 file 读取)
    pub fn open(file: &mut File, header: &super::format::NtxHeader) -> std::io::Result<Self> {
        let wal_offset = header.wal_offset;
        let wal_size = header.wal_size;
        let sequence = header.wal_sequence;
        let checkpoint_pos = header.wal_checkpoint_pos;

        file.seek(SeekFrom::Start(wal_offset))?;
        let mut write_pos = wal_offset;
        let mut entry_count = 0u64;
        let mut buf = Vec::new();

        loop {
            let pos = file.stream_position()?;
            if pos >= wal_offset + wal_size { break; }

            let mut len_buf = [0u8; 4];
            if file.read_exact(&mut len_buf).is_err() { break; }
            let entry_len = u32::from_le_bytes(len_buf) as usize;

            if entry_len == 0 || pos + 4 + entry_len as u64 > wal_offset + wal_size { break; }

            buf.resize(entry_len, 0);
            if file.read_exact(&mut buf).is_err() { break; }

            write_pos = pos + 4 + entry_len as u64;
            entry_count += 1;
        }

        Ok(Self { wal_offset, wal_size, write_pos, sequence, checkpoint_pos, entry_count })
    }

    /// 追加条目 (预分配缓冲区优化)
    pub fn append(&mut self, file: &mut File, frame: &KnowledgeFrame) -> std::io::Result<u64> {
        self.sequence += 1;
        let entry = WalEntry::append(frame, self.sequence);
        // 预分配条目缓冲区 (避免多次 seek)
        let bytes = entry.encode();
        let needed = 4 + bytes.len() as u64;
        if self.write_pos + needed > self.wal_offset + self.wal_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "WAL full, checkpoint needed",
            ));
        }
        file.seek(SeekFrom::Start(self.write_pos))?;
        let mut buf = Vec::with_capacity(4 + bytes.len());
        buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(&bytes);
        file.write_all(&buf)?;
        self.write_pos += needed;
        self.entry_count += 1;
        Ok(self.sequence)
    }

    /// 写入单个条目
    fn write_entry(&mut self, file: &mut File, entry: &WalEntry) -> std::io::Result<()> {
        let bytes = entry.encode();
        let len = (bytes.len() as u32).to_le_bytes();

        let needed = 4 + bytes.len() as u64;
        if self.write_pos + needed > self.wal_offset + self.wal_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "WAL full, checkpoint needed",
            ));
        }

        file.seek(SeekFrom::Start(self.write_pos))?;
        file.write_all(&len)?;
        file.write_all(&bytes)?;
        self.write_pos += needed;
        Ok(())
    }

    /// 检查是否需要检查点
    pub fn needs_checkpoint(&self) -> bool {
        let used = self.write_pos - self.wal_offset;
        let threshold = (self.wal_size as f64 * 0.75) as u64;
        used >= threshold || self.entry_count >= 1000
    }

    /// 检查点: 读取所有条目并清空 WAL
    pub fn checkpoint(&mut self, file: &mut File) -> std::io::Result<Vec<WalEntry>> {
        file.seek(SeekFrom::Start(self.wal_offset))?;
        let mut entries = Vec::new();
        let mut buf = Vec::new();

        loop {
            let pos = file.stream_position()?;
            if pos >= self.write_pos { break; }

            let mut len_buf = [0u8; 4];
            if file.read_exact(&mut len_buf).is_err() { break; }
            let entry_len = u32::from_le_bytes(len_buf) as usize;
            if entry_len == 0 { break; }

            buf.resize(entry_len, 0);
            if file.read_exact(&mut buf).is_err() { break; }

            if let Ok(entry) = WalEntry::decode(&buf) {
                entries.push(entry);
            }
        }

        self.checkpoint_pos = self.sequence;
        self.entry_count = 0;

        // 清空 WAL
        file.seek(SeekFrom::Start(self.wal_offset))?;
        let zero_buf = vec![0u8; 8192];
        let mut remaining = self.wal_size as usize;
        while remaining > 0 {
            let chunk = remaining.min(8192);
            file.write_all(&zero_buf[..chunk])?;
            remaining -= chunk;
        }
        file.flush()?;

        self.write_pos = self.wal_offset;
        Ok(entries)
    }

    /// 恢复: 重放未检查点条目
    pub fn recover(&self, file: &mut File) -> std::io::Result<Vec<WalEntry>> {
        file.seek(SeekFrom::Start(self.wal_offset))?;
        let mut entries = Vec::new();
        let mut buf = Vec::new();

        loop {
            let pos = file.stream_position()?;
            if pos >= self.write_pos { break; }

            let mut len_buf = [0u8; 4];
            if file.read_exact(&mut len_buf).is_err() { break; }
            let entry_len = u32::from_le_bytes(len_buf) as usize;
            if entry_len == 0 { break; }

            buf.resize(entry_len, 0);
            if file.read_exact(&mut buf).is_err() { break; }

            if let Ok(entry) = WalEntry::decode(&buf) {
                if entry.sequence > self.checkpoint_pos {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    pub fn flush(&self, _file: &mut File) -> std::io::Result<()> {
        _file.flush()
    }

    pub fn stats(&self) -> WalStats {
        WalStats {
            pending_bytes: self.write_pos - self.wal_offset,
            total_entries: self.entry_count,
            sequence: self.sequence,
            checkpoint_pos: self.checkpoint_pos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use super::super::frames::{FrameType, Encoding};
    use super::super::format::NtxHeader;

    fn mk_frame(id: u64) -> KnowledgeFrame {
        let mut node_id = [0u8; 36];
        node_id[0] = id as u8;
        KnowledgeFrame::new(id, node_id, FrameType::Node, b"test data", Encoding::Raw, None)
    }

    #[test]
    fn test_wal_append_and_recover() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("test.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024 * 1024;
        header.write_to(&mut file).unwrap();
        file.seek(SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header).unwrap();

        for i in 0..5 {
            let frame = mk_frame(i);
            wal.append(&mut file, &frame).unwrap();
        }

        assert_eq!(wal.stats().sequence, 5);
        assert_eq!(wal.stats().total_entries, 5);

        let entries = wal.recover(&mut file).unwrap();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_wal_checkpoint() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("test.ntx");
        let mut file = File::create(&path).unwrap();

        let mut header = NtxHeader::default();
        header.wal_size = 1024;
        header.write_to(&mut file).unwrap();
        file.seek(SeekFrom::Start(header.wal_offset)).unwrap();
        file.write_all(&vec![0u8; header.wal_size as usize]).unwrap();

        let mut wal = EmbeddedWal::open(&mut file, &header).unwrap();

        for i in 0..20 {
            let frame = mk_frame(i);
            wal.append(&mut file, &frame).unwrap();
        }

        assert!(wal.needs_checkpoint());
        let entries = wal.checkpoint(&mut file).unwrap();
        assert_eq!(entries.len(), 20);
        assert!(!wal.needs_checkpoint());
        assert_eq!(wal.stats().pending_bytes, 0);
    }
}
