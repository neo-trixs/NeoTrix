use crate::core::nt_core_e8::{e8_root_system, E8Weight};
use std::collections::HashMap;

/// Topological charge: integer-valued invariant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopologicalCharge(pub i32);

impl TopologicalCharge {
    pub fn new(c: i32) -> Self {
        Self(c)
    }
    pub fn value(&self) -> i32 {
        self.0
    }
}

/// Half-integer spin: stored as numerator/2 (e.g., 1/2 → 1, 3/2 → 3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HalfIntegerSpin(pub i32);

impl HalfIntegerSpin {
    pub fn new(numerator: i32) -> Self {
        Self(numerator)
    }
    pub fn numerator(&self) -> i32 {
        self.0
    }
    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / 2.0
    }
}

/// Weyl orbit type for E8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeylOrbit {
    Orbit2_2_0_6,
    Orbit1_7,
    Orbit2_1_6,
    Orbit36,
    Orbit0,
}

/// Defect configuration for a Weyl orbit
#[derive(Debug, Clone)]
pub struct DefectConfig {
    pub orbit: WeylOrbit,
    pub charge: TopologicalCharge,
    pub spin: HalfIntegerSpin,
    pub mass_scale: f64,
    pub multiplicity: usize,
}

/// Force type for coupling constants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForceType {
    Electromagnetic,
    Weak,
    Strong,
    Gravitational,
    E8Unified,
}

/// Particle ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleId(pub u64);

/// Mass prediction for a particle
#[derive(Debug, Clone)]
pub struct MassPrediction {
    pub particle: ParticleId,
    pub mass_gev: f64,
    pub uncertainty: f64,
}

/// E8 topological defects: maps Weyl orbits to topological defect configurations
#[derive(Debug, Clone)]
pub struct E8TopologicalDefects {
    pub weyl_orbits: HashMap<WeylOrbit, DefectConfig>,
    roots: Vec<E8Weight>,
}

impl Default for E8TopologicalDefects {
    fn default() -> Self {
        Self::new()
    }
}

impl E8TopologicalDefects {
    pub fn new() -> Self {
        let roots = e8_root_system();
        let mut defects = Self {
            weyl_orbits: HashMap::new(),
            roots,
        };
        defects.initialize_orbits();
        defects
    }

    fn initialize_orbits(&mut self) {
        let mut counts: HashMap<WeylOrbit, usize> = HashMap::new();
        for root in &self.roots {
            let orbit = classify_root(root);
            *counts.entry(orbit).or_insert(0) += 1;
        }
        for (&orbit, &count) in &counts {
            let (charge, spin, mass_scale) = match orbit {
                WeylOrbit::Orbit2_2_0_6 => (TopologicalCharge(2), HalfIntegerSpin(0), 100.0),
                WeylOrbit::Orbit1_7 => (TopologicalCharge(1), HalfIntegerSpin(1), 80.0),
                WeylOrbit::Orbit2_1_6 => (TopologicalCharge(1), HalfIntegerSpin(0), 50.0),
                WeylOrbit::Orbit36 => (TopologicalCharge(3), HalfIntegerSpin(1), 200.0),
                WeylOrbit::Orbit0 => (TopologicalCharge(0), HalfIntegerSpin(0), 0.0),
            };
            self.weyl_orbits.insert(
                orbit,
                DefectConfig {
                    orbit,
                    charge,
                    spin,
                    mass_scale,
                    multiplicity: count,
                },
            );
        }
    }

    pub fn get_defect(&self, orbit: &WeylOrbit) -> Option<&DefectConfig> {
        self.weyl_orbits.get(orbit)
    }

    pub fn all_defects(&self) -> Vec<&DefectConfig> {
        self.weyl_orbits.values().collect()
    }

    pub fn total_topological_charge(&self) -> TopologicalCharge {
        let sum: i32 = self
            .weyl_orbits
            .values()
            .map(|d| d.charge.value() * d.multiplicity as i32)
            .sum();
        TopologicalCharge(sum)
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }
}

fn classify_root(root: &E8Weight) -> WeylOrbit {
    let non_zero = root.coords.iter().filter(|&&c| c != 0).count();
    let max_val = root.coords.iter().map(|&c: &i8| c.abs()).max().unwrap_or(0);
    match (non_zero, max_val) {
        (2, 2) => WeylOrbit::Orbit2_2_0_6,
        (8, 1) => WeylOrbit::Orbit1_7,
        (2, 1) if non_zero == 2 && max_val == 1 => WeylOrbit::Orbit36,
        (0, 0) => WeylOrbit::Orbit0,
        _ => WeylOrbit::Orbit2_1_6,
    }
}

