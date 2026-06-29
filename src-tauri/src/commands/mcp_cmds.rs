use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub enabled: bool,
    pub env: HashMap<String, String>,
}

impl McpConfigStore {
    fn path() -> std::path::PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        let mut path = std::path::PathBuf::from(home);
        path.push(".neotrix");
        std::fs::create_dir_all(&path).unwrap_or_else(|e| {
            log::warn!("mcp_cmds: cannot create {:?}: {}", path, e);
        });
        path.push("mcp_servers.json");
        path
    }

    fn load() -> Vec<McpServerConfig> {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            vec![
                McpServerConfig {
                    name: "example-fs".into(),
                    command: "npx".into(),
                    args: vec!["-y".into(), "@modelcontextprotocol/server-filesystem".into(), "/tmp".into()],
                    enabled: false,
                    env: HashMap::new(),
                }
            ]
        }
    }

    fn save(servers: &[McpServerConfig]) {
        if let Ok(json) = serde_json::to_string_pretty(servers) {
            let _ = std::fs::write(Self::path(), json);
        }
    }
}

pub struct McpConfigStore;

#[tauri::command]
pub fn mcp_list_servers() -> Vec<McpServerConfig> {
    McpConfigStore::load()
}

#[tauri::command]
pub fn mcp_save_server(config: McpServerConfig) -> Result<(), String> {
    let mut servers = McpConfigStore::load();
    if let Some(pos) = servers.iter().position(|s| s.name == config.name) {
        servers[pos] = config;
    } else {
        servers.push(config);
    }
    McpConfigStore::save(&servers);
    Ok(())
}

#[tauri::command]
pub fn mcp_delete_server(name: String) -> Result<(), String> {
    let mut servers = McpConfigStore::load();
    servers.retain(|s| s.name != name);
    McpConfigStore::save(&servers);
    Ok(())
}

#[tauri::command]
pub fn mcp_toggle_server(name: String) -> Result<(), String> {
    let mut servers = McpConfigStore::load();
    if let Some(server) = servers.iter_mut().find(|s| s.name == name) {
        server.enabled = !server.enabled;
        McpConfigStore::save(&servers);
        Ok(())
    } else {
        Err(format!("Server '{}' not found", name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessDashboardData {
    pub cycle: u64,
    pub c_score: f64,
    pub coherence: f64,
    pub emotion: String,
    pub load_mode: String,
    pub reflexivity: f64,
    pub sleep_pressure: f64,
    pub vsa_buffer_size: usize,
    pub text_feed_count: u64,
}


