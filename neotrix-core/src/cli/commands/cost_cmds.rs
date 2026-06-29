//! Cost & approval commands — Cost / Approval

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::approval::{global_approval, ApprovalMode};
use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::cli::cost_tracker::{BudgetPeriod, COST_TRACKER};
use crate::neotrix::nt_mind::SelfIteratingBrain;

// ====== /cost ======

pub struct CostCmd;
impl CliCommand for CostCmd {
    fn name(&self) -> &str {
        "/cost"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/spend"]
    }
    fn description(&self) -> &str {
        "Cost tracking: /cost | /cost detail | /cost budget <amount> [daily|weekly|monthly] | /cost reset"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let plain_args: Vec<&str> = args
            .iter()
            .map(|s| s.as_str())
            .filter(|a| *a != "--json")
            .collect();

        if plain_args.is_empty() {
            let tracker = COST_TRACKER.lock().unwrap_or_else(|e| e.into_inner());
            let s = tracker.summary();
            let msg = format!(
                "💰 费用追踪\n  Total cost:      ${:.4}\n  Tokens in:       {}\n  Tokens out:      {}\n  Sessions:        {}\n  Current session: ${:.4}\n  Top model:       {}\n{}",
                s.total_cost,
                s.total_tokens_in,
                s.total_tokens_out,
                s.session_count,
                s.current_session_cost,
                s.top_model,
                match s.budget_remaining {
                    Some(r) => format!("  Budget remaining: ${:.2}", r),
                    None => "  Budget:          not set".to_string(),
                }
            );
            let out = CommandOutput::ok(&msg);
            return if want_json {
                out.with_json(serde_json::json!({
                    "total_cost": s.total_cost,
                    "tokens_in": s.total_tokens_in,
                    "tokens_out": s.total_tokens_out,
                    "session_count": s.session_count,
                    "current_session_cost": s.current_session_cost,
                    "top_model": s.top_model,
                    "budget_remaining": s.budget_remaining,
                }))
            } else {
                out
            };
        }

        let sub = plain_args[0];
        match sub {
            "detail" | "details" | "sessions" => {
                let tracker = COST_TRACKER.lock().unwrap_or_else(|e| e.into_inner());
                let sessions = tracker.sessions();
                if sessions.is_empty() {
                    return CommandOutput::ok("💰 无已完成的会话记录");
                }
                let mut msg = format!("💰 会话明细 ({}):\n", sessions.len());
                for (i, s) in sessions.iter().enumerate() {
                    msg.push_str(&format!(
                        "  {}. {} | {} | T in:{} out:{} | ${:.4} | {}calls\n",
                        i + 1,
                        s.name,
                        s.model,
                        s.tokens_in,
                        s.tokens_out,
                        s.estimated_cost,
                        s.tool_calls
                    ));
                }
                let out = CommandOutput::ok(&msg);
                if want_json {
                    let json_sessions: Vec<serde_json::Value> = sessions
                        .iter()
                        .map(|s| {
                            serde_json::json!({
                                "id": s.id, "name": s.name, "model": s.model,
                                "tokens_in": s.tokens_in, "tokens_out": s.tokens_out,
                                "estimated_cost": s.estimated_cost, "tool_calls": s.tool_calls,
                            })
                        })
                        .collect();
                    out.with_json(
                        serde_json::json!({"sessions": json_sessions, "count": sessions.len()}),
                    )
                } else {
                    out
                }
            }
            "budget" | "limit" => {
                if plain_args.len() < 2 {
                    return CommandOutput::err(
                        "用法: /cost budget <amount> [daily|weekly|monthly]",
                    );
                }
                let amount: f64 = match plain_args[1].parse() {
                    Ok(v) if v > 0.0 => v,
                    _ => return CommandOutput::err("预算金额必须是正数"),
                };
                let period = match plain_args.get(2).copied() {
                    Some("daily" | "day") => BudgetPeriod::Daily,
                    Some("weekly" | "week") => BudgetPeriod::Weekly,
                    Some("monthly" | "month") | None => BudgetPeriod::Monthly,
                    Some(other) => {
                        return CommandOutput::err(&format!(
                            "未知周期: {} (可选: daily, weekly, monthly)",
                            other
                        ))
                    }
                };
                {
                    let mut tracker = COST_TRACKER.lock().unwrap_or_else(|e| e.into_inner());
                    tracker.set_budget(amount, period);
                }
                let period_label = match period {
                    BudgetPeriod::Daily => "daily",
                    BudgetPeriod::Weekly => "weekly",
                    BudgetPeriod::Monthly => "monthly",
                };
                let msg = format!("💰 预算已设置: ${:.2}/{}", amount, period_label);
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "budget_set": true, "amount": amount, "period": period_label
                    }))
                } else {
                    CommandOutput::ok(&msg)
                }
            }
            "reset" | "clear" => {
                {
                    let mut tracker = COST_TRACKER.lock().unwrap_or_else(|e| e.into_inner());
                    tracker.reset();
                }
                let msg = "💰 费用追踪已重置";
                if want_json {
                    CommandOutput::ok(msg).with_json(serde_json::json!({"reset": true}))
                } else {
                    CommandOutput::ok(msg)
                }
            }
            _ => CommandOutput::err(&format!("未知子命令: {}。可用: detail, budget, reset", sub)),
        }
    }
}

