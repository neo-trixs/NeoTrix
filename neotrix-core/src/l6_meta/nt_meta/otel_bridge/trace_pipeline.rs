#![deny(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::Span;

/// Span categories for structured trace routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpanKind {
    AgentDispatch,
    ToolCall,
    LlmRequest,
    MemoryOp,
}

impl SpanKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpanKind::AgentDispatch => "agent.dispatch",
            SpanKind::ToolCall => "tool.call",
            SpanKind::LlmRequest => "llm.request",
            SpanKind::MemoryOp => "memory.op",
        }
    }
}

/// A captured span record for offline analysis and cost correlation.
#[derive(Debug, Clone)]
pub struct CapturedSpan {
    pub id: u64,
    pub kind: SpanKind,
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub start_ms: u128,
    pub end_ms: Option<u128>,
    pub status: SpanStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

/// Shared span storage for correlation across agent boundaries.
#[derive(Debug, Clone)]
pub struct SpanStore {
    inner: Arc<Mutex<SpanStoreInner>>,
}

#[derive(Debug, Default)]
struct SpanStoreInner {
    spans: Vec<CapturedSpan>,
    next_id: u64,
}

impl SpanStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SpanStoreInner::default())),
        }
    }

    pub fn record(&self, span: CapturedSpan) {
        let mut inner = self.inner.lock().expect("span store lock poisoned");
        inner.spans.push(span);
    }

    pub fn snapshots(&self) -> Vec<CapturedSpan> {
        let inner = self.inner.lock().expect("span store lock poisoned");
        inner.spans.clone()
    }

    pub fn drain(&self) -> Vec<CapturedSpan> {
        let mut inner = self.inner.lock().expect("span store lock poisoned");
        std::mem::take(&mut inner.spans)
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().expect("span store lock poisoned");
        inner.spans.clear();
    }

    fn next_id(&self) -> u64 {
        let mut inner = self.inner.lock().expect("span store lock poisoned");
        let id = inner.next_id;
        inner.next_id += 1;
        id
    }
}

/// Trace pipeline that bridges NeoTrix agent operations to OTel spans.
///
/// Provides structured span creation for agent dispatch, tool calls,
/// LLM requests, and memory operations. Spans are recorded to both
/// the tracing subscriber (for OTel export) and the SpanStore (for
/// cost correlation and offline analysis).
pub struct TracePipeline {
    store: SpanStore,
}

impl TracePipeline {
    pub fn new() -> Self {
        Self {
            store: SpanStore::new(),
        }
    }

