//! Trade Unified Types — 统一类型系统 (Single Source of Truth)
//!
//! 合并 NeoTrix 通用类型 + WSD 外贸业务类型
//! 所有 trade 子模块共用的领域数据结构统一在此定义。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// 1. ID 策略 (从 WSD 迁移)
// ============================================================

/// 幂等 ID 生成：外部原始 ID + 公司 ID → 稳定 UUID
pub fn stable_id(raw_id: i64, company_id: i64) -> Uuid {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    raw_id.hash(&mut h);
    company_id.hash(&mut h);
    let hash = h.finish();
    let b = hash.to_be_bytes();
    Uuid::from_bytes([
        b[0], b[1], b[2], b[3], b[4], b[5],
        b[6] & 0x0f | 0x40, b[7] & 0x3f | 0x80,
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
    ])
}

// ============================================================
// 2. 客户等级 (从 WSD 迁移 + 增强)
// ============================================================

/// 客户等级 (E→VIP，含业务逻辑)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Grade { E, D, C, B, A, S, VIP }

impl Grade {
    /// 等级数值排名
    pub fn rank(&self) -> u8 {
        match self {
            Self::E => 0, Self::D => 1, Self::C => 2, Self::B => 3,
            Self::A => 4, Self::S => 5, Self::VIP => 6,
        }
    }
    
    /// 基于等级的跟进间隔 (天)
    pub fn follow_interval_days(&self) -> i64 {
        match self {
            Self::VIP => 2, Self::S => 3, Self::A => 5,
            Self::B => 7, Self::C => 14, Self::D => 21, Self::E => 30,
        }
    }
    
    /// 从字符串解析等级
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "E" | "E类" => Some(Self::E),
            "D" | "D类" => Some(Self::D),
            "C" | "C类" => Some(Self::C),
            "B" | "B类" => Some(Self::B),
            "A" | "A类" => Some(Self::A),
            "S" | "S类" | "S类重要" => Some(Self::S),
            "VIP" => Some(Self::VIP),
            _ => None,
        }
    }
}

impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.rank().cmp(&other.rank()))
    }
}

impl Ord for Grade {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { 
        self.rank().cmp(&other.rank()) 
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E => write!(f,"E"), Self::D => write!(f,"D"), Self::C => write!(f,"C"),
            Self::B => write!(f,"B"), Self::A => write!(f,"A"), Self::S => write!(f,"S"),
            Self::VIP => write!(f,"VIP"),
        }
    }
}

// ============================================================
// 3. 渠道 (从 WSD 迁移 + 增强)
// ============================================================

/// 客户来源渠道 (18变体 + 中文Display)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Channel {
    Alibaba, MadeInChina, GlobalSources,
    GoogleAds, FacebookAds, LinkedInAds,
    WhatsApp, Email, WeChat, LinkedIn,
    WebsiteForm, WebsiteChat, SEOOrganic,
    Exhibition, Referral, ColdCall,
    CustomsData, Other(String),
}

