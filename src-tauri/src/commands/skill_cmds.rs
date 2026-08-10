use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Clone, Debug)]
pub struct SkillInfo {
    pub name: String,
    pub path: String,
    pub description: String,
    pub line_count: usize,
    pub domain: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct SkillListResult {
    pub skills: Vec<SkillInfo>,
    pub total: usize,
}

fn skills_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let mut p = PathBuf::from(home);
    p.push(".agents");
    p.push("skills");
    p
}

fn scan_skills() -> Vec<SkillInfo> {
    let base = skills_dir();
    if !base.exists() {
        return Vec::new();
    }

    let mut skills = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Read SKILL.md for description
            let skill_md = path.join("SKILL.md");
            let description = if skill_md.exists() {
                std::fs::read_to_string(&skill_md)
                    .ok()
                    .and_then(|c| c.lines().find_map(|l| {
                        let t = l.trim();
                        if !t.is_empty() && !t.starts_with('#') && !t.starts_with("//") { Some(t.to_string()) } else { None }
                    }))
                    .unwrap_or_default()
            } else {
                String::new()
            };

            // Count total lines
            let line_count = count_lines_in_dir(&path);

            // Determine domain from path
            let domain = path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("general")
                .to_string();

            skills.push(SkillInfo {
                name,
                path: path.to_string_lossy().into(),
                description,
                line_count,
                domain,
            });
        }
    }
    skills
}

fn count_lines_in_dir(dir: &std::path::Path) -> usize {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += count_lines_in_dir(&path);
            } else if path.extension().map_or(false, |e| e == "rs" || e == "md" || e == "ts" || e == "js" || e == "py" || e == "sh" || e == "toml" || e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    total += content.lines().count();
                }
            }
        }
    }
    total
}

#[tauri::command]
pub fn skill_list() -> Result<SkillListResult, String> {
    let skills = scan_skills();
    let total = skills.len();
    Ok(SkillListResult { skills, total })
}

#[tauri::command]
pub fn skill_get(name: String) -> Result<SkillInfo, String> {
    let skills = scan_skills();
    skills.into_iter().find(|s| s.name == name)
        .ok_or_else(|| format!("Skill '{}' not found", name))
}

#[tauri::command]
pub fn skill_read(name: String, file: String) -> Result<String, String> {
    let base = skills_dir();
    let file_path = base.join(&name).join(&file);
    if !file_path.exists() {
        return Err(format!("File {} not found in skill {}", file, name));
    }
    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read: {}", e))
}

#[tauri::command]
pub fn skill_search(query: String) -> Result<Vec<SkillInfo>, String> {
    let q = query.to_lowercase();
    let skills = scan_skills();
    Ok(skills.into_iter()
        .filter(|s| s.name.to_lowercase().contains(&q) || s.description.to_lowercase().contains(&q))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_dir_default() {
        let dir = skills_dir();
        assert!(dir.to_string_lossy().contains(".agents/skills"));
    }

    #[test]
    fn test_skill_list_empty_or_not() {
        let result = skill_list().unwrap();
        // May be empty if no ~/.agents/skills dir, but should not error
        assert!(result.total >= 0);
    }

    #[test]
    fn test_search_empty() {
        let results = skill_search("nonexistent-xyz-789".into()).unwrap();
        assert!(results.is_empty());
    }
}
