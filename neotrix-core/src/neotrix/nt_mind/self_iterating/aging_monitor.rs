use super::SelfIteratingBrain;
use super::pipeline::{BrainStage, StageDecision};
use crate::neotrix::error::NeoTrixError;
use std::collections::VecDeque;

/// Tracks agent degradation over deployment — the four aging mechanisms
/// from AgingBench: compression, interference, revision, maintenance.
///
/// Even when model weights are frozen, an agent's effective state changes
/// as it compresses interaction history, retrieves from growing memory,
/// revises facts, and undergoes maintenance. This monitor detects which
/// form of aging is occurring and provides diagnostic information for repair.
#[derive(Debug, Clone)]
pub struct AgingMonitor {
    /// Compression aging: signal loss from KV cache / context compression
    pub compression_score: f64,
    /// Interference aging: cross-talk in growing memory stores
    pub interference_score: f64,
    /// Revision aging: inconsistency from fact updates
    pub revision_score: f64,
    /// Maintenance aging: drift from routine maintenance cycles
    pub maintenance_score: f64,
    /// Rolling window of capability snapshots for trend analysis
    pub capability_history: VecDeque<(u64, Vec<f64>)>,
    /// Max history length
    pub max_history: usize,
    /// Alert threshold per mechanism
    pub alert_threshold: f64,
}

impl AgingMonitor {
    pub fn new(max_history: usize, alert_threshold: f64) -> Self {
        Self {
            compression_score: 0.0,
            interference_score: 0.0,
            revision_score: 0.0,
            maintenance_score: 0.0,
            capability_history: VecDeque::with_capacity(max_history),
            max_history,
            alert_threshold,
        }
    }

    pub fn record_snapshot(&mut self, iteration: u64, capability: &crate::core::CapabilityVector) {
        if self.capability_history.len() >= self.max_history {
            self.capability_history.pop_front();
        }
        self.capability_history.push_back((iteration, capability.to_full_vector()));
    }

    /// Detect compression aging: variance loss in compressed representations.
    pub fn detect_compression(&self) -> f64 {
        let n = self.capability_history.len();
        if n < 3 {
            return 0.0;
        }
        let recent: Vec<&[f64]> = self.capability_history.iter().rev().take(3).map(|(_, v)| v.as_slice()).collect();
        let mut dim_variances = 0.0;
        let dim_count = recent[0].len();
        for d in 0..dim_count {
            let mut vals = Vec::with_capacity(3);
            for &v in &recent {
                if d < v.len() {
                    vals.push(v[d]);
                }
            }
            if vals.len() >= 2 {
                let mean: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
                let variance: f64 = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / vals.len() as f64;
                dim_variances += variance;
            }
        }
        let avg_variance = dim_variances / dim_count as f64;
        (1.0 - avg_variance * 10.0).clamp(0.0, 1.0)
    }

    /// Detect interference aging: retrieval cross-talk in growing memory.
    pub fn detect_interference(&self, brain: &SelfIteratingBrain) -> f64 {
        let memories = brain.reasoning_bank.memories();
        let memory_count = memories.len() as f64;
        if memory_count < 5.0 {
            return 0.0;
        }
        let mem_vec: Vec<_> = memories.iter().collect();
        let similarity_sum: f64 = mem_vec
            .windows(2)
            .map(|w| (w[0].reward - w[1].reward).abs())
            .sum();
        let avg_diff: f64 = if memory_count > 1.0 { similarity_sum / memory_count } else { 0.0 };
        (1.0 - avg_diff * 2.0).max(0.0).min(1.0)
    }

    /// Detect revision aging: inconsistency from sequential fact updates.
    pub fn detect_revision(&self) -> f64 {
        let n = self.capability_history.len();
        if n < 5 {
            return 0.0;
        }
        let recent: Vec<f64> = self.capability_history.iter().rev().take(5)
            .map(|(_, v)| v.iter().sum::<f64>())
            .collect();
        let mut direction_changes = 0;
        for i in 2..recent.len() {
            let d1 = recent[i-1] - recent[i-2];
            let d2 = recent[i] - recent[i-1];
            if d1.signum() != d2.signum() && d1.abs() > 0.001 && d2.abs() > 0.001 {
                direction_changes += 1;
            }
        }
        (direction_changes as f64 / 3.0).max(0.0).min(1.0)
    }

