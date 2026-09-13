//! 能力检测模块 — 自动探测模型能力和健康状态

use std::collections::HashMap;

use super::traits::{ModelCapabilities, ModelHealth, TaskType};

/// 能力检测器
pub struct CapabilityDetector {
    /// 已知模型的能力缓存
    cache: HashMap<String, ModelCapabilities>,
}

impl CapabilityDetector {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        Self::register_known_models(&mut cache);
        Self { cache }
    }

    /// 注册已知模型的能力
    fn register_known_models(cache: &mut HashMap<String, ModelCapabilities>) {
        // OpenAI
        cache.insert(
            "openai/gpt-4o".to_string(),
            ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                supports_thinking: false,
                supports_embedding: false,
                max_context_tokens: 128_000,
                max_output_tokens: 16_384,
                languages: vec!["en".to_string(), "zh".to_string()],
                ..Default::default()
            },
        );
        cache.insert(
            "openai/gpt-4o-mini".to_string(),
            ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                max_context_tokens: 128_000,
                max_output_tokens: 16_384,
                ..Default::default()
            },
        );
        cache.insert(
            "openai/text-embedding-3-large".to_string(),
            ModelCapabilities {
                task_types: vec![TaskType::Embedding],
                supports_embedding: true,
                max_context_tokens: 8_191,
                max_output_tokens: 0,
                embedding_dim: Some(3072),
                ..Default::default()
            },
        );

        // Anthropic
        cache.insert(
            "anthropic/claude-sonnet-4-20250514".to_string(),
            ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                supports_thinking: true,
                max_context_tokens: 200_000,
                max_output_tokens: 16_384,
                languages: vec!["en".to_string(), "zh".to_string(), "ja".to_string()],
                ..Default::default()
            },
        );

        // Gemini
        cache.insert(
            "gemini/gemini-1.5-pro".to_string(),
            ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: true,
                supports_structured_output: true,
                supports_vision: true,
                max_context_tokens: 1_000_000,
                max_output_tokens: 8_192,
                languages: vec!["en".to_string(), "zh".to_string()],
                ..Default::default()
            },
        );

        // Ollama (local models)
        cache.insert(
            "ollama/llama3".to_string(),
            ModelCapabilities {
                task_types: vec![TaskType::Chat, TaskType::Completion],
                supports_streaming: true,
                supports_tools: false,
                supports_vision: false,
                max_context_tokens: 8_192,
                max_output_tokens: 4_096,
                ..Default::default()
            },
        );
    }

    /// 查询模型能力
    pub fn lookup(&self, full_id: &str) -> Option<&ModelCapabilities> {
        self.cache.get(full_id)
    }

    /// 手动注册模型能力
    pub fn register(&mut self, full_id: String, capabilities: ModelCapabilities) {
        self.cache.insert(full_id, capabilities);
    }

    /// 查找所有支持特定任务的模型
    pub fn find_by_task(&self, task: TaskType) -> Vec<(&str, &ModelCapabilities)> {
        self.cache
            .iter()
            .filter(|(_, caps)| caps.task_types.contains(&task))
            .map(|(id, caps)| (id.as_str(), caps))
            .collect()
    }

    /// 查找支持嵌入的模型
    pub fn embedding_models(&self) -> Vec<(&str, &ModelCapabilities)> {
        self.find_by_task(TaskType::Embedding)
    }

    /// 查找支持视觉的模型
    pub fn vision_models(&self) -> Vec<(&str, &ModelCapabilities)> {
        self.cache
            .iter()
            .filter(|(_, caps)| caps.supports_vision)
            .map(|(id, caps)| (id.as_str(), caps))
            .collect()
    }

    /// 查找支持工具调用的模型
    pub fn tool_models(&self) -> Vec<(&str, &ModelCapabilities)> {
        self.cache
            .iter()
            .filter(|(_, caps)| caps.supports_tools)
            .map(|(id, caps)| (id.as_str(), caps))
            .collect()
    }
}

impl Default for CapabilityDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 健康检查器
pub struct HealthChecker {
    /// 最近检查结果
    results: HashMap<String, ModelHealth>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// 记录健康检查结果
    pub fn record(&mut self, model_id: &str, health: ModelHealth) {
        self.results.insert(model_id.to_string(), health);
    }

    /// 获取模型健康状态
    pub fn get(&self, model_id: &str) -> Option<&ModelHealth> {
        self.results.get(model_id)
    }

    /// 获取所有模型健康状态
    pub fn all(&self) -> &HashMap<String, ModelHealth> {
        &self.results
    }

    /// 查找不健康的模型
    pub fn unhealthy_models(&self) -> Vec<(&str, &ModelHealth)> {
        self.results
            .iter()
            .filter(|(_, h)| !h.available || h.circuit_breaker_open)
            .map(|(id, h)| (id.as_str(), h))
            .collect()
    }

    /// 清理过期记录
    pub fn cleanup(&mut self, max_age_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.results.retain(|_, h| {
            if let Some(last) = h.last_success {
                now - last < max_age_secs
            } else {
                true
            }
        });
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_detector_lookup() {
        let detector = CapabilityDetector::new();
        let caps = detector.lookup("openai/gpt-4o");
        assert!(caps.is_some());
        let caps = caps.unwrap();
        assert!(caps.supports_vision);
        assert!(caps.supports_tools);
        assert_eq!(caps.max_context_tokens, 128_000);
    }

    #[test]
    fn test_find_by_task() {
        let detector = CapabilityDetector::new();
        let embedding_models = detector.find_by_task(TaskType::Embedding);
        assert!(!embedding_models.is_empty());
        assert!(embedding_models.iter().any(|(id, _)| id.contains("embedding")));
    }

    #[test]
    fn test_health_checker() {
        let mut checker = HealthChecker::new();
        checker.record("test", ModelHealth::default());
        assert!(checker.get("test").is_some());
        assert!(checker.unhealthy_models().is_empty());
    }
}
