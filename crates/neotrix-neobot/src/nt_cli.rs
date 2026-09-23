//! `nt_cli` — `neobot` CLI 即协议.
//!
//! 移植 cumora `cli-result.ts` (side-effects JSONL 回传) 思想:
//! 模型/外部进程只调本地 `neobot` CLI, 结构化副作用经 JSONL 行回传,
//! Rust sidecar 落库前复核. 模型进程永远拿不到 token/URL.

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 一条副作用行.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub kind: String,
    pub payload: serde_json::Value,
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

#[cfg(test)]
mod tests {
    use super::{format_side_effect, parse_side_effect};

    #[test]
    fn side_effect_roundtrip() {
        let line = format_side_effect("reply", &serde_json::json!({"text": "hi"})).expect("format");
        let effect = parse_side_effect(&line).expect("parse");
        assert_eq!(effect.kind, "reply");
    }
}
