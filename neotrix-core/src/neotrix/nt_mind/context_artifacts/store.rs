use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use super::types::{Artifact, ArtifactType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStore {
    artifacts: HashMap<String, Artifact>,
}

impl Default for ArtifactStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactStore {
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            artifacts: HashMap::with_capacity(cap),
        }
    }

    pub fn store(&mut self, artifact: Artifact) {
        self.artifacts.insert(artifact.name.clone(), artifact);
    }

    pub fn get(&self, name: &str) -> Option<&Artifact> {
        self.artifacts.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Artifact> {
        self.artifacts.remove(name)
    }

    pub fn all(&self) -> Vec<&Artifact> {
        self.artifacts.values().collect()
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&Artifact> {
        let tag_lower = tag.to_lowercase();
        self.artifacts
            .values()
            .filter(|a| a.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    pub fn search_by_tags(&self, tags: &[&str]) -> Vec<&Artifact> {
        let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
        self.artifacts
            .values()
            .filter(|a| {
                let artifact_tags: Vec<String> = a.tags.iter().map(|t| t.to_lowercase()).collect();
                tags_lower.iter().all(|t| artifact_tags.contains(t))
            })
            .collect()
    }

    pub fn search_keyword(&self, keyword: &str) -> Vec<&Artifact> {
        let kw = keyword.to_lowercase();
        self.artifacts
            .values()
            .filter(|a| {
                a.name.to_lowercase().contains(&kw)
                    || a.content.to_lowercase().contains(&kw)
                    || a.tags.iter().any(|t| t.to_lowercase().contains(&kw))
                    || a.source_path
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&kw)
            })
            .collect()
    }

    pub fn search_by_type(&self, artifact_type: ArtifactType) -> Vec<&Artifact> {
        self.artifacts
            .values()
            .filter(|a| a.artifact_type == artifact_type)
            .collect()
    }

    pub fn search(
        &self,
        artifact_type: Option<ArtifactType>,
        keyword: Option<&str>,
        tag: Option<&str>,
    ) -> Vec<&Artifact> {
        let mut results: Vec<&Artifact> = self.artifacts.values().collect();

        if let Some(t) = artifact_type {
            results.retain(|a| a.artifact_type == t);
        }
        if let Some(kw) = keyword {
            let kw_lower = kw.to_lowercase();
            results.retain(|a| {
                a.name.to_lowercase().contains(&kw_lower)
                    || a.content.to_lowercase().contains(&kw_lower)
                    || a.tags.iter().any(|t| t.to_lowercase().contains(&kw_lower))
            });
        }
        if let Some(tag) = tag {
            let tag_lower = tag.to_lowercase();
            results.retain(|a| a.tags.iter().any(|t| t.to_lowercase() == tag_lower));
        }

        results
    }

    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

pub struct ArtifactBuilder;

impl ArtifactBuilder {
    pub fn parse_sql(name: &str, content: &str, source: Option<&str>) -> Option<Artifact> {
        let re = match Regex::new(
            r"(?i)CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(?:\w+\s*\.\s*)?(\w+)\s*\(([^;]*?)\);?",
        ) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[store] regex error: {}", e);
                return None;
            }
        };
        let mut summary = String::new();
        let mut tables_found = 0;
        let col_re = Regex::new(r"^\s*`?(\w+)`?\s+(\w+(?:\s*\([^)]*\))?)").ok()?;

        for cap in re.captures_iter(content) {
            tables_found += 1;
            let table_name = cap.get(1)?.as_str();
            let columns_block = cap.get(2)?.as_str();
            summary.push_str(&format!("TABLE {}\n", table_name));

            for line in columns_block.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("--") || line.starts_with("/*") {
                    continue;
                }
                if let Some(col_cap) = col_re.captures(line) {
                    let col_name = col_cap.get(1)?.as_str();
                    let col_type = col_cap.get(2)?.as_str();
                    summary.push_str(&format!("  {}: {}\n", col_name, col_type));
                }
            }
            summary.push('\n');
        }

        if tables_found == 0 {
            return None;
        }

        let mut artifact = Artifact::new(name, ArtifactType::DatabaseSchema, summary.trim());
        artifact.tags = vec!["database".to_string(), "sql".to_string()];
        artifact.source_path = source.map(|s| s.to_string());
        Some(artifact)
    }

    pub fn parse_openapi_yaml(name: &str, content: &str, source: Option<&str>) -> Option<Artifact> {
        let mut summary = String::new();
        let mut endpoints_found = 0;

        if let Some(ver) = Self::extract_yaml_value(content, "openapi")
            .or_else(|| Self::extract_yaml_value(content, "swagger"))
        {
            summary.push_str(&format!("OpenAPI version: {}\n\n", ver));
        }

        if let Some(title) = Self::extract_yaml_value(content, "info.title") {
            summary.push_str(&format!("API: {}\n", title));
        }
        if let Some(desc) = Self::extract_yaml_value(content, "info.description") {
            summary.push_str(&format!("Description: {}\n", desc));
        }
        summary.push('\n');

        let method_re = match Regex::new(r"(?m)^\s{4}(get|post|put|delete|patch|head|options):\s*$")
        {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[store] regex error: {}", e);
                return None;
            }
        };
        let mut current_path = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('/') && trimmed.ends_with(':') {
                current_path = trimmed.trim_end_matches(':').to_string();
            } else if let Some(method) = Self::match_http_method(trimmed) {
                endpoints_found += 1;
                summary.push_str(&format!("{} {}\n", method.to_uppercase(), current_path));
            }
        }

        if endpoints_found == 0 {
            for _cap in method_re.captures_iter(content) {
                endpoints_found += 1;
            }
            if endpoints_found > 0 {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('/') {
                        current_path = trimmed
                            .trim_end_matches(':')
                            .trim_end_matches(':')
                            .to_string();
                        if current_path.len() < trimmed.len() {
                            endpoints_found += 1;
                            summary.push_str(&format!("{} {}\n", "GET", current_path));
                        }
                    }
                }
            }
        }

        if endpoints_found == 0 {
            return None;
        }

        let mut artifact = Artifact::new(name, ArtifactType::ApiSpec, summary.trim());
        artifact.tags = vec!["api".to_string(), "openapi".to_string(), "spec".to_string()];
        artifact.source_path = source.map(|s| s.to_string());
        Some(artifact)
    }

    pub fn parse_markdown(name: &str, content: &str, source: Option<&str>) -> Artifact {
        let mut summary = String::new();
        let heading_re = Regex::new(r"(?m)^(#{1,6})\s+(.+)$").expect("hardcoded heading regex");
        let total_lines = content.lines().count();

        summary.push_str("# Document Structure\n\n");
        for cap in heading_re.captures_iter(content) {
            let level = cap.get(1).expect("heading level capture").as_str().len();
            let heading = cap.get(2).expect("heading text capture").as_str().trim();
            let indent = "  ".repeat(level - 1);
            summary.push_str(&format!("{}- {}\n", indent, heading));
        }
        summary.push('\n');

        let mut in_section = false;
        let mut section_name = String::new();
        let mut section_body = String::new();
        for line in content.lines() {
            if line.starts_with("## ") {
                if in_section && !section_body.trim().is_empty() {
                    let preview: String = section_body.trim().chars().take(200).collect();
                    summary.push_str(&format!("## {}\n{}\n\n", section_name, preview));
                }
                section_name = line.trim_start_matches("## ").trim().to_string();
                section_body = String::new();
                in_section = true;
            } else if in_section {
                section_body.push_str(line);
                section_body.push('\n');
            }
        }
        if in_section && !section_body.trim().is_empty() {
            let preview: String = section_body.trim().chars().take(200).collect();
            summary.push_str(&format!("## {}\n{}\n\n", section_name, preview));
        }

        summary.push_str(&format!("*Total lines: {}*\n", total_lines));

        let mut artifact = Artifact::new(name, ArtifactType::ArchitectureDoc, summary.trim());
        artifact.tags = vec![
            "documentation".to_string(),
            "architecture".to_string(),
            "markdown".to_string(),
        ];
        artifact.source_path = source.map(|s| s.to_string());
        artifact
    }

    pub fn parse_config(name: &str, content: &str, format: &str, source: Option<&str>) -> Artifact {
        let mut summary = String::new();
        let format_upper = format.to_uppercase();
        summary.push_str(&format!("# {} Configuration\n\n", format_upper));

        let section_re = Regex::new(r"(?m)^(\w[\w_-]*):\s*$").expect("hardcoded section regex");
        let kv_re = Regex::new(r"(?m)^\s*(\w[\w_-]*):\s*(.+)$").expect("hardcoded kv regex");

        if format_upper == "YAML" || format_upper == "YML" {
            if let Ok(toml_section_re) = Regex::new(r"(?m)^\[(\w[\w_.-]*)\]$") {
                let has_toml_sections = toml_section_re.is_match(content);
                for cap in section_re.captures_iter(content) {
                    let section = cap.get(1).expect("section name capture").as_str();
                    summary.push_str(&format!("[{}]\n", section));
                }
                for cap in kv_re.captures_iter(content) {
                    let key = cap.get(1).expect("yaml kv key capture").as_str();
                    let value = cap.get(2).expect("yaml kv value capture").as_str().trim();
                    if !value.is_empty() && !value.starts_with('#') {
                        summary.push_str(&format!("  {} = {}\n", key, value));
                    }
                }
                if !has_toml_sections {
                    summary.clear();
                    summary.push_str(&format!("# {} Configuration\n\n", format_upper));
                    let yaml_kv = Regex::new(r"(?m)^\s*([a-zA-Z_][\w_-]*)\s*:\s*(.+)$")
                        .expect("hardcoded yaml kv regex");
                    for cap in yaml_kv.captures_iter(content) {
                        let key = cap.get(1).expect("yaml kv key capture").as_str();
                        let value = cap.get(2).expect("yaml kv value capture").as_str().trim();
                        if !value.starts_with('#') {
                            summary.push_str(&format!("  {} = {}\n", key, value));
                        }
                    }
                }
            }
        } else if format_upper == "TOML" {
            let toml_section_re =
                Regex::new(r"(?m)^\[(\w[\w_.-]*)\]$").expect("hardcoded toml section regex");
            let toml_kv_re =
                Regex::new(r"(?m)^(\w[\w_-]*)\s*=\s*(.+)$").expect("hardcoded toml kv regex");
            for cap in toml_section_re.captures_iter(content) {
                let section = cap.get(1).expect("toml section name capture").as_str();
                summary.push_str(&format!("[{}]\n", section));
            }
            for cap in toml_kv_re.captures_iter(content) {
                let key = cap.get(1).expect("toml key capture").as_str();
                let value = cap.get(2).expect("toml value capture").as_str().trim();
                summary.push_str(&format!("  {} = {}\n", key, value));
            }
        } else {
            summary = content.to_string();
        }

        let mut artifact = Artifact::new(name, ArtifactType::ConfigFile, summary.trim());
        artifact.tags = vec!["config".to_string(), format.to_lowercase()];
        artifact.source_path = source.map(|s| s.to_string());
        artifact
    }

    pub fn build_from_file(name: &str, path: &str) -> Option<Artifact> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[store] failed to read '{}': {}", path, e);
                return None;
            }
        };
        let path_lower = path.to_lowercase();

        if path_lower.ends_with(".sql") {
            Self::parse_sql(name, &content, Some(path))
        } else if path_lower.ends_with(".yaml") || path_lower.ends_with(".yml") {
            if content.contains("openapi:")
                || content.contains("swagger:")
                || content.contains("\"/")
            {
                Self::parse_openapi_yaml(name, &content, Some(path))
            } else {
                Some(Self::parse_config(name, &content, "yaml", Some(path)))
            }
        } else if path_lower.ends_with(".toml") {
            Some(Self::parse_config(name, &content, "toml", Some(path)))
        } else if path_lower.ends_with(".md") {
            Some(Self::parse_markdown(name, &content, Some(path)))
        } else {
            None
        }
    }

    fn extract_yaml_value(content: &str, dotted_key: &str) -> Option<String> {
        let parts: Vec<&str> = dotted_key.split('.').collect();
        if parts.len() == 1 {
            let re = match Regex::new(&format!(
                r#"(?m)^\s*{}\s*:\s*['"]?(.+?)['"]?\s*$"#,
                regex::escape(parts[0])
            )) {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("[store] regex error: {}", e);
                    return None;
                }
            };
            return re
                .captures(content)?
                .get(1)
                .map(|m| m.as_str().trim().to_string());
        }
        let mut depth = 0;
        let mut target = None;
        for line in content.lines() {
            let indent = line.len() - line.trim_start().len();
            let trimmed = line.trim();
            if depth < parts.len() && indent == depth * 2 && trimmed.starts_with(parts[depth]) {
                if depth == parts.len() - 1 {
                    if let Some(val_start) = trimmed.find(':') {
                        let val = trimmed[val_start + 1..]
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        if !val.is_empty() {
                            target = Some(val);
                        }
                    }
                }
                depth += 1;
            }
        }
        target
    }

    fn match_http_method(s: &str) -> Option<&'static str> {
        match s {
            "get:" => Some("get"),
            "post:" => Some("post"),
            "put:" => Some("put"),
            "delete:" => Some("delete"),
            "patch:" => Some("patch"),
            "head:" => Some("head"),
            "options:" => Some("options"),
            _ => None,
        }
    }
}
