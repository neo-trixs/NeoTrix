use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    pub model_name: String,
    pub provider: String,
    pub context_window: usize,
    pub supports_vision: bool,
    pub supports_audio: bool,
    pub fine_tuning_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSkill {
    pub skill_name: String,
    pub description: String,
}

pub struct ModelSkillRegistry {
    models: HashMap<String, ModelCapability>,
}

impl ModelSkillRegistry {
    pub fn new() -> Self {
        let mut models = HashMap::new();
        for (name, provider, ctx, vision, audio, ft) in BUILTIN_MODELS {
            models.insert(
                name.to_string(),
                ModelCapability {
                    model_name: name.to_string(),
                    provider: provider.to_string(),
                    context_window: *ctx,
                    supports_vision: *vision,
                    supports_audio: *audio,
                    fine_tuning_methods: ft.iter().map(|s| s.to_string()).collect(),
                },
            );
        }
        Self { models }
    }

    pub fn query_model(&self, model_name: &str) -> Option<&ModelCapability> {
        self.models.get(model_name)
    }

    pub fn list_models(&self) -> Vec<&ModelCapability> {
        self.models.values().collect()
    }

    pub fn register_model(&mut self, capability: ModelCapability) {
        let name = capability.model_name.clone();
        self.models.insert(name, capability);
    }

    pub fn models_by_provider(&self, provider: &str) -> Vec<&ModelCapability> {
        self.models.values().filter(|m| m.provider == provider).collect()
    }

    pub fn find_by_capability(&self, supports_vision: bool, min_context: usize) -> Vec<String> {
        self.models.values()
            .filter(|m| m.supports_vision == supports_vision && m.context_window >= min_context)
            .map(|m| m.model_name.clone())
            .collect()
    }
}

impl Default for ModelSkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

const BUILTIN_MODELS: &[(&str, &str, usize, bool, bool, &[&str])] = &[
    ("gemma-2-2b-it", "google", 8192, false, false, &["sft", "dpo"]),
    ("gemma-2-9b-it", "google", 8192, false, false, &["sft", "dpo", "rlhf"]),
    ("gemma-2-27b-it", "google", 8192, false, false, &["sft", "dpo", "rlhf"]),
    ("gemma-3-12b-it", "google", 32768, true, false, &["sft", "dpo", "rlhf"]),
    ("gemma-3-27b-it", "google", 32768, true, false, &["sft", "dpo", "rlhf"]),
    ("gemma-2-2b", "google", 8192, false, false, &["sft", "dpo"]),
];

pub static REGISTRY: LazyLock<ModelSkillRegistry> = LazyLock::new(ModelSkillRegistry::new);

static SKILLS: LazyLock<Vec<ModelSkill>> = LazyLock::new(|| vec![
    ModelSkill {
        skill_name: "gemma-dev".to_string(),
        description: "Guidance for building applications with Gemma models, including model selection and integration patterns".to_string(),
    },
    ModelSkill {
        skill_name: "gemma-trainer".to_string(),
        description: "Fine-tuning Gemma models with SFT, DPO, RLHF, and reward modeling on local hardware".to_string(),
    },
]);

pub fn query_model(model_name: &str) -> Option<&'static ModelCapability> {
    REGISTRY.query_model(model_name)
}

pub fn list_skills() -> &'static [ModelSkill] {
    &SKILLS[..]
}

pub fn find_models_by_capability(supports_vision: bool) -> Vec<String> {
    REGISTRY.find_by_capability(supports_vision, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_known_model() {
        let result = query_model("gemma-2-2b-it");
        assert!(result.is_some());
        let cap = result.unwrap();
        assert_eq!(cap.provider, "google");
        assert!(!cap.supports_vision);
    }

    #[test]
    fn test_query_unknown_model() {
        let result = query_model("nonexistent-model-v1");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_skills() {
        let skills = list_skills();
        assert_eq!(skills.len(), 2);
        assert!(skills.iter().any(|s| s.skill_name == "gemma-dev"));
        assert!(skills.iter().any(|s| s.skill_name == "gemma-trainer"));
    }

    #[test]
    fn test_find_vision_models() {
        let names = find_models_by_capability(true);
        assert!(!names.is_empty());
        assert!(names.iter().all(|name| name.contains("gemma-3")));
    }

    #[test]
    fn test_registry_custom_model() {
        let mut registry = ModelSkillRegistry::new();
        let custom = ModelCapability {
            model_name: "custom-model-v1".to_string(),
            provider: "custom".to_string(),
            context_window: 4096,
            supports_vision: true,
            supports_audio: true,
            fine_tuning_methods: vec!["sft".to_string()],
        };
        registry.register_model(custom);
        let result = registry.query_model("custom-model-v1");
        assert!(result.is_some());
        assert!(result.unwrap().supports_audio);
    }

    #[test]
    fn test_registry_from_static() {
        let models = REGISTRY.list_models();
        assert_eq!(models.len(), 6);
    }
}
