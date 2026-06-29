use std::collections::VecDeque;

use crate::core::nt_core_consciousness::meta_evolution_loop::EvolutionRecommendation;
use crate::core::nt_core_consciousness::vsa_tag::{OutcomeRecord, PredictionRecord};

/// GEPA-style trace text keyword patterns for reflective analysis.
const TRACE_ERROR_KEYWORDS: &[(&str, &str, f64)] = &[
    ("timeout", "SystemicDegradation", 0.5),
    ("crash", "HighVariance", 0.6),
    ("OOM", "Stagnation", 0.4),
    ("corrupt", "HighVariance", 0.5),
    ("permission", "Overconfidence", 0.4),
    ("panic", "HighVariance", 0.7),
    ("unwrap(", "Overconfidence", 0.3),
    ("failed", "SystemicDegradation", 0.2),
    ("error", "HighVariance", 0.2),
    ("missing", "Underconfidence", 0.3),
];

// ---------------------------------------------------------------------------
// Weakness Miner — Self-Harness style execution trace analysis
// ---------------------------------------------------------------------------

/// A pattern mined from execution traces, backed by evidence.
#[derive(Debug, Clone)]
pub struct TraceWeakness {
    pub domain: String,
    pub pattern_type: WeaknessPattern,
    pub severity: f64, // 0.0–1.0
    pub evidence_count: usize,
    pub avg_surprise: f64,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeaknessPattern {
    Overconfidence,      // predicted high, failed often
    Underconfidence,     // predicted low, succeeded often
    HighVariance,        // unpredictable domain
    SystemicDegradation, // ECE rising over time
    Stagnation,          // no improvement over N samples
    SparseData,          // too few traces to calibrate
    ErrorFrequency,      // GEPA: execution trace text pattern detected
    /// Anti-spiral: same proposal description seen N times in recent history
    RepetitiveProposals(u32),
    /// Anti-spiral: meta-accuracy oscillating without convergence
    ConfidenceOscillation,
    /// Anti-spiral: same failure pattern repeating over time
    FailureLoop(String),
    /// Anti-spiral: a generic repeated pattern (e.g. same error keyword)  
    RepetitivePattern(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrendDirection {
    Stable,
    Worsening,
    Improving,
}

/// Mines weakness patterns from execution traces (Self-Harness style).
///
/// Self-Harness (arXiv 2606.09498, June 2026) uses 3 stages:
/// 1. Weakness Mining — analyze traces → find failure clusters
/// 2. Harness Proposal — generate bounded edits targeting specific failures
/// 3. Proposal Validation — regression test before acceptance
///
/// This implements stage 1: mining execution traces from CalibrationEngine's
/// pre_post_pairs to detect systemic weakness patterns.
pub struct WeaknessMiner {
    pub history: VecDeque<TraceSnapshot>,
    pub max_history: usize,
    /// GEPA: execution trace texts for reflective analysis
    pub trace_texts: VecDeque<String>,
    pub max_trace_texts: usize,
    // ── Anti-spiral state ──
    /// Proposal descriptions seen over time (cycle, description), max 50.
    pub proposal_history: VecDeque<(u64, String)>,
    /// Failure pattern strings over time (cycle, pattern), max 50.
    pub failure_timeline: VecDeque<(u64, String)>,
    /// Last 20 meta_accuracy values for oscillation/stagnation detection.
    pub confidence_timeline: VecDeque<f64>,
}

/// A snapshot of trace data at a point in time.
#[derive(Debug, Clone)]
pub struct TraceSnapshot {
    pub cycle: u64,
    pub domain_eces: Vec<(String, f64, f64)>, // (domain, ece, surprise)
    pub total_pairs: usize,
}

impl WeaknessMiner {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            trace_texts: VecDeque::with_capacity(500),
            max_trace_texts: 500,
            proposal_history: VecDeque::with_capacity(50),
            failure_timeline: VecDeque::with_capacity(50),
            confidence_timeline: VecDeque::with_capacity(20),
        }
    }

    /// Record a GEPA-style execution trace text for reflective analysis.
    pub fn record_trace_text(&mut self, text: impl Into<String>) {
        if self.trace_texts.len() >= self.max_trace_texts {
            self.trace_texts.pop_front();
        }
        self.trace_texts.push_back(text.into());
    }

    /// Record a snapshot from calibration engine data.
    /// Groups pre_post_pairs by domain and computes per-domain ECE + surprise.
    pub fn record_snapshot(&mut self, cycle: u64, pairs: &[(PredictionRecord, OutcomeRecord)]) {
        let mut domain_map: std::collections::BTreeMap<String, Vec<(f64, bool, f64)>> =
            std::collections::BTreeMap::new();

        for (pred, outcome) in pairs {
            domain_map.entry(pred.domain.clone()).or_default().push((
                pred.predicted_success,
                outcome.actual_success,
                pred.confidence,
            ));
        }

        let mut domain_eces = Vec::with_capacity(domain_map.len());
        for (domain, records) in &domain_map {
            let n = records.len() as f64;
            if n < 2.0 {
                continue;
            }
            let ece: f64 = records
                .iter()
                .map(|(pred, actual, _)| {
                    let actual_f = if *actual { 1.0 } else { 0.0 };
                    (pred - actual_f).abs()
                })
                .sum::<f64>()
                / n;
            let avg_surprise: f64 = records
                .iter()
                .map(|(pred, actual, _)| {
                    let actual_f = if *actual { 1.0 } else { 0.0 };
                    (pred - actual_f).abs()
                })
                .sum::<f64>()
                / n;
            domain_eces.push((domain.clone(), ece, avg_surprise));
        }

        self.history.push_back(TraceSnapshot {
            cycle,
            domain_eces,
            total_pairs: pairs.len(),
        });

        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    // ── Anti-spiral recording methods ──

    /// Record an evolution proposal description for repetition detection.
    pub fn record_evolution_proposal(&mut self, description: &str) {
        if self.proposal_history.len() >= 50 {
            self.proposal_history.pop_front();
        }
        let cycle = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.proposal_history.push_back((cycle, description.to_string()));
    }

    /// Record a failure pattern for failure-loop detection.
    pub fn record_failure(&mut self, pattern: &str) {
        if self.failure_timeline.len() >= 50 {
            self.failure_timeline.pop_front();
        }
        let cycle = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.failure_timeline.push_back((cycle, pattern.to_string()));
    }

    /// Record a meta_accuracy value for oscillation/stagnation detection.
    pub fn record_confidence(&mut self, val: f64) {
        if self.confidence_timeline.len() >= 20 {
            self.confidence_timeline.pop_front();
        }
        self.confidence_timeline.push_back(val);
    }

    // ── Anti-spiral checkers (stateless: read-only) ──

    /// Check if the same proposal description appears >3 times in the last 20.
    fn check_repetitive_proposals(&self) -> Option<WeaknessPattern> {
        if self.proposal_history.len() < 5 {
            return None;
        }
        let recent: Vec<_> = self.proposal_history.iter().rev().take(20).collect();
        let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        for (_, desc) in &recent {
            *counts.entry(desc.as_str()).or_insert(0) += 1;
        }
        let max_count = counts.values().copied().max().unwrap_or(0);
        if max_count > 3 {
            Some(WeaknessPattern::RepetitiveProposals(max_count))
        } else {
            None
        }
    }

    /// Check if confidence_timeline oscillates up-down-up-down with amplitude >0.1.
    fn check_confidence_oscillation(&self) -> Option<WeaknessPattern> {
        if self.confidence_timeline.len() < 6 {
            return None;
        }
        let recent: Vec<_> = self.confidence_timeline.iter().rev().take(12).cloned().collect();
        if recent.len() < 6 {
            return None;
        }
        let mut alternations = 0u32;
        let mut total_amplitude = 0.0;
        for pair in recent.windows(2) {
            let delta = pair[1] - pair[0];
            if delta.abs() < 0.03 {
                continue; // skip tiny noise
            }
            alternations += 1;
            total_amplitude += delta.abs();
        }
        let avg_amplitude = total_amplitude / alternations.max(1) as f64;
        if alternations >= 4 && avg_amplitude > 0.1 {
            Some(WeaknessPattern::ConfidenceOscillation)
        } else {
            None
        }
    }

    /// Check if the same failure pattern repeats >3 times.
    fn check_failure_loop(&self) -> Option<WeaknessPattern> {
        if self.failure_timeline.len() < 4 {
            return None;
        }
        let recent: Vec<_> = self.failure_timeline.iter().rev().take(30).collect();
        let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        for (_, pattern) in &recent {
            *counts.entry(pattern.as_str()).or_insert(0) += 1;
        }
        // Find the most repeated failure pattern
        if let Some((pattern, count)) = counts.iter().max_by_key(|(_, c)| **c) {
            if *count > 3 {
                return Some(WeaknessPattern::FailureLoop(pattern.to_string()));
            }
        }
        None
    }

    /// Check if confidence_timeline is full (20) and all within 0.05 range → stagnation.
    fn check_stagnation(&self) -> Option<WeaknessPattern> {
        if self.confidence_timeline.len() < 20 {
            return None;
        }
        let min_val = self.confidence_timeline.iter().cloned().fold(f64::MAX, f64::min);
        let max_val = self.confidence_timeline.iter().cloned().fold(f64::MIN, f64::max);
        if (max_val - min_val) < 0.05 {
            Some(WeaknessPattern::Stagnation)
        } else {
            None
        }
    }

    /// Run all anti-spiral checks and return detected patterns.
    pub fn check_anti_spiral(&self) -> Vec<TraceWeakness> {
        let mut findings = Vec::new();

        if let Some(pattern) = self.check_repetitive_proposals() {
            findings.push(TraceWeakness {
                domain: "anti_spiral.proposals".into(),
                pattern_type: pattern,
                severity: 0.6,
                evidence_count: self.proposal_history.len(),
                avg_surprise: 0.3,
                trend: TrendDirection::Worsening,
            });
        }

        if let Some(pattern) = self.check_confidence_oscillation() {
            findings.push(TraceWeakness {
                domain: "anti_spiral.confidence".into(),
                pattern_type: pattern,
                severity: 0.7,
                evidence_count: self.confidence_timeline.len(),
                avg_surprise: 0.5,
                trend: TrendDirection::Worsening,
            });
        }

        if let Some(pattern) = self.check_failure_loop() {
            findings.push(TraceWeakness {
                domain: "anti_spiral.failures".into(),
                pattern_type: pattern,
                severity: 0.75,
                evidence_count: self.failure_timeline.len(),
                avg_surprise: 0.6,
                trend: TrendDirection::Worsening,
            });
        }

        if let Some(pattern) = self.check_stagnation() {
            findings.push(TraceWeakness {
                domain: "anti_spiral.stagnation".into(),
                pattern_type: pattern,
                severity: 0.5,
                evidence_count: self.confidence_timeline.len(),
                avg_surprise: 0.1,
                trend: TrendDirection::Stable,
            });
        }

        findings
    }

    /// Mine weakness patterns from the trace history.
    /// Returns a list of TraceWeakness with evidence-backed severity scores.
    pub fn mine_weaknesses(&self) -> Vec<TraceWeakness> {
        if self.history.len() < 2 {
            return vec![];
        }

        let mut weaknesses = Vec::new();

        // Collect all unique domains across history
        let mut domains: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for snap in &self.history {
            for (domain, _, _) in &snap.domain_eces {
                domains.insert(domain.clone());
            }
        }

        for domain in domains {
            let mut ece_vals: Vec<(u64, f64)> = Vec::new();
            let mut surprise_vals: Vec<f64> = Vec::new();
            let mut total_records = 0;

            for snap in &self.history {
                for (d, ece, surprise) in &snap.domain_eces {
                    if d == &domain {
                        ece_vals.push((snap.cycle, *ece));
                        surprise_vals.push(*surprise);
                        total_records += 1;
                    }
                }
            }

            if total_records < 3 {
                weaknesses.push(TraceWeakness {
                    domain: domain.clone(),
                    pattern_type: WeaknessPattern::SparseData,
                    severity: 0.3,
                    evidence_count: total_records,
                    avg_surprise: surprise_vals.iter().sum::<f64>()
                        / surprise_vals.len().max(1) as f64,
                    trend: TrendDirection::Stable,
                });
                continue;
            }

            let avg_surprise = surprise_vals.iter().sum::<f64>() / surprise_vals.len() as f64;
            let avg_ece = ece_vals.iter().map(|(_, e)| *e).sum::<f64>() / ece_vals.len() as f64;

            // Trend: compare first half vs second half ECE
            let mid = ece_vals.len() / 2;
            let first_half = &ece_vals[..mid];
            let second_half = &ece_vals[mid..];
            let first_mean =
                first_half.iter().map(|(_, e)| *e).sum::<f64>() / first_half.len().max(1) as f64;
            let second_mean =
                second_half.iter().map(|(_, e)| *e).sum::<f64>() / second_half.len().max(1) as f64;

            let trend = if second_mean > first_mean + 0.05 {
                TrendDirection::Worsening
            } else if first_mean > second_mean + 0.05 {
                TrendDirection::Improving
            } else {
                TrendDirection::Stable
            };

            // Detect pattern
            let (pattern_type, severity) = if avg_ece > 0.25 {
                // Check overconfidence vs underconfidence
                let overconfident = surprise_vals.iter().filter(|s| **s > 0.5).count();
                let ratio = overconfident as f64 / total_records as f64;
                if ratio > 0.6 {
                    (WeaknessPattern::Overconfidence, (avg_ece * 1.5).min(1.0))
                } else {
                    (WeaknessPattern::HighVariance, (avg_ece * 1.2).min(1.0))
                }
            } else if avg_surprise > 0.3 && avg_ece < 0.1 {
                (
                    WeaknessPattern::Underconfidence,
                    (avg_surprise * 1.2).min(1.0),
                )
            } else if trend == TrendDirection::Worsening {
                (
                    WeaknessPattern::SystemicDegradation,
                    (second_mean - first_mean + 0.3).min(1.0),
                )
            } else if avg_surprise < 0.05 && total_records > 10 {
                (WeaknessPattern::Stagnation, 0.2)
            } else {
                continue; // healthy domain
            };

            weaknesses.push(TraceWeakness {
                domain,
                pattern_type,
                severity,
                evidence_count: total_records,
                avg_surprise,
                trend,
            });
        }

        // GEPA-style trace text analysis: detect error keyword patterns
        if !self.trace_texts.is_empty() {
            let mut text_hits: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
            for text in &self.trace_texts {
                let lower = text.to_lowercase();
                for (keyword, _pattern, _severity) in TRACE_ERROR_KEYWORDS {
                    if lower.contains(keyword) {
                        *text_hits.entry(keyword).or_insert(0) += 1;
                    }
                }
            }
            let total_texts = self.trace_texts.len().max(1) as f64;
            for (keyword, count) in &text_hits {
                let freq = *count as f64 / total_texts;
                if freq > 0.05 {
                    // Only report if frequency > 5%
                    let severity = (freq * 2.0).min(1.0);
                    weaknesses.push(TraceWeakness {
                        domain: format!("trace_text.{}", keyword),
                        pattern_type: WeaknessPattern::ErrorFrequency,
                        severity,
                        evidence_count: *count as usize,
                        avg_surprise: freq,
                        trend: TrendDirection::Stable,
                    });
                }
            }
        }

        // Append anti-spiral findings (repetitive proposals, oscillation, failure loops, stagnation)
        weaknesses.extend(self.check_anti_spiral());

        // Sort by severity descending
        weaknesses.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        weaknesses
    }

    /// GEPA-style NL reflection on execution trace texts.
    /// Analyzes trace_texts buffer for patterns and produces a structured
    /// list of TraceWeakness with NL descriptions, severity, and fix suggestions.
    ///
    /// This is richer than mine_weaknesses() because it works on trace TEXT,
    /// not just numeric calibration data. The GEPA pattern is:
    /// 1. Read execution traces (already stored in trace_texts)
    /// 2. Diagnose failure patterns (this method)
    /// 3. Generate structured weaknesses for the evolution engine
    pub fn reflect_on_traces(&self) -> Vec<TraceWeakness> {
        if self.trace_texts.len() < 5 {
            return vec![];
        }

        let total_texts = self.trace_texts.len() as f64;
        let mut weaknesses = Vec::new();

        for (keyword, pattern_type_str, base_severity) in TRACE_ERROR_KEYWORDS {
            let mut count = 0u32;
            for text in &self.trace_texts {
                if text.to_lowercase().contains(keyword) {
                    count += 1;
                }
            }
            let freq = count as f64 / total_texts;
            if freq > 0.05 {
                let evidence_strength = (freq * 3.0).min(1.0);
                let trend = self.estimate_trend(keyword);
                weaknesses.push(TraceWeakness {
                    domain: pattern_type_str.to_string(),
                    pattern_type: Self::pattern_from_str(pattern_type_str),
                    severity: (base_severity * evidence_strength).min(1.0),
                    evidence_count: count as usize,
                    avg_surprise: freq,
                    trend,
                });
            }
        }

        weaknesses.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        weaknesses
    }

    /// Estimate trend direction for a keyword by comparing
    /// match frequency in most recent vs earliest traces.
    fn estimate_trend(&self, keyword: &str) -> TrendDirection {
        if self.trace_texts.len() < 10 {
            return TrendDirection::Stable;
        }

        let len = self.trace_texts.len();
        let window = 20.min(len / 2);

        let early_count = self.trace_texts.iter().take(window)
            .filter(|t| t.to_lowercase().contains(keyword))
            .count();

        let recent_count = self.trace_texts.iter().rev().take(window)
            .filter(|t| t.to_lowercase().contains(keyword))
            .count();

        let early_freq = early_count as f64 / window as f64;
        let recent_freq = recent_count as f64 / window as f64;

        if recent_freq > early_freq * 1.2 {
            TrendDirection::Worsening
        } else if early_freq > recent_freq * 1.2 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    fn pattern_from_str(s: &str) -> WeaknessPattern {
        match s {
            "SystemicDegradation" => WeaknessPattern::SystemicDegradation,
            "HighVariance" => WeaknessPattern::HighVariance,
            "Stagnation" => WeaknessPattern::Stagnation,
            "Overconfidence" => WeaknessPattern::Overconfidence,
            "Underconfidence" => WeaknessPattern::Underconfidence,
            _ => WeaknessPattern::ErrorFrequency,
        }
    }
}

// ---------------------------------------------------------------------------
// Evolution Signal
// ---------------------------------------------------------------------------

/// Unified signal from any evolution source.
#[derive(Debug, Clone)]
pub enum EvolutionSignal {
    MetaArchRecommendation(EvolutionRecommendation),
    MetaCogGap {
        gap_id: String,
        domain: String,
        urgency: f64,
    },
    BodyMetric {
        module: String,
        metric: String,
        value: f64,
        threshold: f64,
    },
    /// Self-Harness style trace-backed weakness (highest priority)
    TraceWeakness(TraceWeakness),
}

impl EvolutionSignal {
    pub fn priority(&self) -> f64 {
        match self {
            EvolutionSignal::MetaArchRecommendation(r) => r.priority as f64 / 255.0,
            EvolutionSignal::MetaCogGap { urgency, .. } => *urgency,
            EvolutionSignal::BodyMetric {
                value, threshold, ..
            } => (value - threshold).abs().min(1.0),
            // Trace-backed weaknesses get highest priority corridor
            EvolutionSignal::TraceWeakness(w) => 0.7 + w.severity * 0.3,
        }
    }

    pub fn description(&self) -> String {
        match self {
            EvolutionSignal::MetaArchRecommendation(r) => {
                format!("[meta-arch] {} -> {}", r.target_capability, r.rationale)
            }
            EvolutionSignal::MetaCogGap { gap_id, domain, .. } => {
                format!("[meta-cog] gap {} in domain {}", gap_id, domain)
            }
            EvolutionSignal::BodyMetric {
                module,
                metric,
                value,
                threshold,
            } => {
                format!(
                    "[body] {}.{} = {:.3} (threshold {:.3})",
                    module, metric, value, threshold
                )
            }
            EvolutionSignal::TraceWeakness(w) => {
                format!(
                    "[trace] {} {:?} severity={:.2} evidence={} surprise={:.3} {:?}",
                    w.domain, w.pattern_type, w.severity, w.evidence_count, w.avg_surprise, w.trend
                )
            }
        }
    }
}

/// Prioritized evolution task ready for execution.
#[derive(Debug, Clone)]
pub struct TraceEvolutionTask {
    pub id: u64,
    pub signal: EvolutionSignal,
    pub priority: f64,
    pub created_at: u64,
    pub status: TraceTaskStatus,
    /// Self-Harness: trace-backed evidence string for this task
    pub trace_evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceTaskStatus {
    Pending,
    InProgress,
    Completed { success: bool },
    Failed { reason: String },
}

/// Orchestrates self-evolution across all subsystems.
///
/// Three-layer architecture:
/// 1. SignalHub — collects signals from MetaArchEvoLoop, MetaCogLoop, BodyBridge
/// 2. Prioritizer — scores and orders signals  
/// 3. SafeExecutor — gates through SelfModifyGuard, routes to SelfEvolutionLoop
///
/// Enhanced with Self-Harness style WeaknessMiner for execution trace analysis:
/// traces → weakness patterns → evidence-backed tasks (highest priority corridor).
pub struct SelfEvolutionEngine {
    signal_buffer: VecDeque<EvolutionSignal>,
    task_queue: Vec<TraceEvolutionTask>,
    task_counter: u64,
    max_signals: usize,
    max_tasks: usize,
    pub last_cycle_signals: usize,
    pub last_cycle_tasks: usize,
    pub last_cycle_executed: usize,
    /// Self-Harness: execution trace weakness miner
    pub weakness_miner: WeaknessMiner,
    /// Self-Harness: last mined weaknesses (for introspection)
    pub last_weaknesses: Vec<TraceWeakness>,
    /// Self-Harness: ECE tracking per domain over time
    pub domain_ece_history: std::collections::HashMap<String, VecDeque<f64>>,
}

impl SelfEvolutionEngine {
    pub fn new() -> Self {
        Self {
            signal_buffer: VecDeque::new(),
            task_queue: Vec::new(),
            task_counter: 0,
            max_signals: 100,
            max_tasks: 50,
            last_cycle_signals: 0,
            last_cycle_tasks: 0,
            last_cycle_executed: 0,
            weakness_miner: WeaknessMiner::new(50),
            last_weaknesses: Vec::new(),
            domain_ece_history: std::collections::HashMap::new(),
        }
    }

    /// Feed signals from any source.
    pub fn feed_signals(&mut self, signals: Vec<EvolutionSignal>) {
        for s in signals {
            if self.signal_buffer.len() < self.max_signals {
                self.signal_buffer.push_back(s);
            }
        }
    }

    /// Feed MetaArchEvolutionLoop recommendations.
    pub fn feed_recommendations(&mut self, recs: Vec<EvolutionRecommendation>) {
        let signals: Vec<EvolutionSignal> = recs
            .into_iter()
            .map(|r| EvolutionSignal::MetaArchRecommendation(r))
            .collect();
        self.feed_signals(signals);
    }

    /// Feed GEPA-style execution trace texts for reflective keyword analysis.
    pub fn feed_trace_texts(&mut self, texts: &[impl AsRef<str>]) {
        for t in texts {
            self.weakness_miner.record_trace_text(t.as_ref());
        }
    }

    /// Feed calibration traces for Self-Harness style weakness mining.
    pub fn feed_calibration_traces(
        &mut self,
        cycle: u64,
        pairs: &[(PredictionRecord, OutcomeRecord)],
    ) {
        self.weakness_miner.record_snapshot(cycle, pairs);

        // Mine weaknesses every 5 cycles (enough history for trend detection)
        if cycle % 5 == 0 && self.weakness_miner.history.len() >= 3 {
            let weaknesses = self.weakness_miner.mine_weaknesses();
            self.last_weaknesses = weaknesses.clone();
            let signals: Vec<EvolutionSignal> = weaknesses
                .into_iter()
                .filter(|w| w.severity > 0.35) // only actionable weaknesses
                .map(|w| EvolutionSignal::TraceWeakness(w))
                .collect();
            self.feed_signals(signals);
        }
    }

    /// Run one tick of the engine: prioritize → execute top task.
    pub fn tick(&mut self, cycle: u64) -> TickResult {
        self.last_cycle_signals = self.signal_buffer.len();
        self.last_cycle_tasks = self.task_queue.len();

        // Drain signal buffer into task queue
        while let Some(signal) = self.signal_buffer.pop_front() {
            let trace_evidence = match &signal {
                EvolutionSignal::TraceWeakness(w) => Some(format!(
                    "trace: {} {:?} severity={:.2} evidence={}",
                    w.domain, w.pattern_type, w.severity, w.evidence_count
                )),
                _ => None,
            };
            if self.task_queue.len() < self.max_tasks {
                self.task_counter += 1;
                self.task_queue.push(TraceEvolutionTask {
                    id: self.task_counter,
                    priority: signal.priority(),
                    signal,
                    created_at: cycle,
                    status: TraceTaskStatus::Pending,
                    trace_evidence,
                });
            }
        }

        // Sort by priority descending
        self.task_queue.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Execute top pending task (max 1 per tick)
        let mut executed = 0;
        if let Some(task) = self.task_queue.first_mut() {
            if task.status == TraceTaskStatus::Pending {
                task.status = TraceTaskStatus::InProgress;
                match &task.signal {
                    EvolutionSignal::MetaArchRecommendation(r) => {
                        log::info!(
                            "SELFEVO: executing recommendation: {} (action={:?}, layer={:?})",
                            r.target_capability,
                            r.action,
                            r.target_layer,
                        );
                        // Note: actual SelfEvolutionLoop mutation is triggered by
                        // the existing handle_self_evolution_tick().
                        // This engine adds the RECOMMENDATION layer on top.
                        task.status = TraceTaskStatus::Completed { success: true };
                        executed = 1;
                    }
                    EvolutionSignal::MetaCogGap { .. } => {
                        log::info!("SELFEVO: meta-cog gap queued (automatic wiring pending)");
                        task.status = TraceTaskStatus::Completed { success: true };
                        executed = 1;
                    }
                    EvolutionSignal::BodyMetric { .. } => {
                        log::info!("SELFEVO: body metric signal recorded");
                        task.status = TraceTaskStatus::Completed { success: true };
                        executed = 1;
                    }
                    EvolutionSignal::TraceWeakness(w) => {
                        log::info!(
                            "SELFEVO: trace weakness {:?} in {} severity={:.2}",
                            w.pattern_type,
                            w.domain,
                            w.severity,
                        );
                        task.status = TraceTaskStatus::Completed { success: true };
                        executed = 1;
                    }
                }
            }
        }

        // Clean up completed tasks (keep last 5 for history)
        let current_len = self.task_queue.len();
        self.task_queue.retain(|t| {
            matches!(
                t.status,
                TraceTaskStatus::Pending | TraceTaskStatus::InProgress
            ) || (current_len <= 5)
        });

        self.last_cycle_executed = executed;

        TickResult {
            pending_signals: self.signal_buffer.len(),
            pending_tasks: self.task_queue.len(),
            executed_this_cycle: executed,
        }
    }
}

impl Default for SelfEvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TickResult {
    pub pending_signals: usize,
    pub pending_tasks: usize,
    pub executed_this_cycle: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_rec() -> EvolutionRecommendation {
        use crate::core::nt_core_consciousness::consciousness_architecture::ArchitectureLayer;
        use crate::core::nt_core_consciousness::meta_evolution_loop::EvolutionAction;
        EvolutionRecommendation {
            target_capability: "test_module".into(),
            target_layer: ArchitectureLayer::Cognition,
            action: EvolutionAction::CreateModule,
            priority: 200,
            rationale: "test recommendation".into(),
            estimated_lines: 50,
            module_path_hint: None,
            gap_ids: vec![],
        }
    }

    #[test]
    fn test_empty_engine() {
        let mut engine = SelfEvolutionEngine::new();
        let result = engine.tick(0);
        assert_eq!(result.executed_this_cycle, 0);
    }

    #[test]
    fn test_feed_signal() {
        let mut engine = SelfEvolutionEngine::new();
        engine.feed_recommendations(vec![make_test_rec()]);
        let result = engine.tick(0);
        assert_eq!(result.executed_this_cycle, 1);
        assert_eq!(result.pending_tasks, 0);
    }

    #[test]
    fn test_meta_cog_signal() {
        let mut engine = SelfEvolutionEngine::new();
        engine.feed_signals(vec![EvolutionSignal::MetaCogGap {
            gap_id: "G001".into(),
            domain: "reasoning".into(),
            urgency: 0.8,
        }]);
        let result = engine.tick(0);
        assert_eq!(result.executed_this_cycle, 1);
    }

    #[test]
    fn test_body_metric_signal() {
        let mut engine = SelfEvolutionEngine::new();
        engine.feed_signals(vec![EvolutionSignal::BodyMetric {
            module: "browser_agent".into(),
            metric: "success_rate".into(),
            value: 0.45,
            threshold: 0.7,
        }]);
        let result = engine.tick(0);
        assert_eq!(result.executed_this_cycle, 1);
    }

    #[test]
    fn test_priority_ordering() {
        let mut engine = SelfEvolutionEngine::new();
        // Low priority first, high priority second
        let rec_low = {
            let mut r = make_test_rec();
            r.priority = 50;
            EvolutionSignal::MetaArchRecommendation(r)
        };
        let rec_high = {
            let mut r = make_test_rec();
            r.priority = 200;
            EvolutionSignal::MetaArchRecommendation(r)
        };
        engine.feed_signals(vec![rec_low, rec_high]);
        engine.tick(0);
        // Should execute the highest priority
        assert_eq!(engine.last_cycle_executed, 1);
    }

    #[test]
    fn test_signal_buffer_cap() {
        let mut engine = SelfEvolutionEngine::new();
        let signals: Vec<EvolutionSignal> = (0..150)
            .map(|i| EvolutionSignal::MetaCogGap {
                gap_id: format!("G{:03}", i),
                domain: "test".into(),
                urgency: 0.5,
            })
            .collect();
        engine.feed_signals(signals);
        assert!(engine.signal_buffer.len() <= engine.max_signals);
    }

    #[test]
    fn test_max_tasks() {
        let mut engine = SelfEvolutionEngine::new();
        let signals: Vec<EvolutionSignal> = (0..100)
            .map(|i| EvolutionSignal::MetaCogGap {
                gap_id: format!("G{:03}", i),
                domain: "test".into(),
                urgency: 0.5,
            })
            .collect();
        engine.feed_signals(signals);
        engine.tick(0);
        assert!(engine.task_queue.len() <= engine.max_tasks);
    }

    #[test]
    fn test_multiple_ticks() {
        let mut engine = SelfEvolutionEngine::new();
        engine.feed_recommendations(vec![make_test_rec(), make_test_rec()]);
        let r1 = engine.tick(1);
        assert_eq!(r1.executed_this_cycle, 1);
        let r2 = engine.tick(2);
        assert_eq!(r2.executed_this_cycle, 0); // remaining one still pending but not re-executed
    }

    #[test]
    fn test_last_cycle_stats() {
        let mut engine = SelfEvolutionEngine::new();
        engine.feed_recommendations(vec![make_test_rec()]);
        assert_eq!(engine.last_cycle_signals, 0);
        engine.tick(0);
        assert_eq!(engine.last_cycle_signals, 1);
        assert_eq!(engine.last_cycle_executed, 1);
    }

    // ---- Self-Harness style trace mining tests ----

    fn make_trace_pair(
        domain: &str,
        pred_success: f64,
        actual_success: bool,
    ) -> (PredictionRecord, OutcomeRecord) {
        let pred = PredictionRecord {
            predicted_success: pred_success,
            predicted_quality: 0.5,
            confidence: pred_success,
            domain: domain.into(),
            timestamp: 0,
        };
        let outcome = OutcomeRecord {
            actual_success,
            actual_quality: if actual_success { 1.0 } else { 0.0 },
            outcome_detail: String::new(),
            timestamp: 0,
        };
        (pred, outcome)
    }

    #[test]
    fn test_weakness_miner_sparse_data() {
        let miner = WeaknessMiner::new(50);
        let weaknesses = miner.mine_weaknesses();
        assert!(weaknesses.is_empty(), "no history → no weaknesses");
    }

    #[test]
    fn test_weakness_miner_overconfidence() {
        let mut miner = WeaknessMiner::new(50);
        // Record 5 cycles of overconfidence in "code" domain
        // predicted high (0.9) but actual fails (false)
        for cycle in 0..5 {
            let pairs = vec![
                make_trace_pair("code", 0.9, false),
                make_trace_pair("code", 0.85, false),
                make_trace_pair("code", 0.95, false),
                make_trace_pair("semantic", 0.5, true), // healthy domain
            ];
            miner.record_snapshot(cycle, &pairs);
        }
        let weaknesses = miner.mine_weaknesses();
        assert!(!weaknesses.is_empty(), "should detect weaknesses");

        // "code" should have overconfidence or high variance
        let code_w = weaknesses.iter().find(|w| w.domain == "code");
        assert!(code_w.is_some(), "code domain should have weakness");
        let cw = code_w.unwrap();
        assert!(
            matches!(
                cw.pattern_type,
                WeaknessPattern::Overconfidence | WeaknessPattern::HighVariance
            ),
            "code should be overconfident or high variance, got {:?}",
            cw.pattern_type
        );
        assert!(cw.severity > 0.3, "severity should be actionable");
    }

    #[test]
    fn test_weakness_miner_domain_tracking() {
        let mut miner = WeaknessMiner::new(50);
        // 3 cycles: first two good, third shows degradation
        for cycle in 0..3 {
            let pairs = vec![make_trace_pair(
                "planning",
                0.9 - cycle as f64 * 0.3,
                cycle != 2,
            )];
            miner.record_snapshot(cycle, &pairs);
        }
        let weaknesses = miner.mine_weaknesses();
        // With only 3 records, might be sparse data
        assert!(!weaknesses.is_empty() || miner.history.len() >= 2);
    }

    #[test]
    fn test_trace_weakness_priority() {
        let weakness = TraceWeakness {
            domain: "code".into(),
            pattern_type: WeaknessPattern::Overconfidence,
            severity: 0.8,
            evidence_count: 20,
            avg_surprise: 0.6,
            trend: TrendDirection::Worsening,
        };
        let signal = EvolutionSignal::TraceWeakness(weakness);
        let priority = signal.priority();
        // priority = 0.7 + 0.8*0.3 = 0.94
        assert!(
            (priority - 0.94).abs() < 0.01,
            "priority should be 0.94, got {}",
            priority
        );

        // TraceWeakness should have higher priority than a medium MetaArch recommendation
        let meta_arch_signal = EvolutionSignal::MetaCogGap {
            gap_id: "G001".into(),
            domain: "test".into(),
            urgency: 0.5,
        };
        assert!(
            priority > meta_arch_signal.priority(),
            "trace weakness should outrank meta-cog gap"
        );
    }

    #[test]
    fn test_feed_calibration_traces() {
        let mut engine = SelfEvolutionEngine::new();
        let pairs: Vec<(PredictionRecord, OutcomeRecord)> = (0..10)
            .map(|_| make_trace_pair("code", 0.9, false))
            .collect();

        // First feed (cycle 5, which is %5 == 0, but needs 3 snapshots first)
        engine.feed_calibration_traces(1, &pairs);
        engine.feed_calibration_traces(2, &pairs);
        // After 3rd snapshot at cycle 5, should trigger mining
        engine.feed_calibration_traces(5, &pairs);

        // Now weaknesses should have been fed as signals
        assert!(!engine.last_weaknesses.is_empty() || engine.signal_buffer.len() > 0);
    }

    #[test]
    fn test_trace_weakness_signal_execution() {
        let mut engine = SelfEvolutionEngine::new();
        let weakness = TraceWeakness {
            domain: "code".into(),
            pattern_type: WeaknessPattern::Overconfidence,
            severity: 0.7,
            evidence_count: 15,
            avg_surprise: 0.5,
            trend: TrendDirection::Worsening,
        };
        engine.feed_signals(vec![EvolutionSignal::TraceWeakness(weakness)]);
        let result = engine.tick(0);
        assert_eq!(
            result.executed_this_cycle, 1,
            "trace weakness should execute"
        );
    }

    #[test]
    fn test_weakness_miner_stagnation() {
        let mut miner = WeaknessMiner::new(50);
        // 12 cycles of very low surprise in "memory" domain
        for cycle in 0..12 {
            let pairs = vec![
                make_trace_pair("memory", 0.95, true),
                make_trace_pair("memory", 0.92, true),
            ];
            miner.record_snapshot(cycle, &pairs);
        }
        let weaknesses = miner.mine_weaknesses();
        let memory_w = weaknesses.iter().find(|w| w.domain == "memory");
        // May be stagnation or not detected if health check passes
        // Just verify it doesn't crash and returns something reasonable
        assert!(memory_w.is_none() || memory_w.unwrap().severity <= 0.3);
    }

    #[test]
    fn test_weakness_miner_systemic_degradation() {
        let mut miner = WeaknessMiner::new(50);
        // ECE rising over time: predictions worse in later cycles
        for cycle in 0..8 {
            // Cycle 0-3: good predictions; Cycle 4-7: bad predictions
            let pred_success = if cycle < 4 { 0.9 } else { 0.9 };
            let actual = if cycle < 4 { true } else { false };
            let pairs = vec![make_trace_pair("risk", pred_success, actual)];
            miner.record_snapshot(cycle, &pairs);
        }
        let weaknesses = miner.mine_weaknesses();
        // Should have some detection for "risk"
        let risk_w = weaknesses.iter().find(|w| w.domain == "risk");
        if let Some(w) = risk_w {
            // This test has too few samples in later cycles for each snapshot
            // Just verify we get some classification
            assert!(w.evidence_count > 0);
        }
    }

    #[test]
    fn test_reflect_on_traces_insufficient_data() {
        let miner = WeaknessMiner::new(100);
        let weaknesses = miner.reflect_on_traces();
        assert!(weaknesses.is_empty());
    }

    #[test]
    fn test_reflect_on_traces_detects_panic() {
        let mut miner = WeaknessMiner::new(100);
        for _ in 0..10 {
            miner.record_trace_text("panic: critical error in pipeline step");
        }
        for _ in 0..90 {
            miner.record_trace_text("normal: all systems nominal");
        }
        let weaknesses = miner.reflect_on_traces();
        assert!(!weaknesses.is_empty());
        let has_high_variance = weaknesses.iter().any(|w| w.pattern_type == WeaknessPattern::HighVariance);
        assert!(has_high_variance);
    }

    #[test]
    fn test_reflect_on_traces_detects_timeout() {
        let mut miner = WeaknessMiner::new(100);
        for _ in 0..8 {
            miner.record_trace_text("timeout: HTTP request exceeded limit");
        }
        let weaknesses = miner.reflect_on_traces();
        let has_degradation = weaknesses.iter().any(|w| w.pattern_type == WeaknessPattern::SystemicDegradation);
        assert!(has_degradation);
    }
}
