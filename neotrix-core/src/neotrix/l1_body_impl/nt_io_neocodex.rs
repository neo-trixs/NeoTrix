//! NT-IO neocodex 集成 — 入口 re-export 层。
//!
//! 原 god-file (4399 行, 27 impl + 165 fn) 按职责拆分为 `nt_io_neocodex/` 子目录。
//! 本文件仅保留模块声明 + `pub use` 重导出, 保持 `crate::neotrix::l1_body_impl::nt_io_neocodex::*`
//! 公共 API 面完全不变。

mod acp;
mod agent;
mod context;
mod cost;
mod evolution;
mod goals;
mod hooks;
mod markdown;
mod permissions;
mod provider;
mod stream;
mod subagent;
mod tui;
mod wire;

pub use acp::*;
pub use agent::*;
pub use context::*;
pub use cost::*;
pub use evolution::*;
pub use goals::*;
pub use hooks::*;
pub use markdown::*;
pub use permissions::*;
pub use provider::*;
pub use stream::*;
pub use subagent::*;
pub use tui::*;
pub use wire::*;