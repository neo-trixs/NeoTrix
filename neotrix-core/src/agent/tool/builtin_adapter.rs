use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

use crate::agent::hooks::{HookRegistry, HookEvent, HookContext};
use crate::agent::tool::lifecycle::*;
use crate::agent::tool::impls::*;
use crate::agent::tools::{McpRegistry, McpServer, McpToolDef, McpTransport};

type ToolMap = HashMap<String, Box<dyn AgentTool>>;

fn registry() -> &'static Mutex<ToolMap> {
    static REGISTRY: OnceLock<Mutex<ToolMap>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Global hook registry for tool execution hooks
fn global_hooks() -> &'static Mutex<HookRegistry> {
    static HOOKS: OnceLock<Mutex<HookRegistry>> = OnceLock::new();
    HOOKS.get_or_init(|| Mutex::new(HookRegistry::new()))
}

/// Set the global hook registry (used during initialization)
pub fn set_global_hooks(hooks: HookRegistry) {
    if let Ok(mut g) = global_hooks().lock() {
        *g = hooks;
    }
}

fn dispatch(id: &str, args: &Value) -> Result<String, String> {
    let input = serde_json::to_string(args).unwrap_or_default();

    // Pre-tool hook check
    let pre_ctx = HookContext {
        tool_name: Some(id.to_string()),
        tool_input: Some(input.clone()),
        ..Default::default()
    };
    if let Ok(hooks) = global_hooks().lock() {
        if let Some(skip_msg) = hooks.should_skip(&HookEvent::PreToolUse, &pre_ctx) {
            return Err(skip_msg);
        }
    }

    let start = std::time::Instant::now();
    let result = (|| {
        let map = registry().lock().map_err(|e| e.to_string())?;
        let tool = map.get(id).ok_or_else(|| format!("Tool '{}' not registered", id))?;
        let ctx = ToolContext { input, session_id: "global".into() };
        let output = tool.execute(ctx).map_err(|e| e.to_string())?;
        Ok(output.result)
    })();

    // Post-tool hook trigger
    let duration_ms = start.elapsed().as_millis() as u64;
    let post_ctx = HookContext {
        tool_name: Some(id.to_string()),
        tool_output: result.as_ref().ok().cloned(),
        duration_ms: Some(duration_ms),
        error_message: result.as_ref().err().cloned(),
        ..Default::default()
    };
    if let Ok(hooks) = global_hooks().lock() {
        if result.is_err() {
            hooks.trigger(&HookEvent::OnError, &post_ctx);
        }
        hooks.trigger(&HookEvent::PostToolUse, &post_ctx);
    }

    result
}

// Free functions usable as BuiltinHandler (fn pointer)
pub fn handle_web_scrape(args: &Value) -> Result<String, String> {
    dispatch("web_scrape", args)
}

pub fn handle_security_audit(args: &Value) -> Result<String, String> {
    dispatch("security_audit", args)
}

pub fn handle_react_doctor(args: &Value) -> Result<String, String> {
    dispatch("react_doctor", args)
}

pub fn handle_earn(args: &Value) -> Result<String, String> {
    dispatch("earn", args)
}

pub fn handle_architect(args: &Value) -> Result<String, String> {
    dispatch("architect", args)
}

pub fn handle_image_gen(args: &Value) -> Result<String, String> {
    dispatch("image_gen", args)
}

/// Return a snapshot of all registered tools.
/// Used by ToolUpdateChecker for advisory-only update detection.
pub fn registered_tools() -> Vec<Box<dyn AgentTool>> {
    let map = registry().lock().expect("result");
    map.values().map(|t| {
        // Re-create a fresh instance for each tool type
        let id = t.id().to_string();
        match id.as_str() {
            "web_scrape" => Box::new(WebScrapeTool::new()) as Box<dyn AgentTool>,
            "security_audit" => Box::new(SecurityAuditTool::new()) as Box<dyn AgentTool>,
            "react_doctor" => Box::new(ReactDoctorTool::new()) as Box<dyn AgentTool>,
            "lsp_diagnostics" => Box::new(LspTool::new()) as Box<dyn AgentTool>,
            "earn" => Box::new(EarnTool::new()) as Box<dyn AgentTool>,
            "architect" => Box::new(ArchitectTool::new()) as Box<dyn AgentTool>,
            "image_gen" => Box::new(ImageGenTool::new()) as Box<dyn AgentTool>,
            _ => {
                log::warn!("builtin_adapter: skipping unknown registered tool '{id}'");
                Box::new(WebScrapeTool::new()) as Box<dyn AgentTool> // fallback
            }
        }
    }).collect()
}

/// Initialize the global AgentTool registry with all built-in tools.
/// Idempotent — safe to call multiple times.
pub fn init_builtin_tools() {
    let mut map = registry().lock().expect("result");
    if !map.is_empty() {
        return;
    }
    map.insert("web_scrape".into(), Box::new(WebScrapeTool::new()));
    map.insert("security_audit".into(), Box::new(SecurityAuditTool::new()));
    map.insert("react_doctor".into(), Box::new(ReactDoctorTool::new()));
    map.insert("lsp_diagnostics".into(), Box::new(LspTool::new()));
    map.insert("earn".into(), Box::new(EarnTool::new()));
    map.insert("architect".into(), Box::new(ArchitectTool::new()));
    map.insert("image_gen".into(), Box::new(ImageGenTool::new()));
}

/// Register all adapter handler functions into an McpRegistry.
/// Also reads each tool's manifest.mcp to register real McpToolDef entries.
pub fn register_adapter_tools(registry: &mut McpRegistry) {
    init_builtin_tools();
    registry.register_builtin("web_scrape", handle_web_scrape);
    registry.register_builtin("security_audit", handle_security_audit);
    registry.register_builtin("react_doctor", handle_react_doctor);
    registry.register_builtin("lsp_diagnostics", |args| dispatch("lsp_diagnostics", args));
    registry.register_builtin("earn", handle_earn);
    registry.register_builtin("architect", handle_architect);
    registry.register_builtin("image_gen", handle_image_gen);

    let tool_impls: [Box<dyn AgentTool>; 7] = [
        Box::new(WebScrapeTool::new()),
        Box::new(SecurityAuditTool::new()),
        Box::new(ReactDoctorTool::new()),
        Box::new(LspTool::new()),
        Box::new(EarnTool::new()),
        Box::new(ArchitectTool::new()),
        Box::new(ImageGenTool::new()),
    ];

    for tool in tool_impls {
        let m = tool.manifest();
        if let Some(decl) = &m.mcp {
            registry.register(McpServer {
                name: m.id.clone(),
                transport: McpTransport::Stdio {
                    command: decl.command.clone(),
                    args: decl.args.clone(),
                },
                tools: vec![McpToolDef {
                    name: m.id.clone(),
                    description: m.description.clone(),
                    server_name: m.id.clone(),
                    transport: McpTransport::Stdio {
                        command: decl.command.clone(),
                        args: decl.args.clone(),
                    },
                    input_schema: serde_json::json!({}),
                }],
                healthy: true,
                last_health_check: None,
                latency_ms: 0,
            });
        }
    }
}
