//! nt_backup — 备份系统 (_BackupManifest/_BackupIndex/BackupEngine)，行为零变更纯搬移.

use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::nt_types::{_CleanupLog, _CleanupLogEntry};

// ============================================================
// 备份系统 (.backup/)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _BackupManifest {
    pub backup_id: String,
    pub created_at: i64,
    pub project: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub excluded: Vec<String>,
    pub is_incremental: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct _BackupIndex {
    pub backups: Vec<_BackupManifest>,
    pub last_backup: Option<i64>,
}

impl _BackupIndex {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        fs::write(path, &serde_json::to_string_pretty(self)?)
    }
}

pub struct BackupEngine {
    project_root: PathBuf,
    backup_root: PathBuf,
    exclude_patterns: Vec<String>,
    max_backups: usize,
}

impl BackupEngine {
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            backup_root: project_root.join(".backup"),
            exclude_patterns: vec![
                ".backup".into(),
                ".cleanup".into(),
                "target".into(),
                "node_modules".into(),
                ".git".into(),
            ],
            max_backups: 28,
        }
    }

    /// 执行增量备份, 返回备份清单
    pub fn run_backup(&mut self) -> std::io::Result<_BackupManifest> {
        let ts = Local::now().format("%Y-%m-%d_%H%M%S");
        let backup_id = ts.to_string();
        let backup_dir = self.backup_root.join(&backup_id);
        fs::create_dir_all(&backup_dir)?;

        // 收集需要备份的代码文件
        let mut file_count = 0usize;
        let mut total_bytes = 0u64;

        self.collect_files(
            &self.project_root,
            &backup_dir,
            &mut file_count,
            &mut total_bytes,
        )?;

        let manifest = _BackupManifest {
            backup_id: backup_id.clone(),
            created_at: Utc::now().timestamp(),
            project: self.project_root.to_string_lossy().to_string(),
            file_count,
            total_bytes,
            excluded: self.exclude_patterns.clone(),
            is_incremental: true,
        };

        // 写入清单
        let manifest_path = backup_dir.join("manifest.json");
        fs::write(&manifest_path, &serde_json::to_string_pretty(&manifest)?)?;

        // 更新索引
        let index_path = self.backup_root.join("index.json");
        let mut index = _BackupIndex::load(&index_path);
        index.backups.push(manifest.clone());
        index.last_backup = Some(Utc::now().timestamp());
        index.save(&index_path)?;

        // 更新 latest 符号链接
        self.update_latest(&backup_dir);

        // 清理旧备份
        self.prune_old_backups(&index);

        // 记录日志
        let log_dir = self.project_root.join(".cleanup").join("log");
        _CleanupLog::log(
            &log_dir,
            &_CleanupLogEntry {
                action: "backup".into(),
                kind: "code".into(),
                items: file_count,
                bytes: total_bytes,
                batch_id: backup_id,
                success: true,
                error: None,
            },
        );

        Ok(manifest)
    }

    fn collect_files(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
        file_count: &mut usize,
        total_bytes: &mut u64,
    ) -> std::io::Result<()> {
        if !src_dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();

            // 跳过排除项
            if self
                .exclude_patterns
                .iter()
                .any(|p| name_str == *p || name_str.starts_with(p))
            {
                continue;
            }
            // 跳过隐藏目录 (除了 .cleanup 的 rules.toml)
            if name_str.starts_with('.') {
                continue;
            }

            let src_path = entry.path();
            let rel = src_path
                .strip_prefix(&self.project_root)
                .unwrap_or(&src_path);
            let dst_path = dst_dir.join(rel);

            if src_path.is_dir() {
                // 递归
                self.collect_files(&src_path, dst_dir, file_count, total_bytes)?;
            } else {
                // 只备份代码文件
                let ext = src_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let is_code = matches!(
                    ext.as_str(),
                    "rs" | "toml"
                        | "py"
                        | "js"
                        | "ts"
                        | "tsx"
                        | "jsx"
                        | "json"
                        | "css"
                        | "scss"
                        | "html"
                        | "md"
                        | "sh"
                        | "yml"
                        | "yaml"
                        | "sql"
                        | "proto"
                        | "vue"
                        | "svelte"
                        | "rb"
                        | "go"
                        | "mod"
                        | "sum"
                        | "lock"
                        | "conf"
                        | "cfg"
                        | "ini"
                        | "plist"
                );
                if !is_code {
                    continue;
                }

                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&src_path, &dst_path)?;
                let size = fs::metadata(&src_path).map(|m| m.len()).unwrap_or(0);
                *file_count += 1;
                *total_bytes += size;
            }
        }
        Ok(())
    }

    fn update_latest(&self, backup_dir: &Path) {
        let latest = self.backup_root.join("latest");
        // 删除旧符号链接
        let _ = fs::remove_file(&latest);
        // macOS 上目录符号链接需要特殊处理
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let _ = symlink(backup_dir, &latest);
        }
        #[cfg(not(unix))]
        {
            let _ = std::os::windows::fs::symlink_dir(backup_dir, &latest);
        }
    }

    fn prune_old_backups(&self, index: &_BackupIndex) {
        if index.backups.len() <= self.max_backups {
            return;
        }
        let to_remove = index.backups.len() - self.max_backups;
        for backup in index.backups.iter().take(to_remove) {
            let path = self.backup_root.join(&backup.backup_id);
            if path.exists() {
                let _ = fs::remove_dir_all(&path);
            }
        }
    }
}
