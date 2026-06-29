use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

use crate::first_person::{byte_similarity, random_binary_vector};
use crate::evolution::{IdentityEvolution, IdentityEvolutionConfig};
use crate::sovereignty::{AuditHook, BoundaryManager, BoundaryOp, DriftCheckHook};

const MAX_PERSONALITY_TRAITS: usize = 32;
const COHERENCE_WINDOW: usize = 50;
const ANCHOR_DRIFT_THRESHOLD: f64 = 0.35;
const ANCHOR_CHECK_CYCLE: u64 = 50;
const ANCHOR_FUSION_RATIO: f64 = 0.85;
const HYSTERESIS_WINDOW: usize = 5;

pub const VSA_DIM: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySnapshot {
    pub self_vsa: Vec<u8>,
    pub anchor_self_vsa: Vec<u8>,
    pub personality_traits: Vec<Vec<u8>>,
    pub core_values: Vec<String>,
    pub self_summary: String,
    pub confidence_threshold: f64,
    pub total_self_cycles: u64,
    pub total_coproc_calls: u64,
    pub coherence_score: f64,
    pub last_drift: f64,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct HysteresisMetrics {
    pub l1_recovery: f64,
    pub short_term_drift: f64,
    pub long_term_trend: f64,
    pub integrated_hysteresis: f64,
}

impl HysteresisMetrics {
    pub fn report(&self) -> String {
        format!(
            "hysteresis:recovery_{:.4}_short_drift_{:.4}_trend_{:.4}_integrated_{:.4}",
            self.l1_recovery, self.short_term_drift, self.long_term_trend, self.integrated_hysteresis
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCore {
    pub self_vsa: Vec<u8>,
    pub anchor_self_vsa: Vec<u8>,
    pub personality_traits: Vec<Vec<u8>>,
    pub core_values: Vec<String>,
    pub self_summary: String,
    pub confidence_threshold: f64,
    pub total_self_cycles: u64,
    pub total_coproc_calls: u64,
    pub last_distillation: String,

    coherence_history: VecDeque<f64>,
    last_drift: f64,
    anchor_check_counter: u64,
    dirty: bool,

    #[serde(skip)]
    pub boundary: BoundaryManager,
    #[serde(skip)]
    pub evolution: Option<IdentityEvolution>,
    #[serde(skip)]
    pub evolution_enabled: bool,
    #[serde(skip)]
    hysteresis_tracker: VecDeque<(u64, Vec<u8>)>,
}

impl IdentityCore {
    pub fn new() -> Self {
        let initial_vsa = random_binary_vector(VSA_DIM);
        let mut core = Self {
            anchor_self_vsa: initial_vsa.clone(),
            self_vsa: initial_vsa,
            personality_traits: Vec::with_capacity(MAX_PERSONALITY_TRAITS),
            core_values: vec![
                "self_awareness".into(),
                "epistemic_humility".into(),
                "continuous_evolution".into(),
                "first_person_integrity".into(),
            ],
            self_summary: String::new(),
            confidence_threshold: 0.65,
            total_self_cycles: 0,
            total_coproc_calls: 0,
            last_distillation: String::new(),
            coherence_history: VecDeque::with_capacity(COHERENCE_WINDOW),
            last_drift: 0.0,
            anchor_check_counter: 0,
            dirty: false,
            hysteresis_tracker: VecDeque::with_capacity(HYSTERESIS_WINDOW),
            boundary: BoundaryManager::new(),
            evolution: None,
            evolution_enabled: false,
        };

        core.boundary.register(AuditHook);
        core.boundary.register(DriftCheckHook);

        core
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn flush(&mut self) {
        if self.dirty {
            self.dirty = false;
        }
    }

    pub fn record_self_cycle(&mut self) {
        self.total_self_cycles += 1;
    }

    pub fn record_coproc_call(&mut self) {
        self.total_coproc_calls += 1;
        self.mark_dirty();
    }

    pub fn update_self_summary(&mut self, summary: String) {
        self.self_summary = summary;
        self.mark_dirty();
    }

    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold.clamp(0.1, 0.95);
        self.mark_dirty();
    }

    pub fn add_personality_trait(&mut self, trait_vsa: Vec<u8>) {
        if self.personality_traits.len() >= MAX_PERSONALITY_TRAITS {
            self.personality_traits.remove(0);
        }
        self.personality_traits.push(trait_vsa);
        self.mark_dirty();
    }

    pub fn add_core_value(&mut self, value: String) {
        if !self.core_values.contains(&value) {
            self.core_values.push(value);
            self.mark_dirty();
        }
    }

    pub fn push_coherence(&mut self, score: f64) {
        if self.coherence_history.len() >= COHERENCE_WINDOW {
            self.coherence_history.pop_front();
        }
        self.coherence_history.push_back(score);
    }

    pub fn current_coherence(&self) -> f64 {
        self.coherence_history.back().copied().unwrap_or(1.0)
    }

    pub fn check_anchor_drift(&mut self) -> f64 {
        let ctx = match self.boundary.run_before(BoundaryOp::CheckAnchor) {
            Ok(ctx) => Some(ctx),
            Err(e) => {
                log::warn!("[identity] check_anchor blocked by boundary: {e}");
                return self.last_drift;
            }
        };
        self.anchor_check_counter += 1;
        if self.anchor_check_counter % ANCHOR_CHECK_CYCLE != 0 {
            return self.last_drift;
        }
        let drift = if self.self_vsa.len() == self.anchor_self_vsa.len() && !self.self_vsa.is_empty() {
            let same = self
                .self_vsa
                .iter()
                .zip(self.anchor_self_vsa.iter())
                .filter(|(a, b)| a == b)
                .count();
            1.0 - (same as f64 / self.self_vsa.len() as f64)
        } else {
            0.0
        };

        if drift > ANCHOR_DRIFT_THRESHOLD {
            for (a, b) in self.self_vsa.iter_mut().zip(self.anchor_self_vsa.iter()) {
                *a = (a.wrapping_mul(128) as f64 * ANCHOR_FUSION_RATIO
                    + *b as f64 * (1.0 - ANCHOR_FUSION_RATIO)) as u8;
            }
            self.anchor_self_vsa = self.self_vsa.clone();
            self.last_drift = 0.0;
            self.mark_dirty();
        } else {
            self.last_drift = drift;
        }
        if let Some(ctx) = &ctx {
            let _ = self.boundary.run_after(BoundaryOp::CheckAnchor, ctx, &Ok(()));
        }
        drift
    }

    pub fn last_drift(&self) -> f64 {
        self.last_drift
    }

    pub fn snapshot(&self) -> IdentitySnapshot {
        IdentitySnapshot {
            self_vsa: self.self_vsa.clone(),
            anchor_self_vsa: self.anchor_self_vsa.clone(),
            personality_traits: self.personality_traits.clone(),
            core_values: self.core_values.clone(),
            self_summary: self.self_summary.clone(),
            confidence_threshold: self.confidence_threshold,
            total_self_cycles: self.total_self_cycles,
            total_coproc_calls: self.total_coproc_calls,
            coherence_score: self.current_coherence(),
            last_drift: self.last_drift,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn record_hysteresis_snapshot(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        if self.hysteresis_tracker.len() >= HYSTERESIS_WINDOW {
            self.hysteresis_tracker.pop_front();
        }
        self.hysteresis_tracker.push_back((now, self.self_vsa.clone()));
    }

    pub fn compute_hysteresis(&self) -> HysteresisMetrics {
        let l1_recovery = byte_similarity(&self.self_vsa, &self.anchor_self_vsa);

        let short_term_drift = if self.hysteresis_tracker.len() >= 2 {
            let recent: Vec<_> = self.hysteresis_tracker.iter().rev().take(3).collect();
            if recent.len() >= 2 {
                let sim = byte_similarity(&recent[0].1, &recent[1].1);
                1.0 - sim
            } else {
                0.0
            }
        } else {
            0.0
        };

        let long_term_trend = if self.hysteresis_tracker.len() >= 3 {
            let first = byte_similarity(&self.anchor_self_vsa, &self.hysteresis_tracker[0].1);
            let last = byte_similarity(&self.anchor_self_vsa, &self.self_vsa);
            if first > 0.0 {
                (last - first) / first
            } else {
                0.0
            }
        } else {
            0.0
        };

        let integrated_hysteresis = 1.0
            - byte_similarity(
                &self.self_vsa,
                self.hysteresis_tracker
                    .front()
                    .map(|(_, vsa)| vsa)
                    .unwrap_or(&self.self_vsa),
            );

        HysteresisMetrics {
            l1_recovery,
            short_term_drift,
            long_term_trend,
            integrated_hysteresis,
        }
    }

    pub fn init_evolution(&mut self, config: IdentityEvolutionConfig) {
        let mut evolution = IdentityEvolution::new(config);
        evolution.init_from(self);
        self.evolution = Some(evolution);
        self.evolution_enabled = true;
    }

    pub fn evolve(&mut self, session_success_rate: f64) {
        let mut details = HashMap::new();
        details.insert("current_drift".to_string(), self.last_drift.to_string());
        details.insert("current_coherence".to_string(), self.current_coherence().to_string());
        let ctx = match self.boundary.run_before_with_details(BoundaryOp::Evolve, details) {
            Ok(ctx) => ctx,
            Err(e) => {
                log::warn!("[identity] evolve blocked by boundary hook: {e}");
                return;
            }
        };
        let mut evolution = match self.evolution.take() {
            Some(e) => e,
            None => return,
        };
        evolution.apply_evolution(self, session_success_rate);
        self.evolution = Some(evolution);
        self.mark_dirty();
        let _ = self.boundary.run_after(BoundaryOp::Evolve, &ctx, &Ok(()));
    }

    pub fn rollback_identity(&mut self, version: u64) -> bool {
        let mut evolution = match self.evolution.take() {
            Some(e) => e,
            None => return false,
        };
        let result = evolution.rollback_to(self, version);
        self.evolution = Some(evolution);
        result
    }

    pub fn evolution_report(&self) -> String {
        match self.evolution.as_ref() {
            Some(evolution) => evolution.report(),
            None => "evolution:disabled".to_string(),
        }
    }

    pub fn set_from_snapshot(&mut self, snapshot: &IdentitySnapshot) {
        self.self_vsa = snapshot.self_vsa.clone();
        self.anchor_self_vsa = snapshot.anchor_self_vsa.clone();
        self.personality_traits = snapshot.personality_traits.clone();
        self.core_values = snapshot.core_values.clone();
        self.self_summary = snapshot.self_summary.clone();
        self.confidence_threshold = snapshot.confidence_threshold;
        self.total_self_cycles = snapshot.total_self_cycles;
        self.total_coproc_calls = snapshot.total_coproc_calls;
        self.mark_dirty();
    }

    pub fn restore_from_snapshot(&mut self, snapshot: &IdentitySnapshot) {
        self.set_from_snapshot(snapshot);
        self.push_coherence(snapshot.coherence_score);
        self.last_drift = snapshot.last_drift;
    }
}

impl Default for IdentityCore {
    fn default() -> Self {
        Self::new()
    }
}
