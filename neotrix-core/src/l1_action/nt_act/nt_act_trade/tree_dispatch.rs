//! 能力树 id → 真实执行器的派发接线（P0.2）。
//!
//! # 为什么只有 2 / 5
//!
//! 实测 5 个贸易能力的可执行面**异构**（见 `FOLLOWUP-TASKS` P0.2）：
//! 只有 2 个具备「顶层自由函数 + serde 类型参数」，可直接接线：
//!
//! | 能力 | 执行器 | 输入 schema |
//! |---|---|---|
//! | `foreign_trade_full_cycle` | `execute_trade_full_cycle(TradeContext)` | `TradeContext`（`Serialize+Deserialize`） |
//! | `trade_quote_negotiation` | `execute_quote_negotiation(..)` | `RequirementConfirmation` + `ProductSpec` + `MarketEnvironment` |
//!
//! 其余 3 个（`production_logistics` / `finance_compliance` / `trade_product_spec`）
//! **顶层执行器数为 0**（只有 engine 方法 / 只是知识包工厂）
//! ⇒ 接线它们必须**发明**输入 schema ⇒ **不注册**，保持 fail-closed。
//!
//! # ⛔ 本文件不含任何发明的 schema
//!
//! 输入 schema **就是 Rust 类型本身**（serde 自描述）。
//! 测试的合法输入也**取自仓内** `full_cycle::capability_spec().context`，
//! 而非手写 JSON ⇒ 连测试都不引入虚构数据。

use nt_core_capability_tree::dispatch::{self, BoxFuture};
use serde_json::Value;

use super::full_cycle::{self, TradeContext};
use super::quote_negotiation::RequirementConfirmation;

/// 树 id 常量：打点与注册**必须用同一个串**，故只在此定义。
const _TREE_ID_FULL_CYCLE: &str = "NT-MIND::trade::foreign_trade_full_cycle";
const _TREE_ID_QUOTE: &str = "NT-MIND::trade::trade_quote_negotiation";

/// 派发 `foreign_trade_full_cycle`：输入 = `TradeContext`。
fn dispatch_full_cycle(_id: &str, input: Value) -> BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        // ⛔ 反序列化失败 ⇒ 明确 Err（fail-closed），绝不「填空跑一次」
        let ctx: TradeContext = serde_json::from_value(input)
            .map_err(|e| format!("TradeContext 反序列化失败: {e}"))?;
        let result = full_cycle::execute_trade_full_cycle(ctx);
        // ✅ **执行成功后**才打点（金丝雀必须度量「真被执行」，不是「被查过」）
        neotrix_neobot::nt_capability_canary::signal(_TREE_ID_FULL_CYCLE);
        serde_json::to_value(result).map_err(|e| format!("TradeResult 序列化失败: {e}"))
    })
}

/// 派发 `trade_quote_negotiation`：输入 = `{requirement, product_spec, market_env}`。
fn dispatch_quote_negotiation(_id: &str, input: Value) -> BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        #[derive(serde::Deserialize)]
        struct Args {
            requirement: RequirementConfirmation,
            product_spec: full_cycle::ProductSpec,
            market_env: full_cycle::MarketEnvironment,
        }
        let a: Args = serde_json::from_value(input)
            .map_err(|e| format!("报价参数反序列化失败: {e}"))?;
        neotrix_neobot::nt_capability_canary::signal(_TREE_ID_QUOTE);
        let (quotes, records) = super::quote_negotiation::execute_quote_negotiation(
            a.requirement,
            a.product_spec,
            a.market_env,
        );
        serde_json::to_value(serde_json::json!({ "quotes": quotes, "negotiations": records }))
            .map_err(|e| format!("报价结果序列化失败: {e}"))
    })
}