impl Channel {
    /// 从字符串解析渠道
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "阿里巴巴" | "Alibaba" => Some(Self::Alibaba),
            "中国制造网" | "MadeInChina" => Some(Self::MadeInChina),
            "环球资源" | "GlobalSources" => Some(Self::GlobalSources),
            "Google广告" | "GoogleAds" => Some(Self::GoogleAds),
            "Facebook广告" | "FacebookAds" => Some(Self::FacebookAds),
            "LinkedIn广告" | "LinkedInAds" => Some(Self::LinkedInAds),
            "WhatsApp" => Some(Self::WhatsApp),
            "邮件" | "Email" => Some(Self::Email),
            "微信" | "WeChat" => Some(Self::WeChat),
            "LinkedIn" => Some(Self::LinkedIn),
            "官网表单" | "WebsiteForm" => Some(Self::WebsiteForm),
            "在线聊天" | "WebsiteChat" => Some(Self::WebsiteChat),
            "SEO自然流量" | "SEOOrganic" => Some(Self::SEOOrganic),
            "展会" | "Exhibition" => Some(Self::Exhibition),
            "老客户转介绍" | "Referral" => Some(Self::Referral),
            "陌生拜访" | "ColdCall" => Some(Self::ColdCall),
            "海关数据" | "CustomsData" => Some(Self::CustomsData),
            _ => Some(Self::Other(s.to_string())),
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alibaba => write!(f,"阿里巴巴"), Self::MadeInChina => write!(f,"中国制造网"),
            Self::GlobalSources => write!(f,"环球资源"), Self::GoogleAds => write!(f,"Google广告"),
            Self::FacebookAds => write!(f,"Facebook广告"), Self::LinkedInAds => write!(f,"LinkedIn广告"),
            Self::WhatsApp => write!(f,"WhatsApp"), Self::Email => write!(f,"邮件"),
            Self::WeChat => write!(f,"微信"), Self::LinkedIn => write!(f,"LinkedIn"),
            Self::WebsiteForm => write!(f,"官网表单"), Self::WebsiteChat => write!(f,"在线聊天"),
            Self::SEOOrganic => write!(f,"SEO自然流量"), Self::Exhibition => write!(f,"展会"),
            Self::Referral => write!(f,"老客户转介绍"), Self::ColdCall => write!(f,"陌生拜访"),
            Self::CustomsData => write!(f,"海关数据"), Self::Other(s) => write!(f,"{}", s),
        }
    }
}

// ============================================================
// 4. 产品类别 (阀门行业，从 WSD 迁移)
// ============================================================

/// 产品类别 (阀门行业专用)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProductKind {
    GateValve, BallValve, ButterflyValve, GlobeValve, CheckValve,
    SafetyValve, RegulatingValve, PipeFitting, Flange, Other(String),
}

impl ProductKind {
    /// 从字符串解析产品类型
    pub fn from_str(s: &str) -> Option<Self> {
        let lower = s.to_lowercase();
        if lower.contains("gate") || lower.contains("闸阀") { return Some(Self::GateValve); }
        if lower.contains("ball") || lower.contains("球阀") { return Some(Self::BallValve); }
        if lower.contains("butterfly") || lower.contains("蝶阀") { return Some(Self::ButterflyValve); }
        if lower.contains("globe") || lower.contains("截止阀") { return Some(Self::GlobeValve); }
        if lower.contains("check") || lower.contains("止回阀") { return Some(Self::CheckValve); }
        if lower.contains("safety") || lower.contains("安全阀") { return Some(Self::SafetyValve); }
        if lower.contains("regulat") || lower.contains("调节阀") { return Some(Self::RegulatingValve); }
        if lower.contains("fitting") || lower.contains("管件") { return Some(Self::PipeFitting); }
        if lower.contains("flange") || lower.contains("法兰") { return Some(Self::Flange); }
        Some(Self::Other(s.to_string()))
    }
}

impl std::fmt::Display for ProductKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GateValve => write!(f,"闸阀"), Self::BallValve => write!(f,"球阀"),
            Self::ButterflyValve => write!(f,"蝶阀"), Self::GlobeValve => write!(f,"截止阀"),
            Self::CheckValve => write!(f,"止回阀"), Self::SafetyValve => write!(f,"安全阀"),
            Self::RegulatingValve => write!(f,"调节阀"), Self::PipeFitting => write!(f,"管件"),
            Self::Flange => write!(f,"法兰"), Self::Other(s) => write!(f,"{}", s),
        }
    }
}

// ============================================================
// 5. 货币 (从 WSD 迁移)
// ============================================================

/// 货币类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Currency { CNY, USD, EUR, GBP, JPY, Other(String) }

impl Currency {
    /// 从字符串解析货币
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "CNY" | "RMB" | "人民币" => Some(Self::CNY),
            "USD" | "$" | "美元" => Some(Self::USD),
            "EUR" | "€" | "欧元" => Some(Self::EUR),
            "GBP" | "£" | "英镑" => Some(Self::GBP),
            "JPY" | "¥" | "日元" => Some(Self::JPY),
            _ => Some(Self::Other(s.to_string())),
        }
    }
    
    /// 货币符号
    pub fn symbol(&self) -> &str {
        match self {
            Self::CNY => "¥", Self::USD => "$", Self::EUR => "€",
            Self::GBP => "£", Self::JPY => "¥", Self::Other(_) => "",
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CNY => write!(f,"CNY"), Self::USD => write!(f,"USD"),
            Self::EUR => write!(f,"EUR"), Self::GBP => write!(f,"GBP"),
            Self::JPY => write!(f,"JPY"), Self::Other(s) => write!(f,"{}", s),
        }
    }
}

