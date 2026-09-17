//! BYOA — Bring Your Own Agent
//!
//! 支持接入外部 agent CLI (Claude Code, Codex, Gemini CLI 等) 作为 agent brain。
//! 每个外部 agent 包装为 `ExternalAgent`，通过 stdio/HTTP 通信。
//!
//! 设计启发: Cumora BYOA + Munder Difflin "real terminal-agent CLIs"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::io::Write;

// ─── Provider Registry ─────────────────────────────────────────────────────

/// Provider type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    /// Remote API (e.g. Anthropic, OpenAI)
    Api,
    /// CLI-based provider (e.g. Codex CLI, Gemini CLI)
    Cli,
    /// Local model (e.g. Ollama)
    Local,
}

/// Information about a registered provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub model: String,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
    pub max_context: u32,
    pub capabilities: Vec<String>,
    pub available: bool,
}

// ─── External Agent Config ──────────────────────────────────────────────────

/// Configuration for an external agent CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAgentConfig {
    /// Unique agent ID
    pub id: String,
    /// Display name
    pub name: String,
    /// CLI command (e.g. "claude", "codex", "gemini")
    pub command: String,
    /// Arguments to pass
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Communication mode
    pub mode: AgentMode,
    /// Max concurrent sessions
    pub max_sessions: u32,
    /// Cost per 1k tokens (USD) — for spend gate
    pub cost_per_1k: f64,
    /// Capabilities this agent supports
    pub capabilities: Vec<String>,
}

/// Communication mode with external agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentMode {
    /// stdio: send messages via stdin, read from stdout
    Stdio,
    /// HTTP: send POST requests to local server
    Http { port: u16 },
    /// WebSocket: bidirectional streaming
    WebSocket { url: String },
}

impl Default for ExternalAgentConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            command: String::new(),
            args: Vec::new(),
            env: HashMap::new(),
            mode: AgentMode::Stdio,
            max_sessions: 1,
            cost_per_1k: 0.0,
            capabilities: Vec::new(),
        }
    }
}

// ─── Preset Configs ─────────────────────────────────────────────────────────

impl ExternalAgentConfig {
    /// Claude Code (Anthropic CLI)
    pub fn claude_code() -> Self {
        Self {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            command: "claude".to_string(),
            args: vec!["--output-format".to_string(), "json".to_string()],
            env: HashMap::new(),
            mode: AgentMode::Stdio,
            max_sessions: 3,
            cost_per_1k: 0.015, // Claude Sonnet pricing
            capabilities: vec![
                "code_generation".to_string(),
                "code_review".to_string(),
                "debugging".to_string(),
                "architecture".to_string(),
            ],
        }
    }

    /// OpenAI Codex CLI
    pub fn codex() -> Self {
        Self {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            command: "codex".to_string(),
            args: vec!["--quiet".to_string()],
            env: HashMap::new(),
            mode: AgentMode::Stdio,
            max_sessions: 2,
            cost_per_1k: 0.01, // GPT-4o pricing
            capabilities: vec![
                "code_generation".to_string(),
                "documentation".to_string(),
                "testing".to_string(),
            ],
        }
    }

    /// Google Gemini CLI
    pub fn gemini_cli() -> Self {
        Self {
            id: "gemini-cli".to_string(),
            name: "Gemini CLI".to_string(),
            command: "gemini".to_string(),
            args: vec!["--format".to_string(), "json".to_string()],
            env: HashMap::new(),
            mode: AgentMode::Stdio,
            max_sessions: 2,
            cost_per_1k: 0.005, // Gemini Flash pricing
            capabilities: vec![
                "code_generation".to_string(),
                "research".to_string(),
                "documentation".to_string(),
            ],
        }
    }

    /// All preset configs
    pub fn presets() -> Vec<Self> {
        vec![Self::claude_code(), Self::codex(), Self::gemini_cli()]
    }
}

// ─── External Agent Manager ─────────────────────────────────────────────────

/// Manages external agent CLI sessions
pub struct ExternalAgentManager {
    /// Registered agent configs
    configs: HashMap<String, ExternalAgentConfig>,
    /// Active sessions (agent_id → child process)
    sessions: HashMap<String, Vec<ExternalSession>>,
    /// Pooled (long-lived) sessions — keep processes alive across messages
    pool: HashMap<String, Vec<PooledSession>>,
    /// Max idle time before pool eviction (seconds)
    pool_idle_timeout: u64,
    /// Provider registry
    providers: HashMap<String, ProviderInfo>,
}

