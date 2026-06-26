use crate::core::nt_core_e8::{compute_e8_structure_constants, e8_root_system, E8Weight};

/// 3D lattice point for field discretization
#[derive(Debug, Clone, Copy)]
pub struct LatticePoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl LatticePoint {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// A field value at a lattice site, represented as E8 root activations.
#[derive(Debug, Clone)]
pub struct FieldValue {
    /// Activation per E8 root (length = number of roots)
    pub root_activations: Vec<f64>,
    /// Scalar field magnitude at this point
    pub magnitude: f64,
}

/// PDE solver step configuration
#[derive(Debug, Clone)]
pub struct FieldSolverConfig {
    /// Lattice size (N x N x N) — total N^3 points
    pub lattice_size: usize,
    /// Time step for PDE integration
    pub dt: f64,
    /// Diffusion coefficient
    pub alpha: f64,
    /// Number of integration steps
    pub num_steps: usize,
}

impl Default for FieldSolverConfig {
    fn default() -> Self {
        Self {
            lattice_size: 8,
            dt: 0.01,
            alpha: 0.1,
            num_steps: 100,
        }
    }
}

/// E8 field solver — lattice discretization of E8-grounded PDE evolution.
///
/// Inspired by OpenWave's classical field PDE approach, mapping E8 root
/// system activations onto a 3D lattice and evolving via finite differences.
#[derive(Debug, Clone)]
pub struct E8FieldSolver {
    pub config: FieldSolverConfig,
    pub roots: Vec<E8Weight>,
    pub structure_constants: Vec<(usize, usize, usize, f64)>,
    pub field: Vec<Vec<Vec<FieldValue>>>,
    pub iteration: usize,
}

impl E8FieldSolver {
    pub fn new(config: FieldSolverConfig) -> Self {
        let roots = e8_root_system();
        let sc = compute_e8_structure_constants(&roots);
        let structure_constants: Vec<(usize, usize, usize, f64)> =
            sc.iter().map(|s| (s.0, s.1, s.2, s.3 as f64)).collect();
        let n = config.lattice_size;
        let empty = FieldValue {
            root_activations: vec![0.0; roots.len()],
            magnitude: 0.0,
        };
        let field = vec![vec![vec![empty.clone(); n]; n]; n];
        Self {
            config,
            roots,
            structure_constants,
            field,
            iteration: 0,
        }
    }

