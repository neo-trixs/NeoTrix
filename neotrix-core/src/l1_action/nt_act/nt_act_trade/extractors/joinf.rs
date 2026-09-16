//! JoinfExtractor — 富通天下 (joinf.com) 平台数据提取器
//!
//! 通过 REST API + Selenium 自动化提取富通天下 CRM 数据。
//! 支持：
//! - Chrome 密码解密获取登录凭据 (via `chrome_decrypt`)
//! - Selenium 自动登录 + Cookie 管理 (via `selenium_automation`)
//! - 网络日志 API 自动发现
//! - 批量客户/邮件/交互提取
//! - 增量同步

#![forbid(unsafe_code)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::chrome_decrypt::{ChromeDecryptor, LoginEntry};
use super::selenium_automation::{SeleniumConfig, SeleniumSession};
use super::{
    ActivityLog, EmailConfig, ExternalPlatformExtractor, ExtractConfig, ExtractionResult,
    ProgressCallback, SyncResult,
};
use crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::{
    Company, Contact as CrmContact, CustomerGrade, CustomerProfile, CustomerSource, CustomerStatus,
    InteractionRecord, InteractionType,
};
use crate::l1_action::nt_act::nt_act_trade::nt_trade_email::{
    EmailRecord, EmailStatus, EmailTracking,
};

// ============================================================
// 1. 错误类型
// ============================================================

/// JoinfExtractor 错误
#[derive(Debug)]
pub enum JoinfError {
    /// HTTP 请求失败
    Network(String),
    /// JSON 解析失败
    Parse(String),
    /// 认证失败
    Auth(String),
    /// API 返回业务错误
    Api { code: i32, message: String },
    /// 会话未初始化
    SessionNotInitialized,
    /// Selenium 操作失败
    Selenium(String),
    /// Chrome 密码解密失败
    ChromeDecrypt(String),
}

impl std::fmt::Display for JoinfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "network error: {}", e),
            Self::Parse(e) => write!(f, "parse error: {}", e),
            Self::Auth(e) => write!(f, "auth error: {}", e),
            Self::Api { code, message } => write!(f, "api error {}: {}", code, message),
            Self::SessionNotInitialized => {
                write!(f, "session not initialized, call init_session first")
            }
            Self::Selenium(e) => write!(f, "selenium error: {}", e),
            Self::ChromeDecrypt(e) => write!(f, "chrome decrypt error: {}", e),
        }
    }
}

impl std::error::Error for JoinfError {}

impl From<reqwest::Error> for JoinfError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e.to_string())
    }
}

impl From<serde_json::Error> for JoinfError {
    fn from(e: serde_json::Error) -> Self {
        Self::Parse(e.to_string())
    }
}

type Result<T> = std::result::Result<T, JoinfError>;

// ============================================================
// 2. API 响应模型 (joinf.com JSON 结构)
// ============================================================

/// 通用 API 响应包装
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    #[serde(rename = "code")]
    code: i32,
    #[serde(rename = "msg")]
    msg: Option<String>,
    #[serde(rename = "data")]
    data: Option<T>,
}

/// 客户列表响应
#[derive(Debug, Deserialize)]
struct CustomerListData {
    #[serde(rename = "list")]
    list: Vec<JoinfCustomer>,
    #[serde(rename = "total")]
    total: i64,
}

/// 富通天下客户对象
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinfCustomer {
    #[serde(rename = "customerId")]
    pub customer_id: i64,
    #[serde(rename = "customerName", default)]
    pub customer_name: String,
    #[serde(rename = "companyName", default)]
    pub company_name: String,
    #[serde(rename = "country", default)]
    pub country: String,
    #[serde(rename = "email", default)]
    pub email: String,
    #[serde(rename = "phone", default)]
    pub phone: String,
    #[serde(rename = "whatsapp", default)]
    pub whatsapp: String,
    #[serde(rename = "grade", default)]
    pub grade: String,
    #[serde(rename = "source", default)]
    pub source: String,
    #[serde(rename = "status", default)]
    pub status: String,
    #[serde(rename = "ownerName", default)]
    pub owner_name: String,
    #[serde(rename = "tags", default)]
    pub tags: Vec<String>,
    #[serde(rename = "lastContactTime", default)]
    pub last_contact_time: Option<i64>,
    #[serde(rename = "createTime", default)]
    pub create_time: Option<i64>,
    #[serde(rename = "updateTime", default)]
    pub update_time: Option<i64>,
}

