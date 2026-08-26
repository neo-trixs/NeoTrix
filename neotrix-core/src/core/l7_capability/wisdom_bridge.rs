//! 智慧模块桥接 — P4 三模块挂入 NativeBus (P4.4 接线层)。
//!
//! 将 DaoEngine(符号回归)、MeaningConstructor(意义建构)、ParadigmShiftDetector(范式检测)
//! 注册为 NativeCapability，意识体通过 `bus.dispatch("wisdom.xxx", caller, input)` 统一调用。
//!
//! 调用契约:
//! - `wisdom.regression_fit`:  {"x": [f64], "y": [f64]} → {"expr": "string"}
//! - `wisdom.extract_meaning`: {"text": "string"} → {"units": [{id,pattern,confidence}]}
//! - `wisdom.detect_causal`:   {"text": "string"} → {"cause": "s", "effect": "s"} | null
//! - `wisdom.paradigm_observe`:{"domain":"s","description":"s","severity":f64} → {"ok":true}
//! - `wisdom.paradigm_detect`: {} → [{"description":"s","cross_domain":b,"novelty":f64}]

use crate::core::l7_capability::native_bus::{closure_capability, NativeBus};
use crate::core::nt_core_dao_engine::{DaoEngine, DaoEngineConfig};
use crate::core::nt_core_meaning::{MeaningConstructor, MeaningContext};
use crate::core::nt_core_paradigm::{Anomaly, ParadigmShiftDetector};
use serde_json::json;
use std::sync::{Arc, RwLock};

/// 智慧模块共享状态 — 所有可变内部状态集中在此。
struct WisdomState {
    dao: DaoEngine,
    meaning: MeaningConstructor,
    paradigm: ParadigmShiftDetector,
}

