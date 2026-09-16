//! SkillRegistry — Central registry with content hashing, security scanning, lockfile integrity

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{ContentHash, SecurityScan, SkillManifest, SkillRegistryConfig, SkillRegistryError, SkillLockfile, SkillLockfileEntry};
use super::manifest::SkillTier;
use super::loader::SkillLoader;
use super::validator::SkillValidator;

/// Central skill registry with content hashing, security scanning, lockfile integrity
pub struct SkillRegistry {
    skills: HashMap<String, SkillManifest>,
    content_hashes: HashMap<String, ContentHash>,
    security_results: HashMap<String, SecurityScan>,
    lockfile: SkillLockfile,
    config: SkillRegistryConfig,
    loader: SkillLoader,
    validator: SkillValidator,
    load_count: usize,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            content_hashes: HashMap::new(),
            security_results: HashMap::new(),
            lockfile: SkillLockfile {
                version: "1.0.0".to_string(),
                skills: Vec::new(),
            },
            config: SkillRegistryConfig::default(),
            loader: SkillLoader::new(),
            validator: SkillValidator::new(),
            load_count: 0,
        }
    }

    pub fn with_config(config: SkillRegistryConfig) -> Self {
        Self {
            config: config.clone(),
            loader: SkillLoader::with_config(&config),
            validator: SkillValidator::with_config(&config),
            ..Self::new()
        }
    }

    pub fn register_skill(&mut self, manifest: SkillManifest, content: &[u8]) -> Result<(), SkillRegistryError> {
        let skill_id = manifest.id.clone();
        let hash = ContentHash::sha256(content);
        self.content_hashes.insert(skill_id.clone(), hash);

        let content_str = String::from_utf8_lossy(content);
        let scan = self.config.security_rules.scan(&content_str, &manifest.name, &manifest.purpose);
        self.security_results.insert(skill_id.clone(), scan);

        if scan.rejected {
            return Err(SkillRegistryError::SecurityRejected(
                format!("Skill '{}' rejected by security scan", skill_id),
            ));
        }

        self.validator.validate(&manifest, &content_str)?;
        self.verify_lockfile_integrity(&manifest)?;
        self.skills.insert(skill_id.clone(), manifest);
        self.load_count += 1;
        self.update_lockfile(&skill_id)?;
        Ok(())
    }

    pub fn load_skill(&mut self, skill_id: &str) -> Result<&SkillManifest, SkillRegistryError> {
        let skill = self.skills.get(skill_id)
            .ok_or_else(|| SkillRegistryError::ManifestValidation(
                format!("Skill '{}' not found in registry", skill_id)
            ))?;
        if self.config.enable_progressive_disclosure {
            self.loader.load_references(skill_id, &skill.references)?;
        }
        self.load_count += 1;
        Ok(skill)
    }

    pub fn load_manifest_only(&mut self, skill_id: &str) -> Result<&SkillManifest, SkillRegistryError> {
        self.skills.get(skill_id)
            .ok_or_else(|| SkillRegistryError::ManifestValidation(
                format!("Skill '{}' not found in registry", skill_id)
            ))
    }

    pub fn get_skill(&self, skill_id: &str) -> Option<&SkillManifest> {
        self.skills.get(skill_id)
    }

    pub fn skill_ids(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }

    pub fn all_skills(&self) -> Vec<&SkillManifest> {
        self.skills.values().collect()
    }

    pub fn get_content_hash(&self, skill_id: &str) -> Option<&ContentHash> {
        self.content_hashes.get(skill_id)
    }

    pub fn get_security_scan(&self, skill_id: &str) -> Option<&SecurityScan> {
        self.security_results.get(skill_id)
    }

    pub fn contains(&self, skill_id: &str) -> bool {
        self.skills.contains_key(skill_id)
    }

    pub fn remove_skill(&mut self, skill_id: &str) -> Option<SkillManifest> {
        self.content_hashes.remove(skill_id);
        self.security_results.remove(skill_id);
        self.lockfile.skills.retain(|e| e.skill_id != skill_id);
        self.skills.remove(skill_id)
    }

    pub fn verify_lockfile_integrity(&self, manifest: &SkillManifest) -> Result<(), SkillRegistryError> {
        if let Some(entry) = self.lockfile.skills.iter().find(|e| e.skill_id == manifest.id) {
            if entry.pinned_version != manifest.version {
                return Err(SkillRegistryError::LockfileIntegrity(
                    format!("Version mismatch for '{}'", manifest.id)
                ));
            }
        }
        Ok(())
    }

    fn update_lockfile(&mut self, skill_id: &str) -> Result<(), SkillRegistryError> {
        let manifest = self.skills.get(skill_id)
            .ok_or_else(|| SkillRegistryError::ManifestValidation(
                format!("Cannot update lockfile for unknown skill '{}'", skill_id)
            ))?;
        let hash = self.content_hashes.get(skill_id).map(|h| h.value.clone()).unwrap_or_default();
        let entry = SkillLockfileEntry {
            skill_id: manifest.id.clone(),
            content_hash: hash,
            manifest_path: manifest.manifest_path.clone(),
            references: manifest.references.clone(),
            scripts: manifest.scripts.clone(),
            pinned_version: manifest.version.clone(),
        };
        if let Some(existing) = self.lockfile.skills.iter_mut().find(|e| e.skill_id == manifest.id) {
            *existing = entry;
        } else {
            self.lockfile.skills.push(entry);
        }
        Ok(())
    }

    pub fn recompute_hash(&mut self, skill_id: &str, content: &[u8]) -> Result<(), SkillRegistryError> {
        self.content_hashes.insert(skill_id.to_string(), ContentHash::sha256(content));
        Ok(())
    }

    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_skills: self.skills.len(),
            total_references: self.skills.values().map(|s| s.references.len()).sum(),
            total_scripts: self.skills.values().map(|s| s.scripts.len()).sum(),
            total_loads: self.load_count,
            security_flags: self.security_results.values().flat_map(|s| s.flags.iter()).count(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RegistryStats {
    pub total_skills: usize,
    pub total_references: usize,
    pub total_scripts: usize,
    pub total_loads: usize,
    pub security_flags: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use super::super::manifest::SkillManifest;
    use super::super::manifest::SkillTier;
    use super::super::manifest::ConstellationLevel;

    fn create_test_manifest() -> SkillManifest {
        let mut manifest = SkillManifest::new("test_skill".to_string(), "NT-ACT".to_string(), SkillTier::NotablePassive, ConstellationLevel::C2);
        manifest.version = "1.0.0".to_string();
        manifest.manifest_path = PathBuf::from("skills/test_skill/SKILL.md");
        manifest.references.push(PathBuf::from("references/domain.md"));
        manifest.scripts.push(PathBuf::from("scripts/run.sh"));
        manifest.triggers = vec!["test".to_string()];
        manifest
    }

    #[test]
    fn test_register_and_get_skill() {
        let mut registry = SkillRegistry::new();
        let manifest = create_test_manifest();
        registry.register_skill(manifest, b"# Test\nPurpose: testing").unwrap();
        assert!(registry.contains("nt-act-test_skill"));
        let loaded = registry.get_skill("nt-act-test_skill").unwrap();
        assert_eq!(loaded.name, "test_skill");
    }

    #[test]
    fn test_remove_skill() {
        let mut registry = SkillRegistry::new();
        let manifest = create_test_manifest();
        registry.register_skill(manifest, b"content").unwrap();
        assert!(registry.remove_skill("nt-act-test_skill").is_some());
        assert!(!registry.contains("nt-act-test_skill"));
    }

    #[test]
    fn test_content_hash() {
        let mut registry = SkillRegistry::new();
        let manifest = create_test_manifest();
        registry.register_skill(manifest, b"test content").unwrap();
        let hash = registry.get_content_hash("nt-act-test_skill").unwrap();
        assert_eq!(hash.algorithm, "sha256");
        assert_eq!(hash.value.len(), 64);
    }

    #[test]
    fn test_load_skill() {
        let mut registry = SkillRegistry::new();
        let manifest = create_test_manifest();
        registry.register_skill(manifest, b"content").unwrap();
        let loaded = registry.load_skill("nt-act-test_skill").unwrap();
        assert_eq!(loaded.name, "test_skill");
    }

    #[test]
    fn test_stats() {
        let mut registry = SkillRegistry::new();
        let mut m1 = create_test_manifest();
        m1.references.push(PathBuf::from("ref1"));
        m1.scripts.push(PathBuf::from("script1"));
        registry.register_skill(m1, b"c1").unwrap();
        let mut m2 = create_test_manifest();
        m2.references.push(PathBuf::from("ref2"));
        registry.register_skill(m2, b"c2").unwrap();
        let stats = registry.stats();
        assert_eq!(stats.total_skills, 2);
        assert_eq!(stats.total_references, 3);
        assert_eq!(stats.total_scripts, 1);
    }
}