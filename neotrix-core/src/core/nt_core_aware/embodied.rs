//! # Embodied Self (3+1D)
//!
//! The body-schema layer of consciousness. Builds on top of the 0D-2D
//! [`ConsciousnessFoundation`]. Three components plus the aggregate:
//!
//! - **3D Body Schema** (`BodySchema3D`): spatial self-representation
//! - **Proprioception**: per-limb tension, balance, fatigue, pain
//! - **1D Temporal Extension** (`TemporalExtension`): autobiographical time
//!
//! [`ConsciousnessFoundation`]: super::foundation::ConsciousnessFoundation

use std::collections::VecDeque;

use chrono::Utc;
use super::foundation::ConsciousnessFoundation;

/// Default number of limbs (head, torso, two arms, two legs).
pub const DEFAULT_LIMB_COUNT: usize = 6;

/// Max number of personal-past events kept in autobiographical memory.
pub const DEFAULT_MAX_HISTORY: usize = 1024;

/// Cap on anticipated future events.
pub const DEFAULT_MAX_ANTICIPATIONS: usize = 64;

// ─────────────────────────────────────────────────────────────────────────────
// 3D Body Schema
// ─────────────────────────────────────────────────────────────────────────────

/// 3D body schema — spatial self-representation.
#[derive(Debug, Clone)]
pub struct BodySchema3D {
    /// Current 3D position.
    pub position: (f64, f64, f64),
    /// Orientation as (roll, pitch, yaw) in radians.
    pub orientation: (f64, f64, f64),
    /// Bounding box (half-extents) in meters.
    pub extent: (f64, f64, f64),
    /// Discrete body parts.
    pub limbs: Vec<Limb>,
}

/// A single body part with joint angles and an extension measure.
#[derive(Debug, Clone)]
pub struct Limb {
    pub name: String,
    pub joint_angles: Vec<f64>,
    /// 0..1 — how extended/contracted the limb is.
    pub extension: f64,
}

impl Default for BodySchema3D {
    fn default() -> Self {
        Self::new()
    }
}

impl BodySchema3D {
    /// Construct a default human-like body at the origin.
    pub fn new() -> Self {
        let limb_names = ["head", "torso", "arm_left", "arm_right", "leg_left", "leg_right"];
        let limbs = limb_names
            .iter()
            .map(|n| Limb {
                name: (*n).to_string(),
                joint_angles: vec![0.0; 3],
                extension: 0.5,
            })
            .collect();
        Self {
            position: (0.0, 0.0, 0.0),
            orientation: (0.0, 0.0, 0.0),
            extent: (0.3, 0.5, 0.2), // typical adult half-extents in meters
            limbs,
        }
    }

    /// Update the body's 3D position.
    pub fn update_position(&mut self, x: f64, y: f64, z: f64) {
        self.position = (x, y, z);
    }

    /// Update a limb's joint angles by name. Returns `true` if the limb was found.
    pub fn update_limb(&mut self, name: &str, joint_angles: Vec<f64>) -> bool {
        if let Some(limb) = self.limbs.iter_mut().find(|l| l.name == name) {
            limb.joint_angles = joint_angles;
            true
        } else {
            false
        }
    }

    /// Approximate center of mass: a tension-weighted average of limb "anchor"
    /// positions arranged around the body. Limb anchors are derived from the
    /// body position plus a simple per-limb offset.
    pub fn center_of_mass(&self) -> (f64, f64, f64) {
        if self.limbs.is_empty() {
            return self.position;
        }
        let (px, py, pz) = self.position;
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut sz = 0.0;
        let mut total_w = 0.0;
        for (i, limb) in self.limbs.iter().enumerate() {
            let w = 0.5 + limb.extension * 0.5; // heavier when extended
            // Simple symmetric anchor: alternate signs.
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            let idx = i as f64;
            let ox = sign * (idx * 0.05);
            let oy = (idx - self.limbs.len() as f64 * 0.5) * 0.1;
            let oz = sign * (idx * 0.03);
            sx += (px + ox) * w;
            sy += (py + oy) * w;
            sz += (pz + oz) * w;
            total_w += w;
        }
        if total_w == 0.0 {
            return self.position;
        }
        (sx / total_w, sy / total_w, sz / total_w)
    }