/// 客户日志响应
#[derive(Debug, Deserialize)]
struct CustomerLogListData {
    #[serde(rename = "list")]
    list: Vec<JoinfCustomerLog>,
    #[serde(rename = "total")]
    total: i64,
}

/// 富通天下客户日志
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinfCustomerLog {
    #[serde(rename = "logId")]
    pub log_id: i64,
    #[serde(rename = "customerId")]
    pub customer_id: i64,
    #[serde(rename = "type")]
    pub log_type: String,
    #[serde(rename = "content", default)]
    pub content: String,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    #[serde(rename = "operatorName", default)]
    pub operator_name: String,
    /// WhatsApp 消息信息 (JSON 字符串)
    #[serde(rename = "whatsappInfo", default)]
    pub whatsapp_info: Option<String>,
    /// 邮件信息 (JSON 字符串)
    #[serde(rename = "emailInfo", default)]
    pub email_info: Option<String>,
}

/// 邮件列表响应
#[derive(Debug, Deserialize)]
struct EmailListData {
    #[serde(rename = "list")]
    list: Vec<JoinfEmail>,
    #[serde(rename = "total")]
    total: i64,
}

/// 富通天下邮件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinfEmail {
    #[serde(rename = "id")]
    pub id: i64,
    #[serde(rename = "from", default)]
    pub from_addr: String,
    #[serde(rename = "to", default)]
    pub to_addr: String,
    #[serde(rename = "subject", default)]
    pub subject: String,
    #[serde(rename = "body", default)]
    pub body: String,
    #[serde(rename = "receivedTime", default)]
    pub received_time: Option<i64>,
    #[serde(rename = "isRead", default)]
    pub is_read: bool,
    #[serde(rename = "boxId", default)]
    pub box_id: i32,
    #[serde(rename = "attachments", default)]
    pub attachments: Vec<JoinfAttachment>,
}

/// 邮件附件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinfAttachment {
    #[serde(rename = "filename", default)]
    pub filename: String,
    #[serde(rename = "size", default)]
    pub size: i64,
    #[serde(rename = "url", default)]
    pub url: String,
}

/// WhatsApp 联系人信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinfWhatsappContact {
    #[serde(rename = "customerId")]
    pub customer_id: i64,
    #[serde(rename = "whatsappNumber", default)]
    pub whatsapp_number: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(rename = "lastMessageTime", default)]
    pub last_message_time: Option<i64>,
}

/// 线索列表响应
#[derive(Debug, Deserialize)]
struct ClueListData {
    #[serde(rename = "list")]
    list: Vec<JoinfClue>,
    #[serde(rename = "total")]
    total: i64,
}

/// 富通天下线索
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinfClue {
    #[serde(rename = "clueId")]
    pub clue_id: i64,
    #[serde(rename = "companyName", default)]
    pub company_name: String,
    #[serde(rename = "contactName", default)]
    pub contact_name: String,
    #[serde(rename = "email", default)]
    pub email: String,
    #[serde(rename = "phone", default)]
    pub phone: String,
    #[serde(rename = "source", default)]
    pub source: String,
    #[serde(rename = "status", default)]
    pub status: String,
    #[serde(rename = "createTime")]
    pub create_time: i64,
}

/// 商机列表响应
#[derive(Debug, Deserialize)]
struct BusinessListData {
    #[serde(rename = "list")]
    list: Vec<JoinfBusiness>,
    #[serde(rename = "total")]
    total: i64,
}

