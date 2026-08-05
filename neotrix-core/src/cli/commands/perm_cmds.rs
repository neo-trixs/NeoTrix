//! /perm — 三轴权限统一查询命令

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::cli::permission_profiles::PermissionAxes;
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct PermCmd;

impl CliCommand for PermCmd {
    fn name(&self) -> &str { "/perm" }
    fn aliases(&self) -> Vec<&str> { vec!["/permission", "/perm-axes"] }
    fn description(&self) -> &str { "三轴权限状态: /perm | /perm status | /perm check <action>" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok(&PermissionAxes::summary());
        }
        match args[0].as_str() {
            "status" | "axes" => CommandOutput::ok(&PermissionAxes::summary()),
            "check" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /perm check <action>\n可用 action: write_file, execute_command, network_request, read_file, read_secrets, git_push");
                }
                let action = args[1].clone();
                let decision = PermissionAxes::policy_decision_for(&action);
                let profile = crate::cli::permission_profiles::is_action_allowed(&action);
                CommandOutput::ok(&format!(
                    "🔐 动作 '{}' 评估:\n  轴3 策略决策: {:?}\n  激活画像裁决: {}\n  轴1 审批: {}",
                    action,
                    decision,
                    if profile { "Allow (画像放行)" } else { "Ask/Deny (画像未放行)" },
                    if PermissionAxes::approval_required_for(&crate::cli::approval::ActionType::ShellCommand { command: action.clone() }) {
                        "需要审批"
                    } else {
                        "无需审批"
                    }
                ))
            }
            "set-approval" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /perm set-approval <suggest|auto-edit|full-auto>");
                }
                let mode = match crate::cli::approval::ApprovalMode::from_str(&args[1]) {
                    Some(m) => m,
                    None => return CommandOutput::err("无效模式: suggest|auto-edit|full-auto"),
                };
                if let Ok(mut engine) = crate::cli::approval::global_approval().lock() {
                    engine.set_mode(mode);
                }
                CommandOutput::ok(&format!("✅ 轴1 审批模式已设为 {:?}", mode))
            }
            "set-chain" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /perm set-chain <plan|accept-edits|bypass>");
                }
                let mode = crate::neotrix::l1_body_impl::nt_shield::perm_chain::PermissionMode::from_str(&args[1]);
                if let Ok(shield) = crate::cli::shield_enforcer::global_shield().lock() {
                    shield.set_perm_chain_mode(mode);
                }
                CommandOutput::ok(&format!("✅ 轴2 权限链模式已设为 {}", mode.label()))
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: status, check, set-approval, set-chain", args[0])),
        }
    }
}
