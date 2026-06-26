use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};

/// Type alias for an async verifier function used in cascade execution.
///
/// Takes (query, draft_response) and returns (verified_response, cost, latency_ms).
pub type AsyncVerifierFn = Box<
    dyn Fn(String, String) -> Pin<Box<dyn Future<Output = (String, f64, f64)> + Send>>
        + Send
        + Sync,
>;

/// Quality validation configuration for speculative cascade execution.
#[derive(Debug, Clone)]
pub struct QualityConfig {
    /// Minimum tokens for a valid response.
    pub min_tokens: usize,
    /// Maximum tokens before truncation.
    pub max_tokens: usize,
    /// Whether response must contain structural elements (e.g. JSON, list).
    pub require_format: bool,
    /// Expected format label for validation.
    pub expected_format: Option<String>,
    /// Confidence threshold for passing quality check.
    pub confidence_threshold: f64,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            min_tokens: 20,
            max_tokens: 4096,
            require_format: false,
            expected_format: None,
            confidence_threshold: 0.6,
        }
    }
}

/// Quality score from validating an LLM response.
#[derive(Debug, Clone)]
pub struct QualityScore {
    pub passed: bool,
    pub confidence: f64,
    pub length_valid: bool,
    pub format_valid: bool,
    pub reason: String,
}

/// Validates LLM response quality for cascade decisions.
pub struct QualityEngine {
    config: QualityConfig,
    scores_produced: AtomicU64,
}

impl QualityEngine {
    pub fn new(config: QualityConfig) -> Self {
        Self {
            config,
            scores_produced: AtomicU64::new(0),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(QualityConfig::default())
    }

    /// Score a drafter response for escalation decision.
    pub fn score(&self, response: &str) -> QualityScore {
        self.scores_produced.fetch_add(1, Ordering::Relaxed);
        let token_count = response.split_whitespace().count();
        let length_valid =
            token_count >= self.config.min_tokens && token_count <= self.config.max_tokens;
        let format_valid = if self.config.require_format {
            if let Some(ref fmt) = self.config.expected_format {
                match fmt.as_str() {
                    "json" => response.trim().starts_with('{') || response.trim().starts_with('['),
                    "list" => response
                        .lines()
                        .any(|l| l.trim().starts_with('-') || l.trim().starts_with("*")),
                    "code" => {
                        response.contains("fn ")
                            || response.contains("def ")
                            || response.contains("```")
                    }
                    _ => true,
                }
            } else {
                true
            }
        } else {
            true
        };
        let heuristic = self.heuristic_confidence(response, token_count);
        let passed = length_valid && format_valid && heuristic >= self.config.confidence_threshold;
        QualityScore {
            passed,
            confidence: heuristic,
            length_valid,
            format_valid,
            reason: if passed {
                "quality adequate".into()
            } else {
                let mut reasons = Vec::new();
                if !length_valid {
                    reasons.push(format!(
                        "token_count {} outside [{},{}]",
                        token_count, self.config.min_tokens, self.config.max_tokens
                    ));
                }
                if !format_valid {
                    reasons.push("format invalid".into());
                }
                if heuristic < self.config.confidence_threshold {
                    reasons.push(format!(
                        "confidence {:.2} < threshold {:.2}",
                        heuristic, self.config.confidence_threshold
                    ));
                }
                reasons.join("; ")
            },
        }
    }

    /// Heuristic confidence based on response length diversity and formatting.
    fn heuristic_confidence(&self, response: &str, tokens: usize) -> f64 {
        if response.is_empty() {
            return 0.0;
        }
        let mut score = 0.3f64;
        let has_punctuation = response.contains(&['.', '!', '?', ':'][..]);
        let has_capitals = response.chars().any(|c| c.is_uppercase());
        let newline_ratio = response.matches('\n').count() as f64 / tokens.max(1) as f64;
        if has_punctuation {
            score += 0.2;
        }
        if has_capitals {
            score += 0.1;
        }
        if newline_ratio > 0.01 && newline_ratio < 0.5 {
            score += 0.1;
        }
        if tokens > self.config.min_tokens {
            score += 0.2;
        }
        score.min(1.0)
    }

    pub fn scores_produced(&self) -> u64 {
        self.scores_produced.load(Ordering::Relaxed)
    }
}

/// Cascade execution statistics.
#[derive(Debug, Clone, Default)]
pub struct CascadeStats {
    pub total_queries: u64,
    pub drafter_success: u64,
    pub verifier_escalations: u64,
    pub total_drafter_cost: f64,
    pub total_verifier_cost: f64,
    pub avg_drafter_latency_ms: f64,
    pub avg_verifier_latency_ms: f64,
    pub latency_samples: VecDeque<f64>,
}

impl CascadeStats {
    pub fn record_drafter(&mut self, cost: f64, latency_ms: f64) {
        self.total_queries += 1;
        self.drafter_success += 1;
        self.total_drafter_cost += cost;
        self.avg_drafter_latency_ms =
            (self.avg_drafter_latency_ms * (self.drafter_success as f64 - 1.0) + latency_ms)
                / self.drafter_success as f64;
        self.latency_samples.push_back(latency_ms);
        if self.latency_samples.len() > 1000 {
            self.latency_samples.pop_front();
        }
    }

