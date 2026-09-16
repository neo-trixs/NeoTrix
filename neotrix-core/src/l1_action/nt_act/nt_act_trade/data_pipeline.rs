//! Trade Data Pipeline — 外部平台数据提取与同步
//!
//! 支持多平台适配器注册、增量同步、数据归一化。
//! 所有 platform extractors 实现 `ExternalPlatformExtractor` trait，
//! 由 `TradeDataPipeline` 统一调度。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::unified_types::{ContactInfo, Customer, Grade};

// ============================================================
// 1. 配置类型
// ============================================================

/// 数据提取配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractConfig {
    /// 分页大小 (默认 100)
    pub page_size: Option<u32>,
    /// 最大记录数 (0 = 无限制)
    pub max_records: Option<u32>,
    /// 过滤条件 (平台特有的 key-value)
    pub filters: HashMap<String, String>,
}

/// 邮件提取配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailConfig {
    /// 起始时间
    pub since: Option<DateTime<Utc>>,
    /// 结束时间
    pub until: Option<DateTime<Utc>>,
    /// 最大数量
    pub max_count: Option<u32>,
    /// 只读未读
    pub unread_only: bool,
}

/// 平台交互记录 (通用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// 记录 ID
    pub id: String,
    /// 关联客户 ID
    pub customer_id: String,
    /// 交互类型 (如 email, phone, meeting)
    pub interaction_type: String,
    /// 交互摘要
    pub summary: String,
    /// 发生时间
    pub timestamp: DateTime<Utc>,
    /// 平台原始数据 (platform-specific payload)
    pub raw: Option<serde_json::Value>,
}

/// 平台邮件 (通用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// 邮件 ID
    pub id: String,
    /// 发件人
    pub from: String,
    /// 收件人列表
    pub to: Vec<String>,
    /// 主题
    pub subject: String,
    /// 正文 (plain text)
    pub body: String,
    /// 发送时间
    pub sent_at: DateTime<Utc>,
    /// 是否已读
    pub is_read: bool,
    /// 关联客户 ID
    pub customer_id: Option<String>,
    /// 平台原始数据
    pub raw: Option<serde_json::Value>,
}

/// 增量同步结果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncResult {
    /// 新增客户数
    pub customers_created: u32,
    /// 更新客户数
    pub customers_updated: u32,
    /// 新增交互记录数
    pub interactions_created: u32,
    /// 新增邮件数
    pub emails_created: u32,
    /// 本次同步截止时间 (可作为下一次的 last_sync)
    pub synced_until: DateTime<Utc>,
    /// 同步过程中遇到的错误
    pub errors: Vec<String>,
}

/// 提取结果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractionResult {
    /// 提取到的客户列表
    pub customers: Vec<Customer>,
    /// 提取到的交互记录 (按 customer_id 索引)
    pub interactions: HashMap<String, Vec<Interaction>>,
    /// 提取到的邮件
    pub emails: Vec<Email>,
    /// 提取耗时 (毫秒)
    pub duration_ms: u64,
}

// ============================================================
// 2. ExternalPlatformExtractor trait
// ============================================================

/// 外部平台数据提取器 trait
///
/// 每个平台 (如 Alibaba, MadeInChina 等) 实现此 trait，
/// 由 `PlatformRegistry` 注册管理。
#[async_trait]
pub trait ExternalPlatformExtractor: Send + Sync {
    /// 平台标识符 (如 "alibaba", "made_in_china")
    fn platform_id(&self) -> &str;

    /// 平台显示名称
    fn display_name(&self) -> &str;

    /// 提取客户列表
    async fn extract_customers(
        &self,
        config: ExtractConfig,
    ) -> Result<Vec<Customer>, String>;

    /// 提取交互记录
    async fn extract_interactions(
        &self,
        customer_id: &str,
    ) -> Result<Vec<Interaction>, String>;

    /// 提取邮件
    async fn extract_emails(
        &self,
        config: EmailConfig,
    ) -> Result<Vec<Email>, String>;

    /// 增量同步 (从 last_sync 时刻起)
    async fn sync_incremental(
        &self,
        last_sync: DateTime<Utc>,
    ) -> Result<SyncResult, String>;
}

// ============================================================
// 3. DataNormalizer — 数据归一化
// ============================================================

/// 数据归一化器
///
/// 将不同平台的原始字段映射到统一数据模型，
/// 处理格式差异、编码转换、空值填充等。
#[derive(Debug, Clone, Default)]
pub struct DataNormalizer;

impl DataNormalizer {
    /// 归一化客户记录
    ///
    /// - 清理空白字段
    /// - 标准化渠道名称
    /// - 填充默认等级
    pub fn normalize_customer(&self, mut customer: Customer) -> Customer {
        customer.name = customer.name.trim().to_string();
        customer.contact = self.normalize_contact(customer.contact);
        if customer.grade == Grade::E && !customer.tags.is_empty() {
            // 有标签但等级为最低 → 升至 C
            customer.grade = Grade::C;
        }
        customer
    }

