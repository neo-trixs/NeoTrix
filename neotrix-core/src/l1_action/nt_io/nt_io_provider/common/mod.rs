//! 公共类型 / 工厂 / 工具
pub mod types;
pub mod factory;
pub mod privacy_guard;
pub mod generation_classifier;
pub use types::*;
pub use factory::*;
pub(crate) use privacy_guard::{configure_privacy_guard, privacy_guard_enabled, trust_from_name, domain_egress_route, enforce_egress_policy};
pub use generation_classifier::*;
