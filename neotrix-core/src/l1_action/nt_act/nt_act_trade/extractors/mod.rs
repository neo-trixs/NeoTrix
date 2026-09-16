//! External Platform Extractors — 外部 TMS/CRM 平台数据提取器
//!
//! 定义统一的 `ExternalPlatformExtractor` trait，各平台适配器实现此 trait。
//! 支持批量提取、增量同步、进度回调。

#![forbid(unsafe_code)]

pub mod chrome_decrypt;
pub mod joinf;
pub mod selenium_automation;

pub use chrome_decrypt::{ChromeDecryptor, LoginEntry};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::nt_trade_crm::{CustomerProfile, InteractionRecord};
use super::nt_trade_email::EmailRecord;

// ============================================================
// 1. 提取配置
// ============================================================

/// 客户提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractConfig {
    /// 每页大小
    pub page_size: u32,
    /// 最大页数 (0 = 不限)
    pub max_pages: u32,
    /// 只提取指定客户 ID 列表 (空 = 全量)
    pub customer_ids: Vec<i64>,
    /// 最小客户等级过滤
    pub min_grade: Option<String>,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            page_size: 100,
            max_pages: 0,
            customer_ids: Vec::new(),
            min_grade: None,
        }
    }
}

/// 邮件提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// 每页大小
    pub page_size: u32,
    /// 最大页数
    pub max_pages: u32,
    /// 邮箱箱 ID (-1 = 全部)
    pub box_id: i32,
    /// 只提取未读邮件
    pub unread_only: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            page_size: 50,
            max_pages: 0,
            box_id: -1,
            unread_only: false,
        }
    }
}

/// 增量同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// 新增客户数
    pub new_customers: u32,
    /// 更新客户数
    pub updated_customers: u32,
    /// 新增交互记录数
    pub new_interactions: u32,
    /// 新增邮件数
    pub new_emails: u32,
    /// 同步时间范围
    pub synced_from: DateTime<Utc>,
    pub synced_to: DateTime<Utc>,
    /// 错误信息 (非致命)
    pub errors: Vec<String>,
}

/// 单次提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult<T> {
    /// 提取到的数据
    pub data: Vec<T>,
    /// 总记录数
    pub total: u32,
    /// 是否还有更多页
    pub has_more: bool,
}

/// 交互记录 (从客户日志解析)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    /// 日志 ID
    pub id: i64,
    /// 客户 ID
    pub customer_id: i64,
    /// 交互类型 (email/whatsapp/phone/meeting)
    pub interaction_type: String,
    /// 摘要
    pub summary: String,
    /// 详情
    pub detail: String,
    /// 时间戳 (epoch ms)
    pub timestamp: i64,
    /// 操作人
    pub operator: Option<String>,
}

/// 进度回调类型
pub type ProgressCallback = Box<dyn Fn(u32, u32) + Send + Sync>;

// ============================================================
// 2. ExternalPlatformExtractor trait
// ============================================================

/// 外部平台数据提取器统一 trait
///
/// 各平台 (富通天下、孚盟、小满等) 实现此 trait，
/// 消费方通过 trait object 统一调用。
#[async_trait]
pub trait ExternalPlatformExtractor: Send + Sync {
    /// 平台标识 (如 "joinf", "fumee", "xms")
    fn platform_id(&self) -> &str;

    /// 平台显示名
    fn platform_name(&self) -> &str;

    /// 提取客户列表
    async fn extract_customers(
        &self,
        config: ExtractConfig,
        progress: Option<&ProgressCallback>,
    ) -> Result<ExtractionResult<CustomerProfile>, Box<dyn std::error::Error + Send + Sync>>;

    /// 提取客户交互记录 (含 WhatsApp/邮件/电话)
    async fn extract_interactions(
        &self,
        customer_id: &str,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<InteractionRecord>, Box<dyn std::error::Error + Send + Sync>>;

    /// 提取邮件
    async fn extract_emails(
        &self,
        config: EmailConfig,
        progress: Option<&ProgressCallback>,
    ) -> Result<ExtractionResult<EmailRecord>, Box<dyn std::error::Error + Send + Sync>>;

    /// 增量同步 (自 last_sync 以来的所有变更)
    async fn sync_incremental(
        &self,
        last_sync: DateTime<Utc>,
        progress: Option<&ProgressCallback>,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>>;
}