/// An active session with an external agent
struct ExternalSession {
    session_id: String,
    child: Option<Child>,
    /// Estimated cost for this session
    cost: f64,
}

/// A pooled (long-lived) session — keeps stdin/stdout open for multiple messages
struct PooledSession {
    session_id: String,
    agent_id: String,
    /// The child process (stdin/stdout kept open)
    child: Child,
    /// Total cost accumulated
    cost: f64,
    /// Timestamp of last activity
    last_active: u64,
    /// Messages sent in this session
    message_count: usize,
}

impl PooledSession {
    fn new(session_id: &str, agent_id: &str, child: Child) -> Self {
        Self {
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            child,
            cost: 0.0,
            last_active: now_secs(),
            message_count: 0,
        }
    }

    /// Send a message and read response without killing the process
    fn send_and_receive(&mut self, message: &str, cost_per_1k: f64) -> Result<String, String> {
        use std::io::Write;

        // Write to stdin
        if let Some(ref mut stdin) = self.child.stdin {
            stdin
                .write_all(message.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin
                .write_all(b"\n")
                .map_err(|e| format!("Failed to write newline: {}", e))?;
            stdin
                .flush()
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        } else {
            return Err("stdin not available".to_string());
        }

        // Read from stdout (non-blocking peek first, then blocking read)
        let mut output = String::new();
        if let Some(ref mut stdout) = self.child.stdout {
            use std::io::BufRead;
            let mut reader = std::io::BufReader::new(stdout);
            let mut line = String::new();
            // Read one line (blocking)
            match reader.read_line(&mut line) {
                Ok(0) => return Err("EOF on stdout".to_string()),
                Ok(_) => {
                    output = line.trim_end().to_string();
                }
                Err(e) => return Err(format!("Failed to read stdout: {}", e)),
            }
        }

        // Update stats
        let tokens_est = message.len() as f64 / 4.0 + output.len() as f64 / 4.0;
        self.cost += tokens_est * cost_per_1k / 1000.0;
        self.last_active = now_secs();
        self.message_count += 1;

        Ok(output)
    }

    /// Check if session is still alive
    fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false, // Process exited
            Ok(None) => true,     // Still running
            Err(_) => false,
        }
    }
}

/// Result from an external agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAgentResult {
    pub session_id: String,
    pub agent_id: String,
    pub content: String,
    pub cost: f64,
    pub success: bool,
    pub error: Option<String>,
}

impl ExternalAgentManager {
    pub fn new() -> Self {
        let mut providers = HashMap::new();

        // Preset providers
        providers.insert("claude-code".to_string(), ProviderInfo {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            provider_type: ProviderType::Api,
            model: "claude-sonnet-4-20250514".to_string(),
            cost_per_1k_input: 0.015,
            cost_per_1k_output: 0.075,
            max_context: 200_000,
            capabilities: vec!["code_generation".into(), "code_review".into(), "debugging".into(), "architecture".into()],
            available: true,
        });
        providers.insert("codex".to_string(), ProviderInfo {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            provider_type: ProviderType::Cli,
            model: "gpt-4o".to_string(),
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.03,
            max_context: 128_000,
            capabilities: vec!["code_generation".into(), "documentation".into(), "testing".into()],
            available: true,
        });
        providers.insert("gemini-cli".to_string(), ProviderInfo {
            id: "gemini-cli".to_string(),
            name: "Gemini CLI".to_string(),
            provider_type: ProviderType::Cli,
            model: "gemini-2.0-flash".to_string(),
            cost_per_1k_input: 0.005,
            cost_per_1k_output: 0.015,
            max_context: 1_000_000,
            capabilities: vec!["code_generation".into(), "research".into(), "documentation".into()],
            available: true,
        });
        providers.insert("ollama-local".to_string(), ProviderInfo {
            id: "ollama-local".to_string(),
            name: "Ollama Local".to_string(),
            provider_type: ProviderType::Local,
            model: "llama3".to_string(),
            cost_per_1k_input: 0.0,
            cost_per_1k_output: 0.0,
            max_context: 8_192,
            capabilities: vec!["code_generation".into(), "chat".into()],
            available: true,
        });
        providers.insert("gpt-4o".to_string(), ProviderInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            provider_type: ProviderType::Api,
            model: "gpt-4o".to_string(),
            cost_per_1k_input: 0.005,
            cost_per_1k_output: 0.015,
            max_context: 128_000,
            capabilities: vec!["code_generation".into(), "reasoning".into(), "multimodal".into()],
            available: true,
        });

