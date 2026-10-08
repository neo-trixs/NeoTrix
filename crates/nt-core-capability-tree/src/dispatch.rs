//! 能力**派发端口** —— 与 `market` 清单同一套依赖倒置模式。
//!
//! ## 为什么端口在这里
//!
//! `capability_invoke` 的**执行入口**在 `neotrix-core`（`nt_act_trade`），
//! 而**调用方**在 `neotrix-neobot`。依赖方向固定 `core → neobot`
//! ⇒ neobot 无法直接触达 core 的实现，**且工作区里没有任何 crate 同时依赖两侧**
//! ⇒ 与 `market::TRADE_MANIFEST` 同一处境。
//!
//! 既然清单已经用「接口下沉共享 crate」解开，**派发也用同一手法**：
//!
//! ```text
//!   neotrix-core（实现）  ──register_dispatcher()──▶┐
//!                                                  ├─▶ nt-core-capability-tree
//!   neotrix-neobot（消费）───dispatch()────────────▶┘
//! ```
//!
//! ## 关键性质
//!
//! - **无实现时 `dispatch` 返回 `None`** ⛔ **不是**错误、**不是**空成功
//!   ⇒ 调用方据此保持 fail-closed（`CAPABILITY_BODY_NOT_EXECUTED`）。
//!   这条最重要：端口存在**绝不能**让「未执行」看起来像「执行了」。
//! - **注册与调用可在不同进程**：本进程内未注册就是 `None`，
//!   生产里 neobot 进程不会注册 ⇒ 由装配层在同进程内注册后才可用。
//! - 全局表用 `Mutex` ⇒ 线程安全，且**不含任何 `unsafe`**。
//!
//! ## 与 `signal()` 的关系
//!
//! 实现方在**真正的派发路径**里调
//! `nt_capability_canary::signal(会话键, 树 id)`
//! （core 的 `TradeCapabilityRegistry::get()` 已如此）⇒
//! 端口一旦被调用，金丝雀的 `fired_count` 才会动。
//!
//! **会话键随本端口一起传达**（2026-10-07，修 `OPEN-DEFECTS` P1-5）：
//! 金丝雀窗口已按会话分桶，若实现方拿不到会话键，打点就无处归属
//! ⇒ 端口签名必须携带它。

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// 派发函数签名：`输入 id` → `输入负载` → `会话键` → `结构化结果`。
///
/// # ⛔⛔ **本签名目前与实际执行形态不匹配 —— 不要据此注册实现**
///
/// 实测（2026-10-06）：trade 能力的执行入口是
/// `TradeCapability::execute_trade`，而它是 **`async fn`**
/// （`nt_act_trade/capability_registry.rs:32`）。
///
/// 本类型是**同步** `fn` 指针 ⇒ **承载不了 async 执行**。
///
/// 第二个障碍：`TradeCapabilityRegistry` 只有 `new() -> Self`（实例），
/// **没有全局单例**，而 `create_default_registry()` 的非测试引用为 0
/// ⇒ 无捕获的 `fn` 指针**无处取得注册表**。
///
/// ⇒ **在签名改为 async（装箱 future）且注册表有全局落点之前，
/// `register_dispatcher` 不应被真实实现调用。**
/// 现在唯一正确的用法就是测试（同步、可控），
/// 而生产里 `dispatch` 返回 `None` ⇒ 调用方 fail-closed ⇒ **与接线前行为一致**。
///
/// 详见 `docs/architecture/FOLLOWUP-TASKS-2026-10-06.md`。
/// 装箱 future 的别名 —— **刻意用 `std`，不给本 crate 加 `futures` 依赖**
/// （共享 crate 不该为端口引入运行时依赖）。
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// 派发函数签名（**async**）。
///
/// # ✅ 签名已与实际执行形态对齐（2026-10-06修正）
///
/// 实测：`TradeCapability::execute_trade` 是 `async fn`
/// （`nt_act_trade/capability_registry.rs:32`），而 `execute_trade` 的
/// 非测试调用者为 **0** ⇒ 没有既有 async 上下文约束本端口的形状，
/// 故**直接对齐 async**，而不是逼实现方用 `block_on` 包一层
///（后者在 async 上下文里会**嵌套运行时**，是真实的死锁风险）。
///
/// **执行器归属**：本端口**不选执行器** —— 谁调用谁提供
/// （neobot 与 core 都已有 `tokio`）⇒ 端口只产出 future，不drive 它。
///
/// # 第三个参数 = 金丝雀**会话键**（2026-10-07）
///
/// 实现方执行成功后要调 `signal(会话键, 树 id)`；窗口按会话分桶
/// ⇒ 会话键必须从调用方一路传到这里，否则打点无法归属
///（修 `OPEN-DEFECTS` P1-5「金丝雀窗口进程全局，多会话互相 reset」）。
///
/// # 仍存一个障碍（见 T4.5）
///
/// `TradeCapabilityRegistry` 只有 `new() -> Self`（实例），
/// **无全局单例** ⇒ 无捕获的 `fn` 指针**取不到注册表**。
/// core 侧需增设全局落点（或改用捕获式注册）。在此之前
/// `register_dispatcher` 不应被真实实现调用；生产 `dispatch` 返回 `None`
/// ⇒ 调用方 fail-closed ⇒ **与接线前行为一致**。
pub type DispatchFn =
    fn(&str, serde_json::Value, &str) -> BoxFuture<'static, Result<serde_json::Value, String>>;

