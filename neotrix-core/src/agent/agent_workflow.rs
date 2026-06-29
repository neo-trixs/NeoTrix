use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::agent::hooks::{HookContext, HookEvent, HookRegistry};

/// Commands allowed for agent execution
const COMMAND_ALLOWLIST: [&str; 16] = [
    "ls", "cat", "head", "tail", "grep", "find", "wc", "sort", "uniq", "echo", "pwd", "date",
    "whoami", "which", "diff", "cmp",
];

/// Execution mode: Plan (exploration + design, no persistent side-effects)
/// vs Execute (implementation with write/edit/commit operations).
/// Inspired by Claude Code's Plan Mode separation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanMode {
    /// Exploration & design: read/search/think only, no writes or commands
    Explore,
    /// Full execution: read/write/edit/run, all operations allowed
    Execute,
}

impl PlanMode {
    pub fn name(&self) -> &'static str {
        match self {
            PlanMode::Explore => "explore",
            PlanMode::Execute => "execute",
        }
    }

    /// Whether the given step is allowed under this mode
    pub fn allows_step(&self, step: &AgentStep) -> bool {
        match self {
            PlanMode::Explore => matches!(
                step,
                AgentStep::ReadFile { .. }
                    | AgentStep::Search { .. }
                    | AgentStep::Think { .. }
                    | AgentStep::EndTurn { .. }
            ),
            PlanMode::Execute => true,
        }
    }

    /// Whether write/edit/run operations are allowed
    pub fn allows_mutation(&self) -> bool {
        matches!(self, PlanMode::Execute)
    }
}

impl Default for PlanMode {
    fn default() -> Self {
        PlanMode::Execute
    }
}

/// A step an agent can take during its workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStep {
    /// Run a shell command
    RunCommand {
        command: String,
        description: String,
    },
    /// Read a file
    ReadFile { path: String },
    /// Write/edit a file
    EditFile { path: String, content: String },
    /// Search for text
    Search {
        pattern: String,
        path: Option<String>,
    },
    /// Delegate to a sub-agent
    Delegate { agent_id: String, prompt: String },
    /// Think/reason (adds to context but doesn't execute)
    Think { thought: String },
    /// End the current turn, return control
    EndTurn { result: Option<String> },
    /// Yield a custom tool call
    CustomTool {
        tool: String,
        args: HashMap<String, String>,
    },
}

/// Result of executing an agent workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWorkflowResult {
    pub steps_executed: usize,
    pub final_output: Option<String>,
    pub files_modified: Vec<String>,
    pub commands_run: Vec<String>,
    pub duration_ms: u64,
    pub success: bool,
}

/// A configurable agent workflow that can be loaded from a definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWorkflow {
    pub id: String,
    pub display_name: String,
    pub model_hint: Option<String>,
    pub instructions: String,
    pub plan_mode: PlanMode,
    pub steps: Vec<AgentStep>,
    pub metadata: HashMap<String, String>,
}

