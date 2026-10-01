//! `nt_memory` — 跨会话个人记忆（Claude 内置记忆思想本地版）。
//!
//! 一份 `~/.neobot/MEMORY.md`（上限 8KiB，超限写拒读截）：
//! - `neobot memory set <文本>` 追加一行（去重：完全相同的行不重复记）；
//! - `neobot memory get` 原样输出；`memory clear` 需二次确认在调用方做；
//! - HTTP 引擎每次把记忆注入 system prompt 尾部（`MEMORY.md` 不存在/为空则不注）。
//!
//! 秘密律：记忆文件只存用户自己写下的事实，不做自动抓取；
//! 含密钥的行（沿 `redact_detail` 同一 key 表）拒绝写入。
//!
//! 耐久律（rakazo `capabilities.revisions` 移植，2026-09-30）：
//! - 写一律 `tmp + rename` 原子替换（`write` 是截断式直写，崩一次整份归零，
//!   而记忆是不可再生的用户手打内容）；
//! - 每次写前把**旧整份**推进 `MEMORY.rev.jsonl`（一版一行，上限 `MEMORY_REV_CAP`）；
//! - `memory undo` 回到上一版，且撤前先把当前存回历史 ⇒ 撤销可再撤销；
//! - 历史坏行 = Err（跳过会让 undo 拿回一个「合法但不是上一版」的内容）。

use std::path::{Path, PathBuf};

use crate::nt_error::NtBotError;

/// 记忆文件上限（字节；超限 `set` 拒绝，`get` 截断保读）。
pub const MEMORY_CAP: usize = 8 * 1024;

/// 历史版本保留条数上限（`MEMORY.rev.jsonl` 里的完整快照数）。
///
/// ⛔ 不设上限 = 用户每记一行就永久留一份 8KiB 文件，磁盘只增不减。
/// 20 版 ≈ 160KiB 上界，够回退又不至于变成垃圾场。
pub const MEMORY_REV_CAP: usize = 20;

/// 一条历史（写入前的旧内容 + 时间戳）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRevision {
    pub ts: String,
    pub content: String,
}

/// 记忆文件路径.
pub fn memory_path(data_dir: &Path) -> PathBuf {
    data_dir.join("MEMORY.md")
}

/// 历史文件路径（追加式 JSONL，一行一版）。
pub fn memory_history_path(data_dir: &Path) -> PathBuf {
    data_dir.join("MEMORY.rev.jsonl")
}

/// 原子写（tmp + rename）。
///
/// ⛔ 之前用 `fs::write` 直写：`write` 是「截断 → 写」，中途崩/断电 = 整份记忆
/// 归零。记忆是用户手打、不可再生的东西（不像用量数字能重算），拿它赌一次
/// 直写不值得。同款修法见 `nt_cost::UsageLedger::save`。
fn write_atomic(data_dir: &Path, body: &str) -> Result<(), NtBotError> {
    let path = memory_path(data_dir);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let tmp = path.with_extension("md.tmp");
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// 追加一条历史（超 `MEMORY_REV_CAP` 就重写收敛，只留最近若干条）。
///
/// ⛔ 历史先写、正式文件后写：崩在两步之间 = 记忆没记上但历史多一条，
/// undo 会「撤」一版不存在的内容（幂等判等拦得住，见 `memory_undo`）。
/// 反过来（先写文件后写历史）崩了 = 记上了但撤不回去 —— 那种丢法更难解释。
fn push_revision(data_dir: &Path, content: &str) -> Result<(), NtBotError> {
    let rev = MemoryRevision {
        ts: chrono::Utc::now().to_rfc3339(),
        content: content.to_owned(),
    };
    let path = memory_history_path(data_dir);
    let mut body = std::fs::read_to_string(&path).unwrap_or_default();
    body.push_str(&serde_json::to_string(&rev)?);
    body.push('\n');
    let lines: Vec<&str> = body.lines().collect();
    if lines.len() > MEMORY_REV_CAP {
        let tail = &lines[lines.len() - MEMORY_REV_CAP..];
        body = tail.iter().map(|l| format!("{l}\n")).collect();
    }
    std::fs::write(&path, body)?;
    Ok(())
}

/// 读历史（旧 → 新）。
///
/// ⛔ 坏行 = Err，不跳过：跳过会让 undo 拿回一个「看起来合法、其实不是上一版」
/// 的内容 —— 静默回退错版本比拒绝回退坏得多。崩在追加中途最容易出坏行，
/// 真遇到了看一眼文件尾部即可。
pub fn memory_revisions(data_dir: &Path) -> Result<Vec<MemoryRevision>, NtBotError> {
    let body = std::fs::read_to_string(memory_history_path(data_dir)).unwrap_or_default();
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str::<MemoryRevision>(l)
                .map_err(|e| NtBotError::Store(format!("记忆历史损坏（拒猜）：{e}")))
        })
        .collect()
}

