use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════════
// SubAgentDef — YAML frontmatter agent definition
// Claude Code `.claude/agents/*.md` compatible format
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentDef {
    pub name: String,
    pub description: String,
    pub tools: Option<Vec<String>>,
    pub disallowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub permission_mode: Option<String>,
    pub max_turns: Option<usize>,
    pub skills: Option<Vec<String>>,
    pub memory: Option<String>,
    pub background: Option<bool>,
    pub isolation: Option<String>,
    pub effort: Option<String>,
    pub color: Option<String>,
    pub initial_prompt: Option<String>,
    pub source_path: PathBuf,
    pub body: String,
}

impl SubAgentDef {
    pub fn display_name(&self) -> &str {
        &self.name
    }

    pub fn scope_label(&self) -> &str {
        if self.source_path.starts_with(dirs::home_dir().unwrap_or_default()) {
            "user"
        } else {
            "project"
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// SubAgentDefParser — YAML frontmatter + Markdown body
// ═══════════════════════════════════════════════════════════════════

pub struct SubAgentDefParser;

impl SubAgentDefParser {
    pub fn parse(path: &Path, content: &str) -> Option<SubAgentDef> {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") {
            return None;
        }
        let end = stripped[3..].find("---")?;
        let frontmatter = &stripped[3..3 + end];
        let body = &stripped[3 + end + 3..];

        let mut name = String::new();
        let mut description = String::new();
        let mut tools: Option<Vec<String>> = None;
        let mut disallowed_tools: Option<Vec<String>> = None;
        let mut model: Option<String> = None;
        let mut permission_mode: Option<String> = None;
        let mut max_turns: Option<usize> = None;
        let mut skills: Option<Vec<String>> = None;
        let mut memory: Option<String> = None;
        let mut background: Option<bool> = None;
        let mut isolation: Option<String> = None;
        let mut effort: Option<String> = None;
        let mut color: Option<String> = None;
        let mut initial_prompt: Option<String> = None;

        for line in frontmatter.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(val) = line.strip_prefix("name:") {
                name = Self::parse_scalar(val);
            } else if let Some(val) = line.strip_prefix("description:") {
                description = Self::parse_scalar(val);
            } else if let Some(val) = line.strip_prefix("tools:") {
                tools = Some(Self::parse_list(val));
            } else if let Some(val) = line.strip_prefix("disallowedTools:") {
                disallowed_tools = Some(Self::parse_list(val));
            } else if let Some(val) = line.strip_prefix("model:") {
                model = Some(Self::parse_scalar(val));
            } else if let Some(val) = line.strip_prefix("permissionMode:") {
                permission_mode = Some(Self::parse_scalar(val));
            } else if let Some(val) = line.strip_prefix("maxTurns:") {
                max_turns = Self::parse_scalar(val).parse::<usize>().ok();
            } else if let Some(val) = line.strip_prefix("skills:") {
                skills = Some(Self::parse_list(val));
            } else if let Some(val) = line.strip_prefix("memory:") {
                memory = Some(Self::parse_scalar(val));
            } else if let Some(val) = line.strip_prefix("background:") {
                background = Self::parse_scalar(val) == "true";
            } else if let Some(val) = line.strip_prefix("isolation:") {
                isolation = Some(Self::parse_scalar(val));
            } else if let Some(val) = line.strip_prefix("effort:") {
                effort = Some(Self::parse_scalar(val));
            } else if let Some(val) = line.strip_prefix("color:") {
                color = Some(Self::parse_scalar(val));
            } else if let Some(val) = line.strip_prefix("initialPrompt:") {
                initial_prompt = Some(Self::parse_scalar(val));
            }
        }

        if name.is_empty() || description.is_empty() {
            return None;
        }

        Some(SubAgentDef {
            name,
            description,
            tools,
            disallowed_tools,
            model,
            permission_mode,
            max_turns,
            skills,
            memory,
            background,
            isolation,
            effort,
            color,
            initial_prompt,
            source_path: path.to_path_buf(),
            body: body.trim().to_string(),
        })
    }

    fn parse_scalar(val: &str) -> String {
        val.trim().trim_matches('"').trim_matches('\'').to_string()
    }

    fn parse_list(val: &str) -> Vec<String> {
        let trimmed = val.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let inner = &trimmed[1..trimmed.len() - 1];
            inner
                .split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            trimmed
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// SubAgentRegistry — directory scanner with priority hierarchy
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct AgentScanReport {
    pub total: usize,
    pub new: usize,
    pub updated: usize,
    pub errors: Vec<String>,
}

pub struct SubAgentRegistry {
    agents: HashMap<String, SubAgentDef>,
    source_dirs: Vec<PathBuf>,
    scan_count: u64,
}

impl SubAgentRegistry {
    pub fn new() -> Self {
        let user_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("agents");

        Self {
            agents: HashMap::new(),
            source_dirs: vec![user_dir],
            scan_count: 0,
        }
    }

    pub fn add_project_dir(&mut self, path: PathBuf) {
        let agent_path = path.join(".neotrix").join("agents");
        if !self.source_dirs.contains(&agent_path) {
            self.source_dirs.push(agent_path);
        }
    }

    pub fn scan_all(&mut self) -> AgentScanReport {
        let mut report = AgentScanReport {
            total: 0,
            new: 0,
            updated: 0,
            errors: Vec::new(),
        };

        for dir in &self.source_dirs {
            let scan = self.scan_directory(dir);
            report.new += scan.new;
            report.updated += scan.updated;
            report.errors.extend(scan.errors);
        }

        report.total = self.agents.len();
        self.scan_count += 1;
        report
    }

    fn scan_directory(&mut self, dir: &Path) -> AgentScanReport {
        let mut report = AgentScanReport {
            total: 0,
            new: 0,
            updated: 0,
            errors: Vec::new(),
        };

        if !dir.exists() {
            return report;
        }

        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(true, |e| e != "md") {
                        continue;
                    }
                    match Self::register_file(&mut self.agents, &path) {
                        Ok(IsNew::New) => report.new += 1,
                        Ok(IsNew::Updated) => report.updated += 1,
                        Ok(IsNew::Existing) => {}
                        Err(e) => report.errors.push(format!("{}: {}", path.display(), e)),
                    }
                }
            }
            Err(e) => {
                report.errors.push(format!("read_dir {}: {}", dir.display(), e));
            }
        }

        report
    }

