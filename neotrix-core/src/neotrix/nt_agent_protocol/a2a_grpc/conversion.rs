use std::time::{SystemTime, UNIX_EPOCH};

use super::super::a2a::{A2ATask, TaskState};
use super::types::{GrpcArtifact, GrpcMessage, GrpcPart, GrpcTask};

pub(crate) fn convert_to_grpc_task(task: &A2ATask) -> GrpcTask {
    GrpcTask {
        id: task.id.clone(),
        session_id: task.session_id.clone(),
        status: format!("{:?}", task.status).to_lowercase(),
        messages: task
            .messages
            .iter()
            .map(|m| GrpcMessage {
                role: m.role.clone(),
                parts: m
                    .parts
                    .iter()
                    .map(|p| GrpcPart {
                        part_type: format!("{:?}", p.part_type).to_lowercase(),
                        text: p.text.clone(),
                        mime_type: p.mime_type.clone(),
                    })
                    .collect(),
            })
            .collect(),
        artifacts: task
            .artifacts
            .iter()
            .map(|a| GrpcArtifact {
                id: a.id.clone(),
                name: a.name.clone(),
                mime_type: a.mime_type.clone(),
                uri: a.uri.clone(),
                metadata: a.metadata.clone(),
            })
            .collect(),
        metadata: task.metadata.clone(),
    }
}

pub(crate) fn parse_task_status(s: &str) -> TaskState {
    match s.to_lowercase().as_str() {
        "submitted" => TaskState::Submitted,
        "working" => TaskState::Working,
        "inputrequired" | "input_required" => TaskState::InputRequired,
        "completed" => TaskState::Completed,
        "failed" => TaskState::Failed,
        "canceled" | "cancelled" => TaskState::Canceled,
        _ => TaskState::Submitted,
    }
}

