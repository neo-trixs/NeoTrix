//! Trade Data Model — 统一数据模型 (Single Source of Truth)
//!
//! 所有 trade 子模块共用的领域数据结构统一在此定义。
//! 子模块通过 `use super::data_model::*` 引用，禁止重复定义。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// 1. 枚举类型 —— 真源是 unified_types（见下）
// ============================================================
//
// 2026-10-07 融合（本文件 §2 辅助结构段之前全部 8 个类型）：
//
// 这 8 个类型原先在本文件与 `unified_types.rs` 各有一份，**逐字相同**
// （含 derive 列表、`#[default]` 变体、`TradeTerms::Other(String)` 的负载，
// 以及 `Product` 那 13 行手写 `impl Default`）。
//
// 三处独立证据指向 `unified_types` 为真源：
//   1. 本文件头 `:3-4` 「子模块通过 `use super::data_model::*` 引用，**禁止重复定义**」
//   2. `unified_types.rs:390` 「8. 通用枚举（**从 NeoTrix data_model 迁移**）」
//   3. `nt_act_trade/mod.rs:48-57` 的 re-export 全部指向 `unified_types`
//
// ⇒ 本文件改为 `pub use` 再导出，保持 `use super::data_model::*` 的
// **消费方零改动**（本文件内 `Product`/`Quote`/`Inquiry` 等结构体与
// `tests/data_model_tests.rs:7` 的 glob 导入都无需修改）。
//
// ⛔ **不融合的是结构体**：`Product`/`Quote`/`Order`/`Inquiry`/`InquiryItem`/
// `QuoteItem`/`Supplier` 在两处的**字段集不同**（如 `Inquiry`：本文件
// `{inquiry_no, trade_terms, metadata}` vs unified `{customer, created_at, updated_at}`；
// `currency: String` vs `Currency`），且可见性不同（`pub(crate)` vs `pub`）。
// 那正是 2026-09-29 融合 `MaterialSpec` 时写下的同一条理由。
pub use super::unified_types::{
    ConnectionType, DriveType, InquiryStatus, OrderStatus, Product, ProductCategory,
    QuoteStatus, TradeTerms,
};

// ============================================================
// 2. 辅助结构
// ============================================================

/// 材质规格
// 2026-09-29 融合：`MaterialSpec` 原在本文件与 `unified_types` 各有一份，
// 字段名+类型+顺序完全一致（`{name, standard, grade}: String`），且都无 impl 块。
// `nt_act_trade/mod.rs:51` 的 re-export 早已指向 `unified_types` ⇒ 那里是真源。
//
// 保留 re-export 以兼容本文件内 `Product`(:260) / `Quote`(:353) 仍引用它 ——
// 那两个类型与 unified_types 的同名类型**字段集不同**，不能一起融合。
pub use super::unified_types::MaterialSpec;
// 2026-09-29 自动融合（nt_fuse_types.py）：`PressureRating` 原在本文件与
// `l1_action/nt_act/nt_act_trade/unified_types.rs` 各有一份，字段名+类型+impl 块完全相同。
// 真源是后者（模块 mod.rs 的 re-export 指向它）⇒ 本文件改为 re-export，
// 消除「两份同名类型」的歧义。
pub use super::unified_types::PressureRating;
// 2026-09-29 自动融合（nt_fuse_types.py）：`SizeSpec` 原在本文件与
// `l1_action/nt_act/nt_act_trade/unified_types.rs` 各有一份，字段名+类型+impl 块完全相同。
// 真源是后者（模块 mod.rs 的 re-export 指向它）⇒ 本文件改为 re-export，
// 消除「两份同名类型」的歧义。
pub use super::unified_types::SizeSpec;


// 2026-09-29 自动融合（nt_fuse_types.py）：`PriceInfo` 原在本文件与
// `l1_action/nt_act/nt_act_trade/unified_types.rs` 各有一份，字段名+类型+impl 块完全相同。
// 真源是后者（模块 mod.rs 的 re-export 指向它）⇒ 本文件改为 re-export，
// 消除「两份同名类型」的歧义。
pub use super::unified_types::PriceInfo;


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
// 2026-09-29 自动融合（nt_fuse_types.py）：`PerformanceMetrics` 原在本文件与
// `l1_action/nt_act/nt_act_trade/unified_types.rs` 各有一份，字段名+类型+impl 块完全相同。
// 真源是后者（模块 mod.rs 的 re-export 指向它）⇒ 本文件改为 re-export，
// 消除「两份同名类型」的歧义。
pub use super::unified_types::PerformanceMetrics;


// 2026-09-29 自动融合（nt_fuse_types.py）：`InquiryMetadata` 原在本文件与
// `l1_action/nt_act/nt_act_trade/unified_types.rs` 各有一份，字段名+类型+impl 块完全相同。
// 真源是后者（模块 mod.rs 的 re-export 指向它）⇒ 本文件改为 re-export，
// 消除「两份同名类型」的歧义。
pub use super::unified_types::InquiryMetadata;


// ============================================================
// 3. 核心数据结构
// ============================================================

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
