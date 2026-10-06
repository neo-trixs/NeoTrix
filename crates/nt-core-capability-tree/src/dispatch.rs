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
//! 实现方在**真正的派发路径**里调 `nt_capability_canary::signal(树 id)`
//! （core 的 `TradeCapabilityRegistry::get()` 已如此）⇒
//! 端口一旦被调用，金丝雀的 `fired_count` 才会动。

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// 派发函数签名：`输入 id` → `输入负载` → `结构化结果`。
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
pub type DispatchFn = fn(&str, &serde_json::Value) -> Result<serde_json::Value, String>;

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
    Ok(())
}

/// 派发到某个能力。
///
/// - `Ok(Some(v))` ⇒ **真实执行过**，`v` 是实现返回的结果；
/// - `Ok(None)` ⇒ **本进程没有该能力的实现** ⇒ 调用方**必须** fail-closed；
/// - `Err(e)` ⇒ 实现已调用但**执行失败**（锁投毒等基础设施问题）。
pub fn dispatch(id: &str, input: &serde_json::Value) -> Result<Option<serde_json::Value>, String> {
    let f = {
        let t = table().lock().map_err(|e| format!("派发表锁投毒: {e}"))?;
        t.get(id).copied()
    };
    match f {
        // ⛔ 找不到实现 ⇒ None。**绝不能**退化成 `Ok(Some(json!({})))`
        //    ——那会让「没实现」看起来像「执行成功」。
        None => Ok(None),
        Some(f) => f(id, input).map(Some),
    }
}

/// 已注册的实现数量（供门与诊断读取）。
pub fn registered_count() -> usize {
    table().lock().map(|t| t.len()).unwrap_or(0)
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

    fn ok_impl(_id: &str, _input: &serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"executed": true}))
    }

    fn boom_impl(_id: &str, _input: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("实现内部失败".to_owned())
    }

    #[test]
    fn 未注册必须返回None而非空成功() {
        let _g = test_guard();
        clear_for_tests();
        let got = dispatch("NT-MIND::trade::nonexistent", &serde_json::json!({}));
        assert!(
            matches!(got, Ok(None)),
            "无实现必须是 None（让调用方 fail-closed），实得 {got:?}"
        );
    }

    #[test]
    fn 已注册则真实派发() {
        let _g = test_guard();
        clear_for_tests();
        register_dispatcher("NT-MIND::trade::x", ok_impl).expect("注册");
        let got = dispatch("NT-MIND::trade::x", &serde_json::json!({"a":1})).expect("派发");
        assert_eq!(got, Some(serde_json::json!({"executed": true})));
        assert_eq!(registered_count(), 1);
    }

    #[test]
    fn 实现失败必须冒泡为Err() {
        let _g = test_guard();
        clear_for_tests();
        register_dispatcher("NT-MIND::trade::boom", boom_impl).expect("注册");
        let got = dispatch("NT-MIND::trade::boom", &serde_json::json!({}));
        assert!(matches!(got, Err(ref e) if e.contains("实现内部失败")));
    }

    #[test]
    fn 注册覆盖同名() {
        let _g = test_guard();
        clear_for_tests();
        register_dispatcher("NT-MIND::trade::dup", ok_impl).expect("注册");
        register_dispatcher("NT-MIND::trade::dup", boom_impl).expect("覆盖注册");
        let got = dispatch("NT-MIND::trade::dup", &serde_json::json!({}));
        assert!(got.is_err(), "同名应被后者覆盖");
        clear_for_tests();
    }
}