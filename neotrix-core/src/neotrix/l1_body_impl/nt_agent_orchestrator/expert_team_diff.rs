//! P25 融合节点: `expert_team_diff` (吸收自 deepwrite)。
//!
//! 向 orchestrator 注入可评审的 diff-write 能力: 一切写入先 stage,
//! 评审后 apply / reject。**绝不直接覆写** — 每条写入都走
//! `stage → review_pending → apply/reject` 生命周期, 保证可评审性。
//!
//! 纯内存实现 (不落盘、无 tokio、零 unsafe); apply 仅翻转状态标记,
//! 真实文件写由调用方按 `apply_staged()` 返回值执行。

/// diff 条目的生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStatus {
    /// 已暂存, 待评审。
    Staged,
    /// 已应用。
    Applied,
    /// 已拒绝。
    Rejected,
}

/// 一条可评审的 diff 变更。
#[derive(Debug, Clone, PartialEq)]
pub struct DiffEntry {
    pub path: String,
    pub old_snippet: String,
    pub new_snippet: String,
    pub author: String,
    pub status: DiffStatus,
}

/// 暂存/生命周期错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    /// 新片段为空 — 拒绝空写入。
    EmptySnippet,
    /// 旧片段 == 新片段 — 无变更。
    NoChange,
    /// 同路径已有待评审条目 — 冲突, 先处理旧条目。
    Conflict,
    /// 该条目当前状态不允许此操作 (如对非 Staged 条目 reject/apply)。
    NotStaged,
    /// 条目不属于此 writer。
    UnknownEntry,
}

/// 专家团队 diff 写入器 — 一切写入先 stage, 评审后 apply/reject。
#[derive(Debug, Clone, Default)]
pub struct ExpertTeamWriter {
    pub pending_diff: Vec<DiffEntry>,
}

impl ExpertTeamWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// 暂存一条写入。绝不直接覆写 — 总是先入 pending 队列。
    pub fn stage_write(
        &mut self,
        path: &str,
        old_snippet: &str,
        new_snippet: &str,
        author: &str,
    ) -> Result<DiffEntry, DiffError> {
        if new_snippet.is_empty() {
            return Err(DiffError::EmptySnippet);
        }
        if old_snippet == new_snippet {
            return Err(DiffError::NoChange);
        }
        if self.pending_diff.iter().any(|e| e.path == path && e.status == DiffStatus::Staged) {
            return Err(DiffError::Conflict);
        }
        let entry = DiffEntry {
            path: path.to_string(),
            old_snippet: old_snippet.to_string(),
            new_snippet: new_snippet.to_string(),
            author: author.to_string(),
            status: DiffStatus::Staged,
        };
        self.pending_diff.push(entry.clone());
        Ok(entry)
    }

    /// 评审视角: 只暴露待评审 (Staged) 条目。
    pub fn review_pending(&self) -> Vec<&DiffEntry> {
        self.pending_diff
            .iter()
            .filter(|e| e.status == DiffStatus::Staged)
            .collect()
    }

    /// 应用全部 Staged 条目 (标记 Applied), 返回被应用的条目。
    pub fn apply_staged(&mut self) -> Vec<DiffEntry> {
        let mut applied = Vec::new();
        for entry in self.pending_diff.iter_mut() {
            if entry.status == DiffStatus::Staged {
                entry.status = DiffStatus::Applied;
                applied.push(entry.clone());
            }
        }
        applied
    }

    /// 拒绝一条 Staged 条目 (按 `pending_diff` 下标, 标记 Rejected)。
    pub fn reject(&mut self, index: usize) -> Result<(), DiffError> {
        let entry = self
            .pending_diff
            .get_mut(index)
            .ok_or(DiffError::UnknownEntry)?;
        match entry.status {
            DiffStatus::Staged => {
                entry.status = DiffStatus::Rejected;
                Ok(())
            }
            _ => Err(DiffError::NotStaged),
        }
    }

    /// 按路径拒绝 (评审便利入口)。
    pub fn reject_path(&mut self, path: &str) -> Result<(), DiffError> {
        let idx = self
            .pending_diff
            .iter()
            .position(|e| e.path == path && e.status == DiffStatus::Staged)
            .ok_or(DiffError::UnknownEntry)?;
        self.reject(idx)
    }

    pub fn applied_entries(&self) -> Vec<&DiffEntry> {
        self.pending_diff
            .iter()
            .filter(|e| e.status == DiffStatus::Applied)
            .collect()
    }

    pub fn rejected_entries(&self) -> Vec<&DiffEntry> {
        self.pending_diff
            .iter()
            .filter(|e| e.status == DiffStatus::Rejected)
            .collect()
    }

    pub fn pending_count(&self) -> usize {
        self.pending_diff
            .iter()
            .filter(|e| e.status == DiffStatus::Staged)
            .count()
    }

    /// 已应用条目拼接的目标内容 (不含实际文件写 — 由调用方落盘)。
    pub fn rendered_snippet(&self, path: &str) -> Option<String> {
        let applied = self
            .pending_diff
            .iter()
            .find(|e| e.path == path && e.status == DiffStatus::Applied)?;
        Some(applied.new_snippet.clone())
    }
}

