use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::cli::cost_tracker::{BudgetAction, BudgetPeriod, COST_TRACKER};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct BudgetCmd;
impl CliCommand for BudgetCmd {
    fn name(&self) -> &str {
        "/budget"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/bgt", "/bk"]
    }

    fn description(&self) -> &str {
        "Budget management: set session|daily|monthly <amount> | status | reset | action warn|pause|stop | enable | disable"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let plain_args: Vec<&str> = args.iter().map(|s| s.as_str()).filter(|a| *a != "--json").collect();

        if plain_args.is_empty() {
            let tracker = COST_TRACKER.lock().expect("COST_TRACKER lock");
            let msg = tracker.budget_status();
            return CommandOutput::ok(&msg);
        }

        let sub = plain_args[0];
        match sub {
            "status" => {
                let tracker = COST_TRACKER.lock().expect("COST_TRACKER lock");
                let msg = tracker.budget_status();
                let out = CommandOutput::ok(&msg);
                if want_json {
                    let cfg = tracker.budget.lock().expect("budget lock");
                    out.with_json(serde_json::json!({
                        "enabled": cfg.enabled,
                        "max_session_cost": cfg.max_session_cost,
                        "max_daily_cost": cfg.max_daily_cost,
                        "max_monthly_cost": cfg.max_monthly_cost,
                        "action": format!("{:?}", cfg.action),
                    }))
                } else {
                    out
                }
            }

            "set" => {
                if plain_args.len() < 3 {
                    return CommandOutput::err("用法: /budget set <session|daily|monthly> <amount>");
                }
                let tier = plain_args[1];
                let amount: f64 = match plain_args[2].parse() {
                    Ok(v) if v > 0.0 => v,
                    _ => return CommandOutput::err("金额必须是正数"),
                };
                let tracker = COST_TRACKER.lock().expect("COST_TRACKER lock");
                let mut cfg = tracker.budget.lock().expect("budget lock");
                match tier {
                    "session" | "s" => cfg.max_session_cost = amount,
                    "daily" | "d" => cfg.max_daily_cost = amount,
                    "monthly" | "m" => cfg.max_monthly_cost = amount,
                    other => return CommandOutput::err(&format!("未知层级: {}. 可用: session, daily, monthly", other)),
                }
                drop(cfg);
                tracker.save_budget_config();
                let msg = format!("✅ Budget {} limit set to ${:.2}", tier, amount);
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!({"tier": tier, "amount": amount}))
                } else {
                    CommandOutput::ok(&msg)
                }
            }

            "action" | "on-exceed" | "onexceed" => {
                if plain_args.len() < 2 {
                    return CommandOutput::err("用法: /budget action <warn|pause|stop>");
                }
                let action = match plain_args[1] {
                    "warn" => BudgetAction::Warn,
                    "pause" => BudgetAction::Pause,
                    "stop" => BudgetAction::Stop,
                    other => return CommandOutput::err(&format!("未知动作: {}. 可用: warn, pause, stop", other)),
                };
                let tracker = COST_TRACKER.lock().expect("COST_TRACKER lock");
                let mut cfg = tracker.budget.lock().expect("budget lock");
                cfg.action = action.clone();
                drop(cfg);
                tracker.save_budget_config();
                let action_label = match action {
                    BudgetAction::Warn => "Warn",
                    BudgetAction::Pause => "Pause",
                    BudgetAction::Stop => "Stop",
                };
                let msg = format!("✅ Budget action set to: {}", action_label);
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!({"action": action_label}))
                } else {
                    CommandOutput::ok(&msg)
                }
            }

            "enable" | "on" => {
                let tracker = COST_TRACKER.lock().expect("COST_TRACKER lock");
                let mut cfg = tracker.budget.lock().expect("budget lock");
                cfg.enabled = true;
                drop(cfg);
                tracker.save_budget_config();
                let msg = "✅ Budget limiting enabled".to_string();
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!({"enabled": true}))
                } else {
                    CommandOutput::ok(&msg)
                }
            }

            "disable" | "off" => {
                let tracker = COST_TRACKER.lock().expect("COST_TRACKER lock");
                let mut cfg = tracker.budget.lock().expect("budget lock");
                cfg.enabled = false;
                drop(cfg);
                tracker.save_budget_config();
                let msg = "✅ Budget limiting disabled".to_string();
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!({"enabled": false}))
                } else {
                    CommandOutput::ok(&msg)
                }
            }

            "reset" => {
                let tracker = COST_TRACKER.lock().expect("COST_TRACKER lock");
                tracker.reset_budget_period(BudgetPeriod::Monthly);
                let msg = "✅ Budget period reset (cost data cleared)".to_string();
                if want_json {
                    CommandOutput::ok(&msg).with_json(serde_json::json!({"reset": true}))
                } else {
                    CommandOutput::ok(&msg)
                }
            }

            other => CommandOutput::err(&format!(
                "未知子命令: {}. 可用: status, set, action, enable, disable, reset",
                other
            )),
        }
    }
}
