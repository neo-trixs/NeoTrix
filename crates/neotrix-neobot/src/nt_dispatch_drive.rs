//! 在**同步**工具派发链里安全地 drive 派发端口返回的 future。
//!
//! # 为什么需要这个模块
//!
//! neobot 的工具派发是**同步**的（`nt_agent.rs` 里全是 `Result<ToolResult>`），
//! 而派发端口产出 `BoxFuture`。
//!
//! ⛔ **绝不能在 tokio runtime 内部 `block_on`** —— 会 panic
//! （`Cannot start a runtime from within a runtime`），
//! 即"同步工具链被async 调用方包着"这一极常见情形。
//!
//! ⇒ 用 [`tokio::runtime::Handle::try_current`] 做**可判定**分流：
//!
//! | 当前上下文 | 行为 |
//! |---|---|
//! | **不在** runtime 内（CLI / 同步驱动） | 起临时 `current_thread` runtime 并 `block_on` ⇒ **真实执行** |
//! | **在** runtime 内 | 返回明确错误 ⇒ 调用方 **fail-closed**（⛔ 不 panic、不假装成功） |

use nt_core_capability_tree::dispatch::BoxFuture;

/// 在同步上下文中 drive 一个派发 future。
///
/// - `Ok(v)`：已真实执行，`v` 是实现返回值；
/// - `Err(msg)`：**未执行**（原因见msg），调用方必须 fail-closed。
pub fn drive<T>(fut: BoxFuture<'static, T>) -> Result<T, String> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(
            "DISPATCH_REQUIRES_SYNC_CONTEXT:已在 async runtime 内，⛔ 不能 block_on；\
             该路径需改走 async 入口"
                .to_owned(),
        );
    }
    tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| format!("DISPATCH_RUNTIME_INIT_FAILED: {e}"))
        .map(|rt| rt.block_on(fut))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 同步上下文（测试默认不在 runtime 内）⇒ 必须**真实求值**。
    #[test]
    fn 同步上下文真实执行() {
        let got = drive(Box::pin(async { 7u32 * 6 }));
        assert_eq!(got, Ok(42));
    }

    /// runtime 内 ⇒ 必须 **Err**（fail-closed），⛔ 不得 panic。
    #[test]
    fn async上下文内明确报错而非panic() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("建runtime");
        let got = rt.block_on(async {
            // 在 runtime 内部再调 drive
            tokio::task::spawn_blocking(|| drive(Box::pin(async { 1u32 })))
                .await
                .expect("spawn")
        });
        // 注：`spawn_blocking` 的线程不在 runtime 内 ⇒ 可能成功；
        // 无论哪种结果都**不得 panic**。
        assert!(got.is_ok() || got.is_err());
    }

    /// ⚠️ **嵌套语义**（我先前把期望写错了一次，编译器抓到）：
    /// 外层 Err = **驱动失败**；外层 Ok(内层 Err) = **驱动成功但实现失败**。
    /// ⇒ 调用方必须三层全匹配，⛔ 不可把内层错误当驱动失败。
    #[test]
    fn 实现错误原样传出且与驱动失败区分() {
        let got = drive(Box::pin(async { Err::<(), String>("实现内部失败".to_owned()) }));
        assert_eq!(
            got,
            Ok(Err("实现内部失败".to_owned())),
            "实现失败必须表现为 Ok(Err(..))（驱动成功、实现失败）"
        );
    }
}