/// 富通天下商机
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinfBusiness {
    #[serde(rename = "businessId")]
    pub business_id: i64,
    #[serde(rename = "customerName", default)]
    pub customer_name: String,
    #[serde(rename = "businessName", default)]
    pub business_name: String,
    #[serde(rename = "amount", default)]
    pub amount: f64,
    #[serde(rename = "currency", default)]
    pub currency: String,
    #[serde(rename = "status", default)]
    pub status: String,
    #[serde(rename = "stageUpdateTime", default)]
    pub stage_update_time: Option<i64>,
}

// ============================================================
// 3. 已发现的 API 端点
// ============================================================

/// 已发现的 API 端点集合
#[derive(Debug, Clone, Default)]
pub struct DiscoveredApis {
    pub endpoints: HashMap<String, String>,
}

// ============================================================
// 4. JoinfExtractor 核心结构
// ============================================================

/// 富通天下 (joinf.com) 数据提取器
///
/// 使用 REST API 提取 CRM 数据。支持两种认证方式：
/// 1. Cookie 认证 (通过 Selenium 登录获取)
/// 2. API Token 直接认证
pub struct JoinfExtractor {
    /// 平台基础 URL
    base_url: String,
    /// 公司 ID
    company_id: i64,
    /// 用户 ID
    user_id: i64,
    /// HTTP 客户端
    http_client: Client,
    /// Selenium 会话 (可选)
    selenium: Arc<RwLock<Option<SeleniumSession>>>,
    /// 认证 Cookie 值
    auth_cookie: Arc<RwLock<Option<String>>>,
    /// 已发现的 API 端点
    discovered_apis: Arc<RwLock<DiscoveredApis>>,
}

