use super::types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};

pub struct LspManager {
    pub servers: HashMap<String, LspSession>,
    configs: Vec<LspServerConfig>,
}

pub struct LspSession {
    pub language_id: String,
    process: Option<Child>,
    seq_id: u64,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            configs: Self::default_configs(),
        }
    }

    fn default_configs() -> Vec<LspServerConfig> {
        vec![
            LspServerConfig {
                name: "rust-analyzer".into(),
                language_id: "rust".into(),
                command: "rust-analyzer".into(),
                args: Vec::new(),
                root_patterns: vec!["Cargo.toml".into()],
            },
            LspServerConfig {
                name: "typescript-language-server".into(),
                language_id: "typescript".into(),
                command: "typescript-language-server".into(),
                args: vec!["--stdio".into()],
                root_patterns: vec!["tsconfig.json".into(), "package.json".into()],
            },
            LspServerConfig {
                name: "pyright".into(),
                language_id: "python".into(),
                command: "pyright-langserver".into(),
                args: vec!["--stdio".into()],
                root_patterns: vec!["pyproject.toml".into(), "setup.py".into()],
            },
        ]
    }

    pub fn detect_and_start(&mut self, file_path: &str) -> Option<String> {
        let configs = self.configs.clone();
        for config in &configs {
            if self.servers.contains_key(&config.name) {
                return Some(config.name.clone());
            }
            for pattern in &config.root_patterns {
                if file_path.contains(pattern.trim_end_matches(".toml").trim_end_matches(".json"))
                    || file_path.ends_with(pattern.trim_start_matches("Cargo."))
                {
                    self.start_server(config)?;
                    return Some(config.name.clone());
                }
            }
        }
        None
    }

    fn start_server(&mut self, config: &LspServerConfig) -> Option<&str> {
        let process = Command::new(&config.command)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let session = LspSession {
            language_id: config.language_id.clone(),
            process: Some(process),
            seq_id: 0,
        };
        self.servers.insert(config.name.clone(), session);
        Some("started")
    }

    pub fn next_seq(&mut self, server_name: &str) -> Option<u64> {
        self.servers.get_mut(server_name).map(|s| {
            s.seq_id += 1;
            s.seq_id
        })
    }

    pub fn has_server(&self, name: &str) -> bool {
        self.servers.contains_key(name)
    }

    pub fn send_request(&mut self, server: &str, method: &str, params: Value) -> Option<Value> {
        let session = self.servers.get_mut(server)?;
        let id = session.seq_id + 1;
        session.seq_id = id;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        if let Some(ref mut process) = session.process {
            if let Some(ref mut stdin) = process.stdin {
                let msg = format!(
                    "Content-Length: {}\r\n\r\n{}",
                    request.to_string().len(),
                    request
                );
                let _ = writeln!(stdin, "{}", msg);
            }
            if let Some(ref mut stdout) = process.stdout {
                let mut reader = BufReader::new(stdout);
                let mut header = String::new();
                let mut content_length = 0;
                loop {
                    header.clear();
                    if reader.read_line(&mut header).ok()? == 0 {
                        return None;
                    }
                    let h = header.trim();
                    if h.is_empty() {
                        break;
                    }
                    if let Some(len) = h.strip_prefix("Content-Length: ") {
                        content_length = len.trim().parse().ok()?;
                    }
                }
                let mut buf = vec![0u8; content_length];
                reader.read_exact(&mut buf).ok()?;
                let resp: Value = serde_json::from_slice(&buf).ok()?;
                return Some(resp);
            }
        }
        None
    }

    pub fn shutdown_all(&mut self) {
        for (_, session) in self.servers.iter_mut() {
            if let Some(ref mut p) = session.process {
                let _ = p.kill();
                let _ = p.wait();
            }
        }
        self.servers.clear();
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_manager_new_empty() {
        let mgr = LspManager::new();
        assert!(mgr.servers.is_empty(), "new manager should have no servers");
    }

    #[test]
    fn test_lsp_manager_has_three_configs() {
        let mgr = LspManager::new();
        assert_eq!(mgr.configs.len(), 3);
    }

    #[test]
    fn test_default_configs_contain_known_servers() {
        let mgr = LspManager::new();
        let names: Vec<&str> = mgr.configs.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"rust-analyzer"), "missing rust-analyzer");
        assert!(
            names.contains(&"typescript-language-server"),
            "missing typescript-language-server"
        );
        assert!(names.contains(&"pyright"), "missing pyright");
    }

    #[test]
    fn test_has_server_unknown() {
        let mgr = LspManager::new();
        assert!(!mgr.has_server("nonexistent-server"));
    }

    #[test]
    fn test_next_seq_nonexistent() {
        let mut mgr = LspManager::new();
        assert!(mgr.next_seq("nope").is_none());
    }

    #[test]
    fn test_shutdown_all_empty() {
        let mut mgr = LspManager::new();
        mgr.shutdown_all();
        assert!(mgr.servers.is_empty());
    }

    #[test]
    fn test_config_rust_has_cargo_toml() {
        let mgr = LspManager::new();
        let rc = mgr
            .configs
            .iter()
            .find(|c| c.name == "rust-analyzer")
            .unwrap();
        assert!(rc.root_patterns.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_detect_and_start_unknown_file() {
        let mut mgr = LspManager::new();
        assert!(mgr.detect_and_start("/tmp/random.xyz").is_none());
    }

    #[test]
    fn test_send_request_on_missing_server() {
        let mut mgr = LspManager::new();
        assert!(mgr
            .send_request("missing", "textDocument/hover", serde_json::json!({}))
            .is_none());
    }
}
