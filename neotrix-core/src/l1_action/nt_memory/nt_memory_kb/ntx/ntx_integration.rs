//! NTX ↔ KnowledgeBase 集成层
//!
//! 替代 load_all_embeddings 的 NTX 向量搜索路径。
//! 当 NTX 文件存在时, 优先从 NTX 加载向量; 否则回退 SQLite。

use std::path::{Path, PathBuf};
use super::{NtxFile, NtxStats};
use super::vec_segment::{VecSegment, HnswParams};
use super::frames::{KnowledgeFrame, FrameType, Encoding};
use super::graph_segment::{GraphSegment, GraphEdge, EdgeDirection};
use rusqlite::Connection;

/// NTX 索引管理器
pub struct NtxIndexManager {
    ntx_path: PathBuf,
}

impl NtxIndexManager {
    pub fn new(ntx_dir: impl AsRef<Path>, db_name: &str) -> Self {
        let ntx_path = ntx_dir.as_ref().join(format!("{db_name}.ntx"));
        Self { ntx_path }
    }

    /// 从 SQLite 全量同步到 NTX (增量由 sync 模块处理)
    pub fn full_sync(&self, conn: &Connection, dimension: usize) -> std::io::Result<NtxStats> {
        let mut ntx = NtxFile::create(&self.ntx_path)?;

        // 1. 同步节点帧
        let mut stmt = conn.prepare(
            "SELECT id, namespace, data_json FROM nodes"
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let nodes: Vec<(String, String, String)> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

        let encoding = Encoding::Zstd;
        for (i, (id, ns, data)) in nodes.iter().enumerate() {
            let node_id = uuid_to_bytes(id);
            let frame = KnowledgeFrame::new(
                i as u64, node_id, FrameType::Node,
                data.as_bytes(), encoding,
                Some(format!(r#"{{"ns":"{}"}}"#, ns)),
            );
            ntx.put_frame(&frame)?;
        }

        // 2. 同步边帧
        let mut stmt = conn.prepare(
            "SELECT source_id, target_id, edge_type, weight FROM edges"
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let edges: Vec<(String, String, String, f32)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

        for (i, (src, tgt, etype, weight)) in edges.iter().enumerate() {
            let src_id = uuid_to_bytes(src);
            let edge_data = serde_json::json!({
                "source": src, "target": tgt, "type": etype, "weight": weight,
            });
            let frame = KnowledgeFrame::new(
                (nodes.len() + i) as u64, src_id, FrameType::Edge,
                edge_data.to_string().as_bytes(), encoding, None,
            );
            ntx.put_frame(&frame)?;
        }

        // 3. 同步向量段
        let mut stmt = conn.prepare(
            "SELECT e.node_id, e.vector FROM embeddings e JOIN nodes n ON n.id = e.node_id"
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let embeddings: Vec<(String, Vec<f32>)> = stmt.query_map([], |row| {
            let node_id: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((node_id, blob_to_vector(&blob)))
        })
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

        if !embeddings.is_empty() {
            let mut vec_seg = VecSegment::new(dimension, HnswParams::default());
            for (i, (node_id, vector)) in embeddings.iter().enumerate() {
                if vector.len() == dimension {
                    let nid = uuid_to_bytes(node_id);
                    vec_seg.insert(nid, vector.clone(), 0);
                }
            }
            ntx.put_vec_segment(vec_seg);
        }

        // 4. 同步图谱段
        if !edges.is_empty() {
            let mut graph_seg = GraphSegment::new();
            for (src, tgt, etype, weight) in &edges {
                graph_seg.add_edge(GraphEdge {
                    source: uuid_to_bytes(src),
                    target: uuid_to_bytes(tgt),
                    edge_type: hash_type(etype),
                    weight: *weight,
                    direction: EdgeDirection::Bidirectional,
                });
            }
            ntx.put_graph_segment(graph_seg);
        }

        ntx.commit()?;

        Ok(NtxStats {
            frame_count: (nodes.len() + edges.len()) as u64,
            node_count: nodes.len() as u64,
            edge_count: edges.len() as u64,
            ..Default::default()
        })
    }

    /// NTX 向量搜索 (替代 load_all_embeddings)
    pub fn vector_search(&self, query: &[f32], k: usize) -> std::io::Result<Vec<(String, f32)>> {
        let ntx = NtxFile::open_read_only(&self.ntx_path)?;
        let vec_seg = match ntx.vec_segment() {
            Some(seg) => seg,
            None => return Ok(Vec::new()),
        };

        let results = vec_seg.search(query, k);
        Ok(results.into_iter()
            .map(|r| (bytes_to_uuid(&r.node_id), 1.0 - r.distance))
            .collect())
    }

    /// NTX 图谱邻居搜索
    pub fn graph_neighbors(&self, node_id: &str, max_depth: usize) -> std::io::Result<Vec<(String, usize)>> {
        let ntx = NtxFile::open_read_only(&self.ntx_path)?;
        let graph = match ntx.graph_segment() {
            Some(g) => g,
            None => return Ok(Vec::new()),
        };

        let nid = uuid_to_bytes(node_id);
        Ok(graph.bfs(&nid, max_depth).into_iter()
            .map(|(id, depth)| (bytes_to_uuid(&id), depth))
            .collect())
    }

    /// 检查 NTX 文件是否存在
    pub fn exists(&self) -> bool {
        self.ntx_path.exists()
    }

    /// 获取 NTX 文件路径
    pub fn path(&self) -> &Path {
        &self.ntx_path
    }

    /// 获取统计
    pub fn stats(&self) -> std::io::Result<NtxStats> {
        let ntx = NtxFile::open_read_only(&self.ntx_path)?;
        Ok(ntx.stats())
    }
}

// ── 工具函数 ──────────────────────────────────────

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

fn bytes_to_uuid(bytes: &[u8; 36]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

fn blob_to_vector(blob: &[u8]) -> Vec<f32> {
    blob.chunks(4)
        .filter_map(|c| {
            if c.len() == 4 { Some(f32::from_le_bytes(c.try_into().unwrap())) }
            else { None }
        })
        .collect()
}

fn hash_type(s: &str) -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_roundtrip() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let bytes = uuid_to_bytes(uuid);
        let restored = bytes_to_uuid(&bytes);
        assert_eq!(restored.len(), 36);
    }
}
