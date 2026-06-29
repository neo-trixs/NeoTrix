//! # L3: ContentIndex — 文件内容 → VSA 向量语义索引
//!
//! 基于 trigram 哈希的 VSA 编码，实现文件内容的语义级搜索。
//! 利用现有 `VsaVector<4096>` 和 ngram 特征，通过 `MapVsaBackend` 运算。
//!
//! ## 编码方案
//! 1. 文件每行 → trigram 哈希序列 → 稀疏线级 VSA 向量
//! 2. 所有行 VSA → `MapVsaBackend::bundle()` (majority sum) → 文件级 VSA
//!
//! ## 查询
//! 查询关键词 → trigram → 倒排索引召回 → Jaccard + VSA 融合排序

use crate::core::nt_core_hcube::vsa_vector::{MapVsaBackend, VsaBackend, VsaVector};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::trigram_hash;

/// L3 内容索引条目
#[derive(Debug, Clone)]
pub struct ContentEntry {
    pub path: PathBuf,
    /// 文件整体 VSA 向量（bundle of line vectors）
    pub vsa: VsaVector<4096>,
    /// trigram 特征集合（用于快速交集匹配）
    pub trigrams: Vec<u64>,
    /// 行数
    pub line_count: usize,
    /// 关键行索引（包含特定 trigrams 的行号）
    pub key_lines: Vec<(usize, String)>,
    /// 最后修改时间
    pub modified: u64,
}

/// L3: 基于 VSA trigram 的内容索引
#[derive(Debug, Clone)]
pub struct ContentIndex {
    pub entries: HashMap<PathBuf, ContentEntry>,
    /// trigram → Path 倒排索引（快速查找）
    pub inverted: HashMap<u64, Vec<PathBuf>>,
    vsa_backend: MapVsaBackend,
}

impl ContentIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            inverted: HashMap::new(),
            vsa_backend: MapVsaBackend,
        }
    }

    /// 索引单个 .rs 文件
    pub fn upsert_rs(&mut self, path: &Path) -> Option<ContentEntry> {
        let content = std::fs::read_to_string(path).ok()?;
        let modified = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        let lines: Vec<&str> = content.lines().collect();
        let mut key_lines = Vec::new();
        let mut all_trigrams = Vec::new();
        let mut line_vectors: Vec<VsaVector<4096>> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // 提取关键行
            let is_key = trimmed.starts_with("pub fn")
                || trimmed.starts_with("pub struct")
                || trimmed.starts_with("pub trait")
                || trimmed.starts_with("pub enum")
                || trimmed.starts_with("pub mod")
                || trimmed.starts_with("impl")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub type")
                || trimmed.starts_with("pub const")
                || trimmed.starts_with("pub unsafe")
                || trimmed.starts_with("#[derive")
                || trimmed.starts_with("#[test]")
                || trimmed.starts_with("#[cfg(test)");

            if is_key {
                key_lines.push((i + 1, line.to_string()));
            }

            // trigram 特征
            let trigrams = trigram_hash(trimmed);
            for &t in &trigrams {
                if !all_trigrams.contains(&t) {
                    all_trigrams.push(t);
                }
                self.inverted.entry(t).or_default().push(path.to_path_buf());
            }

            // 行级 VSA：每个 trigram 位置激活对应维
            let mut line_bytes = vec![0u8; 4096];
            for &t in &trigrams {
                let idx = (t as usize) % 4096;
                line_bytes[idx] = 1;
            }
            if let Ok(lv) = VsaVector::<4096>::from_bytes(line_bytes) {
                line_vectors.push(lv);
            }
        }

        // 文件级 VSA：使用 MapVsaBackend::bundle（majority sum）
        let file_vsa = if line_vectors.is_empty() {
            VsaVector::<4096>::new()
        } else {
            let refs: Vec<&VsaVector<4096>> = line_vectors.iter().collect();
            self.vsa_backend.bundle(&refs)
        };

        let entry = ContentEntry {
            path: path.to_path_buf(),
            vsa: file_vsa,
            trigrams: all_trigrams,
            line_count: lines.len(),
            key_lines,
            modified,
        };

        self.entries.insert(path.to_path_buf(), entry.clone());
        Some(entry)
    }

    /// 语义查询：关键词 → trigram → 倒排 + VSA 融合排序
    pub fn query(&self, q: &super::FileQuery) -> Vec<super::ScoredFile> {
        if q.keywords.is_empty() {
            return Vec::new();
        }

        let query_text = q.keywords.join(" ");
        let query_trigrams = trigram_hash(&query_text);
        let query_set: std::collections::HashSet<u64> = query_trigrams.iter().cloned().collect();

        if query_set.is_empty() {
            return Vec::new();
        }

        // 倒排索引召回
        let mut candidate_scores: HashMap<PathBuf, usize> = HashMap::new();
        for t in &query_set {
            if let Some(paths) = self.inverted.get(t) {
                for p in paths {
                    *candidate_scores.entry(p.clone()).or_insert(0) += 1;
                }
            }
        }

        let total_trigrams = query_set.len() as f64;
        let mut results: Vec<super::ScoredFile> = candidate_scores
            .into_iter()
            .map(|(path, matches)| {
                let score = matches as f64 / total_trigrams;
                let score = if let Some(entry) = self.entries.get(&path) {
                    if let Some(vsa_hint) = &q.vsa_hint {
                        let vsa_sim = self.vsa_backend.similarity(&entry.vsa, vsa_hint);
                        score * 0.3 + vsa_sim * 0.7
                    } else {
                        score
                    }
                } else {
                    score
                };

                super::ScoredFile {
                    path,
                    score,
                    source_layer: "L3:Content",
                    snippet: None,
                    line: None,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(q.top_k);
        results
    }

    /// 获取文件中匹配关键字的上下文行
    pub fn get_snippet(&self, path: &Path, keyword: &str) -> Option<(usize, String)> {
        let content = std::fs::read_to_string(path).ok()?;
        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&keyword.to_lowercase()) {
                return Some((i + 1, line.to_string()));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_index_upsert_and_query() {
        let dir = std::env::temp_dir();
        let f = dir.join("_ci_test.rs");
        std::fs::write(
            &f,
            "pub fn hello() -> i32 {\n    42\n}\n\npub struct Test;\n",
        )
        .ok();

        let mut ci = ContentIndex::new();
        ci.upsert_rs(&f).unwrap();

        let q = super::super::FileQuery::new(vec!["hello".into()]);
        let results = ci.query(&q);
        assert!(!results.is_empty(), "should find file by trigram match");

        let _ = std::fs::remove_file(&f);
    }

    #[test]
    fn test_key_line_detection() {
        let dir = std::env::temp_dir();
        let f = dir.join("_ci_keyline.rs");
        std::fs::write(
            &f,
            "pub fn alpha() {}\npub struct Beta;\nimpl Beta {}\nlet x = 1;\n",
        )
        .ok();

        let mut ci = ContentIndex::new();
        let entry = ci.upsert_rs(&f).unwrap();
        assert!(entry.key_lines.iter().any(|(_, l)| l.contains("alpha")));
        assert!(entry.key_lines.iter().any(|(_, l)| l.contains("Beta")));

        let _ = std::fs::remove_file(&f);
    }
}