type Table = BTreeMap<String, DispatchFn>;

fn table() -> &'static Mutex<Table> {
    static T: OnceLock<Mutex<Table>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(Table::new()))
}

/// 注册一个能力的派发实现（**覆盖同名**，便于测试重装）。
///
/// 生产应由**实现所在 crate**（core）在播种/装配时调用，**不要**由消费方注册
/// —— 否则消费方就能为别人的能力提供实现，端口就失去了隔离意义。
pub fn register_dispatcher(id: &str, f: DispatchFn) -> Result<(), String> {
    let mut t = table().lock().map_err(|e| format!("派发表锁投毒: {e}"))?;
    t.insert(id.to_owned(), f);
    drop(t);
    // ⇒ 自动登记到脊柱（存量通路不许在脊柱口径里"消失"）。
    //    登记失败**不**影响 `register_dispatcher` 的成功语义（派发表已就位），
    //    但脊柱里会缺项 ⇒ `dispatchable_ids()` 漏报，故用 `sync_one_into_spine`
    //    的返回值在测试里断言，生产不阻断（派发表才是权威执行面）。
    let _ = sync_one_into_spine(id, f, "", "经 dispatch::register_dispatcher 登记的执行器");
    Ok(())
}

/// 派发到某个能力。
///
/// - `Ok(Some(v))` ⇒ **真实执行过**，`v` 是实现返回的结果；
/// - `Ok(None)` ⇒ **本进程没有该能力的实现** ⇒ 调用方**必须** fail-closed；
/// - `Err(e)` ⇒ 实现已调用但**执行失败**（锁投毒等基础设施问题）。
///
/// `session` 原样传给实现 ⇒ 实现成功后的金丝雀打点归属该会话。
pub fn dispatch(
    id: &str,
    input: serde_json::Value,
    session: &str,
) -> Result<Option<BoxFuture<'static, Result<serde_json::Value, String>>>, String> {
    let f = {
        let t = table().lock().map_err(|e| format!("派发表锁投毒: {e}"))?;
        t.get(id).copied()
    };
    match f {
        // ⛔ 找不到实现 ⇒ None。**绝不能**退化成 `Ok(Some(json!({})))`
        //    ——那会让「没实现」看起来像「执行成功」。
        None => Ok(None),
        Some(f) => Ok(Some(f(id, input, session))),
    }
}

/// 已注册的实现数量（供门与诊断读取）。
pub fn registered_count() -> usize {
    table().lock().map(|t| t.len()).unwrap_or(0)
}

