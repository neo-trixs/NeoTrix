use agent_core::card::{AgentCard, SkillDecl};
use agent_core::task::{A2ATask, TaskState, SendTaskRequest, A2AMessage, A2APart, A2APartType, A2AArtifact};
use std::collections::HashMap;

#[test]
fn agent_card_defaults() {
    let skills = vec![SkillDecl {
        id: "web-search".into(),
        name: "Web Search".into(),
        description: "Search the web".into(),
        tags: vec!["search".into()],
        examples: vec!["find latest news".into()],
    }];
    let card = AgentCard::new("test-agent", "A test agent", "http://localhost:0", "1.0.0", skills);

    assert_eq!(card.name, "test-agent");
    assert_eq!(card.version, "1.0.0");
    assert!(card.capabilities.streaming);
    assert!(!card.capabilities.push_notifications);
    assert_eq!(card.skill_ids(), vec!["web-search"]);
    assert!(card.has_skill("web-search"));
    assert!(!card.has_skill("nonexistent"));
}

#[test]
fn agent_card_roundtrip_serde() {
    let card = AgentCard::new("serde-test", "Serde roundtrip", "http://localhost:0", "0.1.0", vec![]);
    let json = serde_json::to_string(&card).expect("serialize");
    let back: AgentCard = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(card.name, back.name);
    assert_eq!(card.description, back.description);
    assert_eq!(card.url, back.url);
}

#[test]
fn a2a_task_default_state() {
    let task = A2ATask {
        id: "task-1".into(),
        session_id: "session-1".into(),
        status: TaskState::Submitted,
        messages: vec![],
        artifacts: vec![],
        metadata: HashMap::new(),
    };

    assert_eq!(task.id, "task-1");
    assert!(matches!(task.status, TaskState::Submitted));
}

#[test]
fn a2a_task_serde_roundtrip() {
    let msg = A2AMessage {
        role: "user".into(),
        parts: vec![A2APart {
            r#type: A2APartType::Text,
            text: Some("hello".into()),
            data: None,
        }],
    };
    let task = A2ATask {
        id: "task-2".into(),
        session_id: "session-2".into(),
        status: TaskState::Completed,
        messages: vec![msg],
        artifacts: vec![A2AArtifact {
            id: "art-1".into(),
            name: "result".into(),
            mime_type: "text/plain".into(),
        }],
        metadata: [("key".into(), "value".into())].into(),
    };

    let json = serde_json::to_string(&task).expect("serialize task");
    let back: A2ATask = serde_json::from_str(&json).expect("deserialize task");
    assert_eq!(back.messages.len(), 1);
    assert_eq!(back.artifacts.len(), 1);
    assert_eq!(back.metadata.get("key").unwrap(), "value");
}

#[test]
fn send_task_request_construction() {
    let req = SendTaskRequest {
        id: "req-1".into(),
        session_id: "sess-1".into(),
        messages: vec![],
        metadata: [("origin".into(), "test".into())].into(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let back: SendTaskRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.id, "req-1");
}
