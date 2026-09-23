//! V2 Tauri 命令 — NeoTrix V2 架构的 Tauri 后端
//!
//! V2 命令模块：unified, PTY, model_pool, proxy_pool, IM, hive, domain_cmd, file_drop 等。

pub mod domain_cmd;
pub mod file_drop;
pub mod hive;
pub mod im;
pub mod model_commands;
pub mod model_pool;
pub mod neobot;
pub mod neotrix_cli;
pub mod onboarding;
pub mod provider_commands;
pub mod proxy_pool;
pub mod pty;
pub mod chat;
pub mod unified;
