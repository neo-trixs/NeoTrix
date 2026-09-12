use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

static NEXT_SPAN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    Internal,
    Handoff,
    Llm,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug)]
pub struct Span {
    id: u64,
    name: String,
    kind: SpanKind,
    _parent_id: Option<u64>,
    attributes: Mutex<Vec<(String, AttributeValue)>>,
    start: Instant,

    // GenAI semconv dedicated fields
    pub gen_ai_operation_name: Option<String>,
    pub gen_ai_provider_name: Option<String>,
    pub gen_ai_request_stream: bool,
    pub gen_ai_response_id: Option<String>,
    pub gen_ai_response_model: Option<String>,
    pub gen_ai_response_finish_reasons: Vec<String>,
    pub gen_ai_usage_input_tokens: Option<u64>,
    pub gen_ai_usage_output_tokens: Option<u64>,
}

impl Span {
    fn new(name: &str, kind: SpanKind, _parent_id: Option<u64>) -> Self {
        Self {
            id: NEXT_SPAN_ID.fetch_add(1, Ordering::Relaxed),
            name: name.to_string(),
            kind,
            _parent_id,
            attributes: Mutex::new(Vec::new()),
            start: Instant::now(),
            gen_ai_operation_name: None,
            gen_ai_provider_name: None,
            gen_ai_request_stream: false,
            gen_ai_response_id: None,
            gen_ai_response_model: None,
            gen_ai_response_finish_reasons: Vec::new(),
            gen_ai_usage_input_tokens: None,
            gen_ai_usage_output_tokens: None,
        }
    }

    pub fn set_attribute(&self, key: &str, value: AttributeValue) {
        if let Ok(mut attrs) = self.attributes.lock() {
            attrs.push((key.to_string(), value));
        }
    }

    /// Record the requested GenAI model as a span attribute (GenAI semconv `gen_ai.request.model`).
    pub fn set_gen_ai_request_model(&self, model: &str) {
        self.set_attribute(
            "gen_ai.request.model",
            AttributeValue::String(model.to_string()),
        );
    }

    /// Record the GenAI system name as a span attribute (`gen_ai.system`).
    pub fn set_gen_ai_system(&self, system: &str) {
        self.set_attribute("gen_ai.system", AttributeValue::String(system.to_string()));
    }

    /// Look up a recorded attribute (clones the value; used for observability/tests).
    pub fn attribute(&self, key: &str) -> Option<AttributeValue> {
        self.attributes
            .lock()
            .ok()
            .and_then(|attrs| attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()))
    }

    // GenAI semconv setter methods

    /// GenAI semconv: `gen_ai.operation.name` — operation name (e.g., "chat", "complete").
    pub fn set_gen_ai_operation_name(&mut self, name: &str) {
        self.gen_ai_operation_name = Some(name.to_string());
    }

    /// GenAI semconv: `gen_ai.provider.name` — provider name (e.g., "openai", "anthropic", "ollama").
    pub fn set_gen_ai_provider_name(&mut self, name: &str) {
        self.gen_ai_provider_name = Some(name.to_string());
    }

    /// GenAI semconv: `gen_ai.request.stream` — whether the request is streamed.
    pub fn set_gen_ai_request_stream(&mut self, stream: bool) {
        self.gen_ai_request_stream = stream;
    }

    /// GenAI semconv: `gen_ai.response.id` — response ID.
    pub fn set_gen_ai_response_id(&mut self, id: &str) {
        self.gen_ai_response_id = Some(id.to_string());
    }

    /// GenAI semconv: `gen_ai.response.model` — response model.
    pub fn set_gen_ai_response_model(&mut self, model: &str) {
        self.gen_ai_response_model = Some(model.to_string());
    }

    /// GenAI semconv: `gen_ai.response.finish_reasons` — finish reasons.
    pub fn set_gen_ai_response_finish_reasons(&mut self, reasons: &[&str]) {
        self.gen_ai_response_finish_reasons = reasons.iter().map(|s| s.to_string()).collect();
    }

    /// GenAI semconv: `gen_ai.usage.input_tokens` — input tokens count.
    pub fn set_gen_ai_usage_input_tokens(&mut self, tokens: u64) {
        self.gen_ai_usage_input_tokens = Some(tokens);
    }

    /// GenAI semconv: `gen_ai.usage.output_tokens` — output tokens count.
    pub fn set_gen_ai_usage_output_tokens(&mut self, tokens: u64) {
        self.gen_ai_usage_output_tokens = Some(tokens);
    }
}

