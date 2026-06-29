use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::capability_extractor::{CapabilityExtractor, DemonstratedCapability};
use super::capture::{CaptureBuffer, CapturedInteraction};
use super::knowledge_extractor::{KnowledgeExtractor, KnowledgeFragment};
use super::pattern_extractor::{BehavioralPattern, PatternExtractor};

/// Aggregated distillation report — all extracted insights from a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationReport {
    /// Timestamp of distillation
    pub timestamp_ms: u64,
    /// How many interactions were analyzed
    pub total_interactions: usize,
    /// Total capture count across system lifetime
    pub total_captured: u64,
    /// Extracted behavioral patterns
    pub behavioral_patterns: Vec<BehavioralPattern>,
    /// Extracted capabilities
    pub capabilities: Vec<DemonstratedCapability>,
    /// Extracted knowledge fragments
    pub knowledge_fragments: Vec<KnowledgeFragment>,
    /// Per-model performance summary
    pub model_performance: Vec<ModelPerformance>,
    /// Per-provider statistics
    pub provider_stats: Vec<ProviderStat>,
    /// Recommended actions for self-improvement
    pub recommendations: Vec<String>,
    /// Best model per topic
    pub best_model_per_topic: Vec<(String, String, f64)>,
}

/// Performance summary for a single model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model: String,
    pub interaction_count: usize,
    pub avg_outcome: f64,
    pub avg_latency_ms: f64,
    pub avg_prompt_tokens: f64,
    pub avg_completion_tokens: f64,
    pub avg_word_count: f64,
    pub code_response_ratio: f64,
    pub success_rate: f64,
}

/// Provider-level statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStat {
    pub provider: String,
    pub interaction_count: usize,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub models_used: Vec<String>,
}

/// The unified cross-model distillation orchestrator.
///
/// Combines pattern extraction, capability extraction, and knowledge extraction
/// into a single pipeline. Feed it captured interactions from ANY model/provider
/// and get back structured insights for self-improvement.
///
/// Model-agnostic by construction: no model-specific logic anywhere.
#[derive(Clone)]
pub struct CrossModelDistiller {
    capture_buffer: Arc<std::sync::Mutex<CaptureBuffer>>,
    pattern_extractor: PatternExtractor,
    capability_extractor: CapabilityExtractor,
    knowledge_extractor: KnowledgeExtractor,
    distillation_count: u64,
    last_report: Option<DistillationReport>,
    /// Configuration flags
    pub extract_patterns: bool,
    pub extract_capabilities: bool,
    pub extract_knowledge: bool,
    pub generate_recommendations: bool,
}

impl CrossModelDistiller {
    pub fn new(capture_buffer: Arc<std::sync::Mutex<CaptureBuffer>>) -> Self {
        Self {
            capture_buffer,
            pattern_extractor: PatternExtractor::new(),
            capability_extractor: CapabilityExtractor::new(),
            knowledge_extractor: KnowledgeExtractor::new(),
            distillation_count: 0,
            last_report: None,
            extract_patterns: true,
            extract_capabilities: true,
            extract_knowledge: true,
            generate_recommendations: true,
        }
    }

    /// Run a full distillation pass on all currently captured interactions.
    /// Drains the buffer — interactions are consumed after distillation.
    pub fn distill(&mut self) -> DistillationReport {
        let interactions = {
            let mut buf = self.capture_buffer.lock().unwrap();
            let total_captured = buf.capture_count();
            let interactions = buf.drain();
            if interactions.is_empty() {
                return DistillationReport {
                    timestamp_ms: Self::now_ms(),
                    total_interactions: 0,
                    total_captured,
                    behavioral_patterns: vec![],
                    capabilities: vec![],
                    knowledge_fragments: vec![],
                    model_performance: vec![],
                    provider_stats: vec![],
                    recommendations: vec![],
                    best_model_per_topic: vec![],
                };
            }
            (interactions, total_captured)
        };

        let (interactions, total_captured) = interactions;

        let total = interactions.len();

        // Phase 1: Pattern extraction
        let behavioral_patterns = if self.extract_patterns {
            self.pattern_extractor.extract(&interactions)
        } else {
            vec![]
        };

        // Phase 2: Capability extraction
        let capabilities = if self.extract_capabilities {
            self.capability_extractor.extract(&interactions)
        } else {
            vec![]
        };

        // Phase 3: Knowledge extraction
        let knowledge_fragments = if self.extract_knowledge {
            self.knowledge_extractor.extract(&interactions)
        } else {
            vec![]
        };

        // Phase 4: Performance analysis
        let model_performance = Self::compute_model_performance(&interactions);
        let provider_stats = Self::compute_provider_stats(&interactions);

        // Phase 5: Best model per topic
        let best_model_per_topic = self.pattern_extractor.best_model_per_topic(&interactions);

        // Phase 6: Recommendations
        let recommendations = if self.generate_recommendations {
            Self::generate_recommendations(&behavioral_patterns, &capabilities, &model_performance)
        } else {
            vec![]
        };

        self.distillation_count += 1;

        let report = DistillationReport {
            timestamp_ms: Self::now_ms(),
            total_interactions: total,
            total_captured,
            behavioral_patterns,
            capabilities,
            knowledge_fragments,
            model_performance,
            provider_stats,
            recommendations,
            best_model_per_topic,
        };

        self.last_report = Some(report.clone());
        report
    }

