use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 计算 agent 的 E8 模式：显式 e8Mode 优先 → domain 字段 → name 前缀（`nt-scout` → `NT-SCOUT`）。
pub fn e8_mode_for(def: &SubAgentDef) -> u8 {
    def.e8_mode
        .or_else(|| def.domain.as_deref().and_then(domain_default_e8_mode))
        .or_else(|| domain_from_name(&def.name).and_then(|d| domain_default_e8_mode(&d)))
        .unwrap_or(0) as u8
}

/// 从 agent 名解析域：`nt-scout` → `NT-SCOUT`；非 `nt-` 前缀返回 None。
pub fn domain_from_name(name: &str) -> Option<String> {
    name.strip_prefix("nt-")
        .map(|s| format!("NT-{}", s.to_uppercase()))
}

/// NT 域 → 默认 E8 Hexagram 模式映射。
/// 与 E8 推理矩阵对齐：域卦位是固定常量 (NT-CORE=63, NT-WORLD=2, ...)。
pub fn domain_default_e8_mode(domain: &str) -> Option<u32> {
    match domain {
        "NT-CORE" => Some(63),
        "NT-WORLD" => Some(2),
        "NT-ACT" => Some(31),
        "NT-MIND" => Some(14),
        "NT-SHIELD" => Some(37),
        "NT-MEMORY" => Some(21),
        "NT-IO" => Some(40),
        "NT-SCOUT" => Some(11),
        "NT-META" => Some(50),
        "NT-REPAIR" => Some(55),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════
// SubAgentDef — YAML frontmatter agent definition
// Claude Code `.claude/agents/*.md` compatible format
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    // ── NeoTrix 扩展字段 ──
    /// 所属域全名 (NT-WORLD / NT-ACT / ...)。文件驱动的 NT 域 agent 必填。
    pub domain: Option<String>,
    /// E8 Hexagram 模式 (0-63)。NT 域 agent 映射到固定卦位。
    pub e8_mode: Option<u32>,
    /// 采样温度。默认 0.2。
    pub temperature: Option<f64>,
    /// 动态步骤兜底上限。默认 60。
    pub steps: Option<usize>,
    /// 工具权限矩阵 (NT-SHIELD 兼容)。缺省按域默认。
    pub permission: Option<PermissionMatrix>,
    /// 嵌套任务白名单 (可委托的子 agent)。
    pub task: Option<Vec<String>>,
    /// 触发词（逗号分隔，用于 route() 关键词路由）。
    pub trigger: Option<String>,
    pub source_path: PathBuf,
    pub body: String,
}

/// 工具权限矩阵 — 从 frontmatter `permission:` 嵌套 map 解析。
/// 与 NT-SHIELD ToolPermissionSet 互补：此处是文件驱动的宽松配置面，
/// 运行时由 NT-SHIELD 做强制校验。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionMatrix {
    pub edit: Option<ToolPermission>,
    pub write: Option<ToolPermission>,
    pub bash: Option<ToolPermission>,
    pub task: Option<TaskPermission>,
    pub todowrite: Option<ToolPermission>,
    pub webfetch: Option<ToolPermission>,
    pub websearch: Option<ToolPermission>,
}

/// 单个工具权限 (allow/deny + 可选 pattern 白名单)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub allow: bool,
    #[serde(default)]
    pub patterns: Vec<String>,
}

/// 任务委托白名单：`"*": deny` 时默认拒绝，列出名称则允许。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPermission {
    pub deny: Vec<String>,
    pub allow: Vec<String>,
}

impl Default for ToolPermission {
    fn default() -> Self {
        Self { allow: false, patterns: Vec::new() }
    }
}

impl PermissionMatrix {
    /// 工具是否允许（供运行时校验；缺省工具视为不允许）。
    pub fn tool_allowed(&self, tool: &str) -> bool {
        let p = match tool {
            "edit" | "Edit" => &self.edit,
            "write" | "Write" => &self.write,
            "bash" | "Bash" => &self.bash,
            "todowrite" | "TodoWrite" => &self.todowrite,
            "webfetch" | "WebFetch" => &self.webfetch,
            "websearch" | "WebSearch" => &self.websearch,
            _ => return false,
        };
        p.as_ref().is_some_and(|t| t.allow)
    }

    /// 生成可读工具串 (供 catalog 展示)。
    pub fn allowed_tools_string(&self) -> String {
        let mut out = Vec::new();
        let mut push = |name: &str, p: &Option<ToolPermission>| {
            match p {
                Some(t) if t.allow => out.push(name.to_string()),
                Some(_) => {}
                None => out.push(format!("{name}?")),
            }
        };
        push("edit", &self.edit);
        push("write", &self.write);
        push("bash", &self.bash);
        push("todowrite", &self.todowrite);
        push("webfetch", &self.webfetch);
        push("websearch", &self.websearch);
        if out.is_empty() {
            "read-only".into()
        } else {
            out.join(",")
        }
    }
}

