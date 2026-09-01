//! # KB Vector Index — HNSW 近似最近邻搜索
//!
//! 使用 `instant-distance` crate 提供 O(log n) 语义搜索。
//! 替代线性扫描，390K 向量查询从秒级降至毫秒级。

use instant_distance::{Builder, Search};
use rusqlite::Connection;

/// f32 向量包装，实现 Point trait 用于 HNSW
#[derive(Clone)]
pub struct FloatVec(pub Vec<f32>);

impl instant_distance::Point for FloatVec {
    fn distance(&self, other: &Self) -> f32 {
        let dot: f32 = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let na: f32 = self.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb: f32 = other.0.iter().map(|y| y * y).sum::<f32>().sqrt();
        if na * nb > 0.0 {
            1.0 - dot / (na * nb)
        } else {
            1.0
        }
    }
}

/// HNSW 向量索引
pub struct KbVectorIndex {
    hnsw: instant_distance::HnswMap<FloatVec, String>,
    dim: usize,
}

impl KbVectorIndex {
    /// 从数据库构建 HNSW 索引
    pub fn build(conn: &Connection) -> Result<Self, String> {
        let mut stmt = conn
            .prepare("SELECT node_id, vector FROM embeddings")
            .map_err(|e| e.to_string())?;

        let rows: Vec<(String, Vec<u8>)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let dim = if rows.is_empty() { 384 } else { rows[0].1.len() / 4 };
        println!("  构建索引: {} vectors × {}d", rows.len(), dim);

        // 解析向量
        let mut points = Vec::with_capacity(rows.len());
        let mut values = Vec::with_capacity(rows.len());

        for (nid, blob) in &rows {
            if blob.len() != dim * 4 { continue; }
            let vec: Vec<f32> = blob.chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            points.push(FloatVec(vec));
            values.push(nid.clone());
        }

        // 构建 HNSW 索引
        let t0 = std::time::Instant::now();
        let hnsw = Builder::default().build(points, values);
        println!("  ✅ 索引构建完成 ({:.1}s)", t0.elapsed().as_secs_f64());

        Ok(Self { hnsw, dim })
    }

    /// 搜索最相似的 K 个节点
    pub fn search(
        &self,
        conn: &Connection,
        query_node_id: &str,
        top_k: usize,
    ) -> Result<Vec<(String, f64)>, String> {
        let blob: Vec<u8> = conn
            .query_row(
                "SELECT vector FROM embeddings WHERE node_id=?1",
                [query_node_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        let vec: Vec<f32> = blob
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();

        let query = FloatVec(vec);
        let mut search = Search::default();

        let results: Vec<(String, f64)> = self
            .hnsw
            .search(&query, &mut search)
            .take(top_k)
            .filter_map(|item| {
                let sim = (1.0 - item.distance).max(0.0);
                if sim > 0.3 {
                    Some((item.value.clone(), sim as f64))
                } else {
                    None
                }
            })
            .collect();

        Ok(results)
    }

    pub fn dimension(&self) -> usize { self.dim }
}