    pub fn record_verifier(&mut self, cost: f64, latency_ms: f64) {
        self.verifier_escalations += 1;
        self.total_verifier_cost += cost;
        self.avg_verifier_latency_ms =
            (self.avg_verifier_latency_ms * (self.verifier_escalations as f64 - 1.0) + latency_ms)
                / self.verifier_escalations as f64;
    }

    pub fn escalation_rate(&self) -> f64 {
        if self.total_queries == 0 {
            return 0.0;
        }
        self.verifier_escalations as f64 / self.total_queries as f64
    }

    pub fn total_cost(&self) -> f64 {
        self.total_drafter_cost + self.total_verifier_cost
    }

    pub fn estimated_savings(&self) -> f64 {
        let hypothetical_all_verifier = self.total_queries as f64
            * (self.total_verifier_cost / self.verifier_escalations.max(1) as f64);
        hypothetical_all_verifier - self.total_cost()
    }
}

/// Speculative cascade execution engine.
///
/// Runs a cheap drafter model first, validates quality, and escalates to
/// an expensive verifier model only when quality is insufficient.
pub struct CascadeEngine {
    pub config: QualityConfig,
    quality: QualityEngine,
    stats: CascadeStats,
    /// Pending queries awaiting cascade processing.
    pub pending_queries: VecDeque<String>,
    /// Completed cascade outcomes available for consumption.
    pub completed_results: VecDeque<(String, CascadeOutcome)>,
}

impl CascadeEngine {
    pub fn new(config: QualityConfig) -> Self {
        Self {
            quality: QualityEngine::new(config.clone()),
            config,
            stats: CascadeStats::default(),
            pending_queries: VecDeque::new(),
            completed_results: VecDeque::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(QualityConfig::default())
    }

    /// Enqueue a query for background cascade processing.
    pub fn enqueue_query(&mut self, query: String) {
        const MAX_PENDING: usize = 500;
        if self.pending_queries.len() >= MAX_PENDING {
            self.pending_queries.pop_front();
        }
        self.pending_queries.push_back(query);
    }

    /// Process one pending query synchronously via provided drafter/verifier closures.
    /// Returns the outcome if a query was processed.
    pub fn process_pending_sync(
        &mut self,
        drafter_fn: &mut dyn FnMut(&str) -> (String, f64, f64),
        verifier_fn: Option<&mut dyn FnMut(&str) -> (String, f64, f64)>,
    ) -> Option<CascadeOutcome> {
        let query = self.pending_queries.pop_front()?;
        let (drafter_resp, d_cost, d_lat) = drafter_fn(&query);
        let outcome = self.run_cascade(&query, &drafter_resp, d_cost, d_lat, verifier_fn);
        self.completed_results.push_back((query, outcome.clone()));
        if self.completed_results.len() > 1000 {
            self.completed_results.pop_front();
        }
        Some(outcome)
    }

    /// Process one pending query asynchronously via provided drafter/verifier closures.
    /// The verifier is an async closure (AsyncVerifierFn) for LLM-backed verification.
    /// Returns the outcome if a query was processed.
    pub async fn process_pending_async(
        &mut self,
        drafter: &mut (dyn FnMut(&str) -> (String, f64, f64) + Send),
        verifier: Option<&AsyncVerifierFn>,
    ) -> Option<CascadeOutcome> {
        let query = self.pending_queries.pop_front()?;
        let (drafter_resp, d_cost, d_lat) = drafter(&query);
        let score = self.quality.score(&drafter_resp);

        let outcome = if score.passed {
            self.stats.record_drafter(d_cost, d_lat);
            CascadeOutcome {
                content: drafter_resp,
                model_tier: "drafter",
                escalated: false,
                confidence: score.confidence,
                cost: d_cost,
                latency_ms: d_lat,
            }
        } else if let Some(vf) = verifier {
            let (v_response, v_cost, v_latency) = vf(query.clone(), drafter_resp.clone()).await;
            self.stats.record_drafter(d_cost, d_lat);
            self.stats.record_verifier(v_cost, v_latency);
            CascadeOutcome {
                content: v_response,
                model_tier: "verifier",
                escalated: true,
                confidence: score.confidence,
                cost: d_cost + v_cost,
                latency_ms: d_lat + v_latency,
            }
        } else {
            self.stats.record_drafter(d_cost, d_lat);
            CascadeOutcome {
                content: drafter_resp,
                model_tier: "drafter",
                escalated: false,
                confidence: score.confidence,
                cost: d_cost,
                latency_ms: d_lat,
            }
        };

        self.completed_results
            .push_back((query.clone(), outcome.clone()));
        if self.completed_results.len() > 1000 {
            self.completed_results.pop_front();
        }
        Some(outcome)
    }

    /// Number of pending queries.
    pub fn pending_count(&self) -> usize {
        self.pending_queries.len()
    }

    pub fn quality_engine(&self) -> &QualityEngine {
        &self.quality
    }

    pub fn stats(&self) -> &CascadeStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut CascadeStats {
        &mut self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = CascadeStats::default();
    }

    /// Run speculative cascade given a drafter result and optional verifier.
    ///
    /// The `drafter_result` is validated by the quality engine.
    /// If it passes, the drafter response is returned directly.
    /// If it fails, the verifier is invoked via the provided closure.
    pub fn run_cascade(
        &mut self,
        query: &str,
        drafter_response: &str,
        drafter_cost: f64,
        drafter_latency: f64,
        verifier: Option<&mut dyn FnMut(&str) -> (String, f64, f64)>,
    ) -> CascadeOutcome {
        let score = self.quality.score(drafter_response);
        if score.passed {
            self.stats.record_drafter(drafter_cost, drafter_latency);
            CascadeOutcome {
                content: drafter_response.to_string(),
                model_tier: "drafter",
                escalated: false,
                confidence: score.confidence,
                cost: drafter_cost,
                latency_ms: drafter_latency,
            }
        } else if let Some(verifier_fn) = verifier {
            let (v_response, v_cost, v_latency) = verifier_fn(query);
            self.stats.record_drafter(drafter_cost, drafter_latency);
            self.stats.record_verifier(v_cost, v_latency);
            CascadeOutcome {
                content: v_response,
                model_tier: "verifier",
                escalated: true,
                confidence: score.confidence,
                cost: drafter_cost + v_cost,
                latency_ms: drafter_latency + v_latency,
            }
        } else {
            self.stats.record_drafter(drafter_cost, drafter_latency);
            CascadeOutcome {
                content: drafter_response.to_string(),
                model_tier: "drafter",
                escalated: false,
                confidence: score.confidence,
                cost: drafter_cost,
                latency_ms: drafter_latency,
            }
        }
    }
}

/// Outcome of a cascade execution.
#[derive(Debug, Clone)]
pub struct CascadeOutcome {
    pub content: String,
    pub model_tier: &'static str,
    pub escalated: bool,
    pub confidence: f64,
    pub cost: f64,
    pub latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- CascadeEngine construction ---

