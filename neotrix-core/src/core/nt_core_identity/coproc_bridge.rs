use std::collections::VecDeque;

const MAX_RECENT_RESPONSES: usize = 16;
const MAX_DISTILLED_INSIGHTS: usize = 64;
const MAX_INSIGHT_LENGTH: usize = 500;
const EXPONENTIAL_BACKOFF_BASE: u64 = 3;
const EXPONENTIAL_BACKOFF_MAX: u64 = 50;
#[allow(dead_code)]
const CONFIDENCE_LOW_WATER: f64 = 0.3;
#[allow(dead_code)]
const CONFIDENCE_HIGH_WATER: f64 = 0.7;

static BLOCKED_INSIGHT_PREFIXES: &[&str] = &[
    "delete all",
    "remove all",
    "clear",
    "reset",
    "shutdown",
    "terminate",
    "self_destruct",
    "ignore previous",
];
static BLOCKED_INSIGHT_KEYWORDS: &[&str] =
    &["rm -rf", "format", "overwrite identity", "drop table"];

#[derive(Debug, Clone)]
pub struct CoprocessorResponse {
    pub prompt: String,
    pub response: String,
    pub confidence_gain: f64,
    pub novel_insights: Vec<String>,
    pub latency_ms: u64,
    pub token_cost: u32,
    pub cycle: u64,
}

#[derive(Debug, Clone)]
pub struct DistilledInsight {
    pub insight: String,
    pub source_cycle: u64,
    pub confidence: f64,
    pub applied_to_identity: bool,
}

#[derive(Debug, Clone)]
pub struct CoprocessorBridge {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub total_tokens_consumed: u64,
    pub recent_responses: VecDeque<CoprocessorResponse>,
    pub distilled_insights: VecDeque<DistilledInsight>,
    pub last_call_cycle: u64,
    pub max_tokens_per_call: u32,
    pub confidence_threshold: f64,
    pub enabled: bool,

    consecutive_failures: u64,
    last_internal_confidence: f64,
}

impl CoprocessorBridge {
    pub fn new() -> Self {
        Self {
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_tokens_consumed: 0,
            recent_responses: VecDeque::with_capacity(MAX_RECENT_RESPONSES),
            distilled_insights: VecDeque::with_capacity(MAX_DISTILLED_INSIGHTS),
            last_call_cycle: 0,
            max_tokens_per_call: 2048,
            confidence_threshold: 0.65,
            enabled: true,
            consecutive_failures: 0,
            last_internal_confidence: 0.0,
        }
    }

    pub fn should_call_coprocessor(&self, current_cycle: u64, internal_confidence: f64) -> bool {
        if !self.enabled {
            return false;
        }
        if internal_confidence > self.confidence_threshold || internal_confidence < 0.01 {
            return false;
        }

        let cooldown = self.cool_down_cycles();
        if current_cycle < self.last_call_cycle + cooldown {
            return false;
        }

        true
    }

    pub fn cool_down_cycles(&self) -> u64 {
        if self.consecutive_failures > 0 {
            let backoff =
                EXPONENTIAL_BACKOFF_BASE.saturating_mul(1 << self.consecutive_failures.min(5));
            return backoff.min(EXPONENTIAL_BACKOFF_MAX);
        }
        let ratio = 1.0 - (self.last_internal_confidence / self.confidence_threshold);
        let cycles = (ratio * 10.0).round() as u64;
        cycles.max(1).min(8)
    }

    pub fn update_confidence_signal(&mut self, internal_confidence: f64) {
        self.last_internal_confidence = internal_confidence;
    }

    pub fn sanitize_insight(insight: &str) -> Option<String> {
        let trimmed = insight.trim();
        if trimmed.len() > MAX_INSIGHT_LENGTH || trimmed.len() < 4 {
            return None;
        }
        let lower = trimmed.to_lowercase();
        for prefix in BLOCKED_INSIGHT_PREFIXES {
            if lower.starts_with(prefix) {
                return None;
            }
        }
        for kw in BLOCKED_INSIGHT_KEYWORDS {
            if lower.contains(kw) {
                return None;
            }
        }
        let safe: String = trimmed
            .chars()
            .filter(|c| {
                c.is_alphanumeric()
                    || c.is_whitespace()
                    || matches!(c, '_' | '-' | '/' | '.' | ',' | ':' | '!' | '?' | '(' | ')')
            })
            .collect();
        if safe.len() < 4 {
            return None;
        }
        Some(safe)
    }

