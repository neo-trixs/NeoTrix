//! Trade Data Model — 统一数据模型 (Single Source of Truth)
//!
//! 所有 trade 子模块共用的领域数据结构统一在此定义。
//! 子模块通过 `use super::data_model::*` 引用，禁止重复定义。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// 1. 枚举类型
// ============================================================

/// 产品分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProductCategory {
    /// 阀门
    #[default]
    Valve,
    /// 泵
    Pump,
    /// 法兰
    Flange,
    /// 管件
    PipeFitting,
    /// 仪表
    Instrument,
    /// 其他
    Other,
}

/// 驱动类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DriveType {
    /// 手动
    #[default]
    Manual,
    /// 电动
    Electric,
    /// 气动
    Pneumatic,
    /// 液压
    Hydraulic,
    /// 电磁
    Electromagnetic,
    /// 无 (被动件)
    None,
}

/// 连接类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ConnectionType {
    /// 法兰连接
    #[default]
    Flanged,
    /// 焊接
    Welded,
    /// 螺纹连接
    Threaded,
    /// 卡套
    Compression,
    /// 快接
    QuickConnect,
    /// 对夹
    Wafer,
    /// 其他
    Other,
}

/// 贸易条款 (Incoterms)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TradeTerms {
    /// 工厂交货
    #[default]
    ExWorks,
    /// 货交承运人
    Fca,
    /// 船上交货 (装运港)
    Fob,
    /// 成本加运费
    Cfr,
    /// 成本保险费加运费
    Cif,
    /// 目的地交货
    Ddp,
    /// 其他
    Other(String),
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OrderStatus {
    /// 草稿
    #[default]
    Draft,
    /// 已确认
    Confirmed,
    /// 生产中
    InProduction,
    /// 已发货
    Shipped,
    /// 已签收
    Delivered,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

/// 报价状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum QuoteStatus {
    /// 草稿
    #[default]
    Draft,
    /// 已发送
    Sent,
    /// 客户已确认
    Accepted,
    /// 客户已拒绝
    Rejected,
    /// 已过期
    Expired,
    /// 已撤回
    Withdrawn,
}

/// 询价状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum InquiryStatus {
    /// 收到询价
    #[default]
    Received,
    /// 分析中
    Analyzing,
    /// 报价中
    Quoting,
    /// 已报价
    Quoted,
    /// 已成交
    Won,
    /// 已丢失
    Lost,
    /// 已取消
    Cancelled,
}

// ============================================================
// 2. 辅助结构
// ============================================================

/// 材质规格
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaterialSpec {
    /// 材质名称 (如 WCB, CF8M, 316L)
    pub name: String,
    /// 标准 (如 ASTM A216, ASTM A351)
    pub standard: String,
    /// 牌号
    pub grade: String,
}

/// 压力等级
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PressureRating {
    /// 数值 (如 150, 300, 600)
    pub value: u32,
    /// 单位 (如 PN, CLASS)
    pub unit: String,
}

/// 尺寸规格
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeSpec {
    /// 公称直径 (如 DN50, 2")
    pub nominal: String,
    /// 内径 (mm)
    pub inner_diameter: Option<f64>,
    /// 外径 (mm)
    pub outer_diameter: Option<f64>,
    /// 长度 (mm)
    pub length: Option<f64>,
}

/// 价格信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceInfo {
    /// 单价
    pub unit_price: f64,
    /// 货币 (如 CNY, USD, EUR)
    pub currency: String,
    /// 折扣率 (0.0 ~ 1.0)
    pub discount_rate: Option<f64>,
    /// 含税标识
    pub tax_included: bool,
}

/// 联系信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactInfo {
    /// 联系人姓名
    pub name: String,
    /// 电话
    pub phone: String,
    /// 邮箱
    pub email: String,
    /// 职位
    pub title: String,
    /// 微信
    pub wechat: Option<String>,
}

/// 供应商绩效指标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// 交货准时率 (0.0 ~ 1.0)
    pub on_time_delivery_rate: f64,
    /// 质量合格率 (0.0 ~ 1.0)
    pub quality_pass_rate: f64,
    /// 响应时间 (小时)
    pub response_time_hours: f64,
    /// 历史合作次数
    pub cooperation_count: u32,
    /// 综合评分 (0.0 ~ 100.0)
    pub overall_score: f64,
}

/// 询价元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InquiryMetadata {
    /// 项目名称
    pub project_name: Option<String>,
    /// 项目编号
    pub project_no: Option<String>,
    /// 目的地
    pub destination: Option<String>,
    /// 交货期要求
    pub delivery_requirement: Option<String>,
    /// 备注
    pub notes: Option<String>,
}

// ============================================================
// 3. 核心数据结构
// ============================================================