// ============================================================
// 6. 询盘流程步骤 (从 WSD 迁移)
// ============================================================

/// 询盘流程 11 步
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FlowStep {
    Receive,        // 1. 接收
    Classify,       // 2. 分类
    Score,          // 3. 评分
    Assign,         // 4. 分配
    Confirm,        // 5. 确认
    Contact,        // 6. 联系
    Quote,          // 7. 报价
    Negotiate,      // 8. 谈判
    Convert,        // 9. 转化
    FollowUp,       // 10. 跟进
    Close,          // 11. 关闭
}

impl FlowStep {
    /// 下一步
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Receive => Some(Self::Classify),
            Self::Classify => Some(Self::Score),
            Self::Score => Some(Self::Assign),
            Self::Assign => Some(Self::Confirm),
            Self::Confirm => Some(Self::Contact),
            Self::Contact => Some(Self::Quote),
            Self::Quote => Some(Self::Negotiate),
            Self::Negotiate => Some(Self::Convert),
            Self::Convert => Some(Self::FollowUp),
            Self::FollowUp => Some(Self::Close),
            Self::Close => None,
        }
    }
}

impl std::fmt::Display for FlowStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receive => write!(f,"接收"), Self::Classify => write!(f,"分类"),
            Self::Score => write!(f,"评分"), Self::Assign => write!(f,"分配"),
            Self::Confirm => write!(f,"确认"), Self::Contact => write!(f,"联系"),
            Self::Quote => write!(f,"报价"), Self::Negotiate => write!(f,"谈判"),
            Self::Convert => write!(f,"转化"), Self::FollowUp => write!(f,"跟进"),
            Self::Close => write!(f,"关闭"),
        }
    }
}

// ============================================================
// 7. 订单节点 (从 WSD 迁移)
// ============================================================

/// 订单 12 节点状态机
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OrderNode {
    Inquiry,        // 1. 询盘
    Quote,          // 2. 报价
    PI,             // 3. 形式发票
    Contract,       // 4. 合同
    Purchase,       // 5. 采购
    Production,     // 6. 生产
    Warehouse,      // 7. 仓储
    Customs,        // 8. 报关
    ToArrivalCost,  // 9. 到岸成本
    ForexSettle,    // 10. 外汇结算
    TaxRefund,      // 11. 退税
    Complete,       // 12. 完成
}

impl OrderNode {
    /// 所有节点 (用于迭代)
    pub fn all() -> Vec<Self> {
        vec![
            Self::Inquiry, Self::Quote, Self::PI, Self::Contract,
            Self::Purchase, Self::Production, Self::Warehouse, Self::Customs,
            Self::ToArrivalCost, Self::ForexSettle, Self::TaxRefund, Self::Complete,
        ]
    }
    
    /// 下一步
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Inquiry => Some(Self::Quote),
            Self::Quote => Some(Self::PI),
            Self::PI => Some(Self::Contract),
            Self::Contract => Some(Self::Purchase),
            Self::Purchase => Some(Self::Production),
            Self::Production => Some(Self::Warehouse),
            Self::Warehouse => Some(Self::Customs),
            Self::Customs => Some(Self::ToArrivalCost),
            Self::ToArrivalCost => Some(Self::ForexSettle),
            Self::ForexSettle => Some(Self::TaxRefund),
            Self::TaxRefund => Some(Self::Complete),
            Self::Complete => None,
        }
    }
}

impl std::fmt::Display for OrderNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inquiry => write!(f,"询盘"), Self::Quote => write!(f,"报价"),
            Self::PI => write!(f,"形式发票"), Self::Contract => write!(f,"合同"),
            Self::Purchase => write!(f,"采购"), Self::Production => write!(f,"生产"),
            Self::Warehouse => write!(f,"仓储"), Self::Customs => write!(f,"报关"),
            Self::ToArrivalCost => write!(f,"到岸成本"), Self::ForexSettle => write!(f,"外汇结算"),
            Self::TaxRefund => write!(f,"退税"), Self::Complete => write!(f,"完成"),
        }
    }
}