impl SubAgentDef {
    /// 该 agent 的 E8 Hexagram 模式。无显式 e8Mode 时按域默认映射。
    pub fn e8_mode_for(&self) -> u32 {
        self.e8_mode
            .or_else(|| self.domain.as_deref().and_then(domain_default_e8_mode))
            .or_else(|| domain_from_name(&self.name).and_then(|d| domain_default_e8_mode(&d)))
            .unwrap_or(0)
    }

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
        let body = stripped[3 + end + 3..].trim().to_string();

        // 序列化中间表示：Claude Code 兼容字段 + NeoTrix 扩展字段
        #[derive(Deserialize, Default)]
        struct RawFrontmatter {
            name: String,
            description: String,
            #[serde(default)]
            tools: Option<Vec<String>>,
            #[serde(default, rename = "disallowedTools")]
            disallowed_tools: Option<Vec<String>>,
            #[serde(default)]
            model: Option<String>,
            #[serde(default, rename = "permissionMode")]
            permission_mode: Option<String>,
            #[serde(default, rename = "maxTurns")]
            max_turns: Option<usize>,
            #[serde(default)]
            skills: Option<Vec<String>>,
            #[serde(default)]
            memory: Option<String>,
            #[serde(default)]
            background: Option<bool>,
            #[serde(default)]
            isolation: Option<String>,
            #[serde(default)]
            effort: Option<String>,
            #[serde(default)]
            color: Option<String>,
            #[serde(default, rename = "initialPrompt")]
            initial_prompt: Option<String>,
            // ── NeoTrix 扩展 ──
            #[serde(default)]
            domain: Option<String>,
            #[serde(default, rename = "e8Mode")]
            e8_mode: Option<u32>,
            #[serde(default)]
            temperature: Option<f64>,
            #[serde(default)]
            steps: Option<usize>,
            #[serde(default)]
            permission: Option<PermissionMatrix>,
            #[serde(default)]
            task: Option<Vec<String>>,
            #[serde(default)]
            trigger: Option<String>,
        }

        let raw: RawFrontmatter = serde_yaml::from_str(frontmatter).ok()?;
        if raw.name.is_empty() || raw.description.is_empty() {
            return None;
        }

        Some(SubAgentDef {
            name: raw.name,
            description: raw.description,
            tools: raw.tools,
            disallowed_tools: raw.disallowed_tools,
            model: raw.model,
            permission_mode: raw.permission_mode,
            max_turns: raw.max_turns,
            skills: raw.skills,
            memory: raw.memory,
            background: raw.background,
            isolation: raw.isolation,
            effort: raw.effort,
            color: raw.color,
            initial_prompt: raw.initial_prompt,
            domain: raw.domain,
            e8_mode: raw.e8_mode,
            temperature: raw.temperature,
            steps: raw.steps,
            permission: raw.permission,
            task: raw.task,
            trigger: raw.trigger,
            source_path: path.to_path_buf(),
            body,
        })
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

