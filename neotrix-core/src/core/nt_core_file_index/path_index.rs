//! # L1: PathIndex — 路径 → VSA 签名索引
//!
//! 将文件路径编码为稀疏 VSA 签名向量，实现 O(1) 哈希调度。
//! 不需要全量扫描目录树即可快速定位已知文件。
//!
//! ## 编码方案
//! - 文件扩展名 → 4-bit 种子
//! - 目录深度 → 3-bit 种子
//! - 父模块名 → 8-bit 种子
//! - 组合 → 12-bit VSA 签名 → 4096 维向量的稀疏激活

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 目录跳过模式（同 scanner.rs）
static SKIP_DIRS: &[&str] = &["target", ".git", "node_modules", ".fingerprint", "build"];

/// 文件类型 → 扩展名字典
static EXT_MAP: &[(&str, u8)] = &[
    ("rs", 1),
    ("py", 2),
    ("ts", 3),
    ("js", 4),
    ("toml", 5),
    ("md", 6),
    ("json", 7),
    ("yaml", 8),
    ("yml", 8),
    ("sh", 9),
    ("rs", 10),
    ("css", 11),
    ("html", 12),
    ("sql", 13),
    ("proto", 14),
    ("lock", 15),
];

/// L1 路径索引条目
#[derive(Debug, Clone)]
pub struct PathEntry {
    pub path: PathBuf,
    /// VSA 签名（稀疏，以种子表示）
    pub signature: u64,
    /// 文件扩展名
    pub ext: String,
    /// 父模块名
    pub module: String,
    /// 目录深度
    pub depth: u8,
    /// 文件大小
    pub size: u64,
    /// 最后修改时间 (ns)
    pub modified: u64,
    /// Merkle 哈希（内容指纹）
    pub merkle_hash: u64,
}

/// L1: 基于 VSA 签名的文件路径调度器
#[derive(Debug, Clone)]
pub struct PathIndex {
    pub root: PathBuf,
    /// path → PathEntry
    pub entries: HashMap<PathBuf, PathEntry>,
    /// module → Vec<path> 快速模块查找
    pub module_index: HashMap<String, Vec<PathBuf>>,
    /// ext → Vec<path> 快速扩展名查找
    pub ext_index: HashMap<String, Vec<PathBuf>>,
    /// 签名碰撞检测
    pub signature_map: HashMap<u64, Vec<PathBuf>>,
}

impl PathIndex {
    pub fn new(root: &str) -> Self {
        Self {
            root: PathBuf::from(root),
            entries: HashMap::new(),
            module_index: HashMap::new(),
            ext_index: HashMap::new(),
            signature_map: HashMap::new(),
        }
    }

    /// 全量扫描目录，构建 L1 索引
    pub fn full_scan(&self) -> Vec<PathEntry> {
        let mut entries = Vec::new();
        self.scan_dir(&self.root, &self.root, 0, &mut entries);
        entries
    }

