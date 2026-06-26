use std::path::PathBuf;

use crate::neotrix::nt_agent_mod::plugin::manager::PluginManager;
use crate::neotrix::nt_agent_mod::plugin::progressive_disclosure::{
    DisclosureManifest, FullSkill, ProgressiveDisclosureLayer,
};
use crate::neotrix::nt_agent_mod::plugin::skill_executor::SkillExecutor;
use crate::neotrix::nt_agent_mod::plugin::skill_manifest::SkillManifest;

pub struct SkillRegistry {
    plugin_manager: PluginManager,
    executor: SkillExecutor,
    scan_dirs: Vec<PathBuf>,
    disclosure_layer: Option<ProgressiveDisclosureLayer>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut scan_dirs = Vec::new();
        if let Some(config_dir) = dirs::config_dir() {
            scan_dirs.push(config_dir.join("neotrix").join("skills"));
        }
        Self {
            plugin_manager: PluginManager::new(),
            executor: SkillExecutor::new(),
            scan_dirs,
            disclosure_layer: None,
        }
    }

    pub fn with_scan_dir(mut self, dir: PathBuf) -> Self {
        self.scan_dirs.push(dir);
        self
    }

    pub fn scan_and_register(&mut self) -> Result<(usize, usize), String> {
        let mut found = 0usize;
        let mut errors = 0usize;

        for dir in &self.scan_dirs {
            if !dir.exists() {
                continue;
            }
            let entries =
                std::fs::read_dir(dir).map_err(|e| format!("read skills dir {:?}: {}", dir, e))?;
            for entry in entries.flatten() {
                let skill_dir = entry.path();
                if !skill_dir.is_dir() {
                    continue;
                }
                let skill_md_path = skill_dir.join("SKILL.md");
                if !skill_md_path.exists() {
                    continue;
                }
                match std::fs::read_to_string(&skill_md_path) {
                    Ok(content) => match SkillManifest::from_skill_md(&content) {
                        Ok(manifest) => {
                            let _name = manifest.name.clone();
                            if manifest.validate().is_ok() {
                                self.executor.register_skill(manifest.clone());
                                if let Some(ref mut dl) = self.disclosure_layer {
                                    dl.register_skill_manifest(&manifest);
                                }
                                found += 1;
                            } else {
                                log::warn!(
                                    "[SkillRegistry] invalid manifest in {:?}",
                                    skill_md_path
                                );
                                errors += 1;
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "[SkillRegistry] parse error in {:?}: {:?}",
                                skill_md_path,
                                e
                            );
                            errors += 1;
                        }
                    },
                    Err(e) => {
                        log::warn!("[SkillRegistry] read error {:?}: {}", skill_md_path, e);
                        errors += 1;
                    }
                }
            }
        }

        Ok((found, errors))
    }

    pub fn executor(&self) -> &SkillExecutor {
        &self.executor
    }

    pub fn executor_mut(&mut self) -> &mut SkillExecutor {
        &mut self.executor
    }

    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }

    pub fn plugin_manager_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }

    pub fn find_skills_for_input(&self, input: &str) -> Vec<&SkillManifest> {
        self.executor.find_matching_skills(input)
    }

    pub fn execute_skill(&mut self, name: &str, input: &str) -> Result<String, String> {
        self.executor.execute(name, input)
    }

    /// Enable progressive disclosure by attaching a layer.
    pub fn with_disclosure_layer(mut self, max_cache: usize) -> Self {
        self.disclosure_layer =
            Some(ProgressiveDisclosureLayer::new().with_max_cache_size(max_cache));
        self
    }

    /// Search skill metadata without loading full code.
    /// Returns lightweight manifests only. Call `load_full_skill()` on match to get source.
    pub fn search_by_keyword(&self, query: &str) -> Vec<&DisclosureManifest> {
        self.disclosure_layer
            .as_ref()
            .map(|dl| dl.search_metadata(query, 5))
            .unwrap_or_default()
    }

    /// Lazily load the full skill code for a given skill_id.
    /// Returns `None` if the skill is not in cache or the layer is not enabled.
    pub fn load_full_skill(&mut self, skill_id: &str) -> Option<&FullSkill> {
        self.disclosure_layer
            .as_mut()
            .and_then(|dl| dl.load_full(skill_id))
    }

    pub fn all_triggers(&self) -> Vec<String> {
        self.executor.active_triggers()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_skill_dir(dir: &PathBuf, name: &str, trigger_words: &[&str]) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        let triggers: Vec<String> = trigger_words.iter().map(|s| format!("\"{}\"", s)).collect();
        let content = format!(
            "---\nname: {}\ndescription: Test skill {}\nversion: 1.0.0\ntrigger_words: [{}]\ntags: [\"test\"]\n---",
            name, name, triggers.join(", ")
        );
        fs::write(skill_dir.join("SKILL.md"), content).expect("write SKILL.md");
    }

    #[test]
    fn test_scan_and_register() {
        let dir = std::env::temp_dir().join(format!("neotrix-skill-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create test dir");

        setup_skill_dir(&dir, "deploy", &["deploy", "release"]);
        setup_skill_dir(&dir, "test-runner", &["test", "check"]);

        let mut registry = SkillRegistry::new().with_scan_dir(dir.clone());
        let (found, errors) = registry.scan_and_register().expect("scan should succeed");
        assert_eq!(found, 2);
        assert_eq!(errors, 0);

        let triggers = registry.all_triggers();
        assert!(triggers.contains(&"deploy".to_string()));
        assert!(triggers.contains(&"test".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_skills_for_input() {
        let mut registry = SkillRegistry::new();
        let dir = std::env::temp_dir().join(format!("neotrix-skill-find-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create test dir");

        setup_skill_dir(&dir, "search", &["search", "find"]);
        registry = registry.with_scan_dir(dir.clone());
        registry.scan_and_register().expect("scan");

        let matches = registry.find_skills_for_input("please search the web");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].name, "search");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_execute_skill_via_registry() {
        let mut registry = SkillRegistry::new();
        let dir = std::env::temp_dir().join(format!("neotrix-skill-exec-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create test dir");

        setup_skill_dir(&dir, "greeter", &["hello"]);
        registry = registry.with_scan_dir(dir.clone());
        registry.scan_and_register().expect("scan");

        let result = registry.execute_skill("greeter", "hello world");
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_registry() {
        let registry = SkillRegistry::new();
        assert!(registry.find_skills_for_input("anything").is_empty());
        assert!(registry.all_triggers().is_empty());
    }
}
