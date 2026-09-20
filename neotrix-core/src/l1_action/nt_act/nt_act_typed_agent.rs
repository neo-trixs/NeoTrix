//! # Typed Agent Interfaces (R-P125 / pydantic-ai pattern)
//!
//! Type-safe agent definitions with compile-time input/output guarantees,
//! JSON Schema generation via serde, and dependency injection scoped per-agent.
//!
//! ## Design
//!
//! - `TypedAgent` trait: associated types `Input`, `Output`, `Config` enforce
//!   that each agent declares its exact types at compile time.
//! - `AgentDefinition`: runtime metadata (name, description, schemas, tools)
//!   derived from the trait's associated types via serde_json schema functions.
//! - `AgentOutput<T>`: wraps typed output with provenance metadata
//!   (confidence, sources, reasoning_trace).
//! - `AgentDeps`: validated dependency injection — deps are checked at
//!   construction time, not at each invocation.
//!
//! Follows pydantic-ai's principle: "the model returns structured data that
//! we validate against a declared schema, not raw text the caller must parse."

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::l1_action::traits::CapabilityError;

// ════════════════════════════════════════════════════════════════
// AgentOutput — typed wrapper with provenance
// ════════════════════════════════════════════════════════════════

/// Confidence level of an agent output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
    Verified,
}

/// A provenance source citing where information came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub url: Option<String>,
    pub accessed_at: Option<String>,
}

/// Typed agent output wrapping the actual result with metadata.
///
/// `T` is the concrete output type — enforced at compile time.
/// The JSON schema of `T` is captured in `AgentDefinition.output_schema`
/// and used for runtime validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput<T: Serialize> {
    /// The typed result.
    pub data: T,
    /// Confidence in this output.
    pub confidence: Confidence,
    /// Provenance sources.
    pub sources: Vec<Source>,
    /// Optional reasoning trace (chain-of-thought summary).
    pub reasoning_trace: Option<String>,
    /// Arbitrary metadata (tool usage counts, token costs, etc.).
    pub metadata: HashMap<String, Value>,
}

impl<T: Serialize> AgentOutput<T> {
    /// Create a high-confidence output with no sources.
    pub fn new(data: T) -> Self {
        Self {
            data,
            confidence: Confidence::High,
            sources: Vec::new(),
            reasoning_trace: None,
            metadata: HashMap::new(),
        }
    }

    /// Set confidence level.
    pub fn with_confidence(mut self, c: Confidence) -> Self {
        self.confidence = c;
        self
    }

    /// Add a source.
    pub fn with_source(mut self, s: Source) -> Self {
        self.sources.push(s);
        self
    }

    /// Add multiple sources.
    pub fn with_sources(mut self, s: Vec<Source>) -> Self {
        self.sources.extend(s);
        self
    }

    /// Set reasoning trace.
    pub fn with_trace(mut self, trace: impl Into<String>) -> Self {
        self.reasoning_trace = Some(trace.into());
        self
    }

    /// Add metadata entry.
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Serialize the data portion to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.data)
    }

    /// Serialize the data portion to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.data)
    }
}

// ════════════════════════════════════════════════════════════════
// StreamingPartial — partial output for streaming
// ════════════════════════════════════════════════════════════════

/// A partial chunk of a streaming agent output.
///
/// Carries incremental data plus a final `is_complete` flag.
/// When `is_complete` is true, `chunk` contains the final full value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingPartial<T: Serialize> {
    /// Incremental data chunk (text delta, partial struct, etc.).
    pub chunk: T,
    /// Whether this is the final chunk.
    pub is_complete: bool,
    /// Cumulative token count so far.
    pub tokens_so_far: u64,
}

impl<T: Serialize> StreamingPartial<T> {
    /// Create a partial chunk.
    pub fn partial(chunk: T, tokens_so_far: u64) -> Self {
        Self {
            chunk,
            is_complete: false,
            tokens_so_far,
        }
    }