    fn register_file(
        agents: &mut HashMap<String, SubAgentDef>,
        path: &Path,
    ) -> Result<IsNew, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))?;

        let def = SubAgentDefParser::parse(path, &content)
            .ok_or_else(|| "invalid frontmatter (missing name or description)".to_string())?;

        let is_new = if agents.contains_key(&def.name) {
            IsNew::Existing
        } else {
            IsNew::New
        };

        agents.insert(def.name.clone(), def);
        Ok(is_new)
    }

    pub fn get(&self, name: &str) -> Option<&SubAgentDef> {
        self.agents.get(name)
    }

    pub fn list_all(&self) -> Vec<&SubAgentDef> {
        let mut list: Vec<&SubAgentDef> = self.agents.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn search(&self, query: &str) -> Vec<&SubAgentDef> {
        let q = query.to_lowercase();
        self.agents
            .values()
            .filter(|a| {
                a.name.to_lowercase().contains(&q)
                    || a.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn count(&self) -> usize {
        self.agents.len()
    }

    pub fn scan_count(&self) -> u64 {
        self.scan_count
    }

    pub fn create_agent_file(&self, name: &str, def: &SubAgentDef) -> Result<PathBuf, String> {
        let project_dir = self.source_dirs.iter()
            .find(|d| !d.starts_with(dirs::home_dir().unwrap_or_default()))
            .cloned()
            .unwrap_or_else(|| {
                PathBuf::from(".neotrix").join("agents")
            });

        std::fs::create_dir_all(&project_dir)
            .map_err(|e| format!("create agents dir: {e}"))?;

        let file_path = project_dir.join(format!("{}.md", name));

        let mut frontmatter = String::from("---\n");
        frontmatter.push_str(&format!("name: {}\n", def.name));
        frontmatter.push_str(&format!("description: {}\n", def.description));

        if let Some(ref tools) = def.tools {
            frontmatter.push_str(&format!("tools: [{}]\n", tools.join(", ")));
        }
        if let Some(ref model) = def.model {
            frontmatter.push_str(&format!("model: {}\n", model));
        }
        if let Some(ref mode) = def.permission_mode {
            frontmatter.push_str(&format!("permissionMode: {}\n", mode));
        }
        if let Some(max) = def.max_turns {
            frontmatter.push_str(&format!("maxTurns: {}\n", max));
        }
        if let Some(ref skills) = def.skills {
            frontmatter.push_str(&format!("skills: [{}]\n", skills.join(", ")));
        }
        if let Some(ref memory) = def.memory {
            frontmatter.push_str(&format!("memory: {}\n", memory));
        }
        if let Some(bg) = def.background {
            frontmatter.push_str(&format!("background: {}\n", bg));
        }
        if let Some(ref color) = def.color {
            frontmatter.push_str(&format!("color: {}\n", color));
        }
        frontmatter.push_str("---\n\n");
        frontmatter.push_str(&def.body);

        std::fs::write(&file_path, &frontmatter)
            .map_err(|e| format!("write agent file: {e}"))?;

        Ok(file_path)
    }
}

impl Default for SubAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

enum IsNew {
    New,
    Updated,
    Existing,
}

// ═══════════════════════════════════════════════════════════════════
// SubAgentManagerV2 — Extended agent manager with tool/permission model
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefConfig {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub model: String,
    pub permission_mode: String,
    pub max_turns: usize,
    pub skills: Vec<String>,
    pub memory: String,
    pub background: bool,
    pub isolation: String,
    pub effort: String,
}

impl From<SubAgentDef> for AgentDefConfig {
    fn from(def: SubAgentDef) -> Self {
        Self {
            name: def.name,
            description: def.description,
            tools: def.tools.unwrap_or_default(),
            disallowed_tools: def.disallowed_tools.unwrap_or_default(),
            model: def.model.unwrap_or_else(|| "inherit".to_string()),
            permission_mode: def.permission_mode.unwrap_or_else(|| "default".to_string()),
            max_turns: def.max_turns.unwrap_or(50),
            skills: def.skills.unwrap_or_default(),
            memory: def.memory.unwrap_or_else(|| "project".to_string()),
            background: def.background.unwrap_or(false),
            isolation: def.isolation.unwrap_or_default(),
            effort: def.effort.unwrap_or_else(|| "medium".to_string()),
        }
    }
}

impl AgentDefConfig {
    pub fn tool_allowlist(&self) -> Option<Vec<String>> {
        if self.tools.is_empty() {
            None
        } else {
            Some(self.tools.clone())
        }
    }

    pub fn is_readonly(&self) -> bool {
        if let Some(tools) = self.tool_allowlist() {
            !tools.iter().any(|t| t == "Write" || t == "Edit" || t == "Bash")
        } else {
            false
        }
    }

    pub fn allowed_model(&self) -> &str {
        &self.model
    }
}

// ═══════════════════════════════════════════════════════════════════
// AgentExecutionStats — runtime tracking for consciousness tree
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentExecutionStats {
    pub total_spawned: u64,
    pub total_completed: u64,
    pub total_failed: u64,
    pub currently_running: u32,
    pub total_tokens_used: u64,
    pub by_tool: HashMap<String, u64>,
    pub by_model: HashMap<String, u64>,
}

impl AgentExecutionStats {
    pub fn record_spawn(&mut self) {
        self.total_spawned += 1;
        self.currently_running += 1;
    }

    pub fn record_complete(&mut self) {
        self.total_completed += 1;
        self.currently_running = self.currently_running.saturating_sub(1);
    }

    pub fn record_failure(&mut self) {
        self.total_failed += 1;
        self.currently_running = self.currently_running.saturating_sub(1);
    }

    pub fn record_tool_use(&mut self, tool: &str) {
        *self.by_tool.entry(tool.to_string()).or_insert(0) += 1;
    }

    pub fn health_score(&self) -> f64 {
        if self.total_spawned == 0 {
            return 1.0;
        }
        let success_rate = (self.total_completed as f64) / (self.total_spawned as f64);
        let running_ratio = (self.currently_running as f64).min(10.0) / 10.0;
        success_rate * (1.0 - running_ratio * 0.3)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Built-in agents (Claude Code compatible subagent types)
// ═══════════════════════════════════════════════════════════════════

pub fn default_agents_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("agents")
}

pub fn project_agents_dir(project_root: &Path) -> PathBuf {
    project_root.join(".neotrix").join("agents")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_AGENT_MD: &str = r#"---
name: code-reviewer
description: Reviews code for quality and best practices
tools: [Read, Glob, Grep]
model: sonnet
permissionMode: default
maxTurns: 30
memory: project
---

You are a code reviewer. When invoked, analyze the code and provide
specific, actionable feedback on quality, security, and best practices.
"#;

    #[test]
    fn test_parse_valid_agent() {
        let path = Path::new("/fake/path/code-reviewer.md");
        let def = SubAgentDefParser::parse(path, SAMPLE_AGENT_MD).unwrap();
        assert_eq!(def.name, "code-reviewer");
        assert_eq!(def.description, "Reviews code for quality and best practices");
        assert_eq!(def.tools, Some(vec!["Read".into(), "Glob".into(), "Grep".into()]));
        assert_eq!(def.model, Some("sonnet".into()));
        assert_eq!(def.max_turns, Some(30));
        assert_eq!(def.memory, Some("project".into()));
        assert!(def.body.contains("code reviewer"));
    }

    #[test]
    fn test_parse_no_frontmatter_returns_none() {
        let content = "# Just a markdown file\n\nNo frontmatter here.";
        let def = SubAgentDefParser::parse(Path::new("test.md"), content);
        assert!(def.is_none());
    }

    #[test]
    fn test_parse_missing_name_returns_none() {
        let content = r#"---
description: Missing name field
---
body"#;
        let def = SubAgentDefParser::parse(Path::new("test.md"), content);
        assert!(def.is_none());
    }

    #[test]
    fn test_parse_all_fields() {
        let content = r#"---
name: full-agent
description: Agent with all fields
tools: [Read, Write, Bash, Grep]
disallowedTools: [Edit]
model: opus
permissionMode: acceptEdits
maxTurns: 100
skills: [code-review, security-check]
memory: user
background: true
isolation: worktree
effort: high
color: blue
initialPrompt: "Review this PR"
---

Full agent body
"#;
        let def = SubAgentDefParser::parse(Path::new("test.md"), content).unwrap();
        assert_eq!(def.name, "full-agent");
        assert_eq!(def.model, Some("opus".into()));
        assert_eq!(def.permission_mode, Some("acceptEdits".into()));
        assert_eq!(def.max_turns, Some(100));
        assert!(def.background == Some(true));
        assert_eq!(def.isolation, Some("worktree".into()));
        assert_eq!(def.color, Some("blue".into()));
        assert_eq!(def.initial_prompt, Some("Review this PR".into()));
    }

    #[test]
    fn test_parse_list_variants() {
        let content = r#"---
name: list-tester
description: Test list parsing
tools: [Read, Grep]
skills: [skill-a, skill-b]
---
body"#;
        let def = SubAgentDefParser::parse(Path::new("test.md"), content).unwrap();
        assert_eq!(def.tools, Some(vec!["Read".into(), "Grep".into()]));
        assert_eq!(def.skills, Some(vec!["skill-a".into(), "skill-b".into()]));
    }

    #[test]
    fn test_registry_new_empty() {
        let reg = SubAgentRegistry::new();
        assert_eq!(reg.count(), 0);
        assert_eq!(reg.list_all().len(), 0);
    }

    #[test]
    fn test_registry_register_file() {
        let mut reg = SubAgentRegistry::new();
        let dir = std::env::temp_dir().join("neotrix-test-agents");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test-agent.md");
        std::fs::write(&file_path, SAMPLE_AGENT_MD).unwrap();

        reg.add_project_dir(dir.parent().unwrap().to_path_buf());
        let report = reg.scan_all();
        assert_eq!(report.new, 1);
        assert!(reg.get("code-reviewer").is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_registry_search() {
        let mut reg = SubAgentRegistry::new();
        let dir = std::env::temp_dir().join("neotrix-test-search");
        let _ = std::fs::create_dir_all(&dir);

        let agent1 = r#"---
name: security-scanner
description: Security audit and vulnerability scanning
tools: [Read, Grep, Bash]
---"#;
        let agent2 = r#"---
name: test-runner
description: Run tests and report results
tools: [Bash, Read]
---"#;
        std::fs::write(dir.join("security-scanner.md"), agent1).unwrap();
        std::fs::write(dir.join("test-runner.md"), agent2).unwrap();

        reg.add_project_dir(dir.parent().unwrap().to_path_buf());
        reg.scan_all();

        let results = reg.search("security");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "security-scanner");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_agent_def_config_from_def() {
        let path = Path::new("test.md");
        let def = SubAgentDefParser::parse(path, SAMPLE_AGENT_MD).unwrap();
        let config: AgentDefConfig = def.into();
        assert_eq!(config.name, "code-reviewer");
        assert_eq!(config.model, "sonnet");
        assert_eq!(config.max_turns, 30);
        assert_eq!(config.memory, "project");
    }

    #[test]
    fn test_is_readonly_with_write_tool() {
        let config = AgentDefConfig {
            name: "writer".into(),
            description: "".into(),
            tools: vec!["Read".into(), "Write".into()],
            disallowed_tools: vec![],
            model: "inherit".into(),
            permission_mode: "default".into(),
            max_turns: 50,
            skills: vec![],
            memory: "project".into(),
            background: false,
            isolation: "".into(),
            effort: "medium".into(),
        };
        assert!(!config.is_readonly());
    }

    #[test]
    fn test_is_readonly_read_tools_only() {
        let config = AgentDefConfig {
            name: "reader".into(),
            description: "".into(),
            tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
            disallowed_tools: vec![],
            model: "inherit".into(),
            permission_mode: "default".into(),
            max_turns: 50,
            skills: vec![],
            memory: "project".into(),
            background: false,
            isolation: "".into(),
            effort: "medium".into(),
        };
        assert!(config.is_readonly());
    }

    #[test]
    fn test_execution_stats_health() {
        let mut stats = AgentExecutionStats::default();
        assert!((stats.health_score() - 1.0).abs() < 1e-6);

        stats.record_spawn();
        stats.record_complete();
        assert!((stats.health_score() - 1.0).abs() < 1e-6);

        stats.record_spawn();
        stats.record_failure();
        assert!(stats.health_score() < 1.0);
    }

    #[test]
    fn test_create_agent_file_roundtrip() {
        let reg = SubAgentRegistry::new();
        let def = SubAgentDef {
            name: "roundtrip-test".into(),
            description: "Test roundtrip".into(),
            tools: Some(vec!["Read".into(), "Grep".into()]),
            disallowed_tools: None,
            model: Some("sonnet".into()),
            permission_mode: None,
            max_turns: None,
            skills: None,
            memory: None,
            background: None,
            isolation: None,
            effort: None,
            color: Some("green".into()),
            initial_prompt: None,
            source_path: Path::new("").to_path_buf(),
            body: "You are a test agent.".into(),
        };

        let dir = std::env::temp_dir().join("neotrix-test-roundtrip").join(".neotrix").join("agents");
        let _ = std::fs::create_dir_all(&dir);
        let written = reg.create_agent_file("roundtrip-test", &def);
        assert!(written.is_ok());

        let file_path = written.unwrap();
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        let reparsed = SubAgentDefParser::parse(&file_path, &content).unwrap();
        assert_eq!(reparsed.name, "roundtrip-test");
        assert_eq!(reparsed.description, "Test roundtrip");
        assert_eq!(reparsed.tools, Some(vec!["Read".into(), "Grep".into()]));
        assert_eq!(reparsed.model, Some("sonnet".into()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
