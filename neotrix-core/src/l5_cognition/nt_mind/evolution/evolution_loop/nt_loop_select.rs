//! 自进化循环 — 选择阶段 (`nt_loop_select`).
//!
//! 快照 → 问题集: 六类检测器 + 停滞/高危人工介入判断。
//! 从 `evolution_loop.rs` 纯搬移, 行为零变更。

use super::nt_loop_types::{
    EvolutionLoop, Issue, EXCESS_UNWRAP_THRESHOLD, STAGNATION_LIMIT, TODO_LEFTOVERS_THRESHOLD,
};
use crate::l5_cognition::l1_facade::ProjectSnapshot;
use crate::l5_cognition::nt_mind::evolution::evolution_daemon::IssueType;

impl EvolutionLoop {
    // ─── 问题检测器 ───

    pub(crate) fn detect_large_files(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        for file in &snap.large_files {
            issues.push(Issue {
                issue_type: IssueType::LargeFile,
                severity: 5,
                file: Some(file.clone()),
                description: format!("文件过大: {}", file),
                suggestion: "拆分为多个子模块 (<800 行/文件)".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    pub(crate) fn detect_missing_tests(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        for file in &snap.modules_without_tests {
            issues.push(Issue {
                issue_type: IssueType::MissingTests,
                severity: 4,
                file: Some(file.clone()),
                description: format!("模块无测试: {}", file),
                suggestion: "添加 #[cfg(test)] mod tests 单元测试".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    pub(crate) fn detect_excess_unsafe(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        for file in &snap.file_unsafe_hotspots {
            issues.push(Issue {
                issue_type: IssueType::ExcessUnsafe,
                severity: 7,
                file: Some(file.clone()),
                description: format!("unsafe 过多: {}", file),
                suggestion: "审查 unsafe 块, 减少或添加安全抽象".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    pub(crate) fn detect_excess_unwrap(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        if snap.unwrap_count > EXCESS_UNWRAP_THRESHOLD {
            issues.push(Issue {
                issue_type: IssueType::ExcessUnwrap,
                severity: 6,
                file: None,
                description: format!(".unwrap() 过多: {} 处", snap.unwrap_count),
                suggestion: "用 ? 操作符或 match 替代 unwrap".into(),
                auto_fixable: true,
                cycle_discovered: self.cycle,
            });
        }
    }

    pub(crate) fn detect_todo_leftovers(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        if snap.todo_count > TODO_LEFTOVERS_THRESHOLD {
            issues.push(Issue {
                issue_type: IssueType::TodoLeftovers,
                severity: 2,
                file: None,
                description: format!("TODO 残留: {} 处", snap.todo_count),
                suggestion: "清理已完成 TODO, 将未完成转移至 TODO.md".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    pub(crate) fn detect_compile_issues(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        if snap.compile_errors > 0 {
            issues.push(Issue {
                issue_type: IssueType::CompileWarning,
                severity: 10,
                file: None,
                description: format!("编译错误: {} 个", snap.compile_errors),
                suggestion: "运行 cargo check --lib 修复错误".into(),
                auto_fixable: true,
                cycle_discovered: self.cycle,
            });
        }
        if snap.compile_warnings > 0 {
            issues.push(Issue {
                issue_type: IssueType::CompileWarning,
                severity: 3,
                file: None,
                description: format!("编译警告: {} 个", snap.compile_warnings),
                suggestion: "运行 cargo fix --lib 自动修复".into(),
                auto_fixable: true,
                cycle_discovered: self.cycle,
            });
        }
    }

    /// 判断是否需要人工介入
    pub(crate) fn _needs_human_intervention(&self) -> bool {
        self.consecutive_stagnant >= STAGNATION_LIMIT
            || self.issues.iter().any(|i| i.severity >= 9)
    }
}
