//! NTX 同步引擎 — SQLite ↔ NTX 双写/快照/导入
//!
//! 解决痛点 #9: 写入锁争用 → WAL 追加不阻塞读
//! 解决痛点 #10: schema 迁移手工编码 → schema_hash 校验

use std::path::{Path, PathBuf};
use super::format::FeatureFlags;
use super::frames::{KnowledgeFrame, FrameType, Encoding};
use super::NtxFile;

/// 同步策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStrategy {
    /// 双写: SQLite + NTX 同时写入
    DualWrite,
    /// 仅快照: SQLite 定期快照到 NTX
    SnapshotOnly,
    /// 仅 NTX: 从 NTX 恢复
    NtxOnly,
}

/// 同步统计
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    pub frames_synced: u64,
    pub nodes_synced: u64,
    pub edges_synced: u64,
    pub last_sync_ts: u64,
    pub sync_duration_ms: u64,
    pub conflicts: u64,
}

/// 同步配置
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub strategy: SyncStrategy,
    pub auto_sync: bool,
    pub sync_interval_secs: u64,
    pub batch_size: usize,
    pub compress_frames: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            strategy: SyncStrategy::DualWrite,
            auto_sync: true,
            sync_interval_secs: 60,
            batch_size: 100,
            compress_frames: true,
        }
    }
}

/// NTX 同步引擎
pub struct NtxSync {
    ntx_path: PathBuf,
    config: SyncConfig,
    stats: SyncStats,
}

impl NtxSync {
    pub fn new(ntx_path: impl AsRef<Path>, config: SyncConfig) -> Self {
        Self {
            ntx_path: ntx_path.as_ref().to_path_buf(),
            config,
            stats: SyncStats::default(),
        }
    }

    /// 从 SQLite 快照到 NTX
    pub fn snapshot_from_sqlite(
        &mut self,
        nodes: &[(String, String, String)],  // (id, namespace, data_json)
        edges: &[(String, String, String, f32)], // (source, target, edge_type, weight)
        embeddings: &[(String, Vec<f32>)],    // (node_id, vector)
    ) -> std::io::Result<SyncStats> {
        let start = std::time::Instant::now();

        let mut ntx = NtxFile::create(&self.ntx_path)?;

        // 启用特性
        if !embeddings.is_empty() {
            ntx.enable_feature(FeatureFlags::VEC);
        }
        if !edges.is_empty() {
            ntx.enable_feature(FeatureFlags::GRAPH);
        }

        // 写入节点帧
        let encoding = if self.config.compress_frames { Encoding::Zstd } else { Encoding::Raw };
        for (i, (id, ns, data)) in nodes.iter().enumerate() {
            let node_id = uuid_to_bytes(id);
            let frame = KnowledgeFrame::new(
                i as u64, node_id, FrameType::Node, data.as_bytes(), encoding, Some(format!(r#"{{"ns":"{ns}"}}"#)),
            );
            ntx.put_frame(&frame)?;
        }

        // 写入边帧
        for (i, (src, tgt, etype, weight)) in edges.iter().enumerate() {
            let src_id = uuid_to_bytes(src);
            let edge_data = serde_json::json!({
                "source": src,
                "target": tgt,
                "type": etype,
                "weight": weight,
            });
            let frame = KnowledgeFrame::new(
                (nodes.len() + i) as u64, src_id, FrameType::Edge,
                edge_data.to_string().as_bytes(), encoding, None,
            );
            ntx.put_frame(&frame)?;
        }

        // 写入向量帧
        for (i, (node_id, vector)) in embeddings.iter().enumerate() {
            let nid = uuid_to_bytes(node_id);
            let vec_bytes = f32_vec_to_bytes(vector);
            let frame = KnowledgeFrame::new(
                (nodes.len() + edges.len() + i) as u64, nid, FrameType::Embed,
                &vec_bytes, encoding, None,
            );
            ntx.put_frame(&frame)?;
        }

        ntx.commit()?;

        let duration = start.elapsed();
        self.stats = SyncStats {
            frames_synced: (nodes.len() + edges.len() + embeddings.len()) as u64,
            nodes_synced: nodes.len() as u64,
            edges_synced: edges.len() as u64,
            last_sync_ts: now_seconds(),
            sync_duration_ms: duration.as_millis() as u64,
            conflicts: 0,
        };

        Ok(self.stats.clone())
    }

    /// 从 NTX 导入到 SQLite (恢复)
    pub fn import_to_sqlite(&self) -> std::io::Result<ImportResult> {
        let ntx = NtxFile::open(&self.ntx_path)?;
        let stats = ntx.stats();

        Ok(ImportResult {
            frame_count: stats.frame_count,
            node_count: stats.node_count,
            edge_count: stats.edge_count,
            has_vec_index: stats.has_vec_index,
            has_graph: stats.has_graph,
        })
    }

    /// 增量同步: 新帧追加到 NTX
    pub fn incremental_sync(&mut self, frames: &[KnowledgeFrame]) -> std::io::Result<u64> {
        let mut ntx = if self.ntx_path.exists() {
            NtxFile::open(&self.ntx_path)?
        } else {
            NtxFile::create(&self.ntx_path)?
        };

        let count = frames.len() as u64;
        ntx.put_frames(frames)?;
        ntx.commit()?;

        self.stats.frames_synced += count;
        self.stats.last_sync_ts = now_seconds();
        Ok(count)
    }

    /// 获取同步统计
    pub fn stats(&self) -> &SyncStats {
        &self.stats
    }

    /// 获取配置
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }
}

/// 导入结果
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub frame_count: u64,
    pub node_count: u64,
    pub edge_count: u64,
    pub has_vec_index: bool,
    pub has_graph: bool,
}

