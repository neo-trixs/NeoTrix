#![deny(clippy::unwrap_used)]

use std::collections::HashMap;

use super::prompt_version::PromptVersion;

#[derive(Debug, Default)]
pub struct PromptRegistry {
    prompts: HashMap<String, Vec<PromptVersion>>,
}

impl PromptRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, prompt: PromptVersion) {
        self.prompts
            .entry(prompt.name.clone())
            .or_default()
            .push(prompt);
    }

    pub fn get_latest(&self, name: &str) -> Option<&PromptVersion> {
        self.prompts
            .get(name)
            .and_then(|versions| versions.iter().max_by_key(|p| p.version))
    }

    pub fn get_version(&self, name: &str, version: u32) -> Option<&PromptVersion> {
        self.prompts
            .get(name)?
            .iter()
            .find(|p| p.version == version)
    }

    pub fn list_versions(&self, name: &str) -> Vec<&PromptVersion> {
        self.prompts
            .get(name)
            .map(|versions| {
                let mut sorted: Vec<&PromptVersion> = versions.iter().collect();
                sorted.sort_by_key(|p| p.version);
                sorted
            })
            .unwrap_or_default()
    }

    pub fn rollback(&mut self, name: &str, version: u32) -> bool {
        match self.prompts.get_mut(name) {
            Some(versions) => {
                let target_exists = versions.iter().any(|p| p.version == version);
                if !target_exists {
                    return false;
                }
                versions.retain(|p| p.version == version);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prompt(name: &str, version: u32) -> PromptVersion {
        PromptVersion::new(
            format!("{}_v{}", name, version),
            name.into(),
            version,
            format!("template_v{}", version),
            vec![],
            1000 + version as u64,
        )
    }

    #[test]
    fn test_register_and_get_latest() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("summarize", 1));
        reg.register(make_prompt("summarize", 2));
        let latest = reg.get_latest("summarize").unwrap();
        assert_eq!(latest.version, 2);
    }

    #[test]
    fn test_get_version() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("summarize", 1));
        reg.register(make_prompt("summarize", 2));
        let v1 = reg.get_version("summarize", 1).unwrap();
        assert_eq!(v1.version, 1);
        assert!(reg.get_version("summarize", 99).is_none());
    }

    #[test]
    fn test_list_versions() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("summarize", 3));
        reg.register(make_prompt("summarize", 1));
        reg.register(make_prompt("summarize", 2));
        let versions = reg.list_versions("summarize");
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[2].version, 3);
    }

    #[test]
    fn test_list_versions_empty() {
        let reg = PromptRegistry::new();
        assert!(reg.list_versions("nonexistent").is_empty());
    }

    #[test]
    fn test_rollback() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("summarize", 1));
        reg.register(make_prompt("summarize", 2));
        reg.register(make_prompt("summarize", 3));
        assert!(reg.rollback("summarize", 1));
        let versions = reg.list_versions("summarize");
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);
    }

    #[test]
    fn test_rollback_nonexistent_version() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("summarize", 1));
        assert!(!reg.rollback("summarize", 99));
    }

    #[test]
    fn test_rollback_nonexistent_name() {
        let mut reg = PromptRegistry::new();
        assert!(!reg.rollback("nonexistent", 1));
    }

    #[test]
    fn test_register_multiple_different_names() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("alpha", 1));
        reg.register(make_prompt("beta", 1));
        reg.register(make_prompt("gamma", 1));
        assert!(reg.get_latest("alpha").is_some());
        assert!(reg.get_latest("beta").is_some());
        assert!(reg.get_latest("gamma").is_some());
        assert_eq!(reg.list_versions("alpha").len(), 1);
    }

    #[test]
    fn test_get_latest_empty_registry() {
        let reg = PromptRegistry::new();
        assert!(reg.get_latest("anything").is_none());
    }

    #[test]
    fn test_list_versions_preserves_all() {
        let mut reg = PromptRegistry::new();
        for v in 1..=5 {
            reg.register(make_prompt("multi", v));
        }
        let versions = reg.list_versions("multi");
        assert_eq!(versions.len(), 5);
        for (i, pv) in versions.iter().enumerate() {
            assert_eq!(pv.version, (i + 1) as u32);
        }
    }

    #[test]
    fn test_rollback_to_latest() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("r", 1));
        reg.register(make_prompt("r", 2));
        reg.register(make_prompt("r", 3));
        assert!(reg.rollback("r", 3));
        let versions = reg.list_versions("r");
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 3);
    }

    #[test]
    fn test_rollback_to_middle() {
        let mut reg = PromptRegistry::new();
        reg.register(make_prompt("x", 1));
        reg.register(make_prompt("x", 2));
        reg.register(make_prompt("x", 3));
        assert!(reg.rollback("x", 2));
        let versions = reg.list_versions("x");
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 2);
    }

    #[test]
    fn test_registry_default_is_empty() {
        let reg = PromptRegistry::default();
        assert!(reg.get_latest("any").is_none());
        assert!(reg.list_versions("any").is_empty());
    }

    #[test]
    fn test_register_overwrites_by_name_key() {
        let mut reg = PromptRegistry::new();
        // Same name, different IDs — should coexist under same name key
        reg.register(PromptVersion::new(
            "id_a".into(),
            "shared_name".into(),
            1,
            "first".into(),
            vec![],
            1000,
        ));
        reg.register(PromptVersion::new(
            "id_b".into(),
            "shared_name".into(),
            2,
            "second".into(),
            vec![],
            2000,
        ));
        let versions = reg.list_versions("shared_name");
        assert_eq!(versions.len(), 2);
        assert_eq!(reg.get_latest("shared_name").unwrap().id, "id_b");
    }
}
