use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// Response provenance tracker.
///
/// Embeds detectable markers in responses for closed-loop tracing.
/// Stores watermarked response fingerprints in an internal registry.
/// Enables detection of distillation by checking if watermarked responses
/// appear in downstream KB or training pipelines.
#[derive(Debug, Clone)]
pub struct ResponseTracer {
    pub enabled: bool,
    pub tracing_id: u64,
    pub registry: HashMap<u64, TraceRecord>,
    max_records: usize,
}

/// A tracing record for a watermarked response.
#[derive(Debug, Clone)]
pub struct TraceRecord {
    pub tracing_id: u64,
    pub timestamp: u64,
    pub response_hash: u64,
    pub watermark_bits: u8,
    pub model: String,
    pub prompt_prefix: String,
}

impl Default for ResponseTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseTracer {
    pub fn new() -> Self {
        Self {
            enabled: true,
            tracing_id: 0,
            registry: HashMap::new(),
            max_records: 10000,
        }
    }

    /// Embed a watermark marker (session signature) into response text.
    /// Uses a deterministic tag based on tracing_id + timestamp.
    pub fn watermark_response(&self, response: &str) -> String {
        if !self.enabled {
            return response.to_string();
        }
        let tag = self.watermark_tag();
        format!("{}\n\n<!-- nt:tid={} -->", response, tag)
    }

    /// Detect watermark in a response. Returns similarity [0, 1].
    pub fn detect_response_watermark(&self, response: &str) -> f64 {
        let expected = self.watermark_tag();
        if let Some(rest) = response.split("<!-- nt:tid=").nth(1) {
            if let Some(found) = rest.split("-->").next() {
                let found = found.trim();
                if found == expected {
                    return 1.0;
                }
                // Simple character-wise similarity
                let max_len = expected.len().max(found.len());
                if max_len == 0 {
                    return 0.0;
                }
                let matches = expected.chars().zip(found.chars())
                    .filter(|(a, b)| a == b)
                    .count();
                return (matches as f64 / max_len as f64).max(0.0);
            }
        }
        0.0
    }

    /// Register a trace record for a response.
    pub fn register_trace(&mut self, response: &str, bits: u8, model: &str, prompt_prefix: &str) -> TraceRecord {
        self.tracing_id += 1;
        let hash = hash_response(response);
        let record = TraceRecord {
            tracing_id: self.tracing_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_hash: hash,
            watermark_bits: bits,
            model: model.to_string(),
            prompt_prefix: prompt_prefix.chars().take(64).collect(),
        };
        if self.registry.len() >= self.max_records {
            if let Some(oldest) = self.registry.keys().copied().min() {
                self.registry.remove(&oldest);
            }
        }
        self.registry.insert(hash, record.clone());
        record
    }

    /// Check if a response matches any registered trace by hash.
    pub fn lookup_trace(&self, response: &str) -> Option<&TraceRecord> {
        let hash = hash_response(response);
        self.registry.get(&hash)
    }

    /// Generate a deterministic watermark tag for this session.
    fn watermark_tag(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("nt-{}-{:x}", self.tracing_id, now % 0xFFFF)
    }

    /// Statistics about tracing.
    pub fn stats(&self) -> TracerStats {
        TracerStats {
            total_traces: self.registry.len() as u64,
            current_id: self.tracing_id,
            enabled: self.enabled,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TracerStats {
    pub total_traces: u64,
    pub current_id: u64,
    pub enabled: bool,
}

fn hash_response(response: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    response.hash(&mut hasher);
    hasher.finish()
}

/// Check if a KB collection contains watermarked responses (distillation detection).
pub fn detect_watermarked_in_corpus(corpus: &[String], tracer: &ResponseTracer, threshold: f64) -> Vec<(usize, f64)> {
    let mut detections = Vec::new();
    for (i, text) in corpus.iter().enumerate() {
        let sim = tracer.detect_response_watermark(text);
        if sim >= threshold {
            detections.push((i, sim));
        }
    }
    detections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_default_state() {
        let tracer = ResponseTracer::new();
        assert!(tracer.enabled);
        assert_eq!(tracer.tracing_id, 0);
        assert!(tracer.registry.is_empty());
    }

    #[test]
    fn test_response_watermarking() {
        let tracer = ResponseTracer::new();
        let response = "This is a test response.";
        let watermarked = tracer.watermark_response(response);
        assert!(watermarked.starts_with(response));
        assert!(watermarked.len() > response.len());
    }

    #[test]
    fn test_watermark_detection() {
        let tracer = ResponseTracer::new();
        let response = "Hello, world!";
        let watermarked = tracer.watermark_response(response);
        let sim = tracer.detect_response_watermark(&watermarked);
        assert!(sim > 0.0, "similarity should be > 0, got {}", sim);
    }

    #[test]
    fn test_no_false_positive_on_clean() {
        let tracer = ResponseTracer::new();
        let clean = "This is a clean text with no watermark.";
        let sim = tracer.detect_response_watermark(clean);
        assert!(sim < 0.1, "clean text should have near-zero similarity, got {}", sim);
    }

    #[test]
    fn test_register_trace() {
        let mut tracer = ResponseTracer::new();
        let record = tracer.register_trace("test response", 0b001011, "neotrix-v1", "You are NeoTrix");
        assert_eq!(record.watermark_bits, 0b001011);
        assert_eq!(record.model, "neotrix-v1");
        assert_eq!(tracer.tracing_id, 1);
    }

    #[test]
    fn test_lookup_trace_exact() {
        let mut tracer = ResponseTracer::new();
        tracer.register_trace("original response content", 0b101, "model-x", "system prompt");
        let found = tracer.lookup_trace("original response content");
        assert!(found.is_some());
        assert_eq!(found.unwrap().watermark_bits, 0b101);
    }

    #[test]
    fn test_lookup_trace_no_match() {
        let mut tracer = ResponseTracer::new();
        tracer.register_trace("response one", 0b001, "m1", "p1");
        let found = tracer.lookup_trace("completely different text");
        assert!(found.is_none());
    }

    #[test]
    fn test_corpus_detection() {
        let tracer = ResponseTracer::new();
        let clean = "clean text without watermark".to_string();
        let wm_response = {
            let r = tracer.watermark_response("watermarked response");
            r
        };
        let corpus = vec![clean, wm_response];
        let detections = detect_watermarked_in_corpus(&corpus, &tracer, 0.9);
        assert!(!detections.is_empty(), "should detect at least one watermark");
    }

    #[test]
    fn test_hash_consistency() {
        let h1 = hash_response("same text");
        let h2 = hash_response("same text");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different() {
        let h1 = hash_response("text a");
        let h2 = hash_response("text b");
        assert_ne!(h1, h2);
    }
}
