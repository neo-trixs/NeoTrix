use crate::neotrix::nt_mind::self_iterating::pipeline::PermissionLevel;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub version: String,
    pub trigger_words: Vec<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub permission_level: PermissionLevel,
    pub min_core_version: Option<String>,
    pub script_paths: Vec<String>,
    pub reference_paths: Vec<String>,
}

impl SkillManifest {
    pub fn from_skill_md(content: &str) -> Result<Self, Vec<String>> {
        let trimmed = content.trim();
        if !trimmed.starts_with("---") {
            return Err(vec!["Missing opening --- frontmatter delimiter".to_string()]);
        }

        let body = &trimmed[3..];
        let end = body
            .find("---")
            .ok_or_else(|| vec!["Missing closing --- frontmatter delimiter".to_string()])?;

        let frontmatter = &body[..end];
        let kv = parse_frontmatter(frontmatter)?;

        let name = kv.get("name").cloned().unwrap_or_default();
        let description = kv.get("description").cloned().unwrap_or_default();
        let author = kv.get("author").cloned();
        let version = kv
            .get("version")
            .cloned()
            .unwrap_or_else(|| "0.1.0".to_string());
        let trigger_words = parse_array_field(&kv, "trigger_words");
        let tags = parse_array_field(&kv, "tags");
        let dependencies = parse_array_field(&kv, "dependencies");
        let script_paths = parse_array_field(&kv, "script_paths");
        let reference_paths = parse_array_field(&kv, "reference_paths");
        let min_core_version = kv.get("min_core_version").cloned();

        let permission_level = match kv
            .get("permission_level")
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("full") => PermissionLevel::Full,
            Some("suggest") => PermissionLevel::Suggest,
            Some("review") => PermissionLevel::Review,
            _ => PermissionLevel::Suggest,
        };

        Ok(Self {
            name,
            description,
            author,
            version,
            trigger_words,
            tags,
            dependencies,
            permission_level,
            min_core_version,
            script_paths,
            reference_paths,
        })
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("Skill name cannot be empty".to_string());
        }
        if self.trigger_words.is_empty() && self.tags.is_empty() {
            errors.push("Skill must have at least one trigger_word or tag".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn matches_trigger(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.trigger_words
            .iter()
            .any(|tw| lower.contains(&tw.to_lowercase()))
    }
}

fn parse_frontmatter(frontmatter: &str) -> Result<HashMap<String, String>, Vec<String>> {
    let mut map = HashMap::new();
    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_string();
            let value = line[idx + 1..].trim().to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }
    Ok(map)
}

fn parse_array_field(kv: &HashMap<String, String>, key: &str) -> Vec<String> {
    kv.get(key).map_or_else(Vec::new, |raw| {
        let raw = raw.trim();
        if raw.starts_with('[') && raw.ends_with(']') {
            let inner = &raw[1..raw.len() - 1];
            inner
                .split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if raw.is_empty() {
            Vec::new()
        } else {
            vec![raw.trim_matches('"').trim_matches('\'').to_string()]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_skill_md() {
        let content = r#"---
name: my-skill
description: Does X
version: 1.0.0
author: NeoTrix
trigger_words: ["deploy", "release"]
tags: ["devops", "ci"]
permission_level: Full
---"#;
        let skill = SkillManifest::from_skill_md(content).expect("should parse ok");
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.version, "1.0.0");
        assert_eq!(skill.author, Some("NeoTrix".to_string()));
        assert_eq!(skill.trigger_words, vec!["deploy", "release"]);
        assert_eq!(skill.tags, vec!["devops", "ci"]);
        assert_eq!(skill.permission_level, PermissionLevel::Full);
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "no frontmatter here";
        let result = SkillManifest::from_skill_md(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_name() {
        let skill = SkillManifest {
            name: "".to_string(),
            description: "test".to_string(),
            author: None,
            version: "0.1.0".to_string(),
            trigger_words: vec!["go".to_string()],
            tags: vec![],
            dependencies: vec![],
            permission_level: PermissionLevel::Suggest,
            min_core_version: None,
            script_paths: vec![],
            reference_paths: vec![],
        };
        let result = skill.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("name cannot be empty")));
    }

    #[test]
    fn test_validate_no_triggers_or_tags() {
        let skill = SkillManifest {
            name: "silent".to_string(),
            description: "no triggers".to_string(),
            author: None,
            version: "0.1.0".to_string(),
            trigger_words: vec![],
            tags: vec![],
            dependencies: vec![],
            permission_level: PermissionLevel::Suggest,
            min_core_version: None,
            script_paths: vec![],
            reference_paths: vec![],
        };
        let result = skill.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_matches_trigger_exact() {
        let skill = SkillManifest {
            name: "greeter".to_string(),
            description: "".to_string(),
            author: None,
            version: "0.1.0".to_string(),
            trigger_words: vec!["hello".to_string(), "greet".to_string()],
            tags: vec![],
            dependencies: vec![],
            permission_level: PermissionLevel::Suggest,
            min_core_version: None,
            script_paths: vec![],
            reference_paths: vec![],
        };
        assert!(skill.matches_trigger("hello"));
        assert!(skill.matches_trigger("please greet me"));
    }

    #[test]
    fn test_matches_trigger_case_insensitive() {
        let skill = SkillManifest {
            name: "deploy".to_string(),
            description: "".to_string(),
            author: None,
            version: "0.1.0".to_string(),
            trigger_words: vec!["Deploy".to_string()],
            tags: vec![],
            dependencies: vec![],
            permission_level: PermissionLevel::Suggest,
            min_core_version: None,
            script_paths: vec![],
            reference_paths: vec![],
        };
        assert!(skill.matches_trigger("DEPLOY now"));
    }

    #[test]
    fn test_no_match() {
        let skill = SkillManifest {
            name: "deploy".to_string(),
            description: "".to_string(),
            author: None,
            version: "0.1.0".to_string(),
            trigger_words: vec!["deploy".to_string()],
            tags: vec![],
            dependencies: vec![],
            permission_level: PermissionLevel::Suggest,
            min_core_version: None,
            script_paths: vec![],
            reference_paths: vec![],
        };
        assert!(!skill.matches_trigger("build the project"));
    }
}
