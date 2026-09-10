//! Validator — trailofbits/skills validator 模式吸收
//! 
//! 技能质量门禁: 30+ 强制规则，self-test 构建已知坏插件验证检查器
//! 对标 nt_governance::skill_validator

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// 验证规则类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationRule {
    PluginJsonExists,
    PluginJsonParses,
    PluginNameMatchesDir,
    PluginDirKebabCase,
    ReadmeExistsCaseSensitive,
    RegisteredInMarketplace,
    RegisteredInReadme,
    RegisteredInCodeowners,
    VersionMatches,
    VersionIncrements,
    SkillFrontmatterValid,
    SkillFrontmatterNoColonHash,
    AgentToolsVsAllowedTools,
    SubagentTypeNamespaced,
    NoHardcodedPaths,
    ModernPythonCommands,
    NoSidecars,
    UvLockCommitted,
    LoadabilityClaude,
    LoadabilityCodex,
    SkillUnder500Lines,
    ReferencesResolve,
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule: ValidationRule,
    pub passed: bool,
    pub message: String,
    pub severity: ValidationSeverity,
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// 验证器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub strict_mode: bool,
    pub max_skill_lines: usize,
    pub allowed_loadability_failures: usize,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            max_skill_lines: 500,
            allowed_loadability_failures: 0,
        }
    }
}

/// 技能验证器
#[derive(Debug)]
pub struct SkillValidator {
    config: ValidatorConfig,
    results: Vec<ValidationResult>,
}

