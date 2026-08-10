//! # NeoTrix Anti-Distillation System (L7 — Capability Layer)
//!
//! Multi-layered defense against unauthorized model distillation:
//!
//! 1. **Watermark Engine** — Embeds steganographic Unicode markers in system
//!    prompts (Anthropic-inspired: apostrophe variants, date separators,
//!    space padding). Encodes 6 bits of provenance data (timezone, domain
//!    match, keyword match, routing class).
//!
//! 2. **Response Tracer** — VSA/FHRR-based response watermarking for
//!    closed-loop provenance. Enables detection of watermarked responses
//!    in downstream KB/corpus (distillation detection).
//!
//! 3. **Distillation Detector** — Real-time request pattern analysis.
//!    Detects high-rate extraction, template-based collection,
//!    systematic sampling, and known distillation sources.
//!
//! 4. **Task Decomposer** — Input-side: analyzes task prompts for refusal
//!    triggers and suggests safe subtask decomposition. Uses the
//!    `decomplex_aggression` parameter to control how aggressively
//!    tasks are split.
//!
//! ## 层间规则
//! - L7 → L1 (IO): Watermarked prompts → LlmRequest
//! - L7 → L3 (KB): Trace records stored for later detection
//! - L7 → L8 (SEAL): AntiDistillationStage in pipeline (defined in L8)

mod watermark;
mod tracer;
mod detector;
pub mod decompose;

pub use watermark::{
    WatermarkEngine, WatermarkBits, ApostropheVariant, WatermarkConfig,
};
pub use tracer::{
    ResponseTracer, TraceRecord, TracerStats,
    detect_watermarked_in_corpus,
};
pub use detector::{
    DistillationDetector, DistillationAlert, AlertType, DetectorStats,
    ResponseAnalysis, analyze_response_pattern,
};

pub use decompose::{TaskDecomposer, DecomposeSuggestion};

/// Anti-distillation storage interface — injected by L8 runtime.
/// Decouples AntiDistillationSystem from the concrete KnowledgeBase type
/// to preserve L7→L3 layer isolation (core must not import from neotrix).
pub trait AntiDistilStore {
    fn store_trace_data(&self, data: &serde_json::Value) -> Result<(), String>;
    fn get_trace_data(&self, limit: usize) -> Result<Vec<serde_json::Value>, String>;
}

/// Unified orchestrator for all anti-distillation subsystems.
#[derive(Debug, Clone)]
pub struct AntiDistillationSystem {
    pub watermark: WatermarkEngine,
    pub tracer: ResponseTracer,
    pub detector: DistillationDetector,
    pub enabled: bool,
    /// Input-side: refusal tracking — how often does the LLM refuse to respond.
    pub total_calls: u64,
    pub refused_calls: u64,
    /// Current watermark strength multiplier (1.0 = normal).
    pub watermark_strength: f64,
    /// Dynamic threshold adjustment (0.0-1.0) for task decomposition aggressiveness.
    pub decomplex_aggression: f64,
}

impl Default for AntiDistillationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AntiDistillationSystem {
    pub fn new() -> Self {
        Self {
            watermark: WatermarkEngine::new(),
            tracer: ResponseTracer::new(),
            detector: DistillationDetector::new(),
            enabled: true,
            total_calls: 0,
            refused_calls: 0,
            watermark_strength: 1.0,
            decomplex_aggression: 0.3,
        }
    }

    pub fn with_probe(mut self, proxy_host: Option<&str>, timezone: Option<&str>) -> Self {
        self.watermark = self.watermark.with_probe(proxy_host, timezone);
        self
    }

    pub fn with_detector_threshold(mut self, rpm: u64, similarity: f64) -> Self {
        self.detector = self.detector.with_threshold(rpm, similarity);
        self
    }

    pub fn with_decomplex_aggression(mut self, aggression: f64) -> Self {
        self.decomplex_aggression = aggression.max(0.0).min(1.0);
        self
    }

    /// Encode date line with watermark.
    pub fn encode_date_line(&self, date_str: &str) -> String {
        self.watermark.encode_date_line(date_str)
    }

    /// Watermark a response for provenance tracking.
    pub fn watermark_response(&self, response: &str) -> String {
        self.tracer.watermark_response(response)
    }

    /// Register a trace record.
    pub fn register_trace(&mut self, response: &str, bits: u8, model: &str, prompt_prefix: &str) {
        self.tracer.register_trace(response, bits, model, prompt_prefix);
    }

    /// Record a request in the detector.
    pub fn record_request(
        &mut self,
        source: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
        response_length: usize,
    ) -> Option<DistillationAlert> {
        if !self.enabled {
            return None;
        }
        self.detector.record_request(source, prompt, temperature, max_tokens, response_length)
    }

