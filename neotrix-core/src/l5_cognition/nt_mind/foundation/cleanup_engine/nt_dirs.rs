//! nt_dirs — 清理目录布局管理 (_CleanupDirs)，行为零变更纯搬移。

use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================
// Cleanup 目录布局管理
// ============================================================

/// 项目清理系统目录
pub struct _CleanupDirs {
    pub root: PathBuf,       // project/.cleanup/
    pub archive: PathBuf,    // project/.cleanup/archive/
    pub log: PathBuf,        // project/.cleanup/log/
    pub rules_file: PathBuf, // project/.cleanup/rules.toml
    pub index_file: PathBuf, // project/.cleanup/log/index.json
}

impl _CleanupDirs {
    pub fn new(project_root: &Path) -> Self {
        let root = project_root.join(".cleanup");
        Self {
            archive: root.join("archive"),
            log: root.join("log"),
            rules_file: root.join("rules.toml"),
            index_file: root.join("log").join("index.json"),
            root,
        }
    }

    /// 确保所有目录存在，返回 self
    pub fn ensure(&self) -> std::io::Result<&Self> {
        fs::create_dir_all(&self.archive)?;
        fs::create_dir_all(&self.log)?;
        Ok(self)
    }

    /// 创建当前时间戳的归档目录
    pub(crate) fn _create_archive_batch(&self) -> std::io::Result<PathBuf> {
        let ts = Local::now().format("%Y-%m-%d_%H%M%S");
        let batch = self.archive.join(ts.to_string());
        fs::create_dir_all(&batch)?;
        Ok(batch)
    }
}