/// E8 particle spectrum: mass predictions and coupling constants
#[derive(Debug, Clone)]
pub struct E8ParticleSpectrum {
    pub mass_predictions: Vec<MassPrediction>,
    pub coupling_constants: HashMap<ForceType, f64>,
}

impl Default for E8ParticleSpectrum {
    fn default() -> Self {
        Self::new()
    }
}

impl E8ParticleSpectrum {
    pub fn new() -> Self {
        let mut couplings = HashMap::new();
        couplings.insert(ForceType::Electromagnetic, 1.0 / 137.0);
        couplings.insert(ForceType::Weak, 0.65);
        couplings.insert(ForceType::Strong, 0.118);
        couplings.insert(ForceType::Gravitational, 1.0);
        couplings.insert(ForceType::E8Unified, 0.025);
        Self {
            mass_predictions: Vec::new(),
            coupling_constants: couplings,
        }
    }

    pub fn predict_masses(&mut self, defects: &E8TopologicalDefects) {
        self.mass_predictions.clear();
        let mut pid = 0u64;
        for (_orbit, config) in &defects.weyl_orbits {
            let base_mass = config.mass_scale;
            let multiplicity = config.multiplicity;
            for i in 0..multiplicity.min(10) {
                let perturbation = (i as f64 + 1.0) * 0.1;
                let mass = base_mass * (1.0 + perturbation * config.spin.as_f64());
                self.mass_predictions.push(MassPrediction {
                    particle: ParticleId(pid),
                    mass_gev: mass,
                    uncertainty: mass * 0.1,
                });
                pid += 1;
            }
        }
    }

    pub fn get_coupling(&self, force: ForceType) -> f64 {
        self.coupling_constants.get(&force).copied().unwrap_or(0.0)
    }

    pub fn set_coupling(&mut self, force: ForceType, value: f64) {
        self.coupling_constants.insert(force, value.clamp(0.0, 1.0));
    }

    pub fn mass_count(&self) -> usize {
        self.mass_predictions.len()
    }

    pub fn highest_mass(&self) -> f64 {
        self.mass_predictions
            .iter()
            .map(|m| m.mass_gev)
            .fold(0.0_f64, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_defects_creation() {
        let defects = E8TopologicalDefects::new();
        assert!(!defects.weyl_orbits.is_empty());
        assert!(defects.root_count() > 0);
    }

    #[test]
    fn test_classify_orbit_2_2_0_6() {
        let root = E8Weight::new([2, 2, 0, 0, 0, 0, 0, 0]);
        assert_eq!(classify_root(&root), WeylOrbit::Orbit2_2_0_6);
    }

    #[test]
    fn test_classify_orbit_1_7() {
        let root = E8Weight::new([1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(classify_root(&root), WeylOrbit::Orbit1_7);
    }

    #[test]
    fn test_topological_charge() {
        let charge = TopologicalCharge(3);
        assert_eq!(charge.value(), 3);
    }

    #[test]
    fn test_half_integer_spin() {
        let spin = HalfIntegerSpin(1);
        assert_eq!(spin.numerator(), 1);
        assert!((spin.as_f64() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_particle_spectrum_default() {
        let spectrum = E8ParticleSpectrum::new();
        assert_eq!(spectrum.coupling_constants.len(), 5);
        assert!((spectrum.get_coupling(ForceType::Electromagnetic) - 1.0 / 137.0).abs() < 1e-9);
    }

    #[test]
    fn test_predict_masses() {
        let defects = E8TopologicalDefects::new();
        let mut spectrum = E8ParticleSpectrum::new();
        spectrum.predict_masses(&defects);
        assert!(spectrum.mass_count() > 0);
        assert!(spectrum.highest_mass() > 0.0);
    }

    #[test]
    fn test_set_coupling() {
        let mut spectrum = E8ParticleSpectrum::new();
        spectrum.set_coupling(ForceType::E8Unified, 0.05);
        assert!((spectrum.get_coupling(ForceType::E8Unified) - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_get_defect_by_orbit() {
        let defects = E8TopologicalDefects::new();
        let defect = defects.get_defect(&WeylOrbit::Orbit2_2_0_6);
        assert!(defect.is_some());
        assert_eq!(defect.unwrap().multiplicity, 112);
    }

    #[test]
    fn test_total_topological_charge_non_zero() {
        let defects = E8TopologicalDefects::new();
        let total = defects.total_topological_charge();
        assert!(total.value() > 0);
    }
}
