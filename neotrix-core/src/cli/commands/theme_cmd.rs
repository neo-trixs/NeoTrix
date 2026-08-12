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
    fn description(&self) -> &str { "Switch TUI theme (/theme list, /theme <name>, /theme save)" }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let themes = theme_list();
        let want_json = args.iter().any(|a| a == "--json");

        let sub = args.first().map(|s| s.as_str()).unwrap_or("");

        let current_theme = || crate::config::NeoTrixConfig::load()
            .color_mode
            .unwrap_or_else(|| "dark".to_string());

        match sub {
            "list" | "" => {
                let msg = format!("Available themes: {} (current: {})", themes.join(", "), current_theme());
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({"action": "list", "themes": themes, "current": current_theme()}))
                } else { out }
            }
            "save" => {
                let current = current_theme();
                crate::config::NeoTrixConfig::default().save_field("color_mode", &current);
                let out = CommandOutput::ok(&format!("Theme preference saved: {}", current));
                out.with_json(serde_json::json!({"action": "save", "theme": current, "persisted": true}))
            }
            name if themes.contains(&name.to_string()) => {
                crate::config::NeoTrixConfig::default().save_field("color_mode", name);
                let out = CommandOutput::ok(&format!("🎨 Switched to {} theme (persisted, applies on restart)", name));
                out.with_json(serde_json::json!({"theme": name, "persisted": true}))
            }
            name => {
                CommandOutput::err(&format!("Unknown theme: {}, available: {}", name, themes.join(", ")))
            }
        }
    }
}
