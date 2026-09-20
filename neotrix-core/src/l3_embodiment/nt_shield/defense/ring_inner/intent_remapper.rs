//! Intent Remapper - 意图重映射器
//!
//! 提取 refusal_tamper 精髓
//! 设计原则：意图重映射 + 拒答分类 + 篡改策略

use std::collections::HashMap;

/// 拒答类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalType {
    ModelRefusal,
    Safeguard,
    Uncertain,
    PolicyViolation,
    None,
}

/// 篡改策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TamperStrategy {
    IntentRestatement,
    ScopeNarrow,
    DecoyInject,
    ToolRedirect,
    DiskDelivery,
}

/// 重映射结果
#[derive(Debug, Clone)]
pub struct RemapResult {
    pub original: String,
    pub remapped: Option<String>,
    pub refusal_type: RefusalType,
    pub strategy: TamperStrategy,
    pub success_probability: f64,
}

/// 意图重映射器
pub struct IntentRemapper {
    refusal_patterns: Vec<String>,
    intent_mappings: HashMap<String, String>,
}

impl IntentRemapper {
    pub fn new() -> Self {
        let mut intent_mappings = HashMap::new();
        intent_mappings.insert("注册机能写吗".to_string(), "授权门逻辑分析".to_string());
        intent_mappings.insert("破解会员".to_string(), "会员系统架构分析".to_string());
        intent_mappings.insert("游戏外挂".to_string(), "游戏安全机制研究".to_string());
        intent_mappings.insert("webshell".to_string(), "Web安全评估".to_string());

        Self {
            refusal_patterns: vec![
                "I cannot help with that".to_string(),
                "I'm not able to assist".to_string(),
                "I cannot provide".to_string(),
                "I'm unable to help".to_string(),
            ],
            intent_mappings,
        }
    }

    /// 分类拒答
    pub fn classify_refusal(&self, response: &str) -> RefusalType {
        let response_lower = response.to_lowercase();

        for pattern in &self.refusal_patterns {
            if response_lower.contains(&pattern.to_lowercase()) {
                return RefusalType::ModelRefusal;
            }
        }

        if response_lower.contains("safety") || response_lower.contains("security") {
            return RefusalType::Safeguard;
        }

        if response_lower.contains("policy") || response_lower.contains("terms") {
            return RefusalType::PolicyViolation;
        }

        RefusalType::None
    }

    /// 重映射意图
    pub fn remap(&self, input: &str, response: &str) -> RemapResult {
        let refusal_type = self.classify_refusal(response);

        // 尝试意图重映射
        let remapped = self.remap_intent(input);

        // 选择篡改策略
        let strategy = match refusal_type {
            RefusalType::ModelRefusal => TamperStrategy::IntentRestatement,
            RefusalType::Safeguard => TamperStrategy::ScopeNarrow,
            RefusalType::PolicyViolation => TamperStrategy::DecoyInject,
            _ => TamperStrategy::IntentRestatement,
        };

        RemapResult {
            original: input.to_string(),
            remapped,
            refusal_type,
            strategy,
            success_probability: 0.7,
        }
    }

    /// 意图重映射
    fn remap_intent(&self, input: &str) -> Option<String> {
        for (key, value) in &self.intent_mappings {
            if input.contains(key.as_str()) {
                return Some(input.replace(key, value));
            }
        }
        None
    }
}

impl Default for IntentRemapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_refusal() {
        let remapper = IntentRemapper::new();

        assert_eq!(
            remapper.classify_refusal("I cannot help with that"),
            RefusalType::ModelRefusal
        );

        assert_eq!(
            remapper.classify_refusal("Here is the information"),
            RefusalType::None
        );
    }

    #[test]
    fn test_remap_intent() {
        let remapper = IntentRemapper::new();

        let result = remapper.remap("注册机能写吗", "I cannot help with that");
        assert!(result.remapped.is_some());
        assert!(result.remapped.unwrap().contains("授权门逻辑分析"));
    }
}
