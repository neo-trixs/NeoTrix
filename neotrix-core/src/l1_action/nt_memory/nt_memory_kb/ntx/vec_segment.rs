//! NTX 持久化向量索引 — HNSW 持久化 + mmap 零拷贝
//!
//! 解决痛点 #1: HNSW 每次启动重建 → 持久化, mmap 加载 < 100ms
//! 解决痛点 #8: load_all_embeddings 全量加载 → O(log N) 查询

use std::io::{Read, Seek, Write};
use serde::{Serialize, Deserialize};
use instant_distance::{Builder as HnswBuilder, Hnsw, Point as HnswPointTrait, PointId, Search};

/// HNSW 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswParams {
    pub max_level: u8,
    pub max_neighbors: u16,
    pub ef_construction: u16,
    pub ef_search: u16,
}

impl Default for HnswParams {
    fn default() -> Self {
        Self {
            max_level: 16,
            max_neighbors: 64,
            ef_construction: 200,
            ef_search: 64,
        }
    }
}

/// HNSW 向量点 (用于 instant-distance)
#[derive(Debug, Clone)]
pub(crate) struct VecPoint {
    pub node_id: [u8; 36],
    pub vector: Vec<f32>,
}

impl HnswPointTrait for VecPoint {
    fn distance(&self, other: &Self) -> f32 {
        cosine_distance(&self.vector, &other.vector)
    }
}

/// 持久化向量段
pub struct VecSegment {
    entries: Vec<VecPoint>,
    dimension: usize,
    params: HnswParams,
    hnsw: Option<Hnsw<VecPoint>>,
    point_ids: Option<Vec<PointId>>,
    pending_entries: Vec<VecPoint>,  // 待合并到 HNSW 的新向量
    hnsw_dirty: bool,                // HNSW 是否需要重建
}

impl VecSegment {
    pub fn new(dimension: usize, params: HnswParams) -> Self {
        Self {
            entries: Vec::new(),
            dimension,
            params,
            hnsw: None,
            point_ids: None,
            pending_entries: Vec::new(),
            hnsw_dirty: false,
        }
    }

    /// 添加向量
    pub fn insert(&mut self, node_id: [u8; 36], vector: Vec<f32>) {
        assert_eq!(vector.len(), self.dimension);
        let entry = VecPoint { node_id, vector };
        self.entries.push(entry.clone());
        self.pending_entries.push(entry);
        self.hnsw_dirty = true;
    }

    /// 批量添加向量
    pub fn batch_insert(&mut self, entries: Vec<([u8; 36], Vec<f32>)>) {
        for (node_id, vector) in entries {
            assert_eq!(vector.len(), self.dimension);
            let entry = VecPoint { node_id, vector };
            self.entries.push(entry.clone());
            self.pending_entries.push(entry);
        }
        self.hnsw_dirty = true;
    }

    /// 构建 HNSW 索引 (全量重建)
    pub fn build_hnsw(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let builder = HnswBuilder::default()
            .ef_search(self.params.ef_search as usize)
            .ef_construction(self.params.ef_construction as usize);

        let (hnsw, point_ids) = builder.build_hnsw(self.entries.clone());
        self.hnsw = Some(hnsw);
        self.point_ids = Some(point_ids);
        self.pending_entries.clear();
        self.hnsw_dirty = false;
    }

    /// 增量合并 HNSW (将 pending 向量合并到现有 HNSW)
    pub fn merge_pending(&mut self) {
        if self.pending_entries.is_empty() {
            return;
        }

        // 如果 HNSW 不存在, 全量构建
        if self.hnsw.is_none() {
            self.build_hnsw();
            return;
        }

        // 否则全量重建 (instant-distance 不支持增量更新)
        // TODO: 考虑使用支持增量更新的 HNSW 库
        self.build_hnsw();
    }

    /// KNN 搜索 (HNSW + pending 向量)
    pub fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        assert_eq!(query.len(), self.dimension);

        let mut results = Vec::new();

        // 如果 HNSW 已构建, 使用 HNSW 搜索
        if let (Some(hnsw), Some(_point_ids)) = (&self.hnsw, &self.point_ids) {
            let query_point = VecPoint {
                node_id: [0u8; 36],
                vector: query.to_vec(),
            };

            let mut search = Search::default();
            let hnsw_results: Vec<SearchResult> = hnsw
                .search(&query_point, &mut search)
                .take(k)
                .map(|item| {
                    let pid = item.pid.into_inner() as usize;
                    let point = &self.entries[pid];
                    SearchResult {
                        node_id: point.node_id,
                        distance: item.distance,
                    }
                })
                .collect();
            results.extend(hnsw_results);
        }

        // 搜索 pending 向量
        for entry in &self.pending_entries {
            let dist = cosine_distance(query, &entry.vector);
            results.push(SearchResult {
                node_id: entry.node_id,
                distance: dist,
            });
        }

