use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::l8_autonomic_impl::nt_mind_cleanup::{
    CleanupEngine, CleanupKind, Archiver, BackupEngine, CleanupLog,
    ComponentRemover, ComponentTarget, RiskLevel,
};

pub struct CleanupCmd;

impl CliCommand for CleanupCmd {
    fn name(&self) -> &str { "/cleanup" }
    fn aliases(&self) -> Vec<&str> { vec!["/gc", "/purge", "/archive"] }
    fn description(&self) -> &str {
        "清理/归档/备份: /cleanup [status|now|quick|deep|archive|backup|search <q>|log]"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let mode = args.first().map(|s| s.as_str()).unwrap_or("status");

        match mode {
            "scan-system" | "scan" => {
                // 跨平台系统缓存扫描 (含风险分级摘要)
                let engine = CleanupEngine::new();
                let mut lines = vec!["🔎 系统缓存扫描:".to_string()];
                let mut total_bytes = 0u64;
                for kind in &[CleanupKind::ProjectArtifacts, CleanupKind::Cache, CleanupKind::Logs,
                             CleanupKind::TempFiles, CleanupKind::IDECaches] {
                    let r = engine.scan(*kind, true);
                    total_bytes += r.estimated_bytes;
                    lines.push(format!("  [{}] {} 项 (约 {:.1} MB)",
                        kind.description(), r.deletable_count, r.estimated_bytes as f64 / 1_048_576.0));
                }
                lines.push(format!("  合计可释放: {:.1} MB", total_bytes as f64 / 1_048_576.0));
                lines.push("  提示: 使用 /cleanup deep 执行清理 (默认归档到 .cleanup/archive/ 可回滚)".into());
                CommandOutput::ok(&lines.join("\n"))
            }

            "targets" | "list-targets" => {
                // 列出可移除的预装/遥测/后门组件
                let remover = ComponentRemover::new(std::path::Path::new("."));
                let present = remover.scan();
                let mut lines = vec![format!("🧬 组件移除目标 ({} 个存在):", present.len())];
                for (target, _) in &present {
                    lines.push(format!("  [{}] {} — {}\n      风险: {} | 面: {}",
                        target.name, target.description, "",
                        target.risk.label(), target.surface.label()));
                }
                if present.is_empty() {
                    lines.push("  当前平台无符合风险阀的组件目标".to_string());
                }
                lines.push("  用法: /cleanup uninstall <目标名关键词> | restore <目标名>".into());
                CommandOutput::ok(&lines.join("\n"))
            }

            "uninstall" => {
                let query = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if query.is_empty() {
                    return CommandOutput::err("用法: /cleanup uninstall <目标关键词>");
                }
                let mut remover = ComponentRemover::new(std::path::Path::new("."));
                let targets = ComponentTarget::all_targets()
                    .into_iter()
                    .filter(|t| t.name.to_lowercase().contains(&query.to_lowercase()))
                    .collect::<Vec<_>>();
                if targets.is_empty() {
                    return CommandOutput::ok(&format!("未找到匹配 \"{}\" 的组件目标 (用 /cleanup targets 查看)", query));
                }
                let mut lines = vec![format!("🧹 移除 {} 个组件:", targets.len())];
                for target in &targets {
                    let present = remover.target_present(target);
                    if present {
                        remover.dry_run = true;
                        match remover.remove(target) {
                            Ok(_) => lines.push(format!("  ✅ [dry-run 预览] {}", target.name)),
                            Err(e) => lines.push(format!("  ⚠️ {}: {}", target.name, e)),
                        }
                    } else {
                        lines.push(format!("  ⏭️  未检测到: {}", target.name));
                    }
                }
                lines.push("  确认执行: 先运行 /cleanup confirm-uninstall <关键词> 关闭 dry-run".into());
                CommandOutput::ok(&lines.join("\n"))
            }

            "confirm-uninstall" => {
                let query = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if query.is_empty() {
                    return CommandOutput::err("用法: /cleanup confirm-uninstall <目标关键词> (高危, 执行真实移除)");
                }
                let mut remover = ComponentRemover::new(std::path::Path::new("."));
                remover.dry_run = false;
                remover.risk_gate = RiskLevel::High;
                let targets = ComponentTarget::all_targets()
                    .into_iter()
                    .filter(|t| t.name.to_lowercase().contains(&query.to_lowercase()))
                    .collect::<Vec<_>>();
                if targets.is_empty() {
                    return CommandOutput::ok(&format!("未找到匹配 \"{}\" 的组件目标", query));
                }
                let mut lines = vec![format!("🧹 执行移除 {} 个组件 (已建快照, 可 /cleanup restore 回滚):", targets.len())];
                for target in &targets {
                    match remover.remove(target) {
                        Ok(snap) => lines.push(format!("  ✅ {} (快照 @{})", target.name, snap.created_at)),
                        Err(e) => lines.push(format!("  ⚠️ {}: {}", target.name, e)),
                    }
                }
                CommandOutput::ok(&lines.join("\n"))
            }

            "restore" => {
                let target = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if target.is_empty() {
                    return CommandOutput::err("用法: /cleanup restore <目标关键词> (从快照回滚组件移除)");
                }
                let remover = ComponentRemover::new(std::path::Path::new("."));
                match remover.restore(target) {
                    Ok(n) => CommandOutput::ok(&format!("↩️  从快照恢复 {} 项", n)),
                    Err(e) => CommandOutput::err(&format!("恢复失败: {}", e)),
                }
            }

            "status" | "stat" | "info" => {
                let engine = CleanupEngine::new();
                let mut lines = vec!["📊 清理状态:".to_string()];
                for kind in &[CleanupKind::ProjectArtifacts, CleanupKind::Cache, CleanupKind::Logs,
                             CleanupKind::TempFiles, CleanupKind::BrainSnapshot, CleanupKind::IDECaches] {
                    let r = engine.scan(*kind, true);
                    lines.push(format!("  {}: {} 项可清理 (约 {:.1} MB)",
                        kind.description(), r.deletable_count, r.estimated_bytes as f64 / 1_048_576.0));
                }
                // 检查归档和备份目录
                let cleanup_dir = std::path::Path::new(".cleanup");
                let backup_dir = std::path::Path::new(".backup");
                if cleanup_dir.exists() {
                    if let Ok(e) = std::fs::read_dir(cleanup_dir.join("archive")) {
                        let count = e.count();
                        lines.push(format!("  📦 归档: {} 批次", count));
                    }
                }
                if backup_dir.exists() {
                    if let Ok(e) = std::fs::read_dir(backup_dir) {
                        let count: usize = e.flatten()
                            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                            .count();
                        lines.push(format!("  💾 备份: {} 个版本", count));
                    }
                }
                CommandOutput::ok(&lines.join("\n"))
            }

            "now" | "clean" => {
                let mut engine = CleanupEngine::new().with_project_root(PathBuf::from("."));
                engine.dry_run_default = false;
                engine.archive_on_clean = true;
                let kinds = &[CleanupKind::ProjectArtifacts, CleanupKind::Logs];
                let mut total_items = 0usize;
                let mut total_bytes = 0u64;
                for kind in kinds.iter().copied() {
                    let r = engine.clean(kind);
                    total_items += r.deletable_count;
                    total_bytes += r.estimated_bytes;
                }
                CommandOutput::ok(&format!(
                    "🧹 清理完成: 归档 {} 项, 释放 {:.1} MB → .cleanup/archive/",
                    total_items, total_bytes as f64 / 1_048_576.0
                ))
            }

            "quick" => {
                let mut engine = CleanupEngine::new().with_project_root(PathBuf::from("."));
                engine.dry_run_default = false;
                engine.archive_on_clean = true;
                let r = engine.clean(CleanupKind::ProjectArtifacts);
                CommandOutput::ok(&format!(
                    "🧹 快速清理: 归档 {} 项, 释放 {:.1} MB → .cleanup/archive/",
                    r.deletable_count, r.estimated_bytes as f64 / 1_048_576.0
                ))
            }

            "deep" | "all" => {
                let mut engine = CleanupEngine::new().with_project_root(PathBuf::from("."));
                engine.dry_run_default = false;
                engine.archive_on_clean = true;
                let kinds = &[
                    CleanupKind::ProjectArtifacts, CleanupKind::Cache,
                    CleanupKind::Logs, CleanupKind::TempFiles,
                    CleanupKind::BrainSnapshot, CleanupKind::IDECaches,
                ];
                let mut total_items = 0usize;
                let mut total_bytes = 0u64;
                for kind in kinds.iter().copied() {
                    let r = engine.clean(kind);
                    total_items += r.deletable_count;
                    total_bytes += r.estimated_bytes;
                }
                CleanupEngine::prune_brain_snapshots(20);
                CommandOutput::ok(&format!(
                    "🧹 深度清理: 归档 {} 项, 释放 {:.1} MB",
                    total_items, total_bytes as f64 / 1_048_576.0
                ))
            }

            "archive" | "arch" => {
                // 仅归档当前过期文件, 不清理
                let mut engine = CleanupEngine::new().with_project_root(PathBuf::from("."));
                engine.dry_run_default = false;
                engine.archive_on_clean = true;
                let kinds = &[CleanupKind::ProjectArtifacts, CleanupKind::Logs];
                let mut total = 0usize;
                for kind in kinds.iter().copied() {
                    let r = engine.clean(kind);
                    total += r.deletable_count;
                }
                CommandOutput::ok(&format!("📦 归档完成: {} 项 → .cleanup/archive/", total))
            }

            "backup" => {
                let mut engine = BackupEngine::new(&PathBuf::from("."));
                match engine.run_backup() {
                    Ok(m) => CommandOutput::ok(&format!(
                        "💾 备份完成: {} 文件, {:.1} KB → .backup/{}/",
                        m.file_count, m.total_bytes as f64 / 1024.0, m.backup_id
                    )),
                    Err(e) => CommandOutput::err(&format!("备份失败: {}", e)),
                }
            }

            "search" | "find" => {
                let query = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if query.is_empty() {
                    return CommandOutput::err("用法: /cleanup search <关键词>");
                }
                let archiver = Archiver::new(&PathBuf::from("."));
                let hits = archiver.search(query);
                if hits.is_empty() {
                    return CommandOutput::ok(&format!("🔍 未找到匹配 \"{}\" 的归档", query));
                }
                let mut lines = vec![format!("🔍 找到 {} 个归档条目:", hits.len())];
                for (i, entry) in hits.iter().take(20).enumerate() {
                    lines.push(format!("  {}. {} ({} → {}, {})",
                        i + 1, entry.source_path,
                        entry.cleanup_kind, entry.archived_path,
                        crate::cli::commands::cleanup_cmds::format_size(entry.size_bytes)));
                }
                CommandOutput::ok(&lines.join("\n"))
            }

            "log" | "history" => {
                let log_dir = PathBuf::from(".cleanup").join("log");
                let entries = CleanupLog::recent(&log_dir, 20);
                if entries.is_empty() {
                    return CommandOutput::ok("📋 暂无清理日志");
                }
                let mut lines = vec!["📋 最近清理记录:".to_string()];
                for entry in &entries {
                    let ok = if entry.success { "✅" } else { "❌" };
                    lines.push(format!("  {} {}: {} 项 ({}) - {}",
                        ok, entry.action, entry.items,
                        crate::cli::commands::cleanup_cmds::format_size(entry.bytes),
                        entry.batch_id));
                }
                CommandOutput::ok(&lines.join("\n"))
            }

            _ => {
                let engine = CleanupEngine::new();
                let r = engine.scan(CleanupKind::ProjectArtifacts, true);
                let snapshots = CleanupEngine::prune_brain_snapshots(usize::MAX);
                let log_dir = PathBuf::from(".cleanup").join("log");
                let recent = CleanupLog::recent(&log_dir, 3);
                let last = if recent.is_empty() {
                    "无".into()
                } else {
                    format!("{} | {} 项", recent[0].action, recent[0].items)
                };
                CommandOutput::ok(&format!(
                    "🔍 预览 (dry-run):\n  {} 项可归档 (约 {:.1} MB)\n  📸 快照: {} 个\n  📋 上次清理: {}\n\n子命令:\n  /cleanup now      归档并清理构建产物\n  /cleanup quick    仅清理构建产物\n  /cleanup deep     深度清理 (含缓存/日志)\n  /cleanup backup   执行代码备份 → .backup/\n  /cleanup archive  归档过期文件\n  /cleanup search q 搜索归档\n  /cleanup log      查看清理历史\n  /cleanup status   查看状态\n  /cleanup scan-system 跨平台系统缓存扫描\n  /cleanup targets  列出可移除组件 (预装/遥测)\n  /cleanup uninstall q 组件移除预览\n  /cleanup confirm-uninstall q 真实移除 (快照可回滚)\n  /cleanup restore q 从快照恢复",
                    r.deletable_count, r.estimated_bytes as f64 / 1_048_576.0, snapshots, last
                ))
            }
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
