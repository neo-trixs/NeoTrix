use crate::engine::Vec2;
use std::collections::HashMap;
use std::path::PathBuf;

/// Hook event types (from Clawd on Desk)
#[derive(Debug, Clone)]
pub enum HookEvent {
    // Lifecycle
    SessionStart {
        agent: String,
        session_id: String,
    },
    SessionEnd {
        agent: String,
        session_id: String,
    },
    
    // Tool events
    ToolStart {
        tool: String,
        args: serde_json::Value,
    },
    ToolEnd {
        tool: String,
        result: serde_json::Value,
    },
    
    // Permission
    PermissionRequest {
        tool: String,
        args: serde_json::Value,
        request_id: String,
    },
    PermissionResponse {
        request_id: String,
        approved: bool,
        always: bool,
    },
    
    // Subagent
    SubagentStart {
        parent_session: String,
        child_session: String,
    },
    SubagentEnd {
        child_session: String,
    },
    
    // Notification
    Notification {
        title: String,
        body: String,
        agent: String,
    },
    
    // State change
    StateChange {
        session_id: String,
        old_state: String,
        new_state: String,
    },
}

/// Hook registration (per agent)
#[derive(Debug, Clone)]
pub struct HookConfig {
    pub agent: String,
    pub hooks_dir: PathBuf,
    pub http_port: Option<u16>,
    pub enabled: bool,
    pub auto_sync: bool,
}

impl HookConfig {
    pub fn new(agent: &str, hooks_dir: PathBuf) -> Self {
        Self {
            agent: agent.to_string(),
            hooks_dir,
            http_port: None,
            enabled: true,
            auto_sync: true,
        }
    }
}

/// Hook manager
pub struct HookManager {
    configs: HashMap<String, HookConfig>,
    event_handlers: Vec<Box<dyn Fn(&HookEvent) -> Vec<HookEvent> + Send + Sync>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            event_handlers: Vec::new(),
        }
    }

    /// Register an agent hook
    pub fn register(&mut self, config: HookConfig) {
        self.configs.insert(config.agent.clone(), config);
    }

    /// Unregister an agent hook
    pub fn unregister(&mut self, agent: &str) {
        self.configs.remove(agent);
    }

    /// Get hook config for agent
    pub fn get_config(&self, agent: &str) -> Option<&HookConfig> {
        self.configs.get(agent)
    }

    /// Process incoming hook event
    pub fn process_event(&self, event: &HookEvent) -> Vec<HookEvent> {
        let mut output_events = Vec::new();
        
        // Run through event handlers
        for handler in &self.event_handlers {
            let mut events = handler(event);
            output_events.append(&mut events);
        }
        
        // Default transformations
        match event {
            HookEvent::SessionStart { agent: _, session_id } => {
                // Auto-register session
                output_events.push(HookEvent::StateChange {
                    session_id: session_id.clone(),
                    old_state: "unknown".to_string(),
                    new_state: "thinking".to_string(),
                });
            }
            HookEvent::ToolStart { tool, .. } => {
                // Map tool to state
                let state = match tool.as_str() {
                    "edit" | "write" | "bash" => "typing",
                    "read" | "grep" | "glob" => "thinking",
                    _ => "idle",
                };
                output_events.push(HookEvent::StateChange {
                    session_id: String::new(), // Will be filled by caller
                    old_state: "idle".to_string(),
                    new_state: state.to_string(),
                });
            }
            HookEvent::ToolEnd { tool, result } => {
                // Check for errors
                if let Some(error) = result.get("error") {
                    output_events.push(HookEvent::Notification {
                        title: format!("Tool Error: {}", tool),
                        body: error.as_str().unwrap_or("Unknown error").to_string(),
                        agent: String::new(), // Will be filled by caller
                    });
                }
            }
            _ => {}
        }
        
        output_events
    }

    /// Add event handler
    pub fn add_handler(&mut self, handler: Box<dyn Fn(&HookEvent) -> Vec<HookEvent> + Send + Sync>) {
        self.event_handlers.push(handler);
    }

    /// List registered agents
    pub fn registered_agents(&self) -> Vec<&str> {
        self.configs.keys().map(|s| s.as_str()).collect()
    }

    /// Check if agent is registered
    pub fn is_registered(&self, agent: &str) -> bool {
        self.configs.contains_key(agent)
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook event to pet state mapping
pub fn hook_event_to_pet_state(event: &HookEvent) -> Option<String> {
    match event {
        HookEvent::SessionStart { .. } => Some("thinking".to_string()),
        HookEvent::SessionEnd { .. } => Some("idle".to_string()),
        HookEvent::ToolStart { tool, .. } => {
            match tool.as_str() {
                "edit" | "write" | "bash" => Some("typing".to_string()),
                "read" | "grep" | "glob" => Some("thinking".to_string()),
                _ => None,
            }
        }
        HookEvent::ToolEnd { .. } => Some("idle".to_string()),
        HookEvent::PermissionRequest { .. } => Some("notification".to_string()),
        HookEvent::PermissionResponse { .. } => Some("idle".to_string()),
        HookEvent::SubagentStart { .. } => Some("groove".to_string()),
        HookEvent::SubagentEnd { .. } => Some("idle".to_string()),
        HookEvent::Notification { .. } => Some("notification".to_string()),
        HookEvent::StateChange { new_state, .. } => Some(new_state.clone()),
    }
}

/// Permission handling mode
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionMode {
    AskEveryTime,
    QuestionPromptsOnly,
    AutoApprove,
}

impl Default for PermissionMode {
    fn default() -> Self {
        Self::AskEveryTime
    }
}

/// Permission request
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub id: String,
    pub session_id: String,
    pub agent: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub timestamp: std::time::Instant,
}

