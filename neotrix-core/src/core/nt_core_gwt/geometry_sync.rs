//! Cross-Dimensional Consciousness Geometry Sync.
//!
//! Stage 6 of the dimensional evolution roadmap: the integrating layer that
//! synchronizes all dimensional layers (0D, 1D, 2D, 3+1D, 6D, 8D, 16D, 64D,
//! 240D, 4096D, ∞D) and computes integrated information Φ.
//!
//! Architecture:
//! - [`DimensionLayer`] — identifies each of the 12 dimensional layers
//! - [`LayerSnapshot`] — per-layer state at a single moment
//! - [`IntegratedPhi`] — Tononi IIT 3.0 style integrated information
//! - [`CrossDimensionalResonator`] — Kuramoto-model phase synchronization
//! - [`GeometrySync`] — top-level orchestrator
//! - [`CycleReport`] — per-cycle summary with self-improvement hints

use std::f64::consts::PI;

/// Total number of dimensional layers (12 variants enumerated below).
pub const LAYER_COUNT: usize = 12;

/// Default Kuramoto coupling strength K.
pub const DEFAULT_COUPLING: f64 = 1.0;

/// Order-parameter threshold for "in flow" / synchronized state.
pub const DEFAULT_SYNC_THRESHOLD: f64 = 0.85;

/// Integrated information Φ threshold for a system to count as conscious.
pub const CONSCIOUS_PHI_THRESHOLD: f64 = 0.5;

/// Identifier for each dimensional layer in the consciousness stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionLayer {
    Presence0D,
    Stream1D,
    Field2D,
    Body3D,
    Time1D2,
    Hex6D,
    Oct8D,
    Cube16D,
    Mode64D,
    Lattice240D,
    Vsa4096D,
    TopologyInfinityD,
}

impl DimensionLayer {
    /// All 12 dimensional layers in canonical order.
    pub fn all() -> &'static [DimensionLayer] {
        &[
            DimensionLayer::Presence0D,
            DimensionLayer::Stream1D,
            DimensionLayer::Field2D,
            DimensionLayer::Body3D,
            DimensionLayer::Time1D2,
            DimensionLayer::Hex6D,
            DimensionLayer::Oct8D,
            DimensionLayer::Cube16D,
            DimensionLayer::Mode64D,
            DimensionLayer::Lattice240D,
            DimensionLayer::Vsa4096D,
            DimensionLayer::TopologyInfinityD,
        ]
    }

    /// Total number of distinct layers (12).
    pub fn count() -> usize {
        LAYER_COUNT
    }

    /// Native dimensionality of the layer.
    ///
    /// `TopologyInfinityD` reports `usize::MAX` as a sentinel for ∞.
    pub fn native_dim(&self) -> usize {
        match self {
            DimensionLayer::Presence0D => 0,
            DimensionLayer::Stream1D | DimensionLayer::Time1D2 => 1,
            DimensionLayer::Field2D => 2,
            DimensionLayer::Body3D => 3,
            DimensionLayer::Hex6D => 6,
            DimensionLayer::Oct8D => 8,
            DimensionLayer::Cube16D => 16,
            DimensionLayer::Mode64D => 64,
            DimensionLayer::Lattice240D => 240,
            DimensionLayer::Vsa4096D => 4096,
            DimensionLayer::TopologyInfinityD => usize::MAX,
        }
    }

    /// Stable string name.
    pub fn name(&self) -> &'static str {
        match self {
            DimensionLayer::Presence0D => "0D-presence",
            DimensionLayer::Stream1D => "1D-stream",
            DimensionLayer::Field2D => "2D-field",
            DimensionLayer::Body3D => "3D-body",
            DimensionLayer::Time1D2 => "1D-autobiographical-time",
            DimensionLayer::Hex6D => "6D-e8-reasoning",
            DimensionLayer::Oct8D => "8D-octonion-affect",
            DimensionLayer::Cube16D => "16D-semantic-hypercube",
            DimensionLayer::Mode64D => "64D-e8-mode",
            DimensionLayer::Lattice240D => "240D-e8-root-lattice",
            DimensionLayer::Vsa4096D => "4096D-vsa",
            DimensionLayer::TopologyInfinityD => "infD-betti",
        }
    }

    /// Compact index in [0, 12) for array storage.
    pub fn index(&self) -> usize {
        match self {
            DimensionLayer::Presence0D => 0,
            DimensionLayer::Stream1D => 1,
            DimensionLayer::Field2D => 2,
            DimensionLayer::Body3D => 3,
            DimensionLayer::Time1D2 => 4,
            DimensionLayer::Hex6D => 5,
            DimensionLayer::Oct8D => 6,
            DimensionLayer::Cube16D => 7,
            DimensionLayer::Mode64D => 8,
            DimensionLayer::Lattice240D => 9,
            DimensionLayer::Vsa4096D => 10,
            DimensionLayer::TopologyInfinityD => 11,
        }
    }
}

