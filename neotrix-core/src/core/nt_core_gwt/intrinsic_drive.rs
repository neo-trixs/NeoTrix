//! Intrinsic Drive System (Phase 10.2)
//!
//! Curiosity, mastery, and coherence-seeking drives that modulate
//! salience to balance exploration vs exploitation.

use crate::core::nt_core_consciousness::affective_circumplex::AffectiveCircumplex;
use crate::core::nt_core_consciousness::caa_steering::CaaSteeringEngine;
use crate::core::nt_core_gwt::epistemic_queue::{EpistemicQueue, GapType};
use crate::core::nt_core_gwt::resonance::{
    compute_broadcast_entropy, compute_content_entropy, curiosity_signal, MODULE_COUNT,
};

/// Three-component intrinsic motivation drive.
#[derive(Debug, Clone)]
pub struct IntrinsicDrive {
    /// Drive to explore novel content (0-1)
    pub curiosity: f64,
    /// Drive to improve prediction accuracy (0-1)
    pub mastery: f64,
    /// Drive to maintain coherent self (0-1)
    pub coherence_seek: f64,
    /// Weighted sum of all drives (0-1)
    pub overall_arousal: f64,
    /// Structured epistemic gap queue for curiosity-driven exploration
    pub epistemic_queue: Option<EpistemicQueue>,
    /// CAA steering engine for affective residual-stream intervention
    pub steering_engine: Option<CaaSteeringEngine>,
    /// Affective circumplex for generation parameter modulation
    pub circumplex: Option<AffectiveCircumplex>,
}

impl Default for IntrinsicDrive {
    fn default() -> Self {
        Self::new()
    }
}

impl IntrinsicDrive {
    pub fn new() -> Self {
        Self {
            curiosity: 0.1,
            mastery: 0.3,
            coherence_seek: 0.3,
            overall_arousal: 0.3,
            epistemic_queue: None,
            steering_engine: Some(CaaSteeringEngine::new()),
            circumplex: Some(AffectiveCircumplex::new()),
        }
    }

    /// Compute intrinsic drives from current cognitive state.
    /// - `broadcast_history`: recent GWT broadcast winners
    /// - `saliences`: current salience distribution across modules
    /// - `meta_accuracy`: how accurately the system predicts its own performance
    /// - `coherence`: self-coherence (how integrated the self-model is)
    pub fn compute(
        broadcast_history: &[usize],
        saliences: &[f64],
        meta_accuracy: f64,
        coherence: f64,
    ) -> Self {
        Self::compute_inner(broadcast_history, saliences, meta_accuracy, coherence, 0)
    }

    pub fn compute_with_queue(
        broadcast_history: &[usize],
        saliences: &[f64],
        meta_accuracy: f64,
        coherence: f64,
        unresolved_gaps: usize,
    ) -> Self {
        Self::compute_inner(
            broadcast_history,
            saliences,
            meta_accuracy,
            coherence,
            unresolved_gaps,
        )
    }

    fn compute_inner(
        broadcast_history: &[usize],
        saliences: &[f64],
        meta_accuracy: f64,
        coherence: f64,
        unresolved_gaps: usize,
    ) -> Self {
        let content_entropy = compute_content_entropy(saliences);
        let broadcast_entropy = compute_broadcast_entropy(broadcast_history, MODULE_COUNT);
        let mut curiosity = curiosity_signal(content_entropy, broadcast_entropy);

        if unresolved_gaps > 5 {
            let boost = ((unresolved_gaps - 5) as f64 * 0.05).min(0.3);
            curiosity = (curiosity + boost).min(1.0);
        }

        let mastery = (1.0 - meta_accuracy).clamp(0.0, 1.0);
        let coherence_seek = (1.0 - coherence).clamp(0.0, 1.0);
        let overall_arousal =
            (curiosity * 0.4 + mastery * 0.3 + coherence_seek * 0.3).clamp(0.0, 1.0);
        Self {
            curiosity,
            mastery,
            coherence_seek,
            overall_arousal,
            epistemic_queue: None,
            steering_engine: None,
            circumplex: None,
        }
    }

