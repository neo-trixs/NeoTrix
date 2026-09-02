/// 下载策略 trait - 所有下载引擎必须实现的契约
/// 定义下载的生命周期接口，便于统一调度和替换

use crate::models::{DownloadSession, DownloadStatus, DownloadSource, DownloadProgress};
use std::path::PathBuf;
use std::time::SystemTime;

pub trait DownloadStrategy: Send + Sync {
    /// 开始或继续下载
    fn start(&self, session: &mut DownloadSession) -> Result<(), DownloadError>;

    /// 暂停下载
    fn pause(&self, session: &mut DownloadSession) -> Result<(), DownloadError>;

    /// 取消下载
    fn cancel(&self, session: &mut DownloadSession) -> Result<(), DownloadError>;

    /// 获取当前状态
    fn status(&self, session: &DownloadSession) -> DownloadStatus;

    /// 重试失败的下载
    fn retry(&self, session: &mut DownloadSession) -> Result<(), DownloadError>;

    /// 检查是否支持断点续传
    fn supports_resume(&self) -> bool {
        true
    }

    /// 检查是否支持代理
    fn supports_proxy(&self) -> bool {
        true
    }

    /// 获取下载源类型
    fn source(&self) -> DownloadSource;

    /// 获取会话关联的任务列表
    fn related_tasks(&self, session: &DownloadSession) -> &[String];

    /// 更新进度 (由引擎周期性调用)
    fn update_progress(&self, session: &mut DownloadSession, progress: DownloadProgress);

    /// 获取会话持久化路径 (内部使用)
    fn session_path(session: &DownloadSession) -> std::path::PathBuf;
}

impl DownloadStrategy for DownloadSession {
    fn start(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        // 默认实现：状态转 Pending → Downloading
        if self.status != DownloadStatus::Pending && self.status != DownloadStatus::Paused {
            return Err(DownloadError::Interrupted(format!("Cannot start from status {:?}", self.status)));
        }
        Ok(())
    }

    fn pause(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        if self.status == DownloadStatus::Downloading {
            // 状态转 Downloading → Paused，保留进度
            Ok(())
        } else {
            Err(DownloadError::Interrupted(format!("Cannot pause from status {:?}", self.status)))
        }
    }

    fn cancel(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        self.status = DownloadStatus::Cancelled;
        self.updated_at = SystemTime::now();
        Ok(())
    }

    fn status(&self, session: &DownloadSession) -> DownloadStatus {
        self.status.clone()
    }

    fn retry(&self, session: &mut DownloadSession) -> Result<(), DownloadError> {
        if self.retry_count < 5 {
            self.retry_count += 1;
            self.status = DownloadStatus::Downloading;
            self.updated_at = SystemTime::now();
            Ok(())
        } else {
            self.status = DownloadStatus::Failed;
            self.updated_at = SystemTime::now();
            Err(DownloadError::Network("Max retries exceeded".to_string()))
        }
    }

    fn supports_resume(&self) -> bool {
        // 检查目标文件是否存在且大小匹配
        true // 具体检查由引擎实现
    }

    fn supports_proxy(&self) -> bool {
        true
    }

    fn source(&self) -> DownloadSource {
        self.metadata.source
    }

    fn related_tasks(&self, _session: &DownloadSession) -> &[String] {
        &self.metadata.related_tasks
    }

    fn update_progress(&self, session: &mut DownloadSession, progress: DownloadProgress) {
        session.progress = progress;
        session.updated_at = SystemTime::now();
        // 计算百分比
        if session.progress.total > 0 {
            session.progress.percent = (session.progress.downloaded as f32 / session.progress.total as f32) * 100.0;
        }
    }

    fn session_path(session: &DownloadSession) -> std::path::PathBuf {
        session.path.clone()
    }
}