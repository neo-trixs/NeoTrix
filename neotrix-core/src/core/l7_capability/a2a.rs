use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Failed { error: String },
    Canceled,
}

impl TaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Failed { .. } | TaskState::Canceled
        )
    }

    pub fn label(&self) -> &str {
        match self {
            TaskState::Submitted => "submitted",
            TaskState::Working => "working",
            TaskState::InputRequired => "input-required",
            TaskState::Completed => "completed",
            TaskState::Failed { .. } => "failed",
            TaskState::Canceled => "canceled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub role: A2ARole,
    pub parts: Vec<A2APart>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum A2ARole {
    Agent,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum A2APart {
    Text {
        text: String,
    },
    File {
        mime_type: String,
        data: String,
        name: String,
    },
    Data {
        key: String,
        value: serde_json::Value,
    },
}

impl A2AMessage {
    pub fn text(role: A2ARole, text: impl Into<String>) -> Self {
        Self {
            role,
            parts: vec![A2APart::Text { text: text.into() }],
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ATask {
    pub id: String,
    pub session_id: String,
    pub state: TaskState,
    pub messages: Vec<A2AMessage>,
    pub history: Vec<TaskState>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl A2ATask {
    pub fn new(id: impl Into<String>, session_id: impl Into<String>) -> Self {
        let now = now_epoch_ms();
        Self {
            id: id.into(),
            session_id: session_id.into(),
            state: TaskState::Submitted,
            messages: Vec::new(),
            history: vec![TaskState::Submitted],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, new_state: TaskState) {
        self.history.push(new_state.clone());
        self.state = new_state;
        self.updated_at = now_epoch_ms();
    }

    pub fn add_message(&mut self, msg: A2AMessage) {
        self.messages.push(msg);
        self.updated_at = now_epoch_ms();
    }

    pub fn last_message(&self) -> Option<&A2AMessage> {
        self.messages.last()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub agent_name: String,
    pub agent_version: String,
    pub description: String,
    pub capabilities: Vec<AgentCapability>,
    pub skills: Vec<AgentSkill>,
    pub endpoints: HashMap<String, String>,
    pub authentication: Vec<String>,
    pub default_input_modes: Vec<String>,
    pub default_output_modes: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_modes: Vec<String>,
    pub output_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub confidence: f64,
}

impl AgentCard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            agent_name: name.into(),
            agent_version: "1.0.0".to_string(),
            description: String::new(),
            capabilities: Vec::new(),
            skills: Vec::new(),
            endpoints: HashMap::new(),
            authentication: Vec::new(),
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            metadata: HashMap::new(),
        }
    }

    pub fn capability(mut self, id: &str, name: &str, description: &str) -> Self {
        self.capabilities.push(AgentCapability {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            input_modes: vec!["text".to_string()],
            output_modes: vec!["text".to_string()],
        });
        self
    }

    pub fn endpoint(mut self, protocol: &str, url: &str) -> Self {
        self.endpoints.insert(protocol.to_string(), url.to_string());
        self
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct A2ARegistry {
    agents: HashMap<String, AgentCard>,
    tasks: HashMap<String, A2ATask>,
    max_tasks: usize,
}

impl A2ARegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            tasks: HashMap::new(),
            max_tasks: 1000,
        }
    }

    pub fn register(&mut self, card: AgentCard) {
        self.agents.insert(card.agent_name.clone(), card);
    }

    pub fn unregister(&mut self, name: &str) {
        self.agents.remove(name);
    }

    pub fn get(&self, name: &str) -> Option<&AgentCard> {
        self.agents.get(name)
    }

    pub fn list_agents(&self) -> Vec<&AgentCard> {
        self.agents.values().collect()
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentCard> {
        self.agents
            .values()
            .filter(|card| {
                card.capabilities
                    .iter()
                    .any(|c| c.id == capability || c.name == capability)
            })
            .collect()
    }

    pub fn find_by_skill(&self, skill: &str) -> Vec<&AgentCard> {
        self.agents
            .values()
            .filter(|card| card.skills.iter().any(|s| s.name == skill))
            .collect()
    }

    pub fn create_task(&mut self, task: A2ATask) {
        if self.tasks.len() >= self.max_tasks {
            if let Some(oldest) = self.tasks.keys().next().cloned() {
                self.tasks.remove(&oldest);
            }
        }
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get_task(&self, task_id: &str) -> Option<&A2ATask> {
        self.tasks.get(task_id)
    }

    pub fn get_task_mut(&mut self, task_id: &str) -> Option<&mut A2ATask> {
        self.tasks.get_mut(task_id)
    }

    pub fn active_tasks(&self) -> Vec<&A2ATask> {
        self.tasks
            .values()
            .filter(|t| !t.state.is_terminal())
            .collect()
    }

    pub fn complete_task(&mut self, task_id: &str, result_message: A2AMessage) -> Option<A2ATask> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.add_message(result_message);
            task.transition(TaskState::Completed);
            return Some(task.clone());
        }
        None
    }

    pub fn fail_task(&mut self, task_id: &str, error: &str) -> Option<A2ATask> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.transition(TaskState::Failed {
                error: error.to_string(),
            });
            return Some(task.clone());
        }
        None
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn all_tasks(&self) -> Vec<&A2ATask> {
        self.tasks.values().collect()
    }
}

impl Default for A2ARegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_new() {
        let card = AgentCard::new("test-agent")
            .capability(
                "code-gen",
                "Code Generation",
                "Generates code from descriptions",
            )
            .endpoint("a2a", "http://localhost:8080/a2a");
        assert_eq!(card.agent_name, "test-agent");
        assert_eq!(card.capabilities.len(), 1);
        assert_eq!(card.capabilities[0].id, "code-gen");
        assert!(card.endpoints.contains_key("a2a"));
    }

    #[test]
    fn test_agent_card_to_json() {
        let card = AgentCard::new("json-agent").capability("echo", "Echo", "Echoes input");
        let json = card.to_json();
        assert!(json.is_object());
        assert_eq!(json["agent_name"], "json-agent");
    }

    #[test]
    fn test_a2a_task_lifecycle() {
        let mut task = A2ATask::new("task-1", "session-1");
        assert_eq!(task.state, TaskState::Submitted);
        assert!(!task.state.is_terminal());

        task.transition(TaskState::Working);
        assert_eq!(task.state, TaskState::Working);
        assert_eq!(task.history.len(), 2);

        let msg = A2AMessage::text(A2ARole::Agent, "hello from agent");
        task.add_message(msg);
        assert_eq!(task.messages.len(), 1);

        task.transition(TaskState::Completed);
        assert!(task.state.is_terminal());
        assert_eq!(task.history.len(), 3);
    }

    #[test]
    fn test_task_state_labels() {
        assert_eq!(TaskState::Submitted.label(), "submitted");
        assert_eq!(TaskState::Completed.label(), "completed");
        assert_eq!(
            TaskState::Failed {
                error: "err".to_string()
            }
            .label(),
            "failed"
        );
    }

    #[test]
    fn test_a2a_registry_register_and_find() {
        let mut registry = A2ARegistry::new();
        let card = AgentCard::new("finder").capability("search", "Search", "Search capability");
        registry.register(card);

        assert_eq!(registry.agent_count(), 1);
        let found = registry.find_by_capability("search");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].agent_name, "finder");

        let found2 = registry.find_by_capability("nonexistent");
        assert!(found2.is_empty());
    }

    #[test]
    fn test_a2a_registry_task_management() {
        let mut registry = A2ARegistry::new();
        let task = A2ATask::new("t1", "s1");
        registry.create_task(task);
        assert_eq!(registry.task_count(), 1);

        let msg = A2AMessage::text(A2ARole::User, "result");
        let completed = registry.complete_task("t1", msg);
        assert!(completed.is_some());
        assert_eq!(completed.unwrap().state, TaskState::Completed);

        let active = registry.active_tasks();
        assert!(active.is_empty());
    }

    #[test]
    fn test_a2a_registry_fail_task() {
        let mut registry = A2ARegistry::new();
        registry.create_task(A2ATask::new("t2", "s1"));
        let failed = registry.fail_task("t2", "something went wrong");
        assert!(failed.is_some());
        assert_eq!(
            failed.unwrap().state,
            TaskState::Failed {
                error: "something went wrong".to_string()
            }
        );
    }

    #[test]
    fn test_a2a_message_with_metadata() {
        let msg = A2AMessage::text(A2ARole::Agent, "hi").with_metadata("lang", "en");
        assert_eq!(msg.metadata.get("lang").unwrap(), "en");
        assert_eq!(msg.role, A2ARole::Agent);
    }

    #[test]
    fn test_agent_skill() {
        let card = AgentCard::new("skilled-agent").capability("code", "Code", "Coding");
        let mut card_with_skill = card.clone();
        card_with_skill.skills.push(AgentSkill {
            id: "s1".to_string(),
            name: "rust-programming".to_string(),
            description: "Expert Rust developer".to_string(),
            confidence: 0.95,
        });

        let mut registry = A2ARegistry::new();
        registry.register(card_with_skill);

        let found = registry.find_by_skill("rust-programming");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].agent_name, "skilled-agent");
    }

    #[test]
    fn test_task_add_message() {
        let mut task = A2ATask::new("t3", "s1");
        let text_part = A2APart::Text {
            text: "hello".to_string(),
        };
        let file_part = A2APart::File {
            mime_type: "text/plain".to_string(),
            data: "base64data".to_string(),
            name: "file.txt".to_string(),
        };
        let data_part = A2APart::Data {
            key: "count".to_string(),
            value: serde_json::json!(42),
        };

        task.add_message(A2AMessage {
            role: A2ARole::User,
            parts: vec![text_part, file_part, data_part],
            metadata: HashMap::new(),
        });

        assert_eq!(task.messages.len(), 1);
        assert_eq!(task.messages[0].parts.len(), 3);
    }

    #[test]
    fn test_a2a_registry_unregister() {
        let mut registry = A2ARegistry::new();
        registry.register(AgentCard::new("temp"));
        assert_eq!(registry.agent_count(), 1);
        registry.unregister("temp");
        assert_eq!(registry.agent_count(), 0);
    }

    #[test]
    fn test_get_task_mut_updates_state() {
        let mut registry = A2ARegistry::new();
        registry.create_task(A2ATask::new("t4", "s1"));

        if let Some(task) = registry.get_task_mut("t4") {
            task.transition(TaskState::Working);
        }

        let task = registry.get_task("t4").unwrap();
        assert_eq!(task.state, TaskState::Working);
    }
}
