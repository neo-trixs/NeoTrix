//! 智慧模块桥接 — P4 能力挂入 NativeBus (P4.4 接线层)。
//!
//! 调用契约:
//! - `wisdom.extract_meaning`:  {"text":"string"} → {"units":[...]}
//! - `wisdom.detect_causal`:    {"text":"string"} → {cause,effect} | null
//! - `wisdom.paradigm_observe`: {"domain","description","severity"} → {ok:true}
//! - `wisdom.paradigm_detect`:  {} → [hypotheses]

use crate::core::l7_capability::native_bus::{closure_capability, NativeBus};
use crate::core::nt_core_meaning::{MeaningConstructor, MeaningContext};
use crate::core::nt_core_paradigm::{Anomaly, ParadigmShiftDetector};
use serde_json::json;
use std::sync::{Arc, RwLock};

struct WisdomState {
    meaning: MeaningConstructor,
    paradigm: ParadigmShiftDetector,
}

/// 注册全部智慧能力进总线。
pub fn register_wisdom_capabilities(bus: &mut NativeBus) -> Result<usize, String> {
    let state = Arc::new(RwLock::new(WisdomState {
        meaning: MeaningConstructor::new(),
        paradigm: ParadigmShiftDetector::new(3),
    }));
    let mut count = 0usize;

    // wisdom.extract_meaning
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.extract_meaning", "意义建构提取",
            r#"{"text":"string"}"#, r#"{"units":[...]}"#, true,
            move |input| {
                let text = input.get("text").and_then(|v| v.as_str()).ok_or("missing text")?;
                let ctx: MeaningContext = serde_json::from_value(
                    input.get("context").cloned().unwrap_or(json!({}))).unwrap_or_default();
                let mut w = st.write().map_err(|e| e.to_string())?;
                let units = w.meaning.extract_meaning(text, &ctx)?;
                Ok(json!({ "units": units }))
            },
        ))?;
        count += 1;
    }

    // wisdom.detect_causal
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.detect_causal", "因果链检测",
            r#"{"text":"string"}"#, r#"{cause,effect}|null"#, true,
            move |input| {
                let text = input.get("text").and_then(|v| v.as_str()).ok_or("missing text")?;
                let w = st.read().map_err(|e| e.to_string())?;
                match w.meaning.detect_causal_chain(text) {
                    Some((c, e)) => Ok(json!({ "cause": c, "effect": e })),
                    None => Ok(serde_json::Value::Null),
                }
            },
        ))?;
        count += 1;
    }

    // wisdom.paradigm_observe
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.paradigm_observe", "范式异常观测",
            r#"{"domain":"s","description":"s","severity":f64}"#, r#"{"ok":true}"#, true,
            move |input| {
                let d = input.get("domain").and_then(|v| v.as_str()).ok_or("missing domain")?.to_string();
                let desc = input.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let sev = input.get("severity").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let mut w = st.write().map_err(|e| e.to_string())?;
                w.paradigm.observe(Anomaly { domain: d, description: desc, severity: sev, confidence: 0.8 });
                Ok(json!({ "ok": true }))
            },
        ))?;
        count += 1;
    }

    // wisdom.paradigm_detect
    {
        let st = state.clone();
        bus.register(closure_capability(
            "wisdom.paradigm_detect", "范式迁移检测",
            "{}", r#"[...]"#, true,
            move |_input| {
                let w = st.read().map_err(|e| e.to_string())?;
                Ok(json!(w.paradigm.detect()))
            },
        ))?;
        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_capabilities() {
        let mut bus = NativeBus::new();
        let n = register_wisdom_capabilities(&mut bus).unwrap();
        assert_eq!(n, 4);
        assert!(bus.has("wisdom.extract_meaning"));
        assert!(bus.has("wisdom.detect_causal"));
        assert!(bus.has("wisdom.paradigm_observe"));
        assert!(bus.has("wisdom.paradigm_detect"));
    }

    #[test]
    fn test_causal_via_bus() {
        let mut bus = NativeBus::new();
        register_wisdom_capabilities(&mut bus).unwrap();
        let out = bus.dispatch("wisdom.detect_causal", "t",
            json!({ "text": "因为下雨所以地湿" })).unwrap();
        assert_eq!(out["cause"], "因为下雨");
        assert_eq!(out["effect"], "地湿");
    }

    #[test]
    fn test_paradigm_chain() {
        let mut bus = NativeBus::new();
        register_wisdom_capabilities(&mut bus).unwrap();
        for d in ["physics", "bio", "code"] {
            bus.dispatch("wisdom.paradigm_observe", "t",
                json!({ "domain": d, "description": "x", "severity": 0.9 })).unwrap();
        }
        let out = bus.dispatch("wisdom.paradigm_detect", "t", json!({})).unwrap();
        assert!(!out.as_array().unwrap().is_empty());
    }
}
