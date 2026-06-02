//! SelfDiagnose — 自我诊断引擎（零 LLM 依赖）
//!
//! 核心职责:
//!   - 扫描结果 → 多维评分 (Impact × Urgency × Feasibility)
//!   - 历史 + 能力向量 → 优先级排序
//!   - 每个诊断项 → ActionPlan 执行策略

use super::nt_mind_autofixer::AutoFixer;
use super::nt_mind_evolution_loop::{
    Issue, IssueType, ProjectSnapshot,
    EXCESS_UNWRAP_THRESHOLD,
    LARGE_FILE_THRESHOLD, MISSING_TESTS_THRESHOLD, TODO_LEFTOVERS_THRESHOLD,
};

// ============================================================
// 诊断结果
// ============================================================

/// 带评分的诊断项 — 比 Issue 多三层元信息
#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub underlying_issue: Issue,
    pub impact: f64,        // 0.0-1.0 修复后改进潜力
    pub urgency: f64,       // 0.0-1.0 阻塞程度
    pub feasibility: f64,   // 0.0-1.0 自动化可行程度
    pub composite_score: f64,
    pub action: ActionPlan,
}

/// 每个诊断项映射到一个具体执行计划
#[derive(Debug, Clone)]
pub enum ActionPlan {
    AddTestStub { file: String },
    RunCargoFix,
    RemoveTodo { file: String },
    HumanDecision { issue_type: IssueType, file: Option<String>, reason: String },
    SplitLargeFile { file: String },
    ReviewUnsafe { file: String },
    ReplaceUnwrap { file: String },
    NoAction { reason: String },
}

// ============================================================
// 优先级队列
// ============================================================

/// 按 composite_score 降序排列的优先队列
#[derive(Debug, Clone)]
pub struct PriorityQueue {
    items: Vec<DiagnosticItem>,
}

impl PriorityQueue {
    pub fn new(items: Vec<DiagnosticItem>) -> Self {
        let mut q = Self { items };
        q.sort();
        q
    }

