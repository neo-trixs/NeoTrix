use std::path::Path;

use crate::core::nt_core_util;

/// A skill discovered on disk (SKILL.md or .skill.json)
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub name: String,
    pub path: String,
    pub source: crate::neotrix::nt_io_plugin::PluginSource,
}

/// Scan standard paths for discoverable skills
pub fn discover_skills_on_disk() -> Vec<DiscoveredSkill> {
    let mut skills = Vec::new();

    // Scan workspace skills/ directory
    let workspace_skills = Path::new("skills");
    if workspace_skills.exists() {
        if let Ok(entries) = std::fs::read_dir(workspace_skills) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    // Check for .skill.json files directly in skills/
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if path.extension().map(|e| e == "json").unwrap_or(false) {
                            skills.push(DiscoveredSkill {
                                name: name.to_string(),
                                path: path.to_string_lossy().to_string(),
                                source: crate::neotrix::nt_io_plugin::PluginSource::SkillJson,
                            });
                        }
                    }
                    continue;
                }
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let has_skill_md = path.join("SKILL.md").exists();
                if has_skill_md {
                    skills.push(DiscoveredSkill {
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        source: crate::neotrix::nt_io_plugin::PluginSource::SkillMd,
                    });
                } else if path.join(format!("{}.skill.json", name)).exists() {
                    skills.push(DiscoveredSkill {
                        name,
                        path: path.to_string_lossy().to_string(),
                        source: crate::neotrix::nt_io_plugin::PluginSource::SkillJson,
                    });
                }
            }
        }
    }

    // Scan home ~/.neotrix/skills/
    let home = nt_core_util::home_dir().to_string_lossy().to_string();
    let home_skills = std::path::PathBuf::from(&home)
        .join(".neotrix")
        .join("skills");
    if home_skills.exists() {
        if let Ok(entries) = std::fs::read_dir(&home_skills) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if skills.iter().any(|s| s.name == name) {
                        continue;
                    }
                    let has_skill_md = path.join("SKILL.md").exists();
                    if has_skill_md {
                        skills.push(DiscoveredSkill {
                            name,
                            path: path.to_string_lossy().to_string(),
                            source: crate::neotrix::nt_io_plugin::PluginSource::SkillMd,
                        });
                    }
                }
            }
        }
    }

    skills
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_no_crash() {
        let skills = discover_skills_on_disk();
        // len() is usize — always >= 0; just verify no panic
        let _ = skills.len();
        let _count = skills.len();
    }

    #[test]
    fn test_discovered_skill_fields() {
        let skill = DiscoveredSkill {
            name: "test".into(),
            path: "/tmp/test".into(),
            source: crate::neotrix::nt_io_plugin::PluginSource::SkillMd,
        };
        assert_eq!(skill.name, "test");
    }
}