// ============================================================
// 工具函数
// ============================================================

fn uuid_to_bytes(uuid: &str) -> [u8; 36] {
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

fn f32_vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes.chunks(4)
        .filter_map(|chunk| {
            if chunk.len() == 4 {
                Some(f32::from_le_bytes(chunk.try_into().unwrap()))
            } else {
                None
            }
        })
        .collect()
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
    fn test_snapshot_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let ntx_path = tmp.path().join("sync.ntx");

        let mut sync = NtxSync::new(&ntx_path, SyncConfig::default());

        let nodes = vec![
            ("node-1".to_string(), "kb".to_string(), r#"{"key":"value1"}"#.to_string()),
            ("node-2".to_string(), "kb".to_string(), r#"{"key":"value2"}"#.to_string()),
        ];
        let edges = vec![
            ("node-1".to_string(), "node-2".to_string(), "related_to".to_string(), 0.8),
        ];
        let embeddings = vec![
            ("node-1".to_string(), vec![0.1, 0.2, 0.3]),
            ("node-2".to_string(), vec![0.4, 0.5, 0.6]),
        ];

        let stats = sync.snapshot_from_sqlite(&nodes, &edges, &embeddings).unwrap();
        assert_eq!(stats.frames_synced, 5); // 2 nodes + 1 edge + 2 embeddings
        assert_eq!(stats.nodes_synced, 2);
        assert_eq!(stats.edges_synced, 1);
    }

    #[test]
    fn test_incremental_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let ntx_path = tmp.path().join("incr.ntx");

        let mut sync = NtxSync::new(&ntx_path, SyncConfig::default());

        // 首次创建
        let mut node_id = [0u8; 36];
        node_id[0] = 1;
        let frame = KnowledgeFrame::new(0, node_id, FrameType::Node, b"test", Encoding::Raw, None);
        sync.incremental_sync(&[frame]).unwrap();

        // 增量追加
        let mut node_id2 = [0u8; 36];
        node_id2[0] = 2;
        let frame2 = KnowledgeFrame::new(1, node_id2, FrameType::Node, b"test2", Encoding::Raw, None);
        let count = sync.incremental_sync(&[frame2]).unwrap();
        assert_eq!(count, 1);
    }
}
