use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Every LLM interaction is captured here — model-agnostic.
/// No model-specific fields: pure observational record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedInteraction {
    /// Wall-clock timestamp (epoch ms)
    pub timestamp_ms: u64,
    /// Which provider handled this call (e.g. "groq", "openai", "deepseek")
    pub provider: String,
    /// The model string (e.g. "llama-3-70b", "gpt-4", "claude-3")
    pub model: String,
    /// System prompt used (truncated to 1024 chars for storage)
    pub system_prompt: String,
    /// User message(s) concatenated with separator
    pub user_messages: String,
    /// Full response text
    pub response: String,
    /// Prompt token count
    pub prompt_tokens: u32,
    /// Completion token count
    pub completion_tokens: u32,
    /// Total latency in milliseconds
    pub latency_ms: u64,
    /// Whether the call succeeded
    pub success: bool,
    /// Finish reason
    pub finish_reason: String,
    /// Rich structure fingerprint for pattern matching
    pub structure_hash: u64,
    /// Presence of code blocks in response
    pub has_code: bool,
    /// Code block count
    pub code_block_count: usize,
    /// Has structured headings
    pub has_sections: bool,
    /// Has bullet/numbered lists
    pub has_lists: bool,
    /// Response word count
    pub word_count: usize,
    /// Outcome score estimated via heuristics (0.0–1.0)
    pub outcome_score: f64,
}

impl CapturedInteraction {
    /// Create from raw fields with automatic structure analysis.
    pub fn new(
        provider: &str,
        model: &str,
        system_prompt: &str,
        user_messages: &str,
        response: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        latency_ms: u64,
        success: bool,
        finish_reason: &str,
    ) -> Self {
        let code_block_count = response.matches("```").count() / 2;
        let has_code = code_block_count > 0;
        let has_sections = response.contains("##") || response.contains("==");
        let has_lists =
            response.contains("- ") || response.contains("* ") || response.contains("1. ");
        let word_count = response.split_whitespace().count();
        let structure_hash = Self::hash_structure(response);

        let outcome_score = Self::estimate_outcome(
            success,
            finish_reason,
            has_code,
            word_count,
            completion_tokens,
        );

        // Truncate long fields for storage efficiency
        let sp = if system_prompt.len() > 1024 {
            &system_prompt[..1024]
        } else {
            system_prompt
        };
        let um = if user_messages.len() > 2048 {
            &user_messages[..2048]
        } else {
            user_messages
        };
        let resp = if response.len() > 4096 {
            &response[..4096]
        } else {
            response
        };

        Self {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            provider: provider.to_string(),
            model: model.to_string(),
            system_prompt: sp.to_string(),
            user_messages: um.to_string(),
            response: resp.to_string(),
            prompt_tokens,
            completion_tokens,
            latency_ms,
            success,
            finish_reason: finish_reason.to_string(),
            structure_hash,
            has_code,
            code_block_count,
            has_sections,
            has_lists,
            word_count,
            outcome_score,
        }
    }

    /// Hash the structural features of a response for similarity matching.
    fn hash_structure(response: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // Structural features: code blocks, heading patterns, list patterns, length bucket
        let code_present = response.contains("```");
        let heading_present = response.contains("##") || response.contains("==");
        let list_present = response.contains("- ") || response.contains("* ");
        let len_bucket = (response.len() / 256) as u64;
        let newline_count = response.matches('\n').count() as u64;

        code_present.hash(&mut hasher);
        heading_present.hash(&mut hasher);
        list_present.hash(&mut hasher);
        len_bucket.hash(&mut hasher);
        newline_count.hash(&mut hasher);
        hasher.finish()
    }

    /// Heuristic outcome score: combines success, code presence, verbosity, and length.
    fn estimate_outcome(
        success: bool,
        finish_reason: &str,
        has_code: bool,
        word_count: usize,
        completion_tokens: u32,
    ) -> f64 {
        if !success {
            return 0.0;
        }
        let mut score: f64 = 0.5;
        // Complete responses are better
        if finish_reason == "stop" {
            score += 0.2;
        }
        // Code-rich responses tend to be more useful
        if has_code {
            score += 0.15;
        }
        // Substantial responses
        if word_count > 10 && word_count < 5000 {
            score += 0.1;
        }
        // Token efficiency: not too short, not excessive
        if completion_tokens > 20 && completion_tokens < 4096 {
            score += 0.05;
        }
        score.min(1.0)
    }

