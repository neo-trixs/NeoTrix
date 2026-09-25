//! nt_archiver — 归档系统 (ArchiveEntry/_ArchiveManifest/_ArchiveIndex/_Archiver)，行为零变更纯搬移.

use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::nt_dirs::_CleanupDirs;
use super::nt_types::{_CleanupLog, _CleanupLogEntry};

// ============================================================
// 归档系统
// ============================================================

/// 归档清单条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub source_path: String,   // 原始路径 (相对于项目根)
    pub archived_path: String, // 归档后路径
    pub size_bytes: u64,
    pub is_dir: bool,
    pub archived_at: i64,       // Unix timestamp
    pub cleanup_kind: String,   // 清理类型标签
    pub sha256: Option<String>, // 文件哈希 (可选)
}

/// 归档清单 (每个批次一个)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ArchiveManifest {
    pub batch_id: String, // YYYY-MM-DD_HHMMSS
    pub created_at: i64,
    pub entries: Vec<ArchiveEntry>,
    pub total_bytes: u64,
    pub total_items: usize,
}

impl _ArchiveManifest {
    pub fn new(batch_id: &str) -> Self {
        Self {
            batch_id: batch_id.to_string(),
            created_at: Utc::now().timestamp(),
            entries: Vec::new(),
            total_bytes: 0,
            total_items: 0,
        }
    }
}

/// 归档索引 (全局, 用于搜索)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct _ArchiveIndex {
    pub entries: Vec<ArchiveEntry>,
    pub last_updated: i64,
}

impl _ArchiveIndex {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, &json)
    }
}

/// 归档操作: 将匹配的文件移动到 .cleanup/archive/ 而非删除
pub struct _Archiver {
    pub dirs: _CleanupDirs,
    pub index: _ArchiveIndex,
}

impl _Archiver {
    pub fn new(project_root: &Path) -> Self {
        let dirs = _CleanupDirs::new(project_root);
        let index = _ArchiveIndex::load(&dirs.index_file);
        Self { dirs, index }
    }

    /// 归档一批文件 (移动并记录)
    pub(crate) fn _archive_paths(
        &mut self,
        paths: &[String],
        kind: &str,
    ) -> std::io::Result<_ArchiveManifest> {
        self.dirs.ensure()?;
        let ts = Local::now().format("%Y-%m-%d_%H%M%S");
        let batch_id = ts.to_string();
        let batch_dir = self.dirs._create_archive_batch()?;
        let mut manifest = _ArchiveManifest::new(&batch_id);

        for path_str in paths {
            let src = Path::new(path_str);
            if !src.exists() {
                continue;
            }

            let is_dir = src.is_dir();
            let size = fs::metadata(src).map(|m| m.len()).unwrap_or(0);
            let fname = src
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let dest = batch_dir.join(&fname);

            // 如果目标已存在, 加时间戳后缀
            let dest = if dest.exists() {
                let stem = batch_dir.join(format!("{}_{}", Utc::now().timestamp(), fname));
                stem
            } else {
                dest
            };

            let result = if is_dir {
                // 对目录: 压缩为 tar.gz 再归档
                Self::archive_dir(src, &dest)
            } else {
                fs::rename(src, &dest)
                    .or_else(|_| fs::copy(src, &dest).and_then(|_| fs::remove_file(src)))
            };

            match result {
                Ok(()) => {
                    let entry = ArchiveEntry {
                        source_path: path_str.clone(),
                        archived_path: dest.to_string_lossy().to_string(),
                        size_bytes: size,
                        is_dir,
                        archived_at: Utc::now().timestamp(),
                        cleanup_kind: kind.to_string(),
                        sha256: None,
                    };
                    manifest.entries.push(entry);
                    manifest.total_bytes += size;
                    manifest.total_items += 1;
                }
                Err(e) => {
                    log::warn!("[archiver] 归档失败 {}: {}", path_str, e);
                }
            }
        }

        // 写入清单
        let manifest_path = batch_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, &manifest_json)?;

        // 更新索引
        self.index.entries.extend(manifest.entries.clone());
        self.index.last_updated = Utc::now().timestamp();
        if let Err(e) = self.index.save(&self.dirs.index_file) {
            log::warn!("[archiver] 索引保存失败: {}", e);
        }

        // 记录日志
        _CleanupLog::log(
            &self.dirs.log,
            &_CleanupLogEntry {
                action: "archive".into(),
                kind: kind.into(),
                items: manifest.total_items,
                bytes: manifest.total_bytes,
                batch_id: batch_id.clone(),
                success: true,
                error: None,
            },
        );

        Ok(manifest)
    }

    fn archive_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            let tar_path = dest.with_extension("tar.gz");
            // tar 可用时压缩归档; spawn 失败/非零退出 → 落入 fallback (重命名),
            // 而非静默空归档 (Spice Must Flow: 归档必须移动源, 不允许什么都不做)。
            if let Ok(status) = std::process::Command::new("tar")
                .args(["-czf", &tar_path.to_string_lossy(), "-C"])
                .arg(src.parent().unwrap_or(Path::new(".")))
                .arg(src.file_name().unwrap_or_default())
                .status()
            {
                if status.success() {
                    let _ = fs::remove_dir_all(src);
                    return Ok(());
                }
            }
        }
        // fallback: 简单重命名 (当 tar 不可用/负载下 spawn 失败时)
        let renamed = dest.with_extension("dir");
        fs::rename(src, &renamed)
    }

    /// 搜索归档: 按关键词查找已归档条目
    pub fn search(&self, query: &str) -> Vec<&ArchiveEntry> {
        let q = query.to_lowercase();
        self.index
            .entries
            .iter()
            .filter(|e| {
                e.source_path.to_lowercase().contains(&q)
                    || e.cleanup_kind.to_lowercase().contains(&q)
            })
            .collect()
    }
}
