//! Execution Trace Logger — 执行追踪日志
//! 结构化执行轨迹，支持 ROMA 风格的层级追踪

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub event_id: u64,
    pub parent_id: Option<u64>,
    pub event_type: String,
    pub component: String,
    pub description: String,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub metadata: std::collections::HashMap<String, String>,
    pub timestamp: u64,
}

pub struct ExecutionTrace {
    events: Vec<TraceEvent>,
    next_id: u64,
    current_span: Option<u64>,
}

impl ExecutionTrace {
    pub fn new() -> Self {
        Self { events: Vec::new(), next_id: 0, current_span: None }
    }

    pub fn start_span(&mut self, component: &str, description: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        
        self.events.push(TraceEvent {
            event_id: id,
            parent_id: self.current_span,
            event_type: "start".to_string(),
            component: component.to_string(),
            description: description.to_string(),
            duration_ms: None,
            success: true,
            metadata: std::collections::HashMap::new(),
            timestamp: Self::now(),
        });
        
        let old_span = self.current_span;
        self.current_span = Some(id);
        old_span.unwrap_or(id) // return parent for later end_span
    }

    pub fn end_span(&mut self, span_id: u64, success: bool) {
        let start_time = self.events.iter()
            .find(|e| e.event_id == span_id)
            .map(|e| e.timestamp)
            .unwrap_or(0);
        
        let duration = Self::now().saturating_sub(start_time);
        
        self.events.push(TraceEvent {
            event_id: self.next_id,
            parent_id: Some(span_id),
            event_type: "end".to_string(),
            component: String::new(),
            description: String::new(),
            duration_ms: Some(duration * 1000), // to ms
            success,
            metadata: std::collections::HashMap::new(),
            timestamp: Self::now(),
        });
        
        self.current_span = self.events.iter()
            .find(|e| e.event_id == span_id)
            .and_then(|e| e.parent_id);
    }

    pub fn add_event(&mut self, component: &str, description: &str, success: bool) {
        let id = self.next_id;
        self.next_id += 1;
        self.events.push(TraceEvent {
            event_id: id,
            parent_id: self.current_span,
            event_type: "event".to_string(),
            component: component.to_string(),
            description: description.to_string(),
            duration_ms: None,
            success,
            metadata: std::collections::HashMap::new(),
            timestamp: Self::now(),
        });
    }

    pub fn events(&self) -> &[TraceEvent] { &self.events }
    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }

    pub fn flatten(&self) -> Vec<String> {
        self.events.iter().map(|e| {
            format!("[{}] {}:{} {}", e.event_id, e.component, e.event_type, e.description)
        }).collect()
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    }
}

impl Default for ExecutionTrace {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_basic() {
        let mut trace = ExecutionTrace::new();
        let span = trace.start_span("search", "KB search");
        trace.add_event("search", "Found 5 results", true);
        trace.end_span(span, true);
        
        assert_eq!(trace.len(), 3); // start + event + end
        let flat = trace.flatten();
        assert!(flat[0].contains("search"));
    }
}