        Self {
            configs: HashMap::new(),
            sessions: HashMap::new(),
            pool: HashMap::new(),
            pool_idle_timeout: 300, // 5 minutes
            providers,
        }
    }

    /// Set pool idle timeout (seconds)
    pub fn with_pool_idle_timeout(mut self, timeout: u64) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }

    // ─── Provider Registry ─────────────────────────────────────────────────

    /// Register a new provider
    pub fn register_provider(&mut self, info: ProviderInfo) {
        self.providers.insert(info.id.clone(), info);
    }

    /// Unregister a provider by ID
    pub fn unregister_provider(&mut self, id: &str) -> Option<ProviderInfo> {
        self.providers.remove(id)
    }

    /// List all registered providers
    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.providers.values().cloned().collect()
    }

    /// Get a single provider by ID
    pub fn get_provider(&self, id: &str) -> Option<&ProviderInfo> {
        self.providers.get(id)
    }

    /// Check if a provider is available
    pub fn check_provider_availability(&self, id: &str) -> bool {
        self.providers
            .get(id)
            .map(|p| p.available)
            .unwrap_or(false)
    }

    /// Update a provider's availability status
    pub fn update_provider_availability(&mut self, id: &str, available: bool) -> bool {
        if let Some(provider) = self.providers.get_mut(id) {
            provider.available = available;
            true
        } else {
            false
        }
    }

    /// Register an external agent config
    pub fn register(&mut self, config: ExternalAgentConfig) {
        self.configs.insert(config.id.clone(), config);
    }

    /// Register all presets
    pub fn register_presets(&mut self) {
        for config in ExternalAgentConfig::presets() {
            self.register(config);
        }
    }

    /// List registered agents
    pub fn list(&self) -> Vec<&ExternalAgentConfig> {
        self.configs.values().collect()
    }

    /// Get a config by ID
    pub fn get(&self, id: &str) -> Option<&ExternalAgentConfig> {
        self.configs.get(id)
    }

    /// Send a message to an external agent via stdio
    pub fn send_message(
        &mut self,
        agent_id: &str,
        session_id: &str,
        message: &str,
    ) -> Result<ExternalAgentResult, String> {
        let config = self
            .configs
            .get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?
            .clone();

        if config.mode != AgentMode::Stdio {
            return Err("Only Stdio mode is currently supported".to_string());
        }

        // Check session limit
        let active = self.sessions.get(agent_id).map(|s| s.len()).unwrap_or(0);
        if active >= config.max_sessions as usize {
            return Err(format!(
                "Agent '{}' at session limit ({}/{})",
                agent_id, active, config.max_sessions
            ));
        }

        // Spawn process
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .envs(&config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn '{}': {}", config.command, e))?;

        // Write message to stdin
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(message.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin
                .write_all(b"\n")
                .map_err(|e| format!("Failed to write newline: {}", e))?;
            // Close stdin to signal EOF
            drop(child.stdin.take());
        }

        // Read output
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to read output: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        // Estimate cost (rough: message chars / 4 * cost_per_1k / 1000)
        let tokens_est = message.len() as f64 / 4.0;
        let cost = tokens_est * config.cost_per_1k / 1000.0;

        Ok(ExternalAgentResult {
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            content: if success { stdout } else { stderr.clone() },
            cost,
            success,
            error: if success { None } else { Some(stderr) },
        })
    }

    /// Check if an external agent is available (command exists)
    pub fn check_availability(&self, agent_id: &str) -> bool {
        if let Some(config) = self.configs.get(agent_id) {
            Command::new("which")
                .arg(&config.command)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Get availability status for all registered agents
    pub fn availability_status(&self) -> HashMap<String, bool> {
        self.configs
            .keys()
            .map(|id| (id.clone(), self.check_availability(id)))
            .collect()
    }

    /// Active session count for an agent
    pub fn active_sessions(&self, agent_id: &str) -> usize {
        self.sessions.get(agent_id).map(|s| s.len()).unwrap_or(0)
    }

    /// Total estimated cost across all sessions
    pub fn total_cost(&self) -> f64 {
        let session_cost: f64 = self.sessions
            .values()
            .flat_map(|sessions| sessions.iter().map(|s| s.cost))
            .sum();
        let pool_cost: f64 = self.pool
            .values()
            .flat_map(|sessions| sessions.iter().map(|s| s.cost))
            .sum();
        session_cost + pool_cost
    }

    // ─── Pool Management (long-lived processes) ───────────────────────────

    /// Acquire or create a pooled session for an agent.
    /// Reuses existing idle session if available, otherwise spawns a new one.
    pub fn acquire_pooled(
        &mut self,
        agent_id: &str,
    ) -> Result<String, String> {
        let config = self
            .configs
            .get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?
            .clone();

        if config.mode != AgentMode::Stdio {
            return Err("Only Stdio mode is currently supported for pooling".to_string());
        }

        // Check pool limit
        let pool_count = self.pool.get(agent_id).map(|s| s.len()).unwrap_or(0);
        if pool_count >= config.max_sessions as usize {
            // Try to reuse an existing session
            if let Some(sessions) = self.pool.get_mut(agent_id) {
                for session in sessions.iter_mut() {
                    if session.is_alive() {
                        return Ok(session.session_id.clone());
                    }
                }
            }
            return Err(format!(
                "Agent '{}' at pool limit ({}/{})",
                agent_id, pool_count, config.max_sessions
            ));
        }

        // Spawn new pooled process
        let session_id = format!("pool-{}-{}", agent_id, now_secs());
        let child = Command::new(&config.command)
            .args(&config.args)
            .envs(&config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn '{}': {}", config.command, e))?;

        let session = PooledSession::new(&session_id, agent_id, child);
        self.pool
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(session);

        Ok(session_id)
    }

    /// Send a message via pooled session (keeps process alive)
    pub fn send_pooled(
        &mut self,
        agent_id: &str,
        session_id: &str,
        message: &str,
    ) -> Result<ExternalAgentResult, String> {
        let config = self
            .configs
            .get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?
            .clone();

        let sessions = self.pool.get_mut(agent_id).ok_or_else(|| {
            format!("No pooled sessions for agent '{}'", agent_id)
        })?;

        let session = sessions
            .iter_mut()
            .find(|s| s.session_id == session_id)
            .ok_or_else(|| format!("Session '{}' not found in pool", session_id))?;

        let response = session.send_and_receive(message, config.cost_per_1k)?;

        Ok(ExternalAgentResult {
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            content: response,
            cost: session.cost,
            success: true,
            error: None,
        })
    }

    /// Release a pooled session (kill the process)
    pub fn release_pooled(&mut self, agent_id: &str, session_id: &str) -> Result<(), String> {
        if let Some(sessions) = self.pool.get_mut(agent_id) {
            if let Some(pos) = sessions.iter().position(|s| s.session_id == session_id) {
                let mut session = sessions.remove(pos);
                let _ = session.child.kill();
                return Ok(());
            }
        }
        Err(format!("Session '{}' not found", session_id))
    }

    /// Evict idle pooled sessions (call periodically)
    pub fn evict_idle_pooled(&mut self) -> usize {
        let now = now_secs();
        let timeout = self.pool_idle_timeout;
        let mut evicted = 0;

        for sessions in self.pool.values_mut() {
            sessions.retain(|s| {
                if now - s.last_active > timeout {
                    evicted += 1;
                    false
                } else {
                    true
                }
            });
        }

        evicted
    }

    /// Pool stats
    pub fn pool_stats(&self) -> HashMap<String, usize> {
        self.pool
            .iter()
            .map(|(id, sessions)| (id.clone(), sessions.len()))
            .collect()
    }

    /// Manager stats
    pub fn stats(&self) -> ExternalAgentStats {
        let registered = self.configs.len();
        let available = self.availability_status().values().filter(|&&v| v).count();
        let total_sessions: usize = self.sessions.values().map(|s| s.len()).sum();
        let total_cost = self.total_cost();
        ExternalAgentStats {
            registered,
            available,
            total_sessions,
            total_cost,
        }
    }
}

impl Default for ExternalAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

use crate::l0_substrate::nt_core_time::now_secs;

// ─── Stats ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAgentStats {
    pub registered: usize,
    pub available: usize,
    pub total_sessions: usize,
    pub total_cost: f64,
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_configs() {
        let presets = ExternalAgentConfig::presets();
        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].id, "claude-code");
        assert_eq!(presets[1].id, "codex");
        assert_eq!(presets[2].id, "gemini-cli");
    }

    #[test]
    fn test_manager_register() {
        let mut mgr = ExternalAgentManager::new();
        mgr.register(ExternalAgentConfig::claude_code());
        assert_eq!(mgr.list().len(), 1);
        assert!(mgr.get("claude-code").is_some());
    }

    #[test]
    fn test_manager_register_presets() {
        let mut mgr = ExternalAgentManager::new();
        mgr.register_presets();
        assert_eq!(mgr.list().len(), 3);
    }

    #[test]
    fn test_manager_not_found() {
        let mut mgr = ExternalAgentManager::new();
        let result = mgr.send_message("nonexistent", "s1", "hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_stats() {
        let mut mgr = ExternalAgentManager::new();
        mgr.register_presets();
        let stats = mgr.stats();
        assert_eq!(stats.registered, 3);
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_cost, 0.0);
    }

    #[test]
    fn test_manager_availability() {
        let mut mgr = ExternalAgentManager::new();
        mgr.register(ExternalAgentConfig::claude_code());
        let status = mgr.availability_status();
        // This test checks the function works, not that claude is actually installed
        assert!(status.contains_key("claude-code"));
    }

    // ─── Pool Tests ───────────────────────────────────────────────────────

    #[test]
    fn test_pool_idle_timeout_default() {
        let mgr = ExternalAgentManager::new();
        assert_eq!(mgr.pool_idle_timeout, 300);
    }

    #[test]
    fn test_pool_idle_timeout_configurable() {
        let mgr = ExternalAgentManager::new().with_pool_idle_timeout(600);
        assert_eq!(mgr.pool_idle_timeout, 600);
    }

    #[test]
    fn test_pool_stats_empty() {
        let mgr = ExternalAgentManager::new();
        let stats = mgr.pool_stats();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_acquire_pooled_not_found() {
        let mut mgr = ExternalAgentManager::new();
        let result = mgr.acquire_pooled("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_evict_idle_pooled_empty() {
        let mut mgr = ExternalAgentManager::new();
        let evicted = mgr.evict_idle_pooled();
        assert_eq!(evicted, 0);
    }

    #[test]
    fn test_total_cost_includes_pool() {
        let mut mgr = ExternalAgentManager::new();
        // No sessions, cost should be 0
        assert_eq!(mgr.total_cost(), 0.0);
    }

    // ─── Provider Registry Tests ──────────────────────────────────────────

    #[test]
    fn test_provider_register() {
        let mut mgr = ExternalAgentManager::new();
        let info = ProviderInfo {
            id: "custom-llm".to_string(),
            name: "Custom LLM".to_string(),
            provider_type: ProviderType::Local,
            model: "my-model".to_string(),
            cost_per_1k_input: 0.0,
            cost_per_1k_output: 0.0,
            max_context: 4096,
            capabilities: vec!["chat".to_string()],
            available: true,
        };
        mgr.register_provider(info);
        assert!(mgr.get_provider("custom-llm").is_some());
        assert_eq!(mgr.get_provider("custom-llm").unwrap().name, "Custom LLM");
    }

    #[test]
    fn test_provider_list() {
        let mgr = ExternalAgentManager::new();
        let providers = mgr.list_providers();
        assert_eq!(providers.len(), 5);
        let ids: Vec<&str> = providers.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"claude-code"));
        assert!(ids.contains(&"codex"));
        assert!(ids.contains(&"gemini-cli"));
        assert!(ids.contains(&"ollama-local"));
        assert!(ids.contains(&"gpt-4o"));
    }

    #[test]
    fn test_provider_availability() {
        let mut mgr = ExternalAgentManager::new();
        assert!(mgr.check_provider_availability("claude-code"));
        assert!(!mgr.check_provider_availability("nonexistent"));

        mgr.update_provider_availability("claude-code", false);
        assert!(!mgr.check_provider_availability("claude-code"));

        mgr.update_provider_availability("claude-code", true);
        assert!(mgr.check_provider_availability("claude-code"));
    }

    #[test]
    fn test_provider_unregister() {
        let mut mgr = ExternalAgentManager::new();
        assert_eq!(mgr.list_providers().len(), 5);

        let removed = mgr.unregister_provider("ollama-local");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "ollama-local");
        assert_eq!(mgr.list_providers().len(), 4);
        assert!(mgr.get_provider("ollama-local").is_none());

        // Unregister nonexistent returns None
        assert!(mgr.unregister_provider("nonexistent").is_none());
    }
}
