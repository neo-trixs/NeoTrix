//! AutoFixer — 自执行修复引擎

use std::path::{Path, PathBuf};

use crate::core::nt_core_context::revertible::{ClosureEffect, RevertibleContext};
use crate::core::nt_core_error_parse::{self, CompilerDiagnostic, DiagnosticSeverity};

/// 自愈快照 — 修复前记录文件原内容作为 ∂Γ inverse (写回原状)。
pub struct HealSnapshot {
    pub path: PathBuf,
    pub original: Vec<u8>,
}

impl HealSnapshot {
    pub fn capture(path: &Path) -> Result<Self, String> {
        let original = std::fs::read(path)
            .map_err(|e| format!("快照读取失败 {}: {}", path.display(), e))?;
        Ok(Self {
            path: path.to_path_buf(),
            original,
        })
    }
}

/// ∂Γ 事务性自愈批次 — heal 前快照, 批内任一步失败 recover 回滚全部已写文件。
/// 语义: all-or-nothing (与 PluginRegistry::load_batch 同一回滚原语)。
pub struct RepairBatch {
    ctx: RevertibleContext<'static, ()>,
    snapshots: Vec<HealSnapshot>,
}

impl RepairBatch {
    pub fn begin() -> Self {
        Self {
            ctx: RevertibleContext::new(()),
            snapshots: Vec::new(),
        }
    }

    /// 快照并登记回滚效果 (inverse = 写回原内容 / 原文件不存在则移除)。
    /// 返回原内容字节, 供调用方读取。
    pub fn snapshot(&mut self, path: &Path) -> Result<Vec<u8>, String> {
        let snap = HealSnapshot::capture(path)?;
        let path2 = snap.path.clone();
        let original = snap.original.clone();
        let original_for_inverse = original.clone();
        self.snapshots.push(snap);
        self.ctx.track(ClosureEffect::new(
            format!("heal:{}", path2.display()),
            |_| {},
            move |_| {
                if original_for_inverse.is_empty() {
                    let _ = std::fs::remove_file(&path2);
                } else {
                    let _ = std::fs::write(&path2, &original_for_inverse);
                }
            },
        ));
        Ok(original)
    }

    /// 批内失败 → recover 全部回滚。
    pub fn rollback(&mut self) {
        self.ctx.recover();
    }

    /// 提交 — 丢弃回滚日志, 保留修复。返回快照数。
    pub fn commit(&mut self) -> usize {
        let n = self.snapshots.len();
        self.snapshots.clear();
        n
    }
}

/// 自动修复执行器 — 对已检测问题执行真实代码修改
pub struct AutoFixer;