/// 注册全部**已具备权威 schema** 的贸易能力派发器。
///
///幂等：同名覆盖（`dispatch` 表以 `insert` 语义登记）。
/// 未在此注册的能力 ⇒ `dispatch` 返 `None` ⇒ 调用方 **fail-closed**。
/// 返回失败项（成功时为空）。
///
/// ⚠️ **不吞错误**：派发表锁投毒等失败必须能被调用方看见，
///否则「注册失败」会静默退化成「永远 fail-closed」，正是本轮在治的病。
pub fn register_tree_dispatchers() -> Vec<String> {
    let mut failed = Vec::new();
    // 显式标注为 DispatchFn 指针类型：否则数组被推断为首元素的**具体 fn item 类型**
    let pairs: [(&str, dispatch::DispatchFn); 2] = [
        (_TREE_ID_FULL_CYCLE, dispatch_full_cycle),
        (_TREE_ID_QUOTE, dispatch_quote_negotiation),
    ];
    for (id, f) in pairs {
        if let Err(e) = dispatch::register_dispatcher(id, f) {
            failed.push(format!("{id}: {e}"));
        }
    }
    failed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ 金丝雀是**进程级全局**，而测试**并行**执行
    /// ⇒ 两个 canary 测试会互相 `reset()`，实测 `--test-threads=1` 时
    /// 6 绿、并行时必红（与 `dispatch` 派发表同类问题）。
    static CANARY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    use nt_core_capability_tree::dispatch::{dispatch, registered_count};

    /// 注册后，这两个 id 必须**从 `None` 变成`Some`** —— 这是接线生效的判据。
    #[test]
    fn 注册后两个能力可被派发() {
        let failed = register_tree_dispatchers();
        assert!(failed.is_empty(), "注册不应失败: {failed:?}");
        assert!(registered_count() >= 2);
        for id in [
            "NT-MIND::trade::foreign_trade_full_cycle",
            "NT-MIND::trade::trade_quote_negotiation",
        ] {
            assert!(
                dispatch(id, Value::Null).is_ok_and(|o| o.is_some()),
                "id {id} 注册后应可派发"
            );
        }
    }

    /// ⭐ 真实执行闭环：**输入取自仓内** `capability_spec().context`
    /// ⇒ 测试不引入任何虚构 schema。
    #[test]
    fn 合法输入真实执行并产出结构化结果() {
        let failed = register_tree_dispatchers();
        assert!(failed.is_empty(), "{failed:?}");
        let input =
            serde_json::to_value(full_cycle::capability_spec().context).expect("仓内 spec 转 JSON");
        let fut = dispatch("NT-MIND::trade::foreign_trade_full_cycle", input)
            .expect("派发表可用")
            .expect("已注册");
        let out = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(fut)
            .expect("执行应成功");
        // ⛔ 不臆断业务字段：只断言「产出了可序列化的结构化结果」
        assert!(out.is_object(), "结果应为 JSON 对象，实得 {out}");
    }

    /// 非法输入必须 **Err**（fail-closed），⛔ 绝不「填空蒙混一次成功」。
    #[test]
    fn 非法输入明确失败而非空跑成功() {
        let failed = register_tree_dispatchers();
        assert!(failed.is_empty(), "{failed:?}");
        let fut = dispatch(
            "NT-MIND::trade::foreign_trade_full_cycle",
            serde_json::json!({ "这不是 TradeContext": true }),
        )
        .expect("派发表可用")
        .expect("已注册");
        let got = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(fut);
        assert!(
            got.is_err(),
            "非法输入必须 Err（fail-closed），实得 {got:?}"
        );
    }

    /// ⭐ **可观测性闭环**：真实执行后金丝雀的 `fired` 必须 +1。
    ///
    /// 这是「执行确实发生了」的**外部可观测证据** ——
    /// 之前只有「派发返回 Some」这种自证，无第三方观测。
    #[test]
    fn 真实执行后金丝雀必须打点() {
        use neotrix_neobot::nt_capability_canary as cn;
        // ⚠️ 必须先注册派发器，否则 dispatch 返 None（我第一次就漏了这行，
        //    症状是 expect("已注册")  panic —— 错误信息完全指错了方向）
        let failed = register_tree_dispatchers();
        assert!(failed.is_empty(), "{failed:?}");
        let _cg = CANARY_LOCK.lock().expect("金丝雀测试锁");
        cn::reset();
        cn::expect(cn::CanaryCapability {
            id: _TREE_ID_FULL_CYCLE.to_owned(),
            description: "外贸全链".to_owned(),
            fix: "在 tree_dispatch 执行后 signal".to_owned(),
        })
        .expect("登记金丝雀");
        let before = cn::status()
            .expect("status")
            .into_iter()
            .find(|c| c.capability.id == _TREE_ID_FULL_CYCLE)
            .map(|c| c.fired_count)
            .unwrap_or(0);

        let input =
            serde_json::to_value(full_cycle::capability_spec().context).expect("仓内 spec 转 JSON");
        let fut = dispatch(_TREE_ID_FULL_CYCLE, input)
            .expect("表可用")
            .expect("已注册");
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(fut)
            .expect("执行应成功");

        let after = cn::status()
            .expect("status")
            .into_iter()
            .find(|c| c.capability.id == _TREE_ID_FULL_CYCLE)
            .map(|c| c.fired_count)
            .unwrap_or(0);
        // 用 `>` 而非 `== before+1`：其他测试经`TradeCapabilityRegistry::get()`
        //也会 signal（金丝雀是全局的），并行时可能多出计数。
        // 但**必须严格增长** —— 那才是「本次执行确实被打点」的证据。
        assert!(
            after > before,
            "真实执行后金丝雀 fired 必须增长（{before} → {after}）"
        );
    }

    /// ⛔ 反向证据：**非法输入（未真正执行）不得打点**。
    ///
    /// 若「解析失败」也算打点 ⇒ 金丝雀会奖励「失败的能力」。
    #[test]
    fn 执行失败不得打点() {
        use neotrix_neobot::nt_capability_canary as cn;
        // ⚠️ 必须先注册派发器，否则 dispatch 返 None（我第一次就漏了这行，
        //    症状是 expect("已注册")  panic —— 错误信息完全指错了方向）
        let failed = register_tree_dispatchers();
        assert!(failed.is_empty(), "{failed:?}");
        let _cg = CANARY_LOCK.lock().expect("金丝雀测试锁");
        cn::reset();
        cn::expect(cn::CanaryCapability {
            id: _TREE_ID_FULL_CYCLE.to_owned(),
            description: "外贸全链".to_owned(),
            fix: "在 tree_dispatch 执行后 signal".to_owned(),
        })
        .expect("登记金丝雀");
        let before = cn::status()
            .expect("status")
            .into_iter()
            .find(|c| c.capability.id == _TREE_ID_FULL_CYCLE)
            .map(|c| c.fired_count)
            .unwrap_or(0);

        let fut = dispatch(_TREE_ID_FULL_CYCLE, serde_json::json!({ "错的": 1 }))
            .expect("表可用")
            .expect("已注册");
        let got = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(fut);
        assert!(got.is_err(), "非法输入必须失败");
        let after = cn::status()
            .expect("status")
            .into_iter()
            .find(|c| c.capability.id == _TREE_ID_FULL_CYCLE)
            .map(|c| c.fired_count)
            .unwrap_or(0);
        assert_eq!(after, before, "⛔ 执行失败不得打点，否则金丝雀奖励失败");
    }

    /// 未注册的 3 个能力必须仍然返 `None` ⇒ **不得被误接线**。
    #[test]
    fn 未注册的能力仍fail_closed() {
        register_tree_dispatchers();
        for id in [
            "NT-MEMORY::trade::trade_product_spec",
            "NT-MIND::trade::trade_production_logistics",
            "NT-MIND::trade::trade_finance_compliance",
        ] {
            assert!(
                dispatch(id, Value::Null).expect("表可用").is_none(),
                "id {id} 无权威 schema ⇒ 必须保持 None（fail-closed）"
            );
        }
    }
}
