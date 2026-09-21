//! Skill domain plugin — 技能中心：扫描本地 SKILL.md 技能
//!
//! Skills are markdown files (`SKILL.md` or `*.md`) under
//! `{base_dir}/skills/`, with optional YAML frontmatter carrying
//! `name:` / `description:`. The plugin exposes list/get/search over
//! whatever is installed — empty dir yields an honest empty list.

use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct SkillPlugin;

impl SkillPlugin {
    pub fn new() -> Self {
        Self
    }

    fn skills_dir() -> Result<PathBuf, DomainError> {
        crate::config::AppConfig::base_dir()
            .map(|d| d.join("skills"))
            .ok_or_else(|| DomainError {
                code: "CONFIG_DIR_ERROR".into(),
                message: "Cannot determine base directory".into(),
                recoverable: false,
            })
    }

    /// Parse `name:`/`description:` from a YAML frontmatter block.
    /// Returns `(name, description)`; falls back to filename / empty string.
    fn parse_frontmatter(content: &str, fallback_name: &str) -> (String, String) {
        let mut name = fallback_name.to_string();
        let mut description = String::new();
        let trimmed = content.trim_start();
        if let Some(rest) = trimmed.strip_prefix("---") {
            if let Some(end) = rest.find("---") {
                for line in rest[..end].lines() {
                    let line = line.trim();
                    if let Some(v) = line.strip_prefix("name:").map(str::trim) {
                        if !v.is_empty() {
                            name = v.trim_matches('"').trim_matches('\'').to_string();
                        }
                    } else if let Some(v) = line.strip_prefix("description:").map(str::trim)
                    {
                        if !v.is_empty() {
                            description =
                                v.trim_matches('"').trim_matches('\'').to_string();
                        }
                    }
                }
            }
        }
        (name, description)
    }

    fn scan_skills() -> Result<Vec<serde_json::Value>, DomainError> {
        let dir = Self::skills_dir()?;
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        Self::scan_dir(&dir, &dir, &mut out);
        // Stable order for UI + tests.
        out.sort_by(|a, b| {
            a.get("name")
                .and_then(|v| v.as_str())
                .cmp(&b.get("name").and_then(|v| v.as_str()))
        });
        Ok(out)
    }

    fn scan_dir(root: &Path, dir: &Path, out: &mut Vec<serde_json::Value>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_dir(root, &path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("skill");
                let (name, description) = Self::parse_frontmatter(&content, stem);
                let domain = path
                    .parent()
                    .and_then(|p| {
                        p.strip_prefix(root)
                            .ok()
                            .and_then(|rel| rel.iter().next())
                            .and_then(|s| s.to_str())
                            .map(str::to_string)
                    })
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "general".into());
                out.push(serde_json::json!({
                    "name": if name == "SKILL" { stem.to_string() } else { name },
                    "path": path.to_string_lossy(),
                    "description": description,
                    "line_count": content.lines().count(),
                    "domain": domain,
                }));
            }
        }
    }
}

impl Default for SkillPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn err(code: &str, message: String) -> DomainError {
    DomainError {
        code: code.into(),
        message,
        recoverable: true,
    }
}

fn param(name: &str, r#type: &str, description: &str, optional: bool) -> ParamSpec {
    ParamSpec {
        name: name.into(),
        r#type: r#type.into(),
        description: description.into(),
        optional,
    }
}

#[async_trait]
impl DomainPlugin for SkillPlugin {
    fn name(&self) -> &str {
        "skill"
    }

    fn description(&self) -> &str {
        "技能中心：扫描本地技能、搜索、详情"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "list".into(),
                description: "列出已安装技能".into(),
                params: vec![],
                returns: "SkillListResult".into(),
            },
            ActionSpec {
                name: "get".into(),
                description: "获取技能详情（含正文）".into(),
                params: vec![param("name", "string", "技能名", false)],
                returns: "SkillInfo".into(),
            },
            ActionSpec {
                name: "search".into(),
                description: "按关键词搜索技能".into(),
                params: vec![param("query", "string", "关键词", false)],
                returns: "SkillInfo[]".into(),
            },
        ]
    }

    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "list" => {
                let skills = Self::scan_skills()?;
                let total = skills.len();
                Ok(serde_json::json!({
                    "skills": skills,
                    "total": total,
                }))
            }
            "get" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        err("INVALID_ARGS", "Missing required string argument: name".into())
                    })?;
                let skills = Self::scan_skills()?;
                let found = skills.into_iter().find(|s| {
                    s.get("name").and_then(|v| v.as_str()) == Some(name)
                });
                match found {
                    Some(mut skill) => {
                        // Attach full content for the detail view.
                        if let Some(path) = skill
                            .get("path")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                        {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Some(obj) = skill.as_object_mut() {
                                    obj.insert(
                                        "content".into(),
                                        serde_json::Value::String(content),
                                    );
                                }
                            }
                        }
                        Ok(skill)
                    }
                    None => Err(err(
                        "NOT_FOUND",
                        format!("Skill '{name}' not found"),
                    )),
                }
            }
            "search" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                let skills = Self::scan_skills()?;
                let hits: Vec<serde_json::Value> = skills
                    .into_iter()
                    .filter(|s| {
                        if query.is_empty() {
                            return true;
                        }
                        let haystack = format!(
                            "{} {}",
                            s.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            s.get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or(""),
                        )
                        .to_lowercase();
                        haystack.contains(&query)
                    })
                    .collect();
                Ok(serde_json::Value::Array(hits))
            }
            _ => Err(err(
                "UNKNOWN_ACTION",
                format!("Unknown skill action: {action}"),
            )),
        }
    }
}