    #[test]
    fn test_new_defaults() {
        let engine = CascadeEngine::new(QualityConfig::default());
        assert_eq!(engine.config.confidence_threshold, 0.6);
        assert_eq!(engine.pending_count(), 0);
        assert!(engine.completed_results.is_empty());
    }

    #[test]
    fn test_with_defaults() {
        let engine = CascadeEngine::with_defaults();
        assert_eq!(engine.config.min_tokens, 20);
        assert_eq!(engine.config.max_tokens, 4096);
        assert!(!engine.config.require_format);
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_confidence_threshold_configurable() {
        let config = QualityConfig {
            confidence_threshold: 0.8,
            ..QualityConfig::default()
        };
        let engine = CascadeEngine::new(config);
        assert_eq!(engine.config.confidence_threshold, 0.8);
    }

    // --- enqueue_query ---

    #[test]
    fn test_enqueue_query_basic() {
        let mut engine = CascadeEngine::with_defaults();
        engine.enqueue_query("hello".into());
        assert_eq!(engine.pending_count(), 1);
        assert_eq!(engine.pending_queries[0], "hello");
    }

    #[test]
    fn test_enqueue_query_bounded() {
        let mut engine = CascadeEngine::with_defaults();
        for i in 0..501 {
            engine.enqueue_query(format!("query_{}", i));
        }
        assert_eq!(engine.pending_count(), 500);
        assert_eq!(engine.pending_queries[0], "query_1");
        assert_eq!(engine.pending_queries[499], "query_500");
    }

    #[test]
    fn test_enqueue_query_at_boundary() {
        let mut engine = CascadeEngine::with_defaults();
        for i in 0..500 {
            engine.enqueue_query(format!("query_{}", i));
        }
        assert_eq!(engine.pending_count(), 500);
        assert_eq!(engine.pending_queries[0], "query_0");
        assert_eq!(engine.pending_queries[499], "query_499");
    }

    // --- process_pending_sync ---

    #[test]
    fn test_process_pending_sync_empty_queue() {
        let mut engine = CascadeEngine::with_defaults();
        let mut drafter = |q: &str| (format!("draft: {}", q), 0.001, 5.0);
        let result = engine.process_pending_sync(
            &mut drafter,
            None::<&mut dyn FnMut(&str) -> (String, f64, f64)>,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_process_pending_sync_with_drafter() {
        let mut engine = CascadeEngine::with_defaults();
        engine.enqueue_query("test query".into());

        let mut drafter = |q: &str| (format!("response to: {}", q), 0.002, 10.0);
        let outcome = engine.process_pending_sync(
            &mut drafter,
            None::<&mut dyn FnMut(&str) -> (String, f64, f64)>,
        );

        assert!(outcome.is_some());
        let outcome = outcome.unwrap();
        assert!(outcome.content.contains("response to:"));
        assert!(!outcome.escalated);
        assert_eq!(outcome.model_tier, "drafter");
        assert_eq!(engine.pending_count(), 0);
        assert_eq!(engine.completed_results.len(), 1);
    }

    #[test]
    fn test_process_pending_sync_verifier_escalation() {
        let mut engine = CascadeEngine::with_defaults();
        engine.enqueue_query("short".into());

        let mut drafter = |_q: &str| ("Hi".into(), 0.001, 5.0);
        let mut verifier = |q: &str| {
            assert_eq!(q, "short");
            ("Verified: Hi".into(), 0.05, 100.0)
        };

        let outcome = engine.process_pending_sync(&mut drafter, Some(&mut verifier));
        assert!(outcome.is_some());
        let outcome = outcome.unwrap();
        assert!(outcome.escalated);
        assert_eq!(outcome.model_tier, "verifier");
        assert_eq!(outcome.content, "Verified: Hi");
    }

    #[test]
    fn test_process_pending_sync_updates_completed() {
        let mut engine = CascadeEngine::with_defaults();
        engine.enqueue_query("q1".into());
        engine.enqueue_query("q2".into());

        let mut drafter = |q: &str| (format!("draft:{}", q), 0.001, 5.0);

        let o1 = engine.process_pending_sync(
            &mut drafter,
            None::<&mut dyn FnMut(&str) -> (String, f64, f64)>,
        );
        let o2 = engine.process_pending_sync(
            &mut drafter,
            None::<&mut dyn FnMut(&str) -> (String, f64, f64)>,
        );

        assert!(o1.is_some() && o2.is_some());
        assert_eq!(engine.completed_results.len(), 2);
        assert!(engine.completed_results[0].0.contains("q1"));
        assert!(engine.completed_results[1].0.contains("q2"));
    }

    // --- run_cascade ---

    #[test]
    fn test_run_cascade_drafter_passes() {
        let mut engine = CascadeEngine::with_defaults();
        let response = "A sufficiently long response with proper punctuation. Multiple sentences that should pass the quality threshold without issues.";
        let outcome = engine.run_cascade(
            "query",
            response,
            0.001,
            5.0,
            None::<&mut dyn FnMut(&str) -> (String, f64, f64)>,
        );
        assert!(!outcome.escalated);
        assert_eq!(outcome.model_tier, "drafter");
        assert_eq!(outcome.content, response);
    }

    #[test]
    fn test_run_cascade_empty_response_escalates() {
        let mut engine = CascadeEngine::with_defaults();
        let mut verifier = |q: &str| {
            assert_eq!(q, "bad query");
            ("verified response".into(), 0.05, 100.0)
        };
        let outcome = engine.run_cascade("bad query", "", 0.001, 5.0, Some(&mut verifier));
        assert!(outcome.escalated);
        assert_eq!(outcome.model_tier, "verifier");
        assert_eq!(outcome.content, "verified response");
    }

    #[test]
    fn test_run_cascade_no_verifier_fallback() {
        let mut engine = CascadeEngine::with_defaults();
        let outcome = engine.run_cascade(
            "query",
            "Hi",
            0.001,
            5.0,
            None::<&mut dyn FnMut(&str) -> (String, f64, f64)>,
        );
        assert!(!outcome.escalated);
        assert_eq!(outcome.model_tier, "drafter");
        assert_eq!(outcome.content, "Hi");
    }

    // --- QualityEngine ---

    #[test]
    fn test_quality_score_length_invalid() {
        let engine = QualityEngine::with_defaults();
        let score = engine.score("too short");
        assert!(!score.length_valid);
        assert!(!score.passed);
    }

    #[test]
    fn test_quality_score_format_json() {
        let mut config = QualityConfig::default();
        config.require_format = true;
        config.expected_format = Some("json".into());
        let engine = QualityEngine::new(config);
        let score = engine.score("{\"key\": \"value\"}");
        assert!(score.format_valid);

        let score2 = engine.score("just text");
        assert!(!score2.format_valid);
    }

    #[test]
    fn test_quality_score_format_code() {
        let mut config = QualityConfig::default();
        config.require_format = true;
        config.expected_format = Some("code".into());
        let engine = QualityEngine::new(config);
        let score = engine.score("fn hello() { }");
        assert!(score.format_valid);

        let score2 = engine.score("some prose without code markers");
        assert!(!score2.format_valid);
    }

    #[test]
    fn test_heuristic_confidence_empty() {
        let engine = QualityEngine::with_defaults();
        assert_eq!(engine.score("").confidence, 0.0);
    }

    // --- CascadeStats ---

    #[test]
    fn test_cascade_stats_default() {
        let stats = CascadeStats::default();
        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.drafter_success, 0);
        assert_eq!(stats.verifier_escalations, 0);
        assert_eq!(stats.total_cost(), 0.0);
        assert_eq!(stats.escalation_rate(), 0.0);
    }

    #[test]
    fn test_cascade_stats_tracking() {
        let mut stats = CascadeStats::default();
        stats.record_drafter(0.001, 5.0);
        assert_eq!(stats.total_queries, 1);
        assert_eq!(stats.drafter_success, 1);
        assert!((stats.avg_drafter_latency_ms - 5.0).abs() < 1e-10);

        stats.record_verifier(0.05, 100.0);
        assert_eq!(stats.verifier_escalations, 1);
        assert!((stats.avg_verifier_latency_ms - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_cascade_stats_escalation_rate() {
        let mut stats = CascadeStats::default();
        assert_eq!(stats.escalation_rate(), 0.0);
        stats.record_drafter(0.001, 5.0);
        stats.record_drafter(0.001, 5.0);
        stats.record_verifier(0.05, 100.0);
        assert!((stats.escalation_rate() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_cascade_stats_latency_samples_bounded() {
        let mut stats = CascadeStats::default();
        for _ in 0..2000 {
            stats.record_drafter(0.001, 5.0);
        }
        assert_eq!(stats.latency_samples.len(), 1000);
    }

    #[test]
    fn test_reset_stats() {
        let mut engine = CascadeEngine::with_defaults();
        let mut drafter = |_q: &str| {
            (
                "A long enough response with proper punctuation. Multiple sentences to pass quality."
                    .into(),
                0.001,
                5.0,
            )
        };
        engine.enqueue_query("test".into());
        engine.process_pending_sync(
            &mut drafter,
            None::<&mut dyn FnMut(&str) -> (String, f64, f64)>,
        );
        assert!(engine.stats().total_queries > 0);
        engine.reset_stats();
        assert_eq!(engine.stats().total_queries, 0);
    }

    #[test]
    fn test_cascade_outcome_fields() {
        let outcome = CascadeOutcome {
            content: "result".into(),
            model_tier: "drafter",
            escalated: false,
            confidence: 0.95,
            cost: 0.001,
            latency_ms: 5.0,
        };
        assert_eq!(outcome.content, "result");
        assert!(!outcome.escalated);
    }

    #[test]
    fn test_scores_produced() {
        let engine = QualityEngine::with_defaults();
        assert_eq!(engine.scores_produced(), 0);
        engine.score("test");
        assert_eq!(engine.scores_produced(), 1);
    }

    #[test]
    fn test_cascade_stats_estimated_savings() {
        let mut stats = CascadeStats::default();
        stats.record_drafter(0.001, 5.0);
        stats.record_verifier(0.05, 100.0);
        let savings = stats.estimated_savings();
        assert!(savings > 0.0);
    }
}