    /// Modulate a module's raw salience based on intrinsic drive state.
    /// Modules that haven't won recently get boosted when curiosity is high.
    pub fn modulate_salience(
        &self,
        raw_salience: f64,
        module_idx: usize,
        broadcast_history: &[usize],
    ) -> f64 {
        if self.curiosity < 0.2 {
            return raw_salience;
        }
        let window: Vec<usize> = broadcast_history.iter().rev().take(10).copied().collect();
        let has_won_recently = window.iter().any(|&w| w == module_idx);
        if !has_won_recently {
            let boost = raw_salience * self.curiosity * 0.25;
            (raw_salience + boost).min(1.0)
        } else {
            raw_salience
        }
    }

    /// Decay all drives over time (prevents stuck states).
    pub fn step(&mut self, dt: f64) {
        let decay = (-2.0 * dt).exp();
        self.curiosity *= decay;
        self.mastery *= decay;
        self.coherence_seek *= decay;
        self.curiosity = self.curiosity.clamp(0.01, 1.0);
        self.mastery = self.mastery.clamp(0.01, 1.0);
        self.coherence_seek = self.coherence_seek.clamp(0.01, 1.0);
        self.overall_arousal =
            (self.curiosity * 0.4 + self.mastery * 0.3 + self.coherence_seek * 0.3).clamp(0.0, 1.0);
    }

    pub fn ensure_queue(&mut self, max_size: usize) {
        if self.epistemic_queue.is_none() {
            self.epistemic_queue = Some(EpistemicQueue::new(max_size));
        }
    }

    pub fn push_gap(
        &mut self,
        gap_type: GapType,
        priority: f64,
        domain: String,
        description: String,
        vsa_signature: Vec<u8>,
    ) -> Option<u64> {
        self.ensure_queue(100);
        if let Some(ref mut q) = self.epistemic_queue {
            Some(q.push(gap_type, priority, domain, description, vsa_signature))
        } else {
            None
        }
    }

    pub fn integrate_curiosity_with_queue(&mut self, domain: &str, desc: &str, intensity: f64) {
        self.ensure_queue(100);
        if let Some(ref mut q) = self.epistemic_queue {
            let vsa = crate::core::nt_core_hcube::QuantizedVSA::seeded_random(
                desc.bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64)),
                64,
            );
            q.ingest_from_curiosity(domain, desc, intensity, vsa);
            let unresolved = q.unresolved_count();
            if unresolved > 5 {
                let boost = ((unresolved - 5) as f64 * 0.03).min(0.2);
                self.curiosity = (self.curiosity + boost).min(1.0);
                self.overall_arousal =
                    (self.curiosity * 0.4 + self.mastery * 0.3 + self.coherence_seek * 0.3)
                        .clamp(0.0, 1.0);
            }
        }
    }

    /// Apply CAA steering to a generation VSA vector.
    pub fn apply_steering(&mut self, generation_vsa: &mut [u8]) {
        if let Some(ref engine) = self.steering_engine {
            if !engine.enabled {
                return;
            }
        }
        if let Some(ref mut engine) = self.steering_engine {
            let ctx = generation_vsa.to_vec();
            let emotion = if self.curiosity > 0.6 {
                "curiosity"
            } else if self.mastery > 0.6 {
                "mastery"
            } else {
                "coherence_seek"
            };
            let intensity = (self.curiosity * 0.4 + self.mastery * 0.3 + self.coherence_seek * 0.3)
                .clamp(0.0, 1.0);
            let direction = engine.compute_direction(emotion, intensity, &ctx);
            let before = generation_vsa.to_vec();
            engine.steer_generation(&direction, generation_vsa);
            let divergence =
                1.0 - crate::core::nt_core_hcube::QuantizedVSA::similarity(&before, generation_vsa);
            let passed = divergence > 0.01;
            engine.record_steering("computed", emotion, direction.alpha, divergence, passed);
        }
    }
}
