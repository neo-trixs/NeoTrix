//! NTX 时间索引段 — 时间旅行查询
//!
//! 解决痛点 #4: 无时间旅行 → 时间索引 + 追加帧
//! 支持按时间范围查询知识快照

use std::io::{Read, Seek, Write};
use serde::{Serialize, Deserialize};

/// 时间索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub timestamp: u64,     // Unix 秒
    pub frame_id: u64,      // 关联帧 ID
    pub entry_type: u8,     // 0=node, 1=edge, 2=kv
    pub offset: u64,        // 帧在文件中的偏移
}

/// 时间索引段
pub struct TimeSegment {
    entries: Vec<TimeEntry>,
}

impl TimeSegment {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 添加条目
    pub fn insert(&mut self, entry: TimeEntry) {
        self.entries.push(entry);
        // 保持时间有序
        self.entries.sort_by_key(|e| e.timestamp);
    }

    /// 查询时间范围内的条目
    pub fn range(&self, from: u64, to: u64) -> Vec<&TimeEntry> {
        self.entries.iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect()
    }

    /// 查询最近 N 秒的条目
    pub fn recent(&self, seconds: u64) -> Vec<&TimeEntry> {
        let now = now_seconds();
        let from = now.saturating_sub(seconds);
        self.range(from, now)
    }

    /// 查询特定帧
    pub fn by_frame(&self, frame_id: u64) -> Vec<&TimeEntry> {
        self.entries.iter().filter(|e| e.frame_id == frame_id).collect()
    }

    /// 条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取所有条目
    pub fn entries(&self) -> &[TimeEntry] {
        &self.entries
    }

    /// 写入段 (magic + data)
    pub fn write_to(&self, writer: &mut (impl Write + Seek)) -> std::io::Result<u64> {
        let start = writer.stream_position()?;
        // Magic
        writer.write_all(b"NTTI")?;
        writer.write_all(&(self.entries.len() as u64).to_le_bytes())?;
        for entry in &self.entries {
            writer.write_all(&entry.timestamp.to_le_bytes())?;
            writer.write_all(&entry.frame_id.to_le_bytes())?;
            writer.write_all(&[entry.entry_type])?;
            writer.write_all(&entry.offset.to_le_bytes())?;
        }
        Ok(start)
    }

    /// 读取段
    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        // Magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != *b"NTTI" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid Time magic: expected NTTI, got {:?}", magic),
            ));
        }

        let mut count_buf = [0u8; 8];
        reader.read_exact(&mut count_buf)?;
        let count = u64::from_le_bytes(count_buf) as usize;

        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let mut ts_buf = [0u8; 8];
            reader.read_exact(&mut ts_buf)?;
            let mut fid_buf = [0u8; 8];
            reader.read_exact(&mut fid_buf)?;
            let mut type_buf = [0u8; 1];
            reader.read_exact(&mut type_buf)?;
            let mut off_buf = [0u8; 8];
            reader.read_exact(&mut off_buf)?;

            entries.push(TimeEntry {
                timestamp: u64::from_le_bytes(ts_buf),
                frame_id: u64::from_le_bytes(fid_buf),
                entry_type: type_buf[0],
                offset: u64::from_le_bytes(off_buf),
            });
        }

        Ok(Self { entries })
    }
}

impl Default for TimeSegment {
    fn default() -> Self { Self::new() }
}

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_roundtrip() {
        let mut ts = TimeSegment::new();
        ts.insert(TimeEntry { timestamp: 1000, frame_id: 1, entry_type: 0, offset: 0 });
        ts.insert(TimeEntry { timestamp: 2000, frame_id: 2, entry_type: 1, offset: 100 });
        ts.insert(TimeEntry { timestamp: 3000, frame_id: 3, entry_type: 2, offset: 200 });

        let mut buf = std::io::Cursor::new(Vec::new());
        ts.write_to(&mut buf).unwrap();
        buf.set_position(0);
        let restored = TimeSegment::read_from(&mut buf).unwrap();

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.entries()[0].timestamp, 1000);
    }

    #[test]
    fn test_range_query() {
        let mut ts = TimeSegment::new();
        for i in 0..10 {
            ts.insert(TimeEntry { timestamp: i * 1000, frame_id: i, entry_type: 0, offset: 0 });
        }
        let results = ts.range(3000, 7000);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_by_frame() {
        let mut ts = TimeSegment::new();
        ts.insert(TimeEntry { timestamp: 1000, frame_id: 42, entry_type: 0, offset: 0 });
        ts.insert(TimeEntry { timestamp: 2000, frame_id: 42, entry_type: 1, offset: 100 });
        ts.insert(TimeEntry { timestamp: 3000, frame_id: 99, entry_type: 0, offset: 200 });
        let results = ts.by_frame(42);
        assert_eq!(results.len(), 2);
    }
}
