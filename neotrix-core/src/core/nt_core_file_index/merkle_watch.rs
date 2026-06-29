//! # MerkleWatch — 增量索引更新
//!
//! 基于 Merkle 树的变更检测，避免全量扫描。
//!
//! ## 机制
//! - 每个文件维护 Merkle 内容哈希
//! - 扫描时对比哈希，只处理变化文件
//! - 可选文件系统通知（通过 mtime 变更检测）
//!
//! ## 集成
//! - 替换 `CodeScanner::scan()` 的全量扫描
//! - 首次全量构建后，后续只增量更新

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Merkle 节点：文件或目录的内容指纹
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub path: PathBuf,
    /// 内容哈希 (xxhash3-like)
    pub hash: u64,
    /// 最后修改时间
    pub mtime: u64,
    /// 子节点哈希（目录用）
    pub children_hash: u64,
    /// 是否为目录
    pub is_dir: bool,
}

/// Merkle 树 + 变更检测
#[derive(Debug, Clone)]
pub struct MerkleWatch {
    /// path → MerkleNode
    pub tree: HashMap<PathBuf, MerkleNode>,
    /// 上次全量扫描时间
    pub last_full_scan: u64,
    /// 不扫描的目录
    skip_dirs: Vec<String>,
}

impl Default for MerkleWatch {
    fn default() -> Self {
        Self {
            tree: HashMap::new(),
            last_full_scan: 0,
            skip_dirs: vec![
                "target".into(),
                ".git".into(),
                "node_modules".into(),
                ".fingerprint".into(),
                "build".into(),
            ],
        }
    }
}

impl MerkleWatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// 全量构建 Merkle 树
    pub fn build_full(&mut self, root: &Path) {
        self.tree.clear();
        self.build_recursive(root);
        self.last_full_scan = now_nanos();
    }

    fn build_recursive(&mut self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if self.skip_dirs.contains(&name) || name.starts_with('.') {
                    continue;
                }

                if path.is_dir() {
                    self.build_recursive(&path);
                }

                let node = self.compute_node(&path);
                self.tree.insert(path, node);
            }
        }
    }

    /// 计算单个文件/目录的 Merkle 节点
    pub fn compute_node(&self, path: &Path) -> MerkleNode {
        let is_dir = path.is_dir();
        let mtime = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        if is_dir {
            // 目录哈希 = 子节点哈希的 XOR
            let mut children_hash: u64 = 0;
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let child = entry.path();
                    if let Some(child_node) = self.tree.get(&child) {
                        children_hash ^= child_node.hash;
                    }
                }
            }
            MerkleNode {
                path: path.to_path_buf(),
                hash: children_hash.wrapping_mul(0x9E3779B97F4A7C15),
                mtime,
                children_hash,
                is_dir: true,
            }
        } else {
            let hash = compute_file_hash(path);
            MerkleNode {
                path: path.to_path_buf(),
                hash,
                mtime,
                children_hash: 0,
                is_dir: false,
            }
        }
    }

    /// 增量扫描：返回变更和新增的文件列表
    pub fn scan_changes(&mut self, root: &Path) -> ScanResult {
        let mut new_files = Vec::new();
        let mut changed_files = Vec::new();
        let mut removed = Vec::new();
        let mut current_paths = std::collections::HashSet::new();

        self.scan_changes_recursive(root, &mut new_files, &mut changed_files, &mut current_paths);

        // 检测删除的文件
        for existing in self.tree.keys() {
            if existing.starts_with(root) && !current_paths.contains(existing) {
                if !existing.is_dir() {
                    removed.push(existing.clone());
                }
            }
        }

        // 清理已删除的索引
        for p in &removed {
            self.tree.remove(p);
        }

        ScanResult {
            new_files,
            changed_files,
            removed,
            total_scanned: current_paths.len(),
        }
    }

    fn scan_changes_recursive(
        &mut self,
        dir: &Path,
        new_files: &mut Vec<PathBuf>,
        changed_files: &mut Vec<PathBuf>,
        current_paths: &mut std::collections::HashSet<PathBuf>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if self.skip_dirs.contains(&name) || name.starts_with('.') {
                    continue;
                }

                current_paths.insert(path.clone());

                if path.is_dir() {
                    self.scan_changes_recursive(&path, new_files, changed_files, current_paths);
                } else {
                    let new_hash = compute_file_hash(&path);
                    match self.tree.get(&path) {
                        Some(existing) if existing.hash != new_hash => {
                            // 内容变化
                            changed_files.push(path.clone());
                            let node = self.compute_node(&path);
                            self.tree.insert(path, node);
                        }
                        None => {
                            // 新文件
                            new_files.push(path.clone());
                            let node = self.compute_node(&path);
                            self.tree.insert(path, node);
                        }
                        _ => {} // 无变化
                    }
                }
            }
        }
    }
}

/// 增量扫描结果
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub new_files: Vec<PathBuf>,
    pub changed_files: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub total_scanned: usize,
}

impl ScanResult {
    pub fn has_changes(&self) -> bool {
        !self.new_files.is_empty() || !self.changed_files.is_empty() || !self.removed.is_empty()
    }
}

/// 文件内容哈希（快速，非密码学安全）
pub fn compute_file_hash(path: &Path) -> u64 {
    let content = std::fs::read(path).unwrap_or_default();
    if content.is_empty() {
        return 0;
    }
    let len = content.len();
    let mut h: u64 = len as u64;

    // 采样：头部 + 中间 + 尾部
    let segments = [
        &content[..content.len().min(256)],
        if len > 512 {
            &content[len / 2 - 128..len / 2 + 128]
        } else {
            &content[..]
        },
        if len > 256 {
            &content[len - 256..]
        } else {
            &content[..]
        },
    ];

    for seg in &segments {
        for chunk in seg.chunks(8) {
            let mut word: u64 = 0;
            for (i, &b) in chunk.iter().enumerate() {
                word |= (b as u64) << (i * 8);
            }
            h = h.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(word);
        }
    }
    h
}

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_detects_new_file() {
        let dir = std::env::temp_dir().join("_mw_test");
        std::fs::create_dir_all(&dir).ok();
        let f = dir.join("a.rs");
        std::fs::write(&f, "fn a() {}").ok();

        let mut mw = MerkleWatch::new();
        // 第一次：全量扫描
        mw.build_full(&dir);

        // 添加新文件
        let f2 = dir.join("b.rs");
        std::fs::write(&f2, "fn b() {}").ok();

        let result = mw.scan_changes(&dir);
        assert!(
            result.new_files.iter().any(|p| p.ends_with("b.rs")),
            "should detect new file"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_merkle_detects_changed_file() {
        let dir = std::env::temp_dir().join("_mw_test2");
        std::fs::create_dir_all(&dir).ok();
        let f = dir.join("a.rs");
        std::fs::write(&f, "fn a() {}").ok();

        let mut mw = MerkleWatch::new();
        mw.build_full(&dir);

        // 修改文件
        std::fs::write(&f, "fn a() { let x = 1; }").ok();

        let result = mw.scan_changes(&dir);
        assert!(
            result.changed_files.iter().any(|p| p.ends_with("a.rs")),
            "should detect change"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_compute_file_hash_deterministic() {
        let dir = std::env::temp_dir();
        let f = dir.join("_hash_test.rs");
        std::fs::write(&f, "const X: i32 = 42;").ok();
        let h1 = compute_file_hash(&f);
        let h2 = compute_file_hash(&f);
        assert_eq!(h1, h2);
        let _ = std::fs::remove_file(&f);
    }
}
