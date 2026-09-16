//! Trade CRM — 外贸客户关系管理模块
//!
//! 对标富通天下等 TMS 平台的客户管理能力：
//! - 公司/联系人/客户三级层次结构
//! - 客户分级 (A/B/C/D) + 来源追踪
//! - 交互历史 + 跟进记录
//! - 客户标签 + 自定义字段
//! - 客户查重 + 合并

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 客户等级与来源
// ============================================================

/// 客户等级 — 基于成交金额/频次/潜力综合评分
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CustomerGrade {
    /// A级: 核心大客户 (年成交 >$100K 或战略客户)
    A,
    /// B级: 重要客户 (年成交 $20K-$100K)
    B,
    /// C级: 一般客户 (年成交 <$20K)
    C,
    /// D级: 潜在/休眠客户
    D,
}

impl std::fmt::Display for CustomerGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A-核心大客户"),
            Self::B => write!(f, "B-重要客户"),
            Self::C => write!(f, "C-一般客户"),
            Self::D => write!(f, "D-潜在/休眠"),
        }
    }
}

/// 客户来源渠道
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CustomerSource {
    /// 阿里巴巴国际站
    Alibaba,
    /// 独立站
    Website,
    /// 展会
    Exhibition,
    /// Google 搜索/广告
    Google,
    /// LinkedIn
    LinkedIn,
    /// 海关数据
    CustomsData,
    /// 朋友推荐
    Referral,
    /// 主动开发 (Cold Call/Email)
    ColdOutreach,
    /// 其他平台 (Made-in-China, GlobalSources 等)
    OtherPlatform(String),
    /// 线下
    Offline,
}

impl std::fmt::Display for CustomerSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alibaba => write!(f, "阿里巴巴"),
            Self::Website => write!(f, "独立站"),
            Self::Exhibition => write!(f, "展会"),
            Self::Google => write!(f, "Google"),
            Self::LinkedIn => write!(f, "LinkedIn"),
            Self::CustomsData => write!(f, "海关数据"),
            Self::Referral => write!(f, "推荐"),
            Self::ColdOutreach => write!(f, "主动开发"),
            Self::OtherPlatform(p) => write!(f, "其他({})", p),
            Self::Offline => write!(f, "线下"),
        }
    }
}

/// 客户状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CustomerStatus {
    /// 新线索
    Lead,
    /// 已联系
    Contacted,
    /// 有意向
    Interested,
    /// 询盘中
    Inquiring,
    /// 已报价
    Quoted,
    /// 成交
    Won,
    /// 休眠
    Dormant,
    /// 流失
    Lost,
}

// ============================================================
// 2. 公司信息
// ============================================================

/// 公司信息 — 外贸公司/买家公司
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    /// 公司 ID
    pub id: String,
    /// 公司名称
    pub name: String,
    /// 公司英文名
    pub name_en: Option<String>,
    /// 国家/地区
    pub country: String,
    /// 城市
    pub city: Option<String>,
    /// 地址
    pub address: Option<String>,
    /// 官网
    pub website: Option<String>,
    /// 行业
    pub industry: Option<String>,
    /// 公司规模 (如 1-50, 51-200, 201-500, 500+)
    pub company_size: Option<String>,
    /// 年营收范围
    pub annual_revenue: Option<String>,
    /// 统一社会信用代码 / VAT / Tax ID
    pub tax_id: Option<String>,
    /// 公司备注
    pub notes: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 自定义字段
    pub custom_fields: HashMap<String, String>,
    /// 创建时间 (epoch seconds)
    pub created_at: u64,
    /// 更新时间 (epoch seconds)
    pub updated_at: u64,
}

impl Default for Company {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            name_en: None,
            country: String::new(),
            city: None,
            address: None,
            website: None,
            industry: None,
            company_size: None,
            annual_revenue: None,
            tax_id: None,
            notes: None,
            tags: Vec::new(),
            custom_fields: HashMap::new(),
            created_at: 0,
            updated_at: 0,
        }
    }
}