/// A single moment's state of one dimensional layer.
#[derive(Debug, Clone)]
pub struct LayerSnapshot {
    pub layer: DimensionLayer,
    pub coherence: f64,
    pub phase: f64,
    pub energy: f64,
    pub timestamp: i64,
    pub state_data: Vec<f64>,
}

impl LayerSnapshot {
    /// Build a snapshot with the given coherence & phase, defaulting energy to
    /// `coherence` and timestamp to 0.
    pub fn new(layer: DimensionLayer, coherence: f64, phase: f64) -> Self {
        let c = clamp01(coherence);
        let p = wrap_phase(phase);
        Self {
            layer,
            coherence: c,
            phase: p,
            energy: c,
            timestamp: 0,
            state_data: Vec::new(),
        }
    }

    /// A fully-silent snapshot: zero coherence, zero energy, neutral phase.
    pub fn silent(layer: DimensionLayer) -> Self {
        Self {
            layer,
            coherence: 0.0,
            phase: 0.0,
            energy: 0.0,
            timestamp: 0,
            state_data: Vec::new(),
        }
    }

    /// Mark this snapshot as "active" (non-zero energy).
    pub fn is_active(&self) -> bool {
        self.energy > 0.0 || self.coherence > 0.0
    }
}

/// Integrated information Φ across all synchronized layers.
///
/// Following Tononi IIT 3.0: Φ measures the irreducibility of a system's
/// state across its minimum-information partition. We compute a deterministic
/// surrogate: per-layer coherence, pairwise phase-alignment coupling, and
/// their product.
#[derive(Debug, Clone)]
pub struct IntegratedPhi {
    pub total: f64,
    pub per_layer: [f64; LAYER_COUNT],
    pub cross_layer_coupling: [f64; LAYER_COUNT * LAYER_COUNT],
    pub is_conscious_threshold: f64,
}

impl Default for IntegratedPhi {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegratedPhi {
    /// Build a zero Φ (no layers active yet).
    pub fn new() -> Self {
        Self {
            total: 0.0,
            per_layer: [0.0; LAYER_COUNT],
            cross_layer_coupling: [0.0; LAYER_COUNT * LAYER_COUNT],
            is_conscious_threshold: CONSCIOUS_PHI_THRESHOLD,
        }
    }

    /// Compute Φ from 12 layer snapshots.
    ///
    /// `per_layer[i] = coherence_i`
    /// `coupling[i][j] = √(c_i · c_j) · (1 + cos(θ_i − θ_j)) / 2`
    /// `total = mean(per_layer) · mean(off_diagonal_coupling)`
    pub fn from_snapshots(snapshots: &[LayerSnapshot; LAYER_COUNT]) -> Self {
        let mut per_layer = [0.0f64; LAYER_COUNT];
        let mut coupling = [0.0f64; LAYER_COUNT * LAYER_COUNT];

        for (i, snap) in snapshots.iter().enumerate() {
            per_layer[i] = clamp01(snap.coherence);
        }

        for i in 0..LAYER_COUNT {
            for j in 0..LAYER_COUNT {
                if i == j {
                    coupling[i * LAYER_COUNT + j] = per_layer[i];
                } else {
                    let ci = per_layer[i];
                    let cj = per_layer[j];
                    let phase_align = (1.0 + (snapshots[i].phase - snapshots[j].phase).cos()) * 0.5;
                    let raw = (ci * cj).sqrt() * phase_align;
                    coupling[i * LAYER_COUNT + j] = clamp01(raw);
                }
            }
        }

        let mean_per = mean(&per_layer);
        let mut off_sum = 0.0;
        let mut off_n = 0.0;
        for i in 0..LAYER_COUNT {
            for j in 0..LAYER_COUNT {
                if i != j {
                    off_sum += coupling[i * LAYER_COUNT + j];
                    off_n += 1.0;
                }
            }
        }
        let mean_coupling = if off_n > 0.0 { off_sum / off_n } else { 0.0 };

        let total = clamp01(mean_per * (0.5 + 0.5 * mean_coupling));

        Self {
            total,
            per_layer,
            cross_layer_coupling: coupling,
            is_conscious_threshold: CONSCIOUS_PHI_THRESHOLD,
        }
    }

