//! Trade Capabilities — 外贸能力层模块入口
//!
//! 将外贸域能力按职责拆分为独立子模块：
//! - `product_matcher`: 产品匹配（型号/参数/分类检索）
//! - `supplier_matcher`: 供应商匹配（信用评级/地域/产能筛选）
//! - `price_calculator`: 价格计算（成本/利润/汇率/贸易条款）
//! - `risk_assessor`: 风险评估（信用/物流/合规/汇率风险）
//! - `v58_normalize`: V58 七维标准化＋指纹（匹配前归一化）
//! - `quotation_v64`: v6.4 报价单数学＋A/B/C 置信门
//! - `email_attachments`: 邮件附件链（列表解析＋下载 URL＋文本路由）
//! - `follow_logs`: 逐客户跟进日志（时分秒参数＋解析＋IP 配额熔断）
//! - `operator_channel`: 操作员切换与管理员视图（cid/lid＋ViewGuard）

#![forbid(unsafe_code)]

pub mod email_attachments;
pub mod follow_logs;
pub mod operator_channel;
pub mod price_calculator;
pub mod product_matcher;
pub mod quotation_v64;
pub mod risk_assessor;
pub mod supplier_matcher;
pub mod v58_normalize;

pub use email_attachments::{download_url, parse_attachments, AttachmentInfo, ATTACH_DETAIL_PATH};
pub use follow_logs::{
    follow_log_params, is_rate_limited, parse_follow_logs, resume_todo, FollowLogEntry,
    RateCircuitBreaker, FOLLOW_LOGS_PATH,
};
pub use operator_channel::{
    parse_operator_profile, switch_request, switch_succeeded, switch_to, OperatorProfile,
    SwitchRequest, ViewGuard, SWITCH_PATH,
};
pub use price_calculator::{PriceCalcConfig, PriceCalcRequest, PriceCalcResult, PriceCalculator};
pub use product_matcher::{
    ProductMatchConfig, ProductMatchRequest, ProductMatchResult, ProductMatcher,
};
pub use quotation_v64::{
    build_quotation, price_cell, satisfy_text, terms, unit_price, Confidence, Quotation,
    QuoteInput, QuoteRow, DEFAULT_FREIGHT, UNMATCHED,
};
pub use risk_assessor::{RiskAssessConfig, RiskAssessRequest, RiskAssessResult, RiskAssessor};
pub use supplier_matcher::{
    SupplierMatchConfig, SupplierMatchRequest, SupplierMatchResult, SupplierMatcher,
};
pub use v58_normalize::{
    fingerprint, inch_to_dn, norm_cert, norm_conn, norm_drive, norm_material, norm_model,
    norm_pressure, norm_size,
};
