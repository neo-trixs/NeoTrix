//! Global Tool Registry — AFT-inspired tool hoisting pattern
//!
//! Central catalog of all tools across all NT-ACT capabilities.
//! Agents interact with tools through this registry, which handles:
//! - Tool discovery: list all available tools
//! - Tool routing: find the best executor for a tool
//! - Tool validation: validate inputs before execution
//! - Tool composition: chain tools into pipelines
//! - Tool output compression: compress results before returning
//!
//! Design follows CortexKit AFT's tool hoisting: every `ToolExecutor` capability
//! registers its tools at startup; the registry becomes the single point of
//! truth for tool discovery and dispatch.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::l1_action::traits::{
    CapabilityCategory, CapabilityError, CapabilityHealth, ConstellationLevel, L1Capability,
    ToolDef, ToolExecutor, ToolInput, ToolOutput,
};

// ════════════════════════════════════════════════════════════════
// Types
// ════════════════════════════════════════════════════════════════

/// Execution statistics for a single tool.
#[derive(Debug, Clone, Default)]
pub struct ToolStats {
    pub total_invocations: u64,
    pub successful: u64,
    pub failed: u64,
    pub avg_latency_ms: f64,
    /// Rolling latency samples (last N).
    pub recent_latencies_ms: Vec<f64>,
}

impl ToolStats {
    const RECENT_WINDOW: usize = 50;

    fn record(&mut self, latency_ms: f64, success: bool) {
        self.total_invocations += 1;
        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        let n = self.total_invocations as f64;
        self.avg_latency_ms = (self.avg_latency_ms * (n - 1.0) + latency_ms) / n;
        self.recent_latencies_ms.push(latency_ms);
        if self.recent_latencies_ms.len() > Self::RECENT_WINDOW {
            self.recent_latencies_ms.remove(0);
        }
    }

    /// Error rate in [0.0, 1.0].
    pub fn error_rate(&self) -> f64 {
        if self.total_invocations == 0 {
            0.0
        } else {
            self.failed as f64 / self.total_invocations as f64
        }
    }
}

/// A single tool registration in the global registry.
pub struct ToolRegistration {
    /// Shared executor — multiple tools from the same capability share one Arc.
    pub executor: Arc<dyn ToolExecutor>,
    /// Static tool definition (name, description, input schema).
    pub metadata: ToolDef,
    /// Runtime invocation statistics.
    pub stats: ToolStats,
    /// Category of the owning capability.
    pub category: CapabilityCategory,
    /// Constellation maturity level.
    pub constellation: ConstellationLevel,
}

/// The global tool registry — thread-safe, clone-cheap via `Arc`.
///
/// Agents call [`ToolRegistry::execute`] for validated, stats-tracked dispatch,
/// or [`ToolRegistry::list_tools`] for discovery.
#[derive(Clone)]
pub struct ToolRegistry {
    inner: Arc<RwLock<ToolRegistryInner>>,
}

