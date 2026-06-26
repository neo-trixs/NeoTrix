use std::collections::HashMap;
use std::time::Instant;

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use nt_domain::*;

// ── Trait ────────────────────────────────────────────────────────────────

pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Value) -> Result<String, String>;
}

// ── Registry ─────────────────────────────────────────────────────────────

pub struct ToolRegistry {
    tools: HashMap<String, (ToolDescriptor, Box<dyn ToolHandler>)>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        handler: Box<dyn ToolHandler>,
        category: &str,
        input_schema: Value,
        output_schema: Value,
    ) -> Result<(), String> {
        let name = handler.name().to_string();
        if self.tools.contains_key(&name) {
            return Err(format!("Tool '{}' is already registered", name));
        }
        let desc = ToolDescriptor {
            name: name.clone(),
            description: handler.description().to_string(),
            input_schema,
            output_schema,
            category: category.to_string(),
        };
        self.tools.insert(name, (desc, handler));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn ToolHandler> {
        self.tools.get(name).map(|(_, h)| h.as_ref())
    }

    pub fn list_by_category(&self, category: &str) -> Vec<&ToolDescriptor> {
        self.tools
            .values()
            .map(|(desc, _)| desc)
            .filter(|d| d.category == category)
            .collect()
    }

    pub fn list_all(&self) -> Vec<&ToolDescriptor> {
        self.tools.values().map(|(desc, _)| desc).collect()
    }

    pub fn invoke(&self, name: &str, args: Value, _caller_id: Uuid) -> ToolResult {
        let start = Instant::now();
        let inv_id = Uuid::new_v4();

        match self.tools.get(name) {
            None => {
                let elapsed = start.elapsed().as_millis() as u64;
                ToolResult {
                    invocation_id: inv_id,
                    output: String::new(),
                    error: Some(format!("Unknown tool: {}", name)),
                    duration_ms: elapsed,
                    success: false,
                }
            }
            Some((_, handler)) => match handler.execute(args) {
                Ok(output) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    ToolResult {
                        invocation_id: inv_id,
                        output,
                        error: None,
                        duration_ms: elapsed,
                        success: true,
                    }
                }
                Err(err) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    ToolResult {
                        invocation_id: inv_id,
                        output: String::new(),
                        error: Some(err),
                        duration_ms: elapsed,
                        success: false,
                    }
                }
            },
        }
    }
}

// ── ToolService (registry + audit logging) ───────────────────────────────

pub struct ToolService {
    registry: ToolRegistry,
    history: Vec<ToolInvocation>,
}

impl ToolService {
    pub fn new(registry: ToolRegistry) -> Self {
        ToolService {
            registry,
            history: Vec::new(),
        }
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }

    pub fn invoke(&mut self, name: &str, args: Value, caller_id: Uuid) -> ToolResult {
        let invocation = ToolInvocation {
            id: Uuid::new_v4(),
            tool_name: name.to_string(),
            arguments: args.clone(),
            caller_id,
            timestamp: Utc::now(),
        };
        let result = self.registry.invoke(name, args, caller_id);
        self.history.push(invocation);
        result
    }

    pub fn history(&self) -> &[ToolInvocation] {
        &self.history
    }

    pub fn recent_invocations(&self, n: usize) -> Vec<&ToolInvocation> {
        self.history.iter().rev().take(n).collect()
    }
}

// ── Mock Handlers ────────────────────────────────────────────────────────

pub struct EchoHandler;

impl ToolHandler for EchoHandler {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "Returns the input arguments as a JSON string"
    }
    fn execute(&self, args: Value) -> Result<String, String> {
        Ok(serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_string()))
    }
}

pub struct CalcHandler;

impl ToolHandler for CalcHandler {
    fn name(&self) -> &str {
        "calc"
    }
    fn description(&self) -> &str {
        "Evaluates simple arithmetic: add(a,b), sub(a,b), mul(a,b), div(a,b)"
    }
    fn execute(&self, args: Value) -> Result<String, String> {
        let op = args
            .get("op")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'op' field".to_string())?;
        let a = args
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "Missing or invalid 'a' field".to_string())?;
        let b = args
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "Missing or invalid 'b' field".to_string())?;

        let result = match op {
            "add" => a + b,
            "sub" => a - b,
            "mul" => a * b,
            "div" => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                a / b
            }
            _ => return Err(format!("Unknown operation: {}", op)),
        };
        Ok(result.to_string())
    }
}

pub struct FailingHandler;