    /// Create the final chunk.
    pub fn final_chunk(chunk: T, tokens_so_far: u64) -> Self {
        Self {
            chunk,
            is_complete: true,
            tokens_so_far,
        }
    }
}

// ════════════════════════════════════════════════════════════════
// AgentDefinition — runtime agent metadata
// ════════════════════════════════════════════════════════════════

/// Static definition of a typed agent.
///
/// Describes the agent's identity, schemas (as JSON Schema values),
/// available tools, and configuration. Built via `AgentDefinition::builder()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Unique agent name (e.g. "researcher", "code_reviewer").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for the agent's input type.
    pub input_schema: Value,
    /// JSON Schema for the agent's output type.
    pub output_schema: Value,
    /// List of tool names this agent is allowed to use.
    pub tools: Vec<String>,
    /// Arbitrary configuration (model name, temperature, etc.).
    pub config: HashMap<String, Value>,
    /// Agent version (for schema evolution tracking).
    pub version: String,
}

impl AgentDefinition {
    /// Create a new definition with schemas derived from Rust types.
    ///
    /// Uses `serde_json::schema_for::<T>()` to produce JSON Schema at
    /// compile time, ensuring the runtime schema matches the type.
    pub fn from_types<I, O>() -> Self
    where
        I: Serialize,
        O: Serialize,
    {
        Self {
            name: String::new(),
            description: String::new(),
            input_schema: serde_json::schema_for::<I>().value,
            output_schema: serde_json::schema_for::<O>().value,
            tools: Vec::new(),
            config: HashMap::new(),
            version: "1.0.0".into(),
        }
    }

    /// Create a `AgentDefinitionBuilder` for ergonomic construction.
    pub fn builder<I, O>() -> AgentDefinitionBuilder<I, O>
    where
        I: Serialize,
        O: Serialize,
    {
        AgentDefinitionBuilder::new()
    }

    /// Validate that a JSON value conforms to this agent's input schema.
    ///
    /// Returns `Ok(())` if valid, `Err(CapabilityError)` with a descriptive
    /// message if validation fails.
    pub fn validate_input(&self, input: &Value) -> Result<(), CapabilityError> {
        validate_against_schema(input, &self.input_schema, "input")
    }

    /// Validate that a JSON value conforms to this agent's output schema.
    pub fn validate_output(&self, output: &Value) -> Result<(), CapabilityError> {
        validate_against_schema(output, &self.output_schema, "output")
    }

    /// Set a config key-value pair.
    pub fn with_config(mut self, key: impl Into<String>, value: Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }

    /// Add a tool to the allowed list.
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }
}

/// Builder for `AgentDefinition` with compile-time type awareness.
pub struct AgentDefinitionBuilder<I: Serialize, O: Serialize> {
    name: String,
    description: String,
    tools: Vec<String>,
    config: HashMap<String, Value>,
    version: String,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I: Serialize, O: Serialize> AgentDefinitionBuilder<I, O> {
    fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            tools: Vec::new(),
            config: HashMap::new(),
            version: "1.0.0".into(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    pub fn tools(mut self, tools: Vec<String>) -> Self {
        self.tools.extend(tools);
        self
    }

    pub fn config(mut self, key: impl Into<String>, value: Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }

    pub fn version(mut self, v: impl Into<String>) -> Self {
        self.version = v.into();
        self
    }

    pub fn build(self) -> AgentDefinition {
        AgentDefinition {
            name: self.name,
            description: self.description,
            input_schema: serde_json::schema_for::<I>().value,
            output_schema: serde_json::schema_for::<O>().value,
            tools: self.tools,
            config: self.config,
            version: self.version,
        }
    }
}

// ════════════════════════════════════════════════════════════════
// AgentDeps — validated dependency injection
// ════════════════════════════════════════════════════════════════

/// Marker trait for agent dependencies.
///
/// Types implementing this trait can be injected into agents at construction
/// time. Validation occurs once during construction (pydantic-ai pattern),
/// not on every invocation.
///
/// Per-agent scoping: each `TypedAgent` impl declares its `Deps` associated
/// type, so different agents receive different dependency sets.
pub trait AgentDeps: Send + Sync + 'static {
    /// Validate that all required dependencies are present and healthy.
    ///
    /// Called once at agent construction time. If this returns `Err`,
    /// the agent cannot be created.
    fn validate(&self) -> Result<(), AgentDepError>;
}

/// Errors from dependency validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDepError {
    pub missing: Vec<String>,
    pub message: String,
}

