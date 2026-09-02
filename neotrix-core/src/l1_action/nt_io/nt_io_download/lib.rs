/// # NeoTrix 下载引擎
/// 
/// NeoTrix 自研的可续传、分片下载引擎，旨在替代 aria2/huggingface_hub 等外部依赖。
/// 
/// ## 核心特性
/// - **分片并发下载**：默认 8 路并发，可根据带宽自动调整
/// - **HTTP Range 请求**：支持断点续传 (HTTP 206)
/// - **代理集成**：SOCKS5/HTTP 代理自动启用 (通过 Clash Verge 等)
/// - **进度追踪**：实时百分比、速度、ETA 计算
/// - **重试机制**：指数退避备份，最多 5 次重试
/// - **KB 持久化**：下载会话持久化到 SQLite KV store
/// 
/// ## 使用示例
/// 
/// ```rust
/// use neotrix_core::l1_action::nt_io::nt_io_download::{DownloadEngine, EngineConfig, DownloadSession, DownloadSource};
/// use std::path::PathBuf;
/// 
/// # #[tokio::main]
/// # async fn main() {
/// let config = EngineConfig::default();
/// let engine = DownloadEngine::new(config);
/// 
/// let path = PathBuf::from("./models/qwen3.8-27b-IQ3_XS.gguf");
/// let session = DownloadSession::new(
///     "https://huggingface.co/HauhauCS/Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-MTP-GGUF/resolve/main/Qwen3.8-27B-Uncensored-HauhauCS-Aggressive-IQ3_XS.gguf".to_string(),
///     path,
///     "user-001".to_string()
/// );
/// 
/// engine.download_session(&mut session).await.unwrap();
/// println!("下载完成: {} - {:.2}%", session.id, session.progress.percent);
/// # }
/// ```

/// 重新导出关键类型供上层使用
pub use crate::models::{DownloadSession, DownloadProgress, DownloadStatus, DownloadSource, DownloadMetadata};
pub use crate::traits::DownloadStrategy;
pub use crate::engine::DownloadEngine;
pub use crate::proxy::ProxyConfig;

/// 默认引擎配置常量
pub const DEFAULT_CONFIG: EngineConfig = EngineConfig::default();

/// 下载源枚举便利方法
impl DownloadSource {
    /// 从字符串解析源类型
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hf" | "huggingface" | "huggingface.co" => DownloadSource::HuggingFace,
            "github" | "gitHub" => DownloadSource::GitHub,
            "url" | "generic" => DownloadSource::GenericURL,
            _ => DownloadSource::GenericURL,
        }
    }

    /// 是否为 HuggingFace 源
    pub fn is_hf(&self) -> bool {
        matches!(self, DownloadSource::HuggingFace)
    }

    /// 是否为 GitHub 源
    pub fn is_github(&self) -> bool {
        matches!(self, DownloadSource::GitHub)
    }
}