/// 撤一版（返回恢复后的内容；`None` = 无可撤 / 已在历史顶端）。
///
/// ⛔ 先把当前内容存进历史再回退 —— 于是「撤」也能再「撤」回来。
pub fn memory_undo(data_dir: &Path) -> Result<Option<String>, NtBotError> {
    let revs = memory_revisions(data_dir)?;
    let Some(previous) = revs.last() else {
        return Ok(None);
    };
    let previous_content = previous.content.clone();
    let current = read_memory(data_dir);
    if current == previous_content {
        return Ok(None);
    }
    push_revision(data_dir, &current)?;
    write_atomic(data_dir, &previous_content)?;
    Ok(Some(previous_content))
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
    let mut out = current.clone();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(trimmed);
    out.push('\n');
    // ⛔ 历史记的是「写之前」的整份内容，不是这一行：
    // 撤销的单位是「回到上一版记忆」，不是「删掉最后一行」（中间可能重写过）。
    push_revision(data_dir, &current)?;
    write_atomic(data_dir, &out)?;
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
    use super::{append_memory, memory_block, memory_revisions, memory_undo, read_memory, MEMORY_REV_CAP};

    fn tmp(case: &str) -> std::path::PathBuf {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-memory-test-{}", case));
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

    #[test]
    fn 撤销回到上一版且可再撤回来() {
        let dir = tmp("undo");
        // 无历史 = no-op（不是错：用户没记过时不该报错）。
        assert_eq!(memory_undo(&dir).expect("空历史"), None);
        append_memory(&dir, "一").expect("记一");
        append_memory(&dir, "二").expect("记二");
        assert_eq!(read_memory(&dir), "一\n二\n");
        // 撤一版 = 回到只有「一」。
        assert_eq!(memory_undo(&dir).expect("撤").as_deref(), Some("一\n"));
        assert_eq!(read_memory(&dir), "一\n");
        // 撤也能再撤回来（撤前把当前存进历史）—— 不是单向门。
        assert_eq!(memory_undo(&dir).expect("再撤").as_deref(), Some("一\n二\n"));
        assert_eq!(read_memory(&dir), "一\n二\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 重复行不进历史() {
        let dir = tmp("dedup");
        append_memory(&dir, "x").expect("记");
        append_memory(&dir, "x").expect("重复");
        // ⛔ 历史记的是「写之前」的整份内容，所以第一次 append 也留一条
        //    （内容为空 = 回到「没记过」）。要守的是「第二次没再留一条」：
        //    重复行若也推历史，undo 会撤到一个与它一模一样的版本（原地踏步）。
        let revs = memory_revisions(&dir).expect("读历史");
        assert_eq!(revs.len(), 1, "重复行不得留下第二条历史");
        assert_eq!(revs[0].content, "");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 历史有界且坏行拒猜() {
        let dir = tmp("cap");
        for i in 0..(MEMORY_REV_CAP + 5) {
            append_memory(&dir, &format!("行{i}")).expect("记");
        }
        let revs = memory_revisions(&dir).expect("读历史");
        assert!(revs.len() <= MEMORY_REV_CAP, "历史必须收敛，实际 {}", revs.len());
        // 坏行 = Err（跳过会让 undo 拿回一个「合法但不是上一版」的内容）。
        std::fs::write(
            super::memory_history_path(&dir),
            "{\"ts\":\"now\",\"content\":\"好\"}\n{断的\n",
        )
        .expect("造坏行");
        assert!(memory_revisions(&dir).is_err());
        assert!(memory_undo(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
