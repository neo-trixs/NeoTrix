//! 主题命令 — /theme

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::cli::tui::themes::theme_list;

pub struct ThemeCmd;
impl CliCommand for ThemeCmd {
    fn name(&self) -> &str { "/theme" }
    fn aliases(&self) -> Vec<&str> { vec!["/t"] }
    fn description(&self) -> &str { "切换 TUI 主题 (/theme list, /theme <name>, /theme save)" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let themes = theme_list();
        let want_json = args.iter().any(|a| a == "--json");

        let sub = args.first().map(|s| s.as_str()).unwrap_or("");

        match sub {
            "list" | "" => {
                let msg = format!("可用主题: {}", themes.join(", "));
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({"action": "list", "themes": themes}))
                } else { out }
            }
            "save" => {
                let out = CommandOutput::ok("主题偏好已保存");
                out.with_json(serde_json::json!({"action": "save"}))
            }
            name if themes.contains(&name.to_string()) => {
                let out = CommandOutput::ok(&format!("🎨 切换到 {} 主题", name));
                out.with_json(serde_json::json!({"theme": name}))
            }
            name => {
                CommandOutput::err(&format!("未知主题: {}，可用: {}", name, themes.join(", ")))
            }
        }
    }
}
