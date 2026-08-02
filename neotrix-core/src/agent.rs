//! Backward-compat stub for deleted agent/ directory.

pub mod hooks {
    use std::time::Instant;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HookProfile { Standard, Strict, Permissive }

    #[derive(Debug, Clone)]
    pub struct HookRegistry {
        profile: HookProfile,
    }

    impl Default for HookRegistry {
        fn default() -> Self {
            Self { profile: HookProfile::Standard }
        }
    }

    impl HookRegistry {
        pub fn check_blocked(_actions: &[String]) -> Option<String> { None }

        pub fn execute_event(&self, _ctx: &HookContext) -> Vec<String> { Vec::new() }

        pub fn set_profile(&mut self, profile: HookProfile) { self.profile = profile; }

        pub fn hook_count(&self) -> usize { 0 }

        pub fn list_hooks(&self) -> Vec<(String, String)> { Vec::new() }
    }

    #[derive(Debug, Clone)]
    pub enum HookEvent { SessionStart, SessionEnd, PreToolUse, PostToolUse }

    #[derive(Debug, Clone)]
    pub struct HookContext {
        pub event: HookEvent,
        pub session_id: Option<String>,
        pub timestamp: Instant,
        pub file_path: Option<String>,
        pub tool_name: Option<String>,
        pub tool_input: Option<String>,
        pub tool_output: Option<String>,
    }

    impl Default for HookContext {
        fn default() -> Self {
            Self {
                event: HookEvent::SessionStart,
                session_id: None,
                timestamp: Instant::now(),
                file_path: None,
                tool_name: None,
                tool_input: None,
                tool_output: None,
            }
        }
    }

    impl HookContext {
        pub fn new(event: HookEvent) -> Self {
            Self {
                event,
                session_id: None,
                timestamp: Instant::now(),
                file_path: None,
                tool_name: None,
                tool_input: None,
                tool_output: None,
            }
        }
    }
}

pub mod team {
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct AgentTeamResult {
        pub agent_name: String,
        pub success: bool,
        pub output: String,
    }

    #[derive(Debug, Clone)]
    pub struct AgentRole {
        pub name: String,
        pub role: String,
        pub goal: String,
        pub backstory: String,
        pub tools: Vec<String>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProcessType { Sequential, Parallel, Supervised }

    #[derive(Debug, Clone)]
    pub struct AgentTeam {
        pub name: String,
        pub process_type: ProcessType,
        pub agents: Vec<String>,
        pub state: HashMap<String, String>,
    }

    impl AgentTeam {
        pub fn new(name: &str, process_type: ProcessType) -> Self {
            Self { name: name.to_string(), process_type, agents: Vec::new(), state: HashMap::new() }
        }

        pub fn add_agent(&mut self, _role: AgentRole) {
            self.agents.push(_role.name);
        }
        pub fn execute(&self, _task: &str) -> Vec<AgentResult> {
            vec![AgentResult { agent_name: "stub".into(), success: true, output: String::new() }]
        }
    }