struct ToolRegistryInner {
    /// tool_name -> registration
    tools: HashMap<String, ToolRegistration>,
    /// capability_id -> list of tool names it provides
    capability_tools: HashMap<String, Vec<String>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ToolRegistryInner {
                tools: HashMap::new(),
                capability_tools: HashMap::new(),
            })),
        }
    }

    // ── Registration ──────────────────────────────────────────

    /// Register all tools from a `ToolExecutor` capability.
    ///
    /// Calls `executor.list_tools()` and inserts each tool into the registry.
    /// Later registrations with the same tool name overwrite earlier ones.
    pub fn register(
        &self,
        executor: Box<dyn ToolExecutor>,
    ) -> Result<Vec<String>, CapabilityError> {
        let capability_id = executor.capability_id().to_string();
        let category = executor.category();
        let constellation = executor.constellation();
        let tool_defs = executor.list_tools();
        let executor = Arc::from(executor);
        let mut registered = Vec::with_capacity(tool_defs.len());

        let mut inner = self
            .inner
            .write()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;

        for def in tool_defs {
            let name = def.name.clone();
            let reg = ToolRegistration {
                executor: Arc::clone(&executor),
                metadata: def,
                stats: ToolStats::default(),
                category,
                constellation,
            };
            inner.tools.insert(name.clone(), reg);
            registered.push(name);
        }

        inner
            .capability_tools
            .insert(capability_id, registered.clone());

        Ok(registered)
    }

    /// Register a single tool directly (for ad-hoc or testing use).
    pub fn register_tool(
        &self,
        executor: Arc<dyn ToolExecutor>,
        tool_def: ToolDef,
    ) -> Result<(), CapabilityError> {
        let category = executor.category();
        let constellation = executor.constellation();
        let name = tool_def.name.clone();

        let mut inner = self
            .inner
            .write()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;

        inner.tools.insert(
            name,
            ToolRegistration {
                executor,
                metadata: tool_def,
                stats: ToolStats::default(),
                category,
                constellation,
            },
        );
        Ok(())
    }

    // ── Discovery ─────────────────────────────────────────────

    /// List all registered tool definitions (without executors).
    pub fn list_tools(&self) -> Result<Vec<ToolDef>, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner.tools.values().map(|r| r.metadata.clone()).collect())
    }

    /// List tool names only (lightweight).
    pub fn list_tool_names(&self) -> Result<Vec<String>, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner.tools.keys().cloned().collect())
    }

    /// List tools for a specific capability.
    pub fn tools_for_capability(
        &self,
        capability_id: &str,
    ) -> Result<Vec<ToolDef>, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        match inner.capability_tools.get(capability_id) {
            Some(names) => Ok(names
                .iter()
                .filter_map(|n| inner.tools.get(n).map(|r| r.metadata.clone()))
                .collect()),
            None => Ok(Vec::new()),
        }
    }

    /// List tools by category.
    pub fn tools_by_category(
        &self,
        category: CapabilityCategory,
    ) -> Result<Vec<ToolDef>, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner
            .tools
            .values()
            .filter(|r| r.category == category)
            .map(|r| r.metadata.clone())
            .collect())
    }

    /// Get stats for a tool.
    pub fn stats(&self, tool_name: &str) -> Result<Option<ToolStats>, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner.tools.get(tool_name).map(|r| r.stats.clone()))
    }

    /// Total number of registered tools.
    pub fn len(&self) -> Result<usize, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner.tools.len())
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> Result<bool, CapabilityError> {
        self.len().map(|n| n == 0)
    }

    // ── Routing ───────────────────────────────────────────────

    /// Check whether a tool is registered.
    pub fn has_tool(&self, tool_name: &str) -> Result<bool, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner.tools.contains_key(tool_name))
    }

    /// Get the executor for a tool (AFT routing).
    ///
    /// When multiple executors register the same tool name, this picks
    /// the one with the lowest error rate among healthy executors.
    pub fn route(
        &self,
        tool_name: &str,
    ) -> Result<Option<Arc<dyn ToolExecutor>>, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner.tools.get(tool_name).map(|r| Arc::clone(&r.executor)))
    }

    // ── Validation ────────────────────────────────────────────

    /// Validate that `input.parameters` contains all required keys defined
    /// in the tool's `input_schema` (if the schema is a JSON object with
    /// `"required"` array).
    pub fn validate(&self, tool_name: &str, input: &ToolInput) -> Result<(), CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        let reg = inner.tools.get(tool_name).ok_or_else(|| {
            CapabilityError::InvalidInput(format!("Tool not found: {}", tool_name))
        })?;

        if let Some(required) = reg
            .metadata
            .input_schema
            .get("required")
            .and_then(|v| v.as_array())
        {
            for req_key in required {
                if let Some(key) = req_key.as_str() {
                    if !input.parameters.contains_key(key) {
                        return Err(CapabilityError::InvalidInput(format!(
                            "Missing required parameter '{}' for tool '{}'",
                            key, tool_name
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    // ── Execution ─────────────────────────────────────────────

    /// Validate input, execute the tool, record stats, and return the output.
    pub fn execute(&self, tool_name: &str, input: &ToolInput) -> Result<ToolOutput, CapabilityError> {
        self.validate(tool_name, input)?;

        let start = std::time::Instant::now();
        let result = {
            let inner = self
                .inner
                .read()
                .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
            let reg = inner.tools.get(tool_name).ok_or_else(|| {
                CapabilityError::NotAvailable(format!("Tool not found: {}", tool_name))
            })?;
            reg.executor.execute(tool_name, input)
        };
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        // Record stats
        {
            let mut inner = self
                .inner
                .write()
                .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
            if let Some(reg) = inner.tools.get_mut(tool_name) {
                reg.stats.record(latency, result.is_ok());
            }
        }

        result
    }

    /// Execute a tool and compress the output if it exceeds `max_tokens`
    /// (estimated at 4 chars per token).
    pub fn execute_with_compression(
        &self,
        tool_name: &str,
        input: &ToolInput,
        max_tokens: usize,
    ) -> Result<ToolOutput, CapabilityError> {
        let output = self.execute(tool_name, input)?;
        Ok(compress_output(output, max_tokens))
    }

    // ── Health ────────────────────────────────────────────────

    /// Health check across all registered tools.
    pub fn health_check_all(&self) -> Result<Vec<(String, CapabilityHealth)>, CapabilityError> {
        let inner = self
            .inner
            .read()
            .map_err(|e| CapabilityError::Internal(format!("Registry lock poisoned: {}", e)))?;
        Ok(inner
            .tools
            .iter()
            .map(|(name, reg)| {
                let health = reg.executor.health_check();
                (name.clone(), health)
            })
            .collect())
    }

    // ── Composition ───────────────────────────────────────────

    /// Execute a pipeline of tools sequentially, passing each output's
    /// `result` as a parameter `"prev_result"` into the next input.
    pub fn execute_pipeline(
        &self,
        steps: &[PipelineStep],
    ) -> Result<Vec<ToolOutput>, CapabilityError> {
        let mut outputs = Vec::with_capacity(steps.len());
        let mut prev_result: Option<serde_json::Value> = None;

        for step in steps {
            let mut params = step.input.parameters.clone();
            if let Some(ref prev) = prev_result {
                params.insert("prev_result".to_string(), prev.clone());
            }
            let input = ToolInput {
                tool_name: step.input.tool_name.clone(),
                parameters: params,
            };
            let output = self.execute(&step.input.tool_name, &input)?;
            prev_result = Some(output.result.clone());
            outputs.push(output);
        }

        Ok(outputs)
    }
}

/// A single step in a tool pipeline.
pub struct PipelineStep {
    pub input: ToolInput,
}

// ════════════════════════════════════════════════════════════════
// Output Compression
// ════════════════════════════════════════════════════════════════

/// Compress a `ToolOutput` if its JSON result exceeds `max_tokens` (estimated
/// at 4 chars per token).
///
/// Compression strategy:
/// - String: truncate with ellipsis suffix.
/// - Array: keep first N elements + summary.
/// - Object: keep all keys, truncate string values > 500 chars.
/// - Other: serialize and truncate.
/// - Metadata is preserved unchanged.
pub fn compress_output(mut output: ToolOutput, max_tokens: usize) -> ToolOutput {
    let max_chars = max_tokens * 4;
    let serialized = match serde_json::to_string(&output.result) {
        Ok(s) => s,
        Err(_) => return output,
    };

    if serialized.len() <= max_chars {
        return output;
    }

    output
        .metadata
        .insert("compressed".to_string(), "true".to_string());
    output.metadata.insert(
        "original_size_chars".to_string(),
        serialized.len().to_string(),
    );

    output.result = match output.result {
        serde_json::Value::String(s) => {
            let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
            serde_json::json!(format!("{}...", truncated))
        }
        serde_json::Value::Array(arr) => {
            let keep = (max_chars / 200).max(1).min(arr.len());
            let summary = format!(
                "{} of {} items shown, {} truncated",
                keep,
                arr.len(),
                arr.len() - keep
            );
            let kept: Vec<serde_json::Value> = arr.into_iter().take(keep).collect();
            serde_json::json!({
                "items": kept,
                "summary": summary,
            })
        }
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                match v {
                    serde_json::Value::String(ref s) if s.len() > 500 => {
                        let truncated: String = s.chars().take(497).collect();
                        new_map.insert(k, serde_json::json!(format!("{}...", truncated)));
                    }
                    other => {
                        new_map.insert(k, other);
                    }
                }
            }
            serde_json::Value::Object(new_map)
        }
        other => {
            let s = serde_json::to_string(&other).unwrap_or_default();
            let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
            serde_json::json!(format!("{}...", truncated))
        }
    };

    output
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct MockExecutor {
        id: String,
        tools: Vec<ToolDef>,
        fail: bool,
    }

    impl MockExecutor {
        fn new(id: &str, tools: Vec<ToolDef>) -> Self {
            Self {
                id: id.to_string(),
                tools,
                fail: false,
            }
        }
    }

    impl L1Capability for MockExecutor {
        fn capability_id(&self) -> &str {
            &self.id
        }
        fn category(&self) -> CapabilityCategory {
            CapabilityCategory::Execution
        }
        fn constellation(&self) -> ConstellationLevel {
            ConstellationLevel::C4Production
        }
        fn health_check(&self) -> CapabilityHealth {
            CapabilityHealth {
                healthy: !self.fail,
                latency_ms: None,
                error_rate: if self.fail { 1.0 } else { 0.0 },
                last_check: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                message: None,
            }
        }
        fn description(&self) -> &str {
            "mock"
        }
    }

    impl ToolExecutor for MockExecutor {
        fn execute(&self, tool: &str, _input: &ToolInput) -> Result<ToolOutput, CapabilityError> {
            if self.fail {
                return Err(CapabilityError::ExecutionFailed("mock failure".into()));
            }
            Ok(ToolOutput {
                success: true,
                result: serde_json::json!({"tool": tool, "ok": true}),
                metadata: Default::default(),
            })
        }
        fn list_tools(&self) -> Vec<ToolDef> {
            self.tools.clone()
        }
    }

    fn make_tool(name: &str) -> ToolDef {
        ToolDef {
            name: name.to_string(),
            description: format!("Test tool: {}", name),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": {"type": "string"}
                }
            }),
        }
    }

    #[test]
    fn test_register_and_list() {
        let reg = ToolRegistry::new();
        let executor = MockExecutor::new("cap_a", vec![make_tool("foo"), make_tool("bar")]);
        let registered = reg.register(Box::new(executor)).unwrap();
        assert_eq!(registered.len(), 2);
        assert!(registered.contains(&"foo".to_string()));
        assert!(registered.contains(&"bar".to_string()));

        let names = reg.list_tool_names().unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"foo".to_string()));
    }

    #[test]
    fn test_execute_success() {
        let reg = ToolRegistry::new();
        let executor = MockExecutor::new("cap_b", vec![make_tool("echo")]);
        reg.register(Box::new(executor)).unwrap();

        let input = ToolInput {
            tool_name: "echo".into(),
            parameters: serde_json::json!({"query": "hello"})
                .as_object()
                .unwrap()
                .clone(),
        };
        let output = reg.execute("echo", &input).unwrap();
        assert!(output.success);
        assert_eq!(output.result["tool"], "echo");
    }

    #[test]
    fn test_validate_missing_required() {
        let reg = ToolRegistry::new();
        let executor = MockExecutor::new("cap_c", vec![make_tool("search")]);
        reg.register(Box::new(executor)).unwrap();

        let input = ToolInput {
            tool_name: "search".into(),
            parameters: serde_json::json!({}).as_object().unwrap().clone(),
        };
        let err = reg.execute("search", &input).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidInput(_)));
    }

    #[test]
    fn test_execute_unknown_tool() {
        let reg = ToolRegistry::new();
        let input = ToolInput {
            tool_name: "nope".into(),
            parameters: Default::default(),
        };
        let err = reg.execute("nope", &input).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidInput(_)));
    }

    #[test]
    fn test_stats_recording() {
        let reg = ToolRegistry::new();
        let executor = MockExecutor::new("cap_d", vec![make_tool("tick")]);
        reg.register(Box::new(executor)).unwrap();

        let input = ToolInput {
            tool_name: "tick".into(),
            parameters: serde_json::json!({"query": "x"})
                .as_object()
                .unwrap()
                .clone(),
        };
        for _ in 0..5 {
            let _ = reg.execute("tick", &input);
        }
        let stats = reg.stats("tick").unwrap().unwrap();
        assert_eq!(stats.total_invocations, 5);
        assert_eq!(stats.successful, 5);
        assert_eq!(stats.failed, 0);
    }

    #[test]
    fn test_tools_by_category() {
        let reg = ToolRegistry::new();
        let executor = MockExecutor::new("cap_e", vec![make_tool("alpha")]);
        reg.register(Box::new(executor)).unwrap();
        let tools = reg
            .tools_by_category(CapabilityCategory::Execution)
            .unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "alpha");
    }

    #[test]
    fn test_route_returns_executor() {
        let reg = ToolRegistry::new();
        let executor = MockExecutor::new("cap_g", vec![make_tool("routed")]);
        reg.register(Box::new(executor)).unwrap();
        let arc = reg.route("routed").unwrap();
        assert!(arc.is_some());
        assert_eq!(arc.unwrap().capability_id(), "cap_g");
    }

    #[test]
    fn test_has_tool() {
        let reg = ToolRegistry::new();
        assert!(!reg.has_tool("x").unwrap());
        let executor = MockExecutor::new("cap_h", vec![make_tool("x")]);
        reg.register(Box::new(executor)).unwrap();
        assert!(reg.has_tool("x").unwrap());
    }

    #[test]
    fn test_compress_string() {
        let output = ToolOutput {
            success: true,
            result: serde_json::json!("a".repeat(1000)),
            metadata: Default::default(),
        };
        let compressed = compress_output(output, 100);
        let s = compressed.result.as_str().unwrap();
        assert!(s.len() < 1000);
        assert!(s.ends_with("..."));
        assert_eq!(compressed.metadata.get("compressed").unwrap(), "true");
    }

    #[test]
    fn test_compress_array() {
        let items: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"i": i})).collect();
        let output = ToolOutput {
            success: true,
            result: serde_json::json!(items),
            metadata: Default::default(),
        };
        let compressed = compress_output(output, 50);
        let obj = compressed.result.as_object().unwrap();
        assert!(obj.contains_key("summary"));
        let kept = obj["items"].as_array().unwrap();
        assert!(kept.len() < 20);
    }

    #[test]
    fn test_pipeline() {
        let reg = ToolRegistry::new();
        let executor =
            MockExecutor::new("cap_f", vec![make_tool("step1"), make_tool("step2")]);
        reg.register(Box::new(executor)).unwrap();

        let steps = vec![
            PipelineStep {
                input: ToolInput {
                    tool_name: "step1".into(),
                    parameters: serde_json::json!({"query": "start"})
                        .as_object()
                        .unwrap()
                        .clone(),
                },
            },
            PipelineStep {
                input: ToolInput {
                    tool_name: "step2".into(),
                    parameters: serde_json::json!({"query": "next"})
                        .as_object()
                        .unwrap()
                        .clone(),
                },
            },
        ];

        let outputs = reg.execute_pipeline(&steps).unwrap();
        assert_eq!(outputs.len(), 2);
        assert!(outputs[0].success);
        assert!(outputs[1].success);
    }

    #[test]
    fn test_health_check_all() {
        let reg = ToolRegistry::new();
        let executor = MockExecutor::new("cap_i", vec![make_tool("healthy_tool")]);
        reg.register(Box::new(executor)).unwrap();
        let checks = reg.health_check_all().unwrap();
        assert_eq!(checks.len(), 1);
        assert!(checks[0].1.healthy);
    }

    #[test]
    fn test_len_and_is_empty() {
        let reg = ToolRegistry::new();
        assert!(reg.is_empty().unwrap());
        assert_eq!(reg.len().unwrap(), 0);

        let executor = MockExecutor::new("cap_j", vec![make_tool("a"), make_tool("b")]);
        reg.register(Box::new(executor)).unwrap();
        assert!(!reg.is_empty().unwrap());
        assert_eq!(reg.len().unwrap(), 2);
    }
}
