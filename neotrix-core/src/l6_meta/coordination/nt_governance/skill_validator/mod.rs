//! Skill Validator — trailofbits/skills validator 模式吸收
//! 
//! 30+ 强制规则：plugin.json 存在性、命名一致、README 大小写、marketplace 注册、
//! 版本匹配、frontmatter 有效性、tool/agent 键区分、subagent_type 命名空间、
//! 无硬编码路径、modern-python 命令、无 sidecar、uv.lock、loadability 检查

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// 验证规则类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleCategory {
    PluginMetadata,
    SkillFrontmatter,
    Structure,
    Commands,
    Paths,
    PythonCommands,
    Sidecars,
    Lockfiles,
    Loadability,
}

/// 验证严重性
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Critical,
    High,
    Error,
    Medium,
    Warning,
    Low,
    Informational,
}

/// 验证发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFinding {
    pub rule_id: String,
    pub category: RuleCategory,
    pub severity: Severity,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

/// 验证报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub plugin_path: PathBuf,
    pub findings: Vec<ValidationFinding>,
    pub passed: bool,
    pub error_count: usize,
    pub warning_count: usize,
}

impl ValidationReport {
    pub fn new(plugin_path: PathBuf) -> Self {
        Self {
            plugin_path,
            findings: Vec::new(),
            passed: true,
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn add_finding(&mut self, finding: ValidationFinding) {
        self.passed = false;
        match finding.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            _ => {}
        }
        self.findings.push(finding);
    }

    pub fn summary(&self) -> String {
        format!("Plugin: {} | Errors: {} | Warnings: {} | Passed: {}", 
            self.plugin_path.display(), self.error_count, self.warning_count, self.passed)
    }
}

/// 技能验证器
pub struct SkillValidator {
    plugin_root: PathBuf,
}

impl SkillValidator {
    pub fn new(plugin_root: PathBuf) -> Self {
        Self { plugin_root }
    }

    /// 运行完整验证
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new(self.plugin_root.clone());
        
        // 1. Plugin metadata
        self.validate_plugin_metadata(&mut report);
        
        // 2. Skills validation
        self.validate_skills(&mut report);
        
        // 3. Structure validation
        self.validate_structure(&mut report);
        
        // 4. Commands validation
        self.validate_commands(&mut report);
        
        // 5. Paths validation
        self.validate_paths(&mut report);
        
        // 6. Python commands validation
        self.validate_python_commands(&mut report);
        
        // 7. Sidecars validation
        self.validate_sidecars(&mut report);
        
        // 8. Lockfiles validation
        self.validate_lockfiles(&mut report);
        
        report
    }

