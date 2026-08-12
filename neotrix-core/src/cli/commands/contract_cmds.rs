//! /contract — agent-loop 契约管理 (C1-C6 TaskContractWarden)

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_core_parallel::{TaskContractWarden, ContractState, AtomicDecomposer};

pub struct ContractCmd;

impl CliCommand for ContractCmd {
    fn name(&self) -> &str { "/contract" }
    fn aliases(&self) -> Vec<&str> { vec!["/todo"] }
    fn description(&self) -> &str { "Contract management: /contract list | define <desc> | done <id> | fail <id> | cancel <id> | stats" }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let mut warden = TaskContractWarden::new();
        if args.is_empty() {
            return CommandOutput::ok("用法:\n  /contract list                列出全部契约\n  /contract define <desc> [--type <t>]  定义新契约 (C1)\n  /contract done <id>           验收通过 (C4/C5)\n  /contract fail <id>           验收失败 (C4)\n  /contract cancel <id>         取消 (C6)\n  /contract stats               契约统计");
        }
        match args[0].as_str() {
            "list" | "ls" => {
                let contracts = warden.list();
                if contracts.is_empty() {
                    CommandOutput::ok("📋 没有契约")
                } else {
                    let mut s = format!("📋 契约 ({}):\n", contracts.len());
                    for c in contracts {
                        s.push_str(&format!("  {}\n", c.summary()));
                    }
                    CommandOutput::ok(&s)
                }
            }
            "define" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /contract define <desc> [--type <t>]");
                }
                let desc = args[1].clone();
                let task_type = args.iter().position(|a| a == "--type")
                    .and_then(|i| args.get(i + 1).cloned())
                    .unwrap_or_else(|| "generic".to_string());
                // 意图路由 + 原子拆解 (nt_core_parallel::AtomicDecomposer):
                // 生产接线 R-P79 — define 即触发 "类型路由 → 原子单元 → 输出契约" (C2 拆解)
                let kind = AtomicDecomposer::route_kind(&desc);
                let plan = AtomicDecomposer::new().decompose(&desc, kind);
                let subtasks: Vec<String> = plan.all_units()
                    .iter()
                    .map(|u| u.instruction.clone())
                    .collect();
                let id = warden.define(&desc, &task_type, 0);
                if let Some(contract) = warden.get(&id) {
                    // 自动 C2 accept: 拆解单元作为契约子步骤 (TaskContract::accept)
                    warden.record(&contract.accept(subtasks));
                }
                CommandOutput::ok(&format!(
                    "📌 契约已定义 [{}] {} (C1 Defined → C2 Accepted)\n  TaskKind: {:?}, 原子单元: {} (并行 {}, 串行 {})\n  子步骤:\n{}",
                    id.get(..8).unwrap_or(&id),
                    desc,
                    kind,
                    plan.parallel.len() + plan.sequential.len(),
                    plan.parallel.len(),
                    plan.sequential.len(),
                    plan.all_units().iter().enumerate()
                        .map(|(i, u)| format!("    {}. {}", i + 1, u.instruction))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            "done" | "complete" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /contract done <id>");
                }
                let id = args[1].clone();
                match warden.get(&id) {
                    Some(c) => {
                        let completed = c.complete(false);
                        warden.record(&completed);
                        CommandOutput::ok(&format!("✅ 契约已验收通过: {}", completed.summary()))
                    }
                    None => CommandOutput::err(&format!("未找到契约 '{}'", id)),
                }
            }
            "fail" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /contract fail <id>");
                }
                let id = args[1].clone();
                match warden.get(&id) {
                    Some(c) => {
                        let failed = c.fail();
                        warden.record(&failed);
                        CommandOutput::ok(&format!("🔴 契约已标记失败: {}", failed.summary()))
                    }
                    None => CommandOutput::err(&format!("未找到契约 '{}'", id)),
                }
            }
            "cancel" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /contract cancel <id>");
                }
                let id = args[1].clone();
                match warden.get(&id) {
                    Some(c) => {
                        let cancelled = c.cancel();
                        warden.record(&cancelled);
                        CommandOutput::ok(&format!("⚫ 契约已取消: {}", cancelled.summary()))
                    }
                    None => CommandOutput::err(&format!("未找到契约 '{}'", id)),
                }
            }
            "stats" => {
                let stats = warden.stats();
                CommandOutput::ok(&format!(
                    "📊 契约统计:\n  总数: {}\n  已完成: {} (验收率 {:.0}%)\n  进行中: {}\n  失败: {}",
                    stats.total, stats.done, stats.completion_rate * 100.0, stats.in_flight, stats.failed
                ))
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: list, define, done, fail, cancel, stats", args[0])),
        }
    }
}

#[allow(dead_code)]
fn _state_label(state: ContractState) -> String { state.label() }
