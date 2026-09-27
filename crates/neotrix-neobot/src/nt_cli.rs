//! `nt_cli` — `neobot` CLI 即协议.
//!
//! 模型/外部进程只调本地 `neobot` CLI，结构化副作用经 JSONL 行回传，
//! Rust sidecar 落库前复核。模型进程永远拿不到 token/URL.
//!
//! 三条纪律：
//! 1. 坏行只计数不炸（`malformed_line_count`），调用方决定阈值；
//! 2. 数组/包络双形状兼容（裸数组行 + `{sideEffects:[...]}` 行）；
//! 3. 主通道不可用时走 `NEOBOT_RESULT_PATH` 文件回传
//!    （写失败打 `__NEOBOT_SIDE_EFFECTS_WRITE_FAILED__=` 标记到 stderr）。

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 副作用回传文件 env（CLI 子进程写，sidecar 读）。
pub const RESULT_PATH_ENV: &str = "NEOBOT_RESULT_PATH";
/// 写失败标记前缀（stderr 回退通道）。
pub const SIDE_EFFECT_WRITE_FAILED_PREFIX: &str = "__NEOBOT_SIDE_EFFECTS_WRITE_FAILED__=";

/// 一条副作用行.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub kind: String,
    pub payload: serde_json::Value,
}

/// CLI 一次调用的包络（ok + 文本 + 退出码 + 副作用行）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResult {
    pub ok: bool,
    pub text: String,
    pub exit_code: i32,
    #[serde(default)]
    pub side_effects: Vec<SideEffect>,
}

/// 编码一条 JSONL 副作用.
pub fn format_side_effect(kind: &str, payload: &serde_json::Value) -> Result<String, NtBotError> {
    let effect = SideEffect {
        kind: kind.to_owned(),
        payload: payload.clone(),
    };
    Ok(serde_json::to_string(&effect)?)
}

/// 解析一条 JSONL 副作用.
pub fn parse_side_effect(line: &str) -> Result<SideEffect, NtBotError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(NtBotError::Invalid("empty side-effect line".to_owned()));
    }
    Ok(serde_json::from_str(trimmed)?)
}

/// 批量解析 JSONL（整段 stdout）：空行跳过；坏行计数不炸；
/// 裸数组行展平；`{sideEffects:[...]}` 包络行拆包；无 kind 的行丢弃。
/// 返回 `(effects, malformed_line_count)`。
pub fn parse_side_effects_jsonl(raw: &str) -> (Vec<SideEffect>, usize) {
    let mut effects = Vec::new();
    let mut malformed = 0usize;
    for line in raw.split(['\n', '\r']) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // 写失败标记行：载荷在 `=` 之后，抢救解析
        let body = trimmed
            .strip_prefix(SIDE_EFFECT_WRITE_FAILED_PREFIX)
            .unwrap_or(trimmed);
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(value) => effects.extend(normalize_side_effects(&value)),
            Err(_) => malformed += 1,
        }
    }
    (effects, malformed)
}

fn normalize_side_effects(value: &serde_json::Value) -> Vec<SideEffect> {
    if let Some(array) = value.as_array() {
        return array.iter().filter_map(keep_effect).collect();
    }
    // 裸对象行（本协议 `format_side_effect` 形状）即单条 effect。
    if let Some(single) = keep_effect(value) {
        return vec![single];
    }
    if let Some(inner) = value.get("sideEffects") {
        return normalize_side_effects(inner);
    }
    Vec::new()
}

fn keep_effect(value: &serde_json::Value) -> Option<SideEffect> {
    let kind = value.get("kind")?.as_str()?;
    if kind.is_empty() {
        return None;
    }
    Some(SideEffect {
        kind: kind.to_owned(),
        payload: value.get("payload").cloned().unwrap_or(serde_json::Value::Null),
    })
}

/// 往回传文件追加一行包络（子进程侧用；父进程读后删）。
/// 无 env 或 effects 为空 → 静默跳过；写失败 → stderr 标记 + 原错返回。
pub fn append_side_effects_result(
    path: &std::path::Path,
    effects: &[SideEffect],
) -> Result<(), NtBotError> {
    use std::io::Write as _;
    if effects.is_empty() {
        return Ok(());
    }
    let line = serde_json::to_string(&serde_json::json!({ "sideEffects": effects }))?;
    match std::fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            if let Err(err) = writeln!(file, "{line}") {
                eprintln!("{SIDE_EFFECT_WRITE_FAILED_PREFIX}{line}");
                return Err(NtBotError::Io(err.to_string()));
            }
            Ok(())
        }
        Err(err) => {
            eprintln!("{SIDE_EFFECT_WRITE_FAILED_PREFIX}{line}");
            Err(NtBotError::Io(err.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{append_side_effects_result, format_side_effect, parse_side_effect, parse_side_effects_jsonl};

    #[test]
    fn side_effect_roundtrip() {
        let line = format_side_effect("reply", &serde_json::json!({"text": "hi"})).expect("format");
        let effect = parse_side_effect(&line).expect("parse");
        assert_eq!(effect.kind, "reply");
    }

    #[test]
    fn batch_parse_counts_malformed_without_blowing_up() {
        let raw = concat!(
            "{\"kind\":\"reply\",\"payload\":{\"text\":\"hi\"}}\n",
            "\n",
            "[{\"kind\":\"a\",\"payload\":1},{\"no_kind\":true},{\"kind\":\"\",\"payload\":2}]\n",
            "{\"sideEffects\":[{\"kind\":\"b\",\"payload\":null}]}\n",
            "not json at all\n",
        );
        let (effects, malformed) = parse_side_effects_jsonl(raw);
        let kinds: Vec<&str> = effects.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(kinds, vec!["reply", "a", "b"]);
        assert_eq!(malformed, 1);
    }

    #[test]
    fn result_file_roundtrip_and_failed_write_marker() {
        let dir = std::env::temp_dir().join("neobot-cli-result-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("effects.jsonl");
        let _ = std::fs::remove_file(&path);
        let line = format_side_effect("reply", &serde_json::json!({"text": "x"})).expect("format");
        let (effects, _) = parse_side_effects_jsonl(&line);
        append_side_effects_result(&path, &effects).expect("append");
        let raw = std::fs::read_to_string(&path).expect("read");
        let (back, malformed) = parse_side_effects_jsonl(&raw);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].kind, "reply");
        assert_eq!(malformed, 0);
        // 写失败标记行可抢救
        let marked = format!("{}[{{\"kind\":\"m\",\"payload\":{{}}}}]", super::SIDE_EFFECT_WRITE_FAILED_PREFIX);
        let (rescued, _) = parse_side_effects_jsonl(&marked);
        assert_eq!(rescued.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