    /// Extract the domain/topic hint from user messages.
    /// More specific keywords first to avoid false matches.
    pub fn topic_hint(&self) -> &str {
        let lower = self.user_messages.to_lowercase();
        let topics = [
            ("rust", "rust"),
            ("python", "python"),
            ("javascript", "javascript"),
            ("security", "security"),
            ("performance", "performance"),
            ("architect", "architecture"),
            ("algorithm", "algorithm"),
            ("refactor", "refactoring"),
            ("deploy", "deployment"),
            ("debug", "debugging"),
            ("design", "design"),
            ("test", "testing"),
            ("code", "code"),
            ("api", "api"),
            ("data", "data"),
        ];
        for (keyword, topic) in &topics {
            if lower.contains(keyword) {
                return topic;
            }
        }
        "general"
    }
}

/// Ring buffer of captured interactions for sliding-window distillation.
#[derive(Debug, Clone)]
pub struct CaptureBuffer {
    interactions: Vec<CapturedInteraction>,
    max_size: usize,
    capture_count: u64,
}

impl CaptureBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            interactions: Vec::with_capacity(max_size),
            max_size,
            capture_count: 0,
        }
    }

    /// Add a captured interaction, evicting oldest if at capacity.
    pub fn push(&mut self, interaction: CapturedInteraction) {
        if self.interactions.len() >= self.max_size {
            self.interactions.remove(0);
        }
        self.capture_count += 1;
        self.interactions.push(interaction);
    }

    pub fn interactions(&self) -> &[CapturedInteraction] {
        &self.interactions
    }

    pub fn capture_count(&self) -> u64 {
        self.capture_count
    }

    pub fn len(&self) -> usize {
        self.interactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.interactions.is_empty()
    }

    /// Set the max_size (used by modulation commands)
    pub fn set_max_size(&mut self, new_max: usize) {
        self.max_size = new_max;
    }

    /// Drain all interactions (for batch distillation).
    pub fn drain(&mut self) -> Vec<CapturedInteraction> {
        std::mem::take(&mut self.interactions)
    }

    /// Summarize by provider/model.
    pub fn summary(&self) -> String {
        use std::collections::HashMap;
        let mut by_provider: HashMap<String, usize> = HashMap::new();
        let mut by_model: HashMap<String, usize> = HashMap::new();
        for i in &self.interactions {
            *by_provider.entry(i.provider.clone()).or_insert(0) += 1;
            *by_model.entry(i.model.clone()).or_insert(0) += 1;
        }
        let providers: Vec<String> = by_provider
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();
        let models: Vec<String> = by_model
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();
        format!(
            "CaptureBuffer[total={} buffer={} providers=[{}] models=[{}]]",
            self.capture_count,
            self.interactions.len(),
            providers.join(","),
            models.join(",")
        )
    }

    /// Group interactions by model for per-model distillation.
    /// Returns owned data (cloned) to avoid lifetime issues across mutex boundaries.
    pub fn by_model(&self) -> std::collections::HashMap<String, Vec<CapturedInteraction>> {
        let mut map: std::collections::HashMap<String, Vec<CapturedInteraction>> =
            std::collections::HashMap::new();
        for i in &self.interactions {
            map.entry(i.model.clone()).or_default().push(i.clone());
        }
        map
    }

    /// Group interactions by provider for per-provider stats.
    pub fn by_provider(&self) -> std::collections::HashMap<String, Vec<&CapturedInteraction>> {
        let mut map: std::collections::HashMap<String, Vec<&CapturedInteraction>> =
            std::collections::HashMap::new();
        for i in &self.interactions {
            map.entry(i.provider.clone()).or_default().push(i);
        }
        map
    }
}

impl Default for CaptureBuffer {
    fn default() -> Self {
        Self::new(500)
    }
}

/// DistillationProbe is a transparent wrapper around any LlmProvider.
/// It captures every call without modifying behavior.
pub struct DistillationProbe<P> {
    inner: Arc<P>,
    buffer: Arc<std::sync::Mutex<CaptureBuffer>>,
}

impl<P> DistillationProbe<P> {
    pub fn new(inner: Arc<P>, buffer: Arc<std::sync::Mutex<CaptureBuffer>>) -> Self {
        Self { inner, buffer }
    }

    /// Record a completed interaction into the shared buffer.
    pub fn record(
        &self,
        provider: &str,
        model: &str,
        system_prompt: &str,
        user_messages: &str,
        response: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        latency_ms: u64,
        success: bool,
        finish_reason: &str,
    ) {
        let interaction = CapturedInteraction::new(
            provider,
            model,
            system_prompt,
            user_messages,
            response,
            prompt_tokens,
            completion_tokens,
            latency_ms,
            success,
            finish_reason,
        );
        if let Ok(mut buf) = self.buffer.lock() {
            buf.push(interaction);
        }
    }

    pub fn inner(&self) -> &Arc<P> {
        &self.inner
    }

    pub fn buffer(&self) -> &Arc<std::sync::Mutex<CaptureBuffer>> {
        &self.buffer
    }
}