    /// Run distillation by model — get per-model insights.
    pub fn distill_by_model(&mut self) -> HashMap<String, DistillationReport> {
        let by_model = {
            let buf = self.capture_buffer.lock().unwrap();
            buf.by_model()
        };

        let mut reports = HashMap::new();

        let total_captured = {
            let buf = self.capture_buffer.lock().unwrap();
            buf.capture_count()
        };

        for (model, interactions) in by_model {
            let total = interactions.len();

            let patterns = self.pattern_extractor.extract(&interactions);
            let capabilities = self.capability_extractor.extract(&interactions);
            let knowledge = self.knowledge_extractor.extract(&interactions);
            let perf = Self::compute_model_performance(&interactions);

            let recs = if self.generate_recommendations {
                Self::generate_recommendations(&patterns, &capabilities, &perf)
            } else {
                vec![]
            };

            reports.insert(
                model,
                DistillationReport {
                    timestamp_ms: Self::now_ms(),
                    total_interactions: total,
                    total_captured,
                    behavioral_patterns: patterns,
                    capabilities,
                    knowledge_fragments: knowledge,
                    model_performance: perf,
                    provider_stats: vec![],
                    recommendations: recs,
                    best_model_per_topic: vec![],
                },
            );
        }

        reports
    }

    /// Factory: create a standardized self-improvement probe.
    /// Use this as a transparent wrapper around ANY LLM provider.
    pub fn create_probe<P>(inner: Arc<P>) -> super::capture::DistillationProbe<P> {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(500)));
        super::capture::DistillationProbe::new(inner, buffer)
    }

    // ── Internal helpers ──

    fn compute_model_performance(interactions: &[CapturedInteraction]) -> Vec<ModelPerformance> {
        let mut by_model: HashMap<String, Vec<&CapturedInteraction>> = HashMap::new();
        for i in interactions {
            by_model.entry(i.model.clone()).or_default().push(i);
        }

        let mut results = Vec::new();
        for (model, group) in by_model {
            let n = group.len() as f64;
            let avg_outcome: f64 = group.iter().map(|i| i.outcome_score).sum::<f64>() / n;
            let avg_latency: f64 = group.iter().map(|i| i.latency_ms as f64).sum::<f64>() / n;
            let avg_prompt: f64 = group.iter().map(|i| i.prompt_tokens as f64).sum::<f64>() / n;
            let avg_completion: f64 = group
                .iter()
                .map(|i| i.completion_tokens as f64)
                .sum::<f64>()
                / n;
            let avg_words: f64 = group.iter().map(|i| i.word_count as f64).sum::<f64>() / n;
            let code_count = group.iter().filter(|i| i.has_code).count();
            let code_ratio = code_count as f64 / n;
            let success_count = group.iter().filter(|i| i.success).count();
            let success_rate = success_count as f64 / n;

            results.push(ModelPerformance {
                model,
                interaction_count: group.len(),
                avg_outcome,
                avg_latency_ms: avg_latency,
                avg_prompt_tokens: avg_prompt,
                avg_completion_tokens: avg_completion,
                avg_word_count: avg_words,
                code_response_ratio: code_ratio,
                success_rate,
            });
        }

        results.sort_by(|a, b| {
            b.avg_outcome
                .partial_cmp(&a.avg_outcome)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    fn compute_provider_stats(interactions: &[CapturedInteraction]) -> Vec<ProviderStat> {
        let mut by_provider: HashMap<String, Vec<&CapturedInteraction>> = HashMap::new();
        for i in interactions {
            by_provider.entry(i.provider.clone()).or_default().push(i);
        }

        let mut results = Vec::new();
        for (provider, group) in by_provider {
            let n = group.len();
            let success_count = group.iter().filter(|i| i.success).count();
            let success_rate = if n > 0 {
                success_count as f64 / n as f64
            } else {
                0.0
            };
            let avg_latency: f64 =
                group.iter().map(|i| i.latency_ms as f64).sum::<f64>() / n.max(1) as f64;

            let mut models: Vec<String> = group.iter().map(|i| i.model.clone()).collect();
            models.sort();
            models.dedup();

            results.push(ProviderStat {
                provider,
                interaction_count: n,
                success_rate,
                avg_latency_ms: avg_latency,
                models_used: models,
            });
        }

        results.sort_by(|a, b| b.interaction_count.cmp(&a.interaction_count));

        results
    }

    fn generate_recommendations(
        patterns: &[BehavioralPattern],
        capabilities: &[DemonstratedCapability],
        model_perf: &[ModelPerformance],
    ) -> Vec<String> {
        let mut recs = Vec::new();

        // Recommendation: use patterns with high confidence
        for p in patterns.iter().take(3) {
            if p.avg_outcome > 0.7 && p.confidence > 0.6 {
                recs.push(format!(
                    "Pattern[{}]: {} (avg_outcome={:.3}, confidence={:.3}, {} obs)",
                    p.topic, p.description, p.avg_outcome, p.confidence, p.observation_count
                ));
            }
        }

        // Recommendation: nurture strong capabilities
        for cap in capabilities.iter().take(3) {
            if cap.proficiency > 0.7 {
                recs.push(format!(
                    "Capability[{}]: proficiency={:.3} across {} models, {} observations",
                    cap.name,
                    cap.proficiency,
                    cap.observed_models.len(),
                    cap.observation_count
                ));
            }
        }

        // Recommendation: capability gaps
        let weak_caps: Vec<&DemonstratedCapability> = capabilities
            .iter()
            .filter(|c| c.proficiency < 0.4 && c.observation_count > 2)
            .collect();
        for cap in weak_caps.iter().take(2) {
            recs.push(format!(
                "Improve[{}]: proficiency={:.3} is low, needs more practice",
                cap.name, cap.proficiency
            ));
        }

        // Recommendation: model selection advice
        if let Some(best) = model_perf.first() {
            if best.avg_outcome > 0.7 {
                recs.push(format!(
                    "Preferred[{}]: avg_outcome={:.3}, {} calls, {:.0}ms latency",
                    best.model, best.avg_outcome, best.interaction_count, best.avg_latency_ms
                ));
            }
        }

        recs
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Get the last distillation report.
    pub fn last_report(&self) -> Option<&DistillationReport> {
        self.last_report.as_ref()
    }

    /// How many distillations have been run.
    pub fn distillation_count(&self) -> u64 {
        self.distillation_count
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let report_info = match &self.last_report {
            Some(r) => format!(
                "last: {} total_obs, {} patterns, {} caps, {} knowledge",
                r.total_interactions,
                r.behavioral_patterns.len(),
                r.capabilities.len(),
                r.knowledge_fragments.len()
            ),
            None => "no report yet".to_string(),
        };
        let buf_info = {
            let buf = self.capture_buffer.lock().unwrap();
            buf.summary()
        };
        format!(
            "CrossModelDistiller[distillations={} patterns={} caps={} knowledge={} | {} | {}]",
            self.distillation_count,
            self.pattern_extractor.patterns_extracted(),
            self.capability_extractor.extractions(),
            self.knowledge_extractor.extractions(),
            buf_info,
            report_info,
        )
    }
}

impl Default for CrossModelDistiller {
    fn default() -> Self {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(500)));
        Self::new(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distillation::capture::CapturedInteraction;

    fn sample_interactions() -> Vec<CapturedInteraction> {
        vec![
            CapturedInteraction::new(
                "openai",
                "gpt-4",
                "",
                "write a rust function",
                "```rust\nfn hello() -> String {\n    \"world\".to_string()\n}\n```",
                100,
                200,
                500,
                true,
                "stop",
            ),
            CapturedInteraction::new(
                "groq",
                "llama-3-70b",
                "",
                "debug this rust error",
                "The error is a borrow checker issue. Use clone() or change ownership.",
                80,
                150,
                300,
                true,
                "stop",
            ),
            CapturedInteraction::new(
                "openai",
                "gpt-4",
                "",
                "explain async rust",
                "Async in Rust uses poll-based futures. The tokio runtime drives execution.",
                120,
                300,
                800,
                true,
                "stop",
            ),
        ]
    }

    /// Integration test simulating multi-model interaction flow.
    /// Shows how CrossModelDistiller works end-to-end:
    ///   1. Multiple models/providers interact
    ///   2. Distillation captures patterns, capabilities, knowledge
    ///   3. Report contains structured insights
    #[test]
    fn test_end_to_end_distillation() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(1000)));

        // Simulate interactions from different models/providers
        let interactions = vec![
            // GPT-4: code generation task
            CapturedInteraction::new(
                "openai", "gpt-4", "", "write a rust function to parse json",
                "```rust\nuse serde_json;\n\nfn parse_json(data: &str) -> Result<Value, Error> {\n    serde_json::from_str(data)\n}\n```",
                120, 350, 800, true, "stop",
            ),
            // Claude: explanation task
            CapturedInteraction::new(
                "anthropic", "claude-3-opus", "", "explain async rust",
                "Async in Rust uses poll-based futures. The tokio runtime drives execution. Unlike threads, async tasks are cooperative and yield at await points.",
                150, 420, 1200, true, "stop",
            ),
            // Llama: debugging task
            CapturedInteraction::new(
                "groq", "llama-3-70b", "", "debug this borrow checker error",
                "The issue is that you're trying to mutate self while holding an immutable reference. Use RefCell or restructure your code to avoid the conflict.",
                80, 200, 350, true, "stop",
            ),
            // DeepSeek: system design
            CapturedInteraction::new(
                "deepseek", "deepseek-coder", "", "design a microservice architecture",
                "Use API Gateway pattern with service discovery. Each service owns its data store. Communication via message queue for async operations.",
                200, 600, 1500, true, "stop",
            ),
            // Gemini: similar to GPT-4 for cross-validation
            CapturedInteraction::new(
                "google", "gemini-pro", "", "write a rust function to parse json",
                "```rust\nfn parse(data: &str) -> serde_json::Result<serde_json::Value> {\n    serde_json::from_str(data)\n}\n```",
                100, 280, 650, true, "stop",
            ),
        ];

        {
            let mut buf = buffer.lock().unwrap();
            for i in interactions {
                buf.push(i);
            }
        }

        let mut distiller = CrossModelDistiller::new(buffer);

        // Run distillation
        let report = distiller.distill();

        // Verify report structure
        assert_eq!(report.total_interactions, 5);
        assert!(report.total_captured >= 5);
        assert!(report.timestamp_ms > 0);

        // Should detect patterns from multiple interactions
        // At minimum, patterns may exist if structure hashes match
        assert!(!report.model_performance.is_empty());

        // Should have per-model stats
        let model_names: Vec<&str> = report
            .model_performance
            .iter()
            .map(|m| m.model.as_str())
            .collect();
        assert!(model_names.contains(&"gpt-4"));
        assert!(model_names.contains(&"claude-3-opus"));

        // Should have provider stats
        assert!(!report.provider_stats.is_empty());

        // Summary should work
        let summary = distiller.summary();
        assert!(summary.contains("CrossModelDistiller["));

        // Report should be retrievable
        assert!(distiller.last_report().is_some());
        assert_eq!(distiller.distillation_count(), 1);
    }

    #[test]
    fn test_distill_empty() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(100)));
        let mut distiller = CrossModelDistiller::new(buffer);
        let report = distiller.distill();
        assert_eq!(report.total_interactions, 0);
        assert!(report.behavioral_patterns.is_empty());
        assert!(report.capabilities.is_empty());
        assert!(report.knowledge_fragments.is_empty());
    }

    #[test]
    fn test_distill_with_interactions() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(500)));
        {
            let mut buf = buffer.lock().unwrap();
            for i in sample_interactions() {
                buf.push(i);
            }
        }

        let mut distiller = CrossModelDistiller::new(buffer);
        let report = distiller.distill();

        assert_eq!(report.total_interactions, 3);
        // Should find patterns (3 interactions may not meet min_observations)
        assert!(report.total_captured >= 3);
    }

    #[test]
    fn test_compute_model_performance() {
        let interactions = sample_interactions();
        let perf = CrossModelDistiller::compute_model_performance(&interactions);
        assert!(!perf.is_empty());

        let gpt4 = perf.iter().find(|p| p.model == "gpt-4");
        assert!(gpt4.is_some());
        if let Some(g) = gpt4 {
            assert_eq!(g.interaction_count, 2);
            assert!(g.avg_outcome > 0.0);
        }
    }

    #[test]
    fn test_compute_provider_stats() {
        let interactions = sample_interactions();
        let stats = CrossModelDistiller::compute_provider_stats(&interactions);
        assert!(!stats.is_empty());

        let openai = stats.iter().find(|s| s.provider == "openai");
        assert!(openai.is_some());
        if let Some(o) = openai {
            assert_eq!(o.interaction_count, 2);
            assert_eq!(o.success_rate, 1.0);
        }
    }

    #[test]
    fn test_generate_recommendations() {
        let patterns = vec![BehavioralPattern {
            id: "test".into(),
            description: "test pattern".into(),
            structure_hash: 0,
            topic: "rust".into(),
            avg_outcome: 0.85,
            observation_count: 10,
            confidence: 0.75,
            has_code: true,
            code_block_count_avg: 1.5,
            has_sections: true,
            has_lists: false,
            avg_word_count: 200.0,
            observed_models: vec!["gpt-4".into()],
            observed_providers: vec!["openai".into()],
        }];

        let capabilities = vec![DemonstratedCapability {
            name: "code_generation".into(),
            description: "good at code".into(),
            trigger_keywords: vec!["rust".into()],
            proficiency: 0.85,
            observation_count: 10,
            observed_models: vec!["gpt-4".into()],
            sub_capabilities: vec![],
        }];

        let perf = vec![ModelPerformance {
            model: "gpt-4".into(),
            interaction_count: 10,
            avg_outcome: 0.85,
            avg_latency_ms: 500.0,
            avg_prompt_tokens: 100.0,
            avg_completion_tokens: 200.0,
            avg_word_count: 150.0,
            code_response_ratio: 0.7,
            success_rate: 1.0,
        }];

        let recs = CrossModelDistiller::generate_recommendations(&patterns, &capabilities, &perf);
        assert!(!recs.is_empty());
        assert!(recs.iter().any(|r| r.contains("Pattern")));
        assert!(recs.iter().any(|r| r.contains("Capability")));
        assert!(recs.iter().any(|r| r.contains("Preferred")));
    }

    #[test]
    fn test_summary() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(100)));
        let mut distiller = CrossModelDistiller::new(buffer);
        distiller.distill();
        let summary = distiller.summary();
        assert!(summary.contains("CrossModelDistiller["));
    }

    #[test]
    fn test_distill_by_model() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(500)));
        {
            let mut buf = buffer.lock().unwrap();
            for i in sample_interactions() {
                buf.push(i);
            }
        }

        let mut distiller = CrossModelDistiller::new(buffer);
        let by_model = distiller.distill_by_model();
        assert!(by_model.contains_key("gpt-4"));
        assert!(by_model.contains_key("llama-3-70b"));
    }

    #[test]
    fn test_distillation_count() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(100)));
        {
            let mut buf = buffer.lock().unwrap();
            buf.push(CapturedInteraction::new(
                "test", "m1", "", "hello", "world", 10, 20, 50, true, "stop",
            ));
        }
        let mut distiller = CrossModelDistiller::new(buffer);
        assert_eq!(distiller.distillation_count(), 0);
        distiller.distill();
        assert_eq!(distiller.distillation_count(), 1);
        {
            let mut buf = distiller.capture_buffer.lock().unwrap();
            buf.push(CapturedInteraction::new(
                "test", "m1", "", "hello2", "world2", 10, 20, 50, true, "stop",
            ));
        }
        distiller.distill();
        assert_eq!(distiller.distillation_count(), 2);
    }

    #[test]
    fn test_last_report() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(100)));
        {
            let mut buf = buffer.lock().unwrap();
            buf.push(CapturedInteraction::new(
                "test", "m1", "", "hello", "world", 10, 20, 50, true, "stop",
            ));
        }
        let mut distiller = CrossModelDistiller::new(buffer);
        assert!(distiller.last_report().is_none());
        distiller.distill();
        assert!(distiller.last_report().is_some());
    }

    #[test]
    fn test_disable_extractors() {
        let buffer = Arc::new(std::sync::Mutex::new(CaptureBuffer::new(500)));
        {
            let mut buf = buffer.lock().unwrap();
            buf.push(CapturedInteraction::new(
                "test",
                "m1",
                "",
                "write code",
                "fn a() {}",
                10,
                20,
                50,
                true,
                "stop",
            ));
        }

        let mut distiller = CrossModelDistiller::new(buffer);
        distiller.extract_patterns = false;
        distiller.extract_capabilities = false;
        distiller.extract_knowledge = false;

        let report = distiller.distill();
        assert!(report.behavioral_patterns.is_empty());
        assert!(report.capabilities.is_empty());
        assert!(report.knowledge_fragments.is_empty());
        assert_eq!(report.total_interactions, 1);
    }
}
