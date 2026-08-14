//! nt_core_prompting — 提示管理
//!
//! 统一的提示模板、缓存和链管理
//! 节点: nt_core_prompting (L8)
//! Provides: prompt_management, template_chaining
//! Requires: peft_suite, tokenizer
//! Rune: Crimson, Indigo

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptConfig {
    /// 提示模板缓存大小
    pub cache_size: usize,
    /// 是否启用模板变量验证
    pub validate_variables: bool,
    /// 默认温度
    pub default_temperature: f32,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            cache_size: 128,
            validate_variables: true,
            default_temperature: 0.7,
        }
    }
}

/// 提示模板结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: Option<String>,
}

/// 提示管理器
pub struct PromptManager {
    config: PromptConfig,
    templates: HashMap<String, PromptTemplate>,
    cache: Vec<(String, String)>, // (name, filled_template)
    /// 预留: 提示/缓存统计元数据, 待观测通道接入后填充
    _metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl PromptManager {
    pub fn new(config: PromptConfig) -> Self {
        Self {
            config,
            templates: HashMap::new(),
            cache: Vec::new(),
            _metadata: HashMap::new(),
        }
    }

    pub fn add_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
        // 维护缓存大小
        if self.cache.len() >= self.config.cache_size {
            self.cache.remove(0);
        }
    }

    pub fn fill_template(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, NeoTrixError> {
        let template = self.templates.get(name).ok_or_else(|| {
            NeoTrixError::NotFound(format!("Prompt template not found: {}", name))
        })?;

        let mut result = template.template.clone();
        for (var_name, var_value) in variables {
            result = result.replace(&format!("{{{}}}", var_name), var_value);
        }

        if self.config.validate_variables {
            // 检查是否有未填充的变量
            let unreplaced: Vec<&str> = result
                .match_indices("{{")
                .map(|(i, _)| &result[i..])
                .collect();
            if !unreplaced.is_empty() {
                return Err(NeoTrixError::InvalidInput(format!(
                    "Unresolved prompt variables: {:?}",
                    unreplaced
                )));
            }
        }

        Ok(result)
    }

    pub fn get_config(&self) -> &PromptConfig {
        &self.config
    }
}

impl CapabilityNode for PromptManager {
    fn node_id(&self) -> &str {
        "nt_core_prompting"
    }
    fn provides(&self) -> Vec<String> {
        vec!["prompt_management".into(), "template_chaining".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["peft_suite".into(), "tokenizer".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Indigo]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for PromptManager {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut pm = PromptManager::new(PromptConfig::default());

            let template = PromptTemplate {
                name: "test".into(),
                template: "Hello {name}, you are {topic}".into(),
                variables: vec!["name".into(), "topic".into()],
                description: Some("测试模板".into()),
            };
            pm.add_template(template.clone());

            let variables = HashMap::from([
                ("name".into(), "World".into()),
                ("topic".into(), "AI".into()),
            ]);
            let filled = pm.fill_template("test", &variables)?;
            assert_eq!(filled, "Hello World, you are AI");

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_prompting_manager"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_prompt_manager_self_test() {
        let pm = PromptManager::new(PromptConfig::default());
        assert!(pm.self_test().is_ok());
    }
}
