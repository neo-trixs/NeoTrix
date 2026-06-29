//! # Semantic File Index — 从全量扫描到语义索引调度
//!
//! 三层语义索引，替代惯性全量目录扫描：
//!
//! - **L1 PathIndex**: 文件路径 → VSA 签名（O(1) 哈希调度）
//! - **L2 StructureIndex**: 代码结构 → HyperCube 坐标（函数/结构体/特征）
//! - **L3 ContentIndex**: 文件内容 → VSA 向量（ngram 语义搜索）
//!
//! **QueryEngine**: 三层并行查询 + 融合排序
//! **MerkleWatch**: 增量更新（Merkle tree + fs notify）

mod content_index;
mod merkle_watch;
mod path_index;
mod query_engine;
mod structure_index;

pub use content_index::*;
pub use merkle_watch::*;
pub use path_index::*;
pub use query_engine::*;
pub use structure_index::*;

use crate::core::nt_core_hcube::vsa_vector::VsaVector;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 文件索引的完整状态
#[derive(Debug, Clone)]
pub struct FileIndexState {
    pub path_index: PathIndex,
    pub content_index: ContentIndex,
    pub structure_index: StructureIndex,
    pub stats: IndexStats,
}

impl FileIndexState {
    pub fn new(root: &str) -> Self {
        Self {
            path_index: PathIndex::new(root),
            content_index: ContentIndex::new(),
            structure_index: StructureIndex::new(),
            stats: IndexStats::default(),
        }
    }

    /// 增量索引：只重新索引变化的文件
    pub fn update_incremental(&mut self, changed: &[PathBuf]) {
        let now = nanos_now();
        for path in changed {
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                self.path_index.upsert(path);
                self.content_index.upsert_rs(path);
                self.structure_index.upsert_rs(path);
                self.stats.last_indexed = now;
                self.stats.total_files = self.path_index.entries.len();
            }
        }
    }

    /// 全量索引（首次构建或重置）
    pub fn full_rebuild(&mut self) {
        let now = nanos_now();
        let entries = self.path_index.full_scan();
        self.stats.total_files = entries.len();
        self.stats.total_l1_entries = entries.len();

        for entry in &entries {
            let path = Path::new(&entry.path);
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                self.content_index.upsert_rs(path);
                self.structure_index.upsert_rs(path);
            }
        }

        self.stats.last_indexed = now;
        self.stats.last_full_rebuild = now;
    }

    /// 查询：三层并行 + 融合
    pub fn query(&self, q: &FileQuery) -> Vec<ScoredFile> {
        query_engine::fuse_results(
            self.path_index.query(q),
            self.content_index.query(q),
            self.structure_index.query(q),
            q.top_k,
        )
    }
}

/// 索引性能统计
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_l1_entries: usize,
    pub total_l2_entries: usize,
    pub total_l3_entries: usize,
    pub last_indexed: u64,
    pub last_full_rebuild: u64,
    pub total_queries: AtomicCount,
    pub avg_query_us: f64,
}

#[derive(Debug)]
pub struct AtomicCount(std::sync::atomic::AtomicU64);

impl Default for AtomicCount {
    fn default() -> Self {
        Self(std::sync::atomic::AtomicU64::new(0))
    }
}

impl Clone for AtomicCount {
    fn clone(&self) -> Self {
        Self(std::sync::atomic::AtomicU64::new(
            self.0.load(std::sync::atomic::Ordering::Relaxed),
        ))
    }
}

/// 文件查询意图
#[derive(Debug, Clone)]
pub struct FileQuery {
    /// 自然语言关键词（内容语义）
    pub keywords: Vec<String>,
    /// 模块/路径过滤
    pub module_hint: Option<String>,
    /// 文件扩展名过滤
    pub ext_filter: Option<String>,
    /// 最大返回数
    pub top_k: usize,
    /// VSA 查询向量（若已编码）
    pub vsa_hint: Option<VsaVector<4096>>,
}

impl FileQuery {
    pub fn new(keywords: Vec<String>) -> Self {
        Self {
            keywords,
            module_hint: None,
            ext_filter: None,
            top_k: 10,
            vsa_hint: None,
        }
    }

    pub fn with_module(mut self, m: &str) -> Self {
        self.module_hint = Some(m.to_string());
        self
    }

    pub fn with_ext(mut self, e: &str) -> Self {
        self.ext_filter = Some(e.to_string());
        self
    }

    pub fn top(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }
}

/// 排序后的文件匹配结果
#[derive(Debug, Clone)]
pub struct ScoredFile {
    pub path: PathBuf,
    pub score: f64,
    pub source_layer: &'static str,
    pub snippet: Option<String>,
    pub line: Option<usize>,
}

fn nanos_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos() as u64
}

/// trigram 哈希：将字符串映射为 VSA 向量的种子序列
pub fn trigram_hash(s: &str) -> Vec<u64> {
    let bytes = s.as_bytes();
    if bytes.len() < 3 {
        return vec![0u64];
    }
    bytes
        .windows(3)
        .map(|w| {
            let h = ((w[0] as u64) << 16) | ((w[1] as u64) << 8) | (w[2] as u64);
            h.wrapping_mul(0x9E3779B97F4A7C15)
        })
        .collect()
}
