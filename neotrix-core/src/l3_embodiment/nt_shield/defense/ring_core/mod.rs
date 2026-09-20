//! Ring Core - 核心环：零信任，最小权限
//!
//! 信任锚点 + 推理链保护 + ASI 合规
//! 信号流：所有输入必须经过此环验证

pub mod trust_anchor;
pub mod reasoning_shield;
pub mod asi_compliance;

pub use trust_anchor::TrustAnchor;
pub use reasoning_shield::ReasoningShield;
pub use asi_compliance::AsiComplianceChecker;

/// 核心环验证结果
#[derive(Debug, Clone)]
pub struct CoreVerification {
    pub trust_level: TrustLevel,
    pub reasoning_safe: bool,
    pub asi_compliant: bool,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    Unknown = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Trusted = 4,
}