/// 某个 id 在**本进程**是否已注册派发实现（**id 级**探针，只读）。
///
/// # ⭐ 为什么需要它（两个已有的读数都答不了这道题）
///
/// - `market::ManifestEntry::executability` 是**仓级编译期常量**：
///   6 条清单在 `market.rs` 里写死 ⇒ 它答的是「这个能力**设计上**能不能干活」，
///   不是「**这个进程**里能不能跑」，且**没有任何运行期读者**拿它做派发判断。
/// - [`registered_count`] `> 0` 只答「**有没有**接上实现」，答不了
///   「**这个 id** 可不可跑」—— 派发表是**按 id** 的进程级全局表，
///   不同 id 落在不同装配层（trade / genoffice / 未来第三批），
///   完全可能「接了 3 个里的 1 个」而计数依然非 0。
///
/// ⇒ 上架面（清单、`capability_invoke` 回执、诊断/列表命令）需要一道
/// **id 级**探针：「它是否真的 runnable **in THIS process**」只能问表本身。
/// 这是「**写着健康、实际不能跑**」的反面：光看静态字段会**高估**可执行性。
///
/// # ⛔ 它**不执行**实现
///
/// 只 `contains_key`，⛔ 不取值、不构造 future、不碰 `session` ⇒ 探针
/// **零副作用**、零金丝雀打点（打点的语义是「真被执行过」，见本文件头）。
///
/// 锁投毒时返 `false`（fail-closed，与 [`registered_count`] 同款处置）；
/// 但本表**锁内从不跑用户代码**（`dispatch` 先把 fn 指针拷出来、再在锁外调用）
/// ⇒ 投毒实际不可达。
pub fn is_registered(id: &str) -> bool {
    table().lock().map(|t| t.contains_key(id)).unwrap_or(false)
}

/// 清空派发表（**仅测试用**：进程内注册是全局状态，测试之间需隔离）。
///
/// ⚠️ 必须与 [`test_guard`] 一起用：派发表是**进程级全局**，
/// 而测试**并行**执行 ⇒ 一个测试的 `clear_for_tests()` 会清掉另一个刚注册的
/// 实现（实测 `已注册则真实派发` 因此拿到 `None`）。
#[cfg(test)]
pub fn clear_for_tests() {
    if let Ok(mut t) = table().lock() {
        t.clear();
    }
}

/// 测试间串行化派发表访问的守卫。
#[cfg(test)]
pub fn test_guard() -> std::sync::MutexGuard<'static, ()> {
    static G: OnceLock<Mutex<()>> = OnceLock::new();
    let m = G.get_or_init(|| Mutex::new(()));
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_impl(
        _id: &str,
        _input: serde_json::Value,
        _session: &str,
    ) -> BoxFuture<'static, Result<serde_json::Value, String>> {
        Box::pin(async { Ok(serde_json::json!({"executed": true})) })
    }

    fn boom_impl(
        _id: &str,
        _input: serde_json::Value,
        _session: &str,
    ) -> BoxFuture<'static, Result<serde_json::Value, String>> {
        Box::pin(async { Err("实现内部失败".to_owned()) })
    }

    /// 测试用executor。**只在 `#[cfg(test)]` 里存在**，不污染运行时依赖。
    fn block_on<T>(fut: BoxFuture<'static, T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("建测试 runtime")
            .block_on(fut)
    }

    #[test]
    fn 未注册必须返回None而非空成功() {
        let _g = test_guard();
        clear_for_tests();
        let got = dispatch("NT-MIND::trade::nonexistent", serde_json::json!({}), "test:dispatch");
        assert!(
            matches!(got, Ok(None)),
            "无实现必须是 None（让调用方 fail-closed），实得 Ok/Err 已判但有实现={}",
            matches!(got, Ok(Some(_)))
        );
    }

    #[test]
    fn 已注册则真实派发() {
        let _g = test_guard();
        clear_for_tests();
        register_dispatcher("NT-MIND::trade::x", ok_impl).expect("注册");
        let fut = dispatch("NT-MIND::trade::x", serde_json::json!({"a":1}), "test:dispatch").expect("派发").expect("有实现");
        assert_eq!(block_on(fut), Ok(serde_json::json!({"executed": true})));
        assert_eq!(registered_count(), 1);
    }

    #[test]
    fn 实现失败必须冒泡为Err() {
        let _g = test_guard();
        clear_for_tests();
        register_dispatcher("NT-MIND::trade::boom", boom_impl).expect("注册");
        let fut = dispatch("NT-MIND::trade::boom", serde_json::json!({}), "test:dispatch").expect("派发").expect("有实现");
        assert!(matches!(block_on(fut), Err(ref e) if e.contains("实现内部失败")));
    }

    #[test]
    fn 注册覆盖同名() {
        let _g = test_guard();
        clear_for_tests();
        register_dispatcher("NT-MIND::trade::dup", ok_impl).expect("注册");
        register_dispatcher("NT-MIND::trade::dup", boom_impl).expect("覆盖注册");
        let fut = dispatch("NT-MIND::trade::dup", serde_json::json!({}), "test:dispatch").expect("派发").expect("有实现");
        assert!(block_on(fut).is_err(), "同名应被后者覆盖");
        clear_for_tests();
    }

    /// ⭐ 探针答的是「**这个 id** 可不可跑」，不是「有没有接上实现」。
    #[test]
    fn 探针按id判定而非按数量() {
        let _g = test_guard();
        clear_for_tests();
        register_dispatcher("NT-MIND::trade::probe", ok_impl).expect("注册");
        assert!(registered_count() >= 1, "计数非 0 却答不出 id ⇒ 探针没意义");
        assert!(is_registered("NT-MIND::trade::probe"), "刚注册的 id 必须为真");
        // ⛔ 计数非 0 **不能**推出别的 id 可跑 ⇒ 这正是探针存在的理由。
        assert!(
            !is_registered("some-other-id"),
            "未注册 id 必须为假（fail-closed 方向：不得高估可执行性）"
        );
        clear_for_tests();
    }

    /// 空表 ⇒ 任何 id 都不可跑（探针的基线）。
    #[test]
    fn 空表时探针必为假() {
        let _g = test_guard();
        clear_for_tests();
        assert!(!is_registered("NT-MIND::trade::probe"), "空表不得报任何 id 可跑");
        clear_for_tests();
    }
}

