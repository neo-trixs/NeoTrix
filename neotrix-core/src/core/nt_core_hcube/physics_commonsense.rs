use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

pub const PHY_DIM: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    Solid,
    Liquid,
    Gas,
    Granular,
    Rigid,
    Elastic,
    Fragile,
    Slippery,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhysicalProperty {
    Density(f32),
    Elasticity(f32),
    Friction(f32),
    Mass(f32),
    Volume(f32),
    Temperature(f32),
    Phase(Material),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpatialRelation {
    Above,
    Below,
    Inside,
    OnTopOf,
    AttachedTo,
    Near,
    SupportedBy,
}

#[derive(Debug, Clone)]
pub struct EnergyTracker {
    pub kinetic: f64,
    pub potential: f64,
    pub thermal: f64,
    pub total_input: f64,
    pub total_output: f64,
    pub conservation_violation: f64,
}

impl EnergyTracker {
    pub fn new() -> Self {
        Self {
            kinetic: 0.0,
            potential: 0.0,
            thermal: 0.0,
            total_input: 0.0,
            total_output: 0.0,
            conservation_violation: 0.0,
        }
    }

    pub fn record_transfer(
        &mut self,
        d_kinetic: f64,
        d_potential: f64,
        d_thermal: f64,
        external_work: f64,
    ) -> f64 {
        self.kinetic += d_kinetic;
        self.potential += d_potential;
        self.thermal += d_thermal;
        let delta_total = d_kinetic + d_potential + d_thermal;
        let imbalance = (delta_total - external_work).abs();
        if imbalance > 0.0 {
            self.conservation_violation += imbalance;
            self.total_input += external_work.max(0.0);
            self.total_output += (-external_work).max(0.0);
        }
        imbalance
    }

    pub fn energy_ratio(&self) -> f64 {
        let total = self.total_input + self.total_output;
        if total < 1e-12 {
            1.0
        } else {
            1.0 - self.conservation_violation / total
        }
    }
}

#[derive(Debug, Clone)]
pub struct MomentumTracker {
    pub px: f64,
    pub py: f64,
    pub pz: f64,
    pub violations: u64,
}

impl MomentumTracker {
    pub fn new() -> Self {
        Self {
            px: 0.0,
            py: 0.0,
            pz: 0.0,
            violations: 0,
        }
    }

    pub fn apply_impulse(&mut self, fx: f64, fy: f64, fz: f64, dt: f64) {
        self.px += fx * dt;
        self.py += fy * dt;
        self.pz += fz * dt;
    }

    pub fn check_conservation(&mut self, before: (f64, f64, f64), after: (f64, f64, f64)) -> f64 {
        let dp = (
            (after.0 - before.0).abs(),
            (after.1 - before.1).abs(),
            (after.2 - before.2).abs(),
        );
        let total = dp.0 + dp.1 + dp.2;
        if total > 0.01 {
            self.violations += 1;
        }
        total
    }

    pub fn momentum_ratio(&self) -> f64 {
        1.0 - (self.violations as f64 / (self.violations + 1) as f64)
    }
}

pub struct PhysicsCommonsense {
    property_bases: Vec<Vec<u8>>,
    relation_bases: Vec<Vec<u8>>,
    pub energy: EnergyTracker,
    pub momentum: MomentumTracker,
}

impl Default for PhysicsCommonsense {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsCommonsense {
    pub fn new() -> Self {
        let property_bases = vec![
            QuantizedVSA::seeded_random(5001, PHY_DIM),
            QuantizedVSA::seeded_random(5002, PHY_DIM),
            QuantizedVSA::seeded_random(5003, PHY_DIM),
            QuantizedVSA::seeded_random(5004, PHY_DIM),
            QuantizedVSA::seeded_random(5005, PHY_DIM),
            QuantizedVSA::seeded_random(5006, PHY_DIM),
            QuantizedVSA::seeded_random(5007, PHY_DIM),
        ];
        let relation_bases = vec![
            QuantizedVSA::seeded_random(6001, PHY_DIM),
            QuantizedVSA::seeded_random(6002, PHY_DIM),
            QuantizedVSA::seeded_random(6003, PHY_DIM),
            QuantizedVSA::seeded_random(6004, PHY_DIM),
            QuantizedVSA::seeded_random(6005, PHY_DIM),
            QuantizedVSA::seeded_random(6006, PHY_DIM),
            QuantizedVSA::seeded_random(6007, PHY_DIM),
        ];
        Self {
            property_bases,
            relation_bases,
            energy: EnergyTracker::new(),
            momentum: MomentumTracker::new(),
        }
    }

    pub fn encode_property(&self, prop: PhysicalProperty) -> Vec<u8> {
        let (idx, val) = match prop {
            PhysicalProperty::Density(v) => (0, v),
            PhysicalProperty::Elasticity(v) => (1, v),
            PhysicalProperty::Friction(v) => (2, v),
            PhysicalProperty::Mass(v) => (3, v),
            PhysicalProperty::Volume(v) => (4, v),
            PhysicalProperty::Temperature(v) => (5, v),
            PhysicalProperty::Phase(m) => {
                let v = match m {
                    Material::Solid => 0.1,
                    Material::Liquid => 0.3,
                    Material::Gas => 0.5,
                    Material::Granular => 0.7,
                    Material::Rigid => 0.9,
                    Material::Elastic => 0.2,
                    Material::Fragile => 0.4,
                    Material::Slippery => 0.6,
                };
                return QuantizedVSA::seeded_random((v * 1000.0) as u64, PHY_DIM);
            }
        };
        let basis = &self.property_bases[idx];
        let shift = (val.abs() * 100.0) as isize % PHY_DIM as isize;
        QuantizedVSA::permute(basis, shift)
    }

    pub fn encode_relation(&self, rel: SpatialRelation) -> Vec<u8> {
        let (idx, offset) = match rel {
            SpatialRelation::Above => (0, 10),
            SpatialRelation::Below => (1, 20),
            SpatialRelation::Inside => (2, 30),
            SpatialRelation::OnTopOf => (3, 40),
            SpatialRelation::AttachedTo => (4, 50),
            SpatialRelation::Near => (5, 60),
            SpatialRelation::SupportedBy => (6, 70),
        };
        let basis = &self.relation_bases[idx];
        QuantizedVSA::permute(basis, offset)
    }

    pub fn bind_subject_property(&self, subject: &[u8], prop: &[u8]) -> Vec<u8> {
        QuantizedVSA::bind(subject, prop)
    }

    pub fn bind_subject_relation(&self, subject: &[u8], relation: &[u8]) -> Vec<u8> {
        QuantizedVSA::bind(subject, relation)
    }

    pub fn infer_causal_chain(&self, premises: &[&[u8]], candidate: &[u8]) -> f64 {
        if premises.is_empty() {
            return 0.0;
        }
        let bundle = QuantizedVSA::bundle(premises);
        QuantizedVSA::similarity(&bundle, candidate)
    }

    pub fn gravity_check(mass: f64, above_object: &[u8], below_object: &[u8]) -> f64 {
        let mass_factor = if mass > 0.0 {
            1.0 - (-mass * 0.1).exp()
        } else {
            0.0
        };
        let pos_sim = QuantizedVSA::similarity(above_object, below_object);
        pos_sim * mass_factor
    }

    pub fn support_check(load: &[u8], support: &[u8], friction: f64) -> f64 {
        let load_sim = QuantizedVSA::similarity(load, support);
        let friction_factor = friction.min(1.0).max(0.0);
        load_sim * friction_factor
    }

    pub fn record_collision(
        &mut self,
        mass_a: f64,
        v_a: (f64, f64, f64),
        mass_b: f64,
        v_b: (f64, f64, f64),
    ) -> f64 {
        let ke_before = 0.5 * mass_a * (v_a.0.powi(2) + v_a.1.powi(2) + v_a.2.powi(2))
            + 0.5 * mass_b * (v_b.0.powi(2) + v_b.1.powi(2) + v_b.2.powi(2));
        let total_mass = mass_a + mass_b;
        let v_cm_x = (mass_a * v_a.0 + mass_b * v_b.0) / total_mass;
        let v_cm_y = (mass_a * v_a.1 + mass_b * v_b.1) / total_mass;
        let v_cm_z = (mass_a * v_a.2 + mass_b * v_b.2) / total_mass;
        let ke_after = 0.5 * mass_a * (v_cm_x.powi(2) + v_cm_y.powi(2) + v_cm_z.powi(2))
            + 0.5 * mass_b * (v_cm_x.powi(2) + v_cm_y.powi(2) + v_cm_z.powi(2));
        let violation = self
            .energy
            .record_transfer(ke_after - ke_before, 0.0, 0.0, 0.0);
        let momentum_violation = self.momentum.check_conservation(
            (
                mass_a * v_a.0 + mass_b * v_b.0,
                mass_a * v_a.1 + mass_b * v_b.1,
                mass_a * v_a.2 + mass_b * v_b.2,
            ),
            (
                total_mass * v_cm_x,
                total_mass * v_cm_y,
                total_mass * v_cm_z,
            ),
        );
        violation + momentum_violation
    }

    pub fn conservation_report(&self) -> String {
        format!(
            "Energy ratio={:.4}, Momentum ratio={:.4}, Kinetic={:.2}, Potential={:.2}, Thermal={:.2}",
            self.energy.energy_ratio(),
            self.momentum.momentum_ratio(),
            self.energy.kinetic,
            self.energy.potential,
            self.energy.thermal,
        )
    }

    pub fn phase_transition(temperature: f64, material: Material) -> f64 {
        let (melt, boil) = match material {
            Material::Solid => (500.0, 3000.0),
            Material::Liquid => (0.0, 100.0),
            Material::Gas => (-200.0, -100.0),
            Material::Granular => (1000.0, 2500.0),
            Material::Rigid => (800.0, 2000.0),
            Material::Elastic => (300.0, 1500.0),
            Material::Fragile => (200.0, 1000.0),
            Material::Slippery => (100.0, 500.0),
        };
        if temperature >= boil {
            1.0
        } else if temperature >= melt {
            (temperature - melt) / (boil - melt)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_physics() -> PhysicsCommonsense {
        PhysicsCommonsense::new()
    }

    #[test]
    fn test_encode_density_property() {
        let p = make_physics();
        let d1 = p.encode_property(PhysicalProperty::Density(1.0));
        let d2 = p.encode_property(PhysicalProperty::Density(2.0));
        let sim = QuantizedVSA::similarity(&d1, &d2);
        assert!(
            sim < 0.95,
            "different densities should be somewhat dissimilar"
        );
    }

    #[test]
    fn test_encode_relation_above_below() {
        let p = make_physics();
        let above = p.encode_relation(SpatialRelation::Above);
        let below = p.encode_relation(SpatialRelation::Below);
        let sim = QuantizedVSA::similarity(&above, &below);
        assert!(sim < 0.9, "above vs below should be distinguishable");
    }

    #[test]
    fn test_encode_relation_self_similar() {
        let p = make_physics();
        let r = p.encode_relation(SpatialRelation::Inside);
        let sim = QuantizedVSA::similarity(&r, &r);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bind_subject_property() {
        let p = make_physics();
        let obj = QuantizedVSA::seeded_random(123, PHY_DIM);
        let density = p.encode_property(PhysicalProperty::Density(2.7));
        let bound = p.bind_subject_property(&obj, &density);
        assert_eq!(bound.len(), PHY_DIM);
    }

    #[test]
    fn test_infer_causal_chain_empty() {
        let p = make_physics();
        let candidate = QuantizedVSA::seeded_random(999, PHY_DIM);
        let score = p.infer_causal_chain(&[], &candidate);
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_gravity_check_massless() {
        let a = QuantizedVSA::seeded_random(10, PHY_DIM);
        let b = QuantizedVSA::seeded_random(20, PHY_DIM);
        let score = PhysicsCommonsense::gravity_check(0.0, &a, &b);
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_gravity_check_positive_mass() {
        let a = QuantizedVSA::seeded_random(10, PHY_DIM);
        let b = QuantizedVSA::seeded_random(10, PHY_DIM);
        let score = PhysicsCommonsense::gravity_check(10.0, &a, &b);
        // same vector → high sim, heavy mass → high factor
        assert!(score > 0.5);
    }

    #[test]
    fn test_support_check_zero_friction() {
        let _p = make_physics();
        let load = QuantizedVSA::seeded_random(1, PHY_DIM);
        let support = QuantizedVSA::seeded_random(2, PHY_DIM);
        let score = PhysicsCommonsense::support_check(&load, &support, 0.0);
        assert!(score < 0.01, "zero friction → no support");
    }

    #[test]
    fn test_phase_transition_below_melt() {
        let frac = PhysicsCommonsense::phase_transition(-50.0, Material::Liquid);
        assert!((frac - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_phase_transition_above_boil() {
        let frac = PhysicsCommonsense::phase_transition(150.0, Material::Liquid);
        assert!((frac - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_phase_transition_partial_melt() {
        let frac = PhysicsCommonsense::phase_transition(50.0, Material::Liquid);
        assert!(
            frac > 0.0 && frac < 1.0,
            "partial melt should be between 0 and 1"
        );
    }

    #[test]
    fn test_phase_solid_high_temp() {
        let frac = PhysicsCommonsense::phase_transition(1500.0, Material::Solid);
        assert!(frac > 0.5, "1500K should be past solid melt point 500K");
    }

    #[test]
    fn test_material_phase_property() {
        let p = make_physics();
        let liquid = p.encode_property(PhysicalProperty::Phase(Material::Liquid));
        let gas = p.encode_property(PhysicalProperty::Phase(Material::Gas));
        let sim = QuantizedVSA::similarity(&liquid, &gas);
        assert!(sim < 0.9, "liquid and gas phase vectors should be distinct");
    }

    #[test]
    fn test_same_property_same_value() {
        let p = make_physics();
        let a = p.encode_property(PhysicalProperty::Density(1.5));
        let b = p.encode_property(PhysicalProperty::Density(1.5));
        let sim = QuantizedVSA::similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 0.01,
            "same property+value should be nearly identical"
        );
    }

    #[test]
    fn test_energy_tracker_conservation() {
        let mut et = EnergyTracker::new();
        let v = et.record_transfer(10.0, -5.0, -5.0, 0.0);
        assert!(
            (v - 0.0).abs() < 1e-10,
            "Perfectly balanced transfer should have 0 violation, got {}",
            v
        );
        assert!((et.energy_ratio() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_energy_tracker_violation() {
        let mut et = EnergyTracker::new();
        let v = et.record_transfer(10.0, 0.0, 0.0, 0.0);
        assert!(
            v > 0.0,
            "Unbalanced transfer should create violation, got {}",
            v
        );
    }

    #[test]
    fn test_momentum_conservation() {
        let mut mt = MomentumTracker::new();
        let v = mt.check_conservation((10.0, 0.0, 0.0), (10.0, 0.0, 0.0));
        assert!(
            (v - 0.0).abs() < 1e-10,
            "Conserved momentum should have 0 violation, got {}",
            v
        );
    }

    #[test]
    fn test_collision_violation() {
        let mut pc = PhysicsCommonsense::new();
        let v = pc.record_collision(1.0, (2.0, 0.0, 0.0), 1.0, (-2.0, 0.0, 0.0));
        assert!(
            v < 0.01,
            "Elastic collision should have near-zero violation, got {}",
            v
        );
    }

    #[test]
    fn test_conservation_report() {
        let mut pc = PhysicsCommonsense::new();
        pc.record_collision(1.0, (2.0, 0.0, 0.0), 1.0, (0.0, 0.0, 0.0));
        let report = pc.conservation_report();
        assert!(report.contains("Energy ratio"));
        assert!(report.contains("Momentum ratio"));
    }
}