/// 产品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    /// 产品编码
    pub code: String,
    /// 产品分类
    pub category: ProductCategory,
    /// 子分类
    pub subcategory: String,
    /// 型号
    pub model: String,
    /// 材质
    pub materials: Vec<MaterialSpec>,
    /// 驱动类型
    pub drive_type: DriveType,
    /// 连接类型
    pub connection_type: ConnectionType,
    /// 标准 (如 API 600, GB/T 12235)
    pub standard: String,
    /// 压力等级
    pub pressure: PressureRating,
    /// 尺寸规格
    pub size: SizeSpec,
    /// 参考价格
    pub price: PriceInfo,
    /// 供应商 ID
    pub supplier_id: String,
    /// 质量等级
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
    /// 供应商 ID
    pub id: String,
    /// 供应商编码
    pub code: String,
    /// 供应商名称
    pub name: String,
    /// 产品类别
    pub category: ProductCategory,
    /// 所在区域
    pub region: String,
    /// 主营产品
    pub main_products: Vec<String>,
    /// 联系信息
    pub contact: ContactInfo,
    /// 供应商等级
    pub grade: String,
    /// 启用状态
    pub status: String,
    /// 绩效指标
    pub performance: PerformanceMetrics,
}

impl Default for Supplier {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            code: String::new(),
            name: String::new(),
            category: ProductCategory::default(),
            region: String::new(),
            main_products: Vec::new(),
            contact: ContactInfo::default(),
            grade: String::new(),
            status: "active".to_string(),
            performance: PerformanceMetrics::default(),
        }
    }
}

/// 询价项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InquiryItem {
    /// 序号
    pub seq: u32,
    /// 产品名称
    pub product_name: String,
    /// 型号
    pub model: String,
    /// 标准
    pub standard: String,
    /// 材质要求
    pub materials: Vec<MaterialSpec>,
    /// 驱动类型
    pub drive_type: DriveType,
    /// 连接类型
    pub connection_type: ConnectionType,
    /// 压力等级
    pub pressure: PressureRating,
    /// 尺寸规格
    pub size: SizeSpec,
    /// 数量
    pub quantity: u32,
    /// 单位 (如 pcs, set, ton)
    pub unit: String,
    /// 目标单价
    pub target_price: Option<PriceInfo>,
    /// 备注
    pub remarks: Option<String>,
}

/// 询价单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inquiry {
    /// 询价 ID
    pub id: String,
    /// 询价单号
    pub inquiry_no: String,
    /// 客户名称
    pub customer: String,
    /// 贸易条款
    pub trade_terms: TradeTerms,
    /// 询价项
    pub items: Vec<InquiryItem>,
    /// 元数据
    pub metadata: InquiryMetadata,
    /// 状态
    pub status: InquiryStatus,
}

impl Default for Inquiry {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            inquiry_no: String::new(),
            customer: String::new(),
            trade_terms: TradeTerms::default(),
            items: Vec::new(),
            metadata: InquiryMetadata::default(),
            status: InquiryStatus::default(),
        }
    }
}

/// 报价项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuoteItem {
    /// 产品 ID (引用 Product.code)
    pub product_id: String,
    /// 产品名称
    pub product_name: String,
    /// 型号
    pub model: String,
    /// 规格描述
    pub specification: String,
    /// 数量
    pub quantity: u32,
    /// 单位
    pub unit: String,
    /// 单价
    pub unit_price: f64,
    /// 小计
    pub total_price: f64,
    /// 交货周期 (天)
    pub delivery_days: u32,
    /// 备注
    pub remarks: Option<String>,
}

/// 报价单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// 报价 ID
    pub id: String,
    /// 报价单号
    pub quote_no: String,
    /// 关联询价 ID
    pub inquiry_id: String,
    /// 报价项
    pub items: Vec<QuoteItem>,
    /// 总金额
    pub total_amount: f64,
    /// 货币
    pub currency: String,
    /// 贸易条款
    pub trade_terms: TradeTerms,
    /// 有效天数
    pub validity_days: u32,
    /// 状态
    pub status: QuoteStatus,
}

impl Default for Quote {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            quote_no: String::new(),
            inquiry_id: String::new(),
            items: Vec::new(),
            total_amount: 0.0,
            currency: "CNY".to_string(),
            trade_terms: TradeTerms::default(),
            validity_days: 30,
            status: QuoteStatus::default(),
        }
    }
}

/// 订单项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderItem {
    /// 产品 ID (引用 Product.code)
    pub product_id: String,
    /// 产品名称
    pub product_name: String,
    /// 型号
    pub model: String,
    /// 规格描述
    pub specification: String,
    /// 数量
    pub quantity: u32,
    /// 单位
    pub unit: String,
    /// 单价
    pub unit_price: f64,
    /// 小计
    pub total_price: f64,
    /// 交货日期
    pub delivery_date: Option<String>,
    /// 状态
    pub status: OrderStatus,
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单 ID
    pub id: String,
    /// 订单编号
    pub order_no: String,
    /// 关联报价 ID
    pub quote_id: String,
    /// 客户名称
    pub customer: String,
    /// 订单项
    pub items: Vec<OrderItem>,
    /// 总金额
    pub total_amount: f64,
    /// 货币
    pub currency: String,
    /// 贸易条款
    pub trade_terms: TradeTerms,
    /// 付款条款
    pub payment_terms: String,
    /// 状态
    pub status: OrderStatus,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            order_no: String::new(),
            quote_id: String::new(),
            customer: String::new(),
            items: Vec::new(),
            total_amount: 0.0,
            currency: "CNY".to_string(),
            trade_terms: TradeTerms::default(),
            payment_terms: String::new(),
            status: OrderStatus::default(),
        }
    }
}
