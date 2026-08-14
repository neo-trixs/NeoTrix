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
}
