use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub trigger: SkillTrigger,
    pub io: SkillIO,
    pub e8_mode: Option<String>,
    pub quality_threshold: f64,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    pub sources: Vec<String>,
    pub condition: Option<String>,
    pub frequency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIO {
    pub input_type: String,
    pub output_type: String,
    pub input_dims: usize,
    pub output_dims: usize,
}

#[derive(Debug)]
pub enum SkillLoadError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for SkillLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillLoadError::Io(e) => write!(f, "IO error: {}", e),
            SkillLoadError::Parse(e) => write!(f, "parse error: {}", e),
            SkillLoadError::Validation(e) => write!(f, "validation error: {}", e),
        }
    }
}

impl std::error::Error for SkillLoadError {}

pub struct SkillDocLoader;

impl SkillDocLoader {
    pub fn scan_skills(dir: &Path) -> Result<Vec<SkillDefinition>, SkillLoadError> {
        let mut skills = Vec::new();
        let mut seen_ids = HashSet::new();

        let entries = std::fs::read_dir(dir).map_err(SkillLoadError::Io)?;

        for entry in entries {
            let entry = entry.map_err(SkillLoadError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                let sub = Self::scan_skills(&path)?;
                for s in sub {
                    if seen_ids.insert(s.id.clone()) {
                        skills.push(s);
                    }
                }
                continue;
            }

            if path
                .extension()
                .and_then(|e| e.to_str())
                .map_or(false, |e| e == "json")
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map_or(false, |s| s.ends_with(".skill"))
            {
                let def = Self::load_skill(&path)?;
                if seen_ids.insert(def.id.clone()) {
                    skills.push(def);
                }
            }
        }

        Ok(skills)
    }

    pub fn load_skill(path: &Path) -> Result<SkillDefinition, SkillLoadError> {
        let content = std::fs::read_to_string(path).map_err(SkillLoadError::Io)?;
        let def: SkillDefinition =
            serde_json::from_str(&content).map_err(|e| SkillLoadError::Parse(e.to_string()))?;
        Ok(def)
    }

    pub fn validate(&self, skill: &SkillDefinition) -> Vec<String> {
        let mut warnings = Vec::new();

        if skill.id.trim().is_empty() {
            warnings.push("id must not be empty".into());
        }

        if skill.version.is_empty() || !skill.version.contains('.') {
            warnings.push(format!(
                "version '{}' is not semver-like (expected format: x.y.z)",
                skill.version
            ));
        }

        if skill.trigger.sources.is_empty() {
            warnings.push("trigger.sources must not be empty".into());
        }

        if skill.quality_threshold < 0.0 || skill.quality_threshold > 1.0 {
            warnings.push(format!(
                "quality_threshold must be in [0, 1], got {}",
                skill.quality_threshold
            ));
        }

        if skill.dependencies.contains(&skill.id) {
            warnings.push(format!(
                "dependency list must not contain self ('{}')",
                skill.id
            ));
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skill() -> SkillDefinition {
        SkillDefinition {
            id: "code_review_syntax_parser".into(),
            name: "Syntax-Aware Code Reviewer".into(),
            version: "1.2.0".into(),
            description: "Parses code diffs and provides structured review feedback".into(),
            trigger: SkillTrigger {
                sources: vec!["book".into(), "paper".into(), "code".into()],
                condition: Some("git.pre_commit".into()),
                frequency: 5,
            },
            io: SkillIO {
                input_type: "CodeDiff".into(),
                output_type: "ReviewResult".into(),
                input_dims: 4096,
                output_dims: 1024,
            },
            e8_mode: None,
            quality_threshold: 0.75,
            dependencies: vec!["syntax_parser".into(), "pattern_matcher".into()],
        }
    }

    #[test]
    fn test_scan_and_validate_skills() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let file_path = dir.path().join("code_review.skill.json");
        let def = sample_skill();
        let json = serde_json::to_string_pretty(&def).expect("serialization should succeed");
        std::fs::write(&file_path, &json).expect("write should succeed");

        let skills = SkillDocLoader::scan_skills(dir.path()).expect("scan should succeed");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "code_review_syntax_parser");

        let loader = SkillDocLoader;
        let warnings = loader.validate(&skills[0]);
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {:?}",
            warnings
        );
    }
}
