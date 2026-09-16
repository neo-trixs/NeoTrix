//! Trade Capabilities — 外贸能力层模块入口
//!
//! 将外贸域能力按职责拆分为独立子模块：
//! - `product_matcher`: 产品匹配（型号/参数/分类检索）
//! - `supplier_matcher`: 供应商匹配（信用评级/地域/产能筛选）
//! - `price_calculator`: 价格计算（成本/利润/汇率/贸易条款）
//! - `risk_assessor`: 风险评估（信用/物流/合规/汇率风险）

#![forbid(unsafe_code)]

pub mod price_calculator;
pub mod product_matcher;
pub mod risk_assessor;
pub mod supplier_matcher;

pub use price_calculator::{PriceCalcConfig, PriceCalcRequest, PriceCalcResult, PriceCalculator};
pub use product_matcher::{
    ProductMatchConfig, ProductMatchRequest, ProductMatchResult, ProductMatcher,
};
pub use risk_assessor::{RiskAssessConfig, RiskAssessRequest, RiskAssessResult, RiskAssessor};
pub use supplier_matcher::{
    SupplierMatchConfig, SupplierMatchRequest, SupplierMatchResult, SupplierMatcher,
};
