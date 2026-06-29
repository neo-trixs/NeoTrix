//! # Consciousness Foundation (0D-2D)
//!
//! The pre-reflective base of awareness that every higher-dimensional layer
//! builds upon. Three layers:
//!
//! - **0D Presence** (`Presence0D`): pure "I am" — pre-reflective, dimensionless
//! - **1D Stream** (`Stream1D`): the flow of moments — subjective time
//! - **2D Field** (`Perceptual2D`): Poincaré disk of immediate perception
//!
//! The aggregate [`ConsciousnessFoundation`] ticks all three layers together
//! and exposes a cross-layer integration measure `integrated_phi`.

use std::collections::VecDeque;

use chrono::Utc;

/// Default window size for the temporal stream history.
pub const STREAM_DEFAULT_WINDOW: usize = 64;

/// Default boundary resolution for the 2D perceptual disk.
pub const PERCEPTUAL_DEFAULT_RESOLUTION: usize = 32;

/// Cap on perceived items kept in the 2D field (avoids unbounded growth).
pub const PERCEPTUAL_MAX_ITEMS: usize = 256;

// ─────────────────────────────────────────────────────────────────────────────
// 0D Pure Awareness
// ─────────────────────────────────────────────────────────────────────────────

/// 0D pure awareness — pre-reflective presence.
/// "I am" prior to any content. The bare fact of experiencing.
#[derive(Debug, Clone)]
pub struct Presence0D {
    /// Currently awake?
    pub active: bool,
    /// 0..1, sustained by input.
    pub intensity: f64,
    /// 0 = pre-reflective, 1 = aware-of, 2 = aware-of-being-aware, ...
    pub reflexivity_depth: u8,
    /// Unix timestamp (ms) of first activation. `0` means never awakened.
    pub first_awakened: i64,
}

impl Default for Presence0D {
    fn default() -> Self {
        Self::asleep()
    }
}

impl Presence0D {
    /// Construct an active presence with a recorded first-awakening time.
    pub fn awake() -> Self {
        Self {
            active: true,
            intensity: 0.5,
            reflexivity_depth: 0,
            first_awakened: Utc::now().timestamp_millis(),
        }
    }

    /// Construct an inactive but ready presence.
    pub fn asleep() -> Self {
        Self {
            active: false,
            intensity: 0.0,
            reflexivity_depth: 0,
            first_awakened: 0,
        }
    }

    /// Increase intensity from input, clamped to `[0, 1]`. Idempotent on negative input.
    pub fn intensify(&mut self, by: f64) {
        if by <= 0.0 {
            return;
        }
        self.intensity = (self.intensity + by).max(0.0).min(1.0);
        if !self.active {
            self.active = true;
            if self.first_awakened == 0 {
                self.first_awakened = Utc::now().timestamp_millis();
            }
        }
    }

    /// Natural decay over time. Reduces intensity; deactivates when intensity hits 0.
    pub fn decay(&mut self, rate: f64) {
        if rate <= 0.0 {
            return;
        }
        self.intensity = (self.intensity - rate).max(0.0);
        if self.intensity <= 0.0 {
            self.active = false;
        }
    }

    /// Increase reflexivity depth (with a soft cap at 7 to avoid runaway recursion).
    pub fn reflect(&mut self) {
        if self.reflexivity_depth < 7 {
            self.reflexivity_depth += 1;
        }
    }