    #[derive(Debug, Clone)]
    pub struct AgentResult {
        pub agent_name: String,
        pub success: bool,
        pub output: String,
    }
}

pub mod interface {
    #[derive(Debug, Clone)]
    pub struct AgentInterface;
}

pub mod decoder {
    pub fn decode_state(_delta: &[f64], _confidence: f64, _min: f64) -> String { String::new() }
}

pub mod skills {
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone)]
    pub struct DiscoveredSkill {
        pub name: String,
        pub description: String,
        pub path: PathBuf,
    }

    #[derive(Debug, Clone)]
    pub struct SkillsEngine {
        pub skills: Vec<String>,
    }

    impl SkillsEngine {
        pub fn new() -> Self { Self { skills: Vec::new() } }

        pub fn init(&mut self) -> Vec<String> {
            let discovered = Self::discover_all();
            self.skills = discovered.iter().map(|s| s.name.clone()).collect();
            self.skills.clone()
        }

        /// Discover all skills from workspace skills/, ~/.neotrix/skills/, ~/.agents/skills/
        pub fn discover_all() -> Vec<DiscoveredSkill> {
            let mut skills = Vec::new();
            let seen: &mut Vec<String> = &mut Vec::new();

            // 1. Workspace skills/
            let ws = Path::new("skills");
            if ws.exists() {
                Self::scan_dir(ws, seen, &mut skills);
            }

            // 2. ~/.neotrix/skills/
            if let Ok(home) = std::env::var("HOME") {
                let home_dir = PathBuf::from(&home).join(".neotrix").join("skills");
                if home_dir.exists() {
                    Self::scan_dir(&home_dir, seen, &mut skills);
                }
            }

            // 3. ~/.agents/skills/
            if let Ok(home) = std::env::var("HOME") {
                let agents_dir = PathBuf::from(&home).join(".agents").join("skills");
                if agents_dir.exists() {
                    Self::scan_dir(&agents_dir, seen, &mut skills);
                }
            }

            skills
        }

        fn scan_dir(dir: &Path, seen: &mut Vec<String>, skills: &mut Vec<DiscoveredSkill>) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let name_owned = name.clone();
                        if seen.contains(&name_owned) { continue; }
                        let skill_md = path.join("SKILL.md");
                        if skill_md.exists() {
                            let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                            let description = Self::extract_description(&content);
                            seen.push(name_owned.clone());
                            skills.push(DiscoveredSkill {
                                name: name_owned,
                                description,
                                path: skill_md,
                            });
                        }
                    } else if let Some(ext) = path.extension() {
                        if ext == "json" {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                let name_owned = name.to_string();
                                if seen.contains(&name_owned) { continue; }
                                seen.push(name.to_string());
                                skills.push(DiscoveredSkill {
                                    name: name.to_string(),
                                    description: format!(".skill.json: {}", path.display()),
                                    path,
                                });
                            }
                        }
                    }
                }
            }
        }

        fn extract_description(content: &str) -> String {
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("description:") {
                    return val.trim().to_string();
                }
            }
            String::new()
        }

        /// Find all SKILL.md files recursively within a directory
        pub fn find_skill_mds(dir: &Path) -> Vec<PathBuf> {
            let mut results = Vec::new();
            if dir.is_file() && dir.ends_with("SKILL.md") {
                results.push(dir.to_path_buf());
                return results;
            }
            Self::find_skill_mds_recursive(dir, &mut results);
            results
        }

        fn find_skill_mds_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip hidden directories and common excludes
                        let fname = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                        if fname.starts_with('.') || fname == "node_modules" || fname == "target" {
                            continue;
                        }
                        Self::find_skill_mds_recursive(&path, results);
                    } else if path.ends_with("SKILL.md") {
                        results.push(path);
                    }
                }
            }
        }
    }

    impl Default for SkillsEngine {
        fn default() -> Self { Self::new() }
    }

    #[derive(Debug, Clone)]
    pub struct SkillSource;
}

pub mod workflow {
    #[derive(Debug, Clone)]
    pub struct Workflow {
        pub name: String,
        pub description: String,
        pub steps: Vec<WorkflowStep>,
    }

    #[derive(Debug, Clone)]
    pub enum WorkflowStep {
        AgentTask { name: String, task_description: String },
    }

    #[derive(Debug, Clone)]
    pub struct WorkflowResult {
        pub step_name: String,
        pub success: bool,
    }

    #[derive(Debug, Clone)]
    pub struct WorkflowEngine;

    impl WorkflowEngine {
        pub fn new() -> Self { Self }
        pub fn register(&mut self, _workflow: Workflow) {}
        pub fn run(&self, _name: &str, _ctx: &str) -> Vec<WorkflowResult> { Vec::new() }
    }

    impl Default for WorkflowEngine {
        fn default() -> Self { Self }
    }
}

pub mod tool {
    pub mod mcp {
        //! Re-export from the canonical MCP registry module.
        pub use crate::neotrix::l1_body_impl::nt_agent_mcp_registry::*;
    }

    use std::sync::{Arc, RwLock};