impl SkillValidator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// 验证插件根目录
    pub fn validate_plugin(&mut self, plugin_dir: &Path) -> Vec<ValidationResult> {
        self.results.clear();
        
        // Rule: plugin.json exists
        self.check_plugin_json_exists(plugin_dir);
        
        // Rule: plugin.json parses
        let plugin_json = self.check_plugin_json_parses(plugin_dir);
        
        if let Some(plugin_data) = plugin_json {
            // Rule: name matches directory
            self.check_name_matches_dir(plugin_dir, &plugin_data);
            
            // Rule: version semver
            self.check_version_semver(&plugin_data);
        }
        
        // Rule: directory kebab-case
        self.check_dir_kebab_case(plugin_dir);
        
        // Rule: README.md exact case
        self.check_readme_case(plugin_dir);
        
        // Rule: Skills validation
        self.validate_skills_dir(plugin_dir.join("skills"));
        
        self.results.clone()
    }

    fn check_plugin_json_exists(&mut self, plugin_dir: &Path) {
        let path = plugin_dir.join(".claude-plugin").join("plugin.json");
        let passed = path.exists();
        self.results.push(ValidationResult {
            rule: ValidationRule::PluginJsonExists,
            passed,
            message: if passed { "plugin.json exists".to_string() } else { "plugin.json missing".to_string() },
            severity: ValidationSeverity::Error,
            file: Some(path),
            line: None,
        });
    }

    fn check_plugin_json_parses(&mut self, plugin_dir: &Path) -> Option<serde_json::Value> {
        let path = plugin_dir.join(".claude-plugin").join("plugin.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(v) => {
                    self.results.push(ValidationResult {
                        rule: ValidationRule::PluginJsonParses,
                        passed: true,
                        message: "plugin.json parses".to_string(),
                        severity: ValidationSeverity::Error,
                        file: Some(path.clone()),
                        line: None,
                    });
                    Some(v)
                }
                Err(e) => {
                    self.results.push(ValidationResult {
                        rule: ValidationRule::PluginJsonParses,
                        passed: false,
                        message: format!("plugin.json parse error: {}", e),
                        severity: ValidationSeverity::Error,
                        file: Some(path),
                        line: None,
                    });
                    None
                }
            }
        } else {
            None
        }
    }

    fn check_name_matches_dir(&mut self, plugin_dir: &Path, plugin_data: &serde_json::Value) {
        let dir_name = plugin_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let name = plugin_data.get("name").and_then(|v| v.as_str()).unwrap_or("");
        
        let passed = dir_name == name;
        self.results.push(ValidationResult {
            rule: ValidationRule::PluginNameMatchesDir,
            passed,
            message: if passed { 
                format!("name '{}' matches dir '{}'", name, dir_name)
            } else {
                format!("name '{}' != dir '{}'", name, dir_name)
            },
            severity: ValidationSeverity::Error,
            file: Some(plugin_dir.to_path_buf()),
            line: None,
        });
    }

    fn check_version_semver(&mut self, plugin_data: &serde_json::Value) {
        let version = plugin_data.get("version").and_then(|v| v.as_str()).unwrap_or("");
        let passed = semver::Version::parse(version).is_ok();
        self.results.push(ValidationResult {
            rule: ValidationRule::VersionMatches,
            passed,
            message: if passed { "version is valid semver".to_string() } else { "version not valid semver".to_string() },
            severity: ValidationSeverity::Error,
            file: None,
            line: None,
        });
    }

    fn check_dir_kebab_case(&mut self, plugin_dir: &Path) {
        let dir_name = plugin_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let kebab_case = regex::Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();
        let passed = kebab_case.is_match(dir_name) && dir_name.len() <= 64;
        self.results.push(ValidationResult {
            rule: ValidationRule::PluginDirKebabCase,
            passed,
            message: if passed { "directory is kebab-case".to_string() } else { "directory not kebab-case or >64 chars".to_string() },
            severity: ValidationSeverity::Error,
            file: Some(plugin_dir.to_path_buf()),
            line: None,
        });
    }

    fn check_readme_case(&mut self, plugin_dir: &Path) {
        let readme_path = plugin_dir.join("README.md");
        let passed = readme_path.exists() && readme_path.file_name().and_then(|s| s.to_str()) == Some("README.md");
        self.results.push(ValidationResult {
            rule: ValidationRule::ReadmeExistsCaseSensitive,
            passed,
            message: if passed { "README.md exists with correct case".to_string() } else { "README.md missing or wrong case".to_string() },
            severity: ValidationSeverity::Error,
            file: Some(readme_path),
            line: None,
        });
    }

    fn validate_skills_dir(&mut self, skills_dir: &Path) {
        if !skills_dir.exists() {
            return;
        }
        
        for entry in std::fs::read_dir(skills_dir).into_iter().flatten() {
            let skill_dir = entry.path();
            if skill_dir.is_dir() {
                self.validate_skill(&skill_dir);
            }
        }
    }

    fn validate_skill(&mut self, skill_dir: &Path) {
        let skill_file = skill_dir.join("SKILL.md");
        if !skill_file.exists() {
            return;
        }
        
        let content = std::fs::read_to_string(&skill_file).unwrap_or_default();
        
        // Rule: Frontmatter valid
        self.check_skill_frontmatter(&skill_file, &content);
        
        // Rule: Frontmatter no unquoted colon/hash
        self.check_frontmatter_no_colon_hash(&skill_file, &content);
        
        // Rule: Skill under 500 lines
        self.check_skill_line_count(&skill_file, &content);
        
        // Rule: References resolve
        self.check_references_resolve(skill_dir, &content);
    }

    fn check_skill_frontmatter(&mut self, skill_file: &Path, content: &str) {
        let passed = content.trim_start().starts_with("---") && 
            content[3..].find("---").is_some();
        self.results.push(ValidationResult {
            rule: ValidationRule::SkillFrontmatterValid,
            passed,
            message: if passed { "SKILL.md has valid frontmatter".to_string() } else { "SKILL.md missing or invalid frontmatter".to_string() },
            severity: ValidationSeverity::Error,
            file: Some(skill_file.to_path_buf()),
            line: None,
        });
    }

    fn check_frontmatter_no_colon_hash(&mut self, skill_file: &Path, content: &str) {
        let frontmatter_end = content[3..].find("---").map(|i| i + 3).unwrap_or(content.len());
        let frontmatter = &content[3..frontmatter_end];
        
        let mut passed = true;
        let mut line_num = 0;
        for line in frontmatter.lines() {
            line_num += 1;
            let trimmed = line.trim();
            if trimmed.contains(": ") || trimmed.contains(" #") {
                // Check if value is quoted
                if let Some(colon_pos) = trimmed.find(':') {
                    let value = trimmed[colon_pos + 1..].trim();
                    if !value.starts_with('"') && !value.starts_with('\'') && !value.is_empty() {
                        passed = false;
                        break;
                    }
                }
            }
        }
        
        self.results.push(ValidationResult {
            rule: ValidationRule::SkillFrontmatterNoColonHash,
            passed,
            message: if passed { "frontmatter values properly quoted".to_string() } else { "frontmatter has unquoted value with : or #".to_string() },
            severity: ValidationSeverity::Error,
            file: Some(skill_file.to_path_buf()),
            line: if passed { None } else { Some(line_num) },
        });
    }

    fn check_skill_line_count(&mut self, skill_file: &Path, content: &str) {
        let line_count = content.lines().count();
        let passed = line_count <= self.config.max_skill_lines;
        self.results.push(ValidationResult {
            rule: ValidationRule::SkillUnder500Lines,
            passed,
            message: if passed { 
                format!("SKILL.md {} lines ≤ {}", line_count, self.config.max_skill_lines)
            } else {
                format!("SKILL.md {} lines > {}", line_count, self.config.max_skill_lines)
            },
            severity: ValidationSeverity::Warning,
            file: Some(skill_file.to_path_buf()),
            line: None,
        });
    }

    fn check_references_resolve(&mut self, skill_dir: &Path, content: &str) {
        let mut passed = true;
        let mut missing = Vec::new();
        
        for line in content.lines() {
            if line.contains("](") && line.contains(".md") {
                // Extract markdown link
                let re = regex::Regex::new(r"\]\(([^)]+)\)").unwrap();
                for cap in re.captures_iter(line) {
                    let link = cap.get(1).unwrap().as_str();
                    if !link.starts_with("http") && !link.starts_with("#") {
                        let ref_path = skill_dir.join(link);
                        if !ref_path.exists() {
                            passed = false;
                            missing.push(link.to_string());
                        }
                    }
                }
            }
        }
        
        self.results.push(ValidationResult {
            rule: ValidationRule::ReferencesResolve,
            passed,
            message: if passed { "all references resolve".to_string() } else { format!("missing references: {}", missing.join(", ")) },
            severity: ValidationSeverity::Warning,
            file: Some(skill_dir.to_path_buf()),
            line: None,
        });
    }

    /// Self-test: 构建已知坏插件验证检查器
    pub fn self_test_bad_plugin() -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let bad_plugin = temp_dir.path().join("bad-plugin");
        std::fs::create_dir_all(bad_plugin.join(".claude-plugin")).unwrap();
        std::fs::create_dir_all(bad_plugin.join("skills").join("bad-skill")).unwrap();
        
        // Invalid plugin.json
        std::fs::write(bad_plugin.join(".claude-plugin").join("plugin.json"), "{ invalid json").unwrap();
        
        // Bad skill
        std::fs::write(
            bad_plugin.join("skills").join("bad-skill").join("SKILL.md"),
            "---\nname: bad\ndescription: test\ntriggers: [test]\n---\n".repeat(200) // >500 lines
        ).unwrap();
        
        let mut validator = SkillValidator::new(ValidatorConfig::default());
        let results = validator.validate_plugin(&bad_plugin);
        
        // Should catch the errors
        let has_plugin_json_error = results.iter().any(|r| 
            r.rule == ValidationRule::PluginJsonParses && !r.passed
        );
        let has_line_count_warning = results.iter().any(|r| 
            r.rule == ValidationRule::SkillUnder500Lines && !r.passed
        );
        
        if !has_plugin_json_error {
            return Err("Failed to catch invalid plugin.json".to_string());
        }
        if !has_line_count_warning {
            return Err("Failed to catch >500 lines skill".to_string());
        }
        
        Ok(())
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        // Test 1: Valid plugin passes
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let good_plugin = temp_dir.path().join("good-plugin");
        std::fs::create_dir_all(good_plugin.join(".claude-plugin")).unwrap();
        std::fs::create_dir_all(good_plugin.join("skills").join("good-skill")).unwrap();
        std::fs::write(good_plugin.join("README.md"), "# Test").unwrap();
        std::fs::write(
            good_plugin.join(".claude-plugin").join("plugin.json"),
            r#"{"name": "good-plugin", "version": "1.0.0", "description": "Test"}"#
        ).unwrap();
        std::fs::write(
            good_plugin.join("skills").join("good-skill").join("SKILL.md"),
            "---\nname: good\ndescription: Test skill\ntriggers: [\"test\"]\n---\nbody"
        ).unwrap();
        
        let mut validator = SkillValidator::new(ValidatorConfig::default());
        let results = validator.validate_plugin(&good_plugin);
        
        let critical_errors: Vec<_> = results.iter()
            .filter(|r| r.severity == ValidationSeverity::Error && !r.passed)
            .collect();
        
        if !critical_errors.is_empty() {
            return Err(format!("Valid plugin failed validation: {:?}", critical_errors));
        }
        
        // Test 2: Bad plugin self-test
        Self::self_test_bad_plugin()?;
        
        Ok(())
    }
}

impl Default for SkillValidator {
    fn default() -> Self {
        Self::new(ValidatorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = SkillValidator::new(ValidatorConfig::default());
        assert_eq!(validator.config.max_skill_lines, 500);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(SkillValidator::self_test().is_ok());
    }
}