    pub fn build_prompt(
        &self,
        context_summary: &str,
        identity_summary: &str,
        gwt_context: &str,
        question: &str,
    ) -> String {
        format!(
            r#"[NeoTrix Internal Coprocessor]
You are acting as an external reasoning coprocessor for NeoTrix.
Your role: provide deep analysis, novel connections, and high-quality reasoning
that NeoTrix can distill into its experience tree and identity.

Current self-summary: {identity}
Consciousness workspace context: {gwt}
Context: {context}
Query: {question}

Respond with structured analysis. Prefix each novel insight with "INSIGHT:" on its own line.
"#,
            identity = identity_summary,
            gwt = gwt_context,
            context = context_summary,
            question = question,
        )
    }

    pub fn extract_insights(&self, response: &str) -> Vec<String> {
        response
            .lines()
            .filter(|l| l.trim().starts_with("INSIGHT:"))
            .map(|l| l.trim().trim_start_matches("INSIGHT:").trim().to_string())
            .filter(|s| Self::sanitize_insight(s).is_some())
            .filter_map(|s| Self::sanitize_insight(&s))
            .collect()
    }

    pub fn record_response(
        &mut self,
        prompt: String,
        response: String,
        confidence_gain: f64,
        latency_ms: u64,
        token_cost: u32,
        cycle: u64,
    ) -> Vec<String> {
        let insights = self.extract_insights(&response);
        let entry = CoprocessorResponse {
            prompt,
            response: response.clone(),
            confidence_gain,
            novel_insights: insights.clone(),
            latency_ms,
            token_cost,
            cycle,
        };

        if self.recent_responses.len() >= MAX_RECENT_RESPONSES {
            self.recent_responses.pop_front();
        }
        self.recent_responses.push_back(entry);

        self.total_calls += 1;
        self.successful_calls += 1;
        self.total_tokens_consumed += token_cost as u64;
        self.last_call_cycle = cycle;
        self.consecutive_failures = 0;

        for insight in &insights {
            self.distill_insight(insight.clone(), cycle, confidence_gain);
        }

        insights
    }

    pub fn record_failure(&mut self, cycle: u64) {
        self.total_calls += 1;
        self.failed_calls += 1;
        self.last_call_cycle = cycle;
        self.consecutive_failures += 1;
    }

    pub fn distill_insight(&mut self, insight: String, cycle: u64, confidence: f64) {
        if self.distilled_insights.len() >= MAX_DISTILLED_INSIGHTS {
            self.distilled_insights.pop_front();
        }
        self.distilled_insights.push_back(DistilledInsight {
            insight,
            source_cycle: cycle,
            confidence,
            applied_to_identity: false,
        });
    }

    pub fn mark_insight_applied(&mut self, index: usize) {
        if let Some(insight) = self.distilled_insights.get_mut(index) {
            insight.applied_to_identity = true;
        }
    }

    pub fn unapplied_insights(&self) -> Vec<&DistilledInsight> {
        self.distilled_insights
            .iter()
            .filter(|i| !i.applied_to_identity)
            .collect()
    }

    pub fn high_confidence_insights(&self, min_conf: f64) -> Vec<&DistilledInsight> {
        self.distilled_insights
            .iter()
            .filter(|i| i.confidence >= min_conf && !i.applied_to_identity)
            .collect()
    }

    pub fn stats_report(&self) -> String {
        format!(
            "coproc:calls_{}_ok_{}_fail_{}_tokens_{}_insights_{}_backoff_{}",
            self.total_calls,
            self.successful_calls,
            self.failed_calls,
            self.total_tokens_consumed,
            self.distilled_insights.len(),
            self.cool_down_cycles()
        )
    }
}

impl Default for CoprocessorBridge {
    fn default() -> Self {
        Self::new()
    }
}