// ============================================================
// 3. 联系人
// ============================================================

/// 联系人 — 挂在公司下的具体联系人
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// 联系人 ID
    pub id: String,
    /// 关联公司 ID
    pub company_id: String,
    /// 姓名
    pub name: String,
    /// 职位
    pub title: Option<String>,
    /// 部门
    pub department: Option<String>,
    /// 邮箱列表
    pub emails: Vec<String>,
    /// 手机/电话
    pub phones: Vec<String>,
    /// WhatsApp
    pub whatsapp: Option<String>,
    /// Skype
    pub skype: Option<String>,
    /// LinkedIn
    pub linkedin: Option<String>,
    /// 微信
    pub wechat: Option<String>,
    /// 是否主要联系人
    pub is_primary: bool,
    /// 语言偏好
    pub language: Option<String>,
    /// 时区
    pub timezone: Option<String>,
    /// 备注
    pub notes: Option<String>,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

impl Default for Contact {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            company_id: String::new(),
            name: String::new(),
            title: None,
            department: None,
            emails: Vec::new(),
            phones: Vec::new(),
            whatsapp: None,
            skype: None,
            linkedin: None,
            wechat: None,
            is_primary: false,
            language: None,
            timezone: None,
            notes: None,
            created_at: 0,
            updated_at: 0,
        }
    }
}

// ============================================================
// 4. 客户档案 (CRM 核心)
// ============================================================

/// 客户档案 — 整合公司+联系人+业务数据的统一视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProfile {
    /// 客户 ID
    pub id: String,
    /// 关联公司
    pub company: Company,
    /// 联系人列表
    pub contacts: Vec<Contact>,
    /// 客户等级
    pub grade: CustomerGrade,
    /// 客户来源
    pub source: CustomerSource,
    /// 客户状态
    pub status: CustomerStatus,
    /// 负责业务员 ID
    pub owner_id: String,
    /// 累计成交金额 (USD)
    pub total_revenue: f64,
    /// 成交订单数
    pub order_count: u32,
    /// 最后联系时间
    pub last_contact_at: Option<u64>,
    /// 最后下单时间
    pub last_order_at: Option<u64>,
    /// 平均付款周期 (天)
    pub avg_payment_days: Option<u32>,
    /// 信用额度 (USD)
    pub credit_limit: Option<f64>,
    /// 偏好贸易条款
    pub preferred_terms: Option<String>,
    /// 偏好付款方式
    pub preferred_payment: Option<String>,
    /// 常采购产品类别
    pub purchased_categories: Vec<String>,
    /// 交互记录
    pub interactions: Vec<InteractionRecord>,
    /// 备注
    pub notes: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

impl Default for CustomerProfile {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            company: Company::default(),
            contacts: Vec::new(),
            grade: CustomerGrade::D,
            source: CustomerSource::Offline,
            status: CustomerStatus::Lead,
            owner_id: String::new(),
            total_revenue: 0.0,
            order_count: 0,
            last_contact_at: None,
            last_order_at: None,
            avg_payment_days: None,
            credit_limit: None,
            preferred_terms: None,
            preferred_payment: None,
            purchased_categories: Vec::new(),
            interactions: Vec::new(),
            notes: Vec::new(),
            tags: Vec::new(),
            created_at: 0,
            updated_at: 0,
        }
    }
}

// ============================================================
// 5. 交互记录
// ============================================================

/// 交互类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionType {
    /// 邮件
    Email,
    /// 电话
    Phone,
    /// WhatsApp 消息
    WhatsApp,
    /// 展会面谈
    Meeting,
    /// 视频会议
    VideoCall,
    /// 询盘
    Inquiry,
    /// 报价
    Quotation,
    /// 合同签署
    Contract,
    /// 系统自动记录
    System,
}

