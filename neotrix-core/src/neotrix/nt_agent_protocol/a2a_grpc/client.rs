use std::collections::HashMap;

use super::super::a2a::{A2AArtifact, A2AMessage, A2APart, A2APartType, A2ATask, SendTaskRequest};
use super::conversion::parse_task_status;
use super::types::{
    GrpcCancelTaskRequest, GrpcFrame, GrpcGetTaskRequest, GrpcMessage, GrpcMethod, GrpcPart,
    GrpcSendMessageRequest, GrpcSendMessageResponse, GrpcTask,
};

pub struct A2AGrpcClient {
    host: String,
    port: u16,
}

impl A2AGrpcClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    async fn call(&self, method: &str, request_json: &[u8]) -> Result<Vec<u8>, String> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("grpc connect to {addr}: {e}"))?;

        let grpc_frame = GrpcFrame::encode_raw(request_json);
        let content_length = grpc_frame.len();

        let preamble = format!(
            "POST {method} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Content-Type: application/grpc\r\n\
             TE: trailers\r\n\
             Content-Length: {content_length}\r\n\
             \r\n",
            host = self.host,
        );

        let mut full_request = preamble.into_bytes();
        full_request.extend_from_slice(&grpc_frame);

        let (mut reader, mut writer) = stream.into_split();
        use tokio::io::AsyncWriteExt;
        writer
            .write_all(&full_request)
            .await
            .map_err(|e| format!("grpc write: {e}"))?;
        writer
            .flush()
            .await
            .map_err(|e| format!("grpc flush: {e}"))?;

        use tokio::io::AsyncReadExt;
        let mut resp_buf = Vec::new();
        reader
            .read_to_end(&mut resp_buf)
            .await
            .map_err(|e| format!("grpc read: {e}"))?;

        if let Some(body_start) = resp_buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let body = &resp_buf[body_start + 4..];
            if body.is_empty() {
                return Err("grpc empty response body".into());
            }
            let (frame, _) = GrpcFrame::decode(body)?;
            Ok(frame.payload)
        } else {
            Err("grpc no response headers found".into())
        }
    }

    pub async fn send_task(&self, request: &SendTaskRequest) -> Result<A2ATask, String> {
        let grpc_req = GrpcSendMessageRequest {
            tenant: None,
            message: GrpcMessage {
                role: "user".into(),
                parts: request
                    .messages
                    .iter()
                    .flat_map(|m| {
                        m.parts
                            .iter()
                            .map(|p| GrpcPart {
                                part_type: format!("{:?}", p.part_type).to_lowercase(),
                                text: p.text.clone(),
                                mime_type: p.mime_type.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            },
            configuration: None,
            metadata: request.metadata.clone(),
        };
        let req_json = serde_json::to_vec(&grpc_req).map_err(|e| format!("json: {e}"))?;
        let resp_bytes = self.call(GrpcMethod::SendMessage.path(), &req_json).await?;
        let resp: GrpcSendMessageResponse =
            serde_json::from_slice(&resp_bytes).map_err(|e| format!("grpc parse response: {e}"))?;

        let task = resp.task.ok_or_else(|| "no task in response".to_string())?;
        Ok(A2ATask {
            id: task.id,
            session_id: task.session_id,
            status: parse_task_status(&task.status),
            messages: task
                .messages
                .iter()
                .map(|m| A2AMessage {
                    role: m.role.clone(),
                    parts: m
                        .parts
                        .iter()
                        .map(|p| A2APart {
                            part_type: A2APartType::Text,
                            text: p.text.clone(),
                            mime_type: p.mime_type.clone(),
                            file_uri: None,
                            data: None,
                        })
                        .collect(),
                })
                .collect(),
            artifacts: task
                .artifacts
                .iter()
                .map(|a| A2AArtifact {
                    id: a.id.clone(),
                    name: a.name.clone(),
                    mime_type: a.mime_type.clone(),
                    uri: a.uri.clone(),
                    metadata: a.metadata.clone(),
                })
                .collect(),
            error_message: None,
            metadata: task.metadata,
        })
    }

    pub async fn get_task(&self, task_id: &str) -> Result<A2ATask, String> {
        let req = GrpcGetTaskRequest {
            id: task_id.to_string(),
        };
        let req_json = serde_json::to_vec(&req).map_err(|e| format!("json: {e}"))?;
        let resp_bytes = self.call(GrpcMethod::GetTask.path(), &req_json).await?;
        let task: GrpcTask =
            serde_json::from_slice(&resp_bytes).map_err(|e| format!("grpc parse task: {e}"))?;
        Ok(A2ATask {
            id: task.id,
            session_id: task.session_id,
            status: parse_task_status(&task.status),
            messages: vec![],
            artifacts: vec![],
            error_message: None,
            metadata: task.metadata,
        })
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<A2ATask, String> {
        let req = GrpcCancelTaskRequest {
            id: task_id.to_string(),
            metadata: HashMap::new(),
        };
        let req_json = serde_json::to_vec(&req).map_err(|e| format!("json: {e}"))?;
        let resp_bytes = self.call(GrpcMethod::CancelTask.path(), &req_json).await?;
        let task: GrpcTask =
            serde_json::from_slice(&resp_bytes).map_err(|e| format!("grpc parse cancel: {e}"))?;
        Ok(A2ATask {
            id: task.id,
            session_id: task.session_id,
            status: parse_task_status(&task.status),
            messages: vec![],
            artifacts: vec![],
            error_message: None,
            metadata: task.metadata,
        })
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::types::{
        GrpcArtifact, GrpcCancelTaskRequest, GrpcGetTaskRequest, GrpcMessage, GrpcPart,
        GrpcSendMessageRequest, GrpcTask,
    };
    use super::*;

    // ── Client construction ───────────────────────────────────────────────

    #[test]
    fn test_client_new() {
        let client = A2AGrpcClient::new("localhost", 42071);
        assert_eq!(client.host, "localhost");
        assert_eq!(client.port, 42071);
    }

    #[test]
    fn test_client_new_different_port() {
        let client = A2AGrpcClient::new("127.0.0.1", 42072);
        assert_eq!(client.host, "127.0.0.1");
        assert_eq!(client.port, 42072);
    }

    #[test]
    fn test_client_new_default_port() {
        let client = A2AGrpcClient::new("example.com", A2A_DEFAULT_PORT);
        assert_eq!(client.host, "example.com");
        assert_eq!(client.port, crate::core::nt_core_util::A2A_DEFAULT_PORT);
    }

    // ── Request construction ──────────────────────────────────────────────

    #[test]
    fn test_send_task_request_serialization() {
        let req = SendTaskRequest {
            id: "req-1".into(),
            session_id: String::new(),
            messages: vec![A2AMessage {
                role: "user".into(),
                parts: vec![A2APart {
                    part_type: A2APartType::Text,
                    text: Some("hello".into()),
                    mime_type: None,
                    file_uri: None,
                    data: None,
                }],
            }],
            metadata: std::collections::HashMap::new(),
        };
        let grpc_req = GrpcSendMessageRequest {
            tenant: None,
            message: GrpcMessage {
                role: "user".into(),
                parts: req
                    .messages
                    .iter()
                    .flat_map(|m| {
                        m.parts
                            .iter()
                            .map(|p| GrpcPart {
                                part_type: format!("{:?}", p.part_type).to_lowercase(),
                                text: p.text.clone(),
                                mime_type: p.mime_type.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            },
            configuration: None,
            metadata: req.metadata,
        };
        let json = serde_json::to_string(&grpc_req).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"text\":\"hello\""));
        assert!(json.contains("\"type\":\"text\""));
    }

    #[test]
    fn test_get_task_request_serialization() {
        let req = GrpcGetTaskRequest {
            id: "task-abc".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: GrpcGetTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "task-abc");
    }

    #[test]
    fn test_cancel_task_request_serialization() {
        let req = GrpcCancelTaskRequest {
            id: "task-xyz".into(),
            metadata: {
                let mut m = std::collections::HashMap::new();
                m.insert("reason".into(), "test".into());
                m
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: GrpcCancelTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "task-xyz");
        assert_eq!(back.metadata.get("reason").unwrap(), "test");
    }

    // ── Response parsing ─────────────────────────────────────────────────

    fn make_grpc_task(id: &str, status: &str) -> GrpcTask {
        GrpcTask {
            id: id.to_string(),
            session_id: "sess-1".into(),
            status: status.to_string(),
            messages: vec![GrpcMessage {
                role: "assistant".into(),
                parts: vec![GrpcPart {
                    part_type: "text".into(),
                    text: Some("response".into()),
                    mime_type: None,
                }],
            }],
            artifacts: vec![GrpcArtifact {
                id: "art-1".into(),
                name: "result.txt".into(),
                mime_type: "text/plain".into(),
                uri: Some("file:///result.txt".into()),
                metadata: std::collections::HashMap::new(),
            }],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_parse_send_task_response_to_a2a_task() {
        let grpc_task = make_grpc_task("t1", "completed");
        let resp = GrpcSendMessageResponse {
            task: Some(grpc_task),
            message: None,
        };
        let resp_bytes = serde_json::to_vec(&resp).unwrap();
        let parsed_resp: GrpcSendMessageResponse = serde_json::from_slice(&resp_bytes).unwrap();
        let task = parsed_resp.task.unwrap();

        let a2a = A2ATask {
            id: task.id,
            session_id: task.session_id,
            status: parse_task_status(&task.status),
            messages: task
                .messages
                .iter()
                .map(|m| A2AMessage {
                    role: m.role.clone(),
                    parts: m
                        .parts
                        .iter()
                        .map(|p| A2APart {
                            part_type: A2APartType::Text,
                            text: p.text.clone(),
                            mime_type: p.mime_type.clone(),
                            file_uri: None,
                            data: None,
                        })
                        .collect(),
                })
                .collect(),
            artifacts: task
                .artifacts
                .iter()
                .map(|a| A2AArtifact {
                    id: a.id.clone(),
                    name: a.name.clone(),
                    mime_type: a.mime_type.clone(),
                    uri: a.uri.clone(),
                    metadata: a.metadata.clone(),
                })
                .collect(),
            error_message: None,
            metadata: task.metadata,
        };

        assert_eq!(a2a.id, "t1");
        assert_eq!(a2a.status, TaskState::Completed);
        assert_eq!(a2a.messages.len(), 1);
        assert_eq!(a2a.messages[0].role, "assistant");
        assert_eq!(a2a.messages[0].parts[0].text.as_deref(), Some("response"));
        assert_eq!(a2a.artifacts.len(), 1);
        assert_eq!(a2a.artifacts[0].name, "result.txt");
    }

    #[test]
    fn test_parse_get_task_response_to_a2a_task() {
        let grpc_task = make_grpc_task("t2", "working");
        let resp_bytes = serde_json::to_vec(&grpc_task).unwrap();
        let task: GrpcTask = serde_json::from_slice(&resp_bytes).unwrap();

        let a2a = A2ATask {
            id: task.id,
            session_id: task.session_id,
            status: parse_task_status(&task.status),
            messages: vec![],
            artifacts: vec![],
            error_message: None,
            metadata: task.metadata,
        };

        assert_eq!(a2a.id, "t2");
        assert_eq!(a2a.status, TaskState::Working);
        assert!(a2a.messages.is_empty());
        assert!(a2a.artifacts.is_empty());
    }

    #[test]
    fn test_send_task_response_missing_task() {
        let resp = GrpcSendMessageResponse {
            task: None,
            message: None,
        };
        let resp_bytes = serde_json::to_vec(&resp).unwrap();
        let parsed: GrpcSendMessageResponse = serde_json::from_slice(&resp_bytes).unwrap();
        assert!(parsed.task.is_none());
    }

    #[test]
    fn test_cancel_task_response_to_a2a_task() {
        let grpc_task = make_grpc_task("t3", "canceled");
        let resp_bytes = serde_json::to_vec(&grpc_task).unwrap();
        let task: GrpcTask = serde_json::from_slice(&resp_bytes).unwrap();

        let a2a = A2ATask {
            id: task.id,
            session_id: task.session_id,
            status: parse_task_status(&task.status),
            messages: vec![],
            artifacts: vec![],
            error_message: None,
            metadata: task.metadata,
        };

        assert_eq!(a2a.id, "t3");
        assert_eq!(a2a.status, TaskState::Canceled);
    }

    #[test]
    fn test_send_task_request_with_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("session".into(), "sess-42".into());
        metadata.insert("trace".into(), "abc-123".into());

        let req = SendTaskRequest {
            id: "req-2".into(),
            session_id: String::new(),
            messages: vec![A2AMessage {
                role: "user".into(),
                parts: vec![A2APart {
                    part_type: A2APartType::Text,
                    text: Some("hello".into()),
                    mime_type: None,
                    file_uri: None,
                    data: None,
                }],
            }],
            metadata: metadata.clone(),
        };

        let grpc_req = GrpcSendMessageRequest {
            tenant: None,
            message: GrpcMessage {
                role: "user".into(),
                parts: req
                    .messages
                    .iter()
                    .flat_map(|m| {
                        m.parts
                            .iter()
                            .map(|p| GrpcPart {
                                part_type: format!("{:?}", p.part_type).to_lowercase(),
                                text: p.text.clone(),
                                mime_type: p.mime_type.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            },
            configuration: None,
            metadata,
        };

        let json = serde_json::to_string(&grpc_req).unwrap();
        assert!(json.contains("\"session\":\"sess-42\""));
        assert!(json.contains("\"trace\":\"abc-123\""));
    }

    #[test]
    fn test_send_task_request_empty_parts() {
        let req = SendTaskRequest {
            id: "req-3".into(),
            session_id: String::new(),
            messages: vec![A2AMessage {
                role: "user".into(),
                parts: vec![],
            }],
            metadata: std::collections::HashMap::new(),
        };

        let grpc_req = GrpcSendMessageRequest {
            tenant: None,
            message: GrpcMessage {
                role: "user".into(),
                parts: req
                    .messages
                    .iter()
                    .flat_map(|m| {
                        m.parts
                            .iter()
                            .map(|p| GrpcPart {
                                part_type: format!("{:?}", p.part_type).to_lowercase(),
                                text: p.text.clone(),
                                mime_type: p.mime_type.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            },
            configuration: None,
            metadata: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&grpc_req).unwrap();
        assert!(json.contains("\"parts\":[]"));
    }

    use crate::core::nt_core_util::A2A_DEFAULT_PORT;
    use crate::neotrix::nt_agent_protocol::a2a::{
        A2AArtifact, A2AMessage, A2APart, A2APartType, A2ATask, SendTaskRequest, TaskState,
    };
}
