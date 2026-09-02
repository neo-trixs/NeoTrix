/// 下载会话模型 - 持久化到 KV store
/// 单一事实源：`domain_nt_io` namespace via `save_download_session` / `load_download_session`

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSession {
    /// 唯一标识 (UUID v4)
    pub id: String,
    /// 下载源 URL
    pub url: String,
    /// 目标本地路径
    pub path: PathBuf,
    /// 下载状态
    pub status: DownloadStatus,
    /// 进度信息
    pub progress: DownloadProgress,
    /// 创建时间
    pub created_at: SystemTime,
    /// 最后更新时间
    pub updated_at: SystemTime,
    /// 重试计数
    pub retry_count: u32,
    /// 元数据
    pub metadata: DownloadMetadata,
}

/// 下载状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,      // 排队中
    Downloading,  // 进行中
    Paused,       // 暂停
    Completed,    // 完成
    Failed,       // 失败
    Cancelled,    // 取消
}

/// 下载进度结构体
/// 实时更新：percent, downloaded, total, speed, eta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// 百分比 (0.0 - 100.0)
    pub percent: f32,
    /// 已下载字节
    pub downloaded: u64,
    /// 总字节数 (0 表示未知/服务器未提供 Content-Length)
    pub total: u64,
    /// 当前下载速度 (bytes/s)
    pub speed: f64,
    /// 预计剩余秒数 (None 表示无法计算)
    pub eta: Option<f64>,
    /// 最后更新时间戳
    pub last_update: SystemTime,
}

/// 下载元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMetadata {
    /// 下载者标识
    pub user: String,
    /// 备注描述
    pub description: String,
    /// 源类型
    pub source: DownloadSource,
    /// 关联的任务 ID 列表
    pub related_tasks: Vec<String>,
}

/// 下载源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadSource {
    HuggingFace,    // huggingface.co
    GitHub,         // github.com
    GenericURL,     // 自定义 URL
    ModelRepository, // 模型仓库 (如 DavidAU 等)
}

impl Default for DownloadSession {
    fn new(url: String, path: PathBuf, user: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now();
        DownloadSession {
            id,
            url,
            path,
            status: DownloadStatus::Pending,
            progress: DownloadProgress::default(),
            created_at: now,
            updated_at: now,
            retry_count: 0,
            metadata: DownloadMetadata::new(user, String::new()),
        }
    }
}

impl Default for DownloadProgress {
    fn default() -> Self {
        DownloadProgress {
            percent: 0.0,
            downloaded: 0,
            total: 0,
            speed: 0.0,
            eta: None,
            last_update: SystemTime::now(),
        }
    }
}

impl DownloadMetadata {
    pub fn new(user: String, description: String) -> Self {
        DownloadMetadata {
            user,
            description,
            source: DownloadSource::GenericURL,
            related_tasks: Vec::new(),
        }
    }
}