    pub fn lattice_size(&self) -> usize {
        self.config.lattice_size
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    pub fn structure_constant_count(&self) -> usize {
        self.structure_constants.len()
    }

    /// Initialize the field with a seed configuration.
    /// Places a Gaussian-like activation at center lattice site.
    pub fn initialize(&mut self, amplitude: f64) {
        let n = self.config.lattice_size;
        let center = n as i32 / 2;
        let n_roots = self.roots.len();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let dx = (x as i32 - center) as f64;
                    let dy = (y as i32 - center) as f64;
                    let dz = (z as i32 - center) as f64;
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                    let gauss = amplitude * (-dist * dist / 4.0).exp();
                    let mut root_activations = vec![0.0_f64; n_roots];
                    for i in 0..n_roots.min(n_roots) {
                        root_activations[i] = gauss * (i as f64 / n_roots as f64);
                    }
                    self.field[x][y][z] = FieldValue {
                        magnitude: gauss,
                        root_activations,
                    };
                }
            }
        }
    }

    /// Compute the Laplacian at a lattice point using finite differences.
    fn laplacian(&self, x: usize, y: usize, z: usize) -> f64 {
        let n = self.config.lattice_size;
        let mut lap = 0.0;
        let center = self.field[x][y][z].magnitude;
        let mut neighbors = 0;
        if x > 0 {
            lap += self.field[x - 1][y][z].magnitude - center;
            neighbors += 1;
        }
        if x + 1 < n {
            lap += self.field[x + 1][y][z].magnitude - center;
            neighbors += 1;
        }
        if y > 0 {
            lap += self.field[x][y - 1][z].magnitude - center;
            neighbors += 1;
        }
        if y + 1 < n {
            lap += self.field[x][y + 1][z].magnitude - center;
            neighbors += 1;
        }
        if z > 0 {
            lap += self.field[x][y][z - 1].magnitude - center;
            neighbors += 1;
        }
        if z + 1 < n {
            lap += self.field[x][y][z + 1].magnitude - center;
            neighbors += 1;
        }
        if neighbors > 0 {
            lap / neighbors as f64
        } else {
            0.0
        }
    }

    /// Compute the E8 structure constant interaction term at a point.
    fn e8_interaction(&self, x: usize, y: usize, z: usize) -> f64 {
        let n_roots = self.roots.len();
        let mut interaction = 0.0;
        for &(a, b, c, fabc) in &self.structure_constants {
            if a < n_roots && b < n_roots && c < n_roots {
                let ra = self.field[x][y][z]
                    .root_activations
                    .get(a)
                    .copied()
                    .unwrap_or(0.0);
                let rb = self.field[x][y][z]
                    .root_activations
                    .get(b)
                    .copied()
                    .unwrap_or(0.0);
                interaction += fabc * ra * rb;
            }
        }
        interaction
    }

    /// Perform a single PDE time step (diffusion + E8 interaction).
    pub fn step(&mut self) {
        let n = self.config.lattice_size;
        let dt = self.config.dt;
        let alpha = self.config.alpha;
        let mut next = self.field.clone();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let lap = self.laplacian(x, y, z);
                    let e8_int = self.e8_interaction(x, y, z);
                    let dphi = alpha * lap + e8_int;
                    next[x][y][z].magnitude += dt * dphi;
                    // Update root activations proportionally
                    for r in 0..self.roots.len() {
                        next[x][y][z].root_activations[r] += dt * alpha * lap * 0.01;
                    }
                }
            }
        }
        self.field = next;
        self.iteration += 1;
    }

    /// Run full PDE integration for configured number of steps.
    pub fn evolve(&mut self) {
        for _ in 0..self.config.num_steps {
            self.step();
        }
    }

    /// Compute total field energy.
    pub fn total_energy(&self) -> f64 {
        let n = self.config.lattice_size;
        let mut energy = 0.0;
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let m = self.field[x][y][z].magnitude;
                    energy += m * m;
                }
            }
        }
        energy
    }

    /// Compute the maximum field magnitude.
    pub fn max_magnitude(&self) -> f64 {
        let n = self.config.lattice_size;
        let mut max_val = 0.0_f64;
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    max_val = max_val.max(self.field[x][y][z].magnitude);
                }
            }
        }
        max_val
    }

    /// Average field magnitude.
    pub fn avg_magnitude(&self) -> f64 {
        let n = self.config.lattice_size;
        let total: f64 = self
            .field
            .iter()
            .flat_map(|plane| plane.iter().flat_map(|row| row.iter()))
            .map(|fv| fv.magnitude)
            .sum();
        total / (n * n * n) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_initialization() {
        let config = FieldSolverConfig::default();
        let solver = E8FieldSolver::new(config);
        assert_eq!(solver.lattice_size(), 8);
        assert!(!solver.roots.is_empty());
        assert!(!solver.structure_constants.is_empty());
    }

    #[test]
    fn test_field_initialization() {
        let config = FieldSolverConfig::default();
        let mut solver = E8FieldSolver::new(config);
        solver.initialize(1.0);
        assert!(solver.total_energy() > 0.0);
        assert!(solver.max_magnitude() > 0.0);
    }

    #[test]
    fn test_pde_step() {
        let config = FieldSolverConfig {
            lattice_size: 4,
            dt: 0.01,
            alpha: 0.1,
            num_steps: 10,
        };
        let mut solver = E8FieldSolver::new(config);
        solver.initialize(0.5);
        let energy_before = solver.total_energy();
        solver.step();
        let energy_after = solver.total_energy();
        // Energy should have changed
        assert!((energy_before - energy_after).abs() > 1e-12);
        assert_eq!(solver.iteration, 1);
    }

    #[test]
    fn test_full_evolution() {
        let config = FieldSolverConfig {
            lattice_size: 4,
            dt: 0.005,
            alpha: 0.05,
            num_steps: 20,
        };
        let mut solver = E8FieldSolver::new(config);
        solver.initialize(1.0);
        solver.evolve();
        assert_eq!(solver.iteration, 20);
        let avg = solver.avg_magnitude();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_laplacian_at_boundary() {
        let config = FieldSolverConfig {
            lattice_size: 4,
            dt: 0.01,
            alpha: 0.1,
            num_steps: 1,
        };
        let mut solver = E8FieldSolver::new(config);
        solver.initialize(1.0);
        // Corner point should have fewer neighbors — laplacian should work
        let lap = solver.laplacian(0, 0, 0);
        assert!(lap.is_finite());
    }

    #[test]
    fn test_structure_constant_count() {
        let config = FieldSolverConfig::default();
        let solver = E8FieldSolver::new(config);
        assert!(solver.structure_constant_count() > 0);
    }

    #[test]
    fn test_initial_energy_zero() {
        let config = FieldSolverConfig {
            lattice_size: 4,
            dt: 0.01,
            alpha: 0.1,
            num_steps: 1,
        };
        let solver = E8FieldSolver::new(config);
        // Without initialization, energy should be 0
        assert!((solver.total_energy()).abs() < 1e-12);
    }
}