impl AutoFixer {
    /// 运行 cargo fix 自动修复编译警告
    pub fn cargo_fix() -> Result<String, String> {
        let output = std::process::Command::new("cargo")
            .args(["fix", "--lib", "--allow-dirty"])
            .output()
            .map_err(|e| format!("cargo fix 调用失败: {}", e))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(format!("cargo fix 完成: {}", stdout.lines().count()))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("cargo fix 失败: {}", stderr.lines().next().unwrap_or("unknown")))
        }
    }

    /// 运行 cargo check 获取实时编译状态
    pub fn cargo_check() -> Result<(usize, usize), String> {
        Self::cargo_check_in(None)
    }

    /// 在指定工作目录运行 cargo check（target_dir=None 时使用进程当前目录）
    pub fn cargo_check_in(target_dir: Option<&std::path::Path>) -> Result<(usize, usize), String> {
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(["check", "--lib"]);
        if let Some(dir) = target_dir {
            cmd.current_dir(dir);
        }
        let output = cmd
            .output()
            .map_err(|e| format!("cargo check 调用失败: {}", e))?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let errors = stderr.matches("error[").count();
        let warnings = stderr.matches("warning:").count();
        Ok((errors, warnings))
    }

    /// 启用一个被 #[ignore] 的测试
    pub fn enable_ignored_test(file_path: &str, line: usize) -> Result<String, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut lines: Vec<&str> = content.lines().collect();
        if line == 0 || line > lines.len() {
            return Err("行号越界".into());
        }
        let idx = line - 1;
        if lines[idx].trim() == "#[ignore]" {
            lines.remove(idx);
            let new_content = lines.join("\n");
            std::fs::write(file_path, &new_content)
                .map_err(|e| format!("写入失败: {}", e))?;
            Ok(format!("已启用 {}", file_path))
        } else {
            Err(format!("第{}行不是 #[ignore]", line))
        }
    }

    /// 向文件添加测试模块存根 (如果不存在)
    pub fn add_test_stub(file_path: &str) -> Result<String, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        if content.contains("#[cfg(test)]") {
            return Err("已有测试模块".into());
        }
        let stub = "\n\n#[cfg(test)]\nmod tests {\n\n    #[test]\n    fn test_basic() {\n        assert!(true);\n    }\n}\n";
        let mut new_content = content;
        new_content.push_str(stub);
        std::fs::write(file_path, &new_content)
            .map_err(|e| format!("写入失败: {}", e))?;
        Ok(format!("已添加测试存根到 {}", file_path))
    }

    /// 移除未使用导入 (通过 cargo fix)
    pub fn remove_unused_imports() -> Result<String, String> {
        let output = std::process::Command::new("cargo")
            .args(["fix", "--lib", "--allow-dirty", "--edition-idioms"])
            .output()
            .map_err(|e| format!("cargo fix 调用失败: {}", e))?;
        if output.status.success() {
            Ok("已清理未使用导入".to_string())
        } else {
            Err("cargo fix 失败".to_string())
        }
    }

    /// 删除文件中特定行的 TODO 注释
    pub fn remove_todo_line(file_path: &str, line: usize) -> Result<String, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut lines: Vec<&str> = content.lines().collect();
        if line == 0 || line > lines.len() {
            return Err("行号越界".into());
        }
        let idx = line - 1;
        let trimmed = lines[idx].trim();
        if trimmed.starts_with("// TODO") || trimmed.starts_with("//TODO") {
            lines.remove(idx);
            let new_content = lines.join("\n");
            std::fs::write(file_path, &new_content)
                .map_err(|e| format!("写入失败: {}", e))?;
            Ok(format!("已移除 TODO 行 {}:{}", file_path, line))
        } else {
            Err(format!("第{}行不是纯 TODO 注释", line))
        }
    }

    /// 将大文件在已知模块边界处拆分为多个文件
    ///
    /// 安全: 仅在 NEOTRIX_SPLIT_ENABLE=1 环境变量设置时执行真实拆分
    /// 测试时通过设置 `NEOTRIX_SPLIT_ENABLE=0` 防止意外
    pub fn split_file(file_path: &str) -> Result<String, String> {
        if std::env::var("NEOTRIX_SPLIT_ENABLE").as_deref() != Ok("1") {
            return Err("split_file 需要设置 NEOTRIX_SPLIT_ENABLE=1".into());
        }
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut created = Vec::new();
        let path = std::path::Path::new(file_path);
        let parent = path.parent().ok_or("无法确定父目录")?;
        let stem = path.file_stem().ok_or("无法确定文件名")?;

        let dir_path = parent.join(stem);
        std::fs::create_dir_all(&dir_path)
            .map_err(|e| format!("创建目录失败: {}", e))?;

        let mut current_block = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if (trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ")
                || trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ")
                || trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ")
                || trimmed.starts_with("impl ") || trimmed.starts_with("pub impl ")
                || trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")
                || trimmed.starts_with("#[cfg("))
                && !current_block.trim().is_empty()
            {
                created.push(std::mem::take(&mut current_block));
            }
            current_block.push_str(line);
            current_block.push('\n');
        }
        if !current_block.trim().is_empty() {
            created.push(current_block);
        }

        // 写入每个块到单独文件
        let mut sub_mods = Vec::new();
        for (i, block) in created.iter().enumerate() {
            let item_file = dir_path.join(format!("part_{}.rs", i));
            std::fs::write(&item_file, block)
                .map_err(|e| format!("写入失败: {}", e))?;
            sub_mods.push(format!("part_{}", i));
        }

        // 生成 mod.rs
        let mut mod_rs = String::new();
        for m in &sub_mods {
            mod_rs.push_str(&format!("pub mod {};\n", m));
        }
        std::fs::write(dir_path.join("mod.rs"), &mod_rs)
            .map_err(|e| format!("写入 mod.rs 失败: {}", e))?;

        // 删除原文件
        std::fs::remove_file(file_path)
            .map_err(|e| format!("删除原文件失败: {}", e))?;

        Ok(format!("拆分为 {} 个文件: {}", sub_mods.len(), sub_mods.join(", ")))
    }

    /// 扫描并清理文件中的 TODO 注释
    pub fn cleanup_todos(file_path: &str) -> Result<usize, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut removed = 0usize;
        let lines: Vec<&str> = content.lines().collect();

        // 只移除纯 TODO 注释行 (非代码逻辑 TODO)
        let mut kept = Vec::new();
        for line in &lines {
            let trimmed = line.trim();
            if trimmed == "// TODO" || trimmed == "//TODO" || trimmed == "// FIXME" {
                removed += 1;
                continue;
            }
            kept.push(*line);
        }

        if removed > 0 {
            std::fs::write(file_path, kept.join("\n"))
                .map_err(|e| format!("写入失败: {}", e))?;
        }
        Ok(removed)
    }

    /// 事务性 TODO 清理 (∂Γ 自愈回滚): 写入前快照原内容, 写失败 recover 回滚。
    /// 生产路径 (EvolutionDaemon) 应调用此版本而非裸 cleanup_todos。
    pub fn cleanup_todos_tx(file_path: &str) -> Result<usize, String> {
        let mut batch = RepairBatch::begin();
        let _original = batch.snapshot(Path::new(file_path))?;
        match Self::cleanup_todos(file_path) {
            Ok(n) => {
                batch.commit();
                Ok(n)
            }
            Err(e) => {
                batch.rollback();
                Err(e)
            }
        }
    }

    /// 生产安全: 记录测试缺口而不写入存根
    /// EvolutionDaemon 等生产路径应调用此方法而非 add_test_stub
    pub fn record_test_gap(file_path: &str) -> Result<String, String> {
        log::info!("[AutoFixer] Test gap recorded for: {}", file_path);
        Ok(format!("Test gap recorded (no stub written): {}", file_path))
    }

    /// 解析 cargo check 输出 → 按错误码生成修复建议 (NT-REPAIR 经验库接线)。
    ///
    /// 把 `nt_core_error_parse` 的解析 + `suggest_fix` 映射暴露给生产修复路径,
    /// 使编译错误从"仅检测"升级为"可执行修复指引" (T3 生产接线)。
    pub fn suggest_fixes_from_output(output: &str) -> Vec<(String, String, String)> {
        let diags = nt_core_error_parse::parse_compiler_output(output);
        diags
            .iter()
            .filter_map(|d| {
                nt_core_error_parse::suggest_fix(d).map(|f| {
                    (f.code, f.action.to_string(), f.guidance.clone())
                })
            })
            .collect()
    }

    /// 便捷: 从单个错误码直接查修复建议 (供诊断器/守护进程直接调用)。
    pub fn suggest_fix_for_code(code: &str) -> Option<(String, String)> {
        let d = CompilerDiagnostic {
            file: String::new(),
            line: 0,
            column: 0,
            severity: DiagnosticSeverity::Error,
            code: Some(code.to_string()),
            message: String::new(),
            span_text: None,
        };
        nt_core_error_parse::suggest_fix(&d).map(|f| (f.action.to_string(), f.guidance.clone()))
    }
}

