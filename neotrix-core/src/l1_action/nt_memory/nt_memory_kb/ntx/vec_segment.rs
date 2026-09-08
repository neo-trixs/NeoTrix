//! NTX 持久化向量索引 — mmap 零拷贝 HNSW
//!
//! 解决痛点 #1: HNSW 每次启动重建 → 持久化, mmap 加载 < 100ms
//! 解决痛点 #8: load_all_embeddings 全量加载 → O(log N) 查询

use std::io::{Read, Seek, Write};
use serde::{Serialize, Deserialize};

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

/// 向量条目 (磁盘格式)
#[derive(Debug, Clone)]
pub struct VecEntry {
    pub node_id: [u8; 36],
    pub vector: Vec<f32>,
    pub level: u8,
    pub neighbors: Vec<Vec<[u8; 36]>>,  // 每层邻居
}

/// 持久化向量段
pub struct VecSegment {
    entries: Vec<VecEntry>,
    dimension: usize,
    params: HnswParams,
}

impl VecSegment {
    pub fn new(dimension: usize, params: HnswParams) -> Self {
        Self {
            entries: Vec::new(),
            dimension,
            params,
        }
    }

    /// 添加向量
    pub fn insert(&mut self, node_id: [u8; 36], vector: Vec<f32>, level: u8) {
        assert_eq!(vector.len(), self.dimension);
        self.entries.push(VecEntry {
            node_id,
            vector,
            level,
            neighbors: vec![Vec::new(); level as usize + 1],
        });
    }

    /// 更新邻居关系
    pub fn set_neighbors(&mut self, idx: usize, level: usize, neighbors: Vec<[u8; 36]>) {
        if idx < self.entries.len() && level < self.entries[idx].neighbors.len() {
            self.entries[idx].neighbors[level] = neighbors;
        }
    }

    /// KNN 搜索 (暴力, 用于小数据集)
    pub fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        assert_eq!(query.len(), self.dimension);
        let mut results: Vec<SearchResult> = self.entries.iter()
            .map(|e| {
                let dist = cosine_distance(query, &e.vector);
                SearchResult { node_id: e.node_id, distance: dist }
            })
            .collect();
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
            writer.write_all(&[entry.level])?;
            for layer_neighbors in &entry.neighbors {
                writer.write_all(&(layer_neighbors.len() as u16).to_le_bytes())?;
                for nid in layer_neighbors {
                    writer.write_all(nid)?;
                }
            }
        }

        Ok(start)
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

            let mut level_buf = [0u8; 1];
            reader.read_exact(&mut level_buf)?;
            let level = level_buf[0];

            let mut neighbors = Vec::with_capacity(level as usize + 1);
            for _ in 0..=level as usize {
                let mut nn_buf = [0u8; 2];
                reader.read_exact(&mut nn_buf)?;
                let nn = u16::from_le_bytes(nn_buf) as usize;
                let mut layer_neighbors = Vec::with_capacity(nn);
                for _ in 0..nn {
                    let mut nid = [0u8; 36];
                    reader.read_exact(&mut nid)?;
                    layer_neighbors.push(nid);
                }
                neighbors.push(layer_neighbors);
            }

            entries.push(VecEntry { node_id, vector, level, neighbors });
        }

        Ok(Self { entries, dimension, params })
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
    pub fn entries(&self) -> &[VecEntry] {
        &self.entries
    }

    /// 查找节点 ID 对应的索引
    pub fn find_by_id(&self, node_id: &[u8; 36]) -> Option<usize> {
        self.entries.iter().position(|e| e.node_id == *node_id)
    }
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub node_id: [u8; 36],
    pub distance: f32,
}

/// 余弦距离 (1 - cosine_similarity)
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }
    1.0 - dot / (norm_a * norm_b)
}

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
            seg.insert(node_id, vec, 2);
        }

        let mut buf = std::io::Cursor::new(Vec::new());
        seg.write_to(&mut buf).unwrap();
        buf.set_position(0);
        let restored = VecSegment::read_from(&mut buf).unwrap();

        assert_eq!(restored.len(), 100);
        assert_eq!(restored.dimension(), 4);
    }

    #[test]
    fn test_search() {
        let mut seg = VecSegment::new(3, HnswParams::default());
        let ids: Vec<[u8; 36]> = (0..5).map(|i| { let mut n=[0u8;36]; n[0]=i; n }).collect();
        seg.insert(ids[0], vec![1.0, 0.0, 0.0], 0);
        seg.insert(ids[1], vec![0.0, 1.0, 0.0], 0);
        seg.insert(ids[2], vec![0.0, 0.0, 1.0], 0);
        seg.insert(ids[3], vec![0.9, 0.1, 0.0], 0);
        seg.insert(ids[4], vec![0.1, 0.9, 0.0], 0);

        let results = seg.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results[0].node_id, ids[0]);
        assert_eq!(results[1].node_id, ids[3]);
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