// ─── 与能力脊柱的桥（2026-10-08）──────────────────────────────────────
//
// ## 为什么需要桥
//
// 脊柱（`spine::Spine`）是「运行时能力唯一真源」，而本模块的 `Table` 是
// **已接线**的生产派发面。两者若不互通，就又造出「两套能力宇宙」——
// 正是脊柱要消灭的那个病（2026-10-08 审计的头号发现：磁盘 326 节点树与
// 6 个清单 id 互不相干，只在某个进程全局里偶然相遇）。
//
// ## 方向**刻意不对称**，勿对称化
//
// - **`register_dispatcher` ⇒ 自动登记到脊柱**：`Table` 是既有生产通路，
//   存量实现都走它 ⇒ 登记必须**自动**出现在脊柱里，否则
//   `spine::dispatchable_ids()` 会漏掉它们（那会让真实能力重新变不可见）。
// - **脊柱 ⇒ `Table` 反向**（[`sync_spine_into_dispatch`]）：**显式**调用。
//   因为 `Table` 只存**无捕获 `fn` 指针**，带状态的插件塞进去会**悄悄丢状态**
//   ⇒ 绝不能自动同步，只能由调用方在知情下选择。
//   ⛔ 契约由 trait 方法保证：`CapabilityExecutor::as_fn_ptr()` 默认 `None`，
//   **只有** `SpineExecutor`（无捕获适配器）才返回 `Some`。

use crate::spine::{CapabilityDescriptor, SpineExecutor};

/// 把 `Table` 里的实现登记进脊柱（保持调用方给定的元信息）。
///
/// 由 [`register_dispatcher`] 自动调用；也可用于补登记历史实现。
pub fn sync_one_into_spine(
    id: &str,
    f: DispatchFn,
    category: &str,
    description: &str,
) -> Result<(), crate::spine::SpineError> {
    let desc = CapabilityDescriptor::new(id, description).with_category(category);
    crate::spine::with_spine(|sp| sp.register(std::sync::Arc::new(SpineExecutor::new(desc, f))))
        .map_err(|e| e)?
}