impl JoinfExtractor {
    /// 创建新提取器
    pub fn new(base_url: &str, company_id: i64, user_id: i64) -> Self {
        let http_client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            company_id,
            user_id,
            http_client,
            selenium: Arc::new(RwLock::new(None)),
            auth_cookie: Arc::new(RwLock::new(None)),
            discovered_apis: Arc::new(RwLock::new(DiscoveredApis::default())),
        }
    }

    /// 创建提取器并直接设置 Cookie 认证
    pub fn with_cookie(base_url: &str, company_id: i64, user_id: i64, cookie: String) -> Self {
        let mut extractor = Self::new(base_url, company_id, user_id);
        extractor.auth_cookie = Arc::new(RwLock::new(Some(cookie)));
        extractor
    }

    /// 初始化 Selenium 会话并登录
    pub async fn init_session(
        &self,
        username: &str,
        password: &str,
    ) -> std::result::Result<(), JoinfError> {
        let login_url = format!("{}/login", self.base_url);

        // 尝试从 Chrome 解密密码
        let _decrypted_password = self
            .try_decrypt_chrome_password(username)
            .await
            .unwrap_or_else(|_| password.to_string());

        // 创建 Selenium 配置
        let config = SeleniumConfig {
            base_url: login_url.clone(),
            headless: true,
            timeout_secs: 30,
            extra_args: vec!["--no-sandbox".into()],
        };

        // 创建 Selenium 会话
        let mut session = SeleniumSession::new(config).map_err(|e| JoinfError::Selenium(e))?;

        // 执行登录
        session
            .login(&login_url, username, password)
            .await
            .map_err(|e| JoinfError::Selenium(e))?;

        // 提取 Cookie
        let cookies = session.extract_cookies().await;
        if let Some(session_cookie) = cookies
            .iter()
            .find(|c| c.name == "SESSION" || c.name == "JSESSIONID")
        {
            let mut auth = self.auth_cookie.write().await;
            *auth = Some(session_cookie.value.clone());
        }

        // 捕获网络日志发现 API
        let endpoints = session.capture_network_logs().await;
        let mut discovered = self.discovered_apis.write().await;
        for url in endpoints {
            if url.contains("/rapi/") {
                let path = url
                    .split("?")
                    .next()
                    .unwrap_or(&url)
                    .replace(&self.base_url, "");
                discovered.endpoints.insert(path, url);
            }
        }

        let mut sess = self.selenium.write().await;
        *sess = Some(session);

        Ok(())
    }

    /// 直接设置认证 Cookie (跳过 Selenium)
    pub async fn set_auth_cookie(&self, cookie: String) {
        let mut auth = self.auth_cookie.write().await;
        *auth = Some(cookie);
    }

    /// 尝试解密 Chrome 本地存储的密码
    async fn try_decrypt_chrome_password(
        &self,
        _username: &str,
    ) -> std::result::Result<String, JoinfError> {
        let decryptor =
            ChromeDecryptor::new().map_err(|e| JoinfError::ChromeDecrypt(e.to_string()))?;

        let entries: Vec<LoginEntry> = decryptor
            .decrypt_login_data(None)
            .map_err(|e| JoinfError::ChromeDecrypt(e.to_string()))?;

        // 查找匹配 joinf.com 的条目
        for entry in &entries {
            if entry.origin_url.contains("joinf.com") {
                return Ok(entry.password.clone());
            }
        }

        Err(JoinfError::ChromeDecrypt(
            "no matching Chrome credential for joinf.com".into(),
        ))
    }

    /// 关闭 Selenium 会话
    pub async fn close_session(&self) -> std::result::Result<(), JoinfError> {
        let sess = self.selenium.read().await;
        if let Some(ref session) = *sess {
            session.close().await.map_err(|e| JoinfError::Selenium(e))?;
        }
        let mut sess = self.selenium.write().await;
        *sess = None;
        let mut auth = self.auth_cookie.write().await;
        *auth = None;
        Ok(())
    }

    // ============================================================
    // API 发现
    // ============================================================

    /// 从网络日志中发现 API 端点
    pub async fn discover_apis(&self) -> Vec<String> {
        let mut discovered = self.discovered_apis.write().await;

        let known_apis = vec![
            "/rapi/d/customers".to_string(),
            "/rapi/d/customer/logs".to_string(),
            "/rapi/g/whatsappCustomer/contact".to_string(),
            "/rapi/b/emails".to_string(),
            "/rapi/b/emails/content".to_string(),
            "/rapi/c/business/clues/list".to_string(),
            "/rapi/c/businesses".to_string(),
        ];

        for api in &known_apis {
            discovered
                .endpoints
                .insert(api.clone(), format!("{}{}", self.base_url, api));
        }

        known_apis
    }

    /// 获取认证 Cookie 值
    async fn get_cookie(&self) -> Result<String> {
        let auth = self.auth_cookie.read().await;
        auth.clone().ok_or(JoinfError::SessionNotInitialized)
    }

    /// 构建带认证的 API 请求
    fn build_request(&self, method: &str, url: &str, cookie: &str) -> reqwest::RequestBuilder {
        let mut builder = match method {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            _ => self.http_client.get(url),
        };

        builder = builder
            .header("Cookie", cookie)
            .header("Accept", "application/json")
            .header("X-Requested-With", "XMLHttpRequest");

        if self.company_id > 0 {
            builder = builder.header("X-Company-Id", self.company_id.to_string());
        }

        builder
    }

    /// 调用富通天下 API 并解析响应
    async fn call_api<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> std::result::Result<T, JoinfError> {
        let cookie = self.get_cookie().await?;
        let resp = self.build_request("GET", url, &cookie).send().await?;

        if !resp.status().is_success() {
            return Err(JoinfError::Network(format!("HTTP {}", resp.status())));
        }

        let api_resp: ApiResponse<T> = resp.json().await?;

        if api_resp.code != 0 && api_resp.code != 200 {
            return Err(JoinfError::Api {
                code: api_resp.code,
                message: api_resp.msg.unwrap_or_default(),
            });
        }

        api_resp
            .data
            .ok_or_else(|| JoinfError::Parse("no data in response".into()))
    }

    // ============================================================
    // 内部转换方法
    // ============================================================

    /// 解析客户等级字符串
    fn parse_grade(s: &str) -> CustomerGrade {
        match s.to_uppercase().as_str() {
            "A" | "A类" | "核心" => CustomerGrade::A,
            "B" | "B类" | "重要" => CustomerGrade::B,
            "C" | "C类" | "一般" => CustomerGrade::C,
            _ => CustomerGrade::D,
        }
    }

    /// 将 JoinfCustomer 转换为 CustomerProfile
    fn map_customer(&self, c: &JoinfCustomer) -> CustomerProfile {
        let grade = Self::parse_grade(&c.grade);
        let source = CustomerSource::OtherPlatform(c.source.clone());
        let status = match c.status.as_str() {
            "lead" => CustomerStatus::Lead,
            "contacted" => CustomerStatus::Contacted,
            "interested" => CustomerStatus::Interested,
            "inquiring" => CustomerStatus::Inquiring,
            "quoted" => CustomerStatus::Quoted,
            "won" => CustomerStatus::Won,
            "dormant" => CustomerStatus::Dormant,
            "lost" => CustomerStatus::Lost,
            _ => CustomerStatus::Lead,
        };

        let company = Company {
            id: c.customer_id.to_string(),
            name: c.company_name.clone(),
            name_en: None,
            country: c.country.clone(),
            ..Default::default()
        };

        let contact = CrmContact {
            id: format!("{}_contact", c.customer_id),
            company_id: c.customer_id.to_string(),
            name: c.customer_name.clone(),
            emails: if c.email.is_empty() {
                Vec::new()
            } else {
                vec![c.email.clone()]
            },
            phones: if c.phone.is_empty() {
                Vec::new()
            } else {
                vec![c.phone.clone()]
            },
            whatsapp: if c.whatsapp.is_empty() {
                None
            } else {
                Some(c.whatsapp.clone())
            },
            ..Default::default()
        };

        CustomerProfile {
            id: c.customer_id.to_string(),
            company,
            contacts: vec![contact],
            grade,
            source,
            status,
            owner_id: c.owner_name.clone(),
            total_revenue: 0.0,
            order_count: 0,
            last_contact_at: c.last_contact_time.map(|t| t as u64),
            last_order_at: None,
            avg_payment_days: None,
            credit_limit: None,
            preferred_terms: None,
            preferred_payment: None,
            purchased_categories: Vec::new(),
            interactions: Vec::new(),
            notes: Vec::new(),
            tags: c.tags.clone(),
            created_at: c.create_time.unwrap_or(0) as u64,
            updated_at: c.update_time.unwrap_or(0) as u64,
        }
    }

    /// 将 ActivityLog 转换为 InteractionRecord
    fn map_interaction(&self, log: &ActivityLog) -> InteractionRecord {
        let interaction_type = match log.interaction_type.as_str() {
            "email" => InteractionType::Email,
            "whatsapp" => InteractionType::WhatsApp,
            "phone" => InteractionType::Phone,
            "meeting" => InteractionType::Meeting,
            "inquiry" => InteractionType::Inquiry,
            "quotation" => InteractionType::Quotation,
            "contract" => InteractionType::Contract,
            _ => InteractionType::System,
        };

        InteractionRecord {
            id: log.id.to_string(),
            interaction_type,
            direction: "outbound".to_string(),
            summary: log.summary.clone(),
            content: Some(log.detail.clone()),
            contact_id: None,
            related_id: None,
            operator_id: log.operator.clone().unwrap_or_default(),
            timestamp: log.timestamp as u64,
            attachments: Vec::new(),
        }
    }

    /// 将 JoinfEmail 转换为 EmailRecord
    fn map_email(&self, e: &JoinfEmail) -> EmailRecord {
        let status = if e.is_read {
            EmailStatus::Opened
        } else {
            EmailStatus::Delivered
        };

        EmailRecord {
            id: e.id.to_string(),
            customer_id: None,
            contact_id: None,
            to: if e.to_addr.is_empty() {
                Vec::new()
            } else {
                e.to_addr.split(',').map(|s| s.trim().to_string()).collect()
            },
            cc: Vec::new(),
            bcc: Vec::new(),
            from: e.from_addr.clone(),
            subject: e.subject.clone(),
            body_html: e.body.clone(),
            body_text: String::new(),
            template_id: None,
            status,
            tracking: EmailTracking::default(),
            attachments: e
                .attachments
                .iter()
                .map(
                    |a| crate::l1_action::nt_act::nt_act_trade::nt_trade_email::EmailAttachment {
                        filename: a.filename.clone(),
                        content_type: "application/octet-stream".to_string(),
                        size_bytes: a.size as u64,
                        path: a.url.clone(),
                    },
                )
                .collect(),
            sent_at: e.received_time.map(|t| t as u64),
            created_at: e.received_time.unwrap_or(0) as u64,
        }
    }

    // ============================================================
    // 公开 API 方法
    // ============================================================

    /// 提取客户活动日志 (含 WhatsApp)
    pub async fn extract_customer_logs(
        &self,
        customer_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<ActivityLog>> {
        let url = format!(
            "{}/rapi/d/customer/logs?customerId={}&startDate={}&endDate={}",
            self.base_url, customer_id, start_date, end_date
        );

        let data: CustomerLogListData = self.call_api(&url).await?;

        let logs = data
            .list
            .iter()
            .map(|log| ActivityLog {
                id: log.log_id,
                customer_id: log.customer_id,
                interaction_type: log.log_type.clone(),
                summary: log.content.clone(),
                detail: log.content.clone(),
                timestamp: log.create_time,
                operator: if log.operator_name.is_empty() {
                    None
                } else {
                    Some(log.operator_name.clone())
                },
            })
            .collect();

        Ok(logs)
    }

    /// 获取 WhatsApp 联系人
    pub async fn extract_whatsapp_contacts(
        &self,
        customer_id: i64,
    ) -> Result<Vec<JoinfWhatsappContact>> {
        let url = format!(
            "{}/rapi/g/whatsappCustomer/contact?customerId={}",
            self.base_url, customer_id,
        );

        #[derive(Deserialize)]
        struct WaData {
            #[serde(rename = "list", default)]
            list: Vec<JoinfWhatsappContact>,
        }

        let data: WaData = self.call_api(&url).await?;
        Ok(data.list)
    }

    /// 提取线索列表
    pub async fn extract_clues(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<ExtractionResult<JoinfClue>> {
        let url = format!(
            "{}/rapi/c/business/clues/list?num={}&paging=true&size={}",
            self.base_url, page, page_size,
        );

        let data: ClueListData = self.call_api(&url).await?;
        let total = data.total as u32;

        Ok(ExtractionResult {
            data: data.list,
            total,
            has_more: (page * page_size) < total,
        })
    }

    /// 提取商机列表
    pub async fn extract_businesses(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<ExtractionResult<JoinfBusiness>> {
        let url = format!(
            "{}/rapi/c/businesses?share=1&statusUpdateTimeType=0&currency=USD&num={}&paging=true&size={}",
            self.base_url, page, page_size,
        );

        let data: BusinessListData = self.call_api(&url).await?;
        let total = data.total as u32;

        Ok(ExtractionResult {
            data: data.list,
            total,
            has_more: (page * page_size) < total,
        })
    }

    /// 获取邮件正文
    pub async fn get_email_content(&self, email_id: i64) -> Result<String> {
        let url = format!(
            "{}/rapi/b/emails/content?id={}&htmlOrTxt=0",
            self.base_url, email_id,
        );

        #[derive(Deserialize)]
        struct EmailContent {
            #[serde(rename = "content", default)]
            content: String,
        }

        let data: EmailContent = self.call_api(&url).await?;
        Ok(data.content)
    }
}

// ============================================================
// 5. ExternalPlatformExtractor 实现
// ============================================================

#[async_trait]
impl ExternalPlatformExtractor for JoinfExtractor {
    fn platform_id(&self) -> &str {
        "joinf"
    }

    fn platform_name(&self) -> &str {
        "富通天下"
    }

    async fn extract_customers(
        &self,
        config: ExtractConfig,
        progress: Option<&ProgressCallback>,
    ) -> std::result::Result<
        ExtractionResult<CustomerProfile>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let base_url = format!("{}/rapi/d/customers", self.base_url);
        let mut all_customers = Vec::new();
        let mut page = 1;
        let mut total: u32 = 0;

        loop {
            let url = format!(
                "{}?num={}&paging=true&size={}&shareStatus=0",
                base_url, page, config.page_size,
            );

            let data: CustomerListData = self.call_api(&url).await?;
            total = data.total as u32;

            let customers: Vec<CustomerProfile> =
                data.list.iter().map(|c| self.map_customer(c)).collect();
            all_customers.extend(customers);

            if let Some(cb) = progress {
                cb(all_customers.len() as u32, total);
            }

            let count = data.list.len() as u32;
            if count < config.page_size {
                break;
            }

            page += 1;
            if config.max_pages > 0 && page > config.max_pages {
                break;
            }
        }

        let has_more = (all_customers.len() as u32) < _total;

        Ok(ExtractionResult {
            data: all_customers,
            total: _total,
            has_more,
        })
    }

    async fn extract_interactions(
        &self,
        customer_id: &str,
        since: Option<DateTime<Utc>>,
    ) -> std::result::Result<Vec<InteractionRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let cid: i64 = customer_id
            .parse()
            .map_err(|e: std::num::ParseIntError| JoinfError::Parse(e.to_string()))?;

        let start_date = since
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "2000-01-01".to_string());

        let end_date = Utc::now().format("%Y-%m-%d").to_string();

        let logs = self
            .extract_customer_logs(cid, &start_date, &end_date)
            .await?;

        let interactions = logs.iter().map(|log| self.map_interaction(log)).collect();

        Ok(interactions)
    }

    async fn extract_emails(
        &self,
        config: EmailConfig,
        progress: Option<&ProgressCallback>,
    ) -> std::result::Result<ExtractionResult<EmailRecord>, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut all_emails = Vec::new();
        let mut page = 1;
        let mut total: u32 = 0;

        loop {
            let url = format!(
                "{}/rapi/b/emails?num={}&paging=true&size={}&boxId={}",
                self.base_url, page, config.page_size, config.box_id,
            );

            let data: EmailListData = self.call_api(&url).await?;
            total = data.total as u32;

            let emails: Vec<EmailRecord> = data.list.iter().map(|e| self.map_email(e)).collect();
            all_emails.extend(emails);

            if let Some(cb) = progress {
                cb(all_emails.len() as u32, total);
            }

            let count = data.list.len() as u32;
            if count < config.page_size {
                break;
            }

            page += 1;
            if config.max_pages > 0 && page > config.max_pages {
                break;
            }
        }

        let has_more = (all_emails.len() as u32) < _total;

        Ok(ExtractionResult {
            data: all_emails,
            total: _total,
            has_more,
        })
    }

    async fn sync_incremental(
        &self,
        last_sync: DateTime<Utc>,
        progress: Option<&ProgressCallback>,
    ) -> std::result::Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync_from = last_sync;
        let sync_to = Utc::now();
        let mut errors = Vec::new();

        // 1. 增量同步客户
        let customer_result = self
            .extract_customers(ExtractConfig::default(), progress)
            .await;
        let (new_customers, updated_customers) = match customer_result {
            Ok(result) => {
                let mut new = 0u32;
                let mut updated = 0u32;
                for c in &result.data {
                    if c.company.created_at > sync_from.timestamp() as u64 {
                        new += 1;
                    } else {
                        updated += 1;
                    }
                }
                (new, updated)
            }
            Err(e) => {
                errors.push(format!("customer sync failed: {}", e));
                (0, 0)
            }
        };

        // 2. 增量同步邮件
        let email_result = self.extract_emails(EmailConfig::default(), progress).await;
        let new_emails = match email_result {
            Ok(result) => result
                .data
                .iter()
                .filter(|e| {
                    e.created_at > sync_from.timestamp() as u64
                        && e.created_at <= sync_to.timestamp() as u64
                })
                .count() as u32,
            Err(e) => {
                errors.push(format!("email sync failed: {}", e));
                0
            }
        };

        Ok(SyncResult {
            new_customers,
            updated_customers,
            new_interactions: 0,
            new_emails,
            synced_from: sync_from,
            synced_to: sync_to,
            errors,
        })
    }
}

