//! Trade Data Model — 外贸领域数据模型
//!
//! ⛔ **2026-10-08 撤下 SSOT 承诺（ROADMAP T1-2）**：本头曾声明
//! 「统一数据模型 (Single Source of Truth) / 所有 trade 子模块共用的领域数据结构统一在此定义」
//! —— 该承诺**三条断言里有两条实证为假**，且 `unified_types.rs:1-4` **逐字重复同一条承诺**，
//! 同一模块树里两个文件自称 SSOT。事实如下：
//!
//! - **真源在 `unified_types.rs`**（`nt_act_trade/mod.rs:51` 的 re-export 早已指向它）。
//!   本文件 `:35-96` 的 13 个类型**全部是 `pub use` 再导出**，零本地定义。
//! - **「子模块通过 `use super::data_model::*` 引用」= 假**：全仓仅
//!   `tests/data_model_tests.rs:7` 一处命中，**零个生产子模块**用它。
//! - **本文件真正持有的 8 个 struct**（`ContactInfo`/`Supplier`/`InquiryItem`/`Inquiry`/
//!   `QuoteItem`/`Quote`/`OrderItem`/`Order`，`:73-300`）与 `unified_types` 的同名类型
//!   **字段集不同**（同名不同概念，AGENTS.md L15），且**零外部消费者** ——
//!   唯一消费者是 `tests/data_model_tests.rs`，它在**一次 glob 里同时断言两个世界**
//!   （`:14-26` 断 unified 的 `Product`，`:29-74` 断本文件的 `code`/`status`/`order_no`）。
//! - ⛔ **不要按名字删这 8 个 struct**：`scripts/ops/nt_dup_types.py:225-230` 逐字记录了
//!   上一轮按名字删 15 个类型的后果 —— `cargo check` 报 `E0119`+`E0560`，已回滚。
//!   按字段判据（`nt_dup_types.py`）复测 13 个名字，**真重复组为 0**。
//!
//! ⇒ 保留本文件作为「零外部消费者的本地建模 + 测试钉子」，不再宣称 SSOT。

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
// 保留 re-export 供本文件下方的本地 struct 引用 ——
// 注意：`Product` **不在本文件定义**（只在下一行被再导出），`Quote` 是本文件 `:236` 的本地类型。
// ⛔ 2026-10-08 修正腐化指针：原文写「本文件内 `Product`(:260) / `Quote`(:353)」，
// 但本文件仅 338 行且 `:353` 不存在，`Product` 也不是本地定义 ⇒ 该指针已随编辑漂移。
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
