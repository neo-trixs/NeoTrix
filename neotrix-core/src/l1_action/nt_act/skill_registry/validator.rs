//! SkillValidator — Validates SKILL.md contracts and security compliance

use std::path::{Path, PathBuf};

use super::{SkillManifest, SkillRegistryConfig};

/// SKILL.md contract validator
pub struct SkillValidator {
    config: SkillRegistryConfig,
    validation_rules: Vec<ValidationRule>,
}

struct ValidationRule {
    id: String,
    description: String,
    check: Box<dyn Fn(&SkillManifest, &str) -> ValidationResult + Send + Sync>,
}

struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
}

impl SkillValidator {
    pub fn new() -> Self {
        Self::with_config(&SkillRegistryConfig::default())
    }

    pub fn with_config(config: &SkillRegistryConfig) -> Self {
        let mut validator = Self {
            config: config.clone(),
            validation_rules: Vec::new(),
        };
        validator.register_default_rules();
        validator
    }

    fn register_default_rules(&mut self) {
        self.validation_rules.push(ValidationRule {
            id: "R-SPEC-001".to_string(),
            description: "SKILL-SPEC.md must be under 200 lines".to_string(),
            check: Box::new(|manifest: &SkillManifest, content: &str| -> ValidationResult {
                let line_count = content.lines().count();
                if line_count >= 200 {
                    ValidationResult {
                        valid: false,
                        errors: vec![format!("SKILL-SPEC.md for '{}' exceeds 200 lines ({} lines)", manifest.name, line_count)],
                    }
                } else {
                    ValidationResult { valid: true, errors: Vec::new() }
                }
            }),
        });

        self.validation_rules.push(ValidationRule {
            id: "R-SPEC-002".to_string(),
            description: "Skill must have a valid tier classification".to_string(),
            check: Box::new(|manifest: &SkillManifest, _content: &str| -> ValidationResult {
                match manifest.tier {
                    super::manifest::SkillTier::SmallPassive | super::manifest::SkillTier::NotablePassive | super::manifest::SkillTier::Keystone => {
                        ValidationResult { valid: true, errors: Vec::new() }
                    }
                }
            }),
        });

        self.validation_rules.push(ValidationRule {
            id: "R-SPEC-003".to_string(),
            description: "Skill must have a purpose description".to_string(),
            check: Box::new(|manifest: &SkillManifest, _content: &str| -> ValidationResult {
                if manifest.purpose.is_empty() {
                    ValidationResult { valid: false, errors: vec!["Skill must have a non-empty purpose".to_string()] }
                } else {
                    ValidationResult { valid: true, errors: Vec::new() }
                }
            }),
        });

        self.validation_rules.push(ValidationRule {
            id: "R-SPEC-004".to_string(),
            description: "Skill must have trigger patterns".to_string(),
            check: Box::new(|manifest: &SkillManifest, _content: &str| -> ValidationResult {
                if manifest.triggers.is_empty() {
                    ValidationResult { valid: false, errors: vec!["Skill must have at least one trigger".to_string()] }
                } else {
                    ValidationResult { valid: true, errors: Vec::new() }
                }
            }),
        });

        self.validation_rules.push(ValidationRule {
            id: "R-SPEC-006".to_string(),
            description: "Version must follow semantic versioning".to_string(),
            check: Box::new(|manifest: &SkillManifest, _content: &str| -> ValidationResult {
                if semver::Version::parse(&manifest.version).is_err() {
                    ValidationResult { valid: false, errors: vec![format!("Version '{}' is not valid semver", manifest.version)] }
                } else {
                    ValidationResult { valid: true, errors: Vec::new() }
                }
            }),
        });
    }