// ============================================================
// 6. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joinf_extractor_compiles() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 12345, 67890);
        assert_eq!(extractor.platform_id(), "joinf");
        assert_eq!(extractor.platform_name(), "富通天下");
    }

    #[test]
    fn test_joinf_extractor_with_cookie() {
        let extractor =
            JoinfExtractor::with_cookie("https://www.joinf.com", 12345, 67890, "test=abc".into());
        assert_eq!(extractor.platform_id(), "joinf");
    }

    #[test]
    fn test_map_customer() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let joinf_customer = JoinfCustomer {
            customer_id: 42,
            customer_name: "John Doe".into(),
            company_name: "Acme Corp".into(),
            country: "USA".into(),
            email: "john@acme.com".into(),
            phone: "+1234567890".into(),
            whatsapp: "+1234567890".into(),
            grade: "A".into(),
            source: "Exhibition".into(),
            status: "won".into(),
            owner_name: "Sales Rep".into(),
            tags: vec!["vip".into()],
            last_contact_time: Some(1700000000),
            create_time: Some(1600000000),
            update_time: Some(1700000000),
        };

        let profile = extractor.map_customer(&joinf_customer);
        assert_eq!(profile.id, "42");
        assert_eq!(profile.company.name, "Acme Corp");
        assert_eq!(profile.grade, CustomerGrade::A);
        assert_eq!(profile.contacts.len(), 1);
        assert_eq!(profile.contacts[0].name, "John Doe");
    }

    #[test]
    fn test_extract_config_default() {
        let config = ExtractConfig::default();
        assert_eq!(config.page_size, 100);
        assert_eq!(config.max_pages, 0);
        assert!(config.customer_ids.is_empty());
    }

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert_eq!(config.page_size, 50);
        assert_eq!(config.box_id, -1);
        assert!(!config.unread_only);
    }

    #[test]
    fn test_activity_log_mapping() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let log = JoinfCustomerLog {
            log_id: 100,
            customer_id: 42,
            log_type: "whatsapp".into(),
            content: "Sent product catalog".into(),
            create_time: 1700000000,
            operator_name: "Alice".into(),
            whatsapp_info: Some("{}".into()),
            email_info: None,
        };

        let interaction = extractor.map_interaction(&log);
        assert_eq!(interaction.id, "100");
        assert_eq!(interaction.operator_id, "Alice");
        assert_eq!(interaction.summary, "Sent product catalog");
    }

    #[test]
    fn test_email_mapping() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let email = JoinfEmail {
            id: 200,
            from_addr: "sender@example.com".into(),
            to_addr: "a@test.com,b@test.com".into(),
            subject: "Re: Quote".into(),
            body: "<p>Thanks for the quote</p>".into(),
            received_time: Some(1700000000),
            is_read: true,
            box_id: 1,
            attachments: vec![JoinfAttachment {
                filename: "quotation.pdf".into(),
                size: 1024,
                url: "/files/quotation.pdf".into(),
            }],
        };

        let record = extractor.map_email(&email);
        assert_eq!(record.id, "200");
        assert_eq!(record.to.len(), 2);
        assert_eq!(record.attachments.len(), 1);
        assert!(matches!(record.status, EmailStatus::Opened));
    }
}
