//! Provider abstraction models
//!
//! Core types for model-agnostic LLM provider abstraction.
//! R-P128: config-driven, not hardcoded.

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// Task type for cost-aware routing (R-P128: string-configured)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Chat,
    Completion,
    Embedding,
    CodeGeneration,
    Reasoning,
    Translation,
    Summarization,
    Classification,
    Other(String),
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskType::Chat => write!(f, "chat"),
            TaskType::Completion => write!(f, "completion"),
            TaskType::Embedding => write!(f, "embedding"),
            TaskType::CodeGeneration => write!(f, "code_generation"),
            TaskType::Reasoning => write!(f, "reasoning"),
            TaskType::Translation => write!(f, "translation"),
            TaskType::Summarization => write!(f, "summarization"),
            TaskType::Classification => write!(f, "classification"),
            TaskType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for TaskType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "chat" => Ok(TaskType::Chat),
            "completion" => Ok(TaskType::Completion),
            "embedding" => Ok(TaskType::Embedding),
            "code_generation" | "code" => Ok(TaskType::CodeGeneration),
            "reasoning" => Ok(TaskType::Reasoning),
            "translation" => Ok(TaskType::Translation),
            "summarization" | "summary" => Ok(TaskType::Summarization),
            "classification" | "classify" => Ok(TaskType::Classification),
            other => Ok(TaskType::Other(other.to_string())),
        }
    }
}

/// Model cost per 1K tokens (input/output split)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub input_per_1k: f64,
    pub output_per_1k: f64,
}

impl ModelCost {
    pub fn new(input_per_1k: f64, output_per_1k: f64) -> Self {
        Self { input_per_1k, output_per_1k }
    }

    /// Estimate cost for token counts
    pub fn estimate(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        (input_tokens as f64 / 1000.0) * self.input_per_1k
            + (output_tokens as f64 / 1000.0) * self.output_per_1k
    }
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_chat: bool,
    pub supports_completion: bool,
    pub supports_embedding: bool,
    pub supports_code: bool,
    pub supports_reasoning: bool,
    pub max_context_length: u32,
    pub max_output_length: u32,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            supports_chat: true,
            supports_completion: true,
            supports_embedding: false,
            supports_code: false,
            supports_reasoning: false,
            max_context_length: 4096,
            max_output_length: 2048,
        }
    }
}

/// Model information (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub cost: ModelCost,
    pub capabilities: ModelCapabilities,
    pub quality_score: f64,
}

impl ModelInfo {
    pub fn supports_task(&self, task: &TaskType) -> bool {
        match task {
            TaskType::Chat => self.capabilities.supports_chat,
            TaskType::Completion => self.capabilities.supports_completion,
            TaskType::Embedding => self.capabilities.supports_embedding,
            TaskType::CodeGeneration => self.capabilities.supports_code,
            TaskType::Reasoning => self.capabilities.supports_reasoning,
            TaskType::Translation | TaskType::Summarization | TaskType::Classification => {
                self.capabilities.supports_chat
            }
            TaskType::Other(_) => true,
        }
    }

    /// Compute cost score: lower is cheaper
    pub fn cost_score(&self) -> f64 {
        self.cost.input_per_1k + self.cost.output_per_1k
    }
}

/// LLM message for completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    ContentFilter,
    Other(String),
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
    pub usage: TokenUsage,
}

/// Routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub provider_name: String,
    pub model_id: String,
    pub estimated_cost_usd: f64,
    pub estimated_latency_ms: f64,
    pub quality_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_display_roundtrip() {
        let types = vec![
            TaskType::Chat,
            TaskType::Completion,
            TaskType::Embedding,
            TaskType::CodeGeneration,
            TaskType::Reasoning,
            TaskType::Translation,
            TaskType::Summarization,
            TaskType::Classification,
            TaskType::Other("custom".to_string()),
        ];

        for task in &types {
            let s = task.to_string();
            let parsed: TaskType = s.parse().unwrap();
            assert_eq!(*task, parsed);
        }
    }

    #[test]
    fn test_model_cost_estimate() {
        let cost = ModelCost::new(0.03, 0.06);
        let total = cost.estimate(1000, 500);
        assert!((total - 0.06).abs() < 1e-10);
    }

    #[test]
    fn test_model_supports_task() {
        let model = ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            cost: ModelCost::new(0.03, 0.06),
            capabilities: ModelCapabilities {
                supports_chat: true,
                supports_code: true,
                ..Default::default()
            },
            quality_score: 0.9,
        };

        assert!(model.supports_task(&TaskType::Chat));
        assert!(model.supports_task(&TaskType::CodeGeneration));
        assert!(!model.supports_task(&TaskType::Embedding));
    }

    #[test]
    fn test_cost_score() {
        let model = ModelInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            provider: "test".to_string(),
            cost: ModelCost::new(0.01, 0.02),
            capabilities: ModelCapabilities::default(),
            quality_score: 0.5,
        };
        assert!((model.cost_score() - 0.03).abs() < 1e-10);
    }
}
