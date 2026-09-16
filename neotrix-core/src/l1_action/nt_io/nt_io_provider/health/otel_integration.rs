use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Span {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
    pub status: SpanStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

pub struct OtelCollector {
    spans: Vec<Span>,
    metrics: HashMap<String, f64>,
}

impl OtelCollector {
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            metrics: HashMap::new(),
        }
    }

    pub fn start_span(&mut self, name: &str) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.spans.push(Span {
            name: name.to_string(),
            attributes: HashMap::new(),
            start_ms: now,
            end_ms: None,
            status: SpanStatus::Unset,
        });
        (self.spans.len() - 1) as u64
    }

    pub fn end_span(&mut self, idx: usize, status: SpanStatus) {
        if let Some(s) = self.spans.get_mut(idx) {
            s.end_ms = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            );
            s.status = status;
        }
    }

    pub fn set_attribute(&mut self, idx: usize, key: &str, value: &str) {
        if let Some(s) = self.spans.get_mut(idx) {
            s.attributes.insert(key.to_string(), value.to_string());
        }
    }

    pub fn record_metric(&mut self, name: &str, value: f64) {
        *self.metrics.entry(name.to_string()).or_insert(0.0) += value;
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    pub fn metrics(&self) -> &HashMap<String, f64> {
        &self.metrics
    }

    pub fn span_count(&self) -> usize {
        self.spans.len()
    }
}

impl Default for OtelCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_lifecycle() {
        let mut c = OtelCollector::new();
        let idx = c.start_span("op");
        c.set_attribute(idx as usize, "key", "val");
        c.end_span(idx as usize, SpanStatus::Ok);
        assert_eq!(c.spans()[0].status, SpanStatus::Ok);
    }

    #[test]
    fn test_metric() {
        let mut c = OtelCollector::new();
        c.record_metric("latency", 100.0);
        c.record_metric("latency", 200.0);
        assert_eq!(c.metrics()["latency"], 300.0);
    }
}
