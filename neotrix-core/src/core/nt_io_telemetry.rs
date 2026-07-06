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

#[derive(Debug, Clone)]
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
        }
    }

    pub fn set_attribute(&self, key: &str, value: AttributeValue) {
        if let Ok(mut attrs) = self.attributes.lock() {
            attrs.push((key.to_string(), value));
        }
    }

    pub fn set_gen_ai_request_model(&self, _model: &str) {}

    pub fn set_gen_ai_system(&self, _system: &str) {}
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
            span.name, span.kind, elapsed
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
        assert_eq!(
            ct.total_completion_tokens.load(Ordering::Relaxed),
            150
        );
    }
}
