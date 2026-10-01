//! 效果键 —— 同一意图的重复提交收敛到同一个 id。
//!
//! # 出处与许可（Apache-2.0 §4 要求写明）
//!
//! 移植自 rakazo `packages/core/src/approval-effect-key.ts`
//! （Copyright 2026 Rakazo contributors, Apache License 2.0）。
//! 上游语义：`{runId}:{toolName}:{sha256(稳定 JSON(args))}` ——
//! 同一轮、同一工具、同一参数算出同一个键，重试不会留下两条待办。
//!
//! # 本仓的偏离（有意）
//!
//! ① 上游对任意 JS 值做稳定序列化（含循环/NaN/稀疏数组的拒绝逻辑）。
//!    Rust 的 `serde_json::Value` 根本装不下那些东西（循环不可构造，
//!    `f64::NAN` 进 `Value` 会变成 Null），故「非法值拒绝」类坍缩为一条：
//!    非有限浮点在进入 `Value` 前就必须拦（`reject_non_finite`，单元测试锁）。
//! ② scope/tool 禁 `:` —— `a:b:c` 无法区分「scope 含冒号」与「tool 含冒号」
//!    的两种切法，上游是模板拼接，同样有歧义，这里直接拒掉。
//! ③ 本仓用途是补发去重（`deliver_result`），不是审批去重 ——
//!    同一任务同一文本重试只留一行，文本变了就是新意图，键自然不同。
//!
//! # 不能拿它干什么
//!
//! 聊天消息去重（"谢谢" 发两次是两句话）。效果键只用于**机器重试**路径
//! （补发/审批/幂等执行），永不用于用户原语。

use serde_json::Value;
use sha2::{Digest, Sha256};

/// 递归规范 JSON：对象键排序，其余原样。`Value` 无循环/函数/undefined，
///
/// 故规范化是全函数 —— 不存在上游要拒绝的那几类输入。
/// 唯一例外：非有限浮点在 `Value` 里以 Null 现身（serde_json 的选择），
///
/// 调用方若传了 NaN/Inf 进来，这里无法区分「真 Null」与「塌缩的 NaN」，
/// 故要求调用方在构造 `Value` 前自查；本模块另供 `reject_non_finite`
/// 给 `f64` 调用方用（测试锁死）。
fn canonical(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_owned()),
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(canonical).collect();
            format!("[{}]", parts.join(","))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .iter()
                .map(|k| format!("{}:{}", serde_json::to_string(k).unwrap_or_default(), canonical(&map[*k])))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

/// `f64` 调用方的前置检查：非有限值进不了 `Value`（会塌成 Null），
///
/// 在这里拦比在下游对着 Null 猜要早。返回 `Err` 即「别记，数据已坏」。
pub fn reject_non_finite(value: f64) -> Result<f64, String> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err("effect key args must not contain non-finite floats (they collapse to null)".to_owned())
    }
}

/// 效果键：`{scope}:{tool}:{sha256hex(canonical(args))}`。
pub fn effect_key(scope: &str, tool: &str, args: &Value) -> Result<String, String> {
    if scope.is_empty() || tool.is_empty() {
        return Err("effect key scope and tool must be non-empty".to_owned());
    }
    if scope.contains(':') || tool.contains(':') {
        return Err("effect key scope and tool must not contain ':'".to_owned());
    }
    let digest = Sha256::digest(canonical(args).as_bytes());
    Ok(format!("{scope}:{tool}:{digest:x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn 键序不影响键() {
        let a = json!({"x": 1, "y": [true, null, "s"]});
        let b = json!({"y": [true, null, "s"], "x": 1});
        assert_eq!(
            effect_key("s", "t", &a).expect("键"),
            effect_key("s", "t", &b).expect("键")
        );
    }

    #[test]
    fn 参数不同键不同_嵌套亦然() {
        let base = effect_key("s", "t", &json!({"a": {"b": 1}})).expect("键");
        assert_ne!(base, effect_key("s", "t", &json!({"a": {"b": 2}})).expect("键"));
        assert_ne!(base, effect_key("s", "t", &json!({"a": {"b": 1}, "c": 0})).expect("键"));
    }

    #[test]
    fn 域与工具参与键且冒号被拒() {
        let k1 = effect_key("run1", "send", &json!({"a": 1})).expect("键");
        let k2 = effect_key("run2", "send", &json!({"a": 1})).expect("键");
        assert_ne!(k1, k2);
        assert!(effect_key("", "t", &json!({})).is_err());
        assert!(effect_key("a:b", "t", &json!({})).is_err());
        assert!(effect_key("s", "t:t", &json!({})).is_err());
    }

    #[test]
    fn 键形如三段且摘要64hex() {
        let k = effect_key("task-1", "deliver_result", &json!({"chat": "42"})).expect("键");
        let parts: Vec<&str> = k.split(':').collect();
        assert_eq!(&parts[..2], ["task-1", "deliver_result"]);
        assert_eq!(parts[2].len(), 64);
        assert!(parts[2].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn 非有限浮点在入口被拒() {
        assert!(reject_non_finite(1.5).is_ok());
        assert!(reject_non_finite(f64::NAN).is_err());
        assert!(reject_non_finite(f64::INFINITY).is_err());
    }

    #[test]
    fn 跨进程稳定_黄金向量() {
        // 锁定实现：改规范化即改全部历史键，测试先红。
        // canonical({"b":2,"a":1}) == {"a":1,"b":2} 的 sha256
        // （`printf '%s' '{"a":1,"b":2}' | shasum -a 256` 独立复算）。
        let k = effect_key("s", "t", &json!({"b": 2, "a": 1})).expect("键");
        assert_eq!(k, "s:t:43258cff783fe7036d8a43033f830adfc60ec037382473548ac742b888292777");
    }
}
