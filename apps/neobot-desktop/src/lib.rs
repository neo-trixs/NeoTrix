//! NeoBot 桌面端 —— **薄壳**。
//!
//! 这里的哲学只有一句：**逻辑不搬进 app**。
//! 能力实现全在同仓 `crates/neotrix-neobot`，本 crate 只做
//! 「Tauri 命令注册」+「前端资源装配」两件接线活。
//!
//! 依据：d5413335 删除内嵌 app 的理由之一是「桌面端曾是两份同源物」。
//! 这次重建若把库的代码复制进 app，那个问题就原样回来了。

/// 前端资源目录（由 vite build 产出）。
pub const FRONTEND_DIST: &str = "../frontend/dist";
