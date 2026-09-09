use crate::l2_perception::nt_world::source::types::*;
use std::time::Instant;

/// 播放记录
#[derive(Debug, Clone)]
pub struct PlaybackRecord {
    pub item: MediaItem,
    pub quality: Quality,
    pub source: String,
    pub played_at: Instant,
    pub duration: Option<std::time::Duration>,
}

/// 播放历史
pub struct PlaybackHistory {
    records: Vec<PlaybackRecord>,
    max_size: usize,
}

impl PlaybackHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            records: Vec::new(),
            max_size,
        }
    }

    /// 记录播放
    pub fn record(&mut self, item: MediaItem, quality: Quality, source: String) {
        if self.records.len() >= self.max_size {
            self.records.remove(0);
        }
        self.records.push(PlaybackRecord {
            item,
            quality,
            source,
            played_at: Instant::now(),
            duration: None,
        });
    }

    /// 获取最近播放
    pub fn recent(&self, limit: usize) -> Vec<&PlaybackRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// 搜索历史
    pub fn search(&self, query: &str) -> Vec<&PlaybackRecord> {
        let query_lower = query.to_lowercase();
        self.records.iter()
            .filter(|r| r.item.title.to_lowercase().contains(&query_lower)
                || r.item.artist.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// 清除历史
    pub fn clear(&mut self) {
        self.records.clear();
    }
}