// ============================================================
// 8. 通用枚举 (从 NeoTrix data_model 迁移)
// ============================================================

/// 产品分类 (通用)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProductCategory {
    #[default]
    Valve,
    Pump,
    Flange,
    PipeFitting,
    Instrument,
    Other,
}

/// 驱动类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DriveType {
    #[default]
    Manual,
    Electric,
    Pneumatic,
    Hydraulic,
    Electromagnetic,
    None,
}

/// 连接类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ConnectionType {
    #[default]
    Flanged,
    Welded,
    Threaded,
    Compression,
    QuickConnect,
    Wafer,
    Other,
}

/// 贸易条款 (Incoterms)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TradeTerms {
    #[default]
    ExWorks,
    Fca,
    Fob,
    Cfr,
    Cif,
    Ddp,
    Other(String),
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OrderStatus {
    #[default]
    Draft,
    Confirmed,
    InProduction,
    Shipped,
    Delivered,
    Completed,
    Cancelled,
}

/// 报价状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum QuoteStatus {
    #[default]
    Draft,
    Sent,
    Accepted,
    Rejected,
    Expired,
    Withdrawn,
}

/// 询价状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum InquiryStatus {
    #[default]
    Received,
    Analyzing,
    Quoting,
    Quoted,
    Won,
    Lost,
    Cancelled,
}

// ============================================================
// 9. 辅助结构 (从 NeoTrix data_model 迁移)
// ============================================================

/// 材质规格
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaterialSpec {
    pub name: String,
    pub standard: String,
    pub grade: String,
}

/// 压力等级
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PressureRating {
    pub value: u32,
    pub unit: String,
}

/// 尺寸规格
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeSpec {
    pub nominal: String,
    pub inner_diameter: Option<f64>,
    pub outer_diameter: Option<f64>,
    pub length: Option<f64>,
}

/// 价格信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceInfo {
    pub unit_price: f64,
    pub currency: String,
    pub discount_rate: Option<f64>,
    pub tax_included: bool,
}

/// 联系信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactInfo {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub title: String,
    pub wechat: Option<String>,
}

/// 供应商绩效指标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub on_time_delivery_rate: f64,
    pub quality_pass_rate: f64,
    pub response_time_hours: f64,
    pub cooperation_count: u32,
    pub overall_score: f64,
}

/// 询价元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InquiryMetadata {
    pub project_name: Option<String>,
    pub project_no: Option<String>,
    pub destination: Option<String>,
    pub delivery_requirement: Option<String>,
    pub notes: Option<String>,
}

// ============================================================
// 10. 核心数据结构 (从 NeoTrix data_model 迁移)
// ============================================================

/// 产品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub code: String,
    pub category: ProductCategory,
    pub subcategory: String,
    pub model: String,
    pub materials: Vec<MaterialSpec>,
    pub drive_type: DriveType,
    pub connection_type: ConnectionType,
    pub standard: String,
    pub pressure: PressureRating,
    pub size: SizeSpec,
    pub price: PriceInfo,
    pub supplier_id: String,
    pub grade: String,
}

impl Default for Product {
    fn default() -> Self {
        Self {
            code: String::new(),
            category: ProductCategory::default(),
            subcategory: String::new(),
            model: String::new(),
            materials: Vec::new(),
            drive_type: DriveType::default(),
            connection_type: ConnectionType::default(),
            standard: String::new(),
            pressure: PressureRating::default(),
            size: SizeSpec::default(),
            price: PriceInfo::default(),
            supplier_id: String::new(),
            grade: String::new(),
        }
    }
}

/// 供应商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: String,
    pub name: String,
    pub contact: ContactInfo,
    pub performance: PerformanceMetrics,
    pub products: Vec<String>,
    pub location: String,
}

/// 客户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub contact: ContactInfo,
    pub grade: Grade,
    pub channel: Channel,
    pub country: String,
    pub tags: Vec<String>,
    pub description: String,
}