    pub fn validate(&self, manifest: &SkillManifest, content: &str) -> Result<(), SkillValidatorError> {
        let mut all_errors = Vec::new();
        for rule in &self.validation_rules {
            let result = (rule.check)(manifest, content);
            if !result.valid {
                all_errors.extend(result.errors);
            }
        }
        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(SkillValidatorError::ValidationFailed(all_errors))
        }
    }

    pub fn validate_manifest_content(&self, _skill_id: &str, content: &str) -> Result<(), SkillValidatorError> {
        let errors: Vec<String> = content.lines()
            .enumerate()
            .filter(|(_, line)| line.contains("TODO: REMOVE") || line.contains("FIXME: SECURITY"))
            .map(|(i, line)| format!("Line {}: potential security issue", i + 1))
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(SkillValidatorError::ValidationFailed(errors))
        }
    }

    pub fn validate_reference_content(&self, _skill_id: &str, _ref_path: &Path, content: &str) -> Result<(), SkillValidatorError> {
        if content.len() > 100_000 {
            return Err(SkillValidatorError::ValidationFailed(vec![format!("Reference too large ({} bytes)", content.len())]));
        }
        Ok(())
    }

    pub fn validate_script_content(&self, _skill_id: &str, _script_path: &Path, content: &str) -> Result<(), SkillValidatorError> {
        if content.len() > 1_000_000 {
            return Err(SkillValidatorError::ValidationFailed(vec![format!("Script too large ({} bytes)", content.len())]));
        }
        Ok(())
    }

    pub fn add_rule<F>(&mut self, id: String, description: String, check: F)
    where
        F: Fn(&SkillManifest, &str) -> ValidationResult + Send + Sync + 'static,
    {
        self.validation_rules.push(ValidationRule { id, description, check: Box::new(check) });
    }

    pub fn rule_ids(&self) -> Vec<String> {
        self.validation_rules.iter().map(|r| r.id.clone()).collect()
    }
}

/// Skill validator error
#[derive(Debug, thiserror::Error)]
pub enum SkillValidatorError {
    #[error("Validation failed: {0}")]
    ValidationFailed(Vec<String>),
    #[error("Schema validation error: {0}")]
    SchemaError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use super::super::manifest::SkillManifest;
    use super::super::manifest::SkillTier;
    use super::super::manifest::ConstellationLevel;

    fn create_valid_manifest() -> SkillManifest {
        let mut manifest = SkillManifest::new("test_skill".to_string(), "NT-ACT".to_string(), SkillTier::NotablePassive, ConstellationLevel::C2);
        manifest.purpose = "A test skill".to_string();
        manifest.triggers = vec!["test".to_string()];
        manifest.version = "1.0.0".to_string();
        manifest.manifest_path = PathBuf::from("skills/test_skill/SKILL.md");
        manifest
    }

    #[test]
    fn test_validate_valid() {
        let validator = SkillValidator::new();
        let manifest = create_valid_manifest();
        assert!(validator.validate(&manifest, "# Test\nPurpose: test").is_ok());
    }

    #[test]
    fn test_validate_too_many_lines() {
        let validator = SkillValidator::new();
        let manifest = create_valid_manifest();
        let long = "# ".to_string() + &"Line\n".repeat(200);
        assert!(validator.validate(&manifest, &long).is_err());
    }

    #[test]
    fn test_validate_missing_purpose() {
        let validator = SkillValidator::new();
        let mut manifest = create_valid_manifest();
        manifest.purpose = String::new();
        assert!(validator.validate(&manifest, "# Test").is_err());
    }

    #[test]
    fn test_validate_missing_triggers() {
        let validator = SkillValidator::new();
        let mut manifest = create_valid_manifest();
        manifest.triggers = Vec::new();
        assert!(validator.validate(&manifest, "# Test").is_err());
    }

    #[test]
    fn test_manifest_content_validation() {
        let validator = SkillValidator::new();
        assert!(validator.validate_manifest_content("test", "safe").is_ok());
        assert!(validator.validate_manifest_content("test", "TODO: REMOVE").is_err());
    }

    #[test]
    fn test_add_custom_rule() {
        let mut validator = SkillValidator::new();
        validator.add_rule("CUSTOM".to_string(), "Custom".to_string(), Box::new(|_, _| ValidationResult { valid: true, errors: Vec::new() }));
        assert!(validator.rule_ids().contains(&"CUSTOM".to_string()));
    }
}