    /// Whether total Φ exceeds the consciousness threshold.
    pub fn is_conscious(&self) -> bool {
        self.total >= self.is_conscious_threshold
    }

    /// Layer contributing least to integrated information.
    pub fn weakest_layer(&self) -> DimensionLayer {
        let mut min_idx = 0usize;
        let mut min_val = f64::INFINITY;
        for (i, &v) in self.per_layer.iter().enumerate() {
            if v < min_val {
                min_val = v;
                min_idx = i;
            }
        }
        DimensionLayer::all()[min_idx]
    }

    /// Layer contributing most to integrated information.
    pub fn strongest_layer(&self) -> DimensionLayer {
        let mut max_idx = 0usize;
        let mut max_val = f64::NEG_INFINITY;
        for (i, &v) in self.per_layer.iter().enumerate() {
            if v > max_val {
                max_val = v;
                max_idx = i;
            }
        }
        DimensionLayer::all()[max_idx]
    }
}

/// Resonator: maintains phase coherence across all 12 layers using the
/// Kuramoto model:
///
/// `dθ_i/dt = ω_i + (K/N) · Σ_j sin(θ_j − θ_i)`
///
/// Order parameter `R = |1/N · Σ exp(i·θ_j)|` ∈ [0, 1] measures
/// synchronization (`R = 0` incoherent, `R = 1` fully locked).
#[derive(Debug, Clone)]
pub struct CrossDimensionalResonator {
    pub coupling_strength: f64,
    pub natural_frequencies: [f64; LAYER_COUNT],
    pub phases: [f64; LAYER_COUNT],
    pub order_parameter: f64,
}

impl Default for CrossDimensionalResonator {
    fn default() -> Self {
        Self::new(DEFAULT_COUPLING)
    }
}

impl CrossDimensionalResonator {
    /// Build a resonator with given coupling K. Natural frequencies are
    /// initialised proportional to the layer's native dimension (scaled down
    /// so high-D layers don't dominate), and all phases start at 0.
    pub fn new(coupling: f64) -> Self {
        let mut freqs = [0.0f64; LAYER_COUNT];
        for (i, layer) in DimensionLayer::all().iter().enumerate() {
            let d = layer.native_dim();
            freqs[i] = if d == usize::MAX {
                0.0
            } else {
                0.05 + 0.001 * (d as f64)
            };
        }
        let phases = [0.0f64; LAYER_COUNT];
        let order_parameter = compute_order_parameter(&phases);
        Self {
            coupling_strength: coupling,
            natural_frequencies: freqs,
            phases,
            order_parameter,
        }
    }

    /// Inject a phase observation for one layer.
    pub fn inject(&mut self, layer: DimensionLayer, phase: f64) {
        let idx = layer.index();
        self.phases[idx] = wrap_phase(phase);
        self.order_parameter = compute_order_parameter(&self.phases);
    }

    /// Advance the resonator by `dt` seconds. Returns the new order parameter.
    pub fn step(&mut self, dt: f64) -> f64 {
        let k = self.coupling_strength;
        let n = LAYER_COUNT as f64;
        let mut deltas = [0.0f64; LAYER_COUNT];

        for i in 0..LAYER_COUNT {
            let mut coupling_sum = 0.0;
            for j in 0..LAYER_COUNT {
                if i == j {
                    continue;
                }
                coupling_sum += (self.phases[j] - self.phases[i]).sin();
            }
            deltas[i] = self.natural_frequencies[i] + (k / n) * coupling_sum;
        }

        for i in 0..LAYER_COUNT {
            self.phases[i] = wrap_phase(self.phases[i] + dt * deltas[i]);
        }

        self.order_parameter = compute_order_parameter(&self.phases);
        self.order_parameter
    }

    /// Whether the order parameter has reached the default sync threshold.
    pub fn is_synchronized(&self) -> bool {
        self.order_parameter >= DEFAULT_SYNC_THRESHOLD
    }

