//! Goal Loop 命令 — /goal

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_mind::goal_loop::{GoalConfig, GoalTracker};
use crate::neotrix::nt_world_model::TaskType;

// ====== /goal ======

pub struct GoalCmd;
impl CliCommand for GoalCmd {
    fn name(&self) -> &str { "/goal" }
    fn aliases(&self) -> Vec<&str> { vec!["/g"] }
    fn description(&self) -> &str { "24/7 自主目标追求: /goal <desc> | /goal status | /goal list | /goal pause <id> | /goal resume <id> | /goal cancel <id> | /goal history" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            let msg = "Usage:\n  /goal <description>        Start autonomous goal pursuit\n  /goal status [<id>]        Show current/ID goal status\n  /goal list                 List all goals with status\n  /goal pause <id>           Pause a running goal\n  /goal resume <id>          Resume a paused goal\n  /goal cancel <id>          Cancel a goal\n  /goal history              Show completed goals\n  /goal --json               Output as JSON";
            let out = CommandOutput::ok(msg);
            return if want_json { out.with_json(serde_json::json!({"subcommands": ["<desc>", "status", "list", "pause", "resume", "cancel", "history"]})) } else { out };
        }

        let brain_ref = match brain {
            Some(b) => b,
            None => return CommandOutput::err("Brain 不可用 — goal loop 需要 brain 上下文"),
        };

        let cmd = args[0].as_str();
        match cmd {
            "status" => {
                let gl = brain_ref.blocking_read();
                let gl = &gl.goal_loop;
                if args.len() >= 2 {
                    let id = &args[1];
                    if let Some(ref g) = gl.active_goal {
                        if g.id == *id {
                            let out = CommandOutput::ok(&gl.status());
                            return if want_json { out.with_json(serde_json::json!({"goal_id": id, "state": format!("{:?}", g.state)})) } else { out };
                        }
                    }
                    if let Some(g) = gl.completed_goals.iter().find(|g| g.id == *id) {
                        let state_str = format!("{:?}", g.state);
                        let desc = &g.description;
                        let out = CommandOutput::ok(&format!("Goal '{}' ({}): {}", desc, id, state_str));
                        return if want_json { out.with_json(serde_json::json!({"goal_id": id, "state": state_str, "description": desc})) } else { out };
                    }
                    CommandOutput::not_found(&format!("Goal not found: {}", id))
                } else {
                    let out = CommandOutput::ok(&gl.status());
                    if want_json {
                        let state = gl.active_goal.as_ref().map(|g| format!("{:?}", g.state)).unwrap_or_default();
                        out.with_json(serde_json::json!({"status": gl.status(), "state": state}))
                    } else { out }
                }
            }
            "list" | "ls" => {
                let gl = brain_ref.blocking_read();
                let gl = &gl.goal_loop;
                let mut lines = Vec::new();
                let cap = gl.active_goal.as_ref().map(|g| {
                    lines.push(format!("▶ Active: [{}] {} (iter {}/{})", g.state.label(), g.description, g.iterations_completed, g.config.max_iterations));
                    g.id.clone()
                });
                if !gl.goal_queue.is_empty() {
                    lines.push(format!("📋 Queue ({}):", gl.goal_queue.len()));
                    for g in &gl.goal_queue {
                        lines.push(format!("   [{}] {} — prio={:?}", g.state.label(), g.description, g.priority));
                    }
                }
                if !gl.completed_goals.is_empty() {
                    lines.push(format!("✅ Completed ({}):", gl.completed_goals.len()));
                    for g in gl.completed_goals.iter().rev().take(5) {
                        lines.push(format!("   [{}] {} — {}", g.state.label(), g.description, g.id));
                    }
                }
                if lines.is_empty() {
                    lines.push("No goals. Use /goal <description> to start one.".to_string());
                }
                let msg = lines.join("\n");
                let out = CommandOutput::ok(&msg);
                if want_json {
                    let goals: Vec<serde_json::Value> = gl.completed_goals.iter().map(|g| serde_json::json!({
                        "id": g.id, "description": g.description, "state": format!("{:?}", g.state)
                    })).collect();
                    out.with_json(serde_json::json!({"active": cap, "completed": goals, "queue": gl.goal_queue.len()}))
                } else { out }
            }
            "pause" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /goal pause <id>");
                }
                let id = args[1].clone();
                let mut gl = brain_ref.blocking_write();
                let gl = &mut gl.goal_loop;
                if gl.active_goal.as_ref().map(|g| g.id == id).unwrap_or(false) {
                    gl.pause_goal();
                    let _ = gl.save();
                    let out = CommandOutput::ok(&format!("⏸ Goal paused: {}", id));
                    if want_json { out.with_json(serde_json::json!({"action": "pause", "goal_id": id})) } else { out }
                } else {
                    CommandOutput::err(&format!("No active goal with id: {}", id))
                }
            }
            "resume" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /goal resume <id>");
                }
                let id = args[1].clone();
                let mut gl = brain_ref.blocking_write();
                let gl = &mut gl.goal_loop;
                if gl.active_goal.as_ref().map(|g| g.id == id).unwrap_or(false) {
                    gl.resume_goal();
                    let _ = gl.save();
                    let out = CommandOutput::ok(&format!("▶ Goal resumed: {}", id));
                    if want_json { out.with_json(serde_json::json!({"action": "resume", "goal_id": id})) } else { out }
                } else {
                    CommandOutput::err(&format!("No active goal with id: {}", id))
                }
            }
            "cancel" | "clear" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /goal cancel <id>");
                }
                let id = args[1].clone();
                let mut gl = brain_ref.blocking_write();
                let gl = &mut gl.goal_loop;
                if gl.active_goal.as_ref().map(|g| g.id == id).unwrap_or(false) {
                    gl.clear_goal();
                    let _ = gl.save();
                    let out = CommandOutput::ok(&format!("✖ Goal cancelled: {}", id));
                    if want_json { out.with_json(serde_json::json!({"action": "cancel", "goal_id": id})) } else { out }
                } else {
                    CommandOutput::err(&format!("No active goal with id: {}", id))
                }
            }
            "history" => {
                let gl = brain_ref.blocking_read();
                let gl = &gl.goal_loop;
                let out = CommandOutput::ok(&gl.history_summary());
                if want_json {
                    let goals: Vec<serde_json::Value> = gl.completed_goals.iter().map(|g| serde_json::json!({
                        "id": g.id, "description": g.description, "state": format!("{:?}", g.state),
                        "iterations": g.iterations_completed, "score": g.score_current
                    })).collect();
                    out.with_json(serde_json::json!({"completed_goals": goals, "count": goals.len()}))
                } else { out }
            }
            _ => {
                let description = args.join(" ");
                let mut guard = brain_ref.blocking_write();
                let id = uuid::Uuid::new_v4().to_string();
                let score = guard.brain.evaluate_capability(TaskType::General);
                let mut tracker = GoalTracker::new(id.clone(), description.clone(), GoalConfig::default());
                tracker.score_before = score;
                tracker.score_current = score;
                guard.goal_loop.active_goal = Some(tracker);
                let _ = guard.goal_loop.save();
                let msg = format!("🎯 Goal started: {} (id={})", description, id);
                let out = CommandOutput::ok(&msg);
                if want_json { out.with_json(serde_json::json!({"goal": description, "id": id, "state": "pursuing"})) } else { out }
            }
        }
    }
}