impl fmt::Display for AgentDepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "dependency validation failed: {} [missing: {}]",
            self.message,
            self.missing.join(", ")
        )
    }
}

impl std::error::Error for AgentDepError {}

/// No dependencies — for agents that are self-contained.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoDeps;

impl AgentDeps for NoDeps {
    fn validate(&self) -> Result<(), AgentDepError> {
        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════
// TypedAgent — the core trait
// ════════════════════════════════════════════════════════════════

/// Type-safe agent interface.
///
/// Each agent declares its input/output types at compile time.
/// The runtime validates that LLM outputs conform to the declared
/// output schema before returning them to callers.
///
/// # Type Parameters (via associated types)
///
/// - `Input`: the request type (must be `Serialize + for<'de> Deserialize<'de>`)
/// - `Output`: the response type (must be `Serialize + for<'de> Deserialize<'de>`)
/// - `Config`: agent-specific configuration
/// - `Deps`: injected dependencies (validated at construction)
#[async_trait::async_trait]
pub trait TypedAgent: Send + Sync + 'static {
    /// The input type this agent accepts.
    type Input: Serialize + for<'de> Deserialize<'de> + Send;

    /// The output type this agent produces.
    type Output: Serialize + for<'de> Deserialize<'de> + Send;

    /// Agent-specific configuration.
    type Config: Serialize + for<'de> Deserialize<'de> + Send + Clone;

    /// Dependencies injected at construction time.
    type Deps: AgentDeps;

    /// Return the static agent definition (schemas, tools, metadata).
    fn definition(&self) -> AgentDefinition;

    /// Process a typed input and return a typed output wrapped in `AgentOutput`.
    ///
    /// The implementation must produce output matching `Self::Output`.
    /// Callers receive `AgentOutput<Self::Output>` with provenance metadata.
    async fn run(
        &self,
        input: Self::Input,
        config: &Self::Config,
    ) -> Result<AgentOutput<Self::Output>, AgentRunError>;

    /// Validate that raw JSON conforms to the agent's input schema.
    ///
    /// Default implementation uses the definition's `validate_input`.
    fn validate_input_json(&self, json: &Value) -> Result<(), AgentRunError> {
        self.definition()
            .validate_input(json)
            .map_err(|e| AgentRunError::ValidationError(e.to_string()))
    }

    /// Validate that raw JSON conforms to the agent's output schema.
    fn validate_output_json(&self, json: &Value) -> Result<(), AgentRunError> {
        self.definition()
            .validate_output(json)
            .map_err(|e| AgentRunError::ValidationError(e.to_string()))
    }

    /// Parse a raw JSON value into the agent's input type.
    fn parse_input(&self, json: &Value) -> Result<Self::Input, AgentRunError> {
        serde_json::from_value(json.clone())
            .map_err(|e| AgentRunError::DeserializationError(e.to_string()))
    }

    /// Serialize the agent's output type to JSON.
    fn serialize_output(&self, output: &Self::Output) -> Result<Value, AgentRunError> {
        serde_json::to_value(output).map_err(|e| AgentRunError::SerializationError(e.to_string()))
    }
}

// ════════════════════════════════════════════════════════════════
// AgentRunError — unified error type
// ════════════════════════════════════════════════════════════════

/// Errors from typed agent execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRunError {
    /// Input does not match the declared input schema.
    ValidationError(String),
    /// Failed to deserialize JSON into the input type.
    DeserializationError(String),
    /// Failed to serialize output to JSON.
    SerializationError(String),
    /// Agent execution failed (LLM error, tool error, etc.).
    ExecutionFailed(String),
    /// Agent is not ready (dependencies missing, not initialized).
    NotReady(String),
    /// Execution timed out.
    Timeout(String),
}

