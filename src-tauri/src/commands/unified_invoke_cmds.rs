//! Unified Invoke — 极简统一 API 门面
//!
//! 把后端 300+ 同步 Tauri 命令压缩为一个入口 `unified_invoke(module, action, args)`,
//! 前端只依赖这一条 invoke 即可调用全部聚合能力, 保留全部特性, 前端 API 面最小化。
//!
//! - module: 模块名（如 "mcp_host"、"background"、"voice"、"workflow"）
//! - action: 动作名（如 "list_endpoints"、"pause_background_task"）
//! - args: 具名 JSON 对象 { param: value }
//!
//! 分发由 `include!("_gate_table.rs")` 的 `dispatch()` 函数完成 ——
//! 编译期静态 match, 零反射、零装箱。参数经下方 helper 从 `&Value` 提取。

use serde_json::{json, Value};

use super::*;

/* ══════════ 参数提取 helpers（供 _gate_table.rs 内联调用）══════════ */

#[inline]
#[allow(dead_code)]
fn gstr(v: Option<&Value>) -> String {
    v.map(|x| match x {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }).unwrap_or_default()
}
#[inline]
#[allow(dead_code)]
fn goptstr(v: Option<&Value>) -> Option<String> {
    v.filter(|x| !x.is_null()).map(|x| match x {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    })
}
#[inline]
#[allow(dead_code)]
fn gus(v: Option<&Value>) -> usize {
    v.and_then(Value::as_u64).unwrap_or(0) as usize
}
#[inline]
#[allow(dead_code)]
fn gu32(v: Option<&Value>) -> u32 {
    v.and_then(Value::as_u64).unwrap_or(0) as u32
}
#[inline]
#[allow(dead_code)]
fn gu16(v: Option<&Value>) -> u16 {
    v.and_then(Value::as_u64).unwrap_or(0) as u16
}
#[inline]
#[allow(dead_code)]
fn gu8(v: Option<&Value>) -> u8 {
    v.and_then(Value::as_u64).unwrap_or(0) as u8
}
#[inline]
#[allow(dead_code)]
fn gf64(v: Option<&Value>) -> f64 {
    v.and_then(Value::as_f64).unwrap_or(0.0)
}
#[inline]
#[allow(dead_code)]
fn gb(v: Option<&Value>) -> bool {
    v.and_then(Value::as_bool).unwrap_or(false)
}
#[inline]
#[allow(dead_code)]
fn gob(v: Option<&Value>) -> Option<bool> {
    v.filter(|x| !x.is_null()).map(|x| x.as_bool().unwrap_or(false))
}
#[inline]
#[allow(dead_code)]
fn gopt_u32(v: Option<&Value>) -> Option<u32> {
    v.filter(|x| !x.is_null()).map(|x| x.as_u64().unwrap_or(0) as u32)
}
#[inline]
#[allow(dead_code)]
fn gopt_usize(v: Option<&Value>) -> Option<usize> {
    v.filter(|x| !x.is_null()).map(|x| x.as_u64().unwrap_or(0) as usize)
}
#[inline]
#[allow(dead_code)]
fn gopt_u64(v: Option<&Value>) -> Option<u64> {
    v.filter(|x| !x.is_null()).and_then(Value::as_u64)
}
#[inline]
#[allow(dead_code)]
fn gvec(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
}
#[inline]
#[allow(dead_code)]
fn gopt_vec(v: Option<&Value>) -> Option<Vec<String>> {
    if v.is_none_or(|x| x.is_null()) {
        None
    } else {
        Some(gvec(v))
    }
}
#[inline]
#[allow(dead_code)]
fn gjson(v: Option<&Value>) -> Value {
    v.cloned().unwrap_or(Value::Null)
}
#[inline]
#[allow(dead_code)]
fn gopt_json(v: Option<&Value>) -> Option<Value> {
    v.filter(|x| !x.is_null()).cloned()
}

/* ══════════════ 统一分发的 Tauri 命令 — 前端唯一依赖 ══════════════ */

/// 统一分发: `unified_invoke("mcp_host", "list_endpoints", {})`
///
/// 前端仅需此一个 invoke 即可触达后端聚合能力（MCP / agent / voice / workflow 等）。
#[tauri::command]
pub fn unified_invoke(
    module: String,
    action: String,
    args: Value,
) -> Result<Value, String> {
    dispatch(&module, &action, &args)
}

/// 列出统一门面目录（自动生成前端能力清单）
#[tauri::command]
pub fn unified_invoke_catalog() -> Result<Vec<Value>, String> {
    Ok(include!("_catalog_table.rs"))
}

/* ══════════════ 静态 match 分发（由 gate_gen.py 生成）══════════════ */

fn dispatch(module: &str, action: &str, args: &Value) -> Result<Value, String> {
    let _ = args;
    include!("_gate_table.rs")
}