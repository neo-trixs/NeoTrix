//! NT-WORLD NLP 统一能力接口实现

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// NLP能力实现
pub struct _NlpCapability {
    meta: CapabilityMeta,
    health: CapabilityHealth,
}

impl _NlpCapability {
    pub fn new() -> Self {
        Self {
            meta: CapabilityMeta {
                id: "nt-world-nlp".into(),
                name: "NT-WORLD NLP".into(),
                layer: Layer::L2Perception,
                domain: Domain::NtWorld,
                version: "0.1.0".into(),
                description: "自然语言处理能力".into(),
                tags: vec!["nlp".into(), "regex".into(), "similarity".into()],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
                cost_weight: 0.0,
                priority: 1.0,
            },
            health: CapabilityHealth {
                state: CapabilityState::Ready,
                success_rate: 1.0,
                avg_latency_ms: 0.0,
                last_called: None,
                call_count: 0,
            },
        }
    }
}

impl UnifiedCapability for _NlpCapability {
    fn meta(&self) -> CapabilityMeta {
        self.meta.clone()
    }

    fn health(&self) -> CapabilityHealth {
        self.health.clone()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                if text.is_empty() {
                    return Err(CapabilityError::UnsupportedInput("文本不能为空".into()));
                }
                Ok(CapabilityOutput::Text(text))
            }
            CapabilityInput::Nlp(nlp) => {
                if nlp.text.is_empty() {
                    return Err(CapabilityError::UnsupportedInput("NLP文本不能为空".into()));
                }
                Ok(CapabilityOutput::Text(nlp.text))
            }
            _ => Err(CapabilityError::UnsupportedInput("不支持的输入类型".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_) | CapabilityInput::Nlp(_))
    }
}

pub fn create_nlp_capability() -> Arc<dyn UnifiedCapability> {
    Arc::new(_NlpCapability::new())
}