impl ToolHandler for FailingHandler {
    fn name(&self) -> &str {
        "failing"
    }
    fn description(&self) -> &str {
        "Always returns an error"
    }
    fn execute(&self, _args: Value) -> Result<String, String> {
        Err("Intentional failure".to_string())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_registry_with_handlers() -> ToolRegistry {
        let mut reg = ToolRegistry::new();
        reg.register(
            Box::new(EchoHandler),
            "utility",
            json!({"type": "object"}),
            json!({"type": "string"}),
        )
        .unwrap();
        reg.register(
            Box::new(CalcHandler),
            "math",
            json!({"type": "object"}),
            json!({"type": "string"}),
        )
        .unwrap();
        reg.register(
            Box::new(FailingHandler),
            "test",
            json!({"type": "object"}),
            json!({"type": "string"}),
        )
        .unwrap();
        reg
    }

    #[test]
    fn test_register_and_list() {
        let reg = test_registry_with_handlers();
        let all = reg.list_all();
        assert_eq!(all.len(), 3);

        let names: Vec<&str> = all.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"echo"));
        assert!(names.contains(&"calc"));
        assert!(names.contains(&"failing"));
    }

    #[test]
    fn test_invoke_echo() {
        let reg = test_registry_with_handlers();
        let caller = Uuid::new_v4();
        let args = json!({"hello": "world"});
        let result = reg.invoke("echo", args, caller);
        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.output.contains("hello"));
        assert!(result.output.contains("world"));
    }

    #[test]
    fn test_invoke_failing() {
        let reg = test_registry_with_handlers();
        let caller = Uuid::new_v4();
        let result = reg.invoke("failing", json!({}), caller);
        assert!(!result.success);
        assert_eq!(result.error, Some("Intentional failure".to_string()));
    }

    #[test]
    fn test_list_by_category() {
        let reg = test_registry_with_handlers();
        let math_tools = reg.list_by_category("math");
        assert_eq!(math_tools.len(), 1);
        assert_eq!(math_tools[0].name, "calc");

        let utility_tools = reg.list_by_category("utility");
        assert_eq!(utility_tools.len(), 1);
        assert_eq!(utility_tools[0].name, "echo");

        let unknown = reg.list_by_category("unknown");
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_invoke_unknown() {
        let reg = test_registry_with_handlers();
        let caller = Uuid::new_v4();
        let result = reg.invoke("nonexistent", json!({}), caller);
        assert!(!result.success);
        assert_eq!(result.error, Some("Unknown tool: nonexistent".to_string()));
    }

    #[test]
    fn test_invocation_timing() {
        let reg = test_registry_with_handlers();
        let caller = Uuid::new_v4();
        let result = reg.invoke("echo", json!("fast"), caller);
        assert!(result.success);
        assert!(result.duration_ms > 0 || result.duration_ms == 0); // at least non-negative
        assert!(result.duration_ms < 10_000); // sanity: not stuck
    }

    #[test]
    fn test_double_register_fails() {
        let mut reg = ToolRegistry::new();
        reg.register(
            Box::new(EchoHandler),
            "utility",
            json!({}),
            json!({}),
        )
        .unwrap();
        let err = reg.register(
            Box::new(EchoHandler),
            "utility",
            json!({}),
            json!({}),
        )
        .unwrap_err();
        assert!(err.contains("already registered"));
    }

    #[test]
    fn test_calc_add() {
        let reg = test_registry_with_handlers();
        let caller = Uuid::new_v4();
        let args = json!({"op": "add", "a": 3.0, "b": 4.0});
        let result = reg.invoke("calc", args, caller);
        assert!(result.success);
        assert_eq!(result.output, "7");
    }

    #[test]
    fn test_calc_div_by_zero() {
        let reg = test_registry_with_handlers();
        let caller = Uuid::new_v4();
        let args = json!({"op": "div", "a": 1.0, "b": 0.0});
        let result = reg.invoke("calc", args, caller);
        assert!(!result.success);
        assert_eq!(result.error, Some("Division by zero".to_string()));
    }

    #[test]
    fn test_service_invoke_records_history() {
        let reg = test_registry_with_handlers();
        let mut svc = ToolService::new(reg);
        let caller = Uuid::new_v4();
        let r1 = svc.invoke("echo", json!("first"), caller);
        let r2 = svc.invoke("failing", json!({}), caller);

        assert!(r1.success);
        assert!(!r2.success);
        assert_eq!(svc.history().len(), 2);

        let recent = svc.recent_invocations(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].tool_name, "failing");
    }
}
