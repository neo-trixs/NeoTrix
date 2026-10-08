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
    // 包络行：camelCase `sideEffects` 是本协议的出站键名；snake_case
    // `side_effects` 是 serde derive 的默认键 —— 两个都认，
    // 免得「哪端忘了 rename」变成静默 0 条（协议错必须能被消费才发现）。
    if let Some(inner) = value
        .get("sideEffects")
        .or_else(|| value.get("side_effects"))
    {
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

/// 写一个包络行到回传文件（追加；失败打 stderr 标记 + 原错返回）。
fn append_envelope_line(path: &std::path::Path, line: &str) -> Result<(), NtBotError> {
    use std::io::Write as _;
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

/// 往回传文件追加一行包络（子进程侧用；父进程读后删）。
/// 无 env 或 effects 为空 → 静默跳过；写失败 → stderr 标记 + 原错返回。
pub fn append_side_effects_result(
    path: &std::path::Path,
    effects: &[SideEffect],
) -> Result<(), NtBotError> {
    if effects.is_empty() {
        return Ok(());
    }
    let line = serde_json::to_string(&serde_json::json!({ "sideEffects": effects }))?;
    append_envelope_line(path, &line)
}

/// 把一次 CLI 调用的结果打成一行包络 JSON。
///
/// ⭐ 键名必须是 camelCase `sideEffects`：解析端 `parse_side_effects_jsonl`
/// 的包络分支只认这个键（serde derive 出来的 `side_effects` 会被静默丢成
/// 0 条 effect —— 静默协议错，比报错更坏）。
pub fn format_result_envelope(result: &CliResult) -> Result<String, NtBotError> {
    let value = serde_json::json!({
        "ok": result.ok,
        "text": result.text,
        "exit_code": result.exit_code,
        "sideEffects": result.side_effects,
    });
    Ok(serde_json::to_string(&value)?)
}

/// 往回传文件追加一行**完整结果包络**（`ok`/`text`/`exit_code`/`sideEffects`）。
///
/// 与 `append_side_effects_result` 的差别：结果包络**即使零副作用也写**
/// —— 包络本身就是这次调用的回执（`text` 是答复正文），跳过等于把答复丢了。
/// 写失败：stderr 标记 + 原错返回（同族口径）。
pub fn append_cli_result(path: &std::path::Path, result: &CliResult) -> Result<(), NtBotError> {
    let line = format_result_envelope(result)?;
    append_envelope_line(path, &line)
}

#[cfg(test)]
mod tests {
    use super::{
        CliResult, SideEffect, append_cli_result, append_side_effects_result, format_result_envelope,
        format_side_effect, parse_side_effect, parse_side_effects_jsonl,
    };

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
        let dir = crate::nt_testutil::temp_dir("cli-result-test");
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

    /// 结果包络的**键名**是协议的一部分：出站必须 camelCase `sideEffects`，
    /// 否则解析端会静默读出 0 条（本测试锁死这条往返）。
    #[test]
    fn result_envelope_roundtrip_keeps_camel_case_key() {
        let result = CliResult {
            ok: true,
            text: "hello".to_owned(),
            exit_code: 0,
            side_effects: vec![SideEffect {
                kind: "reply".to_owned(),
                payload: serde_json::json!({"text": "hello"}),
            }],
        };
        let line = format_result_envelope(&result).expect("envelope");
        assert!(line.contains("\"sideEffects\""), "出站键必须 camelCase: {line}");
        let (effects, malformed) = parse_side_effects_jsonl(&line);
        assert_eq!(malformed, 0);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].kind, "reply");
        assert_eq!(effects[0].payload["text"], "hello");
    }

    /// serde derive 默认键 `side_effects` 也必须能被解析端认出 ——
    /// 两端各自演化时这是最后一道防线（静默丢包比报错更坏）。
    #[test]
    fn snake_case_envelope_also_parses() {
        let line = serde_json::json!({
            "ok": false,
            "text": "t",
            "exit_code": 3,
            "side_effects": [{"kind": "task", "payload": {"id": "7"}}],
        })
        .to_string();
        let (effects, malformed) = parse_side_effects_jsonl(&line);
        assert_eq!(malformed, 0);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].kind, "task");
    }

    /// 结果包络**零副作用也写**（回执即 `text`/`ok`），与副作用变体的
    /// 「空则跳过」有意不同；写完可被解析端安全读回（0 条 effect、0 坏行）。
    #[test]
    fn append_cli_result_writes_even_with_no_side_effects() {
        let dir = crate::nt_testutil::temp_dir("cli-result-empty");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("result.jsonl");
        let _ = std::fs::remove_file(&path);
        let result = CliResult {
            ok: true,
            text: "no effects here".to_owned(),
            exit_code: 0,
            side_effects: Vec::new(),
        };
        append_cli_result(&path, &result).expect("append");
        let raw = std::fs::read_to_string(&path).expect("read");
        assert!(raw.contains("no effects here"));
        let (effects, malformed) = parse_side_effects_jsonl(&raw);
        assert_eq!(effects.len(), 0);
        assert_eq!(malformed, 0);
        // 空副作用变体仍保持「跳过不写」
        append_side_effects_result(&path, &[]).expect("skip");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
