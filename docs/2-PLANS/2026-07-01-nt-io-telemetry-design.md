# NeoTrix OTel Observability Layer — Design Document

**Blind Spot**: No OpenTelemetry observability for LLM calls, tool invocations, or KB retrievals.
**Source**: Langfuse (29k★), OpenTelemetry GenAI semantic conventions (v1.37+, now in `semantic-conventions-genai` repo).
**Implementation Location**: `neotrix-core/src/nt_core_telemetry/`
**9-Layer Location**: L1 Body (IO layer) — telemetry wraps every I/O boundary.

---

## 1. Architecture Overview

Every LLM call, MCP tool invocation, and KB retrieval produces a **span**. Spans nest into **traces** per HTTP request or REPL command. The span tree mirrors Langfuse's Session → Trace → Observation model:

```
Trace (REPL command or HTTP request)
├── LLM Span (GatewayV2.complete())
│   ├── Tool Span (McpRegistry.call_tool())
│   │   └── Retrieval Span (KB search)
│   └── Tool Span (another tool)
└── LLM Span (second call)
```

### Span Lifecycle

1. `Tracer::start_span()` — creates span with `trace_id`, `span_id`, `parent_span_id`
2. Attributes are set via `set_attribute()` during execution
3. Events are recorded via `add_event()` (e.g. retry, fallback, timeout)
4. `Tracer::end_span()` — records duration, computes cost, flushes

---

## 2. Core Types

```rust
// neotrix-core/src/nt_core_telemetry/types.rs

use std::collections::HashMap;
use std::time::Instant;

/// 16-byte hex string (128-bit trace ID per W3C Trace Context)
pub type TraceId = String;
/// 8-byte hex string (64-bit span ID)
pub type SpanId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// Internal computation (E8 reasoning, GWT cycle, PRM scoring)
    Internal,
    /// LLM provider call (complete or stream)
    Llm,
    /// MCP tool invocation
    Tool,
    /// Knowledge base retrieval (FTS5 or embedding search)
    Retrieval,
    /// Agent-to-agent handoff or delegation
    Handoff,
}

#[derive(Debug, Clone)]
pub struct TraceSpan {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub name: String,
    pub kind: SpanKind,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub attributes: HashMap<String, AttributeValue>,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
    pub resource: ResourceAttributes,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    StringArray(Vec<String>),
    IntArray(Vec<i64>),
}

#[derive(Debug, Clone)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: Instant,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error(String),
}

/// Resource attributes applied to every span from this service instance
#[derive(Debug, Clone)]
pub struct ResourceAttributes {
    pub service_name: String,
    pub service_version: String,
    pub deployment_environment: String,
    pub host_name: String,
}
```

---

## 3. OpenTelemetry GenAI Semantic Conventions

