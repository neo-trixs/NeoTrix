// L6 → L5 评测闸门工厂实现（B5b，2026-09-29）
//
// ## 为什么在 L6 而不是 L5
//
// `EvalHarness::new_default` 需要 `Vec<ModelSpec>` / `Vec<DatasetSpec>` /
// `Arc<dyn LlmProvider>` —— 全是 L6/L1 的具体类型。
// L5 若自己 new，会同时踩两条线：
//   ① `l5_cognition/traits.rs:133-139` 明令禁止 `use crate::l6_meta::*`
//   ② L5 无从取得 provider（L1 的 `create_provider_from_type` 虽在向下方向，
//      但把它接进 brain 启动路径等于让 L5 承担 L6 的装配知识）
//
// ⇒ 本文件在 L6 内完成全部装配，只通过 trait 把 `Box<dyn>` 交出去。
//
// ## 默认装配是「空数据集 + 单基线」
//
// ⛔ 这不是完整评测配置，而是一个**可运行的骨架**：
//    - `datasets` 为空 ⇒ `EvalHarness::run()` 返回空 Vec，不烧 token
//    - `baselines` 留一个占位 ModelSpec，`run_regression_test`
//      （真正被 `seal_loop` 调用的方法）**不依赖** baselines/datasets
//
// 理由：`seal_loop.rs` 的闭环钩子只用
// `generate_regression_test` + `run_regression_test` 两个方法，
// 而这两个是**纯确定性**的（`nt_regression.rs`，无 provider 调用）。
// ⇒ 骨架已足够让闸门真正跑起来，而 `run()` 的能力评测留给后续配置。
//
// ## 诚实声明
//
// ⚠️ **本工厂返回的 harness 只覆盖「确定性回归」这一类判定**。
//    它**不能**回答「这次进化让模型整体变好了吗」—— 那需要
//    `nt_evolution_eval`（臂中立 A/B + 噪声地板）且需要 ≥2 次重复运行。
//    ⛔ 不要因为「闸门接上了」就认为「进化已被验证」。
//    详见 docs/architecture/DECISION-D3-EVOLUTION-EVAL.md §6。

use std::sync::Arc;

use super::nt_types::{DatasetSpec, ModelSpec};
use super::EvalHarness;
use crate::l1_action::nt_io::nt_io_provider::{
    create_provider_from_type, LlmProviderType,
};
use crate::l5_cognition::traits::{EvalHarnessApi, EvalHarnessFactory};

/// 默认工厂。
///
/// 零配置即可构造：**不要求任何环境变量、不要求 API key**。
/// 理由：闸门缺失会让自进化闭环**静默退化为无验证**
/// （见 `seal_loop.rs` 的「未注入 ⇒ 回归闸门**未运行**」日志），
/// 那比「有一个只覆盖确定性判定的弱闸门」更危险。
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultEvalHarnessFactory;

impl DefaultEvalHarnessFactory {
    /// 占位基线：不参与 `run_regression_test`，但 `EvalHarness` 结构要求非空语义。
    ///
    /// ⚠️ `provider_type: "none"` 是**故意的**：L6 的 `build_provider`
    ///    对未知类型返回 `ConfigError`，而本工厂**从不调用**它
    ///    （闭环钩子只走两个纯函数方法）。留一个显式不可用的值，
    ///    比填一个看起来能用、实际会在缺 key 时静默降级的值更诚实。
    fn placeholder_baseline() -> Vec<ModelSpec> {
        vec![ModelSpec {
            name: "regression-gate-placeholder".into(),
            provider_type: "none".into(),
            model_id: "none".into(),
            base_url: None,
            api_key_env: None,
            pricing_per_1m_in: 0.0,
            pricing_per_1m_out: 0.0,
        }]
    }

    /// 判官 provider：仅在配置了 `NT_EVAL_JUDGE_PROVIDER` 时构造。
    ///
    /// ⛔ **未配置则返回 `None`**，不 fallback 到某个默认厂商 ——
    /// fallback 会让「没配 judge」看起来像「配了且跑过了」。
    fn judge_provider() -> Option<Arc<dyn crate::l1_action::nt_io::nt_io_provider::LlmProvider>>
    {
        let name = std::env::var("NT_EVAL_JUDGE_PROVIDER").ok()?;
        let t = LlmProviderType::from_name(&name)?;
        let key = std::env::var("NT_EVAL_JUDGE_API_KEY").ok();
        Some(Arc::from(create_provider_from_type(t, key)))
    }
}

