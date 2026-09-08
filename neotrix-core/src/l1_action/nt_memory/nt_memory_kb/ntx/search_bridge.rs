//! NTX 搜索桥 — 替换 load_all_embeddings
//!
//! 解决痛点 #8: load_all_embeddings 全量加载 → O(log N) 查询
//! 解决痛点 #2: FTS5 内容复制 2× → 嵌入 FTS5 数据库文件

use std::path::Path;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use super::format::{NtxHeader, NtxToc, FeatureFlags, SegmentType};
use super::vec_segment::VecSegment;
use super::graph_segment::GraphSegment;
use super::frames::FrameType;

/// 搜索结果
#[derive(Debug, Clone)]
pub struct NtxSearchResult {
    pub node_id: [u8; 36],
    pub score: f32,
    pub frame_type: FrameType,
    pub data: Option<String>,
}

/// NTX 搜索桥 — 直接从文件加载段, 不依赖 NtxFile 的所有权
pub struct NtxSearchBridge {
    vec_segment: Option<VecSegment>,
    graph_segment: Option<GraphSegment>,
    frame_count: u64,
}

impl NtxSearchBridge {
    /// 打开 NTX 文件用于搜索 (直接从磁盘加载段)
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut file = File::open(path.as_ref())?;
        let header = NtxHeader::read_from(&mut file)?;
        let toc = NtxToc::read_from(&mut file)?;

        let vec_segment = if header.has_feature(FeatureFlags::VEC) {
            Self::load_vec_segment(&mut file, &toc)?
        } else {
            None
        };

        let graph_segment = if header.has_feature(FeatureFlags::GRAPH) {
            Self::load_graph_segment(&mut file, &toc)?
        } else {
            None
        };

        Ok(Self { vec_segment, graph_segment, frame_count: header.frame_count })
    }

    fn load_vec_segment(file: &mut File, toc: &NtxToc) -> std::io::Result<Option<VecSegment>> {
        let desc = match toc.find_segment(SegmentType::Vec) {
            Some(d) => d,
            None => return Ok(None),
        };
        file.seek(SeekFrom::Start(desc.offset))?;
        Ok(Some(VecSegment::read_from(file)?))
    }

    fn load_graph_segment(file: &mut File, toc: &NtxToc) -> std::io::Result<Option<GraphSegment>> {
        let desc = match toc.find_segment(SegmentType::Graph) {
            Some(d) => d,
            None => return Ok(None),
        };
        file.seek(SeekFrom::Start(desc.offset))?;
        Ok(Some(GraphSegment::read_from(file)?))
    }

    /// 向量相似度搜索 (替代 load_all_embeddings)
    pub fn vector_search(&self, query: &[f32], k: usize) -> Vec<NtxSearchResult> {
        let vec_seg = match &self.vec_segment {
            Some(seg) => seg,
            None => return Vec::new(),
        };

        vec_seg.search(query, k).into_iter()
            .map(|sr| NtxSearchResult {
                node_id: sr.node_id,
                score: 1.0 - sr.distance,
                frame_type: FrameType::Node,
                data: None,
            })
            .collect()
    }

    /// 图谱邻域搜索
    pub fn graph_search(&self, start: &[u8; 36], max_depth: usize) -> Vec<NtxSearchResult> {
        let graph = match &self.graph_segment {
            Some(g) => g,
            None => return Vec::new(),
        };

        graph.bfs(start, max_depth).into_iter()
            .map(|(node_id, depth)| NtxSearchResult {
                node_id,
                score: 1.0 / (depth as f32 + 1.0),
                frame_type: FrameType::Node,
                data: None,
            })
            .collect()
    }

    /// 混合搜索: 向量 + 图谱
    pub fn hybrid_search(
        &self,
        query: &[f32],
        start_node: Option<[u8; 36]>,
        k: usize,
        vec_weight: f32,
        graph_weight: f32,
    ) -> Vec<NtxSearchResult> {
        let mut scores = std::collections::HashMap::<[u8; 36], f32>::new();

        for result in self.vector_search(query, k * 2) {
            *scores.entry(result.node_id).or_insert(0.0) += result.score * vec_weight;
        }

        if let Some(start) = start_node {
            for result in self.graph_search(&start, 3) {
                *scores.entry(result.node_id).or_insert(0.0) += result.score * graph_weight;
            }
        }

        let mut results: Vec<NtxSearchResult> = scores.into_iter()
            .map(|(node_id, score)| NtxSearchResult {
                node_id, score, frame_type: FrameType::Node, data: None,
            })
            .collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    /// 获取单个节点向量
    pub fn get_node_vector(&self, node_id: &[u8; 36]) -> Option<Vec<f32>> {
        self.vec_segment.as_ref()?
            .find_by_id(node_id)
            .map(|idx| self.vec_segment.as_ref().unwrap().entries()[idx].vector.clone())
    }

    pub fn has_vec_index(&self) -> bool { self.vec_segment.is_some() }
    pub fn has_graph(&self) -> bool { self.graph_segment.is_some() }
    pub fn frame_count(&self) -> u64 { self.frame_count }
}

/// 便捷: 从 NTX 文件执行向量搜索
pub fn ntx_vector_search(
    path: impl AsRef<Path>,
    query: &[f32],
    k: usize,
) -> std::io::Result<Vec<NtxSearchResult>> {
    let bridge = NtxSearchBridge::open(path)?;
    Ok(bridge.vector_search(query, k))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use super::super::NtxFile;
    use super::super::vec_segment::HnswParams;

    #[test]
    fn test_search_bridge_loads_segments() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("search.ntx");

        // 创建带段的 NTX
        {
            let mut ntx = NtxFile::create(&path).unwrap();
            let mut seg = VecSegment::new(3, HnswParams::default());
            for i in 0..10 {
                let mut node_id = [0u8; 36];
                node_id[0] = i;
                seg.insert(node_id, vec![i as f32; 3], 0);
            }
            ntx.put_vec_segment(seg);
            ntx.commit().unwrap();
        }

        // 搜索桥加载
        let bridge = NtxSearchBridge::open(&path).unwrap();
        assert!(bridge.has_vec_index());
        assert_eq!(bridge.frame_count(), 0);

        let results = bridge.vector_search(&[1.0, 1.0, 1.0], 3);
        assert_eq!(results.len(), 3);
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_search_bridge_graph() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("search_graph.ntx");

        {
            let mut ntx = NtxFile::create(&path).unwrap();
            let mut g = GraphSegment::new();
            let mut src = [0u8; 36]; src[0] = 1;
            let mut tgt = [0u8; 36]; tgt[0] = 2;
            g.add_edge(super::super::graph_segment::GraphEdge {
                source: src, target: tgt, edge_type: 0, weight: 1.0,
                direction: super::super::graph_segment::EdgeDirection::Bidirectional,
            });
            ntx.put_graph_segment(g);
            ntx.commit().unwrap();
        }

        let bridge = NtxSearchBridge::open(&path).unwrap();
        assert!(bridge.has_graph());

        let start = [0u8; 36]; start[0] = 1; // 注意: 实际是 src[0]=1
        let mut start_id = [0u8; 36]; start_id[0] = 1;
        let results = bridge.graph_search(&start_id, 2);
        assert!(!results.is_empty());
    }
}