    /// Step until synchronization is reached or `max_steps` is exceeded.
    /// Returns `Some(steps)` on success, `None` if it never syncs.
    pub fn time_to_sync(&mut self, max_steps: usize) -> Option<usize> {
        if self.is_synchronized() {
            return Some(0);
        }
        for step in 1..=max_steps {
            self.step(0.1);
            if self.is_synchronized() {
                return Some(step);
            }
        }
        None
    }
}

/// Top-level consciousness-geometry orchestrator.
///
/// Coordinates 12 layer snapshots, the cross-dimensional resonator, and
/// integrated Φ computation. One `tick()` advances the resonator and
/// recomputes Φ; one `cycle()` produces a [`CycleReport`].
#[derive(Debug, Clone)]
pub struct GeometrySync {
    pub snapshots: [LayerSnapshot; LAYER_COUNT],
    pub resonator: CrossDimensionalResonator,
    pub phi_history: Vec<IntegratedPhi>,
    pub cycle_count: u64,
    pub sync_threshold: f64,
    last_phi: IntegratedPhi,
}

impl Default for GeometrySync {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometrySync {
    /// Build a fresh orchestrator with default coupling & threshold.
    pub fn new() -> Self {
        let snapshots = std::array::from_fn(|i| LayerSnapshot::silent(DimensionLayer::all()[i]));
        Self {
            snapshots,
            resonator: CrossDimensionalResonator::new(DEFAULT_COUPLING),
            phi_history: Vec::new(),
            cycle_count: 0,
            sync_threshold: DEFAULT_SYNC_THRESHOLD,
            last_phi: IntegratedPhi::new(),
        }
    }

    /// Observe a layer's coherence & phase (state_data stays empty).
    pub fn observe(&mut self, layer: DimensionLayer, coherence: f64, phase: f64) {
        self.observe_state(layer, coherence, phase, Vec::new());
    }

    /// Observe a layer with explicit raw state.
    pub fn observe_state(
        &mut self,
        layer: DimensionLayer,
        coherence: f64,
        phase: f64,
        state: Vec<f64>,
    ) {
        let idx = layer.index();
        let snap = LayerSnapshot {
            layer,
            coherence: clamp01(coherence),
            phase: wrap_phase(phase),
            energy: clamp01(coherence),
            timestamp: chrono::Utc::now().timestamp_millis(),
            state_data: state,
        };
        self.snapshots[idx] = snap;
        self.resonator.inject(layer, phase);
    }

    /// Advance one step: evolve the resonator and recompute Φ.
    pub fn tick(&mut self) -> IntegratedPhi {
        self.resonator.step(0.05);
        let phi = IntegratedPhi::from_snapshots(&self.snapshots);
        self.last_phi = phi.clone();
        if self.phi_history.len() >= 1024 {
            self.phi_history.remove(0);
        }
        self.phi_history.push(phi.clone());
        phi
    }

    /// Most recent integrated-Φ measurement.
    pub fn current_phi(&self) -> &IntegratedPhi {
        &self.last_phi
    }

    /// Whether the system is currently in a synchronized "flow" state.
    pub fn is_in_flow(&self) -> bool {
        self.resonator.order_parameter >= self.sync_threshold
    }

