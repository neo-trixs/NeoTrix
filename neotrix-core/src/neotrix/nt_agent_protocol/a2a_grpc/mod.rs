mod bridge;
mod client;
mod conversion;
mod grpc_bridge;
mod server;
mod types;

pub use self::bridge::*;
pub use self::client::*;
pub use self::grpc_bridge::*;
pub use self::server::*;
pub use self::types::*;

// Re-export from parent a2a module (maintaining original visibility)
pub use super::a2a::{AgentInterface, ProtocolBinding};

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::a2a::TaskState;
    use super::super::a2a_grpc::conversion::{parse_task_status, uuid_v4};
    use super::*;
    use crate::core::nt_core_util::A2A_DEFAULT_PORT;

    #[test]
    fn test_grpc_frame_roundtrip() {
        let payload = b"hello grpc";
        let encoded = GrpcFrame::encode_raw(payload);
        assert_eq!(encoded.len(), 5 + payload.len());
        assert_eq!(encoded[0], 0);

        let (frame, remaining) = GrpcFrame::decode(&encoded).expect("decode should succeed");
        assert!(!frame.compressed);
        assert_eq!(frame.payload, payload);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_grpc_frame_multiple() {
        let payload1 = b"msg1";
        let payload2 = b"msg2";
        let mut combined = GrpcFrame::encode_raw(payload1);
        combined.extend_from_slice(&GrpcFrame::encode_raw(payload2));

        let frames = GrpcFrame::decode_all(&combined).expect("decode all");
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].payload, payload1);
        assert_eq!(frames[1].payload, payload2);
    }

    #[test]
    fn test_grpc_frame_truncated() {
        let buf = vec![0u8, 0, 0, 0, 10];
        let result = GrpcFrame::decode(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_grpc_frame_short_header() {
        let buf = vec![0u8, 0, 0];
        let result = GrpcFrame::decode(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_grpc_frame_invalid_compressed() {
        let buf = vec![2u8, 0, 0, 0, 1, 0];
        let result = GrpcFrame::decode(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_grpc_method_paths() {
        assert_eq!(
            GrpcMethod::from_path("/a2a.A2AService/SendMessage"),
            Some(GrpcMethod::SendMessage)
        );
        assert_eq!(
            GrpcMethod::from_path("/a2a.A2AService/GetTask"),
            Some(GrpcMethod::GetTask)
        );
        assert_eq!(
            GrpcMethod::from_path("/a2a.A2AService/CancelTask"),
            Some(GrpcMethod::CancelTask)
        );
        assert_eq!(
            GrpcMethod::from_path("/a2a.A2AService/SubscribeToTask"),
            Some(GrpcMethod::SubscribeToTask)
        );
        assert_eq!(
            GrpcMethod::from_path("/a2a.A2AService/ListTasks"),
            Some(GrpcMethod::ListTasks)
        );
        assert_eq!(GrpcMethod::from_path("/unknown"), None);
    }

    #[test]
    fn test_grpc_method_path_roundtrip() {
        for method in &[
            GrpcMethod::SendMessage,
            GrpcMethod::GetTask,
            GrpcMethod::CancelTask,
            GrpcMethod::SubscribeToTask,
            GrpcMethod::ListTasks,
        ] {
            let path = method.path();
            let back = GrpcMethod::from_path(path);
            assert_eq!(back, Some(*method));
        }
    }

    #[test]
    fn test_protocol_binding_serialization() {
        let grpc_json = serde_json::to_string(&ProtocolBinding::Grpc).unwrap_or_default();
        assert_eq!(grpc_json, "\"GRPC\"");

        let rest_json = serde_json::to_string(&ProtocolBinding::HttpJsonRest).unwrap_or_default();
        assert_eq!(rest_json, "\"HTTP+JSON\"");

        let jsonrpc_json = serde_json::to_string(&ProtocolBinding::JsonRpc).unwrap_or_default();
        assert_eq!(jsonrpc_json, "\"JSONRPC\"");
    }

    #[test]
    fn test_agent_interface_serialization() {
        let iface = AgentInterface {
            url: "http://localhost:42072".into(),
            protocol_binding: "GRPC".into(),
            protocol_version: "1.0".into(),
        };
        let json = serde_json::to_string(&iface).unwrap_or_default();
        assert!(json.contains("\"url\":"));
        assert!(json.contains("\"protocolBinding\":"));
        assert!(json.contains("\"protocolVersion\":"));
        assert!(json.contains("GRPC"));

        let deserialized: AgentInterface = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(deserialized.url, "http://localhost:42072");
        assert_eq!(deserialized.protocol_binding, "GRPC");
    }

    #[test]
    fn test_grpc_types_roundtrip() {
        let req = GrpcSendMessageRequest {
            tenant: None,
            message: GrpcMessage {
                role: "user".into(),
                parts: vec![GrpcPart {
                    part_type: "text".into(),
                    text: Some("hello".into()),
                    mime_type: None,
                }],
            },
            configuration: None,
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&req).unwrap_or_default();
        let deserialized: GrpcSendMessageRequest = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(deserialized.message.parts.len(), 1);
        assert_eq!(deserialized.message.parts[0].text.as_deref(), Some("hello"));
    }

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
    fn test_parse_task_status() {
        assert_eq!(parse_task_status("submitted"), TaskState::Submitted);
        assert_eq!(parse_task_status("COMPLETED"), TaskState::Completed);
        assert_eq!(parse_task_status("Canceled"), TaskState::Canceled);
        assert_eq!(parse_task_status("cancelled"), TaskState::Canceled);
        assert_eq!(parse_task_status("failed"), TaskState::Failed);
        assert_eq!(parse_task_status("unknown"), TaskState::Submitted);
    }

    #[test]
    fn test_grpc_frame_encode_json() {
        let value = serde_json::json!({"hello": "world"});
        let encoded = GrpcFrame::encode_json(&value).unwrap();
        let (frame, _) = GrpcFrame::decode(&encoded).unwrap();
        let decoded: serde_json::Value = serde_json::from_slice(&frame.payload).unwrap_or_default();
        assert_eq!(decoded["hello"], "world");
    }

    #[test]
    fn test_grpc_agent_card_with_interfaces() {
        let interfaces = vec![
            AgentInterface {
                url: format!("http://localhost:{}", A2A_DEFAULT_PORT),
                protocol_binding: "HTTP+JSON".into(),
                protocol_version: "1.0".into(),
            },
            AgentInterface {
                url: "http://localhost:42072".into(),
                protocol_binding: "GRPC".into(),
                protocol_version: "1.0".into(),
            },
        ];
        let json = serde_json::to_string(&interfaces).unwrap_or_default();
        assert!(json.contains(&A2A_DEFAULT_PORT.to_string()));
        assert!(json.contains("42072"));
        assert!(json.contains("HTTP+JSON"));
        assert!(json.contains("GRPC"));
    }
}
