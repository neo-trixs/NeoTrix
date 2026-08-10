//! CoT Generator — 自然语言思维链生成器 (Phase 2.1)
//!
//! 将 Kernel 的结构化推理状态 (ReasoningTrace + 向量状态) 转为自然语言 CoT，
//! 通过 NT-IO LLM Provider 调用外部模型生成，支持 thinking_budget 扩展思考。
//! 这是 Kernel 与 LLM 解耦的关键桥梁：Kernel 做"推理骨架"，CoTGenerator 做"语言肉"。

use crate::core::nt_core_reasoning::ReasoningTrace;
use crate::neotrix::l1_body_impl::nt_io_provider::types::{LlmProvider, LlmRequest, LlmError, Message, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

type Vector = Vec<f64>;

/// CoT 生成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTConfig {
    /// 使用的模型名称
    pub model: String,
    /// 思考预算 (tokens)，Some(0)=禁用，Some(n)=预算，None=模型默认
    pub thinking_budget: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
    /// 最大输出 tokens
    pub max_tokens: u32,
    /// 是否启用结构化输出 (JSON Schema)
    pub structured_output: bool,
    /// 系统提示词模板
    pub system_prompt: String,
}

impl Default for CoTConfig {
    fn default() -> Self {
        Self {
            model: "deepseek-v4-flash-free".to_string(),
            thinking_budget: Some(2048),
            temperature: Some(0.3),
            max_tokens: 8192,
            structured_output: true,
            system_prompt: r#"You are NeoTrix's CoT Generator. Your role is to produce clear, step-by-step Chain-of-Thought reasoning based on the provided kernel state and task.

Output format (JSON):
{
  "reasoning_steps": [
    {"step": 1, "description": "...", "confidence": 0.9},
    {"step": 2, "description": "...", "confidence": 0.85}
  ],
  "final_answer": "...",
  "overall_confidence": 0.88
}

Guidelines:
- Each step should be logically connected to the previous
- Confidence reflects certainty in that step
- Final answer should directly address the task
- Overall confidence is the geometric mean of step confidences"#.to_string(),
        }
    }
}

impl CoTConfig {
    /// 从环境变量加载配置（NEOTRIX_COT_* 前缀），未设置时回退默认值。
    /// 支持: NEOTRIX_COT_MODEL, NEOTRIX_COT_THINKING_BUDGET, NEOTRIX_COT_TEMPERATURE,
    ///       NEOTRIX_COT_MAX_TOKENS, NEOTRIX_COT_STRUCTURED
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("NEOTRIX_COT_MODEL") {
            if !v.is_empty() {
                cfg.model = v;
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_COT_THINKING_BUDGET") {
            if let Ok(n) = v.parse::<u32>() {
                cfg.thinking_budget = Some(n);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_COT_TEMPERATURE") {
            if let Ok(f) = v.parse::<f32>() {
                cfg.temperature = Some(f.clamp(0.0, 2.0));
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_COT_MAX_TOKENS") {
            if let Ok(n) = v.parse::<u32>() {
                cfg.max_tokens = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_COT_STRUCTURED") {
            cfg.structured_output = v == "1" || v.eq_ignore_ascii_case("true");
        }
        cfg
    }
}

/// CoT 生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTOutput {
    pub reasoning_steps: Vec<CoTStep>,
    pub final_answer: String,
    pub overall_confidence: f64,
    pub raw_response: String,
}

/// 单步 CoT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTStep {
    pub step: usize,
    pub description: String,
    pub confidence: f64,
}

/// CoT Generator trait — 统一接口，支持多种实现
#[async_trait]
pub trait CoTGenerator: Send + Sync {
    /// 生成 CoT：输入任务 + Kernel trace，输出结构化 CoT
    async fn generate_cot(
        &self,
        task: &str,
        kernel_trace: &ReasoningTrace,
        context: Option<HashMap<String, Vector>>,
    ) -> Result<CoTOutput, CoTError>;

    /// 批量生成 (用于 self-consistency 多路径采样)
    async fn generate_cot_batch(
        &self,
        task: &str,
        kernel_trace: &ReasoningTrace,
        n_samples: usize,
    ) -> Result<Vec<CoTOutput>, CoTError> {
        let mut results = Vec::with_capacity(n_samples);
        for _ in 0..n_samples {
            results.push(self.generate_cot(task, kernel_trace, None).await?);
        }
        Ok(results)
    }
}

/// CoT 生成错误
#[derive(Debug, thiserror::Error)]
pub enum CoTError {
    #[error("LLM provider error: {0}")]
    Provider(#[from] LlmError),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Invalid response format: {0}")]
    InvalidFormat(String),
    #[error("Empty response")]
    EmptyResponse,
}

/// 默认实现：基于 NT-IO LLM Provider
pub struct DefaultCoTGenerator {
    provider: Arc<dyn LlmProvider>,
    config: CoTConfig,
}

impl DefaultCoTGenerator {
    pub fn new(provider: Arc<dyn LlmProvider>, config: CoTConfig) -> Self {
        Self { provider, config }
    }

