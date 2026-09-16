//! SkillLoader — Loading with security validation and progressive disclosure
//!
//! Implements progressive disclosure: SKILL.md first, references/scripts on demand.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{ContentHash, SkillManifest, SkillRegistryConfig, SkillValidator};

struct ReferenceCache {
    content: String,
    hash: ContentHash,
    loaded_at: std::time::SystemTime,
}

/// Skill loader with security validation and progressive disclosure
pub struct SkillLoader {
    reference_cache: Arc<Mutex<HashMap<String, HashMap<PathBuf, ReferenceCache>>>>,
    script_cache: Arc<Mutex<HashMap<String, HashMap<PathBuf, String>>>>,
    config: SkillRegistryConfig,
    validator: SkillValidator,
    cache_size: usize,
}

impl SkillLoader {
    pub fn new() -> Self {
        Self::with_config(&SkillRegistryConfig::default())
    }

    pub fn with_config(config: &SkillRegistryConfig) -> Self {
        Self {
            reference_cache: Arc::new(Mutex::new(HashMap::new())),
            script_cache: Arc::new(Mutex::new(HashMap::new())),
            config: config.clone(),
            validator: SkillValidator::with_config(config),
            cache_size: 0,
        }
    }

    pub fn load_manifest(&self, skill_id: &str, manifest_path: &Path) -> Result<String, SkillLoaderError> {
        let content = std::fs::read_to_string(manifest_path).map_err(|e| SkillLoaderError::Io(e))?;
        self.validator.validate_manifest_content(skill_id, &content)?;
        Ok(content)
    }

