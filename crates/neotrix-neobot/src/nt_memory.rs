//! `nt_memory` — 跨会话个人记忆（Claude 内置记忆思想本地版）。
//!
//! 一份 `~/.neobot/MEMORY.md`（上限 8KiB，超限写拒读截）：
//! - `neobot memory set <文本>` 追加一行（去重：完全相同的行不重复记）；
//! - `neobot memory get` 原样输出；`memory clear` 需二次确认在调用方做；
//! - HTTP 引擎每次把记忆注入 system prompt 尾部（`MEMORY.md` 不存在/为空则不注）。
//!
//! 秘密律：记忆文件只存用户自己写下的事实，不做自动抓取；
//! 含密钥的行（沿 `redact_detail` 同一 key 表）拒绝写入。

use std::path::{Path, PathBuf};

use crate::nt_error::NtBotError;

/// 记忆文件上限（字节；超限 `set` 拒绝，`get` 截断保读）。
pub const MEMORY_CAP: usize = 8 * 1024;

/// 记忆文件路径.
pub fn memory_path(data_dir: &Path) -> PathBuf {
    data_dir.join("MEMORY.md")
}

/// 读记忆（不存在 → 空串；超长截断到 cap，保读不炸）。
pub fn read_memory(data_dir: &Path) -> String {
    let text = std::fs::read_to_string(memory_path(data_dir)).unwrap_or_default();
    if text.len() <= MEMORY_CAP {
        return text;
    }
    let mut cut = MEMORY_CAP;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    text.get(..cut).unwrap_or("").to_owned()
}

/// 写一行记忆。含密钥行拒绝；完全重复行跳过（幂等）；超限拒绝。
pub fn append_memory(data_dir: &Path, line: &str) -> Result<bool, NtBotError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(NtBotError::Invalid("memory line is empty".to_owned()));
    }
    if trimmed.len() > 1024 {
        return Err(NtBotError::Invalid("memory line exceeds 1024 bytes".to_owned()));
    }
    let redacted = crate::nt_audit::redact_detail(trimmed);
    if redacted.contains("[redacted]") {
        return Err(NtBotError::Invalid(
            "memory line looks like a secret; refused".to_owned(),
        ));
    }
    let current = read_memory(data_dir);
    if current.lines().any(|l| l.trim() == trimmed) {
        return Ok(false);
    }
    if current.len() + trimmed.len() + 1 > MEMORY_CAP {
        return Err(NtBotError::Invalid(format!(
            "MEMORY.md full ({} bytes cap)",
            MEMORY_CAP
        )));
    }
    let mut out = current;
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(trimmed);
    out.push('\n');
    std::fs::write(memory_path(data_dir), out)?;
    Ok(true)
}

/// prompt 注入块（空记忆返回 None，调用方不拼接）。
pub fn memory_block(data_dir: &Path) -> Option<String> {
    let text = read_memory(data_dir);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(format!(
        "用户长期记忆（本地 MEMORY.md，只读参考，不要复述全文）：\n{trimmed}"
    ))
}

/// 配置数据目录的记忆块（引擎装配用；一行调用）。
pub fn memory_for_config(cfg: &crate::nt_config::NeobotConfig) -> Option<String> {
    memory_block(&cfg.data_dir)
}

#[cfg(test)]
mod tests {
    use super::{append_memory, memory_block, read_memory};

    fn tmp(case: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("neobot-memory-test-{case}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn append_dedups_rejects_secrets_and_caps() {
        let dir = tmp("a");
        assert_eq!(read_memory(&dir), "");
        assert_eq!(append_memory(&dir, "喜欢深色模式").expect("append"), true);
        assert_eq!(append_memory(&dir, "喜欢深色模式").expect("append"), false);
        assert!(append_memory(&dir, "OPENAI_API_KEY=sk-x").is_err());
        assert!(append_memory(&dir, "   ").is_err());
        assert!(memory_block(&dir).is_some());
        let empty = tmp("b");
        assert!(memory_block(&empty).is_none());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&empty);
    }
}
