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

use crate::neotrix::nt_mind_autofixer::AutoFixer;
use crate::neotrix::nt_act_code::PipelineAutoFixer;
use crate::neotrix::nt_mind_self_diagnose::{
    ActionExecutor, CodeUnderlyingIssue, DiagnosticItem, EvolutionLoopProvider,
    PriorityQueue, PrioritizedIssue, RepairCircuitBreaker, SelfDiagnose,
};
use crate::neotrix::nt_core_iit_phi::IITPhiCalculator;
use crate::neotrix::nt_world_infer::ActiveInferenceEngine;
pub use crate::neotrix::l1_body_impl::nt_l1_shared_types::IssueType;
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

/// 自愈修复断路器轮次上限 (retry cap, 典型 1-10 次)
/// 防止 autofix 循环空转烧资源 (Claude Code 事故教训: 无上限导致 25 万 API 调用浪费)
pub const REPAIR_MAX_ROUNDS: usize = 10;

// ============================================================
// 问题类型 (re-exported from L1 nt_l1_shared_types via line 20)
// ============================================================

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

    /// 被进化的目标项目目录（None = 自身/Cargo 项目根，保持旧行为）
    pub target_dir: Option<std::path::PathBuf>,

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
            target_dir: None,
            last_snapshot: None,
        }
    }

    /// 创建针对任意目标项目的进化循环
    pub fn for_target(target_dir: impl Into<std::path::PathBuf>) -> Self {
        let mut el = Self::new();
        el.target_dir = Some(target_dir.into());
        el
    }

    /// 运行一次完整进化周期
    pub fn run_cycle(
        &mut self,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        self.run_cycle_in(None, world_fe, world_phi)
    }

    /// 对指定目标目录运行一次完整进化周期（target=None 回落到自身路径）
    pub fn run_cycle_in(
        &mut self,
        target: Option<&std::path::Path>,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        self.cycle += 1;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut new_patterns = Vec::new();

        // 1. 项目扫描
        let snapshot = self.scan_project_in(target);

        // 2. 问题检测
        self.detect_large_files(&snapshot, &mut issues);
        self.detect_missing_tests(&snapshot, &mut issues);
        self.detect_excess_unsafe(&snapshot, &mut issues);
        self.detect_excess_unwrap(&snapshot, &mut issues);
        self.detect_todo_leftovers(&snapshot, &mut issues);
        self.detect_compile_issues(&snapshot, &mut issues);

        // 3. 世界模型感知的问题检测
        //    显式 world_fe/world_phi 优先 (接 ActiveInference/IIT 的真实世界状态);
        //    None 时从项目快照派生 (让 project-evolve 对任意目标项目也产出非零语义值)。
        let (free_energy, phi) = match (world_fe, world_phi) {
            (Some(fe), Some(phi)) => (fe, phi),
            _ => Self::derive_free_energy_phi(&snapshot),
        };

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
        self.autofix_cycle_in(None, world_fe, world_phi)
    }

    /// 对指定目标目录执行自动修复周期
    pub fn autofix_cycle_in(
        &mut self,
        target: Option<&std::path::Path>,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        let initial_report = self.run_cycle_in(target, world_fe, world_phi);

        // 使用 PipelineAutoFixer 管线处理所有 auto_fixable 问题
        let pipeline_result = PipelineAutoFixer::new().run_pipeline(self);
        let fixes_applied = pipeline_result.auto_applied as u32;

        let final_report = self.run_cycle_in(target, Some(initial_report.free_energy), Some(initial_report.phi));

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

    /// 项目扫描 (基于文件系统, 当前 Cargo 项目)
    pub fn scan_project(&self) -> ProjectSnapshot {
        self.scan_project_in(None)
    }

    /// 项目扫描 — 对指定目标目录扫描（target=None 回落自身；target 为非 Rust 项目时仍扫描 .rs 文件并做 cargo check）
    pub fn scan_project_in(&self, target: Option<&std::path::Path>) -> ProjectSnapshot {
        // 目标目录解析: 显式 target > self.target_dir > 自身 Cargo 项目根
        let root = match target {
            Some(t) => t.to_path_buf(),
            None => match &self.target_dir {
                Some(t) => t.clone(),
                None => std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf(),
            },
        };
        let src_dir = if root.join("src").is_dir() {
            root.join("src")
        } else {
            root.clone()
        };
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
            match AutoFixer::cargo_check_in(Some(&root)) {
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
        const SKIP_DIRS: [&str; 5] = ["target", ".git", ".backup", "node_modules", "_archive"];
        let mut files = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // 跳过 target/.git/.backup/node_modules 等非源码目录
                    if path
                        .file_name()
                        .map(|n| !SKIP_DIRS.contains(&n.to_str().unwrap_or("")))
                        .unwrap_or(true)
                    {
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

    /// 从项目快照派生 free_energy / phi (供无世界模型上下文的调用方使用)。
    ///
    /// 语义:
    /// - `free_energy`: ActiveInference 预测误差 — 项目风险维度 (issue 密度/unsafe/unwrap/
    ///   TODO/大文件/编译警告) 越高, 自由能越高。
    /// - `phi`: IIT 集成信息 — 将健康维度构成状态向量, 度量各健康维度间的
    ///   共振整合度; 纯噪声/均衡态 → 低 Φ, 结构化分化 → 高 Φ。
    fn derive_free_energy_phi(snap: &ProjectSnapshot) -> (f64, f64) {
        let file_n = snap.total_files.max(1) as f64;

        // 项目风险密度 (每文件归一化)
        let issue_density = (snap.large_files.len() as f64 + snap.modules_without_tests.len() as f64) / file_n;
        let unsafe_density = snap.unsafe_count as f64 / file_n;
        let unwrap_density = snap.unwrap_count as f64 / file_n;
        let todo_density = snap.todo_count as f64 / file_n;
        let warning_density = snap.compile_warnings as f64 / file_n;

        // ActiveInference 变分自由能: F = β·E_JEPA - H(E8)/T + γ·|∇E8|
        //   jepa_energy(预测能量)   = 风险密度 (项目越脏, 世界模型预测误差越大)
        //   e8_entropy(E8 状态熵)   = 结构复杂度 (文件量级信息)
        //   e8_energy_gradient      = 0 (单次扫描无时间演化)
        let jepa_energy = issue_density + unsafe_density + unwrap_density + todo_density + warning_density;
        let e8_entropy = (1.0 + snap.total_lines as f64).ln();
        let free_energy = ActiveInferenceEngine::new()
            .compute_free_energy(jepa_energy, e8_entropy, 0.0)
            .variational_fe;

        // IIT Φ: 健康维度状态向量 → 共振集成度
        //   state = [health_ratio, large_file_ratio, no_test_ratio, unsafe_ratio,
        //            unwrap_ratio, todo_ratio, warning_ratio] (0..1 归一)
        let (large_ratio, test_ratio, unsafe_ratio) = (
            snap.large_files.len() as f64 / file_n,
            snap.modules_without_tests.len() as f64 / file_n,
            snap.unsafe_count as f64 / file_n,
        );
        let state = vec![
            1.0 - (jepa_energy / 4.0).min(1.0), // 健康度 (逆风险密度)
            large_ratio,
            test_ratio,
            unsafe_ratio,
            unwrap_density / 4.0,
            todo_density / 2.0,
            warning_density,
        ];
        let phi = IITPhiCalculator::new().compute_phi(&state).phi;

        (free_energy, phi)
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
        let snapshot = self.scan_project_in(None);
        SelfDiagnose::run_diagnosis(&snapshot, self.cycle)
    }

    /// 基于诊断结果的自动修复 — 按优先级顺序执行 ActionPlan
    ///
    /// 循环断路器接线 (R-P79): 每次 autofix 会话持有一个 RepairCircuitBreaker,
    /// 逐 item 执行前检查断路器; 跳闸 (轮次超限或连续无进展) 即停止后续修复,
    /// 防止自愈循环空转 (retry cap / loop detection)。
    pub fn autofix_by_diagnosis(&mut self) -> u32 {
        let (_items, pq) = self.self_diagnose();
        let mut fixes = 0u32;
        let mut breaker = RepairCircuitBreaker::new(REPAIR_MAX_ROUNDS);
        for item in pq.as_slice() {
            if item.composite_score < 0.3 {
                continue;
            }
            if breaker.is_tripped() {
                log::warn!("[EvolutionLoop] 修复断路器已跳闸, 停止自动修复");
                break;
            }
            match ActionExecutor::execute_with_breaker(&ActionExecutor, &item.action, &mut breaker) {
                Ok(_) => fixes += 1,
                Err(e) if breaker.is_tripped() => {
                    log::warn!("[EvolutionLoop] 修复断路器跳闸: {}", e);
                    break;
                }
                Err(_) => {}
            }
        }
        if fixes > 0 {
            self.on_fix_applied();
        }
        fixes
    }
}

impl EvolutionLoopProvider for EvolutionLoop {
    fn self_diagnose(&mut self) -> (Vec<String>, Vec<PrioritizedIssue>) {
        let (items, pq) = Self::self_diagnose(self);
        let issues: Vec<PrioritizedIssue> = pq.into_vec().into_iter().map(|di| {
            PrioritizedIssue {
                action: di.action,
                composite_score: di.composite_score,
                underlying_issue: CodeUnderlyingIssue {
                    file: di.underlying_issue.file.clone(),
                    issue_type: format!("{:?}", di.underlying_issue.issue_type),
                },
            }
        }).collect();
        let messages: Vec<String> = items.into_iter()
            .map(|d| format!("[{:?}] {} (score={:.2})", d.underlying_issue.issue_type, d.underlying_issue.description, d.composite_score))
            .collect();
        (messages, issues)
    }

    fn on_fix_applied(&mut self) {
        self.on_fix_applied()
    }
}

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

    #[test]
    fn test_for_target_sets_target_dir() {
        let el = EvolutionLoop::for_target("/tmp/mock-project");
        assert!(el.target_dir.is_some());
        assert_eq!(el.target_dir.as_deref().unwrap(), std::path::Path::new("/tmp/mock-project"));
    }

    #[test]
    fn test_scan_project_in_arbitrary_dir() {
        // 构造一个临时 mock Rust 项目目录
        let dir = std::env::temp_dir().join(format!("nt-evolve-mock-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("main.rs"), "// TODO: fix me\nfn main() { let x = 1; }\n").expect("write mock main");
        std::fs::write(src.join("hot.rs"), "unsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\n").expect("write mock hot");

        let el = EvolutionLoop::new();
        let snap = el.scan_project_in(Some(&dir));
        assert!(snap.total_files >= 2, "expected >=2 files, got {}", snap.total_files);
        assert!(snap.todo_count >= 1, "expected TODO detected, got {}", snap.todo_count);
        assert!(snap.file_unsafe_hotspots.len() >= 1, "expected unsafe hotspot");
        assert!(snap.unsafe_count >= 6, "expected >=6 unsafe, got {}", snap.unsafe_count);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_run_cycle_in_target_reports_target_snapshot() {
        let dir = std::env::temp_dir().join(format!("nt-evolve-cycle-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("lib.rs"), "// TODO x\n").expect("write mock lib");

        let mut el = EvolutionLoop::new();
        let report = el.run_cycle_in(Some(&dir), None, None);
        assert_eq!(report.cycle, 1);
        assert!(report.snapshot.todo_count >= 1, "expected TODO in target, got {}", report.snapshot.todo_count);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_derive_free_energy_phi_non_finite_or_zero() {
        // 显式接世界模型值时应原样透传 (旧语义不被破坏)
        let dir = std::env::temp_dir().join(format!("nt-evolve-fe-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("lib.rs"), "pub fn f() {}\n").expect("write mock lib");

        let mut el = EvolutionLoop::new();
        let report = el.run_cycle_in(Some(&dir), Some(0.5), Some(0.3));
        assert_eq!(report.free_energy, 0.5);
        assert_eq!(report.phi, 0.3);

        // None 时从快照派生 (不崩溃, 值为有限数)
        let report2 = el.run_cycle_in(Some(&dir), None, None);
        assert!(report2.free_energy.is_finite(), "free_energy must be finite, got {}", report2.free_energy);
        assert!(report2.phi.is_finite(), "phi must be finite, got {}", report2.phi);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_derive_free_energy_phi_dirty_vs_clean() {
        // 脏项目 (多问题) 自由能应高于干净项目 (风险驱动)
        let clean = ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: vec![],
            unsafe_count: 0,
            unwrap_count: 1,
            todo_count: 1,
            test_count: 20,
            test_failures: 0,
            compile_errors: 0,
            compile_warnings: 0,
        };
        let dirty = ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec!["big.rs".into()],
            modules_without_tests: vec!["m.rs".into()],
            file_unsafe_hotspots: vec!["u.rs".into()],
            unsafe_count: 8,
            unwrap_count: 40,
            todo_count: 6,
            test_count: 2,
            test_failures: 1,
            compile_errors: 0,
            compile_warnings: 5,
        };
        let (fe_clean, phi_clean) = EvolutionLoop::derive_free_energy_phi(&clean);
        let (fe_dirty, phi_dirty) = EvolutionLoop::derive_free_energy_phi(&dirty);
        assert!(fe_dirty > fe_clean, "dirty FE ({}) must exceed clean FE ({})", fe_dirty, fe_clean);
        assert!(fe_dirty.is_finite() && phi_dirty.is_finite());
    }
}