pub(crate) fn uuid_v4() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let a = (ts & 0xFFFF_FFFF_FFFF) as u64;
    let b = ((ts >> 48) & 0xFFFF) as u64;
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (a >> 16) as u32,
        (a & 0xFFFF) as u16,
        ((b >> 8) & 0x0FFF) as u16,
        ((b & 0xFF) as u16 | 0x8000),
        (a >> 32) as u64 & 0xFFFF_FFFF_FFFF
    )
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::super::a2a::{
        A2AArtifact, A2AMessage, A2APart, A2APartType, A2ATask, TaskState,
    };
    use super::*;

    // ── convert_to_grpc_task ──────────────────────────────────────────────

    fn sample_a2a_task() -> A2ATask {
        A2ATask {
            id: "task-1".into(),
            session_id: "session-abc".into(),
            status: TaskState::Working,
            messages: vec![A2AMessage {
                role: "user".into(),
                parts: vec![A2APart {
                    part_type: A2APartType::Text,
                    text: Some("hello".into()),
                    mime_type: Some("text/plain".into()),
                    file_uri: None,
                    data: None,
                }],
            }],
            artifacts: vec![A2AArtifact {
                id: "art-1".into(),
                name: "output.txt".into(),
                mime_type: "text/plain".into(),
                uri: Some("file:///output.txt".into()),
                metadata: std::collections::HashMap::new(),
            }],
            error_message: None,
            metadata: {
                let mut m = std::collections::HashMap::new();
                m.insert("source".into(), "test".into());
                m
            },
        }
    }

    #[test]
    fn test_convert_to_grpc_task_basic() {
        let task = sample_a2a_task();
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.id, "task-1");
        assert_eq!(grpc.session_id, "session-abc");
        assert_eq!(grpc.status, "working");
        assert_eq!(grpc.messages.len(), 1);
        assert_eq!(grpc.artifacts.len(), 1);
    }

    #[test]
    fn test_convert_to_grpc_task_messages() {
        let task = sample_a2a_task();
        let grpc = convert_to_grpc_task(&task);
        let msg = &grpc.messages[0];
        assert_eq!(msg.role, "user");
        assert_eq!(msg.parts.len(), 1);
        assert_eq!(msg.parts[0].part_type, "text");
        assert_eq!(msg.parts[0].text.as_deref(), Some("hello"));
        assert_eq!(msg.parts[0].mime_type.as_deref(), Some("text/plain"));
    }

    #[test]
    fn test_convert_to_grpc_task_artifacts() {
        let task = sample_a2a_task();
        let grpc = convert_to_grpc_task(&task);
        let art = &grpc.artifacts[0];
        assert_eq!(art.id, "art-1");
        assert_eq!(art.name, "output.txt");
        assert_eq!(art.mime_type, "text/plain");
        assert_eq!(art.uri.as_deref(), Some("file:///output.txt"));
    }

    #[test]
    fn test_convert_to_grpc_task_metadata() {
        let task = sample_a2a_task();
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.metadata.get("source").unwrap(), "test");
    }

    #[test]
    fn test_convert_to_grpc_task_empty_task() {
        let task = A2ATask {
            id: "empty".into(),
            session_id: String::new(),
            status: TaskState::Submitted,
            messages: vec![],
            artifacts: vec![],
            error_message: None,
            metadata: std::collections::HashMap::new(),
        };
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.id, "empty");
        assert!(grpc.messages.is_empty());
        assert!(grpc.artifacts.is_empty());
        assert!(grpc.metadata.is_empty());
    }

    #[test]
    fn test_convert_to_grpc_task_with_multiple_messages() {
        let task = A2ATask {
            id: "multi".into(),
            session_id: String::new(),
            status: TaskState::Completed,
            messages: vec![
                A2AMessage {
                    role: "user".into(),
                    parts: vec![A2APart {
                        part_type: A2APartType::Text,
                        text: Some("q1".into()),
                        mime_type: None,
                        file_uri: None,
                        data: None,
                    }],
                },
                A2AMessage {
                    role: "assistant".into(),
                    parts: vec![A2APart {
                        part_type: A2APartType::Text,
                        text: Some("a1".into()),
                        mime_type: None,
                        file_uri: None,
                        data: None,
                    }],
                },
            ],
            artifacts: vec![],
            error_message: None,
            metadata: std::collections::HashMap::new(),
        };
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.messages.len(), 2);
        assert_eq!(grpc.messages[0].role, "user");
        assert_eq!(grpc.messages[1].role, "assistant");
    }

    #[test]
    fn test_convert_to_grpc_task_completed_status() {
        let mut task = sample_a2a_task();
        task.status = TaskState::Completed;
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.status, "completed");
    }

    #[test]
    fn test_convert_to_grpc_task_failed_status() {
        let mut task = sample_a2a_task();
        task.status = TaskState::Failed;
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.status, "failed");
    }

    #[test]
    fn test_convert_to_grpc_task_canceled_status() {
        let mut task = sample_a2a_task();
        task.status = TaskState::Canceled;
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.status, "canceled");
    }

    #[test]
    fn test_convert_to_grpc_task_input_required_status() {
        let mut task = sample_a2a_task();
        task.status = TaskState::InputRequired;
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(grpc.status, "inputrequired");
    }

    #[test]
    fn test_convert_to_grpc_task_with_mime_type_on_part() {
        let mut task = sample_a2a_task();
        task.messages[0].parts[0].mime_type = Some("application/json".into());
        let grpc = convert_to_grpc_task(&task);
        assert_eq!(
            grpc.messages[0].parts[0].mime_type.as_deref(),
            Some("application/json")
        );
    }

    // ── parse_task_status ─────────────────────────────────────────────────

    #[test]
    fn test_parse_task_status_submitted() {
        assert_eq!(parse_task_status("submitted"), TaskState::Submitted);
        assert_eq!(parse_task_status("SUBMITTED"), TaskState::Submitted);
    }

    #[test]
    fn test_parse_task_status_working() {
        assert_eq!(parse_task_status("working"), TaskState::Working);
        assert_eq!(parse_task_status("WORKING"), TaskState::Working);
    }

    #[test]
    fn test_parse_task_status_input_required_variants() {
        assert_eq!(parse_task_status("inputrequired"), TaskState::InputRequired);
        assert_eq!(
            parse_task_status("input_required"),
            TaskState::InputRequired
        );
        assert_eq!(parse_task_status("InputRequired"), TaskState::InputRequired);
    }

    #[test]
    fn test_parse_task_status_completed() {
        assert_eq!(parse_task_status("completed"), TaskState::Completed);
        assert_eq!(parse_task_status("COMPLETED"), TaskState::Completed);
    }

    #[test]
    fn test_parse_task_status_failed() {
        assert_eq!(parse_task_status("failed"), TaskState::Failed);
    }

    #[test]
    fn test_parse_task_status_canceled_variants() {
        assert_eq!(parse_task_status("canceled"), TaskState::Canceled);
        assert_eq!(parse_task_status("cancelled"), TaskState::Canceled);
        assert_eq!(parse_task_status("CANCELED"), TaskState::Canceled);
    }

    #[test]
    fn test_parse_task_status_empty_string() {
        assert_eq!(parse_task_status(""), TaskState::Submitted);
    }

    #[test]
    fn test_parse_task_status_whitespace() {
        assert_eq!(parse_task_status("  completed"), TaskState::Submitted);
    }

    #[test]
    fn test_parse_task_status_unknown() {
        assert_eq!(parse_task_status("unknown"), TaskState::Submitted);
        assert_eq!(parse_task_status("pending"), TaskState::Submitted);
    }

    // ── uuid_v4 ───────────────────────────────────────────────────────────

    #[test]
    fn test_uuid_v4_format() {
        let id = uuid_v4();
        assert_eq!(id.len(), 36);
        assert_eq!(&id[8..9], "-");
        assert_eq!(&id[13..14], "-");
        assert_eq!(&id[18..19], "-");
        assert_eq!(&id[23..24], "-");
    }

    #[test]
    fn test_uuid_v4_unique() {
        let a = uuid_v4();
        let b = uuid_v4();
        assert_ne!(a, b);
    }

    #[test]
    fn test_uuid_v4_version_bits() {
        let id = uuid_v4();
        // version nibble should be 4 at position 14 (0-indexed: 14)
        assert_eq!(&id[14..15], "4", "UUID version must be 4");
        // variant bits: position 19 should be 8, 9, a, or b
        let variant_char = &id[19..20];
        assert!(
            ["8", "9", "a", "b"].contains(&variant_char),
            "UUID variant must be 8/9/a/b, got {variant_char}"
        );
    }

    #[test]
    fn test_uuid_v4_hex_characters_only() {
        let id = uuid_v4();
        let hex_chars: String = id.chars().filter(|c| *c != '-').collect();
        assert!(hex_chars.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