/// 将全部智慧模块注册进总线（意识体启动时调用一次）。
///
/// 返回成功注册的能力数。id 冲突 fail-loud (R-P110)。
pub fn register_wisdom_capabilities(bus: &mut NativeBus) -> Result<usize, String> {
    let state = Arc::new(RwLock::new(WisdomState {
        dao: DaoEngine::new(DaoEngineConfig::default()),
        meaning: MeaningConstructor::new(),
        paradigm: ParadigmShiftDetector::new(3),
    }));

    let mut count = 0usize;

    // ── wisdom.regression_fit ──
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.regression_fit",
            "符号回归拟合",
            r#"{"x":[number],"y":[number]}"#,
            r#"{"expr":"string"}"#,
            true,
            move |input| {
                let x: Vec<f64> = serde_json::from_value(input.get("x").cloned().unwrap_or(json!([])))
                    .map_err(|e| format!("x 解析失败: {e}"))?;
                let y: Vec<f64> = serde_json::from_value(input.get("y").cloned().unwrap_or(json!([])))
                    .map_err(|e| format!("y 解析失败: {e}"))?;
                let mut w = st.write().map_err(|e| e.to_string())?;
                let expr = w.dao.fit_linear(&x, &y)?;
                Ok(json!({ "expr": expr }))
            },
        ))?;
        count += 1;
    }

    // ── wisdom.extract_meaning ──
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.extract_meaning",
            "意义建构提取",
            r#"{"text":"string"}"#,
            r#"{"units":[MeaningUnit]}"#,
            true,
            move |input| {
                let text = input.get("text").and_then(|v| v.as_str())
                    .ok_or("缺少 text 字段")?;
                let ctx = serde_json::from_value::<MeaningContext>(
                    input.get("context").cloned().unwrap_or(json!({}))
                ).unwrap_or_default();
                let mut w = st.write().map_err(|e| e.to_string())?;
                let units = w.meaning.extract_meaning(text, &ctx)?;
                Ok(json!({ "units": units }))
            },
        ))?;
        count += 1;
    }

    // ── wisdom.detect_causal ──
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.detect_causal",
            "因果链检测",
            r#"{"text":"string"}"#,
            r#"{"cause":"string","effect":"string"} | null"#,
            true,
            move |input| {
                let text = input.get("text").and_then(|v| v.as_str())
                    .ok_or("缺少 text 字段")?;
                let w = st.read().map_err(|e| e.to_string())?;
                match w.meaning.detect_causal_chain(text) {
                    Some((cause, effect)) => Ok(json!({ "cause": cause, "effect": effect })),
                    None => Ok(serde_json::Value::Null),
                }
            },
        ))?;
        count += 1;
    }

    // ── wisdom.paradigm_observe ──
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.paradigm_observe",
            "范式异常观测",
            r#"{"domain":"string","description":"string","severity":number}"#,
            r#"{"ok":true}"#,
            true,
            move |input| {
                let domain = input.get("domain").and_then(|v| v.as_str())
                    .ok_or("缺少 domain")?.to_string();
                let description = input.get("description").and_then(|v| v.as_str())
                    .unwrap_or("").to_string();
                let severity = input.get("severity").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let mut w = st.write().map_err(|e| e.to_string())?;
                w.paradigm.observe(Anomaly { domain, description, severity });
                Ok(json!({ "ok": true }))
            },
        ))?;
        count += 1;
    }

    // ── wisdom.paradigm_detect ──
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.paradigm_detect",
            "范式迁移检测",
            "{}",
            r#"[ParadigmShiftHypothesis]"#,
            true,
            move |_input| {
                let w = st.read().map_err(|e| e.to_string())?;
                let hypotheses = w.paradigm.detect();
                Ok(json!(hypotheses))
            },
        ))?;
        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l7_capability::native_bus::NativeBus;

    fn make_bus() -> (NativeBus, usize) {
        let mut bus = NativeBus::new();
        let n = register_wisdom_capabilities(&mut bus).unwrap();
        (bus, n)
    }

    #[test]
    fn test_all_five_capabilities_registered() {
        let (bus, n) = make_bus();
        assert_eq!(n, 5, "应注册 5 个智慧能力");
        assert!(bus.has("wisdom.regression_fit"));
        assert!(bus.has("wisdom.extract_meaning"));
        assert!(bus.has("wisdom.detect_causal"));
        assert!(bus.has("wisdom.paradigm_observe"));
        assert!(bus.has("wisdom.paradigm_detect"));
    }

    #[test]
    fn test_regression_fit_via_bus() {
        let (mut bus, _) = make_bus();
        let out = bus
            .dispatch("wisdom.regression_fit", "test", json!({
                "x": [1.0, 2.0, 3.0],
                "y": [3.0, 5.0, 7.0]
            }))
            .unwrap();
        assert!(out["expr"].is_string(), "应返回表达式字符串");
        assert!(out["expr"].as_str().unwrap().contains("x"), "表达式应含变量 x");
    }

    #[test]
    fn test_extract_meaning_via_bus() {
        let (mut bus, _) = make_bus();
        let out = bus
            .dispatch("wisdom.extract_meaning", "test", json!({
                "text": "因为下雨所以地湿"
            }))
            .unwrap();
        assert!(out["units"].is_array(), "应返回意义单元数组");
        assert!(!out["units"].as_array().unwrap().is_empty(), "因果句应有意义单元");
    }

    #[test]
    fn test_detect_causal_via_bus() {
        let (mut bus, _) = make_bus();
        let out = bus
            .dispatch("wisdom.detect_causal", "test", json!({ "text": "因为下雨所以地湿" }))
            .unwrap();
        assert_eq!(out["cause"], "因为下雨");
        assert_eq!(out["effect"], "地湿");

        // 无因果关系文本 → null
        let null_out = bus
            .dispatch("wisdom.detect_causal", "test", json!({ "text": "今天天气不错" }))
            .unwrap();
        assert!(null_out.is_null());
    }

    #[test]
    fn test_paradigm_observe_and_detect_full_chain() {
        let (mut bus, _) = make_bus();

        // 观测 3 个域的异常
        for (domain, desc) in [("physics", "退相干异常"), ("bio", "蛋白质折叠错误"), ("code", "审查通过率骤降")] {
            bus.dispatch("wisdom.paradigm_observe", "test", json!({
                "domain": domain, "description": desc, "severity": 0.9
            })).unwrap();
        }

        // 检测跨域共振
        let out = bus.dispatch("wisdom.paradigm_detect", "test", json!({})).unwrap();
        let arr = out.as_array().expect("应返回假设数组");
        assert!(!arr.is_empty(), "3 域异常应触发范式迁移假设");
        assert_eq!(arr[0]["cross_domain"], true);
    }

    #[test]
    fn test_paradigm_below_threshold_no_hypothesis() {
        let (mut bus, _) = make_bus();
        bus.dispatch("wisdom.paradigm_observe", "t", json!({
            "domain": "physics", "description": "x", "severity": 0.9
        })).unwrap();
        let out = bus.dispatch("wisdom.paradigm_detect", "t", json!({})).unwrap();
        assert!(out.as_array().unwrap().is_empty(), "单域不足阈值不应触发");
    }

    #[test]
    fn test_missing_field_fails_loud() {
        let (mut bus, _) = make_bus();
        let err = bus
            .dispatch("wisdom.regression_fit", "t", json!({"wrong": 1}))
            .unwrap_err();
        assert!(!err.is_empty(), "缺字段应报错而非静默");
    }
}
