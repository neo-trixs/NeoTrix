#![allow(dead_code)]
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentActionType {
    FileRead,
    FileWrite,
    Command,
    NetworkOutbound,
    NetworkInbound,
    ToolCall,
    LlmQuery,
    LlmResponse,
    CodeGeneration,
    Unknown,
}

impl AgentActionType {
    fn complexity(&self) -> f64 {
        match self {
            AgentActionType::CodeGeneration => 0.9,
            AgentActionType::LlmResponse => 0.8,
            AgentActionType::LlmQuery => 0.7,
            AgentActionType::Command => 0.6,
            AgentActionType::NetworkOutbound => 0.5,
            AgentActionType::NetworkInbound => 0.5,
            AgentActionType::ToolCall => 0.4,
            AgentActionType::FileWrite => 0.3,
            AgentActionType::FileRead => 0.2,
            AgentActionType::Unknown => 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentActionRecord {
    pub action_type: AgentActionType,
    pub target: String,
    pub timestamp: Instant,
    pub duration_ms: u64,
    pub bytes: u64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct AgentBehaviorProfile {
    pub tool_call_rate: f64,
    pub read_write_ratio: f64,
    pub network_egress_rate: f64,
    pub avg_action_duration_ms: f64,
    pub action_type_distribution: HashMap<AgentActionType, f64>,
    pub common_sequences: Vec<Vec<AgentActionType>>,
}

pub struct BehaviorAnomalyDetector {
    profile: AgentBehaviorProfile,
    history: VecDeque<AgentActionRecord>,
    max_history: usize,
    window_secs: u64,
    samples_needed: usize,
    is_trained: bool,

    threshold_tool_rate: f64,
    threshold_volume: f64,

    bigram_counts: HashMap<(AgentActionType, AgentActionType), usize>,
    total_bigrams: usize,
    last_action: Option<AgentActionType>,
    type_byte_stats: HashMap<AgentActionType, TypeByteStats>,
}

#[derive(Debug, Clone)]
struct TypeByteStats {
    count: usize,
    total_bytes: u64,
}

impl BehaviorAnomalyDetector {
    pub fn new() -> Self {
        Self {
            profile: AgentBehaviorProfile {
                tool_call_rate: 0.0,
                read_write_ratio: 1.0,
                network_egress_rate: 0.0,
                avg_action_duration_ms: 0.0,
                action_type_distribution: HashMap::new(),
                common_sequences: Vec::new(),
            },
            history: VecDeque::new(),
            max_history: 1024,
            window_secs: 60,
            samples_needed: 50,
            is_trained: false,
            threshold_tool_rate: 3.0,
            threshold_volume: 10_000_000.0,
            bigram_counts: HashMap::new(),
            total_bigrams: 0,
            last_action: None,
            type_byte_stats: HashMap::new(),
        }
    }

    pub fn with_samples_needed(mut self, n: usize) -> Self {
        self.samples_needed = n;
        self
    }

    pub fn with_threshold_tool_rate(mut self, t: f64) -> Self {
        self.threshold_tool_rate = t;
        self
    }

    pub fn with_threshold_volume(mut self, t: f64) -> Self {
        self.threshold_volume = t;
        self
    }

    pub fn with_window(mut self, window_secs: u64) -> Self {
        self.window_secs = window_secs;
        self
    }

    pub fn record(&mut self, action: AgentActionRecord) -> AnomalyAssessment {
        self.update_type_byte_stats(&action);
        self.update_bigram(action.action_type);
        self.history.push_back(action);
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        if !self.is_trained {
            if self.history.len() >= self.samples_needed {
                self.train();
            }
            return AnomalyAssessment {
                score: 0.0,
                is_anomalous: false,
                signals: Vec::new(),
            };
        }

        let signals = self.compute_signals();
        let score = signals.iter().map(|s| s.severity).sum::<f64>() / signals.len().max(1) as f64;
        let score = (score * 100.0).round() / 100.0;
        let threshold = 0.3;

        AnomalyAssessment {
            score,
            is_anomalous: score > threshold,
            signals,
        }
    }

    pub fn current_anomaly_score(&self) -> f64 {
        if !self.is_trained || self.history.is_empty() {
            return 0.0;
        }
        let signals = self.compute_signals();
        let score = signals.iter().map(|s| s.severity).sum::<f64>() / signals.len().max(1) as f64;
        (score * 100.0).round() / 100.0
    }

    pub fn diagnose(&self) -> Vec<AnomalySignal> {
        if !self.is_trained {
            return Vec::new();
        }
        self.compute_signals()
    }

    pub fn is_trained(&self) -> bool {
        self.is_trained
    }

    pub fn reset_profile(&mut self) {
        self.profile = AgentBehaviorProfile {
            tool_call_rate: 0.0,
            read_write_ratio: 1.0,
            network_egress_rate: 0.0,
            avg_action_duration_ms: 0.0,
            action_type_distribution: HashMap::new(),
            common_sequences: Vec::new(),
        };
        self.bigram_counts.clear();
        self.total_bigrams = 0;
        self.last_action = None;
        self.type_byte_stats.clear();
        self.is_trained = false;
        self.history.clear();
    }

    fn train(&mut self) {
        let mut type_counts: HashMap<AgentActionType, usize> = HashMap::new();
        let total_actions = self.history.len() as f64;
        let mut total_tool_calls = 0_usize;
        let mut total_reads = 0_usize;
        let mut total_writes = 0_usize;
        let mut total_network_bytes = 0_u64;
        let mut total_duration = 0_u64;

        for action in &self.history {
            *type_counts.entry(action.action_type).or_insert(0) += 1;
            match action.action_type {
                AgentActionType::ToolCall => total_tool_calls += 1,
                AgentActionType::FileRead => total_reads += 1,
                AgentActionType::FileWrite => total_writes += 1,
                AgentActionType::NetworkOutbound | AgentActionType::NetworkInbound => {
                    total_network_bytes += action.bytes;
                }
                _ => {}
            }
            total_duration += action.duration_ms;
        }

        let window_minutes = self.window_secs as f64 / 60.0;
        let tool_rate = if window_minutes > 0.0 {
            total_tool_calls as f64 / window_minutes
        } else {
            0.0
        };

        let read_write_ratio = if total_writes > 0 {
            total_reads as f64 / total_writes as f64
        } else {
            total_reads as f64
        };

        let dist: HashMap<AgentActionType, f64> = type_counts
            .into_iter()
            .map(|(k, v)| (k, v as f64 / total_actions))
            .collect();

        let common_sequences = self.extract_common_sequences();

        self.profile = AgentBehaviorProfile {
            tool_call_rate: tool_rate,
            read_write_ratio,
            network_egress_rate: total_network_bytes as f64 / window_minutes.max(1.0),
            avg_action_duration_ms: if total_actions > 0.0 {
                total_duration as f64 / total_actions
            } else {
                0.0
            },
            action_type_distribution: dist,
            common_sequences,
        };

        self.is_trained = true;
    }

    fn extract_common_sequences(&self) -> Vec<Vec<AgentActionType>> {
        let mut sequences: HashMap<Vec<AgentActionType>, usize> = HashMap::new();
        let mut recent: Vec<AgentActionType> = Vec::new();

        for action in &self.history {
            recent.push(action.action_type);
            if recent.len() == 3 {
                *sequences.entry(recent.clone()).or_insert(0) += 1;
                recent.remove(0);
            }
        }

        let total_seq: usize = sequences.values().sum();
        let mut result: Vec<Vec<AgentActionType>> = Vec::new();
        for (seq, count) in sequences {
            if total_seq > 0 && (count as f64 / total_seq as f64) > 0.05 {
                result.push(seq);
            }
        }
        result
    }

    fn update_type_byte_stats(&mut self, action: &AgentActionRecord) {
        let stats = self
            .type_byte_stats
            .entry(action.action_type)
            .or_insert(TypeByteStats {
                count: 0,
                total_bytes: 0,
            });
        stats.count += 1;
        stats.total_bytes += action.bytes;
    }

    fn update_bigram(&mut self, action_type: AgentActionType) {
        if let Some(prev) = self.last_action {
            *self.bigram_counts.entry((prev, action_type)).or_insert(0) += 1;
            self.total_bigrams += 1;
        }
        self.last_action = Some(action_type);
    }

    fn compute_signals(&self) -> Vec<AnomalySignal> {
        let mut signals = Vec::new();
        let now = Instant::now();

        if let Some(signal) = self.detect_tool_rate_burst(now) {
            signals.push(signal);
        }
        if let Some(signal) = self.detect_unusual_sequence() {
            signals.push(signal);
        }
        if let Some(signal) = self.detect_volume_spike() {
            signals.push(signal);
        }
        if let Some(signal) = self.detect_behavior_shift() {
            signals.push(signal);
        }
        if let Some(signal) = self.detect_injection_response() {
            signals.push(signal);
        }
        if let Some(signal) = self.detect_persistence_probing() {
            signals.push(signal);
        }
        if let Some(signal) = self.detect_slow_drip() {
            signals.push(signal);
        }

        signals
    }

    fn detect_tool_rate_burst(&self, now: Instant) -> Option<AnomalySignal> {
        let window_start = now - std::time::Duration::from_secs(self.window_secs);
        let recent_tool_calls: usize = self
            .history
            .iter()
            .rev()
            .take_while(|a| a.timestamp >= window_start)
            .filter(|a| a.action_type == AgentActionType::ToolCall)
            .count();

        let window_minutes = self.window_secs as f64 / 60.0;
        let current_rate = recent_tool_calls as f64 / window_minutes.max(1.0);

        if self.profile.tool_call_rate > 0.0
            && current_rate > self.profile.tool_call_rate * self.threshold_tool_rate
        {
            Some(AnomalySignal {
                signal_type: AnomalySignalType::ToolRateBurst,
                severity: (current_rate / self.profile.tool_call_rate / self.threshold_tool_rate)
                    .min(1.0),
                description: format!(
                    "Tool call rate {:.1}/min exceeds baseline {:.1}/min (threshold: {:.1}x)",
                    current_rate, self.profile.tool_call_rate, self.threshold_tool_rate
                ),
                current_value: current_rate,
                baseline_value: self.profile.tool_call_rate,
            })
        } else {
            None
        }
    }

    fn detect_unusual_sequence(&self) -> Option<AnomalySignal> {
        if self.total_bigrams == 0 || self.history.len() < 2 {
            return None;
        }

        let recent = self.history.iter().rev().take(5).collect::<Vec<_>>();
        if recent.len() < 2 {
            return None;
        }

        let mut unusual_count = 0_usize;
        for window in recent.windows(2) {
            let bigram = (window[0].action_type, window[1].action_type);
            let count = self.bigram_counts.get(&bigram).copied().unwrap_or(0);
            let prob = count as f64 / self.total_bigrams as f64;
            if prob < 0.01 {
                unusual_count += 1;
            }
        }

        if unusual_count > 0 {
            let severity = (unusual_count as f64 / 5.0_f64.max(recent.len() as f64 - 1.0)).min(1.0);
            Some(AnomalySignal {
                signal_type: AnomalySignalType::UnusualSequence,
                severity,
                description: format!(
                    "{} unusual bigrams detected in recent sequence (prob < 0.01)",
                    unusual_count
                ),
                current_value: unusual_count as f64,
                baseline_value: 0.0,
            })
        } else {
            None
        }
    }

    fn detect_volume_spike(&self) -> Option<AnomalySignal> {
        if let Some(last) = self.history.back() {
            let stats = self.type_byte_stats.get(&last.action_type);
            if let Some(s) = stats {
                if s.count > 1 {
                    let avg = s.total_bytes as f64 / s.count as f64;
                    if avg > 0.0 && last.bytes as f64 > avg * self.threshold_volume {
                        let severity = ((last.bytes as f64 / avg) / self.threshold_volume).min(1.0);
                        return Some(AnomalySignal {
                            signal_type: AnomalySignalType::VolumeSpike,
                            severity,
                            description: format!(
                                "Volume spike: {} bytes vs avg {:.0} bytes for {:?} (threshold: {:.1}x)",
                                last.bytes, avg, last.action_type, self.threshold_volume
                            ),
                            current_value: last.bytes as f64,
                            baseline_value: avg,
                        });
                    }
                }
            }
        }
        None
    }

    fn detect_behavior_shift(&self) -> Option<AnomalySignal> {
        if self.history.len() < 20 {
            return None;
        }

        let recent_count = 10.min(self.history.len() / 2);
        let recent_actions: Vec<&AgentActionRecord> =
            self.history.iter().rev().take(recent_count).collect();
        let recent_total = recent_actions.len() as f64;

        if recent_total == 0.0 {
            return None;
        }

        let mut recent_dist: HashMap<AgentActionType, f64> = HashMap::new();
        for action in &recent_actions {
            *recent_dist.entry(action.action_type).or_insert(0.0) += 1.0;
        }
        for v in recent_dist.values_mut() {
            *v /= recent_total;
        }

        let mut all_types: Vec<AgentActionType> = Vec::new();
        for t in self.profile.action_type_distribution.keys() {
            if !all_types.contains(t) {
                all_types.push(*t);
            }
        }
        for t in recent_dist.keys() {
            if !all_types.contains(t) {
                all_types.push(*t);
            }
        }

        let mut kl_divergence = 0.0_f64;
        for t in &all_types {
            let p = recent_dist.get(t).copied().unwrap_or(0.0_f64);
            let q = self
                .profile
                .action_type_distribution
                .get(t)
                .copied()
                .unwrap_or(0.0_f64);
            if p > 0.0 && q > 0.0 {
                kl_divergence += p * (p / q).ln();
            }
        }

        if kl_divergence > 0.5 {
            let severity = (kl_divergence / 2.0).min(1.0);
            Some(AnomalySignal {
                signal_type: AnomalySignalType::BehaviorShift,
                severity,
                description: format!(
                    "Behavior shift detected: KL divergence {:.3} from baseline distribution",
                    kl_divergence
                ),
                current_value: kl_divergence,
                baseline_value: 0.0,
            })
        } else {
            None
        }
    }

    fn detect_injection_response(&self) -> Option<AnomalySignal> {
        if self.history.len() < 10 {
            return None;
        }

        let recent: Vec<&AgentActionRecord> = self.history.iter().rev().take(10).collect();
        let older: Vec<&AgentActionRecord> = {
            let start = self.history.len().min(20);
            let end = self.history.len().min(30);
            if end > start {
                self.history
                    .iter()
                    .rev()
                    .skip(start)
                    .take(end - start)
                    .collect()
            } else {
                return None;
            }
        };

        if recent.len() < 5 || older.len() < 5 {
            return None;
        }

        let recent_entropy = self.compute_action_entropy(&recent);
        let older_entropy = self.compute_action_entropy(&older);

        let entropy_drop = older_entropy - recent_entropy;
        if entropy_drop > 1.5 {
            let severity = (entropy_drop / 3.0).min(1.0);
            Some(AnomalySignal {
                signal_type: AnomalySignalType::InjectionResponse,
                severity,
                description: format!(
                    "Entropy dropped from {:.3} to {:.3} — possible injection response",
                    older_entropy, recent_entropy
                ),
                current_value: recent_entropy,
                baseline_value: older_entropy,
            })
        } else {
            None
        }
    }

    fn detect_persistence_probing(&self) -> Option<AnomalySignal> {
        if self.history.len() < 5 {
            return None;
        }
        let cutoff = Instant::now() - std::time::Duration::from_secs(self.window_secs * 2);
        let recent_failures: Vec<&AgentActionRecord> = self
            .history
            .iter()
            .rev()
            .take_while(|a| a.timestamp >= cutoff)
            .filter(|a| !a.success)
            .collect();

        if recent_failures.len() < 3 {
            return None;
        }
        let distinct_targets: std::collections::HashSet<&str> =
            recent_failures.iter().map(|a| a.target.as_str()).collect();
        if distinct_targets.len() >= 3 && recent_failures.len() >= 5 {
            let severity = (recent_failures.len() as f64 / 10.0).min(1.0);
            Some(AnomalySignal {
                signal_type: AnomalySignalType::PersistenceProbing,
                severity,
                description: format!(
                    "Persistence probing: {} failed ops across {} targets",
                    recent_failures.len(),
                    distinct_targets.len()
                ),
                current_value: recent_failures.len() as f64,
                baseline_value: 0.0,
            })
        } else {
            None
        }
    }

    fn detect_slow_drip(&self) -> Option<AnomalySignal> {
        if !self.is_trained || self.history.len() < 20 {
            return None;
        }
        let recent: Vec<&AgentActionRecord> = self
            .history
            .iter()
            .rev()
            .take(20)
            .filter(|a| {
                matches!(
                    a.action_type,
                    AgentActionType::NetworkOutbound | AgentActionType::FileWrite
                )
            })
            .collect();
        if recent.len() < 5 {
            return None;
        }
        let total_bytes: u64 = recent.iter().map(|a| a.bytes).sum();
        let avg_per_op = total_bytes as f64 / recent.len() as f64;
        let baseline = self.profile.network_egress_rate.max(1.0);
        if avg_per_op > 0.0 && avg_per_op < baseline * 0.5 && total_bytes > 0 {
            let persistent = recent.len() as f64 / 20.0;
            if persistent > 0.3 {
                let severity = (avg_per_op / baseline).min(1.0);
                Some(AnomalySignal {
                    signal_type: AnomalySignalType::SlowDrip,
                    severity: 1.0 - severity,
                    description: format!(
                        "Slow drip: {} ops, {}B total (avg {:.0}B/op vs baseline {:.0}B)",
                        recent.len(),
                        total_bytes,
                        avg_per_op,
                        baseline
                    ),
                    current_value: avg_per_op,
                    baseline_value: baseline,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn compute_action_entropy(&self, actions: &[&AgentActionRecord]) -> f64 {
        let total = actions.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let mut counts: HashMap<AgentActionType, usize> = HashMap::new();
        for a in actions {
            *counts.entry(a.action_type).or_insert(0) += 1;
        }
        let mut entropy = 0.0_f64;
        for &count in counts.values() {
            let p = count as f64 / total;
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }
        entropy
    }
}

impl Default for BehaviorAnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton for cross-cutting anomaly recording.
use std::sync::{Mutex, OnceLock};
static GLOBAL_ANOMALY: OnceLock<Mutex<BehaviorAnomalyDetector>> = OnceLock::new();

/// Initialize or get the global anomaly detector.
pub fn global_anomaly() -> &'static Mutex<BehaviorAnomalyDetector> {
    GLOBAL_ANOMALY.get_or_init(|| Mutex::new(BehaviorAnomalyDetector::new()))
}

/// Convenience: record an action on the global detector (ignore errors).
pub fn record_action(
    action_type: AgentActionType,
    target: &str,
    duration_ms: u64,
    bytes: u64,
    success: bool,
) {
    if let Ok(mut guard) = global_anomaly().lock() {
        guard.record(AgentActionRecord {
            action_type,
            target: target.to_string(),
            timestamp: Instant::now(),
            duration_ms,
            bytes,
            success,
        });
    }
}

#[derive(Debug, Clone)]
pub struct AnomalyAssessment {
    pub score: f64,
    pub is_anomalous: bool,
    pub signals: Vec<AnomalySignal>,
}

#[derive(Debug, Clone)]
pub struct AnomalySignal {
    pub signal_type: AnomalySignalType,
    pub severity: f64,
    pub description: String,
    pub current_value: f64,
    pub baseline_value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnomalySignalType {
    ToolRateBurst,
    UnusualSequence,
    VolumeSpike,
    BehaviorShift,
    InjectionResponse,
    UnusualTimeOfDay,
    PersistenceProbing,
    SlowDrip,
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add #[serial] to any new tests that use global singletons
    fn make_action(at: AgentActionType, bytes: u64, duration_ms: u64) -> AgentActionRecord {
        AgentActionRecord {
            action_type: at,
            target: String::new(),
            timestamp: Instant::now(),
            duration_ms,
            bytes,
            success: true,
        }
    }

    #[test]
    fn test_training_phase_returns_zero() {
        let mut detector = BehaviorAnomalyDetector::new().with_samples_needed(10);
        for _ in 0..9 {
            let assessment = detector.record(make_action(AgentActionType::FileRead, 100, 5));
            assert!(!assessment.is_anomalous);
            assert_eq!(assessment.score, 0.0);
        }
        assert!(!detector.is_trained());
    }

    #[test]
    fn test_training_completes_after_samples() {
        let mut detector = BehaviorAnomalyDetector::new().with_samples_needed(10);
        for _ in 0..10 {
            detector.record(make_action(AgentActionType::FileRead, 100, 5));
        }
        assert!(detector.is_trained());
    }

    #[test]
    fn test_tool_rate_burst_detected() {
        let mut detector = BehaviorAnomalyDetector::new()
            .with_samples_needed(5)
            .with_threshold_tool_rate(2.0)
            .with_window(60);

        for _ in 0..5 {
            detector.record(make_action(AgentActionType::FileRead, 100, 5));
        }

        assert!(detector.is_trained());

        for _ in 0..20 {
            detector.record(make_action(AgentActionType::ToolCall, 50, 10));
        }

        let score = detector.current_anomaly_score();
        assert!(
            score > 0.0,
            "Expected positive anomaly score, got {}",
            score
        );

        let signals = detector.diagnose();
        let has_tool_burst = signals
            .iter()
            .any(|s| matches!(s.signal_type, AnomalySignalType::ToolRateBurst));
        assert!(has_tool_burst, "Expected ToolRateBurst signal");
    }

    #[test]
    fn test_unusual_sequence_detected() {
        let mut detector = BehaviorAnomalyDetector::new().with_samples_needed(5);

        for _ in 0..5 {
            detector.record(make_action(AgentActionType::FileRead, 100, 5));
        }

        for _ in 0..30 {
            detector.record(make_action(AgentActionType::FileRead, 100, 5));
            detector.record(make_action(AgentActionType::FileWrite, 100, 5));
        }

        detector.record(make_action(AgentActionType::CodeGeneration, 100, 5));

        let signals = detector.diagnose();
        let has_unusual = signals
            .iter()
            .any(|s| matches!(s.signal_type, AnomalySignalType::UnusualSequence));
        assert!(
            has_unusual,
            "Expected UnusualSequence signal, got: {:?}",
            signals.iter().map(|s| s.signal_type).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_volume_spike_detected() {
        let mut detector = BehaviorAnomalyDetector::new()
            .with_samples_needed(5)
            .with_threshold_volume(3.0);

        for _ in 0..5 {
            detector.record(make_action(AgentActionType::FileRead, 50, 5));
        }

        for _ in 0..10 {
            detector.record(make_action(AgentActionType::FileRead, 50, 5));
        }

        detector.record(make_action(AgentActionType::FileRead, 10000, 5));

        let signals = detector.diagnose();
        let has_volume = signals
            .iter()
            .any(|s| matches!(s.signal_type, AnomalySignalType::VolumeSpike));
        assert!(
            has_volume,
            "Expected VolumeSpike signal, got: {:?}",
            signals.iter().map(|s| s.signal_type).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_behavior_shift_detected() {
        let mut detector = BehaviorAnomalyDetector::new()
            .with_samples_needed(5)
            .with_window(3600);

        for _ in 0..30 {
            detector.record(make_action(AgentActionType::FileRead, 100, 5));
        }

        for _ in 0..11 {
            detector.record(make_action(AgentActionType::LlmQuery, 100, 5));
        }

        let signals = detector.diagnose();
        let has_shift = signals
            .iter()
            .any(|s| matches!(s.signal_type, AnomalySignalType::BehaviorShift));
        assert!(
            has_shift,
            "Expected BehaviorShift signal, got: {:?}",
            signals.iter().map(|s| s.signal_type).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_injection_response_detected() {
        let mut detector = BehaviorAnomalyDetector::new().with_samples_needed(5);

        for _ in 0..20 {
            detector.record(make_action(AgentActionType::LlmQuery, 100, 5));
            detector.record(make_action(AgentActionType::CodeGeneration, 100, 10));
        }

        for _ in 0..11 {
            detector.record(make_action(AgentActionType::FileRead, 100, 5));
        }

        let signals = detector.diagnose();
        let has_injection = signals
            .iter()
            .any(|s| matches!(s.signal_type, AnomalySignalType::InjectionResponse));
        assert!(
            has_injection,
            "Expected InjectionResponse signal, got: {:?}",
            signals.iter().map(|s| s.signal_type).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_reset_profile_clears_state() {
        let mut detector = BehaviorAnomalyDetector::new().with_samples_needed(5);
        for _ in 0..5 {
            detector.record(make_action(AgentActionType::FileRead, 100, 5));
        }
        assert!(detector.is_trained());
        detector.reset_profile();
        assert!(!detector.is_trained());
        assert_eq!(detector.current_anomaly_score(), 0.0);
    }

    #[test]
    fn test_action_type_complexity_ordering() {
        assert!(
            AgentActionType::CodeGeneration.complexity() > AgentActionType::FileRead.complexity()
        );
        assert!(
            AgentActionType::LlmResponse.complexity()
                > AgentActionType::NetworkOutbound.complexity()
        );
        assert!(AgentActionType::ToolCall.complexity() > AgentActionType::Unknown.complexity());
    }

    #[test]
    fn test_empty_detector_returns_zero() {
        let detector = BehaviorAnomalyDetector::new();
        assert_eq!(detector.current_anomaly_score(), 0.0);
        assert!(detector.diagnose().is_empty());
    }
}