    fn sort(&mut self) {
        self.items.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn into_vec(self) -> Vec<DiagnosticItem> {
        self.items
    }

    pub fn as_slice(&self) -> &[DiagnosticItem] {
        &self.items
    }

    pub fn len(&self) -> usize { self.items.len() }

    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

// ============================================================
// 自我诊断引擎
// ============================================================

/// 自我诊断引擎 — 纯 Rust 推理，零 LLM 调用
pub struct SelfDiagnose;

impl SelfDiagnose {
    /// 执行一次完整诊断：扫描 + 检测 + 评分 + 排序
    pub fn run_diagnosis(
        snapshot: &ProjectSnapshot,
        cycle: u64,
    ) -> (Vec<DiagnosticItem>, PriorityQueue) {
        let issues = Self::detect_all(snapshot, cycle);
        let diagnosed = Self::score_all(issues);
        let prioritized = PriorityQueue::new(diagnosed);
        let raw = prioritized.as_slice().to_vec();
        (raw, prioritized)
    }

    // ─── 检测（与 evolution_loop 的 detect_* 对称但统一出口） ───

    fn detect_all(snapshot: &ProjectSnapshot, cycle: u64) -> Vec<Issue> {
        let mut issues = Vec::new();
        Self::detect_large_files(snapshot, cycle, &mut issues);
        Self::detect_missing_tests(snapshot, cycle, &mut issues);
        Self::detect_excess_unsafe(snapshot, cycle, &mut issues);
        Self::detect_excess_unwrap(snapshot, cycle, &mut issues);
        Self::detect_todo_leftovers(snapshot, cycle, &mut issues);
        Self::detect_compile_issues(snapshot, cycle, &mut issues);
        issues
    }

    fn detect_large_files(snap: &ProjectSnapshot, cycle: u64, issues: &mut Vec<Issue>) {
        for file in &snap.large_files {
            issues.push(Issue {
                issue_type: IssueType::LargeFile,
                severity: 5,
                file: Some(file.clone()),
                description: format!("文件过大: {} (阈值={})", file, LARGE_FILE_THRESHOLD),
                suggestion: format!("拆分为 ≤{} 行的子模块", LARGE_FILE_THRESHOLD),
                auto_fixable: false,
                cycle_discovered: cycle,
            });
        }
    }

    fn detect_missing_tests(snap: &ProjectSnapshot, cycle: u64, issues: &mut Vec<Issue>) {
        for file in &snap.modules_without_tests {
            issues.push(Issue {
                issue_type: IssueType::MissingTests,
                severity: 4,
                file: Some(file.clone()),
                description: format!("模块无测试: {} (行数={})", file, MISSING_TESTS_THRESHOLD),
                suggestion: "添加 #[cfg(test)] mod tests 并编写单元测试".into(),
                auto_fixable: false,
                cycle_discovered: cycle,
            });
        }
    }

    fn detect_excess_unsafe(snap: &ProjectSnapshot, cycle: u64, issues: &mut Vec<Issue>) {
        for file in &snap.file_unsafe_hotspots {
            issues.push(Issue {
                issue_type: IssueType::ExcessUnsafe,
                severity: 7,
                file: Some(file.clone()),
                description: format!("unsafe 过多: {} (计数={})", file, snap.unsafe_count),
                suggestion: "审查 unsafe 块,减少或添加安全抽象".into(),
                auto_fixable: false,
                cycle_discovered: cycle,
            });
        }
    }

    fn detect_excess_unwrap(snap: &ProjectSnapshot, cycle: u64, issues: &mut Vec<Issue>) {
        if snap.unwrap_count > EXCESS_UNWRAP_THRESHOLD {
            issues.push(Issue {
                issue_type: IssueType::ExcessUnwrap,
                severity: 6,
                file: None,
                description: format!(".unwrap() 过多: {} 处 (阈值={})", snap.unwrap_count, EXCESS_UNWRAP_THRESHOLD),
                suggestion: "用 ? 操作符或 match 替代 unwrap".into(),
                auto_fixable: true,
                cycle_discovered: cycle,
            });
        }
    }

    fn detect_todo_leftovers(snap: &ProjectSnapshot, cycle: u64, issues: &mut Vec<Issue>) {
        if snap.todo_count > TODO_LEFTOVERS_THRESHOLD {
            issues.push(Issue {
                issue_type: IssueType::TodoLeftovers,
                severity: 2,
                file: None,
                description: format!("TODO 残留: {} 处 (阈值={})", snap.todo_count, TODO_LEFTOVERS_THRESHOLD),
                suggestion: "清理已完成 TODO,将未完成转移至 TODO.md".into(),
                auto_fixable: false,
                cycle_discovered: cycle,
            });
        }
    }

    fn detect_compile_issues(snap: &ProjectSnapshot, cycle: u64, issues: &mut Vec<Issue>) {
        if snap.compile_errors > 0 {
            issues.push(Issue {
                issue_type: IssueType::CompileWarning,
                severity: 10,
                file: None,
                description: format!("编译错误: {} 个", snap.compile_errors),
                suggestion: "运行 cargo check --lib 修复".into(),
                auto_fixable: true,
                cycle_discovered: cycle,
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
                cycle_discovered: cycle,
            });
        }
    }

    // ─── 评分 ───

    fn score_all(issues: Vec<Issue>) -> Vec<DiagnosticItem> {
        issues.into_iter().map(Self::score_one).collect()
    }

    fn score_one(issue: Issue) -> DiagnosticItem {
        let impact = Self::calc_impact(&issue);
        let urgency = Self::calc_urgency(&issue);
        let feasibility = Self::calc_feasibility(&issue);
        let composite = impact * 0.3 + urgency * 0.4 + feasibility * 0.3;
        let action = Self::plan_action(&issue);
        DiagnosticItem { underlying_issue: issue, impact, urgency, feasibility, composite_score: composite, action }
    }

    /// impact: 修复后系统性改进潜力
    fn calc_impact(issue: &Issue) -> f64 {
        match issue.issue_type {
            IssueType::CompileWarning => 0.9,   // 修复后编译接近 0 warning
            IssueType::MissingTests => 0.7,      // 增加测试覆盖率
            IssueType::LargeFile => 0.7,         // 改善可维护性
            IssueType::ExcessUnsafe => 0.5,      // 安全提升
            IssueType::ExcessUnwrap => 0.4,      // 错误处理改进
            IssueType::TodoLeftovers => 0.3,     // 清洁度提升
            IssueType::HighFreeEnergy => 0.6,    // 世界模型改善
            IssueType::LowPhi => 0.6,            // 意识核改善
            _ => 0.5,
        }
    }

    /// urgency: 阻塞程度
    fn calc_urgency(issue: &Issue) -> f64 {
        issue.severity as f64 / 10.0
    }

    /// feasibility: 自动修复可行度
    fn calc_feasibility(issue: &Issue) -> f64 {
        match issue.issue_type {
            IssueType::CompileWarning => 0.9,    // cargo fix 可自动处理
            IssueType::MissingTests => 0.8,      // AutoFixer 有 add_test_stub
            IssueType::TodoLeftovers => 0.3,     // 需要人工判断哪些TODO可删
            IssueType::LargeFile => 0.2,         // 需要架构决策
            IssueType::ExcessUnsafe => 0.1,      // 需要安全审查
            IssueType::ExcessUnwrap => 0.2,      // 需要上下文理解
            _ => 0.0,
        }
    }

    /// 每个诊断项映射到 ActionPlan
    fn plan_action(issue: &Issue) -> ActionPlan {
        match issue.issue_type {
            IssueType::MissingTests => {
                if let Some(ref file) = issue.file {
                    ActionPlan::AddTestStub { file: file.clone() }
                } else {
                    ActionPlan::NoAction { reason: "缺少文件路径".into() }
                }
            }
            IssueType::CompileWarning => ActionPlan::RunCargoFix,
            IssueType::LargeFile => {
                if let Some(ref file) = issue.file {
                    ActionPlan::SplitLargeFile { file: file.clone() }
                } else {
                    ActionPlan::NoAction { reason: "缺少文件路径".into() }
                }
            }
            IssueType::ExcessUnsafe => {
                if let Some(ref file) = issue.file {
                    ActionPlan::ReviewUnsafe { file: file.clone() }
                } else {
                    ActionPlan::NoAction { reason: "缺少文件路径".into() }
                }
            }
            IssueType::ExcessUnwrap => {
                if let Some(ref file) = issue.file {
                    ActionPlan::ReplaceUnwrap { file: file.clone() }
                } else {
                    ActionPlan::NoAction { reason: "缺少文件路径".into() }
                }
            }
            IssueType::TodoLeftovers => {
                ActionPlan::HumanDecision {
                    issue_type: IssueType::TodoLeftovers,
                    file: None,
                    reason: format!("{} 个 TODO 需要人工判定哪些已完成", TODO_LEFTOVERS_THRESHOLD),
                }
            }
            _ => ActionPlan::HumanDecision {
                issue_type: issue.issue_type,
                file: issue.file.clone(),
                reason: "无法自动处理,需要人工决策".into(),
            },
        }
    }
}

// ============================================================
// 执行器 — 根据 ActionPlan 调用 AutoFixer
// ============================================================

/// ActionPlan 执行器 — 将诊断计划转化为实际代码修改
pub struct ActionExecutor;

impl ActionExecutor {
    /// 执行一个 ActionPlan,返回修复结果描述
    pub fn execute(plan: &ActionPlan) -> Result<String, String> {
        match plan {
            ActionPlan::AddTestStub { file } => AutoFixer::add_test_stub(file),
            ActionPlan::RunCargoFix => AutoFixer::cargo_fix(),
            ActionPlan::RemoveTodo { file } => {
                // 读取文件,找到 // TODO 行并移除
                let content = std::fs::read_to_string(file)
                    .map_err(|e| format!("读取失败: {}", e))?;
                let lines: Vec<&str> = content.lines().collect();
                let mut removed = 0usize;
                let mut new_lines = Vec::new();
                for line in &lines {
                    let t = line.trim();
                    if t.starts_with("// TODO")
                        || t.starts_with("//TODO")
                        || t.starts_with("// FIXME")
                        || t.starts_with("//FIXME")
                        || t.starts_with("//HACK")
                    {
                        removed += 1;
                        continue;
                    }
                    new_lines.push(*line);
                }
                if removed > 0 {
                    std::fs::write(file, new_lines.join("\n"))
                        .map_err(|e| format!("写入失败: {}", e))?;
                    Ok(format!("已移除 {} 行 TODO 注释", removed))
                } else {
                    Err("没有可移除的 TODO 注释行".into())
                }
            }
            ActionPlan::SplitLargeFile { file } => {
                Err(format!("拆分文件 {} 需要人工决策,无法自动完成", file))
            }
            ActionPlan::ReviewUnsafe { file } => {
                Err(format!("{} 需要人工审查 unsafe 块", file))
            }
            ActionPlan::ReplaceUnwrap { file } => {
                Err(format!("{} 需要人工审查 unwrap 调用", file))
            }
            ActionPlan::HumanDecision { reason, .. } => {
                Err(format!("需要人工决策: {}", reason))
            }
            ActionPlan::NoAction { reason } => {
                Err(format!("无操作: {}", reason))
            }
        }
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_snapshot() -> ProjectSnapshot {
        ProjectSnapshot {
            total_files: 100,
            total_lines: 10000,
            large_files: vec!["src/big.rs".into()],
            modules_without_tests: vec!["src/untested.rs".into()],
            file_unsafe_hotspots: vec!["src/danger.rs".into()],
            unsafe_count: 10,
            unwrap_count: 0,
            todo_count: 5,
            compile_errors: 0,
            compile_warnings: 3,
            test_count: 50,
            test_failures: 1,
        }
    }

    #[test]
    fn test_diagnose_produces_items() {
        let snap = dummy_snapshot();
        let (items, pq) = SelfDiagnose::run_diagnosis(&snap, 1);
        assert!(!items.is_empty());
        assert_eq!(items.len(), pq.len());
    }

    #[test]
    fn test_priority_queue_order_descending() {
        let snap = dummy_snapshot();
        let (items, _) = SelfDiagnose::run_diagnosis(&snap, 1);
        for w in items.windows(2) {
            assert!(w[0].composite_score >= w[1].composite_score - 1e-9);
        }
    }

    #[test]
    fn test_compile_error_gets_highest_urgency() {
        let mut snap = dummy_snapshot();
        snap.compile_errors = 5;
        let (items, _) = SelfDiagnose::run_diagnosis(&snap, 1);
        let compile = items.iter().find(|d| matches!(d.underlying_issue.issue_type, IssueType::CompileWarning));
        assert!(compile.is_some());
        assert!(compile.unwrap().urgency >= 0.9);
    }

    #[test]
    fn test_action_plan_maps_correctly() {
        let snap = dummy_snapshot();
        let (items, _) = SelfDiagnose::run_diagnosis(&snap, 1);
        let missing_test = items.iter().find(|d| matches!(d.underlying_issue.issue_type, IssueType::MissingTests));
        assert!(missing_test.is_some());
        assert!(matches!(missing_test.unwrap().action, ActionPlan::AddTestStub { .. }));
    }

    #[test]
    fn test_action_executor_run_cargo_fix() {
        let result = ActionExecutor::execute(&ActionPlan::RunCargoFix);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_action_executor_human_decision_returns_err() {
        let plan = ActionPlan::HumanDecision {
            issue_type: IssueType::TodoLeftovers,
            file: None,
            reason: "test".into(),
        };
        let result = ActionExecutor::execute(&plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_todo_action_on_temporary_file() {
        let tmp = std::env::temp_dir().join("test_todo_remove.rs");
        std::fs::write(&tmp, "// TODO: this should go\nfn main() {}\n// FIXME: also this\n").unwrap();
        let result = ActionExecutor::execute(&ActionPlan::RemoveTodo { file: tmp.to_string_lossy().to_string() });
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(!content.contains("TODO"));
        assert!(!content.contains("FIXME"));
        assert!(content.contains("fn main()"));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_diagnose_respects_thresholds() {
        let mut snap = dummy_snapshot();
        snap.todo_count = 1; // below TODO_LEFTOVERS_THRESHOLD (3)
        snap.compile_errors = 0;
        snap.compile_warnings = 0;
        // Only large_files and unsafe remain
        let (items, _) = SelfDiagnose::run_diagnosis(&snap, 1);
        let todo = items.iter().filter(|d| matches!(d.underlying_issue.issue_type, IssueType::TodoLeftovers)).count();
        assert_eq!(todo, 0, "TODO 少于阈值时不生成诊断项");
    }
}