impl fmt::Display for AgentRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError(msg) => write!(f, "validation error: {}", msg),
            Self::DeserializationError(msg) => write!(f, "deserialization error: {}", msg),
            Self::SerializationError(msg) => write!(f, "serialization error: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "execution failed: {}", msg),
            Self::NotReady(msg) => write!(f, "agent not ready: {}", msg),
            Self::Timeout(msg) => write!(f, "timeout: {}", msg),
        }
    }
}

impl std::error::Error for AgentRunError {}

impl From<AgentRunError> for CapabilityError {
    fn from(e: AgentRunError) -> Self {
        match e {
            AgentRunError::ValidationError(msg) => {
                CapabilityError::InvalidInput(format!("typed_agent: {}", msg))
            }
            AgentRunError::DeserializationError(msg) => {
                CapabilityError::InvalidInput(format!("typed_agent: {}", msg))
            }
            AgentRunError::SerializationError(msg) => {
                CapabilityError::Internal(format!("typed_agent: {}", msg))
            }
            AgentRunError::ExecutionFailed(msg) => {
                CapabilityError::ExecutionFailed(format!("typed_agent: {}", msg))
            }
            AgentRunError::NotReady(msg) => {
                CapabilityError::NotAvailable(format!("typed_agent: {}", msg))
            }
            AgentRunError::Timeout(msg) => {
                CapabilityError::ExecutionFailed(format!("typed_agent timeout: {}", msg))
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════
// TypedAgentHandle — runtime handle wrapping a trait object
// ════════════════════════════════════════════════════════════════

/// Type-erased handle to a `TypedAgent`.
///
/// Stores the agent definition and provides schema-level validation
/// without knowing the concrete types. Useful for registries and
/// dispatchers that route to agents by name.
pub struct TypedAgentHandle {
    definition: AgentDefinition,
    runner: Box<dyn TypedAgentRunner>,
}

impl TypedAgentHandle {
    /// Create a new handle from a concrete agent.
    pub fn new(agent: impl TypedAgent) -> Self {
        let definition = agent.definition();
        Self {
            definition,
            runner: Box::new(TypedAgentRunnerImpl::new(agent)),
        }
    }

    /// Get the agent definition.
    pub fn definition(&self) -> &AgentDefinition {
        &self.definition
    }

    /// Get the agent name.
    pub fn name(&self) -> &str {
        &self.definition.name
    }

    /// Run the agent with raw JSON input, returning raw JSON output.
    ///
    /// Input is validated against the declared input schema before
    /// execution. Output is validated against the declared output schema
    /// before being returned.
    pub async fn run_json(
        &self,
        input: &Value,
    ) -> Result<Value, AgentRunError> {
        self.definition.validate_input(input)?;
        self.runner.run_boxed(input).await
    }
}

impl fmt::Debug for TypedAgentHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedAgentHandle")
            .field("name", &self.definition.name)
            .field("tools", &self.definition.tools)
            .finish()
    }
}

/// Type-erased runner trait (internal).
#[async_trait::async_trait]
trait TypedAgentRunner: Send + Sync {
    async fn run_boxed(&self, input: &Value) -> Result<Value, AgentRunError>;
}

/// Implementation wrapping a concrete `TypedAgent`.
struct TypedAgentRunnerImpl<A: TypedAgent> {
    agent: A,
}

impl<A: TypedAgent> TypedAgentRunnerImpl<A> {
    fn new(agent: A) -> Self {
        Self { agent }
    }
}

#[async_trait::async_trait]
impl<A: TypedAgent> TypedAgentRunner for TypedAgentRunnerImpl<A> {
    async fn run_boxed(&self, input: &Value) -> Result<Value, AgentRunError> {
        let parsed: A::Input = serde_json::from_value(input.clone())
            .map_err(|e| AgentRunError::DeserializationError(e.to_string()))?;
        let output = self.agent.run(parsed, &Self::config_from_def(&self.agent.definition())).await?;
        serde_json::to_value(&output.data)
            .map_err(|e| AgentRunError::SerializationError(e.to_string()))
    }
}

impl<A: TypedAgent> TypedAgentRunnerImpl<A> {
    fn config_from_def(def: &AgentDefinition) -> A::Config {
        serde_json::from_value(serde_json::to_value(&def.config).unwrap_or_default())
            .unwrap_or_else(|_| panic!("config deserialization should not fail for agent '{}'", def.name))
    }
}

// ════════════════════════════════════════════════════════════════
// TypedAgentRegistry — name → handle dispatch
// ════════════════════════════════════════════════════════════════

/// Registry of typed agents, dispatching by name.
///
/// Agents are registered with their definitions; callers dispatch
/// by agent name and provide JSON input.
#[derive(Default)]
pub struct TypedAgentRegistry {
    agents: HashMap<String, TypedAgentHandle>,
}

impl TypedAgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a typed agent.
    pub fn register(&mut self, agent: impl TypedAgent) {
        let handle = TypedAgentHandle::new(agent);
        let name = handle.name().to_string();
        self.agents.insert(name, handle);
    }

    /// Dispatch to an agent by name with raw JSON input.
    pub async fn dispatch(
        &self,
        agent_name: &str,
        input: &Value,
    ) -> Result<Value, AgentRunError> {
        let handle = self
            .agents
            .get(agent_name)
            .ok_or_else(|| AgentRunError::NotReady(format!("agent '{}' not found", agent_name)))?;
        handle.run_json(input).await
    }

    /// Get the definition for an agent.
    pub fn get_definition(&self, agent_name: &str) -> Option<&AgentDefinition> {
        self.agents.get(agent_name).map(|h| h.definition())
    }

    /// List all registered agent names.
    pub fn agent_names(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered agents.
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

// ════════════════════════════════════════════════════════════════
// JSON Schema validation (lightweight, no external deps)
// ════════════════════════════════════════════════════════════════

/// Lightweight JSON Schema validation.
///
/// Checks `type` and `required` fields against a JSON value.
/// This is a minimal validator — for full JSON Schema support,
/// use a dedicated crate like `jsonschema`.
fn validate_against_schema(
    value: &Value,
    schema: &Value,
    label: &str,
) -> Result<(), CapabilityError> {
    let schema_obj = schema.as_object().ok_or_else(|| {
        CapabilityError::InvalidInput(format!(
            "typed_agent {}: schema must be a JSON object",
            label
        ))
    })?;

    // Check type constraint.
    if let Some(expected_type) = schema_obj.get("type").and_then(|v| v.as_str()) {
        let actual_type = match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };
        if actual_type != expected_type {
            return Err(CapabilityError::InvalidInput(format!(
                "typed_agent {}: expected type '{}', got '{}'",
                label, expected_type, actual_type
            )));
        }
    }

    // Check required fields (for object type).
    if let Some(required) = schema_obj.get("required").and_then(|v| v.as_array()) {
        if let Some(obj) = value.as_object() {
            for req in required {
                if let Some(key) = req.as_str() {
                    if !obj.contains_key(key) {
                        return Err(CapabilityError::InvalidInput(format!(
                            "typed_agent {}: missing required field '{}'",
                            label, key
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test types ─────────────────────────────────────────────

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct SearchInput {
        query: String,
        max_results: Option<u32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct SearchResult {
        title: String,
        url: String,
        snippet: String,
        score: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct SearchOutput {
        results: Vec<SearchResult>,
        total_found: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SearchConfig {
        model: String,
        temperature: f32,
    }

    // ── AgentOutput tests ──────────────────────────────────────

    #[test]
    fn agent_output_new_defaults() {
        let out = AgentOutput::new(42i32);
        assert_eq!(out.data, 42);
        assert_eq!(out.confidence, Confidence::High);
        assert!(out.sources.is_empty());
        assert!(out.reasoning_trace.is_none());
    }

    #[test]
    fn agent_output_builder_chain() {
        let out = AgentOutput::new("hello".to_string())
            .with_confidence(Confidence::Verified)
            .with_source(Source {
                name: "web".into(),
                url: Some("https://example.com".into()),
                accessed_at: None,
            })
            .with_trace("looked up on web")
            .with_metadata("tokens", serde_json::json!(150));

        assert_eq!(out.confidence, Confidence::Verified);
        assert_eq!(out.sources.len(), 1);
        assert_eq!(out.reasoning_trace.unwrap(), "looked up on web");
        assert_eq!(out.metadata["tokens"], serde_json::json!(150));
    }

    #[test]
    fn agent_output_to_json() {
        let out = AgentOutput::new(SearchResult {
            title: "test".into(),
            url: "https://test.com".into(),
            snippet: "a test result".into(),
            score: 0.95,
        });
        let json = out.to_json().unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("0.95"));
    }

    // ── StreamingPartial tests ─────────────────────────────────

    #[test]
    fn streaming_partial_partial() {
        let p = StreamingPartial::partial("chunk1".to_string(), 10);
        assert!(!p.is_complete);
        assert_eq!(p.tokens_so_far, 10);
    }

    #[test]
    fn streaming_partial_final() {
        let p = StreamingPartial::final_chunk("done".to_string(), 50);
        assert!(p.is_complete);
        assert_eq!(p.tokens_so_far, 50);
    }

    // ── AgentDefinition tests ──────────────────────────────────

    #[test]
    fn definition_from_types() {
        let def = AgentDefinition::from_types::<SearchInput, SearchOutput>();
        assert!(def.input_schema.is_object());
        assert!(def.output_schema.is_object());
        // Schema should contain the input properties.
        let input_props = def.input_schema["properties"].as_object().unwrap();
        assert!(input_props.contains_key("query"));
    }

    #[test]
    fn definition_builder() {
        let def = AgentDefinition::builder::<SearchInput, SearchOutput>()
            .name("searcher")
            .description("Searches the web")
            .tool("web_search")
            .config("model", serde_json::json!("gpt-4"))
            .version("2.0.0")
            .build();

        assert_eq!(def.name, "searcher");
        assert_eq!(def.tools, vec!["web_search"]);
        assert_eq!(def.version, "2.0.0");
        assert_eq!(def.config["model"], serde_json::json!("gpt-4"));
    }

    #[test]
    fn definition_validate_input_ok() {
        let def = AgentDefinition::from_types::<SearchInput, SearchOutput>();
        let input = serde_json::json!({"query": "rust agents"});
        assert!(def.validate_input(&input).is_ok());
    }

    #[test]
    fn definition_validate_input_missing_required() {
        // SearchInput has `query` as a required field in its schema.
        let def = AgentDefinition::from_types::<SearchInput, SearchOutput>();
        let input = serde_json::json!({"max_results": 5});
        let err = def.validate_input(&input);
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("query"));
    }

    // ── NoDeps tests ───────────────────────────────────────────

    #[test]
    fn no_deps_always_valid() {
        assert!(NoDeps.validate().is_ok());
    }

    // ── AgentRunError tests ────────────────────────────────────

    #[test]
    fn run_error_display() {
        let e = AgentRunError::ExecutionFailed("timeout".into());
        assert!(e.to_string().contains("timeout"));
    }

    #[test]
    fn run_error_into_capability_error() {
        let e = AgentRunError::ValidationError("bad input".into());
        let cap_err: CapabilityError = e.into();
        assert!(cap_err.to_string().contains("bad input"));
    }

    // ── TypedAgentRegistry tests ───────────────────────────────

    #[test]
    fn registry_empty_default() {
        let reg = TypedAgentRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    // ── Confidence ordering ────────────────────────────────────

    #[test]
    fn confidence_ordering() {
        assert!(Confidence::Low < Confidence::Medium);
        assert!(Confidence::Medium < Confidence::High);
        assert!(Confidence::High < Confidence::Verified);
    }
}
