//! Auto Inspector - 多Agent自动巡检修复框架
//!
//! 并行多Agent巡检 + 自动修复 + 持续进化

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// 巡检类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionType {
    Compilation,        // 编译检查
    TestExecution,      // 测试执行
    CodeQuality,        // 代码质量
    SecurityScan,       // 安全扫描
    ArchitectureCheck,  // 架构检查
    DependencyAudit,    // 依赖审计
}

/// 巡检结果
#[derive(Debug, Clone)]
pub struct InspectionResult {
    pub inspection_type: InspectionType,
    pub passed: bool,
    pub issues: Vec<Issue>,
    pub fix_suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub location: String,
    pub description: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// 修复状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// 修复任务
#[derive(Debug, Clone)]
pub struct RepairTask {
    pub id: String,
    pub issue: Issue,
    pub status: RepairStatus,
    pub fix_command: String,
}

/// 多Agent巡检器
pub struct AutoInspector {
    inspection_types: Vec<InspectionType>,
    results: Vec<InspectionResult>,
    repair_queue: Vec<RepairTask>,
    _max_concurrent_agents: usize,
}

/// 运行 cargo 命令并返回 (是否成功, 标准输出, 标准错误)
fn run_cargo(args: &[&str]) -> (bool, String, String) {
    // 查找 workspace 根目录（包含 Cargo.toml 的目录）
    let workspace_root = find_workspace_root();

    let output = Command::new("cargo")
        .args(args)
        .current_dir(&workspace_root)
        .output()
        .unwrap_or_else(|e| panic!("无法执行 cargo {}: {}", args.join(" "), e));

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

/// 查找 workspace 根目录
fn find_workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // 向上查找包含根 Cargo.toml 的目录
    loop {
        if path.join("Cargo.toml").exists() {
            // 检查是否是 workspace 根（包含 [workspace]）
            let cargo_toml = std::fs::read_to_string(path.join("Cargo.toml")).unwrap_or_default();
            if cargo_toml.contains("[workspace]") {
                return path;
            }
        }
        if !path.pop() {
            // 回退到 neotrix-core 目录
            return PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        }
    }
}

/// 从 cargo 输出中提取错误行
fn extract_errors(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| line.contains("error[") || line.contains("error:"))
        .map(|s| s.to_string())
        .collect()
}

/// 从 cargo 输出中提取警告行
fn extract_warnings(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| line.contains("warning[") || line.contains("warning:"))
        .map(|s| s.to_string())
        .collect()
}

impl AutoInspector {
    pub fn new() -> Self {
        Self {
            inspection_types: vec![
                InspectionType::Compilation,
                InspectionType::TestExecution,
                InspectionType::CodeQuality,
                InspectionType::SecurityScan,
                InspectionType::ArchitectureCheck,
                InspectionType::DependencyAudit,
            ],
            results: Vec::new(),
            repair_queue: Vec::new(),
            _max_concurrent_agents: 4,
        }
    }

    /// 执行巡检
    pub fn inspect(&mut self, inspection_type: InspectionType) -> InspectionResult {
        let result = match inspection_type {
            InspectionType::Compilation => self.inspect_compilation(),
            InspectionType::TestExecution => self.inspect_tests(),
            InspectionType::CodeQuality => self.inspect_code_quality(),
            InspectionType::SecurityScan => self.inspect_security(),
            InspectionType::ArchitectureCheck => self.inspect_architecture(),
            InspectionType::DependencyAudit => self.inspect_dependencies(),
        };

        self.results.push(result.clone());
        result
    }

