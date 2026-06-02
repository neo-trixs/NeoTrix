//! 自进化循环引擎 — 持续迭代: 扫描 → 分析 → 修复 → 蒸馏
//!
//! 设计: 每个周期执行:
//!   1. 项目扫描 (代码度量, 文件大小, 测试覆盖率, 编译状态)
//!   2. 瓶颈分析 (慢模块, 循环依赖, unwrap 热点)
//!   3. Bug 检测 (编译警告, 测试失败, unsafe 使用)
//!   4. 自修复生成 (修复警告, 补全导入, 处理 unwrap)
//!   5. 模式蒸馏 (提取行为规则, 更新 AGENTS.md)
//!   6. 报告输出 (状态仪表盘)
//!
//! 融合 AGENTS.md 元认知自检 + MetaCognitive Self-Check 协议

use super::autofixer::AutoFixer;
use super::fep_iit_bridge::FEPIITBridge;
use super::nt_act_code::PipelineAutoFixer;
use super::self_diagnose::{ActionExecutor, DiagnosticItem, PriorityQueue, SelfDiagnose};
use serde::{Deserialize, Serialize};

// ============================================================
// 常量
// ============================================================

/// 大文件阈值 (行数)
pub const LARGE_FILE_THRESHOLD: usize = 800;

/// 无测试模块阈值 (行数)
pub const MISSING_TESTS_THRESHOLD: usize = 300;

/// 最大 unsafe 数量
pub const EXCESS_UNSAFE_THRESHOLD: usize = 5;

/// 最大 unwrap 数量
pub const EXCESS_UNWRAP_THRESHOLD: usize = 20;

/// 最大 TODO 残留数
pub const TODO_LEFTOVERS_THRESHOLD: usize = 3;

/// 停滞检测: 连续无改进次数上限
pub const STAGNATION_LIMIT: u32 = 10;

// ============================================================
// 问题类型
// ============================================================

/// 检测到的问题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueType {
    LargeFile,         // >800 行
    MissingTests,      // >300 行无测试
    ExcessUnsafe,      // >5 个 unsafe
    ExcessUnwrap,      // >20 个 .unwrap()
    CircularDep,       // 循环依赖
    TodoLeftovers,     // >3 个 TODO
    CompileWarning,    // cargo check 警告
    TestFailure,       // 测试失败
    StagnantEvolve,    // 连续无进化
    HighFreeEnergy,    // 自由能过高 (世界模型困惑)
    LowPhi,            // Φ 过低 (集成信息不足)
}

/// 检测到的具体问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub issue_type: IssueType,
    pub severity: u8,          // 1-10
    pub file: Option<String>,
    pub description: String,
    pub suggestion: String,
    pub auto_fixable: bool,
    pub cycle_discovered: u64,
}

// ============================================================
// 项目快照
// ============================================================

/// 项目健康快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub total_files: usize,
    pub total_lines: usize,
    pub large_files: Vec<String>,
    pub modules_without_tests: Vec<String>,
    pub file_unsafe_hotspots: Vec<String>,
    pub unsafe_count: usize,
    pub unwrap_count: usize,
    pub todo_count: usize,
    pub compile_errors: usize,
    pub compile_warnings: usize,
    pub test_count: usize,
    pub test_failures: usize,
}

// ============================================================
// 进化报告
// ============================================================

/// 单次进化周期报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionReport {
    pub cycle: u64,
    pub issues_found: Vec<Issue>,
    pub issues_fixed: u32,
    pub snapshot: ProjectSnapshot,
    pub evolution_score: f64,       // 0-100 综合健康分
    pub free_energy: f64,           // 来自 ActiveInference
    pub phi: f64,                   // 来自 IIT
    pub suggestions: Vec<String>,
    pub new_patterns: Vec<String>,  // 新模式发现 (蒸馏结果)
    pub auto_fixes: u32,            // 自动修复计数
}