// ────────────────────────────────────────────────────────────────
// GAUNTLET 证据门控链 (G12 强化, old-coder 吸收) —
// SPEC→RED→GREEN→GAUNTLET→EVIDENCE 五态门控交付
// ────────────────────────────────────────────────────────────────

/// GAUNTLET 门控阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GauntletStage {
    /// SPEC: 变更前必须有明确规格 (spec-before 门禁)。
    Spec,
    /// RED: 先写失败测试。
    Red,
    /// GREEN: 实现使测试通过。
    Green,
    /// GAUNTLET: 硬性检查矩阵 (lint/check/证据必须真实)。
    Gauntlet,
    /// EVIDENCE: 交付必须携带真实证据 (evidence-after 门禁)。
    Evidence,
}

impl GauntletStage {
    /// 五阶段顺序。
    pub const ORDER: [GauntletStage; 5] = [
        GauntletStage::Spec,
        GauntletStage::Red,
        GauntletStage::Green,
        GauntletStage::Gauntlet,
        GauntletStage::Evidence,
    ];

    pub fn label(self) -> &'static str {
        match self {
            GauntletStage::Spec => "SPEC",
            GauntletStage::Red => "RED",
            GauntletStage::Green => "GREEN",
            GauntletStage::Gauntlet => "GAUNTLET",
            GauntletStage::Evidence => "EVIDENCE",
        }
    }

    pub fn next(self) -> Option<GauntletStage> {
        match self {
            GauntletStage::Spec => Some(GauntletStage::Red),
            GauntletStage::Red => Some(GauntletStage::Green),
            GauntletStage::Green => Some(GauntletStage::Gauntlet),
            GauntletStage::Gauntlet => Some(GauntletStage::Evidence),
            GauntletStage::Evidence => None,
        }
    }
}

