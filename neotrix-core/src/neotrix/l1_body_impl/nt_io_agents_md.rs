//! AGENTS.md / CLAUDE.md 行业标准读取器
//!
//! 检测并解析 AI 辅助工具的项目指令文件，
//! 提供结构化内容给 E8 推理引擎。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

const RULE_FILES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    ".cursorrules",
    ".github/copilot-instructions.md",
];

const CACHE_TTL: Duration = Duration::from_secs(60);
const LRU_MAX: usize = 16;

/// 项目规则 — 从 AGENTS.md/CLAUDE.md/.cursorrules 解析的结构化内容
#[derive(Debug, Clone)]
pub struct ProjectRules {
    pub source_files: Vec<PathBuf>,
    pub raw_text: String,
    pub sections: HashMap<String, String>,
    pub architecture: Vec<String>,
    pub conventions: Vec<String>,
    pub build_instructions: Vec<String>,
    pub test_instructions: Vec<String>,
    pub environment: Vec<String>,
    pub loaded_at: SystemTime,
}

impl ProjectRules {
    pub fn empty() -> Self {
        Self {
            source_files: vec![],
            raw_text: String::new(),
            sections: HashMap::new(),
            architecture: vec![],
            conventions: vec![],
            build_instructions: vec![],
            test_instructions: vec![],
            environment: vec![],
            loaded_at: SystemTime::now(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.source_files.is_empty()
    }
}

/// AGENTS.md / CLAUDE.md 读取器 — 发现、解析、缓存项目指令文件
pub struct AgentsMdReader {
    cache: Mutex<HashMap<PathBuf, (ProjectRules, SystemTime, HashMap<PathBuf, SystemTime>)>>,
}

impl Default for AgentsMdReader {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentsMdReader {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// 查找并解析项目目录中及父目录中的规则文件
    pub fn load_project_rules(&self, project_dir: &Path) -> Result<ProjectRules, String> {
        let canonical = Self::canonicalize(project_dir);
        let cache_key = canonical.clone();

        // 检查缓存（LRU + TTL）
        if let Ok(cache) = self.cache.lock() {
            if let Some((rules, loaded_at, _file_mtimes)) = cache.get(&cache_key) {
                if loaded_at.elapsed().unwrap_or(Duration::MAX) < CACHE_TTL
                    && !self._has_any_file_changed(project_dir, _file_mtimes)
                {
                    return Ok(rules.clone());
                }
            }
        }

        // 搜索规则文件
        let found = Self::discover_files(&canonical);
        if found.is_empty() {
            return Ok(ProjectRules::empty());
        }

        // 读取并合并所有文件
        let mut merged = ProjectRules::empty();
        let mut file_mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

        for path in &found {
            let content = std::fs::read_to_string(path.as_path())
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            if merged.source_files.is_empty() {
                merged.raw_text = content.clone();
            } else {
                merged.raw_text.push_str("\n\n");
                merged.raw_text.push_str(&content);
            }
            merged.source_files.push(path.clone());

            if let Ok(meta) = std::fs::metadata(path) {
                if let Ok(mtime) = meta.modified() {
                    file_mtimes.insert(path.clone(), mtime);
                }
            }
        }

        // 解析章节
        merged.sections = Self::parse_sections(&merged.raw_text);

        // 利用已知章节名填充结构化字段
        for (name, content) in &merged.sections {
            let lower = name.to_lowercase();
            if lower.contains("architect") || lower.contains("domain") || lower.contains("layer") {
                merged.architecture.push(format!("## {}\n{}", name, content));
            }
            if lower.contains("convention") || lower.contains("naming") || lower.contains("style") {
                merged.conventions.push(format!("## {}\n{}", name, content));
            }
            if lower.contains("build") || lower.contains("compile") {
                merged.build_instructions.push(format!("## {}\n{}", name, content));
            }
            if lower.contains("test") {
                merged.test_instructions.push(format!("## {}\n{}", name, content));
            }
            if lower.contains("env") || lower.contains("config") || lower.contains("variable") {
                merged.environment.push(format!("## {}\n{}", name, content));
            }
        }

        merged.loaded_at = SystemTime::now();

        // 写入缓存
        if let Ok(mut cache) = self.cache.lock() {
            if cache.len() >= LRU_MAX {
                // 移除最早条目
                let oldest = cache.keys().next().cloned();
                if let Some(k) = oldest {
                    cache.remove(&k);
                }
            }
            cache.insert(cache_key, (merged.clone(), SystemTime::now(), file_mtimes));
        }

        Ok(merged)
    }

    /// 获取指定章节内容
    pub fn get_section(&self, name: &str) -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        let rules = self.load_project_rules(&cwd).ok()?;
        rules.sections.get(name).cloned()
    }

    /// 获取完整规则文本，用于注入推理上下文
    pub fn get_context_text(&self) -> String {
        let cwd = match std::env::current_dir() {
            Ok(d) => d,
            Err(_) => return String::new(),
        };
        let rules = match self.load_project_rules(&cwd) {
            Ok(r) => r,
            Err(_) => return String::new(),
        };
        if rules.is_empty() {
            return String::new();
        }
        let sources: Vec<String> = rules
            .source_files
            .iter()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
            .collect();
        format!(
            "## Project Rules (from {})\n\n{}",
            sources.join(", "),
            rules.raw_text
        )
    }

    /// 检查自上次加载以来是否有文件变动
    pub fn has_changed(&self, project_dir: &Path) -> bool {
        let canonical = Self::canonicalize(project_dir);
        if let Ok(cache) = self.cache.lock() {
            if let Some((_, _, file_mtimes)) = cache.get(&canonical) {
                return self._has_any_file_changed(project_dir, file_mtimes);
            }
        }
        true
    }

    // ─── 内部辅助 ─────────────────────────────────────────────────────────

    fn discover_files(root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut current = Some(root.to_path_buf());

        for _ in 0..4 {
            if let Some(ref dir) = current {
                for filename in RULE_FILES {
                    let candidate = dir.join(filename);
                    if candidate.exists() && candidate.is_file() {
                        found.push(candidate);
                    }
                }
                current = dir.parent().map(|p| p.to_path_buf());
            }
        }
        found.sort();
        found.dedup();
        found
    }

    fn parse_sections(text: &str) -> HashMap<String, String> {
        let mut sections = HashMap::new();
        let mut current_name: Option<String> = None;
        let mut current_lines: Vec<String> = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
                // 刷新上一节
                if let Some(name) = current_name.take() {
                    sections.insert(name, current_lines.join("\n"));
                }
                current_lines = Vec::new();
                current_name = Some(trimmed[3..].trim().to_string());
            } else if current_name.is_some() {
                current_lines.push(line.to_string());
            }
        }

        // 最后一节
        if let Some(name) = current_name {
            sections.insert(name, current_lines.join("\n"));
        }

        sections
    }

