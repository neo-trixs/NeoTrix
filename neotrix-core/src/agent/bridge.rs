//! Bridge between `agent/` (runtime execution) and `neotrix::nt_agent_core/` (data model & protocol).
//! This is the ONLY file in `agent/` that should import from `neotrix::nt_agent_core`.
//!
//! Per AGENT_UNIFICATION.md (2026-06-18):
//! - `agent::agent_bus::AgentBus` = supervisor-worker task orchestration (KEEP)
//! - `neotrix::nt_agent_core::bus::AgentCommunicationBus` = peer-to-peer message passing (KEEP)
//! - This bridge connects them via type conversion.

use crate::neotrix::nt_agent_core;

// ── Re-exports: agent/ modules use these instead of importing neotrix/ directly ──

pub use nt_agent_core::bus::AgentCommunicationBus;
pub use nt_agent_core::bus::BusStats;
pub use nt_agent_core::message::{
    AgentId, AgentMessage, AgentStatus, MessageContent, MessagePriority,
};
pub use nt_agent_core::sub_agent::{IsolationStrategy, SubAgentCapability};

// ── Constants ──

pub const BRIDGE_AGENT_NAME: &str = "agent_bridge";
pub const BRIDGE_AGENT_VERSION: &str = "0.1";

// ── Conversion: agent::agent_bus::BusTask → core::AgentMessage ──

/// Convert a supervisor BusTask into a TaskRequest message for the peer-to-peer bus.
pub fn bus_task_to_task_request(
    task: &super::agent_bus::BusTask,
    priority: MessagePriority,
) -> AgentMessage {
    AgentMessage::new(
        AgentId::new(BRIDGE_AGENT_NAME, BRIDGE_AGENT_VERSION),
        vec![AgentId::new(&task.supervisor_id, "0.1")],
        MessageContent::TaskRequest {
            description: task.description.clone(),
            domain: task
                .required_capabilities
                .first()
                .cloned()
                .unwrap_or_default(),
            constraints: task.required_capabilities.clone(),
        },
        priority,
        std::time::Duration::from_secs(30),
    )
}

/// Extract a summary string from an AgentMessage for agent::agent_bus logging.
pub fn agent_message_summary(msg: &AgentMessage) -> String {
    let kind = msg.content.kind();
    let recipients: Vec<String> = msg.recipients.iter().map(|r| r.name.clone()).collect();
    format!(
        "[#{}] {} → [{}]: {}",
        msg.id,
        msg.sender.name,
        recipients.join(", "),
        kind,
    )
}

/// Convert an AgentMessage into a BusTask for supervisor re-assignment.
pub fn agent_message_to_bus_task(msg: &AgentMessage) -> super::agent_bus::BusTask {
    let desc = match &msg.content {
        MessageContent::TaskRequest { description, .. } => description.clone(),
        MessageContent::Query { question, .. } => question.clone(),
        MessageContent::Response { answer, .. } => answer.clone(),
        MessageContent::StatusUpdate { message, .. } => message.clone(),
        MessageContent::Error { description, .. } => description.clone(),
        MessageContent::Coordination { rationale, .. } => rationale.clone(),
        MessageContent::TaskResult { output, .. } => output.clone(),
    };

    let mut task = super::agent_bus::BusTask::new(&msg.sender.name, &desc);
    for r in &msg.recipients {
        task = task.with_capability(&r.name);
    }
    task
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_task_to_task_request() {
        let mut task = super::super::agent_bus::BusTask::new("supervisor_1", "analyze data");
        task = task.with_capability("nlp");
        let msg = bus_task_to_task_request(&task, MessagePriority::Normal);
        assert_eq!(msg.sender.name, BRIDGE_AGENT_NAME);
        assert_eq!(msg.recipients[0].name, "supervisor_1");
        match &msg.content {
            MessageContent::TaskRequest {
                description,
                domain,
                ..
            } => {
                assert_eq!(description, "analyze data");
                assert_eq!(domain, "nlp");
            }
            _ => panic!("expected TaskRequest"),
        }
    }

    #[test]
    fn test_agent_message_summary() {
        let msg = AgentMessage::new(
            AgentId::new("alice", "1.0"),
            vec![AgentId::new("bob", "1.0")],
            MessageContent::Query {
                question: "hello?".to_string(),
                context: vec![],
            },
            MessagePriority::Normal,
            std::time::Duration::from_secs(5),
        );
        let s = agent_message_summary(&msg);
        assert!(s.contains("alice"));
        assert!(s.contains("bob"));
    }

    #[test]
    fn test_agent_message_to_bus_task() {
        let msg = AgentMessage::new(
            AgentId::new("worker_1", "1.0"),
            vec![AgentId::new("coordinator", "1.0")],
            MessageContent::TaskRequest {
                description: "process image".to_string(),
                domain: "vision".to_string(),
                constraints: vec![],
            },
            MessagePriority::High,
            std::time::Duration::from_secs(60),
        );
        let task = agent_message_to_bus_task(&msg);
        assert_eq!(task.supervisor_id, "worker_1");
        assert_eq!(task.description, "process image");
    }
}