impl crate::core::nt_core_self_test::SelfTest for ExpertTeamWriter {
    fn name(&self) -> &str {
        "nt_agent_orchestrator_expert_team_diff"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut writer = ExpertTeamWriter::new();
        let entry = writer
            .stage_write("src/lib.rs", "old", "new", "expert")
            .map_err(|e| vec![format!("stage failed: {:?}", e)])?;
        if writer.review_pending().len() != 1 {
            return Err(vec!["staged entry must be reviewable".into()]);
        }
        if writer.apply_staged().len() != 1 {
            return Err(vec!["apply must return the staged entry".into()]);
        }
        if entry.status != DiffStatus::Staged || writer.rendered_snippet("src/lib.rs") != Some("new".into()) {
            return Err(vec!["applied entry not recorded".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_write_creates_staged_entry() {
        let mut writer = ExpertTeamWriter::new();
        let entry = writer
            .stage_write("src/a.rs", "old", "new", "alice")
            .expect("valid stage");
        assert_eq!(entry.status, DiffStatus::Staged);
        assert_eq!(entry.author, "alice");
        assert_eq!(entry.path, "src/a.rs");
        assert_eq!(writer.pending_count(), 1);
    }

    #[test]
    fn stage_write_rejects_empty_snippet() {
        let mut writer = ExpertTeamWriter::new();
        assert_eq!(
            writer.stage_write("src/a.rs", "old", "", "alice"),
            Err(DiffError::EmptySnippet)
        );
        assert_eq!(writer.pending_count(), 0);
    }

    #[test]
    fn stage_write_rejects_no_change() {
        let mut writer = ExpertTeamWriter::new();
        assert_eq!(
            writer.stage_write("src/a.rs", "same", "same", "alice"),
            Err(DiffError::NoChange)
        );
    }

    #[test]
    fn stage_write_conflicts_on_same_path() {
        let mut writer = ExpertTeamWriter::new();
        writer.stage_write("src/a.rs", "old", "new", "alice").unwrap();
        assert_eq!(
            writer.stage_write("src/a.rs", "old", "newer", "bob"),
            Err(DiffError::Conflict),
            "second pending entry on same path must conflict"
        );
    }

    #[test]
    fn apply_staged_marks_applied() {
        let mut writer = ExpertTeamWriter::new();
        writer.stage_write("src/a.rs", "old", "new", "alice").unwrap();
        writer.stage_write("src/b.rs", "old", "new", "bob").unwrap();
        let applied = writer.apply_staged();
        assert_eq!(applied.len(), 2);
        assert_eq!(writer.pending_count(), 0);
        assert_eq!(writer.applied_entries().len(), 2);
        assert!(applied.iter().all(|e| e.status == DiffStatus::Applied));
        assert_eq!(writer.rendered_snippet("src/a.rs"), Some("new".into()));
    }

    #[test]
    fn reject_marks_rejected_and_blocks_reapply() {
        let mut writer = ExpertTeamWriter::new();
        writer.stage_write("src/a.rs", "old", "new", "alice").unwrap();
        assert_eq!(writer.pending_diff[0].status, DiffStatus::Staged);
        writer.reject(0).unwrap();
        assert_eq!(writer.rejected_entries().len(), 1);
        assert_eq!(writer.pending_diff[0].status, DiffStatus::Rejected);
        // 已拒绝条目不再出现在评审中。
        assert_eq!(writer.review_pending().len(), 0);
    }

    #[test]
    fn reject_non_staged_is_error() {
        let mut writer = ExpertTeamWriter::new();
        writer.stage_write("src/a.rs", "old", "new", "alice").unwrap();
        writer.apply_staged();
        assert_eq!(writer.reject(0), Err(DiffError::NotStaged));
    }

    #[test]
    fn reject_unknown_index_is_error() {
        let mut writer = ExpertTeamWriter::new();
        writer.stage_write("src/a.rs", "old", "new", "alice").unwrap();
        assert_eq!(writer.reject(5), Err(DiffError::UnknownEntry));
    }

    #[test]
    fn reject_path_by_name() {
        let mut writer = ExpertTeamWriter::new();
        writer.stage_write("src/a.rs", "old", "new", "alice").unwrap();
        writer.stage_write("src/b.rs", "old", "new", "bob").unwrap();
        writer.reject_path("src/a.rs").unwrap();
        assert_eq!(writer.rejected_entries().len(), 1);
        assert_eq!(writer.rejected_entries()[0].path, "src/a.rs");
        assert_eq!(writer.pending_count(), 1);
    }

    #[test]
    fn full_review_lifecycle() {
        let mut writer = ExpertTeamWriter::new();
        writer.stage_write("src/c.rs", "fn a() {}", "fn b() {}", "carol").unwrap();
        assert_eq!(writer.review_pending().len(), 1);
        let all_applied = writer.apply_staged();
        assert_eq!(all_applied.len(), 1);
        assert_eq!(all_applied[0].status, DiffStatus::Applied);
        // 同一路径可再次 stage (旧条目已应用, 无冲突)。
        writer.stage_write("src/c.rs", "fn b() {}", "fn c() {}", "carol").unwrap();
        assert_eq!(writer.review_pending().len(), 1);
    }
}