    pub fn with_store(store: SpanStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &SpanStore {
        &self.store
    }

    /// Create a span for agent dispatch with trace context propagation.
    pub fn agent_dispatch(&self, agent_id: &str, task: &str, parent: Option<&Span>) -> SpanGuard {
        let span = if let Some(parent) = parent {
            tracing::info_span!(
                parent: parent,
                "agent.dispatch",
                agent.id = %agent_id,
                task = %task,
                span.kind = "agent_dispatch",
            )
        } else {
            tracing::info_span!(
                "agent.dispatch",
                agent.id = %agent_id,
                task = %task,
                span.kind = "agent_dispatch",
            )
        };
        self.begin_capture(SpanKind::AgentDispatch, agent_id, span)
    }

    /// Create a span for tool invocation.
    pub fn tool_call(&self, tool_name: &str, agent_id: &str, parent: Option<&Span>) -> SpanGuard {
        let span = if let Some(parent) = parent {
            tracing::info_span!(
                parent: parent,
                "tool.call",
                tool.name = %tool_name,
                agent.id = %agent_id,
                span.kind = "tool_call",
            )
        } else {
            tracing::info_span!(
                "tool.call",
                tool.name = %tool_name,
                agent.id = %agent_id,
                span.kind = "tool_call",
            )
        };
        self.begin_capture(SpanKind::ToolCall, tool_name, span)
    }

    /// Create a span for an LLM API request.
    pub fn llm_request(&self, model: &str, agent_id: &str, parent: Option<&Span>) -> SpanGuard {
        let span = if let Some(parent) = parent {
            tracing::info_span!(
                parent: parent,
                "llm.request",
                model = %model,
                agent.id = %agent_id,
                span.kind = "llm_request",
            )
        } else {
            tracing::info_span!(
                "llm.request",
                model = %model,
                agent.id = %agent_id,
                span.kind = "llm_request",
            )
        };
        self.begin_capture(SpanKind::LlmRequest, model, span)
    }

    /// Create a span for a memory operation (KB read/write, embedding, etc.).
    pub fn memory_op(&self, operation: &str, namespace: &str, parent: Option<&Span>) -> SpanGuard {
        let span = if let Some(parent) = parent {
            tracing::info_span!(
                parent: parent,
                "memory.op",
                operation = %operation,
                namespace = %namespace,
                span.kind = "memory_op",
            )
        } else {
            tracing::info_span!(
                "memory.op",
                operation = %operation,
                namespace = %namespace,
                span.kind = "memory_op",
            )
        };
        self.begin_capture(SpanKind::MemoryOp, operation, span)
    }

    fn begin_capture(&self, kind: SpanKind, name: &str, span: Span) -> SpanGuard {
        let id = self.store.next_id();
        let now = Instant::now();

        SpanGuard {
            id,
            kind,
            name: name.to_string(),
            span,
            start: now,
            store: self.store.clone(),
            status: SpanStatus::Unset,
            attributes: HashMap::new(),
        }
    }
}

impl Default for TracePipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard that records span timing on drop.
pub struct SpanGuard {
    id: u64,
    kind: SpanKind,
    name: String,
    span: Span,
    start: Instant,
    store: SpanStore,
    status: SpanStatus,
    attributes: HashMap<String, String>,
}

impl SpanGuard {
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn mark_ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    pub fn mark_error(&mut self) {
        self.status = SpanStatus::Error;
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn kind(&self) -> SpanKind {
        self.kind
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_millis();
        let elapsed_u64 = elapsed as u64;

        tracing::info!(
            span.kind = self.kind.as_str(),
            span.name = %self.name,
            span.elapsed_ms = elapsed_u64,
            span.status = ?self.status,
            "span completed"
        );

        let captured = CapturedSpan {
            id: self.id,
            kind: self.kind,
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            start_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            end_ms: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
            ),
            status: self.status.clone(),
        };
        self.store.record(captured);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_store_lifecycle() {
        let store = SpanStore::new();
        let id = store.next_id();
        assert_eq!(id, 0);
        assert_eq!(store.next_id(), 1);
    }

    #[test]
    fn test_span_guard_records_on_drop() {
        let pipeline = TracePipeline::new();
        let store = pipeline.store().clone();
        {
            let mut guard = pipeline.agent_dispatch("a1", "test", None);
            guard.set_attribute("key", "val");
            guard.mark_ok();
        }
        let spans = store.snapshots();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].kind, SpanKind::AgentDispatch);
        assert_eq!(spans[0].status, SpanStatus::Ok);
        assert!(spans[0].end_ms.is_some());
    }

    #[test]
    fn test_tool_call_span() {
        let pipeline = TracePipeline::new();
        let guard = pipeline.tool_call("web_search", "a1", None);
        assert_eq!(guard.kind(), SpanKind::ToolCall);
        assert_eq!(guard.name, "web_search");
    }

    #[test]
    fn test_llm_request_span() {
        let pipeline = TracePipeline::new();
        let guard = pipeline.llm_request("gpt-4", "a1", None);
        assert_eq!(guard.kind(), SpanKind::LlmRequest);
    }

    #[test]
    fn test_memory_op_span() {
        let pipeline = TracePipeline::new();
        let guard = pipeline.memory_op("read", "experience", None);
        assert_eq!(guard.kind(), SpanKind::MemoryOp);
    }

    #[test]
    fn test_drain_clears_spans() {
        let pipeline = TracePipeline::new();
        {
            let _guard = pipeline.agent_dispatch("a1", "t", None);
        }
        let drained = pipeline.store().drain();
        assert_eq!(drained.len(), 1);
        assert!(pipeline.store().snapshots().is_empty());
    }
}
