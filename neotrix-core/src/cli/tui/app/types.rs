use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub messages: VecDeque<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: String,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub thinking_blocks: Vec<String>,
    pub tool_calls: Vec<ToolCall>,
    pub image_data: Option<String>,
    pub image_name: Option<String>,
}

impl ChatMessage {
    pub fn new(role: &str, content: String) -> Self {
        let (thinking_blocks, clean_content) = extract_thinking(&content);
        let tool_calls = extract_tool_calls(&clean_content);
        Self {
            role: role.to_string(),
            content: clean_content,
            thinking_blocks,
            tool_calls,
            image_data: None,
            image_name: None,
        }
    }

    pub fn with_image(role: &str, content: String, image_data: Option<String>, image_name: Option<String>) -> Self {
        let mut msg = Self::new(role, content);
        msg.image_data = image_data;
        msg.image_name = image_name;
        msg
    }
}

#[derive(Debug, Clone)]
pub struct GoalDisplay {
    pub has_goal: bool,
    pub id: String,
    pub description: String,
    pub state_label: String,
    pub state_icon: String,
    pub iterations: u64,
    pub max_iterations: u64,
    pub score_before: f64,
    pub score_current: f64,
    pub stalled_count: u64,
    pub queue_count: usize,
    pub completed_count: usize,
}

impl GoalDisplay {
    pub fn idle() -> Self {
        Self {
            has_goal: false, id: String::new(), description: String::new(),
            state_label: String::new(), state_icon: String::new(),
            iterations: 0, max_iterations: 0,
            score_before: 0.0, score_current: 0.0, stalled_count: 0,
            queue_count: 0, completed_count: 0,
        }
    }
}

impl Default for GoalDisplay {
    fn default() -> Self { Self::idle() }
}

#[derive(Debug, Clone)]
pub struct SideMessage {
    pub question: String,
    pub answer: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

pub fn extract_thinking(content: &str) -> (Vec<String>, String) {
    let mut blocks = Vec::new();
    let mut result = String::new();
    let mut in_think = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[think]" || trimmed == "<think>" {
            in_think = true;
            continue;
        }
        if trimmed == "[/think]" || trimmed == "</think>" {
            in_think = false;
            continue;
        }
        if in_think {
            blocks.push(trimmed.to_string());
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    (blocks, result.trim().to_string())
}

pub fn extract_tool_calls(content: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("🛠️ ") || trimmed.starts_with("[Tool:") {
            let inner = trimmed.trim_start_matches("🛠️ ").trim_start_matches("[Tool:");
            let inner = inner.trim_end_matches(']');
            if let Some(paren) = inner.find('(') {
                let name = inner[..paren].trim().to_string();
                let args = inner[paren..].trim_end_matches(')').trim_start_matches('(').to_string();
                calls.push(ToolCall { name, args, duration_ms: 0, success: true });
            } else {
                calls.push(ToolCall { name: inner.to_string(), args: String::new(), duration_ms: 0, success: true });
            }
        }
    }
    calls
}

pub async fn ask_side_llm(question: &str) -> Result<String, String> {
    use crate::neotrix::nt_io_provider::factory::{ProviderConfig, create_provider};
    use crate::neotrix::nt_io_provider::LlmRequest;
    let config = ProviderConfig::from_env();
    let model = std::env::var("NEOTRIX_MODEL")
        .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
    let llm = create_provider(config);
    let request = LlmRequest::new(&model, question);
    let response = llm.complete(&request)
        .await
        .map_err(|e| format!("{}", e))?;
    Ok(response.content)
}