/// 把脊柱里**元信息齐备且能降级为无捕获 fn**的执行器同步进派发表。
///
/// ⚠️ 会**覆盖**同名 `Table` 项（`insert` 语义）⇒ 调用前确认这是想要的。
/// ⚠️ 缺 `license`/`version` 的执行器**不上**派发表（fail-closed，
///    否则脊柱口径说「可派发」而派发表没有，两边又分叉）。
///
/// 返回 `(同步条数, 被跳过 id)`。带状态的执行器一定落在「跳过」里。
pub fn sync_spine_into_dispatch() -> (usize, Vec<String>) {
    let Ok(sp) = crate::spine::global_spine().lock() else {
        return (0, vec!["<锁投毒>".to_owned()]);
    };
    let mut n = 0usize;
    let mut skipped = Vec::new();
    for d in sp.descriptors() {
        if d.license.trim().is_empty() || d.version.trim().is_empty() {
            skipped.push(d.id.clone());
            continue;
        }
        let Some(exec) = sp.get(&d.id) else { continue };
        // ⛔ 带状态执行器（as_fn_ptr == None）**跳过并报出**，绝不静默降级
        let Some(f) = exec.as_fn_ptr() else {
            skipped.push(d.id.clone());
            continue;
        };
        if table()
            .lock()
            .map(|mut t| t.insert(d.id.clone(), f).is_none())
            .unwrap_or(false)
        {
            n += 1;
        }
    }
    (n, skipped)
}

#[cfg(test)]
mod spine_bridge_tests {
    use super::*;
    use crate::spine::{CapabilityExecutor, CapabilityHealth};

    /// 派发表登记 ⇒ 脊柱**自动**可见（否则真实能力会在脊柱口径里消失）。
    #[test]
    fn 派发表登记自动进入脊柱() {
        let id = "test::bridge::fn";
        register_dispatcher(id, |_id: &str, i: serde_json::Value, _s: &str| {
            Box::pin(async move { Ok(i) })
        })
        .expect("登记应成功");
        assert!(
            crate::spine::with_spine(|sp| sp.contains(id)).expect("锁"),
            "派发表里有的执行器，脊柱里必须也有"
        );
        crate::spine::with_spine(|sp| {
            sp.unregister(id, 0).ok();
        });
    }

    /// 反向同步必须**跳过**带状态的执行器（无捕获 fn 指针带不了状态）。
    #[test]
    fn 反向同步跳过带状态执行器() {
        struct Stateful;
        impl CapabilityExecutor for Stateful {
            fn descriptor(&self) -> CapabilityDescriptor {
                CapabilityDescriptor::new("test::bridge::stateful", "带状态")
                    .with_version("1.0.0")
                    .with_license("LicenseRef-Test")
            }
            fn execute(
                &self,
                _i: serde_json::Value,
                _s: &str,
            ) -> BoxFuture<'static, Result<serde_json::Value, String>> {
                Box::pin(async { Ok(serde_json::Value::Null) })
            }
            fn health(&self) -> CapabilityHealth {
                CapabilityHealth::Healthy
            }
            // 不覆写 as_fn_ptr ⇒ 默认 None ⇒ 必须被跳过
        }
        let id = "test::bridge::stateful";
        crate::spine::with_spine(|sp| sp.register(std::sync::Arc::new(Stateful))).expect("登记");
        let (_n, skipped) = sync_spine_into_dispatch();
        assert!(
            skipped.iter().any(|s| s == id),
            "带状态执行器必须被跳过并报出（实得 {skipped:?}）"
        );
        assert!(
            !table().lock().map(|t| t.contains_key(id)).unwrap_or(true),
            "带状态执行器不得被塞进无捕获 fn 指针表"
        );
        crate::spine::with_spine(|sp| {
            sp.unregister(id, 0).ok();
        });
    }

    /// 缺 license/version 的执行器不上派发表（fail-closed，不得两边分叉）。
    #[test]
    fn 缺元信息者不上派发表() {
        let id = "test::bridge::nometa";
        crate::spine::with_spine(|sp| {
            sp.register(std::sync::Arc::new(SpineExecutor::new(
                CapabilityDescriptor::new(id, "缺元信息"),
                |_id: &str, i: serde_json::Value, _s: &str| Box::pin(async move { Ok(i) }),
            )))
        })
        .expect("登记");
        let (_n, skipped) = sync_spine_into_dispatch();
        assert!(skipped.iter().any(|s| s == id), "缺元信息者必须被跳过");
        crate::spine::with_spine(|sp| {
            sp.unregister(id, 0).ok();
        });
    }
}