    /// Detect maintenance aging: drift after consolidation cycles.
    pub fn detect_maintenance(&self, brain: &SelfIteratingBrain) -> f64 {
        let total_absorbs = brain.brain.total_absorb_count;
        if total_absorbs < 5 {
            return 0.0;
        }
        let n = self.capability_history.len();
        if n < 3 {
            return 0.0;
        }
        let first = &self.capability_history.front().map(|(_, v)| v.clone()).unwrap_or_default();
        let last = &self.capability_history.back().map(|(_, v)| v.clone()).unwrap_or_default();
        if first.is_empty() || last.is_empty() {
            return 0.0;
        }
        let drift: f64 = first.iter().zip(last.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>() / first.len() as f64;
        (drift * 5.0).max(0.0).min(1.0)
    }

    pub fn update_all(&mut self, brain: &SelfIteratingBrain) {
        self.compression_score = self.detect_compression();
        self.interference_score = self.detect_interference(brain);
        self.revision_score = self.detect_revision();
        self.maintenance_score = self.detect_maintenance(brain);
    }

    pub fn overall_aging(&self) -> f64 {
        (self.compression_score + self.interference_score + self.revision_score + self.maintenance_score) / 4.0
    }

    pub fn has_alert(&self) -> Option<&'static str> {
        if self.compression_score > self.alert_threshold {
            return Some("compression aging exceeds threshold");
        }
        if self.interference_score > self.alert_threshold {
            return Some("interference aging exceeds threshold");
        }
        if self.revision_score > self.alert_threshold {
            return Some("revision aging exceeds threshold");
        }
        if self.maintenance_score > self.alert_threshold {
            return Some("maintenance aging exceeds threshold");
        }
        None
    }

    pub fn diagnostic_report(&self) -> String {
        format!(
            "AgingDiagnostic {{ compression: {:.3}, interference: {:.3}, revision: {:.3}, maintenance: {:.3}, overall: {:.3} }}",
            self.compression_score,
            self.interference_score,
            self.revision_score,
            self.maintenance_score,
            self.overall_aging(),
        )
    }
}

impl Default for AgingMonitor {
    fn default() -> Self {
        Self::new(50, 0.7)
    }
}

pub struct AgingDiagnosisStage;
impl Default for AgingDiagnosisStage { fn default() -> Self { Self } }
impl AgingDiagnosisStage { pub fn new() -> Self { Self } }
impl BrainStage for AgingDiagnosisStage {
    fn name(&self) -> &str { "aging_diagnosis" }
    fn frequency(&self) -> usize { 5 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let iteration = brain.iteration;
        let capability = brain.brain.capability.clone();
        let memory_count = brain.reasoning_bank.memories().len();
        let memory_pairs: Vec<(f64, f64)> = {
            let mems = brain.reasoning_bank.memories();
            let mems_len = mems.len();
            if mems_len >= 2 {
                let vec: Vec<_> = mems.iter().collect();
                vec.windows(2).map(|w| (w[0].reward, w[1].reward)).collect()
            } else {
                Vec::new()
            }
        };
        let total_absorbs = brain.brain.total_absorb_count;

        {
            let monitor = &mut brain._aging_monitor;
            monitor.record_snapshot(iteration, &capability);
            monitor.compression_score = monitor.detect_compression();
            monitor.interference_score = if memory_count >= 5 && !memory_pairs.is_empty() {
                let similarity_sum: f64 = memory_pairs.iter().map(|(a, b)| (a - b).abs()).sum();
                (1.0 - similarity_sum / memory_count as f64 * 2.0).max(0.0).min(1.0)
            } else { 0.0 };
            monitor.revision_score = monitor.detect_revision();
            monitor.maintenance_score = if total_absorbs >= 5 {
                let n = monitor.capability_history.len();
                if n >= 3 {
                    match (monitor.capability_history.front(), monitor.capability_history.back()) {
                        (Some(first), Some(last)) => {
                            let drift: f64 = first.1.iter().zip(last.1.iter())
                                .map(|(a, b)| (a - b).abs())
                                .sum::<f64>() / first.1.len() as f64;
                            (drift * 5.0).max(0.0).min(1.0)
                        }
                        _ => 0.0,
                    }
                } else { 0.0 }
            } else { 0.0 };
        }

        let report = brain._aging_monitor.diagnostic_report();
        log::info!("[aging] {}", report);

        if let Some(alert) = brain._aging_monitor.has_alert() {
            log::warn!("[aging] ALERT: {}", alert);
            let existing = brain._open_source_insights.clone().unwrap_or_default();
            let combined = if existing.is_empty() {
                format!("Aging alert: {}", alert)
            } else {
                format!("{} | Aging alert: {}", existing, alert)
            };
            brain._set_open_source_insights(Some(combined));
        }

        Ok(StageDecision::Continue)
    }
}