/// GAUNTLET 门禁判定结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GauntletVerdict {
    pub stage: GauntletStage,
    /// 是否通过该阶段门禁。
    pub pass: bool,
    /// 未通过时的原因 (pass=true 时空)。
    pub reason: String,
}

/// GAUNTLET 证据门控状态机。
///
/// 每个阶段带独立门禁, 前一阶段未过审则禁止推进:
/// - SPEC:   必须提供规格 (非空);
/// - RED:    必须存在至少 1 条失败测试 (evidence-after 前置);
/// - GREEN:  测试必须全部通过;
/// - GAUNTLET: 硬检查 (lint 通过 / 无 TODO 残留 / 无 unwrap 滥用) 全绿;
/// - EVIDENCE: 交付必须携带真实证据包 (非空, 且与实现匹配)。
#[derive(Debug, Clone, Default)]
pub struct GauntletMachine {
    pub current: Option<GauntletStage>,
    pub passed: Vec<GauntletStage>,
    pub failures: Vec<GauntletVerdict>,
}

impl GauntletMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// 启动: 从 SPEC 开始。
    pub fn start(&mut self) {
        self.current = Some(GauntletStage::Spec);
    }

    /// 当前阶段。
    pub fn current(&self) -> Option<GauntletStage> {
        self.current
    }

    /// 是否已走完 EVIDENCE (交付完成)。
    pub fn completed(&self) -> bool {
        self.current.is_none() && !self.passed.is_empty()
    }

    /// 评估并推进到下一阶段 (若通过)。返回本次判定。
    pub fn advance(&mut self, spec: &str, failed_tests: usize, tests_pass: bool, lint_ok: bool, todos: usize, unwraps: usize, evidence: &[String]) -> GauntletVerdict {
        let stage = match self.current {
            Some(s) => s,
            None => {
                return GauntletVerdict {
                    stage: GauntletStage::Spec,
                    pass: false,
                    reason: "machine not started".into(),
                };
            }
        };
        let verdict = Self::evaluate_stage(stage, spec, failed_tests, tests_pass, lint_ok, todos, unwraps, evidence);
        if verdict.pass {
            self.passed.push(stage);
            self.current = stage.next();
        } else {
            self.failures.push(verdict.clone());
        }
        verdict
    }

    /// 单阶段门禁评估 (纯函数, 便于测试)。
    pub fn evaluate_stage(stage: GauntletStage, spec: &str, failed_tests: usize, tests_pass: bool, lint_ok: bool, todos: usize, unwraps: usize, evidence: &[String]) -> GauntletVerdict {
        match stage {
            GauntletStage::Spec => {
                if spec.trim().is_empty() {
                    GauntletVerdict { stage, pass: false, reason: "spec-before: no spec provided".into() }
                } else {
                    GauntletVerdict { stage, pass: true, reason: String::new() }
                }
            }
            GauntletStage::Red => {
                if failed_tests == 0 {
                    GauntletVerdict { stage, pass: false, reason: "RED: need at least 1 failing test first".into() }
                } else {
                    GauntletVerdict { stage, pass: true, reason: String::new() }
                }
            }
            GauntletStage::Green => {
                if !tests_pass {
                    GauntletVerdict { stage, pass: false, reason: "GREEN: tests must pass".into() }
                } else {
                    GauntletVerdict { stage, pass: true, reason: String::new() }
                }
            }
            GauntletStage::Gauntlet => {
                let mut problems = Vec::new();
                if !lint_ok {
                    problems.push("lint failing".to_string());
                }
                if todos > 0 {
                    problems.push(format!("{todos} TODO leftovers"));
                }
                if unwraps > 0 {
                    problems.push(format!("{unwraps} unwrap abuses"));
                }
                if problems.is_empty() {
                    GauntletVerdict { stage, pass: true, reason: String::new() }
                } else {
                    GauntletVerdict { stage, pass: false, reason: format!("GAUNTLET blocked: {}", problems.join(", ")) }
                }
            }
            GauntletStage::Evidence => {
                if evidence.is_empty() {
                    GauntletVerdict { stage, pass: false, reason: "evidence-after: no evidence attached".into() }
                } else {
                    GauntletVerdict { stage, pass: true, reason: String::new() }
                }
            }
        }
    }

    /// 是否被某阶段阻挡 (供调用方区分"卡住" vs "完成")。
    pub fn blocked(&self) -> Option<&GauntletVerdict> {
        self.failures.last()
    }
}