    fn scan_dir(&self, root: &Path, dir: &Path, depth: u8, out: &mut Vec<PathEntry>) {
        if let Ok(read) = std::fs::read_dir(dir) {
            for entry in read.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                        continue;
                    }
                    self.scan_dir(root, &path, depth + 1, out);
                } else {
                    let ext = path
                        .extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let module = self.guess_module(root, &path);
                    let sig = Self::compute_signature(&ext, &module, depth);
                    let meta = std::fs::metadata(&path).ok();
                    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                    let modified = meta
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(0);
                    let merkle = Self::quick_hash(&path);

                    out.push(PathEntry {
                        path: path.clone(),
                        signature: sig,
                        ext: ext.clone(),
                        module,
                        depth,
                        size,
                        modified,
                        merkle_hash: merkle,
                    });
                }
            }
        }
    }

    /// 增量插入/更新单个文件
    pub fn upsert(&mut self, path: &Path) -> Option<PathEntry> {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let module = self.guess_module(&self.root, path);
        let depth = path
            .components()
            .count()
            .saturating_sub(self.root.components().count());
        let sig = Self::compute_signature(&ext, &module, depth as u8);
        let meta = std::fs::metadata(path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let merkle = Self::quick_hash(path);

        let entry = PathEntry {
            path: path.to_path_buf(),
            signature: sig,
            ext: ext.clone(),
            module: module.clone(),
            depth: depth as u8,
            size,
            modified,
            merkle_hash: merkle,
        };

        // 更新索引
        self.entries.insert(path.to_path_buf(), entry.clone());
        self.module_index
            .entry(module)
            .or_default()
            .push(path.to_path_buf());
        self.ext_index
            .entry(ext)
            .or_default()
            .push(path.to_path_buf());
        self.signature_map
            .entry(sig)
            .or_default()
            .push(path.to_path_buf());

        Some(entry)
    }

    /// 查询：多条件快速调度
    pub fn query(&self, q: &super::FileQuery) -> Vec<super::ScoredFile> {
        let mut candidates: Vec<super::ScoredFile> = Vec::new();

        // 1. 模块过滤
        let by_module = match &q.module_hint {
            Some(m) => self.module_index.get(m).cloned().unwrap_or_default(),
            None => self.entries.keys().cloned().collect(),
        };

        // 2. 扩展名过滤
        let by_ext: Vec<PathBuf> = match &q.ext_filter {
            Some(e) => self.ext_index.get(e).cloned().unwrap_or_default(),
            None => by_module.clone(),
        }
        .into_iter()
        .filter(|p| {
            q.module_hint
                .as_ref()
                .map_or(true, |_m| by_module.contains(p))
        })
        .collect();

        // 3. 关键词匹配（路径层）
        let kw_lower: Vec<String> = q.keywords.iter().map(|k| k.to_lowercase()).collect();
        for path in &by_ext {
            if let Some(entry) = self.entries.get(path) {
                let path_lower = path.to_string_lossy().to_lowercase();
                let name_lower = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let mut match_count = 0;
                for kw in &kw_lower {
                    if path_lower.contains(kw) || name_lower.contains(kw) {
                        match_count += 1;
                    }
                }
                if match_count > 0 || kw_lower.is_empty() {
                    let score = if kw_lower.is_empty() {
                        1.0
                    } else {
                        match_count as f64 / kw_lower.len() as f64
                    };
                    candidates.push(super::ScoredFile {
                        path: entry.path.clone(),
                        score,
                        source_layer: "L1:Path",
                        snippet: None,
                        line: None,
                    });
                }
            }
        }

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(q.top_k);
        candidates
    }

    /// 计算 VSA 签名：将 ext + module + depth 编码为 64-bit 种子
    fn compute_signature(ext: &str, module: &str, depth: u8) -> u64 {
        let ext_seed = EXT_MAP
            .iter()
            .find(|(e, _)| *e == ext)
            .map(|(_, s)| *s as u64)
            .unwrap_or(0);
        let mod_seed: u64 = module
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let depth_seed = depth as u64;
        ext_seed | (mod_seed << 4) | (depth_seed << 56)
    }

    fn guess_module(&self, root: &Path, path: &Path) -> String {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let components: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        if components.len() >= 2 {
            components[0].clone()
        } else {
            "root".to_string()
        }
    }

    /// 快速内容哈希（用于 Merkle 树变更检测）
    fn quick_hash(path: &Path) -> u64 {
        let content = std::fs::read(path).unwrap_or_default();
        let len = content.len();
        if len == 0 {
            return 0;
        }
        // 采样哈希：前 64 字节 + 中间 64 字节 + 后 64 字节
        let mut h: u64 = len as u64;
        let chunks = [
            &content[..content.len().min(64)],
            if len > 128 {
                &content[len / 2 - 32..len / 2 + 32]
            } else {
                &content[..]
            },
            if len > 64 {
                &content[len - 64..]
            } else {
                &content[..]
            },
        ];
        for chunk in &chunks {
            for &b in *chunk {
                h = h.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(b as u64);
            }
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_deterministic() {
        let s1 = PathIndex::compute_signature("rs", "core", 2);
        let s2 = PathIndex::compute_signature("rs", "core", 2);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_signature_differs_for_different_ext() {
        let rs = PathIndex::compute_signature("rs", "core", 2);
        let py = PathIndex::compute_signature("py", "core", 2);
        assert_ne!(rs, py);
    }

    #[test]
    fn test_quick_hash_deterministic() {
        let dir = std::env::temp_dir();
        let p = dir.join("_test_qh_tmp");
        std::fs::write(&p, b"hello world").ok();
        let h1 = PathIndex::quick_hash(&p);
        let h2 = PathIndex::quick_hash(&p);
        assert_eq!(h1, h2);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_path_index_upsert_and_query() {
        let dir = std::env::temp_dir().join("_pi_test");
        std::fs::create_dir_all(&dir).ok();
        let f = dir.join("test.rs");
        std::fs::write(&f, "fn main() {}").ok();

        let mut pi = PathIndex::new(dir.to_str().unwrap());
        pi.upsert(&f);

        let q = super::super::FileQuery::new(vec!["test".into()]);
        let results = pi.query(&q);
        assert!(!results.is_empty());
        assert!(results[0].path.to_string_lossy().contains("test"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