    pub fn load_reference(&self, skill_id: &str, reference_path: &Path) -> Result<String, SkillLoaderError> {
        {
            let cache = self.reference_cache.lock().map_err(|e| SkillLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            if let Some(skill_cache) = cache.get(skill_id) {
                if let Some(cached) = skill_cache.get(reference_path) {
                    return Ok(cached.content.clone());
                }
            }
        }

        let content = std::fs::read_to_string(reference_path).map_err(|e| SkillLoaderError::Io(e))?;
        self.validator.validate_reference_content(skill_id, reference_path, &content)?;
        let hash = ContentHash::sha256(content.as_bytes());
        let cache_entry = ReferenceCache {
            content: content.clone(),
            hash,
            loaded_at: std::time::SystemTime::now(),
        };
        {
            let mut cache = self.reference_cache.lock().map_err(|e| SkillLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            let skill_cache = cache.entry(skill_id.to_string()).or_insert_with(HashMap::new);
            skill_cache.insert(reference_path.to_path_buf(), cache_entry);
        }
        self.cache_size += 1;
        self.evict_if_needed();
        Ok(content)
    }

    pub fn load_script(&self, skill_id: &str, script_path: &Path) -> Result<String, SkillLoaderError> {
        {
            let cache = self.script_cache.lock().map_err(|e| SkillLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            if let Some(skill_cache) = cache.get(skill_id) {
                if let Some(cached) = skill_cache.get(script_path) {
                    return Ok(cached.clone());
                }
            }
        }
        let content = std::fs::read_to_string(script_path).map_err(|e| SkillLoaderError::Io(e))?;
        self.validator.validate_script_content(skill_id, script_path, &content)?;
        {
            let mut cache = self.script_cache.lock().map_err(|e| SkillLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            let skill_cache = cache.entry(skill_id.to_string()).or_insert_with(HashMap::new);
            skill_cache.insert(script_path.to_path_buf(), content.clone());
        }
        self.cache_size += 1;
        self.evict_if_needed();
        Ok(content)
    }

    pub fn load_all_references(&self, skill: &SkillManifest) -> Result<HashMap<PathBuf, String>, SkillLoaderError> {
        let mut loaded = HashMap::new();
        for ref_path in &skill.references {
            let content = self.load_reference(&skill.id, ref_path)?;
            loaded.insert(ref_path.clone(), content);
        }
        Ok(loaded)
    }

    pub fn load_progressive(&self, skill: &SkillManifest) -> Result<ProgressiveLoadResult, SkillLoaderError> {
        let manifest_content = self.load_manifest(&skill.id, &skill.manifest_path)?;
        Ok(ProgressiveLoadResult {
            skill_id: skill.id.clone(),
            manifest_content,
            references_loaded: Vec::new(),
            scripts_loaded: Vec::new(),
        })
    }

    pub fn get_cached_reference(&self, skill_id: &str, ref_path: &Path) -> Option<String> {
        let cache = self.reference_cache.lock().ok()?;
        cache.get(skill_id)?.get(ref_path).map(|c| c.content.clone())
    }

    pub fn cache_stats(&self) -> CacheStats {
        let ref_cache = self.reference_cache.lock().ok()?;
        let script_cache = self.script_cache.lock().ok()?;
        let ref_entries: usize = ref_cache.values().map(|m| m.len()).sum();
        let script_entries: usize = script_cache.values().map(|m| m.len()).sum();
        CacheStats { reference_entries: ref_entries, script_entries: script_entries, total_entries: ref_entries + script_entries }
    }

    fn evict_if_needed(&self) {
        if self.cache_size > self.config.cache_size_limit {
            let mut ref_cache = self.reference_cache.lock().ok();
            if let Some(cache) = ref_cache.as_mut() {
                for (_, skill_cache) in cache.iter_mut() {
                    if skill_cache.len() > 4 {
                        let to_remove: Vec<_> = skill_cache.keys().take(skill_cache.len() / 2).cloned().collect();
                        for key in to_remove { skill_cache.remove(&key); }
                    }
                }
            }
        }
    }
}

/// Result of a progressive load
#[derive(Debug, Clone)]
pub struct ProgressiveLoadResult {
    pub skill_id: String,
    pub manifest_content: String,
    pub references_loaded: Vec<PathBuf>,
    pub scripts_loaded: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub reference_entries: usize,
    pub script_entries: usize,
    pub total_entries: usize,
}

/// Skill loader error
#[derive(Debug, thiserror::Error)]
pub enum SkillLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Security validation failed for '{skill_id}': {reason}")]
    SecurityValidation { skill_id: String, reason: String },
    #[error("Manifest not found for '{skill_id}'")]
    ManifestNotFound { skill_id: String },
    #[error("Reference not found at {path}")]
    ReferenceNotFound { path: PathBuf },
    #[error("Script not found at {path}")]
    ScriptNotFound { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use std::path::PathBuf;
    use super::super::manifest::SkillManifest;
    use super::super::manifest::SkillTier;
    use super::super::manifest::ConstellationLevel;

    fn create_test_skill_dir() -> (tempfile::TempDir, SkillManifest) {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("SKILL.md");
        fs::write(&manifest_path, "# Test Skill\nPurpose: testing").unwrap();
        let mut manifest = SkillManifest::new("test".to_string(), "NT-ACT".to_string(), SkillTier::SmallPassive, ConstellationLevel::C1);
        manifest.manifest_path = manifest_path.clone();
        manifest.version = "1.0.0".to_string();
        (dir, manifest)
    }

    #[test]
    fn test_load_manifest() {
        let (dir, manifest) = create_test_skill_dir();
        let loader = SkillLoader::new();
        let content = loader.load_manifest("test-skill", &manifest.manifest_path).unwrap();
        assert!(content.contains("Test Skill"));
    }

    #[test]
    fn test_progressive_disclosure() {
        let (dir, manifest) = create_test_skill_dir();
        let loader = SkillLoader::new();
        let result = loader.load_progressive(&manifest).unwrap();
        assert_eq!(result.skill_id, "nt-act-test");
        assert!(result.references_loaded.is_empty());
    }

    #[test]
    fn test_load_reference_on_demand() {
        let (dir, manifest) = create_test_skill_dir();
        let ref_path = dir.path().join("references/domain.md");
        fs::create_dir_all(dir.path().join("references")).unwrap();
        fs::write(&ref_path, "Domain knowledge").unwrap();
        let mut manifest = manifest.clone();
        manifest.references.push(ref_path.clone());
        let loader = SkillLoader::new();
        let content = loader.load_reference("test-skill", &ref_path).unwrap();
        assert_eq!(content, "Domain knowledge");
        let cached = loader.get_cached_reference("test-skill", &ref_path).unwrap();
        assert_eq!(cached, "Domain knowledge");
    }
}