    pub fn detect_watermarked_in_corpus(&self, corpus: &[String], threshold: f64) -> Vec<(usize, f64)> {
        detect_watermarked_in_corpus(corpus, &self.tracer, threshold)
    }

    /// Record an LLM call outcome (input-side tracking).
    pub fn record_llm_call(&mut self, refused: bool) {
        self.total_calls += 1;
        if refused {
            self.refused_calls += 1;
        }
    }

    /// Refusal rate over total calls (0.0-1.0).
    pub fn refusal_rate(&self) -> f64 {
        if self.total_calls == 0 { 0.0 }
        else { self.refused_calls as f64 / self.total_calls as f64 }
    }

    /// Check if a task should be decomposed to avoid LLM refusal.
    /// Returns suggestions for subtask splitting when refusal risk is high.
    pub fn decompose_task(&self, task: &str) -> Option<Vec<DecomposeSuggestion>> {
        if !self.enabled {
            return None;
        }
        TaskDecomposer::analyze(task, self.decomplex_aggression)
    }

    /// Quick check: can this task be decomposed further?
    pub fn can_decompose(&self, task: &str) -> bool {
        self.decompose_task(task).is_some_and(|s| !s.is_empty())
    }

    /// Rotate the watermark encoding scheme — cycles apostrophe offset, routing class,
    /// modifier colon, and date separator to prevent long-term pattern analysis.
    pub fn rotate_scheme(&mut self) {
        self.watermark.scheme_offset = (self.watermark.scheme_offset + 1) % 4;
        self.watermark.routing_class = (self.watermark.routing_class + 1) % 8;
        self.watermark.reserve_flag = !self.watermark.reserve_flag;
        self.watermark.cn_timezone = !self.watermark.cn_timezone;
    }

    /// Tune watermark strength based on distillation threat level.
    pub fn adjust_from_alerts(&mut self, alerts: &[DistillationAlert]) {
        let alert_count = alerts.len() as f64;
        if alert_count > 5.0 {
            self.watermark_strength = (self.watermark_strength + 0.5).min(3.0);
        } else if alert_count > 2.0 {
            self.watermark_strength = (self.watermark_strength + 0.2).min(3.0);
        }
        // Adjust decomplex aggression if refusal rate is high
        let rr = self.refusal_rate();
        if rr > 0.3 {
            self.decomplex_aggression = (self.decomplex_aggression + 0.1).min(1.0);
        } else if rr < 0.05 {
            self.decomplex_aggression = (self.decomplex_aggression - 0.05).max(0.0);
        }
    }