    fn _has_any_file_changed(
        &self,
        project_dir: &Path,
        cached_mtimes: &HashMap<PathBuf, SystemTime>,
    ) -> bool {
        let found = Self::discover_files(project_dir);
        for path in &found {
            if let Ok(meta) = std::fs::metadata(path) {
                if let Ok(mtime) = meta.modified() {
                    if let Some(cached_mtime) = cached_mtimes.get(path) {
                        if mtime > *cached_mtime {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        found.len() != cached_mtimes.len()
    }

    fn canonicalize(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nt_io_agents_md_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_empty_when_no_file() {
        let dir = temp_dir("empty");
        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert!(rules.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detects_agents_md() {
        let dir = temp_dir("agents_md");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Project\n\n## Build\n\ncargo build").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert!(!rules.is_empty());
        assert_eq!(rules.source_files.len(), 1);
        assert!(rules.sections.contains_key("Build"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detects_claude_md() {
        let dir = temp_dir("claude_md");
        let path = dir.join("CLAUDE.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Claude\n\n## Conventions\n\nUse snake_case").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert!(!rules.is_empty());
        assert!(rules.sections.contains_key("Conventions"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detects_cursorrules() {
        let dir = temp_dir("cursorrules");
        let path = dir.join(".cursorrules");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Cursor\n\n## Architecture\n\nMonorepo").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert!(!rules.is_empty());
        assert!(rules.sections.contains_key("Architecture"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detects_parent_directory() {
        let dir = temp_dir("parent_dir");
        let child = dir.join("sub");
        fs::create_dir_all(&child).unwrap();
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Root\n\n## Test\n\ncargo test").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&child).unwrap();
        assert!(!rules.is_empty());
        assert!(rules.sections.contains_key("Test"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_sections() {
        let text = "# Title\n\n## Build\ncargo build\n\n## Test\ncargo test\n\n### Nested\nstill in test\n\n## Config\nenv=prod";
        let sections = AgentsMdReader::parse_sections(text);
        assert_eq!(sections.len(), 3);
        assert!(sections.contains_key("Build"));
        assert!(sections.contains_key("Test"));
        assert!(sections.contains_key("Config"));
        assert!(sections.get("Build").unwrap().contains("cargo build"));
        assert!(sections.get("Test").unwrap().contains("### Nested"));
    }

    #[test]
    fn test_get_context_text() {
        let dir = temp_dir("context_text");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Proj\n\n## Build\ncargo build").unwrap();
        drop(f);

        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let reader = AgentsMdReader::new();
        let ctx = reader.get_context_text();
        assert!(ctx.contains("## Project Rules"));
        assert!(ctx.contains("AGENTS.md"));

        std::env::set_current_dir(orig_dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_has_changed_detects_modification() {
        let dir = temp_dir("has_changed");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Test\n\n## A\ncontent").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let _rules = reader.load_project_rules(&dir).unwrap();

        // 修改文件后应检测到变更
        let mut f = fs::OpenOptions::new().append(true).open(&path).unwrap();
        writeln!(f, "\n## B\nmore").unwrap();
        drop(f);

        assert!(reader.has_changed(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_multiple_rule_files() {
        let dir = temp_dir("multi_file");
        let a = dir.join("AGENTS.md");
        let c = dir.join("CLAUDE.md");
        let mut f = fs::File::create(&a).unwrap();
        writeln!(f, "# A\n\n## SectionA\naaa").unwrap();
        drop(f);
        let mut f = fs::File::create(&c).unwrap();
        writeln!(f, "# C\n\n## SectionC\nccc").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert_eq!(rules.source_files.len(), 2);
        assert!(rules.sections.contains_key("SectionA"));
        assert!(rules.sections.contains_key("SectionC"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cache_ttl() {
        let dir = temp_dir("cache_ttl");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# TTL\n\n## X\nv1").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules1 = reader.load_project_rules(&dir).unwrap();
        assert_eq!(rules1.sections.get("X").unwrap().trim(), "v1");

        // 修改文件 → 缓存检测到文件变更后自动刷新
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# TTL\n\n## X\nv2").unwrap();
        drop(f);

        let rules2 = reader.load_project_rules(&dir).unwrap();
        assert_eq!(rules2.sections.get("X").unwrap().trim(), "v2");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_architecture_field_populated() {
        let dir = temp_dir("arch_field");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Proj\n\n## Architecture\nE8 + GWT\n\n## Test\ncargo test").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert!(!rules.architecture.is_empty());
        assert!(rules.architecture[0].contains("E8 + GWT"));
        assert!(!rules.test_instructions.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_conventions_field_populated() {
        let dir = temp_dir("conventions_field");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Proj\n\n## Code Conventions\nsnake_case").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert!(!rules.conventions.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_sections_when_no_headers() {
        let dir = temp_dir("no_headers");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Just a title\n\nSome plain text without sections.").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules = reader.load_project_rules(&dir).unwrap();
        assert!(rules.sections.is_empty());
        assert!(!rules.raw_text.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_section_by_name() {
        let dir = temp_dir("section_by_name");
        let path = dir.join("CLAUDE.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Claude\n\n## Build\nnpm run build\n\n## Run\nnpm start").unwrap();
        drop(f);

        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let reader = AgentsMdReader::new();
        let build_section = reader.get_section("Build");
        assert!(build_section.is_some());
        assert!(build_section.unwrap().contains("npm run build"));

        std::env::set_current_dir(orig_dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_no_double_discover() {
        let dir = temp_dir("no_double");
        let path = dir.join("AGENTS.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Proj\n\n## X\ncontent").unwrap();
        drop(f);

        let reader = AgentsMdReader::new();
        let rules1 = reader.load_project_rules(&dir).unwrap();
        let rules2 = reader.load_project_rules(&dir).unwrap();
        assert_eq!(rules1.source_files.len(), rules2.source_files.len());
        let _ = fs::remove_dir_all(&dir);
    }
}
