//! NeoTrix 清理维护模块 — 科学目录架构
//!
//! 目录架构:
//!   project/.cleanup/           清理系统根目录
//!   ├── rules.toml              清理规则配置
//!   ├── archive/                过期文件归档 (按日期分目录)
//!   │   └── YYYY-MM-DD_HHMMSS/  每次归档一个目录
//!   │       └── manifest.json   归档清单 (来源路径/大小/哈希)
//!   ├── log/                    清理日志
//!   │   ├── history.jsonl       追加式事件日志
//!   │   └── index.json          归档搜索索引
//!   └── tmp/                    清理过程中临时文件
//!
//!   project/.backup/            代码备份根目录 (同级, 每6h)
//!   ├── latest -> YYYY-MM-DD_HHMMSS/  最新备份软链接
//!   ├── YYYY-MM-DD_HHMMSS/      按时间戳的备份
//!   │   └── manifest.json       备份清单
//!   └── index.json              备份索引
//!
//! 门面: 子模块重导出，行为零变更纯搬移.
//! 拆分: nt_types(共享类型+日志) / nt_dirs(目录布局) / nt_archiver(归档) / nt_backup(备份) / nt_cleaner(清理引擎+命令清理).

pub mod nt_archiver;
pub mod nt_backup;
pub mod nt_cleaner;
pub mod nt_dirs;
pub mod nt_types;

pub use nt_archiver::{ArchiveEntry, _ArchiveIndex, _ArchiveManifest, _Archiver};
pub use nt_backup::{BackupEngine, _BackupIndex, _BackupManifest};
pub use nt_cleaner::{
    CleanupEngine, CleanupEngineSelfTest, _CleanupResult, _CommandCleaner, _CommandCleanup,
    _CommandResult,
};
pub use nt_dirs::_CleanupDirs;
pub use nt_types::{CleanupKind, CleanupPattern, CleanupRiskLevel, Platform, _CleanupLog, _CleanupLogEntry};

#[cfg(test)]
mod nt_tests;