/// 询价项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquiryItem {
    pub product: Product,
    pub quantity: u32,
    pub required_delivery: Option<String>,
    pub notes: Option<String>,
}

/// 询价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inquiry {
    pub id: String,
    pub customer: Customer,
    pub items: Vec<InquiryItem>,
    pub status: InquiryStatus,
    pub metadata: InquiryMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 报价项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteItem {
    pub product: Product,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_price: f64,
    pub discount_rate: Option<f64>,
}

/// 报价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: String,
    pub inquiry_id: String,
    pub customer: Customer,
    pub items: Vec<QuoteItem>,
    pub total_amount: f64,
    pub currency: Currency,
    pub trade_terms: TradeTerms,
    pub status: QuoteStatus,
    pub valid_until: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub quote_id: String,
    pub customer: Customer,
    pub items: Vec<QuoteItem>,
    pub total_amount: f64,
    pub currency: Currency,
    pub trade_terms: TradeTerms,
    pub status: OrderStatus,
    pub current_node: OrderNode,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================
// 11. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_ordering() {
        assert!(Grade::E < Grade::D);
        assert!(Grade::D < Grade::C);
        assert!(Grade::C < Grade::B);
        assert!(Grade::B < Grade::A);
        assert!(Grade::A < Grade::S);
        assert!(Grade::S < Grade::VIP);
    }

    #[test]
    fn test_grade_follow_interval() {
        assert_eq!(Grade::VIP.follow_interval_days(), 2);
        assert_eq!(Grade::S.follow_interval_days(), 3);
        assert_eq!(Grade::A.follow_interval_days(), 5);
        assert_eq!(Grade::B.follow_interval_days(), 7);
        assert_eq!(Grade::C.follow_interval_days(), 14);
        assert_eq!(Grade::D.follow_interval_days(), 21);
        assert_eq!(Grade::E.follow_interval_days(), 30);
    }

    #[test]
    fn test_flow_step_chain() {
        let steps = vec![
            FlowStep::Receive, FlowStep::Classify, FlowStep::Score,
            FlowStep::Assign, FlowStep::Confirm, FlowStep::Contact,
            FlowStep::Quote, FlowStep::Negotiate, FlowStep::Convert,
            FlowStep::FollowUp, FlowStep::Close,
        ];
        for i in 0..steps.len()-1 {
            assert_eq!(steps[i].next(), Some(steps[i+1].clone()));
        }
        assert_eq!(steps.last().unwrap().next(), None);
    }

    #[test]
    fn test_order_node_chain() {
        let nodes = OrderNode::all();
        assert_eq!(nodes.len(), 12);
        for i in 0..nodes.len()-1 {
            assert_eq!(nodes[i].next(), Some(nodes[i+1].clone()));
        }
        assert_eq!(nodes.last().unwrap().next(), None);
    }

    #[test]
    fn test_stable_id() {
        let id1 = stable_id(12345, 67890);
        let id2 = stable_id(12345, 67890);
        let id3 = stable_id(12346, 67890);
        assert_eq!(id1, id2); // 相同输入产生相同ID
        assert_ne!(id1, id3); // 不同输入产生不同ID
    }

    #[test]
    fn test_channel_from_str() {
        assert_eq!(Channel::from_str("阿里巴巴"), Some(Channel::Alibaba));
        assert_eq!(Channel::from_str("Email"), Some(Channel::Email));
        assert_eq!(Channel::from_str("自定义渠道"), Some(Channel::Other("自定义渠道".into())));
    }

    #[test]
    fn test_product_kind_from_str() {
        assert_eq!(ProductKind::from_str("ball valve"), Some(ProductKind::BallValve));
        assert_eq!(ProductKind::from_str("球阀"), Some(ProductKind::BallValve));
        assert_eq!(ProductKind::from_str("gate valve"), Some(ProductKind::GateValve));
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!(Currency::from_str("USD"), Some(Currency::USD));
        assert_eq!(Currency::from_str("$"), Some(Currency::USD));
        assert_eq!(Currency::from_str("人民币"), Some(Currency::CNY));
    }
}
