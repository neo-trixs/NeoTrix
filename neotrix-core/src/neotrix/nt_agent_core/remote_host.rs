#![forbid(unsafe_code)]

use super::error::AgentError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RemoteHostConfig {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub identity_file: Option<String>,
    pub project_root: String,
}

impl RemoteHostConfig {
    pub fn new(host: &str, user: &str, port: u16, project_root: &str) -> Self {
        Self {
            host: host.to_string(),
            user: user.to_string(),
            port,
            identity_file: None,
            project_root: project_root.to_string(),
        }
    }

    pub fn with_identity(mut self, path: &str) -> Self {
        self.identity_file = Some(path.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteHostStatus {
    Disconnected,
    Connecting,
    Connected(u64),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct RemoteActionRecord {
    pub session_id: u64,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timestamp: u64,
}

const MAX_SESSIONS: usize = 100;
const MAX_ACTION_HISTORY: usize = 1000;

pub struct RemoteAgentHost {
    next_session_id: u64,
    sessions: HashMap<u64, RemoteHostConfig>,
    action_history: Vec<RemoteActionRecord>,
    session_statuses: HashMap<u64, RemoteHostStatus>,
    configs: Vec<RemoteHostConfig>,
}

impl RemoteAgentHost {
    fn prune_sessions(&mut self) {
        if self.sessions.len() > MAX_SESSIONS {
            let key = self.sessions.keys().next().cloned();
            if let Some(k) = key {
                self.sessions.remove(&k);
                self.session_statuses.remove(&k);
            }
        }
    }

    fn prune_action_history(&mut self) {
        if self.action_history.len() > MAX_ACTION_HISTORY {
            let overflow = self.action_history.len() - MAX_ACTION_HISTORY;
            self.action_history.drain(0..overflow);
        }
    }
}

impl RemoteAgentHost {
    pub fn new() -> Self {
        Self {
            next_session_id: 1,
            sessions: HashMap::new(),
            action_history: Vec::new(),
            session_statuses: HashMap::new(),
            configs: Vec::new(),
        }
    }

    pub fn discover_from_ssh_config(&mut self) -> Vec<RemoteHostConfig> {
        let configs = vec![
            RemoteHostConfig::new("github.com", "git", 22, "/home/git"),
            RemoteHostConfig::new("dev-server.local", "deploy", 2222, "/opt/projects"),
        ];
        self.configs = configs.clone();
        configs
    }

    pub fn connect(&mut self, config: &RemoteHostConfig) -> Result<u64, AgentError> {
        let id = self.next_session_id;
        self.next_session_id += 1;
        self.sessions.insert(id, config.clone());
        self.session_statuses
            .insert(id, RemoteHostStatus::Connected(id));
        self.prune_sessions();
        self.configs.push(config.clone());
        Ok(id)
    }

    pub fn execute(
        &mut self,
        session_id: u64,
        command: &str,
        cwd: &str,
    ) -> Result<RemoteActionRecord, AgentError> {
        if !self.sessions.contains_key(&session_id) {
            return Err(AgentError::NotFound(format!(
                "Session {} not found",
                session_id
            )));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let record = RemoteActionRecord {
            session_id,
            command: command.to_string(),
            stdout: format!("[{}]$ {}: output (simulated)", cwd, command),
            stderr: String::new(),
            exit_code: 0,
            timestamp: now,
        };
        self.action_history.push(record.clone());
        self.prune_action_history();
        Ok(record)
    }

    pub fn read_file(&mut self, session_id: u64, path: &str) -> Result<String, AgentError> {
        if !self.sessions.contains_key(&session_id) {
            return Err(AgentError::NotFound(format!(
                "Session {} not found",
                session_id
            )));
        }
        Ok(format!(
            "// simulated content of {}\nfn hello() {{\n    println!(\"world\");\n}}\n",
            path
        ))
    }

    pub fn write_file(
        &mut self,
        session_id: u64,
        path: &str,
        _content: &str,
    ) -> Result<(), AgentError> {
        if !self.sessions.contains_key(&session_id) {
            return Err(AgentError::NotFound(format!(
                "Session {} not found",
                session_id
            )));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.action_history.push(RemoteActionRecord {
            session_id,
            command: format!("write {}", path),
            stdout: format!("Wrote {} bytes", _content.len()),
            stderr: String::new(),
            exit_code: 0,
            timestamp: now,
        });
        self.prune_action_history();
        Ok(())
    }

    pub fn disconnect(&mut self, session_id: u64) -> Result<(), AgentError> {
        if !self.sessions.contains_key(&session_id) {
            return Err(AgentError::NotFound(format!(
                "Session {} not found",
                session_id
            )));
        }
        self.sessions.remove(&session_id);
        self.session_statuses
            .insert(session_id, RemoteHostStatus::Disconnected);
        Ok(())
    }

    pub fn health_check(&self, session_id: u64) -> bool {
        self.sessions.contains_key(&session_id)
    }

    pub fn session_status(&self, session_id: u64) -> RemoteHostStatus {
        self.session_statuses
            .get(&session_id)
            .cloned()
            .unwrap_or(RemoteHostStatus::Disconnected)
    }

    pub fn recent_actions(&self, n: usize) -> Vec<RemoteActionRecord> {
        let n = n.min(self.action_history.len());
        self.action_history.iter().rev().take(n).cloned().collect()
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn available_configs(&self) -> &[RemoteHostConfig] {
        &self.configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_connect_creates_session() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("test.local", "user", 22, "/home/user");
        let id = host.connect(&config).unwrap();
        assert!(id > 0);
        assert!(host.health_check(id));
    }

    #[test]
    fn test_execute_returns_record() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("test.local", "user", 22, "/home/user");
        let id = host.connect(&config).unwrap();
        let record = host.execute(id, "ls -la", "/home/user").unwrap();
        assert_eq!(record.exit_code, 0);
        assert!(record.stdout.contains("simulated"));
    }

    #[test]
    fn test_execute_invalid_session_fails() {
        let mut host = RemoteAgentHost::new();
        let result = host.execute(999, "ls", "/");
        assert!(result.is_err());
    }

    #[test]
    fn test_disconnect_removes_session() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("t.local", "u", 22, "/home");
        let id = host.connect(&config).unwrap();
        host.disconnect(id).unwrap();
        assert!(!host.health_check(id));
    }

    #[test]
    fn test_read_file_returns_content() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("t.local", "u", 22, "/home");
        let id = host.connect(&config).unwrap();
        let content = host.read_file(id, "/home/file.rs").unwrap();
        assert!(content.contains("simulated content"));
    }

    #[test]
    fn test_write_file_succeeds() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("t.local", "u", 22, "/home");
        let id = host.connect(&config).unwrap();
        host.write_file(id, "/tmp/test", "hello").unwrap();
        assert_eq!(host.recent_actions(1).len(), 1);
    }

    #[test]
    fn test_recent_actions_returns_n() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("t.local", "u", 22, "/home");
        let id = host.connect(&config).unwrap();
        for i in 0..5 {
            host.execute(id, &format!("cmd_{}", i), "/home").unwrap();
        }
        assert_eq!(host.recent_actions(3).len(), 3);
    }

    #[test]
    fn test_discover_from_ssh_config() {
        let mut host = RemoteAgentHost::new();
        let configs = host.discover_from_ssh_config();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].host, "github.com");
    }

