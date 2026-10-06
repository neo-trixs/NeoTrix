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

/// 派发 `foreign_trade_full_cycle`：输入 = `TradeContext`。
fn dispatch_full_cycle(_id: &str, input: Value) -> BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        // ⛔ 反序列化失败 ⇒ 明确 Err（fail-closed），绝不「填空跑一次」
        let ctx: TradeContext = serde_json::from_value(input)
            .map_err(|e| format!("TradeContext 反序列化失败: {e}"))?;
        let result = full_cycle::execute_trade_full_cycle(ctx);
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
        ("NT-MIND::trade::foreign_trade_full_cycle", dispatch_full_cycle),
        (
            "NT-MIND::trade::trade_quote_negotiation",
            dispatch_quote_negotiation,
        ),
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