    /// 归一化联系信息
    pub fn normalize_contact(&self, mut contact: ContactInfo) -> ContactInfo {
        contact.name = contact.name.trim().to_string();
        contact.email = contact.email.trim().to_lowercase();
        contact.phone = contact.phone.trim().to_string();
        contact
    }

    /// 归一化交互记录
    pub fn normalize_interaction(&self, mut interaction: Interaction) -> Interaction {
        interaction.summary = interaction.summary.trim().to_string();
        interaction.interaction_type = interaction.interaction_type.to_lowercase();
        interaction
    }

    /// 归一化邮件
    pub fn normalize_email(&self, mut email: Email) -> Email {
        email.subject = email.subject.trim().to_string();
        email.body = email.body.trim().to_string();
        email.from = email.from.trim().to_lowercase();
        email
    }

    /// 批量归一化客户
    pub fn normalize_customers(&self, customers: Vec<Customer>) -> Vec<Customer> {
        customers.into_iter().map(|c| self.normalize_customer(c)).collect()
    }
}

// ============================================================
// 4. PlatformRegistry — 平台注册表
// ============================================================

/// 平台注册表
///
/// 管理所有已注册的 platform extractors，支持按 ID 查找。
pub struct PlatformRegistry {
    extractors: HashMap<String, Arc<dyn ExternalPlatformExtractor>>,
}

impl PlatformRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            extractors: HashMap::new(),
        }
    }

    /// 注册平台提取器
    pub fn register(&mut self, extractor: Arc<dyn ExternalPlatformExtractor>) {
        let id = extractor.platform_id().to_string();
        self.extractors.insert(id, extractor);
    }

    /// 获取平台提取器
    pub fn get(&self, platform_id: &str) -> Option<&Arc<dyn ExternalPlatformExtractor>> {
        self.extractors.get(platform_id)
    }

    /// 列出所有已注册平台 ID
    pub fn platform_ids(&self) -> Vec<&str> {
        self.extractors.keys().map(|s| s.as_str()).collect()
    }

    /// 已注册平台数量
    pub fn len(&self) -> usize {
        self.extractors.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.extractors.is_empty()
    }
}

impl Default for PlatformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 5. TradeDataPipeline — 统一数据提取管线
// ============================================================

/// 贸易数据管线
///
/// 统一调度多平台数据提取、归一化、增量同步。
pub struct TradeDataPipeline {
    registry: PlatformRegistry,
    normalizer: DataNormalizer,
}

impl TradeDataPipeline {
    /// 创建新的数据管线
    pub fn new(registry: PlatformRegistry, normalizer: DataNormalizer) -> Self {
        Self { registry, normalizer }
    }

    /// 创建带默认归一器的管线
    pub fn with_registry(registry: PlatformRegistry) -> Self {
        Self {
            registry,
            normalizer: DataNormalizer::default(),
        }
    }

    /// 获取底层注册表引用
    pub fn registry(&self) -> &PlatformRegistry {
        &self.registry
    }

    /// 获取底层注册表可变引用
    pub fn registry_mut(&mut self) -> &mut PlatformRegistry {
        &mut self.registry
    }

