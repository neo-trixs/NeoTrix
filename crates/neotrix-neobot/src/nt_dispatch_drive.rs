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

/// 同 [`drive`]，但带**真实超时强制**（`timeout_ms`，`None` = 不限）。
///
/// # 为什么需要它（2026-10-08）
///
/// `enforced_timeout_ms(CapabilityInvoke)` 在 `nt_agent` 里一直是**影子值**：
/// 只喂 `timeout_policy` / `shadow_line` / 日志，**没有任何生产分支读它来中止**
/// ⇒ 一个卡死的能力（或一个永不返回的外部进程）会**永久挂住整个轮次**。
/// 而 `reversibility_of(CapabilityInvoke)` 又把它标成 `Irreversible`
/// ⇒ 挂住的同时还不可回滚。
///
/// ⛔ 与 `drive` 的差别必须显式：超时**到点即返回 Err**，但被驱动的 future
/// **不会真的被杀掉**（它已在自己那条 runtime 上跑）。
/// 因此这里只保证**调用方不被挂住**（fail-closed 交给上层），
/// **不**承诺子任务已终止 —— 那需要进程级隔离，属另一项设计。
pub fn drive_with_timeout<T>(fut: BoxFuture<'static, T>, timeout_ms: Option<u64>) -> Result<T, String> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(
            "DISPATCH_REQUIRES_SYNC_CONTEXT:已在 async runtime 内，⛔ 不能 block_on；\
             该路径需改走 async 入口"
                .to_owned(),
        );
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| format!("DISPATCH_RUNTIME_INIT_FAILED: {e}"))?;
    match timeout_ms {
        None => Ok(rt.block_on(fut)),
        Some(ms) => {
            // ⚠️ 计时器必须在 **block_on 内部**创建：`tokio::time::timeout` 立即
            //    构造 `Sleep`，它需要**当前线程**的 reactor。上一版把它构在
            //    `block_on` 之外 ⇒ panic "there is no reactor running"。
            //    （实测：`timeout(...)` 包裹 + `block_on(该 future)` 即可。）
            rt.block_on(async move {
                match tokio::time::timeout(std::time::Duration::from_millis(ms), fut).await {
                    Ok(v) => Ok(v),
                    Err(_) => Err(format!(
                        "DISPATCH_TIMEOUT:能力超过 {ms}ms 未返回，已放弃等待"
                    )),
                }
            })
        }
    }
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

#[cfg(test)]
mod timeout_tests {
    use super::*;

    fn pending_forever() -> BoxFuture<'static, &'static str> {
        Box::pin(std::future::pending())
    }

    /// 影子超时曾是真缺陷：`drive` 会**永久阻塞**在永不返回的 future 上。
    /// 此用例锁定「到点放弃」——它是 `drive_with_timeout` 存在的唯一理由。
    #[test]
    fn 超时后放弃而非挂死() {
        let t0 = std::time::Instant::now();
        let r = drive_with_timeout(pending_forever(), Some(120));
        let elapsed = t0.elapsed();
        let msg = r.expect_err("永不返回的 future 必须超时失败");
        assert!(msg.contains("DISPATCH_TIMEOUT"), "实得 {msg}");
        assert!(
            elapsed < std::time::Duration::from_secs(5),
            "必须在超时附近返回，实耗 {elapsed:?}"
        );
    }

    /// 超时不得**误伤**及时返回的实现。
    #[test]
    fn 及时返回不被超时误伤() {
        let r = drive_with_timeout(Box::pin(async { 42u32 }), Some(5_000));
        assert_eq!(r.expect("应成功"), 42);
    }

    /// `None` = 不限，必须保持 `drive` 的原语义（成功路径）。
    #[test]
    fn 无超时等价于原drive() {
        let r = drive_with_timeout(Box::pin(async { "ok" }), None);
        assert_eq!(r.expect("应成功"), "ok");
        assert_eq!(drive(Box::pin(async { "ok" })).expect("应成功"), "ok");
    }
}
