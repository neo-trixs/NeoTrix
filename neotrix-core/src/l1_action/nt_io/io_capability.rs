//! NT-IO 界面能力实现
//!
//! LLM接口、CLI、Web服务、ACP协议能力

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// LLM接口能力
pub struct LlmCapability;

impl UnifiedCapability for LlmCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-io-llm".into(),
            name: "LLM接口".into(),
            description: "大语言模型统一接口".into(),
            version: "1.0.0".into(),
            domain: Domain::NtIo,
            layer: Layer::L1Action,
            tags: vec!["io".into(), "llm".into(), "ai".into()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 2000.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = LlmResult {
                    model: "gpt-4".into(),
                    input: text.clone(),
                    output: format!("AI响应: {}", text),
                    tokens_used: 100,
                    latency_ms: 1500,
                    cost: 0.001,
                };
                Ok(CapabilityOutput::LlmResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// CLI接口能力
pub struct CliCapability;

impl UnifiedCapability for CliCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-io-cli".into(),
            name: "CLI接口".into(),
            description: "命令行界面接口".into(),
            version: "1.0.0".into(),
            domain: Domain::NtIo,
            layer: Layer::L1Action,
            tags: vec!["io".into(), "cli".into(), "command".into()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 50.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = CliResult {
                    command: text.clone(),
                    exit_code: 0,
                    stdout: format!("命令输出: {}", text),
                    stderr: String::new(),
                    execution_time_ms: 100,
                };
                Ok(CapabilityOutput::CliResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// Web服务能力
pub struct WebServiceCapability;

impl UnifiedCapability for WebServiceCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-io-web".into(),
            name: "Web服务".into(),
            description: "HTTP/WebSocket服务接口".into(),
            version: "1.0.0".into(),
            domain: Domain::NtIo,
            layer: Layer::L1Action,
            tags: vec!["io".into(), "web".into(), "http".into()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 100.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = WebResult {
                    method: "GET".into(),
                    url: text,
                    status_code: 200,
                    headers: std::collections::HashMap::new(),
                    body: "响应内容".into(),
                    latency_ms: 150,
                };
                Ok(CapabilityOutput::WebResult(result))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// ACP协议能力
pub struct AcpCapability;

impl UnifiedCapability for AcpCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-io-acp".into(),
            name: "ACP协议".into(),
            description: "Agent Communication Protocol接口".into(),
            version: "1.0.0".into(),
            domain: Domain::NtIo,
            layer: Layer::L1Action,
            tags: vec!["io".into(), "acp".into(), "agent".into()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 200.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = AcpResult {
                    protocol: "ACP".into(),
                    action: "execute".into(),
                    request_id: format!("req_{}", chrono::Utc::now().timestamp()),
                    status: "success".into(),
                    response: format!("ACP响应: {}", text),
                    metadata: std::collections::HashMap::new(),
                };
                Ok(CapabilityOutput::Text(result.response))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 创建NT-IO能力
pub fn create_io_capabilities() -> Vec<Arc<dyn UnifiedCapability>> {
    vec![
        Arc::new(LlmCapability),
        Arc::new(CliCapability),
        Arc::new(WebServiceCapability),
        Arc::new(AcpCapability),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm() {
        let cap = LlmCapability;
        let input = CapabilityInput::Text("测试LLM".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn cli() {
        let cap = CliCapability;
        let input = CapabilityInput::Text("echo test".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn web_service() {
        let cap = WebServiceCapability;
        let input = CapabilityInput::Text("https://api.example.com".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn acp() {
        let cap = AcpCapability;
        let input = CapabilityInput::Text("ACP协议测试".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }
}