    /// 提取指定平台的全部数据 (customers + interactions + emails)
    pub async fn extract_all(
        &self,
        platform: &str,
    ) -> Result<ExtractionResult, String> {
        let extractor = self
            .registry
            .get(platform)
            .ok_or_else(|| format!("platform '{}' not registered", platform))?;

        let start = std::time::Instant::now();

        let config = ExtractConfig::default();
        let email_config = EmailConfig::default();

        // 并行提取 customers 和 emails
        let (customers_res, emails_res) = tokio::join!(
            extractor.extract_customers(config.clone()),
            extractor.extract_emails(email_config),
        );

        let customers = self.normalizer.normalize_customers(customers_res?);
        let emails: Vec<Email> = emails_res?
            .into_iter()
            .map(|e| self.normalizer.normalize_email(e))
            .collect();

        // 逐客户提取交互记录
        let mut interactions = HashMap::new();
        for customer in &customers {
            match extractor.extract_interactions(&customer.id).await {
                Ok(records) => {
                    let normalized: Vec<Interaction> = records
                        .into_iter()
                        .map(|r| self.normalizer.normalize_interaction(r))
                        .collect();
                    interactions.insert(customer.id.clone(), normalized);
                }
                Err(e) => {
                    // 单个客户提取失败不阻塞整个管线
                    interactions
                        .entry(customer.id.clone())
                        .or_default();
                    eprintln!(
                        "[data_pipeline] failed to extract interactions for customer {}: {}",
                        customer.id, e
                    );
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ExtractionResult {
            customers,
            interactions,
            emails,
            duration_ms,
        })
    }

    /// 增量同步指定平台
    pub async fn sync_incremental(
        &self,
        platform: &str,
        last_sync: DateTime<Utc>,
    ) -> Result<SyncResult, String> {
        let extractor = self
            .registry
            .get(platform)
            .ok_or_else(|| format!("platform '{}' not registered", platform))?;

        extractor.sync_incremental(last_sync).await
    }

    /// 提取所有已注册平台的全部数据
    pub async fn extract_all_platforms(&self) -> Result<HashMap<String, ExtractionResult>, String> {
        let mut results = HashMap::new();
        for platform_id in self.registry.platform_ids() {
            let platform = platform_id.to_string();
            match self.extract_all(&platform).await {
                Ok(result) => {
                    results.insert(platform, result);
                }
                Err(e) => {
                    eprintln!("[data_pipeline] failed to extract from '{}': {}", platform, e);
                }
            }
        }
        Ok(results)
    }
}

// ============================================================
// 6. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::unified_types::Channel;

    // ── Mock extractor for compile verification ──

    struct MockPlatformExtractor;

    #[async_trait]
    impl ExternalPlatformExtractor for MockPlatformExtractor {
        fn platform_id(&self) -> &str {
            "mock_platform"
        }

        fn display_name(&self) -> &str {
            "Mock Platform"
        }

        async fn extract_customers(
            &self,
            _config: ExtractConfig,
        ) -> Result<Vec<Customer>, String> {
            Ok(vec![Customer {
                id: "c1".into(),
                name: "Test Customer".into(),
                contact: ContactInfo::default(),
                grade: Grade::B,
                channel: Channel::Email,
                country: "CN".into(),
                tags: vec![],
                description: String::new(),
            }])
        }

        async fn extract_interactions(
            &self,
            _customer_id: &str,
        ) -> Result<Vec<Interaction>, String> {
            Ok(vec![Interaction {
                id: "i1".into(),
                customer_id: "c1".into(),
                interaction_type: "email".into(),
                summary: "Inquiry about ball valves".into(),
                timestamp: Utc::now(),
                raw: None,
            }])
        }

        async fn extract_emails(
            &self,
            _config: EmailConfig,
        ) -> Result<Vec<Email>, String> {
            Ok(vec![Email {
                id: "e1".into(),
                from: "buyer@example.com".into(),
                to: vec!["sales@company.com".into()],
                subject: "RFQ for Gate Valves".into(),
                body: "We need 100 units.".into(),
                sent_at: Utc::now(),
                is_read: false,
                customer_id: Some("c1".into()),
                raw: None,
            }])
        }

        async fn sync_incremental(
            &self,
            _last_sync: DateTime<Utc>,
        ) -> Result<SyncResult, String> {
            Ok(SyncResult {
                customers_created: 1,
                customers_updated: 0,
                interactions_created: 1,
                emails_created: 1,
                synced_until: Utc::now(),
                errors: vec![],
            })
        }
    }

    #[test]
    fn test_registry_and_pipeline_compile() {
        let mut registry = PlatformRegistry::new();
        assert!(registry.is_empty());

        registry.register(Arc::new(MockPlatformExtractor));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.platform_ids(), vec!["mock_platform"]);

        let pipeline = TradeDataPipeline::with_registry(registry);
        assert!(!pipeline.registry().is_empty());
    }

    #[test]
    fn test_normalizer_customer_upgrade() {
        let normalizer = DataNormalizer::default();
        let customer = Customer {
            id: "c1".into(),
            name: "  Acme Corp  ".into(),
            contact: ContactInfo {
                name: "  John  ".into(),
                email: "  JOHN@EXAMPLE.COM  ".into(),
                phone: "  +86 138 0000 0000  ".into(),
                title: "Manager".into(),
                wechat: None,
            },
            grade: Grade::E, // lowest grade
            channel: Channel::Exhibition,
            country: "US".into(),
            tags: vec!["vip_prospect".into()], // has tags
            description: "Important lead".into(),
        };

        let normalized = normalizer.normalize_customer(customer);
        assert_eq!(normalized.name, "Acme Corp");
        assert_eq!(normalized.contact.name, "John");
        assert_eq!(normalized.contact.email, "john@example.com");
        // grade upgraded because tags were non-empty
        assert_eq!(normalized.grade, Grade::C);
    }

    #[tokio::test]
    async fn test_extract_all_with_mock() {
        let mut registry = PlatformRegistry::new();
        registry.register(Arc::new(MockPlatformExtractor));

        let pipeline = TradeDataPipeline::with_registry(registry);
        let result = pipeline.extract_all("mock_platform").await.unwrap();

        assert_eq!(result.customers.len(), 1);
        assert_eq!(result.customers[0].id, "c1");
        assert_eq!(result.interactions.len(), 1);
        assert!(result.interactions.contains_key("c1"));
        assert_eq!(result.emails.len(), 1);
    }

    #[tokio::test]
    async fn test_sync_incremental_with_mock() {
        let mut registry = PlatformRegistry::new();
        registry.register(Arc::new(MockPlatformExtractor));

        let pipeline = TradeDataPipeline::with_registry(registry);
        let sync_result = pipeline
            .sync_incremental("mock_platform", Utc::now())
            .await
            .unwrap();

        assert_eq!(sync_result.customers_created, 1);
        assert_eq!(sync_result.interactions_created, 1);
        assert_eq!(sync_result.emails_created, 1);
        assert!(sync_result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_extract_unknown_platform_returns_error() {
        let registry = PlatformRegistry::new();
        let pipeline = TradeDataPipeline::with_registry(registry);

        let result = pipeline.extract_all("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not registered"));
    }
}
