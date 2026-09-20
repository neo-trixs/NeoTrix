//! V2 Tauri 命令 — NeoTrix V2 架构的 Tauri 后端
//!
//! 当前只保留 unified + PTY + model_pool + proxy_pool 命令，其他旧命令模块暂不编译。

pub mod domain_cmd;
pub mod file_drop;
pub mod hive;
pub mod im;
pub mod model_commands;
pub mod model_pool;
pub mod neotrix_cli;
pub mod onboarding;
pub mod proxy_pool;
pub mod provider_commands;
pub mod pty;
pub mod unified;