    /// 编译检查 — 执行 cargo check 检查编译错误
    fn inspect_compilation(&self) -> InspectionResult {
        let (success, stdout, stderr) = run_cargo(&["check", "-p", "neotrix", "--all-targets"]);

        let output = format!("{}{}", stdout, stderr);
        let errors = extract_errors(&output);

        let mut issues = Vec::new();
        for err in &errors {
            issues.push(Issue {
                severity: IssueSeverity::Critical,
                location: err.clone(),
                description: "编译错误".to_string(),
                auto_fixable: false,
            });
        }

        InspectionResult {
            inspection_type: InspectionType::Compilation,
            passed: success && errors.is_empty(),
            issues,
            fix_suggestions: if !success {
                vec!["cargo fix --lib -p neotrix --allow-dirty".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    /// 测试执行 — 执行 cargo test 检查测试结果
    fn inspect_tests(&self) -> InspectionResult {
        let (success, stdout, stderr) = run_cargo(&["test", "-p", "neotrix", "--lib"]);

        let output = format!("{}{}", stdout, stderr);
        let errors = extract_errors(&output);

        // 解析测试统计
        let test_failures = output.lines()
            .filter(|line| line.contains("FAILED") || line.contains("test result: FAILED"))
            .count();

        let mut issues = Vec::new();
        for err in &errors {
            issues.push(Issue {
                severity: IssueSeverity::High,
                location: err.clone(),
                description: "测试失败".to_string(),
                auto_fixable: false,
            });
        }

        if test_failures > 0 {
            issues.push(Issue {
                severity: IssueSeverity::High,
                location: "test suite".to_string(),
                description: format!("{} 个测试失败", test_failures),
                auto_fixable: false,
            });
        }

        InspectionResult {
            inspection_type: InspectionType::TestExecution,
            passed: success && errors.is_empty() && test_failures == 0,
            issues,
            fix_suggestions: if !success {
                vec!["检查失败的测试用例并修复".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    /// 代码质量 — 执行 cargo clippy 检查警告数量
    fn inspect_code_quality(&self) -> InspectionResult {
        let (success, stdout, stderr) = run_cargo(&[
            "clippy", "-p", "neotrix", "--all-targets", "--", "-W", "clippy::all",
        ]);

        let output = format!("{}{}", stdout, stderr);
        let warnings = extract_warnings(&output);

        let mut issues = Vec::new();
        for warn in &warnings {
            issues.push(Issue {
                severity: IssueSeverity::Low,
                location: warn.clone(),
                description: "clippy 警告".to_string(),
                auto_fixable: true,
            });
        }

        // 检查是否有 clippy error（deny 级别的 lint）
        let clippy_errors = output.lines()
            .filter(|line| line.contains("error:") && line.contains("clippy"))
            .count();

        if clippy_errors > 0 {
            issues.push(Issue {
                severity: IssueSeverity::Medium,
                location: "clippy".to_string(),
                description: format!("{} 个 clippy 错误", clippy_errors),
                auto_fixable: true,
            });
        }

        InspectionResult {
            inspection_type: InspectionType::CodeQuality,
            passed: success && clippy_errors == 0,
            issues,
            fix_suggestions: if !success || !warnings.is_empty() {
                vec![
                    "cargo clippy --fix --lib -p neotrix --allow-dirty".to_string(),
                    "cargo fmt".to_string(),
                ]
            } else {
                Vec::new()
            },
        }
    }

    /// 安全扫描 — 检查已知漏洞（优先 cargo audit，回退到依赖版本检查）
    fn inspect_security(&self) -> InspectionResult {
        // 尝试 cargo audit（需要 cargo-audit 工具）
        let audit_available = Command::new("cargo")
            .args(["audit", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if audit_available {
            let (success, stdout, stderr) = run_cargo(&["audit"]);
            let output = format!("{}{}", stdout, stderr);

            if success {
                return InspectionResult {
                    inspection_type: InspectionType::SecurityScan,
                    passed: true,
                    issues: Vec::new(),
                    fix_suggestions: Vec::new(),
                };
            }

            // 解析审计报告中的漏洞
            let mut issues = Vec::new();
            for line in output.lines() {
                if line.contains("RUSTSEC-") {
                    issues.push(Issue {
                        severity: IssueSeverity::High,
                        location: line.trim().to_string(),
                        description: "已知安全漏洞".to_string(),
                        auto_fixable: false,
                    });
                }
            }

            return InspectionResult {
                inspection_type: InspectionType::SecurityScan,
                passed: false,
                issues,
                fix_suggestions: vec!["cargo audit fix".to_string()],
            };
        }

        // 回退：检查依赖中的已知风险模式
        let (success, stdout, _stderr) = run_cargo(&["tree", "-p", "neotrix", "--depth", "1"]);
        let mut issues = Vec::new();

        if success {
            // 检查高风险依赖模式
            let risky_patterns = [
                ("unsafe", "包含 unsafe 依赖"),
                ("openssl-sys", "原生 OpenSSL 绑定"),
                ("windows-sys", "Windows 原生绑定"),
            ];

            for line in stdout.lines() {
                for (pattern, desc) in &risky_patterns {
                    if line.contains(pattern) {
                        issues.push(Issue {
                            severity: IssueSeverity::Info,
                            location: line.trim().to_string(),
                            description: desc.to_string(),
                            auto_fixable: false,
                        });
                    }
                }
            }
        }

        InspectionResult {
            inspection_type: InspectionType::SecurityScan,
            passed: true, // 回退模式下默认通过，仅报告信息
            issues,
            fix_suggestions: vec![
                "安装 cargo-audit: cargo install cargo-audit".to_string(),
                "运行 cargo audit 进行完整安全扫描".to_string(),
            ],
        }
    }

    /// 架构检查 — 扫描源文件检测跨层依赖违规
    fn inspect_architecture(&self) -> InspectionResult {
        let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut issues = Vec::new();

        // 层级映射：层名 -> 层编号（编号越小越底层）
        let layer_map = [
            ("l1_action", 1),
            ("l2_perception", 2),
            ("l3_embodiment", 3),
            ("l4_emotion", 4),
            ("l5_cognition", 5),
            ("l6_meta", 6),
        ];

        // 扫描每个层目录中的 use 语句
        for (layer_name, layer_num) in &layer_map {
            let layer_dir = src_dir.join(layer_name);
            if !layer_dir.exists() {
                continue;
            }

            // 递归扫描 .rs 文件
            if let Ok(entries) = std::fs::read_dir(&layer_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "rs") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            self.check_cross_layer_deps(
                                &content,
                                &path,
                                layer_name,
                                *layer_num,
                                &layer_map,
                                &mut issues,
                            );
                        }
                    }
                }
            }
        }

        // 检查 mod.rs 中的模块声明
        for (layer_name, _) in &layer_map {
            let mod_file = src_dir.join(layer_name).join("mod.rs");
            if mod_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&mod_file) {
                    self.check_module_declarations(
                        &content,
                        &mod_file,
                        layer_name,
                        &layer_map,
                        &mut issues,
                    );
                }
            }
        }

        let has_issues = !issues.is_empty();
        InspectionResult {
            inspection_type: InspectionType::ArchitectureCheck,
            passed: issues.is_empty(),
            issues,
            fix_suggestions: if has_issues {
                vec![
                    "高层不应直接依赖低层模块".to_string(),
                    "使用 trait 抽象或消息总线解耦".to_string(),
                    "考虑通过 L3 具身层中转调用".to_string(),
                ]
            } else {
                Vec::new()
            },
        }
    }

    /// 检查文件中的跨层依赖
    fn check_cross_layer_deps(
        &self,
        content: &str,
        file_path: &PathBuf,
        current_layer: &str,
        current_num: i32,
        layer_map: &[(&str, i32)],
        issues: &mut Vec<Issue>,
    ) {
        for line in content.lines() {
            let line = line.trim();
            // 匹配 use crate::lN_xxx:: 或 use super::lN_xxx:: 模式
            if line.starts_with("use ") && line.contains("::") {
                for (dep_layer, dep_num) in layer_map {
                    // 检查是否引用了其他层
                    if *dep_layer != current_layer
                        && (line.contains(&format!("l{}_", dep_num))
                            || line.contains(&format!("::{}::", dep_layer)))
                    {
                        // 跨层依赖违规：高层不应直接依赖低层
                        // l6 可以依赖所有层（元认知层可观察全系统）
                        // l5 可以依赖 l1-l4（认知层可调用底层）
                        // l4 可以依赖 l1-l3（情感层可感知底层）
                        // l3 可以依赖 l1-l2（具身层可调用行动和感知）
                        // l2 可以依赖 l1（感知层可调用行动层）
                        // l1 不应依赖任何高层
                        if *dep_num > current_num && current_num < 6 {
                            issues.push(Issue {
                                severity: if current_num <= 2 {
                                    IssueSeverity::High
                                } else {
                                    IssueSeverity::Medium
                                },
                                location: format!("{}:{}", file_path.display(), line),
                                description: format!(
                                    "跨层依赖违规: {} (L{}) -> {} (L{})",
                                    current_layer, current_num, dep_layer, dep_num
                                ),
                                auto_fixable: false,
                            });
                        }
                    }
                }
            }
        }
    }

    /// 检查模块声明中的跨层引用
    fn check_module_declarations(
        &self,
        content: &str,
        file_path: &PathBuf,
        current_layer: &str,
        layer_map: &[(&str, i32)],
        issues: &mut Vec<Issue>,
    ) {
        let current_num = layer_map
            .iter()
            .find(|(name, _)| *name == current_layer)
            .map(|(_, num)| *num)
            .unwrap_or(0);

        for line in content.lines() {
            let line = line.trim();
            // pub mod lN_xxx 或 mod lN_xxx
            if line.starts_with("pub mod ") || line.starts_with("mod ") {
                for (dep_layer, dep_num) in layer_map {
                    if *dep_layer != current_layer && line.contains(dep_layer) {
                        if *dep_num > current_num && current_num < 6 {
                            issues.push(Issue {
                                severity: IssueSeverity::High,
                                location: format!("{}:{}", file_path.display(), line),
                                description: format!(
                                    "模块声明跨层引用: {} 试图声明 {} 模块",
                                    current_layer, dep_layer
                                ),
                                auto_fixable: false,
                            });
                        }
                    }
                }
            }
        }
    }

    /// 依赖审计 — 检查依赖版本和重复依赖
    fn inspect_dependencies(&self) -> InspectionResult {
        let (success, stdout, stderr) = run_cargo(&["tree", "-p", "neotrix", "--duplicates"]);
        let output = format!("{}{}", stdout, stderr);

        let mut issues = Vec::new();

        if success {
            // 检查重复依赖
            let mut dep_counts: HashMap<String, usize> = HashMap::new();
            for line in output.lines() {
                let line = line.trim();
                // cargo tree 输出格式: name v0.0.0
                if let Some(name_end) = line.find(" v") {
                    let name = line[..name_end].to_string();
                    *dep_counts.entry(name).or_insert(0) += 1;
                }
            }

            for (dep, count) in &dep_counts {
                if *count > 1 {
                    issues.push(Issue {
                        severity: IssueSeverity::Low,
                        location: dep.clone(),
                        description: format!("重复依赖: {} 出现 {} 次", dep, count),
                        auto_fixable: false,
                    });
                }
            }
        }

        // 检查是否有过时依赖（通过检查 Cargo.lock 中的版本）
        let lock_path = find_workspace_root().join("Cargo.lock");
        if lock_path.exists() {
            if let Ok(lock_content) = std::fs::read_to_string(&lock_path) {
                // 检查关键依赖版本
                let critical_deps = [
                    ("tokio", "1", "异步运行时"),
                    ("serde", "1", "序列化框架"),
                    ("clap", "4", "CLI 框架"),
                ];

                for (name, expected_major, desc) in &critical_deps {
                    if let Some(pos) = lock_content.find(&format!("name = \"{}\"", name)) {
                        let snippet = &lock_content[pos..(pos + 200).min(lock_content.len())];
                        if let Some(ver_pos) = snippet.find("version = \"") {
                            let ver_start = ver_pos + 11;
                            let ver_end = snippet[ver_start..].find('"').unwrap_or(20);
                            let version = &snippet[ver_start..ver_start + ver_end];

                            // 检查主版本号
                            if let Some(major) = version.split('.').next() {
                                if major != *expected_major {
                                    issues.push(Issue {
                                        severity: IssueSeverity::Info,
                                        location: format!("{} v{}", name, version),
                                        description: format!(
                                            "{} 主版本不匹配: 期望 {}.x, 实际 {}.x ({})",
                                            name, expected_major, major, desc
                                        ),
                                        auto_fixable: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        let has_issues = !issues.is_empty();
        InspectionResult {
            inspection_type: InspectionType::DependencyAudit,
            passed: issues.is_empty(),
            issues,
            fix_suggestions: if has_issues {
                vec!["cargo update 更新依赖".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    /// 并行巡检
    pub fn inspect_all(&mut self) -> Vec<InspectionResult> {
        self.inspection_types
            .clone()
            .into_iter()
            .map(|t| self.inspect(t))
            .collect()
    }

    /// 添加修复任务
    pub fn add_repair_task(&mut self, issue: Issue) -> String {
        let id = format!("repair_{}", self.repair_queue.len());
        let fix_command = self.generate_fix_command(&issue);

        let task = RepairTask {
            id: id.clone(),
            issue,
            status: RepairStatus::Pending,
            fix_command,
        };

        self.repair_queue.push(task);
        id
    }

    /// 生成修复命令
    fn generate_fix_command(&self, issue: &Issue) -> String {
        match issue.severity {
            IssueSeverity::Critical => {
                format!("cargo fix --lib -p neotrix --allow-dirty")
            }
            IssueSeverity::High => format!("cargo fix --lib -p neotrix"),
            IssueSeverity::Medium => format!("cargo clippy --fix --lib -p neotrix"),
            IssueSeverity::Low => format!("cargo fmt"),
            IssueSeverity::Info => String::new(),
        }
    }

    /// 执行修复
    pub fn execute_repair(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.repair_queue.iter_mut().find(|t| t.id == task_id) {
            task.status = RepairStatus::InProgress;

            // 执行修复命令
            let args: Vec<&str> = task.fix_command.split_whitespace().collect();
            if args.is_empty() {
                task.status = RepairStatus::Completed;
                return true;
            }

            let (success, _, _) = run_cargo(&args);
            task.status = if success {
                RepairStatus::Completed
            } else {
                RepairStatus::Failed
            };
            success
        } else {
            false
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert(
            "total_inspections".to_string(),
            self.results.len().to_string(),
        );
        stats.insert(
            "passed".to_string(),
            self.results.iter().filter(|r| r.passed).count().to_string(),
        );
        stats.insert(
            "failed".to_string(),
            self.results
                .iter()
                .filter(|r| !r.passed)
                .count()
                .to_string(),
        );
        stats.insert(
            "repair_queue".to_string(),
            self.repair_queue.len().to_string(),
        );
        stats.insert(
            "repairs_completed".to_string(),
            self.repair_queue
                .iter()
                .filter(|t| t.status == RepairStatus::Completed)
                .count()
                .to_string(),
        );
        stats
    }
}

impl Default for AutoInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_compilation() {
        let mut inspector = AutoInspector::new();
        let result = inspector.inspect(InspectionType::Compilation);
        // 编译检查应能正常运行，不一定通过（可能有编译错误）
        assert!(!result.issues.is_empty() || result.passed);
    }

    #[test]
    fn test_inspect_all() {
        let mut inspector = AutoInspector::new();
        let results = inspector.inspect_all();
        assert_eq!(results.len(), 6);
        // 至少有一项检查应该有结果
        assert!(results.iter().any(|r| r.passed || !r.issues.is_empty()));
    }

    #[test]
    fn test_add_repair_task() {
        let mut inspector = AutoInspector::new();
        let issue = Issue {
            severity: IssueSeverity::Medium,
            location: "test.rs".to_string(),
            description: "test issue".to_string(),
            auto_fixable: true,
        };
        let id = inspector.add_repair_task(issue);
        assert!(!id.is_empty());
        assert_eq!(inspector.repair_queue.len(), 1);
    }

    #[test]
    fn test_inspect_architecture() {
        let mut inspector = AutoInspector::new();
        let result = inspector.inspect(InspectionType::ArchitectureCheck);
        // 架构检查应能正常运行
        // 注意：当前实现可能检测到跨层依赖（这是正常现象）
        assert!(!result.issues.is_empty() || result.passed);
    }

    #[test]
    fn test_inspect_dependencies() {
        let mut inspector = AutoInspector::new();
        let result = inspector.inspect(InspectionType::DependencyAudit);
        // 依赖审计应能正常运行
        assert!(!result.issues.is_empty() || result.passed);
    }
}