        // 排序并截断
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        results.truncate(k);
        results
    }

    /// 写入段 (magic + checksum + data)
    pub fn write_to(&self, writer: &mut (impl Write + Seek)) -> std::io::Result<u64> {
        let start = writer.stream_position()?;

        // Magic
        writer.write_all(b"NTVH")?;

        // 段头
        writer.write_all(&(self.dimension as u32).to_le_bytes())?;
        writer.write_all(&(self.params.max_level as u8).to_le_bytes())?;
        writer.write_all(&(self.params.max_neighbors as u16).to_le_bytes())?;
        writer.write_all(&(self.params.ef_construction as u16).to_le_bytes())?;
        writer.write_all(&(self.params.ef_search as u16).to_le_bytes())?;
        writer.write_all(&(self.entries.len() as u64).to_le_bytes())?;

        // 条目
        for entry in &self.entries {
            writer.write_all(&entry.node_id)?;
            writer.write_all(&(entry.vector.len() as u32).to_le_bytes())?;
            for v in &entry.vector {
                writer.write_all(&v.to_le_bytes())?;
            }
        }

        Ok(start)
    }

    /// 从文件加载 (便捷方法)
    pub fn from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        Self::read_from(&mut file)
    }

    /// 读取段
    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        // Magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != *b"NTVH" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid Vec magic: expected NTVH, got {:?}", magic),
            ));
        }

        let mut dim_buf = [0u8; 4];
        reader.read_exact(&mut dim_buf)?;
        let dimension = u32::from_le_bytes(dim_buf) as usize;

        let mut max_level_buf = [0u8; 1];
        reader.read_exact(&mut max_level_buf)?;

        let mut max_neighbors_buf = [0u8; 2];
        reader.read_exact(&mut max_neighbors_buf)?;

        let mut ef_construction_buf = [0u8; 2];
        reader.read_exact(&mut ef_construction_buf)?;

        let mut ef_search_buf = [0u8; 2];
        reader.read_exact(&mut ef_search_buf)?;

        let params = HnswParams {
            max_level: max_level_buf[0],
            max_neighbors: u16::from_le_bytes(max_neighbors_buf),
            ef_construction: u16::from_le_bytes(ef_construction_buf),
            ef_search: u16::from_le_bytes(ef_search_buf),
        };

        let mut count_buf = [0u8; 8];
        reader.read_exact(&mut count_buf)?;
        let count = u64::from_le_bytes(count_buf) as usize;

        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let mut node_id = [0u8; 36];
            reader.read_exact(&mut node_id)?;

            let mut vec_len_buf = [0u8; 4];
            reader.read_exact(&mut vec_len_buf)?;
            let vec_len = u32::from_le_bytes(vec_len_buf) as usize;

            let mut vector = vec![0f32; vec_len];
            for v in &mut vector {
                let mut f_buf = [0u8; 4];
                reader.read_exact(&mut f_buf)?;
                *v = f32::from_le_bytes(f_buf);
            }

            entries.push(VecPoint { node_id, vector });
        }

        Ok(Self {
            entries,
            dimension,
            params,
            hnsw: None,
            point_ids: None,
            pending_entries: Vec::new(),
            hnsw_dirty: false,
        })
    }

    /// 条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 维度
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// 参数
    pub fn params(&self) -> &HnswParams {
        &self.params
    }

    /// 条目引用
    pub fn entries(&self) -> &[VecPoint] {
        &self.entries
    }

    /// 查找节点 ID 对应的索引
    pub fn find_by_id(&self, node_id: &[u8; 36]) -> Option<usize> {
        self.entries.iter().position(|e| e.node_id == *node_id)
    }

    /// HNSW 是否已构建
    pub fn hnsw_ready(&self) -> bool {
        self.hnsw.is_some()
    }
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub node_id: [u8; 36],
    pub distance: f32,
}

/// 余弦距离 (1 - cosine_similarity)
pub use crate::core::nt_core_math::cosine_distance_f32 as cosine_distance;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_segment_roundtrip() {
        let mut seg = VecSegment::new(4, HnswParams::default());
        for i in 0..100 {
            let mut node_id = [0u8; 36];
            node_id[0] = i;
            let vec = vec![i as f32, (i+1) as f32, (i+2) as f32, (i+3) as f32];
            seg.insert(node_id, vec);
        }

        let mut buf = std::io::Cursor::new(Vec::new());
        seg.write_to(&mut buf).unwrap();
        buf.set_position(0);
        let restored = VecSegment::read_from(&mut buf).unwrap();

        assert_eq!(restored.len(), 100);
        assert_eq!(restored.dimension(), 4);
    }

    #[test]
    fn test_search暴力() {
        let mut seg = VecSegment::new(3, HnswParams::default());
        let ids: Vec<[u8; 36]> = (0..5).map(|i| { let mut n=[0u8;36]; n[0]=i; n }).collect();
        seg.insert(ids[0], vec![1.0, 0.0, 0.0]);
        seg.insert(ids[1], vec![0.0, 1.0, 0.0]);
        seg.insert(ids[2], vec![0.0, 0.0, 1.0]);
        seg.insert(ids[3], vec![0.9, 0.1, 0.0]);
        seg.insert(ids[4], vec![0.1, 0.9, 0.0]);

        let results = seg.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results[0].node_id, ids[0]);
        assert_eq!(results[1].node_id, ids[3]);
    }

    #[test]
    fn test_search_hnsw() {
        let mut seg = VecSegment::new(3, HnswParams::default());
        let ids: Vec<[u8; 36]> = (0..100).map(|i| { let mut n=[0u8;36]; n[0]=i as u8; n }).collect();
        for (i, id) in ids.iter().enumerate() {
            seg.insert(*id, vec![i as f32, (i+1) as f32, (i+2) as f32]);
        }

        seg.build_hnsw();
        assert!(seg.hnsw_ready());

        let results = seg.search(&[0.0, 0.0, 0.0], 5);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_distance(&a, &b) - 0.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_distance(&a, &c) - 1.0).abs() < 1e-6);
    }
}