/// 交互记录 — 每次与客户的接触
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    /// 记录 ID
    pub id: String,
    /// 交互类型
    pub interaction_type: InteractionType,
    /// 方向 (inbound/outbound)
    pub direction: String,
    /// 摘要
    pub summary: String,
    /// 详细内容
    pub content: Option<String>,
    /// 关联联系人 ID
    pub contact_id: Option<String>,
    /// 关联订单/询盘 ID
    pub related_id: Option<String>,
    /// 执行人 ID
    pub operator_id: String,
    /// 时间戳
    pub timestamp: u64,
    /// 附件列表
    pub attachments: Vec<String>,
}

// ============================================================
// 6. CRM 查询/过滤
// ============================================================

/// 客户查询过滤器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerFilters {
    /// 客户名称关键词
    pub name_keyword: Option<String>,
    /// 国家/地区
    pub country: Option<String>,
    /// 客户等级
    pub grade: Option<CustomerGrade>,
    /// 客户状态
    pub status: Option<CustomerStatus>,
    /// 客户来源
    pub source: Option<CustomerSource>,
    /// 负责业务员 ID
    pub owner_id: Option<String>,
    /// 标签 (OR)
    pub tags: Vec<String>,
    /// 最小成交金额
    pub min_revenue: Option<f64>,
    /// 最后联系时间 (之前 N 天未联系)
    pub inactive_days: Option<u32>,
    /// 分页
    pub page_size: Option<u32>,
    pub page: Option<u32>,
}

/// 客户查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerQueryResult {
    pub items: Vec<CustomerProfile>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

// ============================================================
// 7. CRM 引擎
// ============================================================

/// CRM 引擎 — 外贸客户关系管理核心
pub struct TradeCrmEngine {
    /// 客户数据 (in-memory, 生产环境应接数据库)
    customers: HashMap<String, CustomerProfile>,
    /// 公司索引
    company_index: HashMap<String, Vec<String>>, // company_id → customer_ids
    /// 联系人索引
    contact_index: HashMap<String, String>, // contact_id → customer_id
}

impl Default for TradeCrmEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeCrmEngine {
    pub fn new() -> Self {
        Self {
            customers: HashMap::new(),
            company_index: HashMap::new(),
            contact_index: HashMap::new(),
        }
    }