// ====== /approval ======

pub struct ApprovalCmd;
impl CliCommand for ApprovalCmd {
    fn name(&self) -> &str {
        "/approval"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/app"]
    }
    fn description(&self) -> &str {
        "Approval management: /approval mode <suggest|auto-edit|full-auto> | status | list | approve <id> | deny <id> | approve-all"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            let msg = "用法:\n  /approval mode <suggest|auto-edit|full-auto>  切换审批模式\n  /approval status                            当前模式和待审批数\n  /approval list                              列出待审批操作\n  /approval approve <id>                      批准指定操作\n  /approval deny <id>                         拒绝指定操作\n  /approval approve-all                       批准全部待审批\n  /approval deny-all                          拒绝全部待审批";
            let out = CommandOutput::ok(msg);
            return if want_json {
                out.with_json(serde_json::json!({"subcommands": ["mode", "status", "list", "approve", "deny", "approve-all", "deny-all"]}))
            } else {
                out
            };
        }

        let engine = global_approval();
        let cmd = args[0].as_str();
        match cmd {
            "mode" => {
                if args.len() < 2 {
                    let e = engine.lock().unwrap_or_else(|e| e.into_inner());
                    return CommandOutput::err(&format!(
                        "用法: /approval mode <suggest|auto-edit|full-auto>\n当前: {}",
                        e.summary()
                    ));
                }
                let mode = args[1].as_str();
                match ApprovalMode::from_str(mode) {
                    Some(m) => {
                        let mut e = engine.lock().unwrap_or_else(|e| e.into_inner());
                        e.set_mode(m);
                        let out = CommandOutput::ok(&format!("✅ 审批模式已切换: {:?}", m));
                        if want_json {
                            out.with_json(serde_json::json!({"mode": format!("{:?}", m)}))
                        } else {
                            out
                        }
                    }
                    None => CommandOutput::err(&format!(
                        "无效模式: {}. 可用: suggest, auto-edit, full-auto",
                        mode
                    )),
                }
            }
            "status" => {
                let e = engine.lock().unwrap_or_else(|e| e.into_inner());
                let mode_str = format!("{:?}", e.mode());
                let count = e.pending_count();
                let msg = format!("⚙️ 模式: {:?}\n📋 待审批: {} 项", e.mode(), count);
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({"mode": mode_str, "pending_count": count}))
                } else {
                    out
                }
            }
            "list" => {
                let e = engine.lock().unwrap_or_else(|e| e.into_inner());
                let pending = e.pending_actions();
                if pending.is_empty() {
                    return CommandOutput::ok("📋 没有待审批的操作");
                }
                let mut s = format!("📋 待审批操作 ({} 项):\n", pending.len());
                for pa in pending {
                    s.push_str(&format!("  {} — {}\n", pa.id, pa.description));
                }
                let out = CommandOutput::ok(&s);
                if want_json {
                    let items: Vec<serde_json::Value> = pending
                        .iter()
                        .map(|p| serde_json::json!({"id": p.id, "description": p.description}))
                        .collect();
                    out.with_json(serde_json::json!({"pending": items, "count": items.len()}))
                } else {
                    out
                }
            }
            "approve" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /approval approve <id>");
                }
                let id = args[1].as_str();
                let mut e = engine.lock().unwrap_or_else(|e| e.into_inner());
                match e.approve(id) {
                    Ok(()) => CommandOutput::ok(&format!("✅ 已批准: {}", id)),
                    Err(msg) => CommandOutput::err(&msg),
                }
            }
            "deny" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /approval deny <id>");
                }
                let id = args[1].as_str();
                let mut e = engine.lock().unwrap_or_else(|e| e.into_inner());
                match e.deny(id) {
                    Ok(()) => CommandOutput::ok(&format!("❌ 已拒绝: {}", id)),
                    Err(msg) => CommandOutput::err(&msg),
                }
            }
            "approve-all" | "approve_all" => {
                let mut e = engine.lock().unwrap_or_else(|e| e.into_inner());
                let count = e.approve_all();
                CommandOutput::ok(&format!("✅ 已全部批准 ({} 项)", count))
            }
            "deny-all" | "deny_all" => {
                let mut e = engine.lock().unwrap_or_else(|e| e.into_inner());
                let count = e.deny_all();
                CommandOutput::ok(&format!("❌ 已全部拒绝 ({} 项)", count))
            }
            _ => CommandOutput::err(&format!(
                "未知子命令: {}. 可用: mode, status, list, approve, deny, approve-all, deny-all",
                cmd
            )),
        }
    }
}