pub trait Tracer: Send + Sync {
    fn start_span(&self, name: &str, kind: SpanKind) -> Span;
    fn start_child_span(&self, parent: &Span, name: &str, kind: SpanKind) -> Span;
    fn end_span(&self, span: Span);
}

#[derive(Debug)]
pub struct ConsoleTracer;

impl Tracer for ConsoleTracer {
    fn start_span(&self, name: &str, kind: SpanKind) -> Span {
        Span::new(name, kind, None)
    }

    fn start_child_span(&self, parent: &Span, name: &str, kind: SpanKind) -> Span {
        Span::new(name, kind, Some(parent.id))
    }

    fn end_span(&self, span: Span) {
        let elapsed = span.start.elapsed();
        log::debug!(
            "[tracer] end span={} kind={:?} elapsed={:?}",
            span.name,
            span.kind,
            elapsed
        );
    }
}

pub struct NoopTracer;

impl Tracer for NoopTracer {
    fn start_span(&self, name: &str, kind: SpanKind) -> Span {
        Span::new(name, kind, None)
    }

    fn start_child_span(&self, parent: &Span, name: &str, kind: SpanKind) -> Span {
        Span::new(name, kind, Some(parent.id))
    }

    fn end_span(&self, _span: Span) {}
}

#[derive(Debug)]
pub struct CostTracker {
    pub total_prompt_tokens: AtomicU64,
    pub total_completion_tokens: AtomicU64,
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            total_prompt_tokens: AtomicU64::new(0),
            total_completion_tokens: AtomicU64::new(0),
        }
    }

    pub fn record(&mut self, _model: &str, prompt_tokens: u64, completion_tokens: u64) {
        self.total_prompt_tokens
            .fetch_add(prompt_tokens, Ordering::Relaxed);
        self.total_completion_tokens
            .fetch_add(completion_tokens, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_tracer() {
        let tracer = NoopTracer;
        let span = tracer.start_span("test", SpanKind::Internal);
        span.set_attribute("key", AttributeValue::String("val".into()));
        tracer.end_span(span);
    }

    #[test]
    fn test_console_tracer() {
        let tracer = ConsoleTracer;
        let span = tracer.start_span("test", SpanKind::Handoff);
        tracer.end_span(span);
    }

    #[test]
    fn test_child_span() {
        let tracer = ConsoleTracer;
        let parent = tracer.start_span("parent", SpanKind::Llm);
        let child = tracer.start_child_span(&parent, "child", SpanKind::Internal);
        tracer.end_span(child);
        tracer.end_span(parent);
    }

    #[test]
    fn test_cost_tracker() {
        let mut ct = CostTracker::new();
        ct.record("gpt-4", 100, 50);
        ct.record("gpt-4", 200, 100);
        assert_eq!(ct.total_prompt_tokens.load(Ordering::Relaxed), 300);
        assert_eq!(ct.total_completion_tokens.load(Ordering::Relaxed), 150);
    }

    #[test]
    fn test_gen_ai_setters_record_attributes() {
        let tracer = ConsoleTracer;
        let span = tracer.start_span("llm_call", SpanKind::Llm);
        span.set_gen_ai_request_model("gpt-4o");
        span.set_gen_ai_system("neotrix");
        assert_eq!(
            span.attribute("gen_ai.request.model"),
            Some(AttributeValue::String("gpt-4o".into()))
        );
        assert_eq!(
            span.attribute("gen_ai.system"),
            Some(AttributeValue::String("neotrix".into()))
        );
        assert_eq!(span.attribute("missing"), None);
        tracer.end_span(span);
    }
}