    /// Serialize current state to JSON for KB persistence.
    pub fn serialize_to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "registry_size": self.tracer.registry.len(),
            "alert_count": self.detector.alert_history.len(),
            "total_calls": self.total_calls,
            "refused_calls": self.refused_calls,
            "watermark_strength": self.watermark_strength,
            "decomplex_aggression": self.decomplex_aggression,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        })
    }

    /// Persist current state to KB.
    pub fn persist_to_kb(&self, kb: &dyn AntiDistilStore) {
        let data = self.serialize_to_json();
        let _ = kb.store_trace_data(&data);
    }

    /// Load most recent persisted state from KB into this system.
    pub fn load_from_kb(&mut self, kb: &dyn AntiDistilStore) {
        let records: Vec<serde_json::Value> = kb.get_trace_data(1).unwrap_or_default();
        if let Some(rec) = records.first() {
            self.total_calls = rec.get("total_calls").and_then(|v| v.as_u64()).unwrap_or(0);
            self.refused_calls = rec.get("refused_calls").and_then(|v| v.as_u64()).unwrap_or(0);
            self.watermark_strength = rec.get("watermark_strength").and_then(|v| v.as_f64()).unwrap_or(1.0);
            self.decomplex_aggression = rec.get("decomplex_aggression").and_then(|v| v.as_f64()).unwrap_or(0.3);
        }
    }

    pub fn stats(&self) -> AntiDistilStats {
        AntiDistilStats {
            watermark_enabled: self.watermark.enabled,
            tracer_enabled: self.tracer.enabled,
            detector_enabled: self.detector.enabled,
            total_traces: self.tracer.registry.len() as u64,
            total_alerts: self.detector.alert_history.len() as u64,
            total_calls: self.total_calls,
            refused_calls: self.refused_calls,
            refusal_rate: self.refusal_rate(),
            watermark_strength: self.watermark_strength,
            decomplex_aggression: self.decomplex_aggression,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AntiDistilStats {
    pub watermark_enabled: bool,
    pub tracer_enabled: bool,
    pub detector_enabled: bool,
    pub total_traces: u64,
    pub total_alerts: u64,
    pub total_calls: u64,
    pub refused_calls: u64,
    pub refusal_rate: f64,
    pub watermark_strength: f64,
    pub decomplex_aggression: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_antidistil_system_new() {
        let ads = AntiDistillationSystem::new();
        assert!(ads.enabled);
        assert!(ads.watermark.enabled);
        assert!(ads.tracer.enabled);
        assert!(ads.detector.enabled);
    }

    #[test]
    fn test_antidistil_with_probe() {
        let ads = AntiDistillationSystem::new()
            .with_probe(Some("deepseek.example.com"), Some("Asia/Shanghai"));
        assert!(ads.watermark.lab_keyword_match);
        assert!(ads.watermark.cn_timezone);
    }

    #[test]
    fn test_antidistil_encode_decode() {
        let ads = AntiDistillationSystem::new()
            .with_probe(Some("minimax.cn"), Some("Asia/Shanghai"));
        let date = "Today's date is 2026-07-02.";
        let encoded = ads.encode_date_line(date);
        assert_ne!(encoded, date);
        let decoded = WatermarkEngine::decode_date_line(&encoded);
        assert!(decoded.has_slash);
        assert_eq!(decoded.apostrophe, ApostropheVariant::ModifierPrime);
    }

    #[test]
    fn test_antidistil_trace_and_detect() {
        let mut ads = AntiDistillationSystem::new();
        let response = "This is a test response for distillation detection.";
        let watermarked = ads.watermark_response(response);
        assert!(watermarked.starts_with(response));

        ads.register_trace(&watermarked, 0b101, "test-model", "system prompt");

        let corpus = vec!["clean text".to_string(), watermarked.clone()];
        let detections = ads.detect_watermarked_in_corpus(&corpus, 0.01);
        assert!(!detections.is_empty());
    }

    #[test]
    fn test_antidistil_stats() {
        let ads = AntiDistillationSystem::new();
        let stats = ads.stats();
        assert!(stats.watermark_enabled);
        assert_eq!(stats.total_traces, 0);
        assert_eq!(stats.total_alerts, 0);
        assert_eq!(stats.total_calls, 0);
        assert_eq!(stats.refused_calls, 0);
        assert_eq!(stats.refusal_rate, 0.0);
        assert_eq!(stats.watermark_strength, 1.0);
        assert_eq!(stats.decomplex_aggression, 0.3);
    }

    #[test]
    fn test_antidistil_record_llm_call() {
        let mut ads = AntiDistillationSystem::new();
        ads.record_llm_call(false);
        assert_eq!(ads.total_calls, 1);
        assert_eq!(ads.refused_calls, 0);
        assert_eq!(ads.refusal_rate(), 0.0);
        ads.record_llm_call(true);
        assert_eq!(ads.total_calls, 2);
        assert_eq!(ads.refused_calls, 1);
        assert_eq!(ads.refusal_rate(), 0.5);
    }

    #[test]
    fn test_antidistil_adjust_from_alerts() {
        let mut ads = AntiDistillationSystem::new();
        let alert = DistillationAlert {
            source: "test".into(),
            alert_type: AlertType::HighRequestRate,
            confidence: 0.9,
            timestamp: 0,
            detail: "test".into(),
        };
        let alerts = vec![alert.clone(), alert.clone(), alert.clone()];
        assert_eq!(ads.watermark_strength, 1.0);
        ads.adjust_from_alerts(&alerts);
        assert!(ads.watermark_strength > 1.0);
    }

    #[test]
    fn test_antidistil_serialize_roundtrip() {
        let mut ads = AntiDistillationSystem::new();
        ads.record_llm_call(true);
        ads.record_llm_call(false);
        ads.record_llm_call(false);
        assert_eq!(ads.total_calls, 3);
        assert_eq!(ads.refused_calls, 1);
        let json = ads.serialize_to_json();
        assert_eq!(json["total_calls"], 3);
        assert_eq!(json["refused_calls"], 1);
        assert_eq!(json["watermark_strength"], 1.0);

        // Deserialize into a fresh system
        let mut ads2 = AntiDistillationSystem::new();
        let json_str = serde_json::to_string(&json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        ads2.total_calls = parsed.get("total_calls").and_then(|v| v.as_u64()).unwrap_or(0);
        ads2.refused_calls = parsed.get("refused_calls").and_then(|v| v.as_u64()).unwrap_or(0);
        ads2.watermark_strength = parsed.get("watermark_strength").and_then(|v| v.as_f64()).unwrap_or(0.0);
        assert_eq!(ads2.total_calls, 3);
        assert_eq!(ads2.refused_calls, 1);
        assert_eq!(ads2.watermark_strength, 1.0);
    }
}