    /// ToolOrchestrator — 统一原生工具编排器
    ///
    /// 吸收管线终点：外部 MCP 服务器 → McpToolAdapter → ToolOrchestrator。
    /// 上层（GWT、SEAL、nt_cap）把每个 NativeTool 视为普通工具。
    #[derive(Default)]
    pub struct ToolOrchestrator {
        tools: Vec<Box<dyn crate::core::nt_core_traits::NativeTool>>,
    }

    impl ToolOrchestrator {
        pub fn new(_cap: Arc<RwLock<crate::core::nt_core_cap::CapabilityVector>>) -> Self {
            Self::default()
        }

        /// 批量注册 NativeTool（吸收的 MCP 服务器走此路径）。
        pub fn register_native_all(
            &mut self,
            tools: Vec<Box<dyn crate::core::nt_core_traits::NativeTool>>,
        ) {
            self.tools.extend(tools);
        }

        pub fn native_count(&self) -> usize {
            self.tools.len()
        }

        /// 列出所有已注册工具的 ToolDef（供 /mcp native 与上层消费）。
        pub fn list_defs(&self) -> Vec<crate::core::nt_core_traits::ToolDef> {
            self.tools.iter().map(|t| t.to_def()).collect()
        }

        /// 按 id 派发调用。
        pub fn call(
            &self,
            name: &str,
            args: &serde_json::Value,
        ) -> Result<crate::core::nt_core_traits::ToolOutput, String> {
            self.tools
                .iter()
                .find(|t| t.id() == name)
                .ok_or_else(|| format!("Native tool '{}' not registered", name))
                .and_then(|t| t.execute(args))
        }
    }

    /// 从全局 McpRegistry 重建吸收的原生工具列表（真实路径，非空壳）。
    pub fn all_native_tools() -> Vec<Box<dyn crate::core::nt_core_traits::NativeTool>> {
        crate::cli::commands::agent_cmds::get_mcp_registry()
            .blocking_read()
            .as_native_tools()
    }
}

pub type McpServer = tool::mcp::McpServerEntry;

pub use team::{AgentTeam, AgentRole, ProcessType};

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true);
    }

    use crate::core::nt_core_traits::{NativeTool, ToolOutput};
    use serde_json::json;

    struct DummyTool(&'static str);

    impl NativeTool for DummyTool {
        fn id(&self) -> &str {
            self.0
        }
        fn description(&self) -> &str {
            "dummy"
        }
        fn input_schema(&self) -> serde_json::Value {
            json!({"type": "object", "properties": {}})
        }
        fn capability_tags(&self) -> Vec<&'static str> {
            vec![]
        }
        fn execute(&self, _args: &serde_json::Value) -> Result<ToolOutput, String> {
            Ok(ToolOutput { success: true, content: format!("ran {}", self.0) })
        }
    }

    #[test]
    fn test_tool_orchestrator_register_and_count() {
        let mut orch = super::tool::ToolOrchestrator::default();
        assert_eq!(orch.native_count(), 0);
        orch.register_native_all(vec![
            Box::new(DummyTool("alpha")) as Box<dyn NativeTool>,
            Box::new(DummyTool("beta")) as Box<dyn NativeTool>,
        ]);
        assert_eq!(orch.native_count(), 2);
        assert_eq!(orch.list_defs().len(), 2);
    }

    #[test]
    fn test_tool_orchestrator_dispatch() {
        let mut orch = super::tool::ToolOrchestrator::default();
        orch.register_native_all(vec![Box::new(DummyTool("calc")) as Box<dyn NativeTool>]);
        let out = orch.call("calc", &json!({"a": 1})).expect("dispatch");
        assert!(out.success);
        assert!(out.content.contains("ran calc"));
        let err = orch.call("ghost", &json!({}));
        assert!(err.is_err(), "unregistered tool must error");
        let msg = match err {
            Ok(_) => String::new(),
            Err(e) => e,
        };
        assert!(msg.contains("not registered"));
    }

    #[test]
    fn test_all_native_tools_from_global_registry() {
        // Before any registry is set, must return empty (never panic).
        let tools = super::tool::all_native_tools();
        assert!(tools.is_empty());
    }
}