impl AgentWorkflow {
    pub fn new(id: &str, display_name: &str, instructions: &str) -> Self {
        AgentWorkflow {
            id: id.to_string(),
            display_name: display_name.to_string(),
            model_hint: None,
            instructions: instructions.to_string(),
            plan_mode: PlanMode::Execute,
            steps: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new workflow in Plan Mode (exploration only)
    pub fn new_plan(id: &str, display_name: &str, instructions: &str) -> Self {
        AgentWorkflow {
            id: id.to_string(),
            display_name: display_name.to_string(),
            model_hint: None,
            instructions: instructions.to_string(),
            plan_mode: PlanMode::Explore,
            steps: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a step to the workflow
    pub fn add_step(&mut self, step: AgentStep) {
        self.steps.push(step);
    }

    /// Execute all steps and collect results (no hooks)
    pub fn execute(&self) -> AgentWorkflowResult {
        self.run_steps(None)
    }

    /// Execute all steps with PreToolUse/PostToolUse hook callbacks
    pub fn execute_with_hooks(&self, hooks: &HookRegistry) -> AgentWorkflowResult {
        self.run_steps(Some(hooks))
    }

    fn run_steps(&self, hooks: Option<&HookRegistry>) -> AgentWorkflowResult {
        let start = std::time::Instant::now();
        let mut steps_executed = 0usize;
        let mut final_output: Option<String> = None;
        let mut files_modified: Vec<String> = Vec::new();
        let mut commands_run: Vec<String> = Vec::new();
        let mut all_ok = true;

        for step in &self.steps {
            steps_executed += 1;

            // Plan mode enforcement: Explore mode blocks mutation steps
            if !self.plan_mode.allows_step(step) {
                let msg =
                    format!("blocked by PlanMode::Explore (mutation not allowed in explore mode)");
                commands_run.push(format!("step {}: {}", steps_executed, msg));
                final_output = Some(msg);
                all_ok = false;
                break;
            }

            // PreToolUse hook
            if let Some(hooks) = hooks {
                let step_desc = format!("{:?}", step);
                let pre_ctx = HookContext {
                    event: HookEvent::PreToolUse,
                    tool_name: Some(step_desc.clone()),
                    tool_input: None,
                    tool_output: None,
                    file_path: None,
                    session_id: None,
                    timestamp: std::time::Instant::now(),
                };
                let pre_actions = hooks.execute_event(&pre_ctx);
                if let Some(block_reason) = HookRegistry::check_blocked(&pre_actions) {
                    commands_run.push(format!("step {} blocked: {}", steps_executed, block_reason));
                    all_ok = false;
                    break;
                }
            }

            match step {
                AgentStep::RunCommand {
                    command,
                    description: _,
                } => match Self::execute_command(command) {
                    Ok(out) => {
                        commands_run.push(command.clone());
                        final_output = Some(out);
                    }
                    Err(e) => {
                        commands_run.push(format!("{} (failed: {})", command, e));
                        all_ok = false;
                    }
                },
                AgentStep::ReadFile { path } => match Self::execute_read(path) {
                    Ok(content) => {
                        final_output = Some(content);
                    }
                    Err(e) => {
                        final_output = Some(format!("Error: {}", e));
                        all_ok = false;
                    }
                },
                AgentStep::EditFile { path, content: _ } => {
                    files_modified.push(path.clone());
                }
                AgentStep::Search {
                    pattern: _,
                    path: _,
                } => {}
                AgentStep::Delegate {
                    agent_id: _,
                    prompt: _,
                } => {}
                AgentStep::Think { thought: _ } => {}
                AgentStep::EndTurn { result } => {
                    final_output = result.clone();
                    break;
                }
                AgentStep::CustomTool { tool, args } => match Self::execute_custom(tool, args) {
                    Ok(out) => {
                        final_output = Some(out);
                    }
                    Err(e) => {
                        final_output = Some(format!("Error: {}", e));
                        all_ok = false;
                    }
                },
            }

            // PostToolUse hook
            if let Some(hooks) = hooks {
                let step_desc = format!("{:?}", step);
                let post_ctx = HookContext {
                    event: HookEvent::PostToolUse,
                    tool_name: Some(step_desc),
                    tool_input: None,
                    tool_output: final_output.clone(),
                    file_path: None,
                    session_id: None,
                    timestamp: std::time::Instant::now(),
                };
                let _ = hooks.execute_event(&post_ctx);
            }
        }

        let duration = start.elapsed();
        AgentWorkflowResult {
            steps_executed,
            final_output,
            files_modified,
            commands_run,
            duration_ms: duration.as_millis() as u64,
            success: all_ok,
        }
    }

    /// Execute a command and return stdout (no shell — split on whitespace)
    pub fn execute_command(command: &str) -> Result<String, String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("empty command".to_string());
        }
        let cmd_name = parts[0];
        if !COMMAND_ALLOWLIST.contains(&cmd_name) {
            let msg = format!("command '{}' not in allowlist", cmd_name);
            log::warn!("[agent_workflow] {}", msg);
            return Err(msg);
        }
        let out = std::process::Command::new(cmd_name)
            .args(&parts[1..])
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            Err(format!(
                "Command failed (exit {:?}): {}",
                out.status.code(),
                err
            ))
        }
    }

    /// Read a file and return its contents
    pub fn execute_read(path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
    }

    /// Execute a custom tool call (stub — real impl dispatches via ToolRegistry)
    pub fn execute_custom(tool: &str, args: &HashMap<String, String>) -> Result<String, String> {
        Ok(format!(
            "CustomTool('{}') called with {} args: {:?}",
            tool,
            args.len(),
            args
        ))
    }
}

/// Registry of available agent workflows
pub struct WorkflowRegistry {
    workflows: HashMap<String, AgentWorkflow>,
}

impl WorkflowRegistry {
    pub fn new() -> Self {
        WorkflowRegistry {
            workflows: HashMap::new(),
        }
    }