    #[test]
    fn test_session_status_lifecycle() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("t.local", "u", 22, "/home");
        let id = host.connect(&config).unwrap();
        assert_eq!(host.session_status(id), RemoteHostStatus::Connected(id));
        host.disconnect(id).unwrap();
        assert_eq!(host.session_status(id), RemoteHostStatus::Disconnected);
    }

    #[test]
    fn test_duplicate_connect_creates_unique_ids() {
        let mut host = RemoteAgentHost::new();
        let config = RemoteHostConfig::new("t.local", "u", 22, "/home");
        let id1 = host.connect(&config).unwrap();
        let id2 = host.connect(&config).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_empty_recent_actions() {
        let host = RemoteAgentHost::new();
        assert!(host.recent_actions(5).is_empty());
    }

    #[test]
    fn test_invalid_disconnect_fails() {
        let mut host = RemoteAgentHost::new();
        let result = host.disconnect(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_identity() {
        let config = RemoteHostConfig::new("h", "u", 22, "/r").with_identity("/home/u/.ssh/id_rsa");
        assert_eq!(config.identity_file, Some("/home/u/.ssh/id_rsa".into()));
    }

    #[test]
    fn test_available_configs() {
        let mut host = RemoteAgentHost::new();
        host.discover_from_ssh_config();
        assert_eq!(host.available_configs().len(), 2);
    }

    #[test]
    fn test_multiple_sessions() {
        let mut host = RemoteAgentHost::new();
        let c1 = RemoteHostConfig::new("h1", "u", 22, "/h1");
        let c2 = RemoteHostConfig::new("h2", "u", 22, "/h2");
        let id1 = host.connect(&c1).unwrap();
        let id2 = host.connect(&c2).unwrap();
        assert_eq!(host.active_session_count(), 2);
        assert!(host.health_check(id1));
        assert!(host.health_check(id2));
    }
}
