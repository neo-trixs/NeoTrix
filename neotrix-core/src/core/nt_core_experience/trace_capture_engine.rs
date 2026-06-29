use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Severity level of a captured trace event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl TraceSeverity {
    pub fn is_error_or_worse(self) -> bool {
        matches!(self, TraceSeverity::Error | TraceSeverity::Critical)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TraceSeverity::Debug => "debug",
            TraceSeverity::Info => "info",
            TraceSeverity::Warning => "warning",
            TraceSeverity::Error => "error",
            TraceSeverity::Critical => "critical",
        }
    }
}

/// Where the trace event originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceSource {
    CalibrationEngine,
    EvolutionEngine,
    EscherLoop,
    LlmCall,
    PipelineStep,
    MetaLayer,
    External,
}

impl TraceSource {
    pub fn as_str(self) -> &'static str {
        match self {
            TraceSource::CalibrationEngine => "calibration_engine",
            TraceSource::EvolutionEngine => "evolution_engine",
            TraceSource::EscherLoop => "escher_loop",
            TraceSource::LlmCall => "llm_call",
            TraceSource::PipelineStep => "pipeline_step",
            TraceSource::MetaLayer => "meta_layer",
            TraceSource::External => "external",
        }
    }
}

/// A single captured trace event with structured metadata.
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub cycle: u64,
    pub timestamp: Instant,
    pub source: TraceSource,
    pub severity: TraceSeverity,
    pub category: String,
    pub summary: String,
    pub detail: String,
    pub key_values: Vec<(String, String)>,
}

/// Aggregation statistics for a (category, severity) pair over the current buffer window.
#[derive(Debug, Clone)]
pub struct AggregatedTrace {
    pub category: String,
    pub severity: TraceSeverity,
    pub count: usize,
    pub latest_summary: String,
    pub first_seen: Instant,
    pub last_seen: Instant,
}

/// Summary statistics of the current capture buffer.
#[derive(Debug, Clone)]
pub struct TraceCaptureStats {
    pub total_events: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub unique_categories: usize,
    pub error_rate: f64,
    pub llm_call_count: usize,
    pub evolution_step_count: usize,
    pub calibration_event_count: usize,
}

/// Active trace capture engine — hooks into runtime events, captures structured traces.
///
/// Implements GEPA-style execution trace capture (ICLR 2026 Oral):
/// - Passive: accepts events via `capture()`
/// - Active: convenience methods for LLM calls, evolution steps, calibration events
/// - Storage: ring buffer with configurable capacity
/// - Aggregation: auto-aggregates by (category, severity) for reflective analysis
pub struct TraceCaptureEngine {
    events: VecDeque<TraceEvent>,
    max_events: usize,
    aggregations: HashMap<(String, TraceSeverity), AggregatedTrace>,
    next_cycle: u64,
}

