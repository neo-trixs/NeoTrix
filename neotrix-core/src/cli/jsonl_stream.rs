use std::io::{self, Write};

use serde::Serialize;

/// JSON Lines event types for structured streaming output.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum JsonlEvent {
    #[serde(rename = "start")]
    Start {
        timestamp: String,
        data: StartData,
    },
    #[serde(rename = "message")]
    Message {
        timestamp: String,
        data: MessageData,
    },
    #[serde(rename = "tool_call")]
    ToolCall {
        timestamp: String,
        data: ToolCallData,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        timestamp: String,
        data: ToolResultData,
    },
    #[serde(rename = "error")]
    Error {
        timestamp: String,
        data: ErrorData,
    },
    #[serde(rename = "finish")]
    Finish {
        timestamp: String,
        data: FinishData,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct StartData {
    pub prompt: String,
    pub prompt_tokens: Option<u32>,
    pub model: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageData {
    pub role: String,
    pub content: String,
    pub tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCallData {
    pub name: String,
    pub arguments: serde_json::Value,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResultData {
    pub name: String,
    pub result: serde_json::Value,
    pub id: Option<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorData {
    pub message: String,
    pub code: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinishData {
    pub output: String,
    pub tokens_used: u32,
    pub duration_ms: u64,
    pub exit_code: i32,
}

fn fmt_timestamp() -> String {
    use chrono::Utc;
    Utc::now().to_rfc3339()
}

/// Writes JSON Lines (JSONL) events to an output destination.
pub struct JsonlWriter {
    writer: Box<dyn Write + Send>,
}

impl JsonlWriter {
    /// Create a new writer that writes to stdout.
    pub fn new() -> Self {
        Self {
            writer: Box::new(io::LineWriter::new(io::stdout())),
        }
    }

    /// Create a writer that writes to a custom output.
    pub fn new_with_writer(writer: Box<dyn Write + Send>) -> Self {
        Self { writer }
    }

    /// Serialize and write a single JSONL event.
    pub fn write_event(&mut self, event: &JsonlEvent) -> io::Result<()> {
        let line = serde_json::to_string(event)?;
        writeln!(self.writer, "{}", line)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Emit a `start` event at the beginning of execution.
    pub fn emit_start(
        &mut self,
        prompt: &str,
        prompt_tokens: Option<u32>,
        model: Option<&str>,
        mode: Option<&str>,
    ) {
        let _ = self.write_event(&JsonlEvent::Start {
            timestamp: fmt_timestamp(),
            data: StartData {
                prompt: prompt.to_string(),
                prompt_tokens,
                model: model.map(|s| s.to_string()),
                mode: mode.map(|s| s.to_string()),
            },
        });
    }

    /// Emit a `message` event for an assistant or user turn.
    pub fn emit_message(&mut self, role: &str, content: &str, tokens: Option<u32>) {
        let _ = self.write_event(&JsonlEvent::Message {
            timestamp: fmt_timestamp(),
            data: MessageData {
                role: role.to_string(),
                content: content.to_string(),
                tokens,
            },
        });
    }

    /// Emit a `tool_call` event for tool invocations.
    pub fn emit_tool_call(&mut self, name: &str, arguments: serde_json::Value, id: Option<&str>) {
        let _ = self.write_event(&JsonlEvent::ToolCall {
            timestamp: fmt_timestamp(),
            data: ToolCallData {
                name: name.to_string(),
                arguments,
                id: id.map(|s| s.to_string()),
            },
        });
    }

    /// Emit a `tool_result` event for tool results.
    pub fn emit_tool_result(&mut self, name: &str, result: serde_json::Value, id: Option<&str>, success: bool) {
        let _ = self.write_event(&JsonlEvent::ToolResult {
            timestamp: fmt_timestamp(),
            data: ToolResultData {
                name: name.to_string(),
                result,
                id: id.map(|s| s.to_string()),
                success,
            },
        });
    }

    /// Emit an `error` event.
    pub fn emit_error(&mut self, message: &str, code: Option<&str>, recoverable: bool) {
        let _ = self.write_event(&JsonlEvent::Error {
            timestamp: fmt_timestamp(),
            data: ErrorData {
                message: message.to_string(),
                code: code.map(|s| s.to_string()),
                recoverable,
            },
        });
    }

    /// Emit a `finish` event at the end of execution.
    pub fn emit_finish(
        &mut self,
        output: &str,
        tokens_used: u32,
        duration_ms: u64,
        exit_code: i32,
    ) {
        let _ = self.write_event(&JsonlEvent::Finish {
            timestamp: fmt_timestamp(),
            data: FinishData {
                output: output.to_string(),
                tokens_used,
                duration_ms,
                exit_code,
            },
        });
    }
}

impl Default for JsonlWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_start() {
        let event = JsonlEvent::Start {
            timestamp: "2026-06-01T00:00:00Z".into(),
            data: StartData {
                prompt: "hello".into(),
                prompt_tokens: Some(10),
                model: Some("gpt-4".into()),
                mode: None,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"start\""));
        assert!(json.contains("\"prompt\":\"hello\""));
    }

    #[test]
    fn test_serialize_message() {
        let event = JsonlEvent::Message {
            timestamp: "2026-06-01T00:00:00Z".into(),
            data: MessageData {
                role: "assistant".into(),
                content: "Hello world".into(),
                tokens: Some(50),
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"message\""));
        assert!(json.contains("\"role\":\"assistant\""));
    }

    #[test]
    fn test_serialize_tool_call() {
        let event = JsonlEvent::ToolCall {
            timestamp: "2026-06-01T00:00:00Z".into(),
            data: ToolCallData {
                name: "bash".into(),
                arguments: serde_json::json!({"command": "ls"}),
                id: Some("call_1".into()),
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"tool_call\""));
        assert!(json.contains("\"name\":\"bash\""));
    }

    #[test]
    fn test_serialize_tool_result() {
        let event = JsonlEvent::ToolResult {
            timestamp: "2026-06-01T00:00:00Z".into(),
            data: ToolResultData {
                name: "bash".into(),
                result: serde_json::json!({"stdout": "file.txt"}),
                id: Some("call_1".into()),
                success: true,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"tool_result\""));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_serialize_error() {
        let event = JsonlEvent::Error {
            timestamp: "2026-06-01T00:00:00Z".into(),
            data: ErrorData {
                message: "timeout".into(),
                code: Some("TIMEOUT".into()),
                recoverable: true,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"message\":\"timeout\""));
    }

    #[test]
    fn test_serialize_finish() {
        let event = JsonlEvent::Finish {
            timestamp: "2026-06-01T00:00:00Z".into(),
            data: FinishData {
                output: "done".into(),
                tokens_used: 150,
                duration_ms: 1200,
                exit_code: 0,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"finish\""));
        assert!(json.contains("\"exit_code\":0"));
    }

    #[test]
    fn test_writer_roundtrip() {
        let buf: Vec<u8> = Vec::new();
        let mut writer = JsonlWriter::new_with_writer(Box::new(std::io::BufWriter::new(buf)));
        writer.emit_start("test", Some(5), Some("gpt-4"), Some("debug"));
        writer.emit_finish("ok", 100, 1000, 0);
        drop(writer);
    }
}
