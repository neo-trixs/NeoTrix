use std::collections::HashMap;
use std::time::Duration;

use crate::core::nt_core_agent::message::AgentId;
use crate::core::nt_core_agent::message::AgentMessage;
use crate::core::nt_core_agent::message::MessageContent;
use crate::core::nt_core_agent::message::MessagePriority;

use super::types::{A2AMessage, A2APart, A2APartType, A2ATask, TaskState};

// ── A2A Bridge ─────────────────────────────────────────────────────────────

pub fn a2a_task_to_message(task: &A2ATask, self_id: &AgentId, target: AgentId) -> AgentMessage {
    let text = task
        .messages
        .iter()
        .flat_map(|m| m.parts.iter().filter_map(|p| p.text.clone()))
        .collect::<Vec<_>>()
        .join("\n");

    AgentMessage::new(
        self_id.clone(),
        vec![target],
        MessageContent::TaskRequest {
            description: text,
            domain: "a2a".into(),
            constraints: task.metadata.values().cloned().collect(),
        },
        MessagePriority::Normal,
        Duration::from_secs(300),
    )
}

pub fn agent_message_to_a2a_task(msg_id: &str, msg: &AgentMessage) -> A2ATask {
    let text = match &msg.content {
        MessageContent::Query { question, .. } => question.clone(),
        MessageContent::TaskRequest { description, .. } => description.clone(),
        MessageContent::Response { answer, .. } => answer.clone(),
        _ => format!("{:?}", msg.content),
    };
    A2ATask {
        id: msg_id.to_string(),
        session_id: msg.conversation_id.to_string(),
        status: TaskState::Completed,
        messages: vec![A2AMessage {
            role: if msg.sender.instance_id == 0 {
                "user"
            } else {
                "assistant"
            }
            .into(),
            parts: vec![A2APart {
                part_type: A2APartType::Text,
                text: Some(text),
                mime_type: Some("text/plain".into()),
                file_uri: None,
                data: None,
            }],
        }],
        artifacts: vec![],
        error_message: None,
        metadata: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_agent::bus::AgentCommunicationBus;

    fn test_agent_id() -> AgentId {
        AgentId::with_random_instance("a2a-test", "1.0")
    }

    #[test]
    fn test_bridge_message_conversion() {
        let self_id = test_agent_id();
        let target = AgentId::with_random_instance("remote-agent", "1.0");
        let task = A2ATask {
            id: "bridge-test".into(),
            session_id: "".into(),
            status: TaskState::Submitted,
            messages: vec![A2AMessage {
                role: "user".into(),
                parts: vec![A2APart {
                    part_type: A2APartType::Text,
                    text: Some("translate this".into()),
                    mime_type: None,
                    file_uri: None,
                    data: None,
                }],
            }],
            artifacts: vec![],
            error_message: None,
            metadata: HashMap::new(),
        };

        let msg = a2a_task_to_message(&task, &self_id, target);
        assert_eq!(msg.sender.name, "a2a-test");

        let back = agent_message_to_a2a_task("rev-1", &msg);
        assert_eq!(back.id, "rev-1");
        assert!(back.messages[0].parts[0]
            .text
            .as_deref()
            .unwrap_or("")
            .contains("translate"));
    }
}