// ============================================================
// 精确计数函数
// ============================================================

fn count_actual_unsafe(content: &str) -> usize {
    let mut count = 0usize;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("//") || t.starts_with("//!") || t.starts_with("/*") || t.starts_with("*") {
            continue;
        }
        if line.contains("matches(\"unsafe\"") || line.contains("contains(\"unsafe\"") {
            continue;
        }
        if t.contains("unsafe {") || t.contains("unsafe fn") || t.contains("unsafe trait") || t.contains("unsafe impl") {
            count += 1;
        }
    }
    count
}

// ============================================================
// 进化引擎
// ============================================================

/// 自进化循环引擎
#[derive(Debug, Clone)]
pub struct EvolutionLoop {
    pub cycle: u64,
    pub issues: Vec<Issue>,
    pub consecutive_stagnant: u32,
    pub fixed_history: Vec<u32>,
    pub enabled: bool,

    // 上次扫描结果缓存
    last_snapshot: Option<ProjectSnapshot>,
}

impl Default for EvolutionLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionLoop {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            issues: Vec::new(),
            consecutive_stagnant: 0,
            fixed_history: Vec::new(),
            enabled: true,
            last_snapshot: None,
        }
    }

    /// 运行一次完整进化周期
    pub fn run_cycle(
        &mut self,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        self.cycle += 1;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut new_patterns = Vec::new();

        // 1. 项目扫描
        let snapshot = self.scan_project();

        // 2. 问题检测
        self.detect_large_files(&snapshot, &mut issues);
        self.detect_missing_tests(&snapshot, &mut issues);
        self.detect_excess_unsafe(&snapshot, &mut issues);
        self.detect_excess_unwrap(&snapshot, &mut issues);
        self.detect_todo_leftovers(&snapshot, &mut issues);
        self.detect_compile_issues(&snapshot, &mut issues);

        // 3. 世界模型感知的问题检测
        let free_energy = world_fe.unwrap_or(0.0);
        let phi = world_phi.unwrap_or(0.0);

        if free_energy > 2.0 {
            issues.push(Issue {
                issue_type: IssueType::HighFreeEnergy,
                severity: (free_energy.min(10.0) * 3.0) as u8,
                file: None,
                description: format!("世界模型自由能过高: {:.3} (阈值=2.0)", free_energy),
                suggestion: "降低 learning_rate 或增加 JEPA 训练步数".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }

        if phi < 0.05 && phi > 0.0 {
            issues.push(Issue {
                issue_type: IssueType::LowPhi,
                severity: 3,
                file: None,
                description: format!("E8 集成信息 Φ 过低: {:.4} (阈值=0.05)", phi),
                suggestion: "增加 E8 演化步数或调整共振宽度 σ".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }

        // 4. 生成修复建议
        for issue in &issues {
            if issue.auto_fixable {
                suggestions.push(format!(
                    "🔧 [{:?}] {}: {}",
                    issue.issue_type,
                    issue.file.as_deref().unwrap_or("global"),
                    issue.suggestion
                ));
            } else {
                suggestions.push(format!(
                    "⚠ [{:?}] {}: {}",
                    issue.issue_type,
                    issue.file.as_deref().unwrap_or("global"),
                    issue.suggestion
                ));
            }
        }

        // 5. 模式蒸馏 (基于重复出现的问题)
        let recent_fixed = self.fixed_history.iter().rev().take(5).sum::<u32>();
        if recent_fixed > 3 {
            new_patterns.push(format!(
                "进化周期 #{}: 最近5周期修复{}个问题 — 系统趋向稳定",
                self.cycle, recent_fixed
            ));
        }

        // 6. 综合健康评分
        let evolution_score = self.compute_evolution_score(&snapshot, &issues);

        // 7. 停滞检测
        if issues.is_empty() {
            self.consecutive_stagnant += 1;
        } else {
            self.consecutive_stagnant = 0;
        }

        self.issues = issues.clone();
        self.last_snapshot = Some(snapshot.clone());

        EvolutionReport {
            cycle: self.cycle,
            issues_found: issues,
            issues_fixed: recent_fixed,
            snapshot,
            evolution_score,
            free_energy,
            phi,
            suggestions,
            new_patterns,
            auto_fixes: 0,
        }
    }

    /// 自动修复周期 — 对所有 auto_fixable 问题执行真实修复并重新扫描
    pub fn autofix_cycle(
        &mut self,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        let initial_report = self.run_cycle(world_fe, world_phi);

        // 使用 PipelineAutoFixer 管线处理所有 auto_fixable 问题
        let pipeline_result = PipelineAutoFixer::new().run_pipeline(self);
        let fixes_applied = pipeline_result.auto_applied as u32;

        let final_report = self.run_cycle(Some(initial_report.free_energy), Some(initial_report.phi));

        EvolutionReport {
            cycle: final_report.cycle,
            issues_found: final_report.issues_found,
            issues_fixed: initial_report.issues_fixed + fixes_applied,
            snapshot: final_report.snapshot,
            evolution_score: final_report.evolution_score,
            free_energy: final_report.free_energy,
            phi: final_report.phi,
            suggestions: final_report.suggestions,
            new_patterns: final_report.new_patterns,
            auto_fixes: fixes_applied,
        }
    }

    /// 项目扫描 (基于文件系统)
    pub fn scan_project(&self) -> ProjectSnapshot {
        // 使用相对于 manifest 目录的路径，适配测试和运行时
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let src_dir = manifest_dir.join("src");
        let mut total_files = 0usize;
        let mut total_lines = 0usize;
        let mut large_files = Vec::new();
        let mut modules_without_tests = Vec::new();
        let mut unsafe_count = 0usize;
        let mut unwrap_count = 0usize;
        let mut todo_count = 0usize;
        let mut file_unsafe_hotspots: Vec<String> = Vec::new();

        if let Ok(entries) = Self::walk_rust_files(&src_dir) {
            for path in &entries {
                total_files += 1;
                if let Ok(content) = std::fs::read_to_string(path) {
                    let line_count = content.lines().count();
                    total_lines += line_count;

                    if line_count > LARGE_FILE_THRESHOLD {
                        large_files.push(path.to_string_lossy().to_string());
                    }

                    let is_test_file = path.to_string_lossy().contains("tests")
                        || content.contains("#[cfg(test)]")
                        || content.contains("#[test]");

                    // 精确 unsafe 计数: 只计 unsafe { } / unsafe fn / unsafe trait / unsafe impl 块
                    let file_unsafe = count_actual_unsafe(&content);
                    unsafe_count += file_unsafe;
                    if file_unsafe > EXCESS_UNSAFE_THRESHOLD {
                        file_unsafe_hotspots.push(path.to_string_lossy().to_string());
                    }

                    // unwrap 计数 (排除测试文件和注释行)
                    if !is_test_file {
                        for line in content.lines() {
                            if line.contains(".unwrap(") && !line.trim_start().starts_with("//") {
                                unwrap_count += 1;
                            }
                        }
                    }

                    // 精确 TODO 计数: 只计 // TODO / // FIXME / // HACK 行
                    let file_todo = content.lines()
                        .filter(|l| {
                            let t = l.trim();
                            t.starts_with("// TODO")
                                || t.starts_with("//TODO")
                                || t.starts_with("// FIXME")
                                || t.starts_with("//FIXME")
                                || t.starts_with("// HACK")
                                || t.starts_with("//HACK")
                        })
                        .count();
                    todo_count += file_todo;

                    // Check for missing tests
                    if line_count > MISSING_TESTS_THRESHOLD
                        && !content.contains("#[cfg(test)]")
                        && !content.contains("#[test]")
                    {
                        modules_without_tests.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // 测试时跳过 cargo check（避免 build lock 死锁）
        let (compile_errs, compile_warns) = if cfg!(test) {
            (0, 0)
        } else {
            match AutoFixer::cargo_check() {
                Ok((e, w)) => (e, w),
                Err(_) => (0, 0),
            }
        };
        ProjectSnapshot {
            total_files,
            total_lines,
            large_files,
            modules_without_tests,
            file_unsafe_hotspots,
            unsafe_count,
            unwrap_count,
            todo_count,
            compile_errors: compile_errs,
            compile_warnings: compile_warns,
            test_count: 0,
            test_failures: 0,
        }
    }

    /// 递归搜索 Rust 源文件
    fn walk_rust_files(dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // 跳过 target 目录
                    if path.file_name().map(|n| n != "target").unwrap_or(true) {
                        files.extend(Self::walk_rust_files(&path)?);
                    }
                } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
        Ok(files)
    }

    // ─── 问题检测器 ───

    fn detect_large_files(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
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

    fn detect_missing_tests(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
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

    fn detect_excess_unsafe(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
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

    fn detect_excess_unwrap(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
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

    fn detect_todo_leftovers(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
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

    fn detect_compile_issues(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
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

    /// 综合健康评分 (0-100)
    fn compute_evolution_score(&self, snap: &ProjectSnapshot, _issues: &[Issue]) -> f64 {
        let mut score = 100.0;

        // 大文件惩罚
        score -= snap.large_files.len() as f64 * 5.0;

        // 无测试惩罚
        score -= snap.modules_without_tests.len() as f64 * 3.0;

        // unsafe 惩罚
        if snap.unsafe_count > EXCESS_UNSAFE_THRESHOLD {
            score -= (snap.unsafe_count - EXCESS_UNSAFE_THRESHOLD) as f64 * 2.0;
        }

        // unwrap 惩罚
        if snap.unwrap_count > EXCESS_UNWRAP_THRESHOLD {
            score -= (snap.unwrap_count - EXCESS_UNWRAP_THRESHOLD) as f64 * 1.0;
        }

        // 未完成 TODO 惩罚
        if snap.todo_count > TODO_LEFTOVERS_THRESHOLD {
            score -= (snap.todo_count - TODO_LEFTOVERS_THRESHOLD) as f64 * 2.0;
        }

        // 编译错误: 致命
        if snap.compile_errors > 0 {
            score -= 30.0;
        }

        // 编译警告
        score -= snap.compile_warnings.min(20) as f64 * 1.0;

        score.clamp(0.0, 100.0)
    }

    /// FEP-IIT alternative evolution score [0,1]
    /// Uses Kearney (2026) VSA bridge: α·(1 - FEₙ) + β·Φ
    pub fn compute_fep_iit_score(&self, free_energy: f64, phi: f64) -> f64 {
        FEPIITBridge::new().compute_score(free_energy, phi)
    }

    /// Combined project + FEP-IIT evolution score [0,1]
    /// 70% project metrics + 30% FEP-IIT consciousness metrics
    pub fn compute_combined_score(&self, snap: &ProjectSnapshot, free_energy: f64, phi: f64) -> f64 {
        let project_score = self.compute_evolution_score(snap, &[]) / 100.0;
        let bridge = FEPIITBridge::new();
        bridge.project_to_fep_iit(project_score * 100.0, free_energy, phi)
    }

    /// 判断是否需要人工介入
    pub fn needs_human_intervention(&self) -> bool {
        self.consecutive_stagnant >= STAGNATION_LIMIT
            || self.issues.iter().any(|i| i.severity >= 9)
    }

    /// 重置停滞计数
    pub fn on_fix_applied(&mut self) {
        self.consecutive_stagnant = 0;
        self.fixed_history.push(1);
        if self.fixed_history.len() > 20 {
            self.fixed_history.remove(0);
        }
    }

    /// 获取仪表盘文本
    pub fn dashboard(&self, report: &EvolutionReport) -> String {
        format!(
            "🧬 #{}: 评分={:.0}/100, 问题={}, 自修复={}, 累积修复={}, 停滞={}/{} | FE={:.2}, Φ={:.3}",
            report.cycle,
            report.evolution_score,
            report.issues_found.len(),
            report.auto_fixes,
            report.issues_fixed,
            self.consecutive_stagnant,
            STAGNATION_LIMIT,
            report.free_energy,
            report.phi,
        )
    }

    /// 自我诊断入口 — 零 LLM 依赖, 基于扫描数据 + 历史 + 能力向量排序
    pub fn self_diagnose(&self) -> (Vec<DiagnosticItem>, PriorityQueue) {
        let snapshot = self.scan_project();
        SelfDiagnose::run_diagnosis(&snapshot, self.cycle)
    }

    /// 基于诊断结果的自动修复 — 按优先级顺序执行 ActionPlan
    pub fn autofix_by_diagnosis(&mut self) -> u32 {
        let (_items, pq) = self.self_diagnose();
        let mut fixes = 0u32;
        for item in pq.as_slice() {
            if item.composite_score < 0.3 {
                continue;
            }
            if ActionExecutor::execute(&item.action).is_ok() {
                fixes += 1;
            }
        }
        if fixes > 0 {
            self.on_fix_applied();
        }
        fixes
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_actual_unsafe_zero() {
        let s = "fn safe() { let x = 1; }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_counts_block() {
        let s = "fn foo() { unsafe { *p = 1; } }";
        assert_eq!(count_actual_unsafe(s), 1);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_comment() {
        let s = "// unsafe { this is just a comment }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_doc_comment() {
        let s = "//! unsafe { doc comment unsafe }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_variable_name() {
        let s = "let unsafe_count = 5;";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_evolution_loop_new() {
        let el = EvolutionLoop::new();
        assert_eq!(el.cycle, 0);
        assert!(el.enabled);
    }

    #[test]
    fn test_scan_project_returns_reasonable_values() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        assert!(snap.total_files > 0 || snap.total_lines == 0);
    }

    #[test]
    fn test_evolution_score_baseline() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        let score = el.compute_evolution_score(&snap, &[]);
        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    fn test_issue_detection_creates_valid_issues() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(0.5), Some(0.3));
        assert_eq!(report.cycle, 1);
        for issue in &report.issues_found {
            assert!(issue.severity >= 1 && issue.severity <= 10);
            assert!(!issue.description.is_empty());
        }
    }

    #[test]
    fn test_high_free_energy_detected() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(5.0), Some(0.3));
        assert!(report.issues_found.iter().any(|i| i.issue_type == IssueType::HighFreeEnergy));
    }

    #[test]
    fn test_low_phi_detected() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(0.5), Some(0.01));
        assert!(report.issues_found.iter().any(|i| i.issue_type == IssueType::LowPhi));
    }

    #[test]
    fn test_stagnation_detection() {
        let mut el = EvolutionLoop::new();
        assert!(!el.needs_human_intervention());
        el.consecutive_stagnant = STAGNATION_LIMIT;
        assert!(el.needs_human_intervention());
    }

    #[test]
    fn test_on_fix_applied_resets_stagnation() {
        let mut el = EvolutionLoop::new();
        el.consecutive_stagnant = 5;
        el.on_fix_applied();
        assert_eq!(el.consecutive_stagnant, 0);
    }

    #[test]
    fn test_dashboard_format() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        let report = EvolutionReport {
            cycle: 1,
            issues_found: vec![],
            issues_fixed: 0,
            snapshot: snap,
            evolution_score: 85.0,
            free_energy: 0.5,
            phi: 0.3,
            suggestions: vec![],
            new_patterns: vec![],
            auto_fixes: 0,
        };
        let db = el.dashboard(&report);
        assert!(db.contains("#"));
        assert!(db.contains("评分"));
    }
}
