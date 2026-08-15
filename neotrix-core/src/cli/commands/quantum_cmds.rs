//! Quantum Test 命令 — 量子态测试选择 (避免全量测试)
//!
//! /qtest plan <files...>    对给定变更文件集做量子坍缩, 输出最小充分测试集 + cargo 命令
//! /qtest index [--root <dir>]  重建测试纠缠索引 (扫描 #[test])
//! /qtest scale [--root <dir>]  输出索引规模 (测试文件/函数数)

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_qtest::QTestIndex;
use crate::neotrix::nt_mind::SelfIteratingBrain;

/// 默认扫描根 — neotrix-core 源树。
fn default_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

pub struct QTestCmd;

impl CliCommand for QTestCmd {
    fn name(&self) -> &str {
        "/qtest"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/quantum-test"]
    }

    fn description(&self) -> &str {
        "量子态测试选择:\n  /qtest plan <files...>   对变更文件集坍缩出最小测试集\n  /qtest index [--root <dir>]  重建测试纠缠索引\n  /qtest scale [--root <dir>]  索引规模报告"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.iter().find(|a| *a != "--json").cloned().unwrap_or_else(|| "plan".to_string());
        let want_json = args.iter().any(|a| a == "--json");

        // 提取 --root
        let root = args
            .iter()
            .position(|a| a == "--root")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .unwrap_or_else(default_root);

        let mut idx = QTestIndex::new(vec![root]);
        idx.build();

        match sub.as_str() {
            "scale" => {
                let files = idx.test_file_count();
                let fns = idx.test_fn_count();
                if want_json {
                    return CommandOutput::ok("").with_json(serde_json::json!({
                        "test_files": files,
                        "test_functions": fns,
                    }));
                }
                CommandOutput::ok(&format!("🧬 量子态测试索引: {} 测试文件 / {} 测试函数 (全量基线)", files, fns))
            }
            "index" => {
                CommandOutput::ok(&format!("✅ 测试纠缠索引重建完成: {} 测试文件 / {} 测试函数", idx.test_file_count(), idx.test_fn_count()))
            }
            "plan" => {
                // 无参数时自动从 git diff 获取变更文件
                let changed: Vec<PathBuf> = if args.iter().any(|a| a.starts_with("/") && !a.starts_with("/qtest") && !a.starts_with("--")) || args.is_empty() {
                    auto_changed_files()
                } else {
                    args.iter()
                        .filter(|a| !a.starts_with("--") && !a.starts_with('/'))
                        .map(PathBuf::from)
                        .collect()
                };

                if changed.is_empty() {
                    return CommandOutput::ok("🌀 无变更文件, 无需测试 (或用 `git diff` 提供)");
                }

                let report = idx.collapse(&changed, None);
                if report.fallback_full {
                    return CommandOutput::ok(&format!("🧿 量子坍缩回退全量: {} (变更 {} 文件)", report.reason, report.changed_count));
                }
                if want_json {
                    return CommandOutput::ok("").with_json(serde_json::json!({
                        "changed_count": report.changed_count,
                        "closure_size": report.closure_size,
                        "selected_files": report.selected_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                        "selected_tests": report.selected_tests,
                        "cargo_filters": report.cargo_filters,
                        "reason": report.reason,
                        "index_total_files": report.index_total_files,
                    }));
                }
                let mut out = format!(
                    "🧿 量子态坍缩: {}\n📁 命中 {} 测试文件 / 纠缠闭包 {} 文件 (变更 {} 个)\n",
                    report.reason,
                    report.selected_files.len(),
                    report.closure_size,
                    report.changed_count,
                );
                for f in &report.selected_files {
                    out.push_str(&format!("   ▸ {}\n", f.display()));
                }
                out.push_str(&format!(
                    "🎯 建议命令: `cargo test --lib -p neotrix -- {}`",
                    report.cargo_filters.join(" ")
                ));
                CommandOutput::ok(&out)
            }
            other => CommandOutput::err(&format!("未知子命令: {} (plan|index|scale)", other)),
        }
    }
}

/// 自动获取 git 变更文件 (与 HEAD 相比)。
fn auto_changed_files() -> Vec<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .output();
    let Ok(out) = output else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines()
        .filter(|l| l.ends_with(".rs"))
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qtest_scale_runs() {
        let cmd = QTestCmd;
        let out = cmd.execute(&["scale".into()], None);
        assert!(out.success);
        assert!(out.message.contains("测试索引"));
    }

    #[test]
    fn test_qtest_plan_no_changes() {
        let cmd = QTestCmd;
        // 显式传空 plan (无 git 变更时安全)
        let out = cmd.execute(&[], None);
        assert!(out.success);
    }

    #[test]
    fn test_qtest_unknown_subcommand() {
        let cmd = QTestCmd;
        let out = cmd.execute(&["bogus".into()], None);
        assert!(!out.success);
    }
}