        let source_dirs = self.source_dirs.clone();
        for dir in &source_dirs {
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
                    if path.extension().is_none_or(|e| e != "md") {
                        continue;
                    }
                    match Self::register_file(&mut self.agents, &path) {
                        Ok(IsNew::New) => report.new += 1,
                        Ok(IsNew::Existing) => report.updated += 1,
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

    /// 列出所有 NT 域 agent（name 以 `nt-` 前缀）。
    pub fn nt_domain_agents(&self) -> Vec<&SubAgentDef> {
        self.agents
            .values()
            .filter(|a| a.name.starts_with("nt-"))
            .collect()
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
        // ── NeoTrix 扩展字段序列化 ──
        if let Some(ref domain) = def.domain {
            frontmatter.push_str(&format!("domain: {}\n", domain));
        }
        if let Some(e8) = def.e8_mode {
            frontmatter.push_str(&format!("e8Mode: {}\n", e8));
        }
        if let Some(t) = def.temperature {
            frontmatter.push_str(&format!("temperature: {}\n", t));
        }
        if let Some(s) = def.steps {
            frontmatter.push_str(&format!("steps: {}\n", s));
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
    // ── NeoTrix 扩展 ──
    pub domain: Option<String>,
    pub e8_mode: Option<u32>,
    pub temperature: Option<f64>,
    pub steps: Option<usize>,
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
            domain: def.domain,
            e8_mode: def.e8_mode,
            temperature: def.temperature,
            steps: def.steps,
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
        let tmp_root = std::env::temp_dir().join("neotrix-test-agents");
        let agent_dir = tmp_root.join(".neotrix").join("agents");
        std::fs::create_dir_all(&agent_dir).unwrap();
        let file_path = agent_dir.join("test-agent.md");
        std::fs::write(&file_path, SAMPLE_AGENT_MD).unwrap();

        reg.add_project_dir(tmp_root.clone());
        let report = reg.scan_all();
        // 注册的文件必须被扫描到（用户目录 ~/.neotrix/agents/ 可能存在 NT 域文件，不设精确计数）
        assert!(reg.get("code-reviewer").is_some(), "registered file should be found");
        assert!(report.new >= 1, "at least the test file must be new");

        let _ = std::fs::remove_dir_all(&tmp_root);
    }

    #[test]
    fn test_registry_search() {
        let mut reg = SubAgentRegistry::new();
        let tmp_root = std::env::temp_dir().join("neotrix-test-search");
        let agent_dir = tmp_root.join(".neotrix").join("agents");
        std::fs::create_dir_all(&agent_dir).unwrap();

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
        std::fs::write(agent_dir.join("security-scanner.md"), agent1).unwrap();
        std::fs::write(agent_dir.join("test-runner.md"), agent2).unwrap();

        reg.add_project_dir(tmp_root.clone());
        reg.scan_all();

        let results = reg.search("security");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "security-scanner");

        let _ = std::fs::remove_dir_all(&tmp_root);
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
            domain: None,
            e8_mode: None,
            temperature: None,
            steps: None,
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
            domain: None,
            e8_mode: None,
            temperature: None,
            steps: None,
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
        // 显式注册 temp 项目目录，确保 create_agent_file 写入可控位置而非 cwd 相对路径
        let tmp_root = std::env::temp_dir().join("neotrix-test-roundtrip");
        let mut reg = SubAgentRegistry::new();
        reg.add_project_dir(tmp_root.clone());

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
            domain: Some("NT-WORLD".into()),
            e8_mode: Some(2),
            temperature: Some(0.1),
            steps: Some(30),
            permission: None,
            task: None,
            trigger: None,
            source_path: Path::new("").to_path_buf(),
            body: "You are a test agent.".into(),
        };

        let dir = tmp_root.join(".neotrix").join("agents");
        let _ = std::fs::create_dir_all(&dir);
        let written = reg.create_agent_file("roundtrip-test", &def);
        assert!(written.is_ok());

        let file_path = written.unwrap();
        assert!(file_path.exists());
        // 必须写入注册的 temp 项目目录，绝不落到仓库 cwd 相对路径
        assert!(file_path.starts_with(&tmp_root), "agent file must be under temp project dir");

        let content = std::fs::read_to_string(&file_path).unwrap();
        let reparsed = SubAgentDefParser::parse(&file_path, &content).unwrap();
        assert_eq!(reparsed.name, "roundtrip-test");
        assert_eq!(reparsed.description, "Test roundtrip");
        assert_eq!(reparsed.tools, Some(vec!["Read".into(), "Grep".into()]));
        assert_eq!(reparsed.model, Some("sonnet".into()));
        // NeoTrix 扩展字段 roundtrip
        assert_eq!(reparsed.domain, Some("NT-WORLD".into()));
        assert_eq!(reparsed.e8_mode, Some(2));
        assert_eq!(reparsed.temperature, Some(0.1));
        assert_eq!(reparsed.e8_mode_for(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_domain_default_e8_mode() {
        assert_eq!(domain_default_e8_mode("NT-CORE"), Some(63));
        assert_eq!(domain_default_e8_mode("NT-WORLD"), Some(2));
        assert_eq!(domain_default_e8_mode("NT-ACT"), Some(31));
        assert_eq!(domain_default_e8_mode("NT-SHIELD"), Some(37));
        assert_eq!(domain_default_e8_mode("UNKNOWN"), None);
    }

    #[test]
    fn test_e8_mode_fallback_from_domain() {
        // 无显式 e8Mode，但有 domain → 用域默认
        let def = SubAgentDef {
            name: "nt-act".into(),
            description: "action".into(),
            tools: None,
            disallowed_tools: None,
            model: None,
            permission_mode: None,
            max_turns: None,
            skills: None,
            memory: None,
            background: None,
            isolation: None,
            effort: None,
            color: None,
            initial_prompt: None,
            domain: Some("NT-ACT".into()),
            e8_mode: None,
            temperature: None,
            steps: None,
            permission: None,
            task: None,
            trigger: None,
            source_path: Path::new("").to_path_buf(),
            body: "".into(),
        };
        assert_eq!(def.e8_mode_for(), 31);
    }
}