Based on the official [OpenTelemetry GenAI semantic conventions](https://github.com/open-telemetry/semantic-conventions-genai) (stabilized 2026). All constants follow the `gen_ai.*` namespace.

```rust
// neotrix-core/src/nt_core_telemetry/semconv.rs

// ── Request attributes ──
pub const GEN_AI_SYSTEM: &str = "gen_ai.system";
pub const GEN_AI_REQUEST_MODEL: &str = "gen_ai.request.model";
pub const GEN_AI_RESPONSE_MODEL: &str = "gen_ai.response.model";
pub const GEN_AI_REQUEST_MAX_TOKENS: &str = "gen_ai.request.max_tokens";
pub const GEN_AI_REQUEST_TEMPERATURE: &str = "gen_ai.request.temperature";
pub const GEN_AI_REQUEST_TOP_P: &str = "gen_ai.request.top_p";
pub const GEN_AI_REQUEST_FREQUENCY_PENALTY: &str = "gen_ai.request.frequency_penalty";
pub const GEN_AI_REQUEST_PRESENCE_PENALTY: &str = "gen_ai.request.presence_penalty";
pub const GEN_AI_REQUEST_SEED: &str = "gen_ai.request.seed";
pub const GEN_AI_REQUEST_STOP_SEQUENCES: &str = "gen_ai.request.stop_sequences";
pub const GEN_AI_REQUEST_REASONING_EFFORT: &str = "gen_ai.request.reasoning_effort";

// ── Operation attributes ──
pub const GEN_AI_OPERATION_NAME: &str = "gen_ai.operation.name";

// ── Response attributes ──
pub const GEN_AI_RESPONSE_FINISH_REASONS: &str = "gen_ai.response.finish_reasons";
pub const GEN_AI_RESPONSE_ID: &str = "gen_ai.response.id";

// ── Usage (token counts) ──
pub const GEN_AI_USAGE_PROMPT_TOKENS: &str = "gen_ai.usage.prompt_tokens";
pub const GEN_AI_USAGE_COMPLETION_TOKENS: &str = "gen_ai.usage.completion_tokens";
pub const GEN_AI_USAGE_TOTAL_TOKENS: &str = "gen_ai.usage.total_tokens";
pub const GEN_AI_USAGE_REASONING_TOKENS: &str = "gen_ai.usage.reasoning_tokens";
pub const GEN_AI_USAGE_COST: &str = "gen_ai.usage.cost_usd";

// ── Tool call attributes ──
pub const GEN_AI_TOOL_CALL_ID: &str = "gen_ai.tool.call.id";
pub const GEN_AI_TOOL_CALL_NAME: &str = "gen_ai.tool.call.name";
pub const GEN_AI_TOOL_CALL_ARGUMENTS: &str = "gen_ai.tool.call.arguments";
pub const GEN_AI_TOOL_CALL_RESULT: &str = "gen_ai.tool.call.result";

// ── Retrieval attributes ──
pub const GEN_AI_RETRIEVAL_SOURCE: &str = "gen_ai.retrieval.source";
pub const GEN_AI_RETRIEVAL_DOCUMENT_COUNT: &str = "gen_ai.retrieval.document_count";
pub const GEN_AI_RETRIEVAL_RESULT_ID: &str = "gen_ai.retrieval.result.id";
pub const GEN_AI_RETRIEVAL_RESULT_SCORE: &str = "gen_ai.retrieval.result.score";

// ── Provider attributes ──
pub const GEN_AI_PROVIDER_NAME: &str = "gen_ai.provider.name";
pub const GEN_AI_PROVIDER_VERSION: &str = "gen_ai.provider.version";

// ── Agent attributes (proposed 2026 OTel agent conventions) ──
pub const GEN_AI_AGENT_ID: &str = "gen_ai.agent.id";
pub const GEN_AI_AGENT_NAME: &str = "gen_ai.agent.name";
pub const GEN_AI_TASK_ID: &str = "gen_ai.task.id";
pub const GEN_AI_ACTION_TYPE: &str = "gen_ai.action.type";

// ── Custom NeoTrix attributes ──
pub const NEOTRIX_E8_STATE: &str = "neotrix.e8.state";
pub const NEOTRIX_E8_MODE: &str = "neotrix.e8.mode";
pub const NEOTRIX_GWT_TEMPERATURE: &str = "neotrix.gwt.temperature";
pub const NEOTRIX_GWT_RESONANCE_CYCLES: &str = "neotrix.gwt.resonance_cycles";
pub const NEOTRIX_GATEWAY_RETRY_COUNT: &str = "neotrix.gateway.retry_count";
pub const NEOTRIX_GATEWAY_FALLBACK_PROVIDER: &str = "neotrix.gateway.fallback_provider";
```

---

## 4. Tracer Trait and Implementations

```rust
// neotrix-core/src/nt_core_telemetry/tracer.rs

pub trait Tracer: Send + Sync {
    fn start_span(&self, name: &str, kind: SpanKind) -> TraceSpan;
    fn end_span(&self, span: TraceSpan);
    fn add_event(&self, span_id: &SpanId, event: SpanEvent);
    fn set_attribute(&self, span_id: &SpanId, key: &str, value: AttributeValue);
    fn get_trace(&self, trace_id: &TraceId) -> Option<Trace>;
    fn current_span_id(&self) -> Option<SpanId>;
    fn flush(&self);
}
```

### 4.1 NoopTracer (Zero-Cost When Disabled)

```rust
pub struct NoopTracer;

impl Tracer for NoopTracer {
    fn start_span(&self, name: &str, kind: SpanKind) -> TraceSpan {
        // Returns a minimal span; no allocation for attributes/events
        TraceSpan {
            trace_id: String::new(),
            span_id: String::new(),
            parent_span_id: None,
            name: name.to_string(),
            kind,
            start_time: Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Unset,
            resource: ResourceAttributes::empty(),
        }
    }

    fn end_span(&self, _span: TraceSpan) {} // no-op
    fn add_event(&self, _span_id: &SpanId, _event: SpanEvent) {} // no-op
    fn set_attribute(&self, _span_id: &SpanId, _key: &str, _value: AttributeValue) {} // no-op
    fn get_trace(&self, _trace_id: &TraceId) -> Option<Trace> { None }
    fn current_span_id(&self) -> Option<SpanId> { None }
    fn flush(&self) {}
}
```

### 4.2 ConsoleTracer (CLI Dev Mode)

Writes JSON-per-line ndjson to stdout or a file. Each span is one JSON object on one line. This is the primary output format for CLI `neotrix --trace`:

```json
{
  "timestamp": "2026-07-01T10:30:00.123Z",
  "trace_id": "7b8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a",
  "span_id": "a1b2c3d4e5f6a7b8",
  "parent_span_id": "f0e1d2c3b4a59687",
  "name": "chat gpt-4o",
  "kind": "LLM",
  "duration_ms": 2347,
  "status": "OK",
  "attributes": {
    "gen_ai.request.model": "gpt-4o",
    "gen_ai.usage.prompt_tokens": 1234,
    "gen_ai.usage.completion_tokens": 89,
    "gen_ai.usage.total_tokens": 1323,
    "gen_ai.usage.cost_usd": 0.003975,
    "gen_ai.response.finish_reasons": ["stop"]
  },
  "events": [
    {"name": "retry", "timestamp": "2026-07-01T10:30:00.100Z", "attributes": {"attempt": 2}}
  ]
}
```

Key design: append-only, lock-free via `crossbeam_channel`. A background writer thread drains the channel and writes to the output. This avoids blocking the hot LLM path.

```rust
pub struct ConsoleTracer {
    span_sender: crossbeam_channel::Sender<SpanEvent>,
    writer_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    active_spans: DashMap<SpanId, TraceSpan>,
}

impl ConsoleTracer {
    pub fn new(output: OutputTarget) -> Self { ... }
}

enum OutputTarget {
    Stdout,
    File(PathBuf),
}
```

### 4.3 OTelTracer (Production via OTLP)

Uses the `opentelemetry` and `opentelemetry_otlp` crates as optional dependencies behind `#[cfg(feature = "tracing")]`. Exports spans via OTLP/gRPC or OTLP/HTTP to any OTel-compatible backend (Langfuse, Datadog, Grafana Tempo).

```rust
#[cfg(feature = "tracing")]
pub struct OTelTracer {
    tracer: opentelemetry::trace::Tracer,
    provider: opentelemetry_sdk::trace::SdkTracerProvider,
}

#[cfg(feature = "tracing")]
impl OTelTracer {
    pub fn new(endpoint: &str, headers: HashMap<String, String>) -> Self {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_headers(headers)
            .build()
            .unwrap();

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_resource(Resource::new([
                KeyValue::new("service.name", "neotrix"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ]))
            .build();

        let tracer = provider.tracer("neotrix-tracer");
        OTelTracer { tracer, provider }
    }
}

impl Tracer for OTelTracer {
    // delegates to opentelemetry::trace::Span
}
```

### 4.4 LangfuseTracer (Managed Platform)

Wraps OTelTracer with Langfuse-specific routing. Sends OTLP to `https://cloud.langfuse.com/api/public/otel` with HTTP Basic Auth using `LANGFUSE_PUBLIC_KEY`/`LANGFUSE_SECRET_KEY`.

Langfuse maps the OTel span tree to its own data model:
- **Trace** → Langfuse Trace (session-level grouping)
- **LLM Span** → Langfuse Generation (with token usage and cost)
- **Tool Span** → Langfuse Span (with input/output)
- **Retrieval Span** → Langfuse Span (with retrieval metadata)

---

## 5. CostTracker

```rust
// neotrix-core/src/nt_core_telemetry/cost.rs

/// Standard pricing model per 1M tokens (hardcoded, update quarterly)
/// Sources: OpenAI 2026-06, Anthropic 2026-06, Google 2026-06
const PRICING_TABLE: &[(&str, f64, f64)] = &[
    ("gpt-4o",              2.50,  10.00),
    ("gpt-4o-mini",         0.15,   0.60),
    ("gpt-4.1",             2.00,   8.00),
    ("gpt-4.1-mini",        0.40,   1.60),
    ("gpt-4.1-nano",        0.10,   0.40),
    ("o3",                 10.00,  40.00),
    ("o4-mini",             1.10,   4.40),
    ("claude-sonnet-4",     3.00,  15.00),
    ("claude-haiku-3.5",    0.80,   4.00),
    ("claude-opus-4",      15.00,  75.00),
    ("gemini-2.0-flash",    0.10,   0.40),
    ("gemini-2.5-pro",      1.25,  10.00),
    ("deepseek-v3",         0.27,   1.10),
    ("deepseek-r1",         0.55,   2.19),
];

/// Fuzzy model name matching: "gpt-4o-2026-05-13" → "gpt-4o"
fn normalize_model(name: &str) -> &str {
    PRICING_TABLE
        .iter()
        .find(|(prefix, _, _)| name.starts_with(prefix))
        .map(|(prefix, _, _)| *prefix)
        .unwrap_or("unknown")
}

#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_cost_usd: f64,
    pub total_requests: u64,
    pub per_model: HashMap<String, ModelCost>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelCost {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost_usd: f64,
    pub request_count: u64,
}

impl CostTracker {
    pub fn record(&mut self, model: &str, prompt_tokens: u64, completion_tokens: u64) {
        self.total_prompt_tokens += prompt_tokens;
        self.total_completion_tokens += completion_tokens;
        self.total_requests += 1;

        let normalized = normalize_model(model).to_string();
        let entry = self.per_model.entry(normalized).or_default();
        entry.prompt_tokens += prompt_tokens;
        entry.completion_tokens += completion_tokens;
        entry.request_count += 1;

        if let Some((_, prompt_price, completion_price)) = PRICING_TABLE
            .iter()
            .find(|(prefix, _, _)| model.starts_with(prefix))
        {
            let cost = (prompt_tokens as f64 / 1_000_000.0) * prompt_price
                + (completion_tokens as f64 / 1_000_000.0) * completion_price;
            entry.cost_usd += cost;
            self.total_cost_usd += cost;
        }
    }

    pub fn average_cost_per_request(&self) -> f64 {
        if self.total_requests == 0 { 0.0 } else { self.total_cost_usd / self.total_requests as f64 }
    }

    pub fn total_tokens(&self) -> u64 {
        self.total_prompt_tokens + self.total_completion_tokens
    }

    pub fn estimated_monthly_cost(&self, days_active: f64) -> f64 {
        self.total_cost_usd * (30.0 / days_active)
    }
}
```

---

## 6. Integration Points

### 6.1 GatewayV2 — LLM Span

In `neotrix-core/src/nt_io_provider/gateway.rs`, `complete_with_selection()`:

```rust
pub async fn complete_with_selection(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
    let span_id = self.tracer.start_span(
        &format!("chat {}", request.model),
        SpanKind::Llm,
    );

    // Set GenAI semantic convention attributes
    self.tracer.set_attribute(&span_id, GEN_AI_SYSTEM, AttributeValue::String(self.provider_name().to_string()));
    self.tracer.set_attribute(&span_id, GEN_AI_REQUEST_MODEL, AttributeValue::String(request.model.clone()));
    self.tracer.set_attribute(&span_id, GEN_AI_REQUEST_MAX_TOKENS, AttributeValue::Int(request.max_tokens as i64));
    self.tracer.set_attribute(&span_id, GEN_AI_REQUEST_TEMPERATURE, AttributeValue::Double(request.temperature as f64));
    self.tracer.set_attribute(&span_id, GEN_AI_OPERATION_NAME, AttributeValue::String("chat".to_string()));

    let result = self.complete_inner(request).await;

    match &result {
        Ok(response) => {
            self.tracer.set_attribute(&span_id, GEN_AI_RESPONSE_MODEL, AttributeValue::String(response.model.clone()));
            self.tracer.set_attribute(&span_id, GEN_AI_USAGE_PROMPT_TOKENS, AttributeValue::Int(response.usage.prompt_tokens as i64));
            self.tracer.set_attribute(&span_id, GEN_AI_USAGE_COMPLETION_TOKENS, AttributeValue::Int(response.usage.completion_tokens as i64));
            self.tracer.set_attribute(&span_id, GEN_AI_USAGE_TOTAL_TOKENS, AttributeValue::Int((response.usage.prompt_tokens + response.usage.completion_tokens) as i64));
            self.tracer.set_attribute(&span_id, GEN_AI_RESPONSE_FINISH_REASONS, AttributeValue::StringArray(response.finish_reasons.clone()));
            self.tracer.set_attribute(&span_id, GEN_AI_USAGE_COST, AttributeValue::Double(compute_cost(&response.model, response.usage.prompt_tokens, response.usage.completion_tokens)));

            // Track cost
            self.cost_tracker.lock().record(&response.model, response.usage.prompt_tokens, response.usage.completion_tokens);
        }
        Err(e) => {
            self.tracer.add_event(&span_id, SpanEvent {
                name: "error".to_string(),
                timestamp: Instant::now(),
                attributes: HashMap::from([("error.message".to_string(), AttributeValue::String(e.to_string()))]),
            });
        }
    }

    self.tracer.end_span(TraceSpan { /* ... */ });
    result
}
```

### 6.2 McpRegistry — Tool Span

```rust
pub async fn call_tool(&self, name: &str, args: Value) -> Result<McpToolResponse> {
    let child_span_id = self.tracer.start_span(
        &format!("tool {}", name),
        SpanKind::Tool,
    );

    self.tracer.set_attribute(&child_span_id, GEN_AI_TOOL_CALL_NAME, AttributeValue::String(name.to_string()));
    self.tracer.set_attribute(&child_span_id, GEN_AI_TOOL_CALL_ARGUMENTS, AttributeValue::String(args.to_string()));

    let result = self.call_tool_inner(name, args).await;

    match &result {
        Ok(response) => {
            self.tracer.set_attribute(&child_span_id, GEN_AI_TOOL_CALL_RESULT, AttributeValue::String(response.content.to_string()));
        }
        Err(e) => { /* record error event */ }
    }

    self.tracer.end_span(/* ... */);
    result
}
```

### 6.3 KB Search — Retrieval Span

In `nt_memory_search.rs`, `search_fused()`:

```rust
pub async fn search_fused(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
    let span_id = self.tracer.start_span(
        &format!("kb_search {}", &query[..query.len().min(50)]),
        SpanKind::Retrieval,
    );

    self.tracer.set_attribute(&span_id, GEN_AI_RETRIEVAL_SOURCE, AttributeValue::String("knowledge_base".to_string()));
    self.tracer.set_attribute(&span_id, GEN_AI_RETRIEVAL_DOCUMENT_COUNT, AttributeValue::Int(top_k as i64));

    let results = self.search_fused_inner(query, top_k).await?;

    self.tracer.set_attribute(&span_id, GEN_AI_RETRIEVAL_DOCUMENT_COUNT, AttributeValue::Int(results.len() as i64));
    for (i, r) in results.iter().enumerate().take(5) {
        self.tracer.set_attribute(&span_id,
            &format!("gen_ai.retrieval.result.{}.id", i),
            AttributeValue::String(r.id.clone()));
        self.tracer.set_attribute(&span_id,
            &format!("gen_ai.retrieval.result.{}.score", i),
            AttributeValue::Double(r.score as f64));
    }

    self.tracer.end_span(/* ... */);
    Ok(results)
}
```

### 6.4 GWT Resonance — Internal Span

```rust
pub async fn resonate(&self, state: &E8State) -> Result<f64> {
    let span_id = self.tracer.start_span("gwt_resonance", SpanKind::Internal);
    self.tracer.set_attribute(&span_id, NEOTRIX_E8_STATE, AttributeValue::String(state.to_string()));
    self.tracer.set_attribute(&span_id, NEOTRIX_E8_MODE, AttributeValue::String(state.mode.to_string()));

    let (temperature, cycles) = self.resonate_inner(state).await;

    self.tracer.set_attribute(&span_id, NEOTRIX_GWT_TEMPERATURE, AttributeValue::Double(temperature as f64));
    self.tracer.set_attribute(&span_id, NEOTRIX_GWT_RESONANCE_CYCLES, AttributeValue::Int(cycles as i64));
    self.tracer.end_span(/* ... */);
    Ok(temperature)
}
```

---

## 7. Sampling Strategy

OTel-compatible head-based sampling to control volume in production:

```rust
pub enum Sampler {
    /// Record every span (dev mode)
    AlwaysOn,
    /// Record no spans (CI, batch jobs)
    AlwaysOff,
    /// Record a random fraction [0.0, 1.0]
    TraceIdRatio(f64),
    /// Record specified operation names, drop everything else
    Filter(HashSet<String>),
}

impl Sampler {
    pub fn should_sample(&self, trace_id: &TraceId) -> bool {
        match self {
            Sampler::AlwaysOn => true,
            Sampler::AlwaysOff => false,
            Sampler::TraceIdRatio(rate) => {
                // Hash the trace_id to [0.0, 1.0) for deterministic sampling
                let hash = fxhash::hash64(trace_id);
                (hash as f64 / u64::MAX as f64) < *rate
            }
            Sampler::Filter(ops) => ops.contains(/* current span name */),
        }
    }
}
```

Default: `TraceIdRatio(0.1)` in production (10% of traces sampled). CLI `--trace` overrides to `AlwaysOn`.

---

## 8. CLI Integration

```rust
// neotrix-core/src/cli/entry.rs

#[arg(long, help = "Enable per-request OpenTelemetry tracing")]
trace: bool,

#[arg(long, help = "Trace output format [console, otlp, langfuse]")]
trace_format: Option<String>,

#[arg(long, help = "OTLP endpoint URL")]
trace_endpoint: Option<String>,
```

```rust
pub fn create_tracer(config: &TraceConfig) -> Arc<dyn Tracer> {
    match config.format {
        TraceFormat::Console => Arc::new(ConsoleTracer::new(OutputTarget::Stdout)),
        TraceFormat::Otlp => Arc::new(OTelTracer::new(&config.endpoint, config.headers.clone())),
        TraceFormat::Langfuse => Arc::new(LangfuseTracer::from_env()),
        TraceFormat::None => Arc::new(NoopTracer),
    }
}
```

---

## 9. File Layout

```
neotrix-core/src/nt_core_telemetry/
├── mod.rs            # re-exports, #[cfg(feature = "tracing")] gating
├── types.rs          # TraceSpan, SpanKind, AttributeValue, SpanEvent, SpanStatus
├── semconv.rs        # OTel GenAI semantic convention constants
├── tracer.rs         # Tracer trait
├── noop.rs           # NoopTracer
├── console.rs        # ConsoleTracer (ndjson stdout/file)
├── otel.rs           # OTelTracer (OTLP, needs opentelemetry crate)
├── langfuse.rs       # LangfuseTracer (wraps OTel with Langfuse routing)
├── cost.rs           # CostTracker with pricing table
├── sampler.rs        # Sampler trait + TraceIdRatio sampler
└── macros.rs         # convenience macros: trace_span!, add_event!
```

---

## 10. Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_noop_tracer_zero_overhead() {
        let tracer = NoopTracer;
        let span = tracer.start_span("test", SpanKind::Internal);
        tracer.end_span(span);
        // No panics, no allocations beyond minimal struct
    }

    #[test]
    fn test_cost_tracker_gpt4o() {
        let mut ct = CostTracker::default();
        ct.record("gpt-4o", 1000, 200);
        // 1000 prompt = 1000/1M * 2.50 = 0.0025
        // 200 completion = 200/1M * 10 = 0.002
        // Total = 0.0045
        assert!((ct.total_cost_usd - 0.0045).abs() < 0.0001);
    }

    #[test]
    fn test_cost_tracker_model_normalization() {
        let mut ct = CostTracker::default();
        ct.record("gpt-4o-2026-05-13", 500, 100);
        assert_eq!(ct.per_model.get("gpt-4o").unwrap().request_count, 1);
    }

    #[test]
    fn test_span_hierarchy() {
        let tracer = ConsoleTracer::new(OutputTarget::File("/tmp/test_trace.json".into()));
        let parent = tracer.start_span("chat", SpanKind::Llm);
        let child = tracer.start_span("tool_search", SpanKind::Tool);
        assert_eq!(child.parent_span_id.unwrap(), parent.span_id);
    }

    #[test]
    fn test_sampler_trace_id_ratio() {
        let sampler = Sampler::TraceIdRatio(1.0);
        assert!(sampler.should_sample("anything"));
        let sampler = Sampler::TraceIdRatio(0.0);
        assert!(!sampler.should_sample("anything"));
    }
}
```

---

## 11. Implementation Plan

| Phase | Description | Files | Effort |
|-------|-------------|-------|--------|
| 1 | Core types + NoopTracer + ConsoleTracer | types.rs, semconv.rs, tracer.rs, noop.rs, console.rs | 2 days |
| 2 | GatewayV2 integration + CostTracker | cost.rs, gateway.rs (modify) | 2 days |
| 3 | OTel exporter (optional dep) + LangfuseTracer | otel.rs, langfuse.rs, Cargo.toml | 2 days |
| 4 | CLI `--trace` flag + sampling | sampler.rs, entry.rs (modify) | 1 day |
| 5 | McpRegistry + KB search + GWT integration | mcp/mod.rs, search.rs, gwt.rs (modify) | 2 days |

**Total: ~9 days**

---

## 12. References

1. [OpenTelemetry GenAI Semantic Conventions](https://github.com/open-telemetry/semantic-conventions-genai) — official repo
2. [Langfuse + OTel Integration](https://langfuse.com/integrations/native/opentelemetry) — OTLP to Langfuse data model mapping
3. [OpenLLMetry semantic conventions](https://www.traceloop.com/docs/openllmetry/contributing/semantic-conventions) — `gen_ai.*` attribute reference
4. [Langfuse Python SDK v4](https://github.com/langfuse/langfuse-python) — OTel-based SDK architecture
5. [Datadog LLM Observability + OTel](https://www.datadoghq.com/blog/llm-otel-semantic-convention) — production OTel GenAI adoption
