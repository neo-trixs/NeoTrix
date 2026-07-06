use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::l1_body_impl::nt_io_session_recovery::SessionRecoveryManager;

pub struct SessionRecoveryCmd;

impl CliCommand for SessionRecoveryCmd {
    fn name(&self) -> &str {
        "/session-recovery"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/recover", "/snap"]
    }

    fn description(&self) -> &str {
        "会话恢复: /session-recovery [status|snapshot|restore|list <id>]"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let mut mgr = SessionRecoveryManager::new("cli-session");
        let mode = args.first().map(|s| s.as_str()).unwrap_or("status");

        match mode {
            "status" | "info" => {
                let info = mgr.get_recovery_info();
                let msg = format!(
                    "会话恢复状态:\n  Session ID: {}\n  快照存在: {}\n  Git备份: {}\n  快照计数: {}\n  快照目录: {}",
                    info.session_id,
                    info.has_snapshot,
                    info.has_git_backup,
                    info.snapshot_count,
                    info.snapshot_dir,
                );
                CommandOutput::ok(&msg)
            }
            "snapshot" | "save" => {
                match mgr.create_snapshot(&[], &[], "") {
                    Ok(snap) => CommandOutput::ok(&format!(
                        "快照已创建: {} (at {})",
                        snap.session_id, snap.created_at
                    )),
                    Err(e) => CommandOutput::err(&format!("创建快照失败: {}", e)),
                }
            }
            "restore" | "load" => {
                match mgr.load_latest_snapshot() {
                    Some(snap) => CommandOutput::ok(&format!(
                        "已恢复会话快照: session={}, messages={}, plans={}",
                        snap.session_id,
                        snap.message_count,
                        snap.plan_ids.len(),
                    )),
                    None => CommandOutput::err("没有可恢复的快照"),
                }
            }
            "list" | "ls" => {
                let info = mgr.get_recovery_info();
                if info.has_snapshot {
                    CommandOutput::ok(&format!(
                        "快照目录: {}\n  最新快照: {}-latest.json\n  计数: {}",
                        info.snapshot_dir, info.session_id, info.snapshot_count
                    ))
                } else {
                    CommandOutput::ok("没有快照")
                }
            }
            _ => CommandOutput::err(&format!("未知子命令: {}。可用: status, snapshot, restore, list", mode)),
        }
    }
}
