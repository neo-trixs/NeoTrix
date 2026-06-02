use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_conn::{ConnectorConfig, ConnectorKind, CONNECTOR_MANAGER};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct ConnectorCmd;
impl CliCommand for ConnectorCmd {
    fn name(&self) -> &str {
        "/connector"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/conn", "/webhook"]
    }

    fn description(&self) -> &str {
        "Manage connectors: list|add|remove|enable|disable|server"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("list");
        match subcmd {
            "list" => {
                let mgr = CONNECTOR_MANAGER.lock().expect("CONNECTOR_MANAGER lock");
                let connectors = mgr.list_connectors();
                if connectors.is_empty() {
                    return CommandOutput::ok(
                        "No connectors configured. Use `/connector add` to create one.",
                    );
                }
                let mut msg = format!("Connectors ({}):\n", connectors.len());
                for c in &connectors {
                    let status = if c.enabled { "enabled" } else { "disabled" };
                    let last = c
                        .last_event
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_else(|| "never".to_string());
                    msg.push_str(&format!(
                        "  {} {:?}/{} [{}] events:{} last:{}\n",
                        c.id, c.kind, c.name, status, c.event_count, last
                    ));
                }
                CommandOutput::ok(&msg)
            }
            "add" => {
                if args.len() < 3 {
                    return CommandOutput::err(
                        "Usage: /connector add <github|slack|webhook> <name> [config...]",
                    );
                }
                let kind_str = &args[1];
                let name = &args[2];
                let mut mgr = CONNECTOR_MANAGER.lock().expect("CONNECTOR_MANAGER lock");
                let id = match kind_str.as_str() {
                    "github" => {
                        let secret = args.get(3).map(|s| s.as_str()).unwrap_or("default-secret");
                        let repos: Vec<String> = args
                            .get(4)
                            .map(|s| s.split(',').map(String::from).collect())
                            .unwrap_or_default();
                        mgr.add_connector(
                            name,
                            ConnectorKind::GitHub,
                            ConnectorConfig::GitHub {
                                webhook_secret: secret.to_string(),
                                repos,
                                events: vec!["push".to_string(), "pull_request".to_string()],
                            },
                        )
                    }
                    "slack" => {
                        let token = args.get(3).map(|s| s.as_str()).unwrap_or("xoxb-default");
                        let channels: Vec<String> = args
                            .get(4)
                            .map(|s| s.split(',').map(String::from).collect())
                            .unwrap_or_default();
                        mgr.add_connector(
                            name,
                            ConnectorKind::Slack,
                            ConnectorConfig::Slack {
                                token: token.to_string(),
                                channels,
                            },
                        )
                    }
                    "webhook" => {
                        let url = args
                            .get(3)
                            .map(|s| s.as_str())
                            .unwrap_or("http://localhost:9090/hook");
                        mgr.add_connector(
                            name,
                            ConnectorKind::Webhook,
                            ConnectorConfig::Webhook {
                                url: url.to_string(),
                                method: "POST".to_string(),
                                headers: std::collections::HashMap::new(),
                            },
                        )
                    }
                    _ => {
                        return CommandOutput::err(&format!(
                            "Unknown connector kind: {}. Use github, slack, or webhook.",
                            kind_str
                        ));
                    }
                };
                if let Err(e) = mgr.save() {
                    log::warn!("Failed to save connectors: {}", e);
                }
                CommandOutput::ok(&format!("Connector created: {} (id: {})", name, id))
            }
            "remove" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    return CommandOutput::err("Usage: /connector remove <id>");
                }
                let mut mgr = CONNECTOR_MANAGER.lock().expect("CONNECTOR_MANAGER lock");
                match mgr.remove_connector(id) {
                    Ok(()) => {
                        if let Err(e) = mgr.save() {
                            log::warn!("Failed to save connectors: {}", e);
                        }
                        CommandOutput::ok(&format!("Connector removed: {}", id))
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "enable" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    return CommandOutput::err("Usage: /connector enable <id>");
                }
                let mut mgr = CONNECTOR_MANAGER.lock().expect("CONNECTOR_MANAGER lock");
                match mgr.enable_connector(id) {
                    Ok(()) => {
                        if let Err(e) = mgr.save() {
                            log::warn!("Failed to save connectors: {}", e);
                        }
                        CommandOutput::ok(&format!("Connector enabled: {}", id))
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "disable" => {
                let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if id.is_empty() {
                    return CommandOutput::err("Usage: /connector disable <id>");
                }
                let mut mgr = CONNECTOR_MANAGER.lock().expect("CONNECTOR_MANAGER lock");
                match mgr.disable_connector(id) {
                    Ok(()) => {
                        if let Err(e) = mgr.save() {
                            log::warn!("Failed to save connectors: {}", e);
                        }
                        CommandOutput::ok(&format!("Connector disabled: {}", id))
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "server" => {
                let action = args.get(1).map(|s| s.as_str()).unwrap_or("status");
                let mut mgr = CONNECTOR_MANAGER.lock().expect("CONNECTOR_MANAGER lock");
                match action {
                    "start" => match mgr.start_server() {
                        Ok(()) => CommandOutput::ok(&format!(
                            "Webhook server started on port {}",
                            mgr.server_port()
                        )),
                        Err(e) => CommandOutput::err(&e),
                    },
                    "stop" => match mgr.stop_server() {
                        Ok(()) => CommandOutput::ok("Webhook server stopped"),
                        Err(e) => CommandOutput::err(&e),
                    },
                    "status" => {
                        let status = if mgr.server_running() { "running" } else { "stopped" };
                        CommandOutput::ok(&format!(
                            "Webhook server: {} (port {})",
                            status,
                            mgr.server_port()
                        ))
                    }
                    _ => CommandOutput::err(&format!(
                        "Unknown server action: {}. Use start, stop, or status.",
                        action
                    )),
                }
            }
            _ => CommandOutput::err("Usage: /connector list|add|remove|enable|disable|server"),
        }
    }
}
