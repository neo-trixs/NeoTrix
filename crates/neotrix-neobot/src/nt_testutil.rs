//! 测试期临时目录工具（仅测试目标编译）。
//!
//! ## 为什么需要它
//!
//! 本 crate 的测试里有 **30 处**用 `std::env::temp_dir().join(...)` 造夹具，
//! 其中：
//!
//! - 19 处是**完全固定的名字**（`neobot-bash-test`、`neobot-agent-test`…），
//! - 11 处只按用例名唯一化（`neobot-serve-test-{case}`）——**同一用例跑两遍仍撞**。
//!
//! 后果不是「慢」，是**互相擦除**：一个测试的 `remove_dir_all` 会把另一个正在用的
//! 夹具删掉，于是断言在**与被测代码无关**的地方炸掉。
//!
//! 实测（2026-09-28）：把同一个 test binary **并发跑 3 份**，每份各挂 9–18 个测试，
//! 且**每份挂的还不是同一批** —— 正是共享夹具互相踩的指纹。串行跑则 353 全绿。
//! 这类 flake 比没有测试更糟：它会让人学会忽略红灯。
//!
//! ## 用法
//!
//! ```ignore
//! let dir = crate::nt_testutil::temp_dir("bash");  // 每个进程/每次调用都不同
//! ```
//!
//! 唯一性来自 **进程号 + 单调时钟纳秒**：进程号区分并发跑的多个 test binary
//! （cargo 只串行化**构建**，两个 `cargo test` 的**执行**阶段是并行的），
//! 纳秒区分同一进程内的多次调用与用例重跑。
//!
//! **不要**退回「固定名 + 手工 `remove_dir_all`」的写法：那正是本文件要修的东西。
//! 返回值已带 `neobot-test-<pid>-<nanos>-<tag>` 前缀，调用方不必再套一层目录。

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 一个**本进程本次调用独有**的临时目录路径（不创建；由调用方 `create_dir_all`）。
///
/// `tag` 只影响可读性，不参与唯一性 —— 唯一性由 pid + 纳秒保证。
pub fn temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "neobot-test-{}-{nanos}-{tag}",
        std::process::id()
    ))
}
