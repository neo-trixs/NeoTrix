//! Structured Archive — Delta Spec 归档 + 冲突检测
//!
//! 借鉴 OpenSpec 的 archive 机制：
//! - 每个 Change 包含一组 DeltaEdits（ADDED/MODIFIED/REMOVED）
//! - Archive 时检测冲突（同一维度被多个 Change 修改）
//! - 冲突自动降级为警告，不阻塞归档

use super::self_edit::MicroEdit;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Delta 变更记录（一次 absorb 或 apply_edit 的完整记录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaChange {
    pub id: String,
    pub timestamp: i64,
    pub description: String,
    pub source: String,
    pub edits: Vec<MicroEdit>,
    pub dimension_deltas: HashMap<String, DeltaKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaKind {
    Added,
    Modified,
    Removed,
    Adjusted,
}

impl DeltaKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Added => "## ADDED",
            Self::Modified => "## MODIFIED",
            Self::Removed => "## REMOVED",
            Self::Adjusted => "## ADJUSTED",
        }
    }
}

/// 归档条目（包含完整的变更记录和冲突标记）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub change: DeltaChange,
    pub archived_at: i64,
    pub conflicts: Vec<ConflictWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictWarning {
    pub dimension: String,
    pub message: String,
    pub involved_changes: Vec<String>,
}

/// 变更归档管理器
pub struct ChangeArchive {
    entries: Vec<ArchiveEntry>,
    /// 维度→最近修改的 Change ID 映射（用于冲突检测）
    dimension_index: HashMap<String, Vec<String>>,
}

impl ChangeArchive {
    const MAX_ENTRIES: usize = 10000;

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            dimension_index: HashMap::new(),
        }
    }

    /// 将一组 edits 记录为 DeltaChange，检测冲突，返回归档条目
    pub fn record(&mut self, description: &str, source: &str, edits: &[MicroEdit]) -> ArchiveEntry {
        let now_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0i64);
        let change_id = format!("change_{}", now_ns);
        let now = Utc::now().timestamp();

        let mut dimension_deltas = HashMap::new();
        for edit in edits {
            if let Some(name) = edit.dimension_name() {
                let kind = match edit.delta_label() {
                    "ADDED" => DeltaKind::Added,
                    "MODIFIED" => DeltaKind::Modified,
                    "REMOVED" => DeltaKind::Removed,
                    _ => DeltaKind::Adjusted,
                };
                dimension_deltas.insert(name.to_string(), kind);
            }
        }

        // 冲突检测：检查每个维度之前是否被其他 Change 修改过
        let mut conflicts = Vec::new();
        for dim in dimension_deltas.keys() {
            let previous = self.dimension_index.get(dim);
            if let Some(prev_ids) = previous {
                if !prev_ids.is_empty() && !prev_ids.contains(&change_id) {
                    conflicts.push(ConflictWarning {
                        dimension: dim.clone(),
                        message: format!("维度 '{}' 已被历史修改影响", dim),
                        involved_changes: prev_ids.clone(),
                    });
                }
            }
        }

        // 更新维度索引
        for dim in dimension_deltas.keys() {
            self.dimension_index
                .entry(dim.clone())
                .or_default()
                .push(change_id.clone());
        }

        let change = DeltaChange {
            id: change_id.clone(),
            timestamp: now,
            description: description.to_string(),
            source: source.to_string(),
            edits: edits.to_vec(),
            dimension_deltas,
        };

        let entry = ArchiveEntry {
            change,
            archived_at: now,
            conflicts,
        };

        self.entries.push(entry.clone());
        if self.entries.len() > Self::MAX_ENTRIES {
            self.entries.drain(0..Self::MAX_ENTRIES / 5);
        }
        entry
    }

    /// 获取所有冲突警告
    pub fn all_conflicts(&self) -> Vec<&ConflictWarning> {
        self.entries
            .iter()
            .flat_map(|e| e.conflicts.iter())
            .collect()
    }

    /// 获取归档条数
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 生成归档报告（Markdown 格式）
    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# Change Archive Report".to_string());
        lines.push(format!("Total entries: {}\n", self.entries.len()));

        for entry in &self.entries {
            lines.push(format!("## {}", entry.change.id));
            lines.push(format!("- **Description**: {}", entry.change.description));
            lines.push(format!("- **Source**: {}", entry.change.source));
            lines.push(format!("- **Timestamp**: {}", entry.change.timestamp));
            lines.push(String::new());
            lines.push("### Delta Spec".to_string());
            for (dim, kind) in &entry.change.dimension_deltas {
                lines.push(format!("{} {}", kind.label(), dim));
            }
            if !entry.conflicts.is_empty() {
                lines.push(String::new());
                lines.push("### ⚠️ Conflicts".to_string());
                for c in &entry.conflicts {
                    lines.push(format!("- {}: {}", c.dimension, c.message));
                }
            }
            lines.push(String::new());
            lines.push("---".to_string());
        }

        lines.join("\n")
    }
}

impl Default for ChangeArchive {
    fn default() -> Self {
        Self::new()
    }
}

/// 通过 CapabilityVector 生成 Delta Spec 报告（不含源代码）
pub fn generate_delta_spec_report(changes: &[DeltaChange]) -> String {
    let mut lines = Vec::new();
    lines.push("# Delta Spec Report".to_string());
    lines.push(String::new());

    for change in changes {
        lines.push(format!("## Change: {}", change.id));
        lines.push(format!("- {}", change.description));
        lines.push(String::new());
        for (dim, kind) in &change.dimension_deltas {
            lines.push(format!("{} {}", kind.label(), dim));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_change() {
        let mut archive = ChangeArchive::new();
        let edits = vec![
            MicroEdit::AddedDimension("new_skill".to_string(), 0.8),
            MicroEdit::ModifiedDimension("analysis".to_string(), 0.5, 0.7),
        ];
        let entry = archive.record("Add new skill", "test", &edits);
        assert_eq!(entry.change.edits.len(), 2);
        assert!(entry.conflicts.is_empty());
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_conflict_detection() {
        let mut archive = ChangeArchive::new();
        let edits_a = vec![MicroEdit::ModifiedDimension(
            "analysis".to_string(),
            0.5,
            0.7,
        )];
        let edits_b = vec![MicroEdit::ModifiedDimension(
            "analysis".to_string(),
            0.7,
            0.9,
        )];

        archive.record("First change", "user_a", &edits_a);
        let entry = archive.record("Second change", "user_b", &edits_b);

        assert!(!entry.conflicts.is_empty(), "应检测到维度冲突");
        assert_eq!(entry.conflicts[0].dimension, "analysis");
    }

    #[test]
    fn test_delta_spec_report() {
        let changes = vec![DeltaChange {
            id: "change_001".to_string(),
            timestamp: 1000,
            description: "Add accessibility".to_string(),
            source: "test".to_string(),
            edits: vec![],
            dimension_deltas: {
                let mut m = HashMap::new();
                m.insert("accessibility".to_string(), DeltaKind::Added);
                m.insert("analysis".to_string(), DeltaKind::Modified);
                m
            },
        }];
        let report = generate_delta_spec_report(&changes);
        assert!(report.contains("ADDED accessibility"));
        assert!(report.contains("MODIFIED analysis"));
    }

    #[test]
    fn test_archive_report() {
        let mut archive = ChangeArchive::new();
        archive.record(
            "test",
            "me",
            &[MicroEdit::AdjustDimension("test".to_string(), 0.1)],
        );
        let report = archive.report();
        assert!(report.contains("Change Archive Report"));
        assert_eq!(archive.all_conflicts().len(), 0);
    }
}
