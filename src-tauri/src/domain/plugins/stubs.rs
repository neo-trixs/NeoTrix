use crate::domain::{DomainPlugin, ActionSpec, DomainError, serde_json};

// ========== Helper ==========

fn stub_action(name: &str) -> ActionSpec {
    ActionSpec { name: name.into(), description: String::new(), params: vec![], returns: "Value".into() }
}

fn stub_call(action: &str, actions: &[&str]) -> Result<serde_json::Value, DomainError> {
    if actions.contains(&action) {
        Ok(serde_json::json!({ "ok": true, "stub": true }))
    } else {
        Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true })
    }
}

// ========== Agent Plugin ==========

pub struct AgentPlugin;

impl DomainPlugin for AgentPlugin {
    fn name(&self) -> &str { "agent" }
    fn description(&self) -> &str { "Agent 状态、任务、provider 配置" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["status","start","stop","set_provider","test_provider","fetch_models",
             "config","set_project","get_project","health"].iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["status","start","stop","set_provider","test_provider","fetch_models",
                           "config","set_project","get_project","health"])
    }
}

// ========== KB Plugin ==========

pub struct KbPlugin;

impl DomainPlugin for KbPlugin {
    fn name(&self) -> &str { "kb" }
    fn description(&self) -> &str { "知识库：搜索、图谱、KV、地理" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["search","get","graph","stats","kv_set","kv_get","kv_list",
             "geo_points","geo_layers","trajectory_add"].iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["search","get","graph","stats","kv_set","kv_get","kv_list",
                           "geo_points","geo_layers","trajectory_add"])
    }
}

// ========== Plugin Plugin (meta) ==========

pub struct PluginPlugin;

impl DomainPlugin for PluginPlugin {
    fn name(&self) -> &str { "plugin" }
    fn description(&self) -> &str { "插件：安装/卸载/启停、marketplace" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["list","install","uninstall","enable","disable","marketplace","update","config"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["list","install","uninstall","enable","disable","marketplace","update","config"])
    }
}

// ========== Workflow Plugin ==========

pub struct WorkflowPlugin;

impl DomainPlugin for WorkflowPlugin {
    fn name(&self) -> &str { "workflow" }
    fn description(&self) -> &str { "工作流：CRUD、执行、调度" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["list","create","delete","run","status","schedule","import","export"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["list","create","delete","run","status","schedule","import","export"])
    }
}

// ========== Tool Plugin ==========

pub struct ToolPlugin;

impl DomainPlugin for ToolPlugin {
    fn name(&self) -> &str { "tool" }
    fn description(&self) -> &str { "工具：MCP、harness、computer、voice" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["mcp_list","mcp_register","harness_execute","harness_resolve",
             "computer_capture","computer_click","computer_type","voice_synthesize"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["mcp_list","mcp_register","harness_execute","harness_resolve",
                           "computer_capture","computer_click","computer_type","voice_synthesize"])
    }
}

// ========== System Plugin ==========

pub struct SystemPlugin;

impl DomainPlugin for SystemPlugin {
    fn name(&self) -> &str { "system" }
    fn description(&self) -> &str { "系统：窗口、PTY、更新、配置" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["window_minimize","window_maximize","window_close","pty_spawn","pty_write",
             "pty_resize","pty_close","update_check","config_get","config_set"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["window_minimize","window_maximize","window_close","pty_spawn","pty_write",
                           "pty_resize","pty_close","update_check","config_get","config_set"])
    }
}

// ========== Security Plugin ==========

pub struct SecurityPlugin;

impl DomainPlugin for SecurityPlugin {
    fn name(&self) -> &str { "security" }
    fn description(&self) -> &str { "安全：扫描、权限、隐身、企业合规" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["scan","permission_request","permission_respond","stealth_status",
             "audit_log","policy_list","policy_set"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["scan","permission_request","permission_respond","stealth_status",
                           "audit_log","policy_list","policy_set"])
    }
}

// ========== Memory Plugin ==========

pub struct MemoryPlugin;

impl DomainPlugin for MemoryPlugin {
    fn name(&self) -> &str { "memory" }
    fn description(&self) -> &str { "记忆：管理、insights、context" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["list","search","clear","export","import","stats","timeline","insights"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["list","search","clear","export","import","stats","timeline","insights"])
    }
}

// ========== Ext Plugin ==========

pub struct ExtPlugin;

impl DomainPlugin for ExtPlugin {
    fn name(&self) -> &str { "ext" }
    fn description(&self) -> &str { "扩展：远程桥接、频道、协作、通知" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["remote_connect","remote_disconnect","channel_send","channel_list",
             "cowork_start","cowork_stop","notify"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["remote_connect","remote_disconnect","channel_send","channel_list",
                           "cowork_start","cowork_stop","notify"])
    }
}