impl<P: Clone> Clone for DistillationProbe<P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            buffer: self.buffer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_new() {
        let cap = CapturedInteraction::new(
            "test-provider",
            "test-model",
            "be helpful",
            "write code",
            "```rust\nfn hello() {}\n```",
            50,
            100,
            200,
            true,
            "stop",
        );
        assert_eq!(cap.provider, "test-provider");
        assert_eq!(cap.model, "test-model");
        assert!(cap.has_code);
        assert_eq!(cap.code_block_count, 1);
        assert!(cap.outcome_score > 0.0);
    }

    #[test]
    fn test_capture_failure_score() {
        let cap = CapturedInteraction::new("p", "m", "", "hello", "", 0, 0, 0, false, "error");
        assert_eq!(cap.outcome_score, 0.0);
    }

    #[test]
    fn test_buffer_push_and_drain() {
        let mut buf = CaptureBuffer::new(10);
        assert!(buf.is_empty());
        for i in 0..5 {
            buf.push(CapturedInteraction::new(
                "p",
                &format!("m{}", i),
                "",
                &format!("msg{}", i),
                "ok",
                10,
                20,
                50,
                true,
                "stop",
            ));
        }
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.capture_count(), 5);
        let drained = buf.drain();
        assert_eq!(drained.len(), 5);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffer_eviction() {
        let mut buf = CaptureBuffer::new(3);
        for i in 0..5 {
            buf.push(CapturedInteraction::new(
                "p",
                "m",
                "",
                &format!("msg{}", i),
                "ok",
                10,
                20,
                50,
                true,
                "stop",
            ));
        }
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.capture_count(), 5);
    }

    #[test]
    fn test_topic_hint() {
        let cap = CapturedInteraction::new(
            "p",
            "m",
            "",
            "how do I debug this rust code",
            "use println!",
            10,
            20,
            50,
            true,
            "stop",
        );
        assert_eq!(cap.topic_hint(), "rust");
        let cap2 =
            CapturedInteraction::new("p", "m", "", "hello world", "hi", 10, 5, 10, true, "stop");
        assert_eq!(cap2.topic_hint(), "general");
    }

    #[test]
    fn test_summary_format() {
        let mut buf = CaptureBuffer::new(100);
        buf.push(CapturedInteraction::new(
            "p1", "m1", "", "hello", "world", 10, 20, 50, true, "stop",
        ));
        buf.push(CapturedInteraction::new(
            "p2", "m2", "", "test", "ok", 5, 10, 30, true, "stop",
        ));
        let s = buf.summary();
        assert!(s.contains("CaptureBuffer["));
        assert!(s.contains("total="));
    }

    #[test]
    fn test_by_model_groups() {
        let mut buf = CaptureBuffer::new(100);
        buf.push(CapturedInteraction::new(
            "p", "model-a", "", "q1", "a1", 10, 20, 50, true, "stop",
        ));
        buf.push(CapturedInteraction::new(
            "p", "model-b", "", "q2", "a2", 10, 20, 50, true, "stop",
        ));
        buf.push(CapturedInteraction::new(
            "p", "model-a", "", "q3", "a3", 10, 20, 50, true, "stop",
        ));
        let by_m = buf.by_model();
        assert_eq!(by_m.get("model-a").unwrap().len(), 2);
        assert_eq!(by_m.get("model-b").unwrap().len(), 1);
    }

    #[test]
    fn test_by_provider_groups() {
        let mut buf = CaptureBuffer::new(100);
        buf.push(CapturedInteraction::new(
            "prov-x", "m", "", "q1", "a1", 10, 20, 50, true, "stop",
        ));
        buf.push(CapturedInteraction::new(
            "prov-y", "m", "", "q2", "a2", 10, 20, 50, true, "stop",
        ));
        let by_p = buf.by_provider();
        assert_eq!(by_p.get("prov-x").unwrap().len(), 1);
        assert_eq!(by_p.get("prov-y").unwrap().len(), 1);
    }

    #[test]
    fn test_probe_creation() {
        let buf = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(100)));
        // In real usage, P would be an LlmProvider. Here we just test construction.
        struct DummyProvider;
        let provider = Arc::new(DummyProvider);
        let probe = DistillationProbe::new(provider, buf.clone());
        probe.record(
            "test",
            "test-model",
            "",
            "hello",
            "response",
            10,
            20,
            30,
            true,
            "stop",
        );
        let b = buf.lock().unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b.capture_count(), 1);
    }

    #[test]
    fn test_estimate_outcome_edge_cases() {
        let cap = CapturedInteraction::new("p", "m", "", "", "", 0, 0, 0, true, "stop");
        assert!(cap.outcome_score > 0.0);
        let cap2 = CapturedInteraction::new("p", "m", "", "", "short", 0, 5, 0, true, "stop");
        assert!(cap2.outcome_score > 0.5);
    }
}