    /// 构建 CoT 生成提示词
    fn build_prompt(
        &self,
        task: &str,
        kernel_trace: &ReasoningTrace,
        context: Option<HashMap<String, Vector>>,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str(&self.config.system_prompt);
        prompt.push_str("\n\n---\n\n");
        prompt.push_str(&format!("Task: {}\n\n", task));
        
        // Kernel 状态摘要
        prompt.push_str("Kernel State:\n");
        prompt.push_str(&format!("- Method: {:?}\n", kernel_trace.method));
        prompt.push_str(&format!("- Stage: {}\n", kernel_trace.stage));
        prompt.push_str(&format!("- Hexagram: {:?}\n", kernel_trace.hexagram));
        prompt.push_str(&format!("- Convergence: {:.3}\n", kernel_trace.convergence));
        prompt.push_str(&format!("- Intermediate states: {}\n", kernel_trace.intermediate_states.len()));
        
        // 步骤详情（如果有）
        if !kernel_trace.steps.is_empty() {
            prompt.push_str("\nReasoning Steps:\n");
            for step in &kernel_trace.steps {
                prompt.push_str(&format!("- Step {}: {} (confidence: {:.2})\n", step.step_index, step.description, step.reward.unwrap_or(0.0)));
            }
        }
        
        // Context 向量摘要
        if let Some(ctx) = context {
            prompt.push_str(&format!("\nContext vectors: {} entries\n", ctx.len()));
        }
        
        prompt.push_str("\nGenerate CoT now:\n");
        prompt
    }

    /// 解析 LLM 响应为 CoTOutput
    fn parse_response(&self, response: &str) -> Result<CoTOutput, CoTError> {
        // 尝试解析 JSON
        if let Ok(parsed) = serde_json::from_str::<CoTOutput>(response) {
            return Ok(parsed);
        }
        
        // 如果不是标准 JSON，尝试提取 JSON 部分
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str::<CoTOutput>(json_str) {
                    return Ok(parsed);
                }
            }
        }
        
        // 兜底：将整个响应作为 final_answer
        Ok(CoTOutput {
            reasoning_steps: vec![],
            final_answer: response.to_string(),
            overall_confidence: 0.5,
            raw_response: response.to_string(),
        })
    }
}

#[async_trait]
impl CoTGenerator for DefaultCoTGenerator {
    async fn generate_cot(
        &self,
        task: &str,
        kernel_trace: &ReasoningTrace,
        context: Option<HashMap<String, Vector>>,
    ) -> Result<CoTOutput, CoTError> {
        let prompt = self.build_prompt(task, kernel_trace, context);
        
        let request = LlmRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message::new(Role::System, &self.config.system_prompt),
                Message::new(Role::User, &prompt),
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            tools: vec![],
            image_data: None,
            thinking_budget: self.config.thinking_budget,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: if self.config.structured_output {
                Some(crate::neotrix::l1_body_impl::nt_io_provider::types::StructuredOutputConfig::JsonObject)
            } else {
                None
            },
        };
        
        let response = self.provider.complete(&request).await?;
        let text = response.content;
        if text.is_empty() {
            return Err(CoTError::EmptyResponse);
        }
        self.parse_response(&text)
    }
}

/// 便利构造函数
pub fn create_cot_generator(provider: Arc<dyn LlmProvider>) -> DefaultCoTGenerator {
    DefaultCoTGenerator::new(provider, CoTConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cot_config_default() {
        let config = CoTConfig::default();
        assert_eq!(config.thinking_budget, Some(2048));
        assert!(config.structured_output);
    }
    
    #[test]
    fn test_cot_output_serialization() {
        let output = CoTOutput {
            reasoning_steps: vec![
                CoTStep { step: 1, description: "test".to_string(), confidence: 0.9 },
            ],
            final_answer: "answer".to_string(),
            overall_confidence: 0.9,
            raw_response: "raw".to_string(),
        };
        let json = serde_json::to_string(&output).unwrap();
        let parsed: CoTOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.final_answer, "answer");
    }
}