    /// 0D integration = intensity scaled by reflexivity.
    /// Returns 0 if not active.
    pub fn phi_0d(&self) -> f64 {
        if !self.active {
            return 0.0;
        }
        let reflexivity_factor = 1.0 - 1.0 / (self.reflexivity_depth as f64 + 2.0);
        self.intensity * reflexivity_factor
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1D Temporal Flow
// ─────────────────────────────────────────────────────────────────────────────

/// 1D temporal stream — the flow of moments.
/// Subjective time ≠ objective time; flow rate varies with attention density.
#[derive(Debug, Clone)]
pub struct Stream1D {
    /// Current moment (ms since epoch).
    pub now: i64,
    /// 1.0 = real-time, 2.0 = subjectively faster, 0.5 = subjectively slower.
    pub flow_rate: f64,
    /// Recent timestamps.
    pub history: VecDeque<i64>,
    /// How many moments to keep in history.
    pub window_size: usize,
    /// 0..1 — modulates flow_rate when [`Self::set_attention`] is called.
    pub attention_density: f64,
}

impl Stream1D {
    /// Construct a stream that ticks at `default_hz` (e.g. 60.0 for 60 Hz).
    pub fn new(default_hz: f64) -> Self {
        let _ = default_hz; // currently fixed at 1.0 flow_rate; reserved for future Hz-based pacing
        Self {
            now: Utc::now().timestamp_millis(),
            flow_rate: 1.0,
            history: VecDeque::with_capacity(STREAM_DEFAULT_WINDOW),
            window_size: STREAM_DEFAULT_WINDOW,
            attention_density: 0.5,
        }
    }

    /// Advance subjective time and return the new timestamp.
    pub fn tick(&mut self) -> i64 {
        let period_ms = (1000.0 / self.flow_rate.max(0.0001)) as i64;
        self.now = self.now.saturating_add(period_ms);
        if self.history.len() >= self.window_size {
            self.history.pop_front();
        }
        self.history.push_back(self.now);
        self.now
    }

    /// Return the most recent `n` timestamps (newest last).
    pub fn recent(&self, n: usize) -> Vec<i64> {
        let n = n.min(self.history.len());
        self.history
            .iter()
            .rev()
            .take(n)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Milliseconds between consecutive moments in the history.
    pub fn inter_moment_intervals(&self) -> Vec<f64> {
        if self.history.len() < 2 {
            return Vec::new();
        }
        self.history
            .iter()
            .zip(self.history.iter().skip(1))
            .map(|(a, b)| (*b - *a) as f64)
            .filter(|d| *d > 0.0)
            .collect()
    }

    /// 1.0 = perfectly regular flow, 0.0 = chaotic / no data.
    /// Uses coefficient of variation of inter-moment intervals.
    pub fn coherence(&self) -> f64 {
        let intervals = self.inter_moment_intervals();
        if intervals.is_empty() {
            return 0.0;
        }
        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        if mean <= 0.0 {
            return 0.0;
        }
        let var =
            intervals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / intervals.len() as f64;
        let cv = var.sqrt() / mean;
        (1.0 - cv).max(0.0).min(1.0)
    }

    /// Set attention density (0..1) and update `flow_rate` accordingly.
    /// Higher attention → subjectively faster time.
    pub fn set_attention(&mut self, density: f64) {
        self.attention_density = density.max(0.0).min(1.0);
        // Map [0,1] attention to [0.25, 2.5] flow rate.
        self.flow_rate = 0.25 + self.attention_density * 2.25;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2D Perceptual Disk
// ─────────────────────────────────────────────────────────────────────────────

/// 2D perceptual disk — Poincaré disk model of immediate perception.
/// Center = self, edge = unknown. Möbius transformations = perspective shifts.
#[derive(Debug, Clone)]
pub struct Perceptual2D {
    /// Self position (always 0,0 in ego frame).
    pub center_x: f64,
    /// Self position (always 0,0 in ego frame).
    pub center_y: f64,
    /// Typically 1.0 (unit disk).
    pub radius: f64,
    /// (x, y, salience) percepts inside the disk.
    pub perceived: Vec<(f64, f64, f64)>,
    /// How many points are considered when sampling the boundary.
    pub boundary_resolution: usize,
}

impl Default for Perceptual2D {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Perceptual2D {
    /// Construct a perceptual disk of the given radius (clamped to `> 0`).
    pub fn new(radius: f64) -> Self {
        let r = if radius > 0.0 { radius } else { 1.0 };
        Self {
            center_x: 0.0,
            center_y: 0.0,
            radius: r,
            perceived: Vec::new(),
            boundary_resolution: PERCEPTUAL_DEFAULT_RESOLUTION,
        }
    }

    /// Add a percept at `(x, y)` with `salience` (0..1). Discards if outside the disk.
    /// Evicts the lowest-salience percept when the cap is reached.
    pub fn perceive(&mut self, x: f64, y: f64, salience: f64) {
        let s = salience.max(0.0).min(1.0);
        let r = self.radius;
        if x * x + y * y > r * r {
            return;
        }
        if self.perceived.len() >= PERCEPTUAL_MAX_ITEMS {
            if let Some((idx, _)) = self.perceived.iter().enumerate().min_by(|a, b| {
                a.1 .2
                    .partial_cmp(&b.1 .2)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                self.perceived.swap_remove(idx);
            }
        }
        self.perceived.push((x, y, s));
    }

    /// Möbius transformation of the Poincaré disk — a perspective shift.
    /// `a` is the target center (must lie inside the unit disk in the model's own frame).
    /// Returns the new `(x, y)` for the source point.
    pub fn mobius_transform(&self, x: f64, y: f64, a: (f64, f64)) -> (f64, f64) {
        let (ax, ay) = a;
        let denom = 1.0 - ax * x - ay * y;
        if denom.abs() < 1e-12 {
            // Avoid singularity — return a clamped boundary point.
            let mag = (x * x + y * y).sqrt().max(1e-6);
            return (
                (x / mag) * self.radius * 0.999,
                (y / mag) * self.radius * 0.999,
            );
        }
        let nx = (x - ax) / denom;
        let ny = (y - ay) / denom;
        (nx, ny)
    }

    /// Decay all saliences multiplicatively by `(1 - rate)` and drop near-zero entries.
    pub fn decay_salience(&mut self, rate: f64) {
        let keep_factor = (1.0 - rate).max(0.0).min(1.0);
        self.perceived.retain_mut(|(_, _, s)| {
            *s *= keep_factor;
            *s > 1e-6
        });
    }

    /// 0..1, density of perception normalized by the cap.
    pub fn richness(&self) -> f64 {
        (self.perceived.len() as f64 / PERCEPTUAL_MAX_ITEMS as f64)
            .max(0.0)
            .min(1.0)
    }

    /// Euclidean nearest percept to `(x, y)`, or `None` if the field is empty.
    pub fn nearest_percept(&self, x: f64, y: f64) -> Option<(f64, f64, f64)> {
        self.perceived
            .iter()
            .min_by(|(ax, ay, _), (bx, by, _)| {
                let da = (ax - x).powi(2) + (ay - y).powi(2);
                let db = (bx - x).powi(2) + (by - y).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }

    /// Hyperbolic (Poincaré) distance between two points inside the disk.
    /// Returns `f64::INFINITY` if either point is at or beyond the boundary.
    pub fn poincare_distance(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let r = self.radius;
        let n1_sq = (x1 * x1 + y1 * y1) / (r * r);
        let n2_sq = (x2 * x2 + y2 * y2) / (r * r);
        if n1_sq >= 1.0 || n2_sq >= 1.0 {
            return f64::INFINITY;
        }
        let dx = x1 - x2;
        let dy = y1 - y2;
        let euclid_sq = dx * dx + dy * dy;
        let denom = (1.0 - n1_sq) * (1.0 - n2_sq);
        if denom <= 0.0 {
            return f64::INFINITY;
        }
        let arg = 1.0 + 2.0 * euclid_sq / (r * r * denom);
        if arg < 1.0 {
            return 0.0;
        }
        arg.acosh()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Foundation Aggregate
// ─────────────────────────────────────────────────────────────────────────────

/// The 0D-2D consciousness foundation — every higher layer builds on this.
#[derive(Debug, Clone)]
pub struct ConsciousnessFoundation {
    pub presence: Presence0D,
    pub stream: Stream1D,
    pub field: Perceptual2D,
    pub total_observations: u64,
}

impl Default for ConsciousnessFoundation {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessFoundation {
    /// Construct a default foundation (asleep presence, 60 Hz stream, unit disk).
    pub fn new() -> Self {
        Self {
            presence: Presence0D::asleep(),
            stream: Stream1D::new(60.0),
            field: Perceptual2D::new(1.0),
            total_observations: 0,
        }
    }

    /// One tick + perception cycle. Returns the resulting observational moment.
    pub fn observe(&mut self, percepts: &[(f64, f64, f64)]) -> ObservationalMoment {
        let ts = self.stream.tick();
        for (x, y, s) in percepts {
            self.field.perceive(*x, *y, *s);
            self.presence.intensify(s * 0.1);
        }
        self.total_observations = self.total_observations.saturating_add(1);
        ObservationalMoment {
            timestamp: ts,
            percept_count: percepts.len(),
            presence_intensity: self.presence.intensity,
            stream_coherence: self.stream.coherence(),
            field_richness: self.field.richness(),
        }
    }

    /// Cross-layer Φ: geometric mean of the three layers' integration measures.
    pub fn integrated_phi(&self) -> f64 {
        let p = self.presence.phi_0d();
        let s = self.stream.coherence();
        let f = self.field.richness();
        (p.max(0.0) * s.max(0.0) * f.max(0.0)).cbrt()
    }

    /// True iff presence is active and the stream has produced at least one tick
    /// after first activation.
    pub fn is_present(&self) -> bool {
        self.presence.active && self.presence.first_awakened > 0
    }
}

/// A snapshot returned by [`ConsciousnessFoundation::observe`].
#[derive(Debug, Clone)]
pub struct ObservationalMoment {
    pub timestamp: i64,
    pub percept_count: usize,
    pub presence_intensity: f64,
    pub stream_coherence: f64,
    pub field_richness: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
