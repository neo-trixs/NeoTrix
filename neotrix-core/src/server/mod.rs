//! 服务层 — 仅保留 `session` (被 `cli/commands/session_cmds` 生产引用)。
//! HTTP API / WebSocket / H5 通信能力已拆解融合到 `nt_io_web` (R-P42)。

pub mod session;
pub use session::{SessionManager, SessionShare, SessionShareManager};
