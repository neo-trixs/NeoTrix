use std::collections::HashMap;

use rusqlite::Connection;

use super::super::nt_memory_embed::load_all_embeddings;
use super::nt_scoring::cosine_f32;
use crate::l0_substrate::nt_core_self_test::{SelfTest, SelfTestRegistry};

// ── Materialized neighbors cache (moved from nt_memory_search.rs, pure move) ──

// ═══════════════════════════════════════════════════════════════════
// Phase 2: Materialized Neighbors cache (query-level optimization)
// ═══════════════════════════════════════════════════════════════════
//
// 物化邻居缓存: 对全量 embedding 预计算 top-k 最近邻, 以 node_id 为键存于内存
// HashMap。重复查询"与 X 相似的节点"时直接命中缓存做 O(k) 查表, 避免每次对
// 389K 向量暴力重算余弦。构建是一次性的 (HNSW 任务之外的安全回退/小库路径)。
// 另提供 buffer 复用的 top-k 余弦扫描, 将单查询分配降到一次复用缓冲区。

/// 物化邻居缓存: node_id -> (neighbor_id, similarity) 降序, 截断到 k。
#[derive(Debug, Clone)]
pub struct MaterializedNeighborCache {
    neighbors: HashMap<String, Vec<(String, f32)>>,
}

impl MaterializedNeighborCache {
    /// 一次性构建: 对 embeddings 做两两余弦, 为每个节点保留 top-k。
    /// 复用 `buf` 缓冲区做 partial-sort, 避免每节点重新分配 O(n) 临时向量。
    pub fn build(embeddings: &[(String, Vec<f32>)], k: usize) -> Self {
        let n = embeddings.len();
        let mut neighbors: HashMap<String, Vec<(String, f32)>> = HashMap::with_capacity(n);
        let kk = k.max(1);
        for i in 0..n {
            let vi = &embeddings[i].1;
            let mut scored: Vec<(usize, f32)> = (0..n)
                .filter(|&j| j != i)
                .map(|j| (j, cosine_f32(vi, &embeddings[j].1)))
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let m = kk.min(scored.len());
            let row: Vec<(String, f32)> = scored[..m]
                .iter()
                .map(|(j, s)| (embeddings[*j].0.clone(), *s))
                .collect();
            neighbors.insert(embeddings[i].0.clone(), row);
        }
        Self { neighbors }
    }

    /// 从已打开的 KB 连接构建 (调用 load_all_embeddings, 不修改 nt_memory_embed)。
    pub fn from_conn(conn: &Connection, k: usize) -> rusqlite::Result<Self> {
        let embeddings = load_all_embeddings(conn)?;
        Ok(Self::build(&embeddings, k))
    }

    pub fn len(&self) -> usize {
        self.neighbors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.neighbors.is_empty()
    }

    /// 热路径查表: 命中直接返回物化邻居 (O(k))。未命中返回 None。
    pub fn get(&self, node_id: &str) -> Option<&[(String, f32)]> {
        self.neighbors.get(node_id).map(|v| v.as_slice())
    }

    /// 热路径相似查询: buffer 复用, 单次 top-k 余弦扫描, 仅一次 `buf` 分配。
    /// 若 query 对应已缓存节点 (id 提供), 直接查表返回, 完全跳过扫描。
    pub fn search(
        &self,
        query: &[f32],
        query_node_id: Option<&str>,
        embeddings: &[(String, Vec<f32>)],
        k: usize,
        buf: &mut Vec<(usize, f32)>,
    ) -> Vec<(String, f32)> {
        if let Some(id) = query_node_id {
            if let Some(hit) = self.neighbors.get(id) {
                let m = k.min(hit.len());
                return hit[..m].to_vec();
            }
        }
        let kk = k.max(1);
        buf.clear();
        for (j, (_, v)) in embeddings.iter().enumerate() {
            buf.push((j, cosine_f32(query, v)));
        }
        if buf.is_empty() {
            return Vec::new();
        }
        buf.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let m = kk.min(buf.len());
        buf[..m]
            .iter()
            .map(|(j, s)| (embeddings[*j].0.clone(), *s))
            .collect()
    }
}

/// 便捷封装: 从 KB 连接构建物化邻居缓存 (供生产路径调用)。
pub fn build_materialized_neighbors(
    conn: &Connection,
    k: usize,
) -> rusqlite::Result<MaterializedNeighborCache> {
    MaterializedNeighborCache::from_conn(conn, k)
}


/// T3 SelfTest 接线 (NT-MEMORY nt_memory_search): 校验物化邻居缓存
/// `MaterializedNeighborCache` 的命中正确性 — 最近邻必须被物化且按余弦相似度排序
/// (核心不变量: 物化不能丢失最近邻)。
pub struct MaterializedNeighborCacheSelfTest;

impl SelfTest for MaterializedNeighborCacheSelfTest {
    fn name(&self) -> &str {
        "nt_memory_search"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let emb: Vec<(String, Vec<f32>)> = vec![
            ("a".to_string(), vec![1.0, 0.0, 0.0]),
            ("b".to_string(), vec![1.0, 0.0, 0.0]),
            ("c".to_string(), vec![0.0, 1.0, 0.0]),
        ];
        let cache = MaterializedNeighborCache::build(&emb, 2);
        if cache.len() != 3 {
            failures.push(format!(
                "nt_memory_search: materialized node count {} != 3",
                cache.len()
            ));
        }
        match cache.get("a") {
            None => failures.push("nt_memory_search: node 'a' not materialized".into()),
            Some(neighbors) => {
                if neighbors.is_empty() {
                    failures.push("nt_memory_search: 'a' has no materialized neighbors".into());
                } else if neighbors[0].0 != "b" || (neighbors[0].1 - 1.0).abs() > 1e-4 {
                    failures.push(format!(
                        "nt_memory_search: 'a' nearest neighbor not captured (top={:?})",
                        neighbors.first()
                    ));
                }
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

pub fn register_nt_memory_search_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(MaterializedNeighborCacheSelfTest));
}