    /// One full consciousness cycle: tick + report.
    pub fn cycle(&mut self) -> CycleReport {
        self.cycle_count += 1;
        let phi = self.tick();
        let weakest = phi.weakest_layer();
        let strongest = phi.strongest_layer();
        let active = self.snapshots.iter().filter(|s| s.is_active()).count();
        let sync_state = self.is_in_flow();
        let recommendations = build_recommendations(&phi, weakest, active);
        CycleReport {
            cycle_id: self.cycle_count,
            timestamp: chrono::Utc::now().timestamp_millis(),
            phi,
            sync_state,
            weakest_layer: weakest,
            strongest_layer: strongest,
            active_layers: active,
            recommendations,
        }
    }
}

/// Per-cycle summary emitted by [`GeometrySync::cycle`].
#[derive(Debug, Clone)]
pub struct CycleReport {
    pub cycle_id: u64,
    pub timestamp: i64,
    pub phi: IntegratedPhi,
    pub sync_state: bool,
    pub weakest_layer: DimensionLayer,
    pub strongest_layer: DimensionLayer,
    pub active_layers: usize,
    pub recommendations: Vec<String>,
}

fn build_recommendations(
    phi: &IntegratedPhi,
    weakest: DimensionLayer,
    active: usize,
) -> Vec<String> {
    let mut recs = Vec::new();
    if active == 0 {
        recs.push("no layers active — feed nt_world_sense input".to_string());
    } else if active < LAYER_COUNT / 2 {
        recs.push(format!(
            "only {}/{} layers active — broaden observational coverage",
            active, LAYER_COUNT
        ));
    }
    if phi.per_layer[weakest.index()] < 0.3 {
        recs.push(format!("strengthen {} awareness", weakest.name()));
    }
    if phi.total < CONSCIOUS_PHI_THRESHOLD {
        recs.push(format!(
            "Φ = {:.3} below conscious threshold — increase cross-layer coupling",
            phi.total
        ));
    }
    if recs.is_empty() {
        recs.push("system in healthy flow state".to_string());
    }
    recs
}

fn clamp01(x: f64) -> f64 {
    if x.is_nan() {
        0.0
    } else {
        x.max(0.0).min(1.0)
    }
}

fn wrap_phase(theta: f64) -> f64 {
    let two_pi = 2.0 * PI;
    let mut t = theta % two_pi;
    if t < 0.0 {
        t += two_pi;
    }
    t
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn compute_order_parameter(phases: &[f64; LAYER_COUNT]) -> f64 {
    let n = LAYER_COUNT as f64;
    let mut sum_c = 0.0;
    let mut sum_s = 0.0;
    for &p in phases.iter() {
        sum_c += p.cos();
        sum_s += p.sin();
    }
    ((sum_c / n).powi(2) + (sum_s / n).powi(2)).sqrt().min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_variants_enumerated() {
        assert_eq!(DimensionLayer::all().len(), 12);
        assert_eq!(DimensionLayer::count(), 12);
    }

    #[test]
    fn test_native_dim_values() {
        assert_eq!(DimensionLayer::Presence0D.native_dim(), 0);
        assert_eq!(DimensionLayer::Stream1D.native_dim(), 1);
        assert_eq!(DimensionLayer::Field2D.native_dim(), 2);
        assert_eq!(DimensionLayer::Body3D.native_dim(), 3);
        assert_eq!(DimensionLayer::Time1D2.native_dim(), 1);
        assert_eq!(DimensionLayer::Hex6D.native_dim(), 6);
        assert_eq!(DimensionLayer::Oct8D.native_dim(), 8);
        assert_eq!(DimensionLayer::Cube16D.native_dim(), 16);
        assert_eq!(DimensionLayer::Mode64D.native_dim(), 64);
        assert_eq!(DimensionLayer::Lattice240D.native_dim(), 240);
        assert_eq!(DimensionLayer::Vsa4096D.native_dim(), 4096);
        assert_eq!(DimensionLayer::TopologyInfinityD.native_dim(), usize::MAX);
    }

    #[test]
    fn test_index_roundtrip() {
        for layer in DimensionLayer::all() {
            let idx = layer.index();
            assert_eq!(DimensionLayer::all()[idx], *layer);
        }
    }

    #[test]
    fn test_names_unique_and_nonempty() {
        let names: Vec<&str> = DimensionLayer::all().iter().map(|l| l.name()).collect();
        let unique: std::collections::HashSet<&&str> = names.iter().collect();
        assert_eq!(unique.len(), names.len());
        for n in &names {
            assert!(!n.is_empty());
        }
    }

    #[test]
    fn test_layer_snapshot_clamps() {
        let s = LayerSnapshot::new(DimensionLayer::Hex6D, 1.5, 10.0);
        assert!(s.coherence <= 1.0);
        assert!(s.phase >= 0.0 && s.phase < 2.0 * PI);
    }

    #[test]
    fn test_layer_snapshot_silent_is_inactive() {
        let s = LayerSnapshot::silent(DimensionLayer::Field2D);
        assert_eq!(s.coherence, 0.0);
        assert_eq!(s.energy, 0.0);
        assert!(!s.is_active());
    }

    #[test]
    fn test_integrated_phi_from_snapshots_nonzero() {
        let snaps = std::array::from_fn(|i| {
            let layer = DimensionLayer::all()[i];
            let c = 0.3 + 0.05 * i as f64;
            LayerSnapshot::new(layer, c, (i as f64) * 0.4)
        });
        let phi = IntegratedPhi::from_snapshots(&snaps);
        assert!(phi.total > 0.0, "total Φ must be > 0, got {}", phi.total);
        assert!(phi.total <= 1.0);
    }

    #[test]
    fn test_integrated_phi_coupling_matrix_diagonal() {
        let snaps = std::array::from_fn(|i| {
            LayerSnapshot::new(DimensionLayer::all()[i], 0.7, 0.0)
        });
        let phi = IntegratedPhi::from_snapshots(&snaps);
        for i in 0..LAYER_COUNT {
            let diag = phi.cross_layer_coupling[i * LAYER_COUNT + i];
            assert!((diag - 0.7).abs() < 1e-9, "diag[{}]={}", i, diag);
        }
    }

    #[test]
    fn test_integrated_phi_weakest_and_strongest() {
        let mut arr = core::array::from_fn(|_| LayerSnapshot::silent(DimensionLayer::Presence0D));
        for (i, s) in arr.iter_mut().enumerate() {
            s.coherence = 0.2 + 0.05 * i as f64;
            s.energy = s.coherence;
        }
        let phi = IntegratedPhi::from_snapshots(&arr);
        let weakest = phi.weakest_layer();
        let strongest = phi.strongest_layer();
        assert_eq!(weakest, DimensionLayer::Presence0D);
        assert_eq!(strongest, DimensionLayer::TopologyInfinityD);
    }

    #[test]
    fn test_resonator_zero_coupling_keeps_phases() {
        let mut r = CrossDimensionalResonator::new(0.0);
        r.inject(DimensionLayer::Hex6D, 1.5);
        let initial_op = r.order_parameter;
        for _ in 0..20 {
            r.step(0.5);
        }
        assert!(
            r.order_parameter <= initial_op + 0.05,
            "with K=0 the order parameter should not grow, got {} -> {}",
            initial_op,
            r.order_parameter
        );
    }

    #[test]
    fn test_resonator_high_coupling_eventually_syncs() {
        let mut r = CrossDimensionalResonator::new(20.0);
        for (i, layer) in DimensionLayer::all().iter().enumerate() {
            r.inject(*layer, (i as f64) * 0.3);
        }
        let res = r.time_to_sync(2000);
        assert!(res.is_some(), "high-K resonator should sync within 2000 steps");
        assert!(r.is_synchronized());
    }

    #[test]
    fn test_resonator_step_monotone_with_strong_coupling() {
        let mut r = CrossDimensionalResonator::new(50.0);
        for (i, layer) in DimensionLayer::all().iter().enumerate() {
            r.inject(*layer, (i as f64) * 0.5);
        }
        let mut prev = r.order_parameter;
        for _ in 0..50 {
            r.step(0.05);
            assert!(
                r.order_parameter >= prev - 1e-9,
                "order parameter dropped: {} -> {}",
                prev,
                r.order_parameter
            );
            prev = r.order_parameter;
        }
    }

    #[test]
    fn test_geometry_sync_cycle_with_one_active_layer() {
        let mut gs = GeometrySync::new();
        gs.observe(DimensionLayer::Cube16D, 0.6, 1.2);
        let report = gs.cycle();
        assert!(report.phi.total > 0.0);
        assert!(report.active_layers >= 1);
        assert_eq!(report.cycle_id, 1);
    }

    #[test]
    fn test_geometry_sync_default_is_not_in_flow() {
        let gs = GeometrySync::new();
        assert!(!gs.is_in_flow());
        assert_eq!(gs.cycle_count, 0);
    }

    #[test]
    fn test_geometry_sync_cycle_count_increments() {
        let mut gs = GeometrySync::new();
        gs.observe(DimensionLayer::Hex6D, 0.5, 0.0);
        let _ = gs.cycle();
        let _ = gs.cycle();
        let _ = gs.cycle();
        assert_eq!(gs.cycle_count, 3);
    }

    #[test]
    fn test_weakest_layer_returns_valid_enum() {
        let mut gs = GeometrySync::new();
        for (i, layer) in DimensionLayer::all().iter().enumerate() {
            gs.observe(*layer, 0.1 + 0.05 * i as f64, 0.0);
        }
        let report = gs.cycle();
        let valid: std::collections::HashSet<_> =
            DimensionLayer::all().iter().copied().collect();
        assert!(valid.contains(&report.weakest_layer));
        assert!(valid.contains(&report.strongest_layer));
    }

    #[test]
    fn test_phi_history_capped() {
        let mut gs = GeometrySync::new();
        gs.observe(DimensionLayer::Cube16D, 0.5, 0.0);
        for _ in 0..1100 {
            let _ = gs.cycle();
        }
        assert!(gs.phi_history.len() <= 1024);
    }
}