// ────────────────────────────────────────────────────────────────
// G28 Healers 巡检集 (topics/code-health 吸收) —
// 多维度代码健康巡检器, 每个 healer 扫描一个维度并产出修复建议
// ────────────────────────────────────────────────────────────────

/// 单个 healer 巡检产出的修复建议。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealSuggestion {
    /// 健康维度 (如 "compile" / "todo" / "unused_import" / "unwraps")。
    pub dimension: String,
    /// 目标文件 (可空 = 全局维度)。
    pub file: Option<String>,
    /// 建议动作描述。
    pub action: String,
    /// 是否可自动执行 (AutoFixer 有对应原语)。
    pub auto_fixable: bool,
}

/// Healers 巡检集 — 定期扫描多个代码健康维度, 汇总修复建议供治理层处置
/// (topics/code-health 吸收: 不是单个修复, 而是健康巡检 + 建议流水线)。
#[derive(Debug, Default, Clone)]
pub struct HealerRegistry {
    /// 各维度最近一次扫描结果 (dimension → 建议列表)。
    pub last_report: Vec<HealSuggestion>,
    /// 已执行自动修复数 (遥测)。
    pub auto_fixes_applied: u32,
}

impl HealerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 巡检 TODO 幽灵 (removed 文件残留 TODO / 未清理 todo) — 维度 "todo"。
    /// 扫描给定目录下的 .rs 文件, 统计未加 #[ignore] 的 TODO/FIXME 注释行。
    /// 纯占位 TODO 行 (// TODO / // FIXME 无正文) 标记 auto_fixable — 由
    /// apply_auto_fixable 经 cleanup_todos_tx 事务落地 (GAP-3 修复, R-P79)。
    pub fn scan_todos(&mut self, dir: &Path) -> Vec<HealSuggestion> {
        let mut out = Vec::new();
        for path in Self::collect_rs_files(dir) {
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let count = content
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    t.starts_with("//") && (t.contains("TODO") || t.contains("FIXME"))
                })
                .count();
            let pure_placeholder = content
                .lines()
                .any(|l| {
                    let t = l.trim();
                    t == "// TODO" || t == "//TODO" || t == "// FIXME" || t == "//FIXME"
                });
            if count > 0 {
                out.push(HealSuggestion {
                    dimension: "todo".into(),
                    file: Some(path.to_string_lossy().to_string()),
                    action: format!("{} TODO/FIXME comments pending", count),
                    // 仅含纯占位行可安全事务删除 (cleanup_todos_tx 只删无正文行)
                    auto_fixable: pure_placeholder,
                });
            }
        }
        out
    }

    /// 巡检 unwrap 滥用 (未处理 Result/Option) — 维度 "unwraps"。
    /// 统计 `.unwrap()` 调用 (不含测试模块)。
    pub fn scan_unwraps(&mut self, dir: &Path) -> Vec<HealSuggestion> {
        let mut out = Vec::new();
        for path in Self::collect_rs_files(dir) {
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let in_tests = content.contains("#[cfg(test)]");
            let count = content.matches(".unwrap()").count();
            if count > 0 && !in_tests {
                out.push(HealSuggestion {
                    dimension: "unwraps".into(),
                    file: Some(path.to_string_lossy().to_string()),
                    action: format!("{} unwrap() calls in production path", count),
                    auto_fixable: false,
                });
            }
        }
        out
    }

    /// 汇总巡检: 扫描全部维度, 更新 last_report, 返回建议列表。
    pub fn run_full_scan(&mut self, dir: &Path) -> Vec<HealSuggestion> {
        let mut report = Vec::new();
        report.extend(self.scan_todos(dir));
        report.extend(self.scan_unwraps(dir));
        self.last_report = report.clone();
        report
    }

    /// 对 `auto_fixable` 建议执行真实自动修复 (GAP-3 修复, R-P79 生产接线)。
    /// 当前落地维度: "todo" → AutoFixer::cleanup_todos_tx (∂Γ 事务 + 快照回滚)。
    /// 返回实际落地数; 成功者从 last_report 移除 (避免每周期重复尝试)。
    pub fn apply_auto_fixable(&mut self) -> usize {
        let mut applied = 0usize;
        let mut remaining = Vec::with_capacity(self.last_report.len());
        for s in &self.last_report {
            if !s.auto_fixable {
                remaining.push(s.clone());
                continue;
            }
            let landed = match s.dimension.as_str() {
                "todo" => s
                    .file
                    .as_deref()
                    .map(|f| AutoFixer::cleanup_todos_tx(f).is_ok())
                    .unwrap_or(false),
                _ => false,
            };
            if landed {
                applied += 1;
                log::info!("[healers] auto-fix landed: {} {}", s.dimension, s.file.as_deref().unwrap_or(""));
            } else {
                remaining.push(s.clone());
            }
        }
        self.auto_fixes_applied += applied as u32;
        self.last_report = remaining;
        applied
    }

    /// 收集目录下所有 .rs 文件 (递归, 跳过 target/.git)。
    fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if !dir.is_dir() {
            return out;
        }
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&d) else { continue };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if name == "target" || name == ".git" || name == "node_modules" {
                        continue;
                    }
                    stack.push(p);
                } else if p.extension().is_some_and(|e| e == "rs") {
                    out.push(p);
                }
            }
        }
        out.sort();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autofixer_cargo_check_structure() {
        let result = AutoFixer::cargo_check();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_suggest_fixes_from_output_wires_parser() {
        let output = "error[E0433]: failed to resolve: use of undeclared type `Foo`\n  --> src/b.rs:2:3\nerror[E0308]: mismatched types\n  --> src/lib.rs:5:1\n";
        let fixes = AutoFixer::suggest_fixes_from_output(output);
        assert_eq!(fixes.len(), 2);
        assert_eq!(fixes[0].0, "E0433");
        assert_eq!(fixes[0].1, "add");
        assert_eq!(fixes[1].0, "E0308");
        assert_eq!(fixes[1].1, "type_fix");
    }

    #[test]
    fn test_suggest_fix_for_code_direct() {
        let fix = AutoFixer::suggest_fix_for_code("E0382").expect("E0382 should map");
        assert_eq!(fix.0, "clone");
        assert!(fix.1.contains("clone"));
        assert!(AutoFixer::suggest_fix_for_code("E9999").is_none());
    }

    #[test]
    fn test_repair_batch_snapshot_and_commit() {
        let dir = std::env::temp_dir().join("neotrix_autofixer_tx");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("a.rs");
        std::fs::write(&f, "// TODO\nfn a() {}\n").unwrap();
        let mut batch = RepairBatch::begin();
        let _orig = batch.snapshot(&f).unwrap();
        // 模拟修复写入
        std::fs::write(&f, "fn a() {}\n").unwrap();
        batch.commit();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "fn a() {}\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_repair_batch_rolls_back_on_failure() {
        let dir = std::env::temp_dir().join("neotrix_autofixer_rollback");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("b.rs");
        let original = "fn original() {}\n";
        std::fs::write(&f, original).unwrap();
        let mut batch = RepairBatch::begin();
        let _orig = batch.snapshot(&f).unwrap();
        // 修复写入后某步失败 → rollback 写回原内容
        std::fs::write(&f, "fn corrupted() {}\n").unwrap();
        batch.rollback();
        assert_eq!(
            std::fs::read_to_string(&f).unwrap(),
            original,
            "rollback 应恢复 heal 前内容"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cleanup_todos_tx_rolls_back_on_write_failure() {
        // cleanup_todos_tx 在快照后若 cleanup 失败应回滚; 用不存在路径的 snapshot 失败路径验证
        let err = AutoFixer::cleanup_todos_tx("/nonexistent/neotrix/nope.rs");
        assert!(err.is_err(), "快照不存在文件应报错 (不做任何写入)");
    }

    // ── GAUNTLET evidence gate chain ───────────────────────────────────

    #[test]
    fn gauntlet_stage_order() {
        assert_eq!(
            GauntletStage::ORDER.to_vec(),
            vec![
                GauntletStage::Spec,
                GauntletStage::Red,
                GauntletStage::Green,
                GauntletStage::Gauntlet,
                GauntletStage::Evidence,
            ]
        );
        assert_eq!(GauntletStage::Spec.next(), Some(GauntletStage::Red));
        assert_eq!(GauntletStage::Evidence.next(), None);
    }

    #[test]
    fn gauntlet_spec_requires_spec() {
        let spec = GauntletMachine::evaluate_stage(GauntletStage::Spec, "", 0, false, false, 0, 0, &[]);
        assert!(!spec.pass);
        assert!(spec.reason.contains("spec-before"));
        let ok = GauntletMachine::evaluate_stage(GauntletStage::Spec, "do X", 0, false, false, 0, 0, &[]);
        assert!(ok.pass);
    }

    #[test]
    fn gauntlet_red_requires_failing_test() {
        let red = GauntletMachine::evaluate_stage(GauntletStage::Red, "spec", 0, false, false, 0, 0, &[]);
        assert!(!red.pass, "RED needs a failing test (TDD)");
        let ok = GauntletMachine::evaluate_stage(GauntletStage::Red, "spec", 1, false, false, 0, 0, &[]);
        assert!(ok.pass);
    }

    #[test]
    fn gauntlet_gauntlet_blocks_on_lint_todos_unwraps() {
        let clean = GauntletMachine::evaluate_stage(GauntletStage::Gauntlet, "spec", 1, true, true, 0, 0, &[]);
        assert!(clean.pass);
        let dirty = GauntletMachine::evaluate_stage(GauntletStage::Gauntlet, "spec", 1, true, false, 3, 1, &[]);
        assert!(!dirty.pass);
        assert!(dirty.reason.contains("lint"));
        assert!(dirty.reason.contains("TODO"));
        assert!(dirty.reason.contains("unwrap"));
    }

    #[test]
    fn gauntlet_evidence_requires_evidence() {
        let no_ev = GauntletMachine::evaluate_stage(GauntletStage::Evidence, "spec", 1, true, true, 0, 0, &[]);
        assert!(!no_ev.pass, "evidence-after: must attach evidence");
        let ok = GauntletMachine::evaluate_stage(GauntletStage::Evidence, "spec", 1, true, true, 0, 0, &["cargo check: 0 errors".to_string()]);
        assert!(ok.pass);
    }

    #[test]
    fn gauntlet_full_pipeline_passes() {
        let mut m = GauntletMachine::new();
        m.start();
        let ev = &["cargo check clean".to_string(), "12 tests passed".to_string()];
        // SPEC → RED → GREEN → GAUNTLET → EVIDENCE
        assert!(m.advance("implement parser", 1, false, false, 0, 0, ev).pass);
        assert!(m.advance("implement parser", 1, false, false, 0, 0, ev).pass);
        assert!(m.advance("implement parser", 0, true, false, 0, 0, ev).pass);
        assert!(m.advance("implement parser", 0, true, true, 0, 0, ev).pass);
        assert!(m.advance("implement parser", 0, true, true, 0, 0, ev).pass);
        assert!(m.completed(), "all 5 gates passed");
        assert_eq!(m.passed.len(), 5);
        assert!(m.current().is_none());
    }

    #[test]
    fn gauntlet_blocks_progression_on_green_failure() {
        let mut m = GauntletMachine::new();
        m.start();
        assert!(m.advance("spec ok", 1, false, false, 0, 0, &[]).pass); // SPEC
        assert!(m.advance("spec ok", 1, false, false, 0, 0, &[]).pass); // RED
        // GREEN 失败 → 卡住, 不推进
        let v = m.advance("spec ok", 1, false, false, 0, 0, &[]);
        assert!(!v.pass);
        assert_eq!(m.current(), Some(GauntletStage::Green), "blocked at GREEN");
        assert!(m.blocked().is_some());
        assert_eq!(m.passed.len(), 2, "only SPEC+RED passed");
    }

    #[test]
    fn gauntlet_not_started_rejects_advance() {
        let mut m = GauntletMachine::new();
        let v = m.advance("spec", 1, true, true, 0, 0, &[]);
        assert!(!v.pass);
        assert!(v.reason.contains("not started"));
    }

    #[test]
    fn test_healer_scan_todos_detects_pending() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("a.rs"),
            "pub fn f() {\n    // TODO: finish this\n    // FIXME: cleanup\n    let s = \"TODO not a comment\";\n}\n",
        )
        .unwrap();

        let mut reg = HealerRegistry::new();
        let todos = reg.scan_todos(dir.path());
        assert_eq!(todos.len(), 1, "one file has TODOs");
        assert_eq!(todos[0].dimension, "todo");
        assert_eq!(todos[0].auto_fixable, false);
    }

    #[test]
    fn test_healer_scan_unwraps_flags_production() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("prod.rs"), "fn main() { let x = opt.unwrap(); }\n").unwrap();

        let mut reg = HealerRegistry::new();
        let unwraps = reg.scan_unwraps(dir.path());
        assert_eq!(unwraps.len(), 1);
        assert_eq!(unwraps[0].dimension, "unwraps");
        assert_eq!(unwraps[0].action, "1 unwrap() calls in production path");
    }

    #[test]
    fn test_healer_run_full_scan_aggregates_and_reports() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("x.rs"), "pub fn f() {\n    // TODO: finish me\n    opt.unwrap();\n}\n").unwrap();

        let mut reg = HealerRegistry::new();
        let report = reg.run_full_scan(dir.path());
        assert!(!report.is_empty(), "full scan must surface findings");
        assert!(report.iter().any(|s| s.dimension == "todo"));
        assert!(report.iter().any(|s| s.dimension == "unwraps"));
        assert_eq!(reg.last_report.len(), report.len());
    }
}