    pub fn register(&mut self, workflow: AgentWorkflow) {
        self.workflows.insert(workflow.id.clone(), workflow);
    }

    pub fn get(&self, id: &str) -> Option<&AgentWorkflow> {
        self.workflows.get(id)
    }

    pub fn list(&self) -> Vec<&str> {
        self.workflows.keys().map(|k| k.as_str()).collect()
    }

    pub fn unregister(&mut self, id: &str) -> bool {
        self.workflows.remove(id).is_some()
    }

    /// Load workflows from a directory of JSON files
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize, String> {
        if !dir.is_dir() {
            return Err(format!("'{}' is not a directory", dir.display()));
        }

        let mut count = 0usize;
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
                let wf: AgentWorkflow = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse '{}': {}", path.display(), e))?;
                self.register(wf);
                count += 1;
            }
        }
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// Built-in workflows
// ---------------------------------------------------------------------------

pub fn file_picker_workflow() -> AgentWorkflow {
    let mut wf = AgentWorkflow::new(
        "file-picker",
        "File Picker",
        "List and read files from the workspace to understand the codebase.",
    );
    wf.add_step(AgentStep::RunCommand {
        command: "ls -la".to_string(),
        description: "List workspace root".to_string(),
    });
    wf.add_step(AgentStep::EndTurn { result: None });
    wf
}

pub fn planner_workflow() -> AgentWorkflow {
    let mut wf = AgentWorkflow::new(
        "planner",
        "Planner",
        "Analyze the task and produce a step-by-step plan before executing.",
    );
    wf.add_step(AgentStep::Think {
        thought: "Parsing task requirements and decomposing into actionable steps.".to_string(),
    });
    wf.add_step(AgentStep::EndTurn { result: None });
    wf
}

pub fn editor_workflow() -> AgentWorkflow {
    let mut wf = AgentWorkflow::new(
        "editor",
        "Editor",
        "Read, modify, and write files to implement changes.",
    );
    wf.add_step(AgentStep::RunCommand {
        command: "cargo check --lib 2>&1".to_string(),
        description: "Verify current compile state".to_string(),
    });
    wf.add_step(AgentStep::EditFile {
        path: "".to_string(),
        content: "".to_string(),
    });
    wf.add_step(AgentStep::RunCommand {
        command: "cargo check --lib 2>&1".to_string(),
        description: "Re-check after edits".to_string(),
    });
    wf.add_step(AgentStep::EndTurn { result: None });
    wf
}

pub fn reviewer_workflow() -> AgentWorkflow {
    let mut wf = AgentWorkflow::new(
        "reviewer",
        "Reviewer",
        "Review changes for correctness, style, and potential issues.",
    );
    wf.add_step(AgentStep::RunCommand {
        command: "git diff HEAD".to_string(),
        description: "Show uncommitted changes".to_string(),
    });
    wf.add_step(AgentStep::Think {
        thought: "Evaluating changes for bugs, style issues, and regressions.".to_string(),
    });
    wf.add_step(AgentStep::EndTurn {
        result: Some("Review complete.".to_string()),
    });
    wf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_new_and_steps() {
        let wf = AgentWorkflow::new("test", "Test WF", "Do the thing");
        assert_eq!(wf.id, "test");
        assert_eq!(wf.display_name, "Test WF");
        assert!(wf.steps.is_empty());

        let mut wf = wf;
        wf.add_step(AgentStep::Think {
            thought: "hmm".to_string(),
        });
        assert_eq!(wf.steps.len(), 1);
    }

    #[test]
    fn test_execute_run_command() {
        let mut wf = AgentWorkflow::new("exec-test", "Exec Test", "Run a command");
        wf.add_step(AgentStep::RunCommand {
            command: "echo hello".to_string(),
            description: "test echo".to_string(),
        });
        let result = wf.execute();
        assert!(result.success);
        assert_eq!(result.steps_executed, 1);
        assert!(result.commands_run.contains(&"echo hello".to_string()));
    }

    #[test]
    fn test_execute_end_turn_breaks() {
        let mut wf = AgentWorkflow::new("break-test", "Break Test", "");
        wf.add_step(AgentStep::Think {
            thought: "first".to_string(),
        });
        wf.add_step(AgentStep::EndTurn {
            result: Some("stopped".to_string()),
        });
        wf.add_step(AgentStep::Think {
            thought: "should not run".to_string(),
        });

        let result = wf.execute();
        assert_eq!(result.steps_executed, 2);
        assert_eq!(result.final_output, Some("stopped".to_string()));
    }

    #[test]
    fn test_registry_register_get_list_unregister() {
        let mut reg = WorkflowRegistry::new();
        let wf = AgentWorkflow::new("alpha", "Alpha", "");
        reg.register(wf);

        assert!(reg.get("alpha").is_some());
        assert!(reg.get("missing").is_none());

        let ids = reg.list();
        assert_eq!(ids, vec!["alpha"]);

        assert!(reg.unregister("alpha"));
        assert!(!reg.unregister("alpha"));
        assert!(reg.get("alpha").is_none());
    }

    #[test]
    fn test_load_from_dir() {
        let dir = tempfile::tempdir().expect("failed to create temp dir for workflow test");
        let wf_def = r#"{
            "id": "loaded-wf",
            "display_name": "Loaded WF",
            "instructions": "From JSON",
            "steps": [],
            "metadata": {}
        }"#;
        std::fs::write(dir.path().join("workflow.json"), wf_def)
            .expect("failed to write workflow test file");

        let mut reg = WorkflowRegistry::new();
        let count = reg
            .load_from_dir(dir.path())
            .expect("load_from_dir should succeed");
        assert_eq!(count, 1);
        assert!(reg.get("loaded-wf").is_some());
    }

