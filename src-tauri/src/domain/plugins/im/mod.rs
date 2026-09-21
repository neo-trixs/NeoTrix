//! # IM Domain Plugin
//!
//! 基于 DSH-IM 架构的多渠道 IM 插件：
//! - 9 个内置渠道：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp
//! - 流式响应支持
//! - 访问模式控制（白名单/响应模式）
//! - 超时恢复机制
//! - 会话渠道前缀路由
//!
//! Split from single-file `im.rs`: `types` holds all public data types,
//! `plugin` holds `ImPlugin` and its `DomainPlugin` impl.
//! Re-exports preserve the original `plugins::im::{...}` paths.

pub mod plugin;
pub mod types;

pub use plugin::ImPlugin;
pub use types::*;