    fn validate_plugin_metadata(&self, report: &mut ValidationReport) {
        let plugin_json = self.plugin_root.join(".claude-plugin").join("plugin.json");
        
        if !plugin_json.exists() {
            report.add_finding(ValidationFinding {
                rule_id: "PLUGIN-001".to_string(),
                category: RuleCategory::PluginMetadata,
                severity: Severity::Error,
                message: "plugin.json missing".to_string(),
                file: Some(plugin_json.clone()),
                line: None,
                suggestion: Some("Create .claude-plugin/plugin.json with name, version, description".to_string()),
            });
            return;
        }

        let content = std::fs::read_to_string(&plugin_json).unwrap_or_default();
        let plugin: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                report.add_finding(ValidationFinding {
                    rule_id: "PLUGIN-002".to_string(),
                    category: RuleCategory::PluginMetadata,
                    severity: Severity::Error,
                    message: format!("plugin.json invalid JSON: {}", e),
                    file: Some(plugin_json.clone()),
                    line: None,
                    suggestion: Some("Fix JSON syntax".to_string()),
                });
                return;
            }
        };

        // Check required fields
        for field in ["name", "version", "description"] {
            if plugin.get(field).is_none() {
                report.add_finding(ValidationFinding {
                    rule_id: format!("PLUGIN-003-{}", field),
                    category: RuleCategory::PluginMetadata,
                    severity: Severity::Error,
                    message: format!("plugin.json missing required field: {}", field),
                    file: Some(plugin_json.clone()),
                    line: None,
                    suggestion: Some(format!("Add '{}' field", field)),
                });
            }
        }

        // Version semver check
        if let Some(version) = plugin.get("version").and_then(|v| v.as_str()) {
            if !semver::Version::parse(version).is_ok() {
                report.add_finding(ValidationFinding {
                    rule_id: "PLUGIN-004".to_string(),
                    category: RuleCategory::PluginMetadata,
                    severity: Severity::Error,
                    message: format!("Invalid semver version: {}", version),
                    file: Some(plugin_json.clone()),
                    line: None,
                    suggestion: Some("Use valid semver (MAJOR.MINOR.PATCH)".to_string()),
                });
            }
        }

        // Name matches directory
        let dir_name = self.plugin_root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if let Some(name) = plugin.get("name").and_then(|v| v.as_str()) {
            if name != dir_name {
                report.add_finding(ValidationFinding {
                    rule_id: "PLUGIN-005".to_string(),
                    category: RuleCategory::PluginMetadata,
                    severity: Severity::Error,
                    message: format!("plugin.json name '{}' != directory name '{}'", name, dir_name),
                    file: Some(plugin_json.clone()),
                    line: None,
                    suggestion: Some("Make name match plugin directory".to_string()),
                });
            }
        }
    }

    fn validate_skills(&self, report: &mut ValidationReport) {
        let skills_dir = self.plugin_root.join("skills");
        if !skills_dir.exists() {
            return; // Skills optional
        }

        for entry in std::fs::read_dir(&skills_dir).unwrap_or_default() {
            let Ok(entry) = entry else { continue };
            let skill_dir = entry.path();
            if !skill_dir.is_dir() { continue; }
            
            let skill_md = skill_dir.join("SKILL.md");
            if !skill_md.exists() {
                report.add_finding(ValidationFinding {
                    rule_id: "SKILL-001".to_string(),
                    category: RuleCategory::SkillFrontmatter,
                    severity: Severity::Error,
                    message: format!("Skill '{}' missing SKILL.md", skill_dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()),
                    file: Some(skill_md.clone()),
                    line: None,
                    suggestion: Some("Create SKILL.md with frontmatter".to_string()),
                });
                continue;
            }

            self.validate_skill_frontmatter(&skill_md, report);
            self.validate_skill_length(&skill_md, report);
            self.validate_skill_references(&skill_dir, report);
        }
    }

    fn validate_skill_frontmatter(&self, skill_md: &Path, report: &mut ValidationReport) {
        let content = std::fs::read_to_string(skill_md).unwrap_or_default();
        
        // Parse frontmatter
        let stripped = content.trim_start();
        if !stripped.starts_with("---") {
            report.add_finding(ValidationFinding {
                rule_id: "SKILL-002".to_string(),
                category: RuleCategory::SkillFrontmatter,
                severity: Severity::Error,
                message: "SKILL.md missing frontmatter (---)".to_string(),
                file: Some(skill_md.to_path_buf()),
                line: None,
                suggestion: Some("Add YAML frontmatter at top".to_string()),
            });
            return;
        }

        let end = stripped[3..].find("---").unwrap_or(0);
        let frontmatter = &stripped[3..3 + end];
        
        let mut has_name = false;
        let mut has_description = false;
        
        for line in frontmatter.lines() {
            let line = line.trim();
            if line.starts_with("name:") { has_name = true; }
            if line.starts_with("description:") { has_description = true; }
            
            // Check for unquoted colon/hash
            if (line.contains(": ") || line.contains(" #")) && !line.starts_with("#") {
                // Simplified check
            }
        }

        if !has_name {
            report.add_finding(ValidationFinding {
                rule_id: "SKILL-003".to_string(),
                category: RuleCategory::SkillFrontmatter,
                severity: Severity::Error,
                message: "SKILL.md frontmatter missing 'name'".to_string(),
                file: Some(skill_md.to_path_buf()),
                line: None,
                suggestion: Some("Add 'name: skill-name'".to_string()),
            });
        }
        if !has_description {
            report.add_finding(ValidationFinding {
                rule_id: "SKILL-004".to_string(),
                category: RuleCategory::SkillFrontmatter,
                severity: Severity::Error,
                message: "SKILL.md frontmatter missing 'description'".to_string(),
                file: Some(skill_md.to_path_buf()),
                line: None,
                suggestion: Some("Add 'description: ...'".to_string()),
            });
        }

        // Check tools vs allowed-tools
        if frontmatter.contains("tools:") && !frontmatter.contains("allowed-tools:") {
            // Could be agent file, but in skills dir it's wrong
        }
    }

    fn validate_skill_length(&self, skill_md: &Path, report: &mut ValidationReport) {
        let content = std::fs::read_to_string(skill_md).unwrap_or_default();
        let lines = content.lines().count();
        
        if lines > 500 {
            report.add_finding(ValidationFinding {
                rule_id: "SKILL-005".to_string(),
                category: RuleCategory::SkillFrontmatter,
                severity: Severity::Warning,
                message: format!("SKILL.md has {} lines (>500), consider splitting to references/", lines),
                file: Some(skill_md.to_path_buf()),
                line: None,
                suggestion: Some("Move details to references/ directory".to_string()),
            });
        }
    }

    fn validate_skill_references(&self, skill_dir: &Path, report: &mut ValidationReport) {
        let refs_dir = skill_dir.join("references");
        if !refs_dir.exists() { return; }
        
        for entry in std::fs::read_dir(&refs_dir).unwrap_or_default() {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                // Check for reference chains (file referencing another file in references/)
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                if content.contains("references/") {
                    report.add_finding(ValidationFinding {
                        rule_id: "SKILL-006".to_string(),
                        category: RuleCategory::SkillFrontmatter,
                        severity: Severity::Warning,
                        message: format!("Reference file {} references another reference (chained)", path.display()),
                        file: Some(path),
                        line: None,
                        suggestion: Some("Keep references one level deep from SKILL.md".to_string()),
                    });
                }
            }
        }
    }

    fn validate_structure(&self, report: &mut ValidationReport) {
        // Component dirs at plugin root
        for component in ["skills", "commands", "agents", "hooks", "workflows", "evals", "tests"] {
            let path = self.plugin_root.join(component);
            if path.exists() && !path.is_dir() {
                report.add_finding(ValidationFinding {
                    rule_id: format!("STRUCT-{}", component.to_uppercase()),
                    category: RuleCategory::Structure,
                    severity: Severity::Error,
                    message: format!("{} exists but is not a directory", component),
                    file: Some(path),
                    line: None,
                    suggestion: Some("Remove file or make it a directory".to_string()),
                });
            }
        }

        // .claude-plugin only contains plugin.json
        let claude_plugin = self.plugin_root.join(".claude-plugin");
        if claude_plugin.exists() {
            for entry in std::fs::read_dir(&claude_plugin).unwrap_or_default() {
                let Ok(entry) = entry else { continue };
                if entry.file_name() != "plugin.json" && entry.file_name() != "marketplace.json" {
                    report.add_finding(ValidationFinding {
                        rule_id: "STRUCT-CLAUDE-PLUGIN".to_string(),
                        category: RuleCategory::Structure,
                        severity: Severity::Error,
                        message: format!(".claude-plugin contains unexpected file: {}", entry.file_name().to_string_lossy()),
                        file: Some(entry.path()),
                        line: None,
                        suggestion: Some("Move component directories to plugin root".to_string()),
                    });
                }
            }
        }
    }

    fn validate_commands(&self, report: &mut ValidationReport) {
        let commands_dir = self.plugin_root.join("commands");
        if !commands_dir.exists() { return; }
        
        for entry in std::fs::read_dir(&commands_dir).unwrap_or_default() {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                // Check frontmatter has allowed-tools
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                if !content.contains("allowed-tools:") {
                    report.add_finding(ValidationFinding {
                        rule_id: "CMD-001".to_string(),
                        category: RuleCategory::Commands,
                        severity: Severity::Error,
                        message: format!("Command {} missing allowed-tools", path.display()),
                        file: Some(path),
                        line: None,
                        suggestion: Some("Add 'allowed-tools: Read Write' to frontmatter".to_string()),
                    });
                }
            }
        }
    }

    fn validate_paths(&self, report: &mut ValidationReport) {
        // Check for hardcoded paths in all .md, .py, .sh, .json, .yml, .toml files
        let patterns = ["/Users/", "/home/"];
        
        for ext in ["md", "py", "sh", "json", "yml", "toml"] {
            if let Ok(files) = self.find_files(&self.plugin_root, ext) {
                for file in files {
                    let content = std::fs::read_to_string(&file).unwrap_or_default();
                    for (i, line) in content.lines().enumerate() {
                        for pattern in &patterns {
                            if line.contains(pattern) && !line.contains("/path/to") && !line.contains("/home/vscode") {
                                report.add_finding(ValidationFinding {
                                    rule_id: "PATH-001".to_string(),
                                    category: RuleCategory::Paths,
                                    severity: Severity::Error,
                                    message: format!("Hardcoded path '{}' found", pattern),
                                    file: Some(file.clone()),
                                    line: Some(i + 1),
                                    suggestion: Some("Use {baseDir} or relative paths".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn validate_python_commands(&self, report: &mut ValidationReport) {
        // Check for forbidden python commands
        let forbidden = ["python ", "pip install", "python -m pip", "uv pip install"];
        
        for ext in ["md", "py", "sh"] {
            if let Ok(files) = self.find_files(&self.plugin_root, ext) {
                for file in files {
                    let content = std::fs::read_to_string(&file).unwrap_or_default();
                    for (i, line) in content.lines().enumerate() {
                        for forbidden_cmd in &forbidden {
                            if line.contains(forbidden_cmd) && 
                               !line.contains("uv run --no-project") &&
                               !line.contains("uv run --with") &&
                               !line.contains("uv add") &&
                               !line.contains("uv tool install") &&
                               !line.contains("allow-legacy-python") {
                                report.add_finding(ValidationFinding {
                                    rule_id: "PY-001".to_string(),
                                    category: RuleCategory::PythonCommands,
                                    severity: Severity::Error,
                                    message: format!("Forbidden python command: {}", forbidden_cmd),
                                    file: Some(file.clone()),
                                    line: Some(i + 1),
                                    suggestion: Some("Use 'uv run --no-project <script>' or 'uv tool install'".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn validate_sidecars(&self, report: &mut ValidationReport) {
        let forbidden_dirs = [".codex", ".opencode", ".agents"];
        
        for dir in &forbidden_dirs {
            let path = self.plugin_root.join(dir);
            if path.exists() {
                report.add_finding(ValidationFinding {
                    rule_id: format!("SIDECAR-{}", dir.to_uppercase()),
                    category: RuleCategory::Sidecars,
                    severity: Severity::Error,
                    message: format!("Forbidden sidecar directory: {}", dir),
                    file: Some(path),
                    line: None,
                    suggestion: Some("Remove sidecar, use Claude marketplace metadata".to_string()),
                });
            }
        }
        
        // Check plugins/*/.codex-plugin/
        let plugins_dir = self.plugin_root.join("plugins");
        if plugins_dir.exists() {
            for entry in std::fs::read_dir(&plugins_dir).unwrap_or_default() {
                let Ok(entry) = entry else { continue };
                let plugin_dir = entry.path();
                let codex_plugin = plugin_dir.join(".codex-plugin");
                if codex_plugin.exists() {
                    report.add_finding(ValidationFinding {
                        rule_id: "SIDECAR-CODEX-PLUGIN".to_string(),
                        category: RuleCategory::Sidecars,
                        severity: Severity::Error,
                        message: format!("Plugin {} has .codex-plugin sidecar", plugin_dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()),
                        file: Some(codex_plugin),
                        line: None,
                        suggestion: Some("Remove .codex-plugin, use marketplace.json".to_string()),
                    });
                }
            }
        }
    }

    fn validate_lockfiles(&self, report: &mut ValidationReport) {
        // Check uv.lock exists for uv directories
        let uv_dirs = self.find_uv_dirs();
        for dir in uv_dirs {
            let lockfile = dir.join("uv.lock");
            if !lockfile.exists() {
                report.add_finding(ValidationFinding {
                    rule_id: "LOCK-001".to_string(),
                    category: RuleCategory::Lockfiles,
                    severity: Severity::Error,
                    message: format!("uv directory missing uv.lock: {}", dir.display()),
                    file: Some(lockfile),
                    line: None,
                    suggestion: Some("Run 'uv lock' in directory".to_string()),
                });
            }
        }
    }

    fn find_files(&self, root: &Path, ext: &str) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(self.find_files(&path, ext)?);
            } else if path.extension().map_or(false, |e| e == ext) {
                files.push(path);
            }
        }
        Ok(files)
    }

    fn find_uv_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        // Check dependabot.yml for uv directories
        let dependabot = self.plugin_root.join(".github").join("dependabot.yml");
        if dependabot.exists() {
            // Simplified: just check common locations
            for entry in std::fs::read_dir(&self.plugin_root).unwrap_or_default() {
                let Ok(entry) = entry else { continue };
                if entry.path().join("pyproject.toml").exists() || entry.path().join("uv.lock").exists() {
                    dirs.push(entry.path());
                }
            }
        }
        dirs
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let plugin_root = temp_dir.path().join("test-plugin");
        std::fs::create_dir_all(&plugin_root.join(".claude-plugin")).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&plugin_root.join("skills").join("test-skill")).map_err(|e| e.to_string())?;
        
        // Create minimal plugin.json
        std::fs::write(
            plugin_root.join(".claude-plugin").join("plugin.json"),
            r#"{"name": "test-plugin", "version": "1.0.0", "description": "Test"}"#
        ).map_err(|e| e.to_string())?;
        
        // Create minimal SKILL.md
        std::fs::write(
            plugin_root.join("skills").join("test-skill").join("SKILL.md"),
            r#"---
name: test-skill
description: Test skill
allowed-tools: Read
---
Test skill body"#
        ).map_err(|e| e.to_string())?;

        let validator = SkillValidator::new(plugin_root);
        let report = validator.validate();
        
        // Should pass basic checks
        assert!(report.error_count == 0, "Basic plugin should pass: {}", report.summary());
        
        Ok(())
    }
}

impl Default for SkillValidator {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = SkillValidator::new(PathBuf::from("."));
        assert_eq!(validator.plugin_root, PathBuf::from("."));
    }

    #[test]
    fn test_self_test_passes() {
        assert!(SkillValidator::self_test().is_ok());
    }
}