impl EvalHarnessFactory for DefaultEvalHarnessFactory {
    fn make_eval_harness(&self) -> Option<Box<dyn EvalHarnessApi>> {
        // judge provider 缺失**不阻断**构造：闭环钩子只用两个纯函数方法。
        // 但显式记录，因为「有 judge」与「无 judge」的 harness 能力不同。
        let (judge_provider, judge_model) = match Self::judge_provider() {
            Some(p) => (p, std::env::var("NT_EVAL_JUDGE_MODEL").unwrap_or_else(|_| "none".into())),
            None => (
                // 无 judge 配置 ⇒ 用 Ollama（**本地 provider，无需 API key**）。
                // ⛔ 不用「随便挑个云厂商」：那会在没配 key 时静默降级成别的模型，
                //    让「没配 judge」看起来像「配了且跑过了」。
                // 且闭环钩子只用两个纯函数方法，此 provider 实际不会被调用。
                Arc::from(create_provider_from_type(
                    LlmProviderType::Ollama,
                    None,
                )) as Arc<dyn crate::l1_action::nt_io::nt_io_provider::LlmProvider>,
                "none".into(),
            ),
        };

        let harness = EvalHarness::new_default(
            Self::placeholder_baseline(),
            Vec::<DatasetSpec>::new(),
            judge_provider,
            judge_model,
        );
        Some(Box::new(harness))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::traits::EvalHarnessApi;

    /// ⛔ **本测试是整个 B5b 的非空门证明。**
    ///
    /// A9 之前，`seal_loop` 的闭环钩子硬传 `None`，而 `EvalHarnessApi` 的
    /// impl 早已存在 ⇒ **实现了但永远收不到调用**，效果上等于没有闸门。
    ///
    /// ⛔ 只断言「工厂不返回 None」**不够** —— 那只证明构造函数跑完了，
    /// 不证明产出的对象真能用。故本测试走完 `seal_loop` 实际调用的
    /// **那两个方法**（`generate_regression_test` + `run_regression_test`），
    /// 并断言产出的 case 与 verdict 是**有意义**的。
    #[test]
    fn factory_produces_a_usable_gate() {
        let factory = DefaultEvalHarnessFactory;
        let harness = factory
            .make_eval_harness()
            .expect("零配置也必须能构造闸门（否则闭环会静默退化为无验证）");

        // ① 能从候选产出回归用例
        let case = harness.generate_regression_test("T6 removed the memory dedup path");
        assert!(!case.id.is_empty(), "回归用例必须有 id");
        assert!(!case.candidate.is_empty(), "必须回填候选内容");

        // ② 能跑出裁决
        let result = harness.run_regression_test(&case);
        // ⛔ **不断言 passed == true**：回归用例是**从候选反推**的，
        //   候选里含 "removed" 这类动作词时判失败是合理行为。
        //   断言的只是「裁决机制真的跑了」——有 verdict 结构可读。
        let _ = result.passed;
        // 失败的裁决必须给出可读的 reason，否则调用方无从行动
        if !result.passed {
            assert!(
                !result.reasons.is_empty(),
                "失败的回归裁决必须带 reason，否则 seal_loop 的告警无信息量"
            );
        }
    }

    /// 闸门必须**可跨线程共享** —— `SelfIteratingBrain` 实现了
    /// `BrainHandle: Send + Sync`，字段里存着 `Box<dyn EvalHarnessApi>`。
    /// 若工厂产出不可 Send 的对象，会在装进 brain 时编译失败。
    #[test]
    fn gate_is_send_and_sync() {
        // ⛔ 用 `&dyn Trait` 而不是 `dyn Trait`：后者是 unsized，
        //   进不了泛型参数（E0277，实测踩到）。
        //   `&T where T: Send + Sync` 能成立，正说明 pointee 满足约束。
        fn assert_send_sync<T: Send + Sync + ?Sized>(_: &T) {}
        let harness: Box<dyn EvalHarnessApi> = DefaultEvalHarnessFactory
            .make_eval_harness()
            .expect("构造必须成功");
        assert_send_sync(&*harness);
    }

    /// 零配置下**不得 panic** —— 工厂在启动路径上被调用。
    ///
    /// ⛔ 特别检查 `judge_provider()`：未设 `NT_EVAL_JUDGE_PROVIDER` 时
    ///   必须走 `None` 分支而不是 `unwrap()`。
    #[test]
    fn zero_config_never_panics() {
        std::env::remove_var("NT_EVAL_JUDGE_PROVIDER");
        std::env::remove_var("NT_EVAL_JUDGE_MODEL");
        std::env::remove_var("NT_EVAL_JUDGE_API_KEY");
        let harness = DefaultEvalHarnessFactory.make_eval_harness();
        assert!(harness.is_some(), "零配置也必须造出闸门（弱闸门优于无闸门）");
    }
}
