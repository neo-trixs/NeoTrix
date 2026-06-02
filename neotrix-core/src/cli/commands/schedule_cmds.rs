use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_mind_background_loop::always_on::{
    ScheduleExpr, ALWAYS_ON_ENGINE,
};

pub struct ScheduleCmd;

impl CliCommand for ScheduleCmd {
    fn name(&self) -> &str {
        "/schedule"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["sched", "cron"]
    }

    fn description(&self) -> &str {
        "Scheduled tasks: add|list|remove|pause|resume"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("list");

        match subcmd {
            "add" => {
                let every_pos = args.iter().position(|a| a == "--every");
                let daily_pos = args.iter().position(|a| a == "--daily");
                let hourly_pos = args.iter().position(|a| a == "--hourly");

                let (desc_end, schedule) = match (every_pos, daily_pos, hourly_pos) {
                    (Some(idx), _, _) => {
                        let secs = match args.get(idx + 1).and_then(|s| s.parse::<u64>().ok())
                        {
                            Some(s) => s,
                            None => {
                                return CommandOutput::err(
                                    "--every requires a number of seconds",
                                )
                            }
                        };
                        (idx, ScheduleExpr::Every { interval_secs: secs })
                    }
                    (_, Some(idx), _) => {
                        let time = match args.get(idx + 1) {
                            Some(t) => t.clone(),
                            None => {
                                return CommandOutput::err("--daily requires HH:MM");
                            }
                        };
                        let parts: Vec<&str> = time.split(':').collect();
                        if parts.len() != 2 {
                            return CommandOutput::err("--daily requires HH:MM (e.g. 09:00)");
                        }
                        let hour: u8 = match parts[0].parse() {
                            Ok(h) if h <= 23 => h,
                            _ => return CommandOutput::err("Invalid hour (0-23)"),
                        };
                        let minute: u8 = match parts[1].parse() {
                            Ok(m) if m <= 59 => m,
                            _ => return CommandOutput::err("Invalid minute (0-59)"),
                        };
                        (idx, ScheduleExpr::Daily { hour, minute })
                    }
                    (_, _, Some(idx)) => (idx, ScheduleExpr::Hourly),
                    _ => {
                        return CommandOutput::err(
                            "Usage: /schedule add <description> --every <secs> | --daily HH:MM | --hourly",
                        )
                    }
                };

                let desc: String = args[1..desc_end].join(" ");
                if desc.trim().is_empty() {
                    return CommandOutput::err("Description required before flags");
                }

                let mut engine = ALWAYS_ON_ENGINE.lock().expect("ALWAYS_ON_ENGINE lock");
                let id = engine.add_scheduled(desc.trim(), schedule);
                let _ = engine.save();
                CommandOutput::ok(&format!("Scheduled task added: {} (id={})", desc.trim(), id))
            }
            "list" => {
                let engine = ALWAYS_ON_ENGINE.lock().expect("ALWAYS_ON_ENGINE lock");
                let tasks = engine.list_scheduled();
                if tasks.is_empty() {
                    return CommandOutput::ok("No scheduled tasks");
                }
                let mut msg = format!("Scheduled tasks ({}):\n", tasks.len());
                for t in &tasks {
                    let cron = t.cron_description.as_deref().unwrap_or("?");
                    let status = if t.paused { "PAUSED" } else { "active" };
                    let next_run = t
                        .last_run
                        .and_then(|lr| {
                            t.schedule
                                .as_ref()
                                .map(|s| s.next_run(lr).to_rfc3339())
                        })
                        .unwrap_or_else(|| "now".to_string());
                    msg.push_str(&format!(
                        "  [{}] {} ({}) {} next: {}\n",
                        t.id, t.description, cron, status, next_run
                    ));
                }
                CommandOutput::ok(msg.trim())
            }
            "remove" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    return CommandOutput::err("Usage: /schedule remove <id>");
                }
                let mut engine = ALWAYS_ON_ENGINE.lock().expect("ALWAYS_ON_ENGINE lock");
                match engine.remove_task(id) {
                    Ok(()) => {
                        let _ = engine.save();
                        CommandOutput::ok(&format!("Removed scheduled task: {}", id))
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "pause" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    return CommandOutput::err("Usage: /schedule pause <id>");
                }
                let mut engine = ALWAYS_ON_ENGINE.lock().expect("ALWAYS_ON_ENGINE lock");
                match engine.pause_scheduled(id) {
                    Ok(()) => {
                        let _ = engine.save();
                        CommandOutput::ok(&format!("Paused scheduled task: {}", id))
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "resume" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    return CommandOutput::err("Usage: /schedule resume <id>");
                }
                let mut engine = ALWAYS_ON_ENGINE.lock().expect("ALWAYS_ON_ENGINE lock");
                match engine.resume_scheduled(id) {
                    Ok(()) => {
                        let _ = engine.save();
                        CommandOutput::ok(&format!("Resumed scheduled task: {}", id))
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            _ => CommandOutput::err(
                "Usage: /schedule list|add|remove|pause|resume",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_parse_every() {
        let s = ScheduleExpr::parse("every 300").unwrap();
        assert!(matches!(s, ScheduleExpr::Every { interval_secs: 300 }));
    }

    #[test]
    fn test_schedule_parse_daily() {
        let s = ScheduleExpr::parse("daily at 09:00").unwrap();
        assert!(matches!(s, ScheduleExpr::Daily { hour: 9, minute: 0 }));
    }

    #[test]
    fn test_schedule_cmd_name() {
        let cmd = ScheduleCmd;
        assert_eq!(cmd.name(), "/schedule");
    }

    #[test]
    fn test_schedule_cmd_aliases() {
        let cmd = ScheduleCmd;
        let aliases = cmd.aliases();
        assert!(aliases.contains(&"sched"));
        assert!(aliases.contains(&"cron"));
    }

    #[test]
    fn test_schedule_cmd_list_empty() {
        let cmd = ScheduleCmd;
        let result = cmd.execute(&[], None);
        assert!(result.success);
    }
}