impl PermissionRequest {
    pub fn new(agent: &str, session_id: &str, tool: &str, args: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            agent: agent.to_string(),
            tool: tool.to_string(),
            args,
            timestamp: std::time::Instant::now(),
        }
    }
}

/// Permission bubble layout
#[derive(Debug, Clone)]
pub struct PermissionBubbleLayout {
    pub requests: Vec<PermissionRequest>,
    pub position: Vec2,
    pub stacking_offset: f32,
    pub max_visible: usize,
}

impl PermissionBubbleLayout {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            position: Vec2::new(100.0, 100.0), // Bottom-right
            stacking_offset: 80.0,
            max_visible: 3,
        }
    }

    pub fn add_request(&mut self, request: PermissionRequest) {
        if self.requests.len() < self.max_visible {
            self.requests.push(request);
        }
    }

    pub fn remove_request(&mut self, request_id: &str) {
        self.requests.retain(|r| r.id != request_id);
    }

    pub fn get_position_for_index(&self, index: usize) -> Vec2 {
        Vec2::new(
            self.position.x,
            self.position.y - (index as f32 * self.stacking_offset),
        )
    }
}

impl Default for PermissionBubbleLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Global hotkeys for permission
pub struct PermissionHotkeys {
    pub allow: String,  // Ctrl+Shift+Y
    pub deny: String,   // Ctrl+Shift+N
}

impl Default for PermissionHotkeys {
    fn default() -> Self {
        Self {
            allow: "Ctrl+Shift+Y".to_string(),
            deny: "Ctrl+Shift+N".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_to_pet_state() {
        let event = HookEvent::SessionStart {
            agent: "claude".to_string(),
            session_id: "test".to_string(),
        };
        assert_eq!(hook_event_to_pet_state(&event), Some("thinking".to_string()));
    }

    #[test]
    fn test_permission_bubble_layout() {
        let mut layout = PermissionBubbleLayout::new();
        let request = PermissionRequest::new("claude", "session1", "bash", serde_json::json!({"command": "ls"}));
        layout.add_request(request.clone());
        
        assert_eq!(layout.requests.len(), 1);
        assert_eq!(layout.get_position_for_index(0), Vec2::new(100.0, 100.0));
        assert_eq!(layout.get_position_for_index(1), Vec2::new(100.0, 20.0));
    }

    #[test]
    fn test_hook_manager() {
        let mut manager = HookManager::new();
        let config = HookConfig::new("claude", PathBuf::from("~/.claude/hooks"));
        manager.register(config);
        
        assert!(manager.is_registered("claude"));
        assert!(!manager.is_registered("codex"));
        assert_eq!(manager.registered_agents().len(), 1);
    }
}