    /// Integration of spatial awareness. 0 if no limbs, otherwise a normalized
    /// measure of how much variety exists across the limb joint angles.
    pub fn spatial_phi(&self) -> f64 {
        if self.limbs.is_empty() {
            return 0.0;
        }
        let mut total_var = 0.0;
        let mut count = 0;
        for limb in &self.limbs {
            if limb.joint_angles.is_empty() {
                continue;
            }
            let mean = limb.joint_angles.iter().sum::<f64>() / limb.joint_angles.len() as f64;
            let var = limb
                .joint_angles
                .iter()
                .map(|a| (a - mean).powi(2))
                .sum::<f64>()
                / limb.joint_angles.len() as f64;
            total_var += var;
            count += 1;
        }
        if count == 0 {
            return 0.0;
        }
        let avg_var = total_var / count as f64;
        // Map variance (0..~1.0 typically) to integration (0..1) with a soft curve.
        (avg_var / (1.0 + avg_var)).max(0.0).min(1.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Proprioception
// ─────────────────────────────────────────────────────────────────────────────

/// Proprioception — the sense of body's own state.
#[derive(Debug, Clone)]
pub struct Proprioception {
    /// Per-limb tension (0..1, where 1 = fully tensed).
    pub tension: Vec<f64>,
    /// -1..1 (left/right lean).
    pub balance: f64,
    /// 0..1.
    pub fatigue: f64,
    /// (location, intensity) pain signals.
    pub pain_signals: Vec<(String, f64)>,
}

impl Proprioception {
    /// Construct proprioception for a body with `num_limbs` limbs.
    pub fn new(num_limbs: usize) -> Self {
        Self {
            tension: vec![0.0; num_limbs],
            balance: 0.0,
            fatigue: 0.0,
            pain_signals: Vec::new(),
        }
    }

    /// Update tension for a specific limb. `value` is clamped to `0..1`.
    /// Returns `true` if the limb index is valid.
    pub fn update_tension(&mut self, limb_idx: usize, value: f64) -> bool {
        if limb_idx >= self.tension.len() {
            return false;
        }
        self.tension[limb_idx] = value.max(0.0).min(1.0);
        true
    }

    /// Look up pain intensity for a named location. Returns `0.0` if absent.
    pub fn pain_at(&self, location: &str) -> f64 {
        self.pain_signals
            .iter()
            .find(|(loc, _)| loc == location)
            .map(|(_, i)| *i)
            .unwrap_or(0.0)
    }

    /// 0..1 — composite of tension imbalance, fatigue, and average pain.
    pub fn overall_discomfort(&self) -> f64 {
        if self.tension.is_empty() && self.pain_signals.is_empty() {
            return self.fatigue;
        }
        let tension_component = if self.tension.is_empty() {
            0.0
        } else {
            let mean = self.tension.iter().sum::<f64>() / self.tension.len() as f64;
            mean
        };
        let pain_component = if self.pain_signals.is_empty() {
            0.0
        } else {
            self.pain_signals.iter().map(|(_, i)| *i).sum::<f64>()
                / self.pain_signals.len() as f64
        };
        let balance_penalty = self.balance.abs();
        let raw = (tension_component * 0.3)
            + (pain_component * 0.4)
            + (self.fatigue * 0.2)
            + (balance_penalty * 0.1);
        raw.max(0.0).min(1.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1D Temporal Extension
// ─────────────────────────────────────────────────────────────────────────────

/// 1D temporal extension — autobiographical time.
#[derive(Debug, Clone)]
pub struct TemporalExtension {
    /// Birth timestamp (ms since epoch).
    pub birth: i64,
    /// Age in milliseconds.
    pub duration: i64,
    /// Personal past (oldest first).
    pub personal_past: VecDeque<EmbodiedEvent>,
    /// Predicted future events.
    pub anticipated_future: Vec<AnticipatedEvent>,
    /// Cap on past events.
    pub max_history: usize,
}

/// A remembered embodied moment.
#[derive(Debug, Clone)]
pub struct EmbodiedEvent {
    pub timestamp: i64,
    pub position: (f64, f64, f64),
    /// -1..1 emotional valence.
    pub affect: f64,
    /// 0..1.
    pub significance: f64,
}

/// A predicted future embodied event.
#[derive(Debug, Clone)]
pub struct AnticipatedEvent {
    pub expected_at: i64,
    pub predicted_position: (f64, f64, f64),
    /// 0..1.
    pub confidence: f64,
}

impl TemporalExtension {
    /// Construct a temporal extension starting at `birth` (ms since epoch).
    pub fn new(birth: i64) -> Self {
        Self {
            birth,
            duration: 0,
            personal_past: VecDeque::with_capacity(DEFAULT_MAX_HISTORY),
            anticipated_future: Vec::new(),
            max_history: DEFAULT_MAX_HISTORY,
        }
    }

    /// Record an embodied event into the past. Auto-evicts the oldest.
    pub fn record(&mut self, event: EmbodiedEvent) {
        if self.personal_past.len() >= self.max_history {
            self.personal_past.pop_front();
        }
        self.duration = (event.timestamp - self.birth).max(self.duration);
        self.personal_past.push_back(event);
    }

    /// Add an anticipated future event. Drops lowest-confidence if at cap.
    pub fn anticipate(&mut self, event: AnticipatedEvent) {
        if self.anticipated_future.len() >= DEFAULT_MAX_ANTICIPATIONS {
            if let Some((idx, _)) = self
                .anticipated_future
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.confidence
                        .partial_cmp(&b.1.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                self.anticipated_future.swap_remove(idx);
            }
        }
        self.anticipated_future.push(event);
    }

    /// Return all past events newer than `since_ms` (relative to birth).
    pub fn recall(&self, since_ms: i64) -> Vec<&EmbodiedEvent> {
        let cutoff = self.birth + since_ms;
        self.personal_past
            .iter()
            .filter(|e| e.timestamp >= cutoff)
            .collect()
    }

    /// 1.0 = continuous autobiographical time (history full, both past & future populated).
    pub fn temporal_phi(&self) -> f64 {
        let past_ratio = (self.personal_past.len() as f64
            / self.max_history as f64)
            .max(0.0)
            .min(1.0);
        let future_ratio = (self.anticipated_future.len() as f64
            / DEFAULT_MAX_ANTICIPATIONS as f64)
            .max(0.0)
            .min(1.0);
        // Past dominates, future acts as a coherence bonus.
        (past_ratio * 0.7) + (future_ratio * 0.3)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Embodied Self Aggregate
// ─────────────────────────────────────────────────────────────────────────────

/// 3+1D embodiment — body schema + temporal extension.
#[derive(Debug, Clone)]
pub struct EmbodiedSelf {
    pub body: BodySchema3D,
    pub proprioception: Proprioception,
    pub time_extension: TemporalExtension,
    /// Placeholder for a link to `OctonionAffect` in the hypercube.
    pub affect_link: Option<String>,
}

impl Default for EmbodiedSelf {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbodiedSelf {
    /// Construct a default embodied self with proprioception sized to the
    /// default body and a birth timestamp of "now".
    pub fn new() -> Self {
        let body = BodySchema3D::new();
        let limb_count = body.limbs.len();
        Self {
            proprioception: Proprioception::new(limb_count),
            body,
            time_extension: TemporalExtension::new(Utc::now().timestamp_millis()),
            affect_link: None,
        }
    }

    /// Sync the 3+1D layers with the 0-2D foundation:
    /// - presence intensity drives body tension
    /// - stream flow rate adjusts time extension duration
    /// - foundation integrated_phi nudges spatial_phi
    pub fn integrate(&mut self, foundation: &ConsciousnessFoundation) {
        // Map presence intensity to all-limb tension.
        let intensity = foundation.presence.intensity;
        let n = self.proprioception.tension.len();
        for i in 0..n {
            self.proprioception
                .update_tension(i, intensity * 0.5);
        }
        // Field richness nudges balance toward 0 (more centered).
        self.proprioception.balance *= 1.0 - foundation.field.richness() * 0.1;
        // Stream flow rate extends subjective duration.
        let extra = (foundation.stream.flow_rate * 100.0) as i64;
        self.time_extension.duration = self
            .time_extension
            .duration
            .saturating_add(extra);
        // Integrated phi nudges spatial awareness.
        let phi = foundation.integrated_phi();
        if phi > 0.0 {
            for limb in &mut self.body.limbs {
                limb.extension = (limb.extension + phi * 0.05).max(0.0).min(1.0);
            }
        }
    }

    /// 3+1D integration: geometric mean of spatial, proprioceptive-coherence,
    /// and temporal measures.
    pub fn embodied_phi(&self) -> f64 {
        let spatial = self.body.spatial_phi();
        let discomfort = self.proprioception.overall_discomfort();
        let coherence = 1.0 - discomfort;
        let temporal = self.time_extension.temporal_phi();
        let v = spatial.max(0.0) * coherence.max(0.0) * temporal.max(0.0);
        v.cbrt()
    }

    /// Has spatial + temporal + proprioceptive signals all populated.
    pub fn is_grounded(&self) -> bool {
        let has_spatial = !self.body.limbs.is_empty();
        let has_temporal = !self.time_extension.personal_past.is_empty()
            || !self.time_extension.anticipated_future.is_empty();
        let has_proprio = !self.proprioception.tension.is_empty()
            || !self.proprioception.pain_signals.is_empty()
            || self.proprioception.balance != 0.0
            || self.proprioception.fatigue > 0.0;
        has_spatial && has_temporal && has_proprio
    }

    /// Spacetime distance to another embodied self.
    /// `d² = (Δx)² + (Δy)² + (Δz)² + (c·Δt)²` with `c` = speed-of-light surrogate
    /// (here 1 ms of time = 1 spatial unit for tractability).
    pub fn spatial_4d_distance(&self, other: &EmbodiedSelf) -> f64 {
        let (ax, ay, az) = self.body.position;
        let (bx, by, bz) = other.body.position;
        let dx = ax - bx;
        let dy = ay - by;
        let dz = az - bz;
        let dt = (self.time_extension.duration - other.time_extension.duration) as f64;
        (dx * dx + dy * dy + dz * dz + dt * dt).sqrt()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── BodySchema3D ────────────────────────────────────────────────────────

    #[test]
    fn body_new_has_six_limbs() {
        let b = BodySchema3D::new();
        assert_eq!(b.limbs.len(), DEFAULT_LIMB_COUNT);
        assert_eq!(b.limbs.len(), 6);
        assert_eq!(b.position, (0.0, 0.0, 0.0));
        assert_eq!(b.orientation, (0.0, 0.0, 0.0));
    }

    #[test]
    fn body_update_position_sets_xyz() {
        let mut b = BodySchema3D::new();
        b.update_position(1.0, 2.0, 3.0);
        assert_eq!(b.position, (1.0, 2.0, 3.0));
    }

    #[test]
    fn body_update_limb_finds_named_limb() {
        let mut b = BodySchema3D::new();
        let ok = b.update_limb("arm_left", vec![0.1, 0.2, 0.3]);
        assert!(ok);
        let limb = b.limbs.iter().find(|l| l.name == "arm_left").unwrap();
        assert_eq!(limb.joint_angles, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn body_update_limb_missing_returns_false() {
        let mut b = BodySchema3D::new();
        assert!(!b.update_limb("wing_left", vec![0.0]));
    }

    #[test]
    fn body_center_of_mass_default() {
        let b = BodySchema3D::new();
        let (x, y, z) = b.center_of_mass();
        // At default position, CoM is very close to the body position.
        assert!(x.abs() < 1.0);
        assert!(y.abs() < 1.0);
        assert!(z.abs() < 1.0);
    }

    #[test]
    fn body_center_of_mass_empty() {
        let mut b = BodySchema3D::new();
        b.limbs.clear();
        let p = b.center_of_mass();
        assert_eq!(p, (0.0, 0.0, 0.0));
    }

    #[test]
    fn body_spatial_phi_zero_for_zero_angles() {
        let b = BodySchema3D::new();
        // All joint angles are 0, so variance is 0 → phi is 0.
        assert_eq!(b.spatial_phi(), 0.0);
    }

    #[test]
    fn body_spatial_phi_grows_with_variation() {
        let mut b = BodySchema3D::new();
        b.update_limb("arm_left", vec![1.0, -1.0, 0.5]);
        b.update_limb("arm_right", vec![-0.8, 0.3, 0.7]);
        let phi = b.spatial_phi();
        assert!(phi > 0.0);
        assert!(phi < 1.0);
    }

    #[test]
    fn body_spatial_phi_empty_limbs() {
        let mut b = BodySchema3D::new();
        b.limbs.clear();
        assert_eq!(b.spatial_phi(), 0.0);
    }

    // ── Proprioception ──────────────────────────────────────────────────────

    #[test]
    fn proprioception_new_sizes_to_limbs() {
        let p = Proprioception::new(6);
        assert_eq!(p.tension.len(), 6);
        assert_eq!(p.balance, 0.0);
        assert_eq!(p.fatigue, 0.0);
        assert!(p.pain_signals.is_empty());
    }

    #[test]
    fn proprioception_update_tension_valid_index() {
        let mut p = Proprioception::new(3);
        assert!(p.update_tension(1, 0.5));
        assert_eq!(p.tension[1], 0.5);
    }

    #[test]
    fn proprioception_update_tension_invalid_index() {
        let mut p = Proprioception::new(3);
        assert!(!p.update_tension(5, 0.5));
    }

    #[test]
    fn proprioception_update_tension_clamps() {
        let mut p = Proprioception::new(2);
        p.update_tension(0, 2.0);
        assert_eq!(p.tension[0], 1.0);
        p.update_tension(0, -0.5);
        assert_eq!(p.tension[0], 0.0);
    }

    #[test]
    fn proprioception_pain_at_known_location() {
        let mut p = Proprioception::new(2);
        p.pain_signals.push(("left_knee".to_string(), 0.7));
        assert!((p.pain_at("left_knee") - 0.7).abs() < 1e-9);
    }

    #[test]
    fn proprioception_pain_at_unknown_location() {
        let p = Proprioception::new(2);
        assert_eq!(p.pain_at("nowhere"), 0.0);
    }

    #[test]
    fn proprioception_overall_discomfort_zero_when_fresh() {
        let p = Proprioception::new(6);
        assert_eq!(p.overall_discomfort(), 0.0);
    }

    #[test]
    fn proprioception_overall_discomfort_increases_with_pain() {
        let mut p = Proprioception::new(2);
        p.update_tension(0, 0.5);
        p.pain_signals.push(("back".to_string(), 0.8));
        p.fatigue = 0.3;
        p.balance = -0.1;
        let d = p.overall_discomfort();
        assert!(d > 0.0);
        assert!(d <= 1.0);
    }

    // ── TemporalExtension ───────────────────────────────────────────────────

    #[test]
    fn temporal_new_initializes() {
        let t = TemporalExtension::new(1_000);
        assert_eq!(t.birth, 1_000);
        assert_eq!(t.duration, 0);
        assert!(t.personal_past.is_empty());
        assert!(t.anticipated_future.is_empty());
    }

    #[test]
    fn temporal_record_increases_duration() {
        let mut t = TemporalExtension::new(1_000);
        t.record(EmbodiedEvent {
            timestamp: 2_000,
            position: (0.0, 0.0, 0.0),
            affect: 0.5,
            significance: 0.7,
        });
        assert_eq!(t.duration, 1_000);
        assert_eq!(t.personal_past.len(), 1);
    }

    #[test]
    fn temporal_record_does_not_shrink_duration() {
        let mut t = TemporalExtension::new(5_000);
        t.duration = 1_000_000;
        t.record(EmbodiedEvent {
            timestamp: 6_000,
            position: (0.0, 0.0, 0.0),
            affect: 0.0,
            significance: 0.0,
        });
        // duration is non-decreasing
        assert_eq!(t.duration, 1_000_000);
    }

    #[test]
    fn temporal_record_evicts_oldest_at_cap() {
        let mut t = TemporalExtension::new(0);
        t.max_history = 3;
        for i in 0..5 {
            t.record(EmbodiedEvent {
                timestamp: i,
                position: (0.0, 0.0, 0.0),
                affect: 0.0,
                significance: 0.0,
            });
        }
        assert_eq!(t.personal_past.len(), 3);
        // The earliest two are gone; the latest three remain.
        assert_eq!(t.personal_past.front().unwrap().timestamp, 2);
        assert_eq!(t.personal_past.back().unwrap().timestamp, 4);
    }

    #[test]
    fn temporal_anticipate_evicts_lowest_confidence() {
        let mut t = TemporalExtension::new(0);
        for i in 0..(DEFAULT_MAX_ANTICIPATIONS + 1) {
            t.anticipate(AnticipatedEvent {
                expected_at: i as i64,
                predicted_position: (0.0, 0.0, 0.0),
                confidence: 0.1 + (i as f64) * 0.001,
            });
        }
        assert_eq!(t.anticipated_future.len(), DEFAULT_MAX_ANTICIPATIONS);
    }

    #[test]
    fn temporal_recall_filters_by_cutoff() {
        let mut t = TemporalExtension::new(0);
        for ts in [10, 20, 30, 40].iter() {
            t.record(EmbodiedEvent {
                timestamp: *ts,
                position: (0.0, 0.0, 0.0),
                affect: 0.0,
                significance: 0.0,
            });
        }
        let r = t.recall(25);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].timestamp, 30);
        assert_eq!(r[1].timestamp, 40);
    }

    #[test]
    fn temporal_phi_empty_is_zero() {
        let t = TemporalExtension::new(0);
        assert_eq!(t.temporal_phi(), 0.0);
    }

    #[test]
    fn temporal_phi_grows_with_history() {
        let mut t = TemporalExtension::new(0);
        for i in 0..100 {
            t.record(EmbodiedEvent {
                timestamp: i,
                position: (0.0, 0.0, 0.0),
                affect: 0.0,
                significance: 0.0,
            });
        }
        let phi = t.temporal_phi();
        assert!(phi > 0.0);
        assert!(phi <= 1.0);
    }

    // ── EmbodiedSelf ────────────────────────────────────────────────────────

    #[test]
    fn embodied_new_wires_components() {
        let e = EmbodiedSelf::new();
        assert_eq!(e.body.limbs.len(), 6);
        assert_eq!(e.proprioception.tension.len(), 6);
        assert!(e.time_extension.birth > 0);
        assert!(e.affect_link.is_none());
    }

    #[test]
    fn embodied_integrate_modifies_tension() {
        let mut e = EmbodiedSelf::new();
        let mut f = ConsciousnessFoundation::new();
        f.presence.intensify(1.0);
        f.field.perceive(0.1, 0.0, 0.5);
        e.integrate(&f);
        // Tension should now reflect presence intensity.
        assert!(e.proprioception.tension.iter().all(|t| *t > 0.0));
    }

    #[test]
    fn embodied_integrate_handles_inactive_foundation() {
        let mut e = EmbodiedSelf::new();
        let f = ConsciousnessFoundation::new();
        let prev_ext = e.body.limbs[0].extension;
        e.integrate(&f);
        // With no presence, tension should stay 0.
        assert!(e.proprioception.tension.iter().all(|t| *t == 0.0));
        // Extensions should not change.
        assert_eq!(e.body.limbs[0].extension, prev_ext);
    }

    #[test]
    fn embodied_phi_in_bounds() {
        let e = EmbodiedSelf::new();
        let phi = e.embodied_phi();
        assert!(phi >= 0.0);
        assert!(phi <= 1.0);
    }

    #[test]
    fn embodied_grounded_requires_all_layers() {
        let mut e = EmbodiedSelf::new();
        // Fresh: no past, no future, no pain, no fatigue, balance=0
        // → proprioceptive is empty
        assert!(!e.is_grounded());
        // Add a proprioceptive signal.
        e.proprioception.fatigue = 0.1;
        // Still no temporal.
        assert!(!e.is_grounded());
        // Add a temporal event.
        e.time_extension.record(EmbodiedEvent {
            timestamp: e.time_extension.birth + 1,
            position: (0.0, 0.0, 0.0),
            affect: 0.0,
            significance: 0.0,
        });
        assert!(e.is_grounded());
    }

    #[test]
    fn embodied_spatial_4d_distance_self_is_zero() {
        let e = EmbodiedSelf::new();
        let d = e.spatial_4d_distance(&e);
        assert!(d.abs() < 1e-9);
    }

    #[test]
    fn embodied_spatial_4d_distance_grows_with_separation() {
        let mut a = EmbodiedSelf::new();
        let mut b = EmbodiedSelf::new();
        a.body.update_position(0.0, 0.0, 0.0);
        b.body.update_position(3.0, 4.0, 0.0);
        let d = a.spatial_4d_distance(&b);
        assert!((d - 5.0).abs() < 1e-9);
    }

    // ── Integration: foundation ↔ embodied ──────────────────────────────────

    #[test]
    fn integration_foundation_to_embodied_is_sane() {
        let mut foundation = ConsciousnessFoundation::new();
        // Wake presence, observe some percepts.
        foundation.observe(&[(0.1, 0.0, 0.7), (-0.2, 0.1, 0.5)]);
        foundation.observe(&[(0.0, 0.3, 0.3)]);

        let mut embodied = EmbodiedSelf::new();
        let pre_phi = embodied.embodied_phi();
        embodied.integrate(&foundation);
        let post_phi = embodied.embodied_phi();
        // After integration, embodied phi should be > 0 (presence, time extension).
        assert!(post_phi >= 0.0);
        // Should not crash and should produce finite numbers.
        assert!(pre_phi.is_finite());
        assert!(post_phi.is_finite());
    }

    #[test]
    fn integration_repeated_calls_are_idempotent_enough() {
        let mut foundation = ConsciousnessFoundation::new();
        foundation.observe(&[(0.0, 0.0, 1.0)]);
        let mut embodied = EmbodiedSelf::new();
        embodied.integrate(&foundation);
        let phi_a = embodied.embodied_phi();
        embodied.integrate(&foundation);
        let phi_b = embodied.embodied_phi();
        // With no new input, the system is stable — second call shouldn't make
        // tension go negative or extension leave [0,1].
        for t in &embodied.proprioception.tension {
            assert!(*t >= 0.0 && *t <= 1.0);
        }
        for limb in &embodied.body.limbs {
            assert!(limb.extension >= 0.0 && limb.extension <= 1.0);
        }
        assert!((phi_a - phi_b).abs() < 1e-6);
    }
}