    /// 创建客户档案
    pub fn create_customer(
        &mut self,
        company: Company,
        source: CustomerSource,
        owner_id: &str,
    ) -> CustomerProfile {
        let now = self.current_timestamp();
        let customer_id = company.id.clone();
        let profile = CustomerProfile {
            id: customer_id.clone(),
            company,
            contacts: Vec::new(),
            grade: CustomerGrade::D,
            source,
            status: CustomerStatus::Lead,
            owner_id: owner_id.to_string(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        };
        self.customers.insert(customer_id.clone(), profile.clone());
        profile
    }

    /// 添加联系人到客户
    pub fn add_contact(
        &mut self,
        customer_id: &str,
        contact: Contact,
    ) -> Result<Contact, String> {
        let now = self.current_timestamp();
        let contact_id = contact.id.clone();
        let company_id = self
            .customers
            .get(customer_id)
            .ok_or_else(|| format!("Customer {} not found", customer_id))?
            .company
            .id
            .clone();
        let customer = self
            .customers
            .get_mut(customer_id)
            .ok_or_else(|| format!("Customer {} not found", customer_id))?;
        customer.contacts.push(contact.clone());
        customer.updated_at = now;
        self.contact_index.insert(contact_id.clone(), customer_id.to_string());
        self.company_index
            .entry(company_id)
            .or_default()
            .push(customer_id.to_string());
        Ok(contact)
    }

    /// 记录交互
    pub fn record_interaction(
        &mut self,
        customer_id: &str,
        interaction: InteractionRecord,
    ) -> Result<(), String> {
        let now = self.current_timestamp();
        let customer = self
            .customers
            .get_mut(customer_id)
            .ok_or_else(|| format!("Customer {} not found", customer_id))?;
        customer.last_contact_at = Some(interaction.timestamp);
        customer.interactions.push(interaction);
        customer.updated_at = now;
        Ok(())
    }

    /// 更新客户等级 — 基于成交金额自动分级
    pub fn auto_grade(&mut self, customer_id: &str) -> Result<CustomerGrade, String> {
        let now = self.current_timestamp();
        let customer = self
            .customers
            .get_mut(customer_id)
            .ok_or_else(|| format!("Customer {} not found", customer_id))?;
        let grade = if customer.total_revenue >= 100_000.0 {
            CustomerGrade::A
        } else if customer.total_revenue >= 20_000.0 {
            CustomerGrade::B
        } else if customer.total_revenue > 0.0 {
            CustomerGrade::C
        } else {
            CustomerGrade::D
        };
        customer.grade = grade;
        customer.updated_at = now;
        Ok(grade)
    }

    /// 记录成交
    pub fn record_order(
        &mut self,
        customer_id: &str,
        amount_usd: f64,
    ) -> Result<(), String> {
        let now = self.current_timestamp();
        let customer = self
            .customers
            .get_mut(customer_id)
            .ok_or_else(|| format!("Customer {} not found", customer_id))?;
        customer.total_revenue += amount_usd;
        customer.order_count += 1;
        customer.last_order_at = Some(now);
        customer.status = CustomerStatus::Won;
        customer.updated_at = now;
        Ok(())
    }

    /// 查重 — 按公司名+国家查重
    pub fn find_duplicates(&self, company_name: &str, country: &str) -> Vec<&CustomerProfile> {
        self.customers
            .values()
            .filter(|c| {
                c.company.name.eq_ignore_ascii_case(company_name)
                    && c.company.country.eq_ignore_ascii_case(country)
            })
            .collect()
    }

    /// 查询客户
    pub fn query_customers(&self, filters: &CustomerFilters) -> CustomerQueryResult {
        let mut items: Vec<&CustomerProfile> = self
            .customers
            .values()
            .filter(|c| {
                if let Some(ref kw) = filters.name_keyword {
                    if !c.company.name.to_lowercase().contains(&kw.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(ref country) = filters.country {
                    if !c.company.country.eq_ignore_ascii_case(country) {
                        return false;
                    }
                }
                if let Some(grade) = filters.grade {
                    if c.grade != grade {
                        return false;
                    }
                }
                if let Some(ref status) = filters.status {
                    if c.status != *status {
                        return false;
                    }
                }
                if let Some(ref source) = filters.source {
                    if c.source != *source {
                        return false;
                    }
                }
                if let Some(ref owner) = filters.owner_id {
                    if c.owner_id != *owner {
                        return false;
                    }
                }
                if !filters.tags.is_empty() {
                    if !filters.tags.iter().any(|t| c.tags.contains(t)) {
                        return false;
                    }
                }
                if let Some(min_rev) = filters.min_revenue {
                    if c.total_revenue < min_rev {
                        return false;
                    }
                }
                true
            })
            .collect();
        let total = items.len() as u64;
        let page = filters.page.unwrap_or(0);
        let page_size = filters.page_size.unwrap_or(20);
        let start = (page * page_size) as usize;
        items.sort_by(|a, b| b.total_revenue.partial_cmp(&a.total_revenue).unwrap());
        let paged: Vec<CustomerProfile> = items
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .cloned()
            .collect();
        CustomerQueryResult {
            items: paged,
            total,
            page,
            page_size,
        }
    }

    /// 获取客户统计摘要
    pub fn summary(&self) -> CrmSummary {
        let total = self.customers.len() as u64;
        let grade_a = self
            .customers
            .values()
            .filter(|c| c.grade == CustomerGrade::A)
            .count() as u64;
        let grade_b = self
            .customers
            .values()
            .filter(|c| c.grade == CustomerGrade::B)
            .count() as u64;
        let total_revenue: f64 = self.customers.values().map(|c| c.total_revenue).sum();
        let total_orders: u64 = self
            .customers
            .values()
            .map(|c| c.order_count as u64)
            .sum();
        CrmSummary {
            total_customers: total,
            grade_a_count: grade_a,
            grade_b_count: grade_b,
            total_revenue_usd: total_revenue,
            total_orders,
            avg_revenue_per_customer: if total > 0 {
                total_revenue / total as f64
            } else {
                0.0
            },
        }
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// CRM 汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmSummary {
    pub total_customers: u64,
    pub grade_a_count: u64,
    pub grade_b_count: u64,
    pub total_revenue_usd: f64,
    pub total_orders: u64,
    pub avg_revenue_per_customer: f64,
}

// ============================================================
// 8. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_company() -> Company {
        Company {
            id: "COMP-001".into(),
            name: "Acme Corp".into(),
            name_en: Some("Acme Corporation".into()),
            country: "US".into(),
            city: Some("New York".into()),
            ..Default::default()
        }
    }

    #[test]
    fn test_create_customer() {
        let mut engine = TradeCrmEngine::new();
        let profile = engine.create_customer(
            make_test_company(),
            CustomerSource::Alibaba,
            "sales-01",
        );
        assert_eq!(profile.grade, CustomerGrade::D);
        assert_eq!(profile.status, CustomerStatus::Lead);
        assert_eq!(engine.customers.len(), 1);
    }

    #[test]
    fn test_add_contact() {
        let mut engine = TradeCrmEngine::new();
        let profile = engine.create_customer(make_test_company(), CustomerSource::Alibaba, "s1");
        let contact = Contact {
            company_id: profile.id.clone(),
            name: "John Doe".into(),
            emails: vec!["john@acme.com".into()],
            is_primary: true,
            ..Default::default()
        };
        let result = engine.add_contact(&profile.id, contact);
        assert!(result.is_ok());
        let customer = engine.customers.get(&profile.id).unwrap();
        assert_eq!(customer.contacts.len(), 1);
        assert_eq!(customer.contacts[0].name, "John Doe");
    }

    #[test]
    fn test_auto_grade() {
        let mut engine = TradeCrmEngine::new();
        let profile = engine.create_customer(make_test_company(), CustomerSource::Alibaba, "s1");
        engine.record_order(&profile.id, 150_000.0).unwrap();
        let grade = engine.auto_grade(&profile.id).unwrap();
        assert_eq!(grade, CustomerGrade::A);
    }

    #[test]
    fn test_find_duplicates() {
        let mut engine = TradeCrmEngine::new();
        engine.create_customer(make_test_company(), CustomerSource::Alibaba, "s1");
        let dupes = engine.find_duplicates("Acme Corp", "US");
        assert_eq!(dupes.len(), 1);
        let no_dupes = engine.find_duplicates("Other Corp", "US");
        assert!(no_dupes.is_empty());
    }

    #[test]
    fn test_query_customers() {
        let mut engine = TradeCrmEngine::new();
        engine.create_customer(make_test_company(), CustomerSource::Alibaba, "s1");
        let filters = CustomerFilters {
            country: Some("US".into()),
            ..Default::default()
        };
        let result = engine.query_customers(&filters);
        assert_eq!(result.total, 1);
    }

    #[test]
    fn test_summary() {
        let mut engine = TradeCrmEngine::new();
        engine.create_customer(make_test_company(), CustomerSource::Alibaba, "s1");
        engine.record_order("COMP-001", 50_000.0).unwrap();
        let summary = engine.summary();
        assert_eq!(summary.total_customers, 1);
        assert_eq!(summary.total_revenue_usd, 50_000.0);
    }
}