impl TraceCaptureEngine {
    /// Create a new engine with a maximum ring-buffer capacity.
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events.min(1024)),
            max_events,
            aggregations: HashMap::new(),
            next_cycle: 0,
        }
    }

    /// Store a captured trace event and update aggregations.
    /// Evicts the oldest event if capacity is exceeded.
    pub fn capture(&mut self, event: TraceEvent) {
        let cat = event.category.clone();
        let sev = event.severity;
        let ts = event.timestamp;
        let summary = event.summary.clone();

        if self.events.len() >= self.max_events {
            if let Some(evicted) = self.events.pop_front() {
                self.decrement_aggregation(&evicted.category, evicted.severity);
            }
        }

        self.increment_aggregation(&cat, sev, &summary, ts);
        self.events.push_back(event);
    }

    /// Convenience: capture an LLM call trace.
    pub fn capture_llm_call(
        &mut self,
        model: &str,
        prompt_len: usize,
        response_len: usize,
        success: bool,
        latency_ms: u64,
        error: Option<&str>,
    ) {
        let (severity, detail) = if success {
            (TraceSeverity::Info, String::new())
        } else {
            (
                TraceSeverity::Error,
                error.unwrap_or("unknown error").to_string(),
            )
        };

        self.capture(TraceEvent {
            cycle: self.next_cycle,
            timestamp: Instant::now(),
            source: TraceSource::LlmCall,
            severity,
            category: "llm_call".into(),
            summary: if success {
                format!("{} {} tok in {}ms", model, response_len, latency_ms)
            } else {
                format!("{} FAILED: {}", model, detail)
            },
            detail,
            key_values: vec![
                ("model".into(), model.into()),
                ("prompt_len".into(), prompt_len.to_string()),
                ("response_len".into(), response_len.to_string()),
                ("success".into(), success.to_string()),
                ("latency_ms".into(), latency_ms.to_string()),
            ],
        });
    }

    /// Convenience: capture an evolution step event.
    pub fn capture_evolution_step(
        &mut self,
        step_name: &str,
        task_id: u64,
        result: &str,
        metrics: Vec<(String, f64)>,
    ) {
        let severity = match result {
            "completed" | "accepted" => TraceSeverity::Info,
            "rejected" | "rolled_back" => TraceSeverity::Warning,
            "failed" | "crashed" => TraceSeverity::Error,
            _ => TraceSeverity::Debug,
        };

        let mut kv: Vec<(String, String)> = vec![
            ("step_name".into(), step_name.into()),
            ("task_id".into(), task_id.to_string()),
            ("result".into(), result.into()),
        ];
        for (k, v) in &metrics {
            kv.push((k.clone(), format!("{:.4}", v)));
        }

        let metrics_summary = if metrics.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = metrics
                .iter()
                .map(|(k, v)| format!("{}={:.3}", k, v))
                .collect();
            parts.join(", ")
        };

        self.capture(TraceEvent {
            cycle: self.next_cycle,
            timestamp: Instant::now(),
            source: TraceSource::EvolutionEngine,
            severity,
            category: "evolution_step".into(),
            summary: format!(
                "[{}] task={} → {} {}",
                step_name, task_id, result, metrics_summary
            ),
            detail: String::new(),
            key_values: kv,
        });
    }

    /// Convenience: capture a calibration event.
    pub fn capture_calibration(
        &mut self,
        domain: &str,
        ece: f64,
        surprise: f64,
        sample_count: usize,
    ) {
        let severity = if ece > 0.15 || surprise > 0.3 {
            TraceSeverity::Warning
        } else if ece > 0.25 || surprise > 0.5 {
            TraceSeverity::Error
        } else {
            TraceSeverity::Info
        };

        self.capture(TraceEvent {
            cycle: self.next_cycle,
            timestamp: Instant::now(),
            source: TraceSource::CalibrationEngine,
            severity,
            category: "calibration".into(),
            summary: format!(
                "domain={} ece={:.4} surprise={:.4} samples={}",
                domain, ece, surprise, sample_count
            ),
            detail: String::new(),
            key_values: vec![
                ("domain".into(), domain.into()),
                ("ece".into(), format!("{:.4}", ece)),
                ("surprise".into(), format!("{:.4}", surprise)),
                ("sample_count".into(), sample_count.to_string()),
            ],
        });
    }

    /// Mark a cycle boundary — increments the internal cycle counter.
    pub fn cycle_boundary(&mut self) {
        self.next_cycle = self.next_cycle.wrapping_add(1);
    }

    /// Current cycle number.
    pub fn current_cycle(&self) -> u64 {
        self.next_cycle
    }

    /// Last N events with severity >= Error, most recent first.
    pub fn recent_errors(&self, n: usize) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .rev()
            .filter(|e| e.severity.is_error_or_worse())
            .take(n)
            .collect()
    }

    /// All aggregations sorted by count descending.
    pub fn aggregated(&self) -> Vec<&AggregatedTrace> {
        let mut items: Vec<&AggregatedTrace> = self.aggregations.values().collect();
        items.sort_by(|a, b| b.count.cmp(&a.count));
        items
    }

    /// Filter events by category.
    pub fn events_by_category(&self, category: &str) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Clear all state.
    pub fn clear(&mut self) {
        self.events.clear();
        self.aggregations.clear();
        self.next_cycle = 0;
    }

    /// Fraction of events with severity >= Error in the buffer.
    pub fn error_rate(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }
        let err_count = self
            .events
            .iter()
            .filter(|e| e.severity.is_error_or_worse())
            .count();
        err_count as f64 / self.events.len() as f64
    }

    /// Summary statistics of the current buffer.
    pub fn stats(&self) -> TraceCaptureStats {
        let total = self.events.len();
        let err_count = self
            .events
            .iter()
            .filter(|e| e.severity.is_error_or_worse())
            .count();
        let warn_count = self
            .events
            .iter()
            .filter(|e| e.severity == TraceSeverity::Warning)
            .count();
        let mut categories: HashMap<&str, ()> = HashMap::new();
        for e in &self.events {
            categories.insert(e.category.as_str(), ());
        }
        let llm_calls = self.events_by_category("llm_call").len();
        let evo_steps = self.events_by_category("evolution_step").len();
        let cal_events = self.events_by_category("calibration").len();

        TraceCaptureStats {
            total_events: total,
            error_count: err_count,
            warning_count: warn_count,
            unique_categories: categories.len(),
            error_rate: if total > 0 {
                err_count as f64 / total as f64
            } else {
                0.0
            },
            llm_call_count: llm_calls,
            evolution_step_count: evo_steps,
            calibration_event_count: cal_events,
        }
    }

    // --- private helpers ---

    fn increment_aggregation(
        &mut self,
        category: &str,
        severity: TraceSeverity,
        summary: &str,
        timestamp: Instant,
    ) {
        let key = (category.to_string(), severity);
        let entry = self.aggregations.entry(key);
        match entry {
            std::collections::hash_map::Entry::Occupied(mut o) => {
                let agg = o.get_mut();
                agg.count += 1;
                agg.latest_summary = summary.to_string();
                agg.last_seen = timestamp;
            }
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(AggregatedTrace {
                    category: category.to_string(),
                    severity,
                    count: 1,
                    latest_summary: summary.to_string(),
                    first_seen: timestamp,
                    last_seen: timestamp,
                });
            }
        }
    }

    fn decrement_aggregation(&mut self, category: &str, severity: TraceSeverity) {
        let key = (category.to_string(), severity);
        if let std::collections::hash_map::Entry::Occupied(mut o) = self.aggregations.entry(key) {
            let agg = o.get_mut();
            if agg.count > 1 {
                agg.count -= 1;
            } else {
                o.remove();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn make_engine() -> TraceCaptureEngine {
        TraceCaptureEngine::new(100)
    }

    #[test]
    fn test_capture_stores_event() {
        let mut eng = make_engine();
        assert_eq!(eng.stats().total_events, 0);

        eng.capture(TraceEvent {
            cycle: 1,
            timestamp: Instant::now(),
            source: TraceSource::MetaLayer,
            severity: TraceSeverity::Info,
            category: "test".into(),
            summary: "hello".into(),
            detail: "detail".into(),
            key_values: vec![],
        });

        assert_eq!(eng.stats().total_events, 1);
        assert_eq!(eng.events_by_category("test").len(), 1);
    }

    #[test]
    fn test_capture_llm_call() {
        let mut eng = make_engine();

        eng.capture_llm_call("gpt-4", 500, 200, true, 1200, None);
        eng.capture_llm_call("claude-3", 300, 150, false, 3000, Some("timeout"));

        let stats = eng.stats();
        assert_eq!(stats.llm_call_count, 2);

        let errors = eng.recent_errors(10);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, TraceSeverity::Error);
        assert!(errors[0].summary.contains("FAILED"));
    }

    #[test]
    fn test_capture_evolution_step() {
        let mut eng = make_engine();

        eng.capture_evolution_step("propose", 42, "completed", vec![("fitness".into(), 0.95)]);
        eng.capture_evolution_step("evaluate", 7, "failed", vec![]);

        let stats = eng.stats();
        assert_eq!(stats.evolution_step_count, 2);

        let errors = eng.recent_errors(10);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].key_values[0].1, "propose");
    }

    #[test]
    fn test_capture_calibration() {
        let mut eng = make_engine();

        eng.capture_calibration("reasoning", 0.05, 0.10, 100);
        eng.capture_calibration("planning", 0.30, 0.45, 50);

        let stats = eng.stats();
        assert_eq!(stats.calibration_event_count, 2);

        let warnings = eng
            .events
            .iter()
            .filter(|e| e.severity == TraceSeverity::Warning)
            .count();
        assert_eq!(warnings, 1);
    }

    #[test]
    fn test_recent_errors() {
        let mut eng = make_engine();

        for i in 0..20 {
            let sev = if i % 3 == 0 {
                TraceSeverity::Error
            } else {
                TraceSeverity::Info
            };
            eng.capture(TraceEvent {
                cycle: i as u64,
                timestamp: Instant::now(),
                source: TraceSource::PipelineStep,
                severity: sev,
                category: "test".into(),
                summary: format!("event {}", i),
                detail: String::new(),
                key_values: vec![],
            });
        }

        let errors = eng.recent_errors(3);
        assert_eq!(errors.len(), 3);
        for e in &errors {
            assert!(e.severity.is_error_or_worse());
        }
        // Most recent first
        assert_eq!(errors[0].cycle, 18);
        assert_eq!(errors[1].cycle, 15);
        assert_eq!(errors[2].cycle, 12);
    }

    #[test]
    fn test_aggregated_by_category() {
        let mut eng = make_engine();

        for i in 0..10 {
            let cat = if i < 6 { "alpha" } else { "beta" };
            let sev = if i % 2 == 0 {
                TraceSeverity::Info
            } else {
                TraceSeverity::Warning
            };
            eng.capture(TraceEvent {
                cycle: i,
                timestamp: Instant::now(),
                source: TraceSource::External,
                severity: sev,
                category: cat.into(),
                summary: format!("evt {}", i),
                detail: String::new(),
                key_values: vec![],
            });
        }

        let agg = eng.aggregated();
        let total_count: usize = agg.iter().map(|a| a.count).sum();
        assert_eq!(total_count, 10);

        let alpha_entries: Vec<_> = agg.iter().filter(|a| a.category == "alpha").collect();
        assert_eq!(alpha_entries.len(), 2); // Info + Warning
        let alpha_total: usize = alpha_entries.iter().map(|a| a.count).sum();
        assert_eq!(alpha_total, 6);
    }

    #[test]
    fn test_error_rate() {
        let mut eng = make_engine();

        assert_eq!(eng.error_rate(), 0.0);

        eng.capture(TraceEvent {
            cycle: 0,
            timestamp: Instant::now(),
            source: TraceSource::MetaLayer,
            severity: TraceSeverity::Info,
            category: "a".into(),
            summary: "ok".into(),
            detail: String::new(),
            key_values: vec![],
        });
        eng.capture(TraceEvent {
            cycle: 1,
            timestamp: Instant::now(),
            source: TraceSource::MetaLayer,
            severity: TraceSeverity::Error,
            category: "b".into(),
            summary: "err".into(),
            detail: String::new(),
            key_values: vec![],
        });
        eng.capture(TraceEvent {
            cycle: 2,
            timestamp: Instant::now(),
            source: TraceSource::MetaLayer,
            severity: TraceSeverity::Critical,
            category: "c".into(),
            summary: "crit".into(),
            detail: String::new(),
            key_values: vec![],
        });

        assert!((eng.error_rate() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_cycle_boundary() {
        let mut eng = make_engine();

        assert_eq!(eng.current_cycle(), 0);

        eng.capture_llm_call("m1", 10, 20, true, 100, None);
        eng.cycle_boundary();
        assert_eq!(eng.current_cycle(), 1);

        let events = eng.events_by_category("llm_call");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].cycle, 0);

        eng.capture_llm_call("m2", 5, 10, true, 50, None);
        let events2 = eng.events_by_category("llm_call");
        assert_eq!(events2.len(), 2);
        assert_eq!(events2[1].cycle, 1);
    }

    #[test]
    fn test_clear_resets_all() {
        let mut eng = make_engine();

        eng.capture_llm_call("gpt-4", 100, 200, true, 500, None);
        eng.capture_evolution_step("tune", 1, "completed", vec![]);
        eng.cycle_boundary();

        assert!(eng.stats().total_events > 0);
        assert_eq!(eng.current_cycle(), 1);
        assert!(eng.aggregated().len() > 0);

        eng.clear();

        assert_eq!(eng.stats().total_events, 0);
        assert_eq!(eng.current_cycle(), 0);
        assert_eq!(eng.aggregated().len(), 0);
        assert_eq!(eng.error_rate(), 0.0);
    }

    #[test]
    fn test_ring_buffer_eviction() {
        let mut eng = TraceCaptureEngine::new(5);

        for i in 0..10 {
            eng.capture(TraceEvent {
                cycle: i,
                timestamp: Instant::now(),
                source: TraceSource::External,
                severity: TraceSeverity::Debug,
                category: "evict".into(),
                summary: format!("evt {}", i),
                detail: String::new(),
                key_values: vec![],
            });
        }

        assert_eq!(eng.stats().total_events, 5);
        // Oldest retained is cycle 5
        let cats = eng.events_by_category("evict");
        assert_eq!(cats.len(), 5);
        assert_eq!(cats[0].cycle, 5);
        assert_eq!(cats[4].cycle, 9);
    }

    #[test]
    fn test_stats_structure() {
        let mut eng = make_engine();

        let s = eng.stats();
        assert_eq!(s.total_events, 0);

        eng.capture_llm_call("x", 1, 1, true, 10, None);
        eng.capture_llm_call("y", 1, 1, false, 20, Some("err"));
        eng.capture_evolution_step("s1", 1, "completed", vec![]);
        eng.capture_evolution_step("s2", 2, "failed", vec![]);
        eng.capture_calibration("c1", 0.05, 0.1, 10);
        eng.capture_calibration("c2", 0.35, 0.6, 5);

        let s2 = eng.stats();
        assert_eq!(s2.total_events, 6);
        // errors: failed llm(Error), failed evo step(Error), c2 ece=0.35(Error) = 3
        assert_eq!(s2.error_count, 3);
        assert_eq!(s2.warning_count, 0); // c1 is Info (ece=0.05 ≤ 0.15)
        assert_eq!(s2.unique_categories, 3);
        assert_eq!(s2.llm_call_count, 2);
        assert_eq!(s2.evolution_step_count, 2);
        assert_eq!(s2.calibration_event_count, 2);
        assert!((s2.error_rate - 3.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_events_by_category_filter() {
        let mut eng = make_engine();

        eng.capture_llm_call("a", 1, 1, true, 10, None);
        eng.capture_evolution_step("b", 1, "completed", vec![]);
        eng.capture_calibration("c", 0.05, 0.1, 10);

        assert_eq!(eng.events_by_category("llm_call").len(), 1);
        assert_eq!(eng.events_by_category("evolution_step").len(), 1);
        assert_eq!(eng.events_by_category("calibration").len(), 1);
        assert_eq!(eng.events_by_category("nonexistent").len(), 0);
    }

    #[test]
    fn test_decrement_aggregation_on_eviction() {
        let mut eng = TraceCaptureEngine::new(3);

        // Fill buffer with 3 events of same category+severity
        for i in 0..3 {
            eng.capture(TraceEvent {
                cycle: i,
                timestamp: Instant::now(),
                source: TraceSource::External,
                severity: TraceSeverity::Info,
                category: "same".into(),
                summary: format!("evt {}", i),
                detail: String::new(),
                key_values: vec![],
            });
        }

        let agg = eng.aggregated();
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0].count, 3);

        // Push a 4th → evicts the oldest (evt 0) → count drops to 3 (after 4th add is 3 again since max is 3)
        // Actually let's trace: [0,1,2] len=3. capture(3) → pop_front (0) → push_back(3) → [1,2,3] → count stays 3
        // Then push a 5th → evicts 1 → [2,3,4] → count stays 3
        eng.capture(TraceEvent {
            cycle: 3,
            timestamp: Instant::now(),
            source: TraceSource::External,
            severity: TraceSeverity::Info,
            category: "same".into(),
            summary: "evt 3".into(),
            detail: String::new(),
            key_values: vec![],
        });

        let agg2 = eng.aggregated();
        assert_eq!(agg2.len(), 1);
        assert_eq!(agg2[0].count, 3);

        // Now push a different category to test eviction of mixed buckets
        // [1,2,3] are 'same'/Info. Push 'other'/Info → evicts 1 → [2,3,4('other')]
        eng.capture(TraceEvent {
            cycle: 4,
            timestamp: Instant::now(),
            source: TraceSource::External,
            severity: TraceSeverity::Info,
            category: "other".into(),
            summary: "other".into(),
            detail: String::new(),
            key_values: vec![],
        });

        let agg3 = eng.aggregated();
        assert_eq!(agg3.len(), 2);
        let same_agg = agg3.iter().find(|a| a.category == "same").unwrap();
        assert_eq!(same_agg.count, 2); // [2, 3] remaining of 'same'
        let other_agg = agg3.iter().find(|a| a.category == "other").unwrap();
        assert_eq!(other_agg.count, 1);
    }
}
