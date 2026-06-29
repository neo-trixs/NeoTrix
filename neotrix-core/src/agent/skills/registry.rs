use super::types::{Skill, SkillMeta, SkillSource};
use crate::core::nt_core_util;
use std::collections::HashMap;

// ==============================
//  1. Discovery — 发现与加载
// ==============================

pub struct SkillDiscovery {
    /// 已发现的 Skill 索引
    skills: HashMap<String, Skill>,
    /// 搜索路径
    pub(crate) search_paths: Vec<String>,
}

impl Default for SkillDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillDiscovery {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            search_paths: vec![
                "./skills/".to_string(),
                "~/.neotrix/skills/".to_string(),
                "/usr/local/share/neotrix/skills/".to_string(),
                // ECC 兼容路径
                "~/.claude/skills/".to_string(),
                "~/.opencode/skills/".to_string(),
            ],
        }
    }

    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.meta.name.clone(), skill);
    }

    pub fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(path.to_string());
    }

    /// 发现全部来源（本地 + 已注册）
    pub fn discover_all(&mut self) -> Vec<String> {
        self.discover_local()
    }

    /// 从本地目录发现 Skills
    pub fn discover_local(&mut self) -> Vec<String> {
        let mut found = Vec::new();
        for path in &self.search_paths {
            let expanded =
                path.replace("~", &nt_core_util::home_dir().to_string_lossy().to_string());
            if let Ok(entries) = std::fs::read_dir(&expanded) {
                for entry in entries.flatten() {
                    let fpath = entry.path();
                    if fpath.is_dir() {
                        let skill_file = fpath.join("SKILL.md");
                        if skill_file.exists() {
                            if let Some(name) = fpath.file_name().and_then(|n| n.to_str()) {
                                let content =
                                    std::fs::read_to_string(&skill_file).unwrap_or_default();
                                let meta = Self::parse_frontmatter(&content);
                                let system_prompt = Self::extract_system_prompt(&content);
                                self.skills.insert(
                                    name.to_string(),
                                    Skill::new(
                                        meta,
                                        SkillSource::LocalDir(fpath.to_string_lossy().to_string()),
                                        content.clone(),
                                        system_prompt,
                                    ),
                                );
                                found.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        found
    }

    /// 从 GitHub 发现 Skill（ECC 兼容：raw.githubusercontent.com）
    pub fn discover_github(
        &mut self,
        owner: &str,
        repo: &str,
        path: &str,
        branch: Option<&str>,
    ) -> Result<String, String> {
        let name = format!("{}/{}/{}", owner, repo, path);
        let branch = branch.unwrap_or("main");
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, path
        );

        match reqwest::blocking::get(&url) {
            Ok(response) => {
                let content = response.text().map_err(|e| format!("read error: {}", e))?;
                let meta = Self::parse_frontmatter(&content);
                let system_prompt = Self::extract_system_prompt(&content);
                self.skills.insert(
                    name.clone(),
                    Skill::new(
                        meta,
                        SkillSource::GitHub {
                            owner: owner.to_string(),
                            repo: repo.to_string(),
                            path: path.to_string(),
                            branch: Some(branch.to_string()),
                        },
                        content.clone(),
                        system_prompt,
                    ),
                );
                Ok(name)
            }
            Err(e) => Err(format!("GitHub fetch failed for {}: {}", url, e)),
        }
    }

    /// 从 ECC 社区注册表发现 Skill
    pub fn discover_ecc_community(
        &mut self,
        skill_id: &str,
        version: &str,
    ) -> Result<String, String> {
        let url = format!("https://raw.githubusercontent.com/affaan-m/everything-claude-code/main/skills/{}/SKILL.md", skill_id);
        match reqwest::blocking::get(&url) {
            Ok(response) => {
                let content = response.text().map_err(|e| format!("read error: {}", e))?;
                let meta = Self::parse_frontmatter(&content);
                let system_prompt = Self::extract_system_prompt(&content);
                let name = format!("ecc/{}/v{}", skill_id, version);
                self.skills.insert(
                    name.clone(),
                    Skill::new(
                        meta,
                        SkillSource::EccCommunity {
                            skill_id: skill_id.to_string(),
                            version: version.to_string(),
                        },
                        content,
                        system_prompt,
                    ),
                );
                Ok(name)
            }
            Err(e) => Err(format!("ECC community fetch failed: {}", e)),
        }
    }

    /// 解析 Frontmatter（ECC 兼容 --- 分隔的简易 YAML）
    ///
    /// ECC 格式示例：
    /// ---
    /// name: agent-harness-construction
    /// description: Design and optimize AI agent action spaces...
    /// origin: ECC
    /// ---
    pub(crate) fn parse_frontmatter(content: &str) -> SkillMeta {
        let mut meta = SkillMeta {
            name: "unknown".to_string(),
            description: String::new(),
            version: "0.1.0".to_string(),
            author: None,
            origin: None,
            triggers: Vec::new(),
            condition: None,
            requires_tools: Vec::new(),
            requires_capabilities: Vec::new(),
            mitre_attack_ids: Vec::new(),
        };

        if !content.starts_with("---") {
            return meta;
        }

        let after_first = &content[3..];
        if let Some(end) = after_first.find("---") {
            let front = after_first[..end].trim();
            for line in front.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(idx) = line.find(':') {
                    let key = line[..idx].trim().to_lowercase();
                    let val = line[idx + 1..].trim().to_string();
                    match key.as_str() {
                        "name" => meta.name = val,
                        "description" => meta.description = val,
                        "version" => meta.version = val,
                        "author" => meta.author = Some(val),
                        "origin" => meta.origin = Some(val),
                        "trigger" | "triggers" => {
                            meta.triggers = val.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "condition" => meta.condition = Some(val),
                        "requires_tools" | "requires-tools" => {
                            meta.requires_tools =
                                val.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "requires_capabilities" | "requires-capabilities" => {
                            meta.requires_capabilities =
                                val.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "mitre_attack" | "mitre" | "technique" => {
                            meta.mitre_attack_ids =
                                val.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        _ => {}
                    }
                }
            }
        }

        meta
    }

    /// 提取 system prompt（Frontmatter 之后的第一个段落，ECC 兼容）
    pub(crate) fn extract_system_prompt(content: &str) -> String {
        if let Some(rest) = content.strip_prefix("---") {
            if let Some(end) = rest.find("---") {
                let after = &rest[end + 3..];
                return after.trim().lines().take(10).collect::<Vec<_>>().join("\n");
            }
        }
        content
            .trim()
            .lines()
            .take(10)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

// ==============================
//  2. Injector — 注入到 Prompt
// ==============================

pub struct SkillInjector;

impl SkillInjector {
    /// 将匹配的 Skills 注入到 system prompt
    pub fn inject(skills: &[&Skill], base_prompt: &str) -> String {
        if skills.is_empty() {
            return base_prompt.to_string();
        }

        let mut prompt = base_prompt.to_string();
        prompt.push_str("\n\n## 可用 Skills\n\n");

        for skill in skills {
            prompt.push_str(&format!(
                "### {}\n{}\n{}\n\n",
                skill.meta.name, skill.meta.description, skill.system_prompt
            ));
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_discovery_new() {
        let d = SkillDiscovery::new();
        assert!(d.is_empty());
    }

    #[test]
    fn test_skill_discovery_add_skill() {
        let mut d = SkillDiscovery::new();
        let meta = SkillMeta {
            name: "my-skill".into(),
            description: "desc".into(),
            version: "1.0".into(),
            author: None,
            origin: None,
            triggers: vec![],
            condition: None,
            requires_tools: vec![],
            requires_capabilities: vec![],
            mitre_attack_ids: vec![],
        };
        let skill = Skill::new(
            meta,
            SkillSource::LocalDir("./".into()),
            "content".into(),
            "prompt".into(),
        );
        d.add_skill(skill);
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn test_skill_discovery_get() {
        let mut d = SkillDiscovery::new();
        let meta = SkillMeta {
            name: "my-skill".into(),
            description: "desc".into(),
            version: "1.0".into(),
            author: None,
            origin: None,
            triggers: vec![],
            condition: None,
            requires_tools: vec![],
            requires_capabilities: vec![],
            mitre_attack_ids: vec![],
        };
        let skill = Skill::new(
            meta,
            SkillSource::LocalDir("./".into()),
            "content".into(),
            "prompt".into(),
        );
        d.add_skill(skill);
        assert!(d.get("my-skill").is_some());
    }

    #[test]
    fn test_parse_frontmatter_full() {
        let content = r#"---
name: test-skill
description: A test skill
version: 2.0.0
author: neotrix
trigger: test, demo
mitre: T1595, T1046
condition: filetype:rust
---
Skill content here"#;
        let meta = SkillDiscovery::parse_frontmatter(content);
        assert_eq!(meta.name, "test-skill");
        assert_eq!(meta.author, Some("neotrix".into()));
        assert_eq!(meta.triggers.len(), 2);
        assert_eq!(meta.mitre_attack_ids.len(), 2);
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "just content without frontmatter";
        let meta = SkillDiscovery::parse_frontmatter(content);
        assert_eq!(meta.name, "unknown");
    }

    #[test]
    fn test_parse_frontmatter_empty() {
        let meta = SkillDiscovery::parse_frontmatter("");
        assert_eq!(meta.name, "unknown");
    }

    #[test]
    fn test_parse_frontmatter_partial() {
        let content = r#"---
name: partial-skill
---"#;
        let meta = SkillDiscovery::parse_frontmatter(content);
        assert_eq!(meta.name, "partial-skill");
    }

    #[test]
    fn test_extract_system_prompt_with_frontmatter() {
        let content = r#"---
name: test
---
System prompt here
Second line"#;
        let prompt = SkillDiscovery::extract_system_prompt(content);
        assert!(prompt.contains("System prompt here"));
    }

    #[test]
    fn test_extract_system_prompt_no_frontmatter() {
        let content = "Direct content\nSecond line";
        let prompt = SkillDiscovery::extract_system_prompt(content);
        assert_eq!(prompt, "Direct content\nSecond line");
    }

    #[test]
    fn test_skill_injector_inject_empty() {
        let result = SkillInjector::inject(&[], "base prompt");
        assert_eq!(result, "base prompt");
    }

    #[test]
    fn test_skill_injector_inject_with_skills() {
        let meta = SkillMeta {
            name: "test".into(),
            description: "Test skill".into(),
            version: "1.0".into(),
            author: None,
            origin: None,
            triggers: vec![],
            condition: None,
            requires_tools: vec![],
            requires_capabilities: vec![],
            mitre_attack_ids: vec![],
        };
        let skill = Skill::new(
            meta,
            SkillSource::LocalDir("./".into()),
            "content".into(),
            "system prompt".into(),
        );
        let result = SkillInjector::inject(&[&skill], "base");
        assert!(result.contains("Test skill"));
        assert!(result.contains("system prompt"));
    }

    #[test]
    fn test_discovery_default_search_paths() {
        let d = SkillDiscovery::new();
        assert_eq!(d.search_paths.len(), 5);
    }
}