    #[test]
    fn test_builtin_workflows() {
        let fp = file_picker_workflow();
        assert_eq!(fp.id, "file-picker");

        let pl = planner_workflow();
        assert_eq!(pl.id, "planner");

        let ed = editor_workflow();
        assert_eq!(ed.id, "editor");

        let rv = reviewer_workflow();
        assert_eq!(rv.id, "reviewer");
    }

    #[test]
    fn test_execute_read_file() {
        let dir = tempfile::tempdir().expect("failed to create temp dir for read test");
        let file_path = dir.path().join("sample.txt");
        std::fs::write(&file_path, "file content")
            .expect("failed to write sample file for read test");

        let mut wf = AgentWorkflow::new("read-test", "Read Test", "");
        wf.add_step(AgentStep::ReadFile {
            path: file_path.to_string_lossy().to_string(),
        });
        let result = wf.execute();
        assert!(result.success);
        assert_eq!(result.final_output, Some("file content".to_string()));
    }

    #[test]
    fn test_execute_failed_command() {
        let mut wf = AgentWorkflow::new("fail-test", "Fail Test", "");
        wf.add_step(AgentStep::RunCommand {
            command: "false".to_string(),
            description: "will fail".to_string(),
        });
        let result = wf.execute();
        assert!(!result.success);
    }
}
