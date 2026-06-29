use crate::core::nt_core_e8::{e8_root_system, E8_DIM, E8_RANK};

/// 3D lattice for field discretization
#[derive(Debug, Clone)]
pub struct Lattice3D {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub dx: f64,
}

impl Lattice3D {
    pub fn new(nx: usize, ny: usize, nz: usize, dx: f64) -> Self {
        Self {
            nx: nx.max(2),
            ny: ny.max(2),
            nz: nz.max(2),
            dx: dx.max(1e-6),
        }
    }

    pub fn total_points(&self) -> usize {
        self.nx * self.ny * self.nz
    }

    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        (i % self.nx) * self.ny * self.nz + (j % self.ny) * self.nz + (k % self.nz)
    }

    pub fn laplacian(&self, field: &[f64], i: usize, j: usize, k: usize) -> f64 {
        let inv_dx2 = 1.0 / (self.dx * self.dx);
        let center = field[self.index(i, j, k)];
        let neighbors = [
            field[self.index(i + 1, j, k)],
            field[self.index(i.wrapping_sub(1), j, k)],
            field[self.index(i, j + 1, k)],
            field[self.index(i, j.wrapping_sub(1), k)],
            field[self.index(i, j, k + 1)],
            field[self.index(i, j, k.wrapping_sub(1))],
        ];
        let sum: f64 = neighbors.iter().sum();
        (sum - 6.0 * center) * inv_dx2
    }

    /// Gradient magnitude at a point
    pub fn gradient_mag(&self, field: &[f64], i: usize, j: usize, k: usize) -> f64 {
        let inv_dx = 1.0 / (2.0 * self.dx);
        let dx_val = field[self.index(i + 1, j, k)] - field[self.index(i.wrapping_sub(1), j, k)];
        let dy_val = field[self.index(i, j + 1, k)] - field[self.index(i, j.wrapping_sub(1), k)];
        let dz_val = field[self.index(i, j, k + 1)] - field[self.index(i, j, k.wrapping_sub(1))];
        ((dx_val * dx_val + dy_val * dy_val + dz_val * dz_val).sqrt()) * inv_dx
    }
}

/// E8-derived Lagrangian density: computes L = T - V from field configuration
#[derive(Debug, Clone)]
pub struct E8Lagrangian {
    /// Structure constant coupling strength
    pub g_coupling: f64,
    /// Mass term coefficient
    pub mass_term: f64,
    /// Self-interaction strength (phi^4)
    pub lambda: f64,
    /// Pre-computed structure constants magnitude from E8
    pub structure_scale: f64,
}

impl Default for E8Lagrangian {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Lagrangian {
    pub fn new() -> Self {
        let roots = e8_root_system();
        let total_coupling: f64 = roots
            .iter()
            .flat_map(|r| r.coords.iter())
            .map(|&c| (c as f64).abs())
            .sum();
        let structure_scale = total_coupling / (E8_DIM * E8_RANK) as f64;
        Self {
            g_coupling: 0.1,
            mass_term: 1.0,
            lambda: 0.05,
            structure_scale,
        }
    }

    /// Kinetic term: T = 0.5 * (∇φ)²
    pub fn kinetic(&self, lattice: &Lattice3D, field: &[f64]) -> f64 {
        let mut t = 0.0;
        let mut count = 0;
        for i in 0..lattice.nx {
            for j in 0..lattice.ny {
                for k in 0..lattice.nz {
                    let grad = lattice.gradient_mag(field, i, j, k);
                    t += 0.5 * grad * grad;
                    count += 1;
                }
            }
        }
        t / count.max(1) as f64
    }

    /// Potential term: V = 0.5 * m² * φ² + λ/4! * φ⁴
    pub fn potential(&self, field: &[f64]) -> f64 {
        let n = field.len().max(1);
        let sum_phi2: f64 = field.iter().map(|&x| x * x).sum::<f64>() / n as f64;
        let sum_phi4: f64 = field.iter().map(|&x| x * x * x * x).sum::<f64>() / n as f64;
        0.5 * self.mass_term * sum_phi2 + self.lambda / 24.0 * sum_phi4
    }

    /// Lagrangian density: L = T - V
    pub fn density(&self, lattice: &Lattice3D, field: &[f64]) -> f64 {
        self.kinetic(lattice, field) - self.potential(field)
    }

    /// Action: S = ∫L d⁴x ≈ L * volume
    pub fn action(&self, lattice: &Lattice3D, field: &[f64]) -> f64 {
        let volume = lattice.nx as f64 * lattice.ny as f64 * lattice.nz as f64 * lattice.dx.powi(3);
        self.density(lattice, field) * volume
    }

    /// Equation of motion residual: □φ + ∂V/∂φ
    pub fn eom_residual(
        &self,
        lattice: &Lattice3D,
        field: &[f64],
        i: usize,
        j: usize,
        k: usize,
    ) -> f64 {
        let idx = lattice.index(i, j, k);
        let phi = field[idx];
        let laplacian = lattice.laplacian(field, i, j, k);
        let d_v_dphi = self.mass_term * phi + self.lambda / 6.0 * phi * phi * phi;
        -laplacian + d_v_dphi
    }

    /// E8 interaction term: adds structure-constant-weighted field coupling
    pub fn e8_interaction(&self, fields: &[Vec<f64>], lattice: &Lattice3D) -> Vec<f64> {
        let n_fields = fields.len();
        if n_fields == 0 {
            return Vec::new();
        }
        let n = lattice.total_points();
        let mut interaction = vec![0.0; n];
        for a in 0..n_fields.min(8) {
            for b in 0..n_fields.min(8) {
                let weight = self.g_coupling * self.structure_scale;
                for i in 0..n {
                    let fa = fields[a].get(i).copied().unwrap_or(0.0);
                    let fb = fields[b].get(i).copied().unwrap_or(0.0);
                    interaction[i] += weight * fa * fb;
                }
            }
        }
        interaction
    }
}

/// PDE solver: finite difference time evolution
#[derive(Debug, Clone)]
pub struct PDESolver {
    pub dt: f64,
    pub steps: usize,
}

impl Default for PDESolver {
    fn default() -> Self {
        Self {
            dt: 0.01,
            steps: 100,
        }
    }
}

impl PDESolver {
    pub fn new(dt: f64, steps: usize) -> Self {
        Self {
            dt: dt.max(1e-6),
            steps: steps.max(1),
        }
    }

    /// Solve wave equation: ∂²φ/∂t² = □φ - ∂V/∂φ
    pub fn solve_wave(
        &self,
        lattice: &Lattice3D,
        lagrangian: &E8Lagrangian,
        initial: &[f64],
        initial_velocity: &[f64],
    ) -> Vec<Vec<f64>> {
        let n = lattice.total_points();
        let mut phi = initial.to_vec();
        let mut velocity = initial_velocity.to_vec();
        let mut trajectory = Vec::with_capacity(self.steps + 1);
        trajectory.push(phi.clone());

        for _step in 0..self.steps {
            let mut phi_next = vec![0.0; n];
            for i in 0..lattice.nx {
                for j in 0..lattice.ny {
                    for k in 0..lattice.nz {
                        let idx = lattice.index(i, j, k);
                        let laplacian = lattice.laplacian(&phi, i, j, k);
                        let d_v = lagrangian.mass_term * phi[idx]
                            + lagrangian.lambda / 6.0 * phi[idx].powi(3);
                        let acceleration = laplacian - d_v;
                        velocity[idx] += acceleration * self.dt;
                        phi_next[idx] = phi[idx] + velocity[idx] * self.dt;
                    }
                }
            }
            phi = phi_next;
            trajectory.push(phi.clone());
        }
        trajectory
    }

    /// Solve diffusion equation: ∂φ/∂t = D∇²φ
    pub fn solve_diffusion(
        &self,
        lattice: &Lattice3D,
        diffusivity: f64,
        initial: &[f64],
    ) -> Vec<Vec<f64>> {
        let _n = lattice.total_points();
        let mut phi = initial.to_vec();
        let mut trajectory = Vec::with_capacity(self.steps + 1);
        trajectory.push(phi.clone());

        for _step in 0..self.steps {
            let mut phi_next = phi.clone();
            for i in 0..lattice.nx {
                for j in 0..lattice.ny {
                    for k in 0..lattice.nz {
                        let idx = lattice.index(i, j, k);
                        let laplacian = lattice.laplacian(&phi, i, j, k);
                        phi_next[idx] = phi[idx] + diffusivity * laplacian * self.dt;
                    }
                }
            }
            phi = phi_next;
            trajectory.push(phi.clone());
        }
        trajectory
    }
}

/// Full E8 field equation integrator
#[derive(Debug, Clone)]
pub struct E8FieldIntegrator {
    pub lagrangian: E8Lagrangian,
    pub solver: PDESolver,
    pub lattice: Lattice3D,
}

impl E8FieldIntegrator {
    pub fn new(lattice: Lattice3D, lagrangian: E8Lagrangian, solver: PDESolver) -> Self {
        Self {
            lagrangian,
            solver,
            lattice,
        }
    }

    /// Initialize a Gaussian wave packet in the field
    pub fn gaussian_initial(
        &self,
        center: (f64, f64, f64),
        sigma: f64,
        amplitude: f64,
    ) -> Vec<f64> {
        let n = self.lattice.total_points();
        let mut field = vec![0.0; n];
        let (cx, cy, cz) = center;
        for i in 0..self.lattice.nx {
            let x = i as f64 * self.lattice.dx;
            for j in 0..self.lattice.ny {
                let y = j as f64 * self.lattice.dx;
                for k in 0..self.lattice.nz {
                    let z = k as f64 * self.lattice.dx;
                    let r2 = (x - cx).powi(2) + (y - cy).powi(2) + (z - cz).powi(2);
                    let idx = self.lattice.index(i, j, k);
                    field[idx] = amplitude * (-r2 / (2.0 * sigma * sigma)).exp();
                }
            }
        }
        field
    }

    /// Zero initial velocity field
    pub fn zero_velocity(&self) -> Vec<f64> {
        vec![0.0; self.lattice.total_points()]
    }

    /// Run field evolution
    pub fn evolve(&self, initial: &[f64], initial_velocity: &[f64]) -> Vec<Vec<f64>> {
        self.solver
            .solve_wave(&self.lattice, &self.lagrangian, initial, initial_velocity)
    }

    /// Compute Lagrangian density at each timestep
    pub fn compute_density_trajectory(&self, trajectory: &[Vec<f64>]) -> Vec<f64> {
        trajectory
            .iter()
            .map(|field| self.lagrangian.density(&self.lattice, field))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_lattice() -> Lattice3D {
        Lattice3D::new(8, 8, 8, 0.5)
    }

    #[test]
    fn test_lattice_3d_basic() {
        let lattice = make_test_lattice();
        assert_eq!(lattice.total_points(), 512);
        assert!(lattice.dx > 0.0);
    }

    #[test]
    fn test_lattice_laplacian_uniform() {
        let lattice = make_test_lattice();
        let field = vec![1.0; lattice.total_points()];
        let lap = lattice.laplacian(&field, 1, 1, 1);
        assert!(
            (lap).abs() < 1e-12,
            "uniform field should have zero laplacian"
        );
    }

    #[test]
    fn test_lattice_gradient() {
        let lattice = make_test_lattice();
        let mut field = vec![0.0; lattice.total_points()];
        for i in 0..lattice.nx {
            for j in 0..lattice.ny {
                for k in 0..lattice.nz {
                    let idx = lattice.index(i, j, k);
                    field[idx] = i as f64;
                }
            }
        }
        let grad = lattice.gradient_mag(&field, 4, 4, 4);
        assert!(
            (grad - 1.0).abs() < 0.1,
            "linear gradient should have mag ~1, got {}",
            grad
        );
    }

    #[test]
    fn test_e8_lagrangian_default() {
        let lag = E8Lagrangian::new();
        assert!(lag.structure_scale > 0.0);
        assert!((lag.g_coupling - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_lagrangian_kinetic_uniform() {
        let lattice = make_test_lattice();
        let lag = E8Lagrangian::new();
        let field = vec![1.0; lattice.total_points()];
        let kinetic = lag.kinetic(&lattice, &field);
        assert!(
            kinetic < 1e-12,
            "uniform field should have near-zero kinetic energy"
        );
    }

    #[test]
    fn test_lagrangian_potential() {
        let lag = E8Lagrangian::new();
        let field = vec![1.0; 64];
        let potential = lag.potential(&field);
        assert!(potential > 0.0);
    }

    #[test]
    fn test_pde_solver_wave() {
        let lattice = make_test_lattice();
        let lag = E8Lagrangian::new();
        let solver = PDESolver::new(0.01, 10);
        let initial = vec![0.1; lattice.total_points()];
        let velocity = vec![0.0; lattice.total_points()];
        let trajectory = solver.solve_wave(&lattice, &lag, &initial, &velocity);
        assert_eq!(trajectory.len(), 11);
        assert_eq!(trajectory[0].len(), lattice.total_points());
    }

    #[test]
    fn test_pde_solver_diffusion() {
        let lattice = make_test_lattice();
        let solver = PDESolver::new(0.001, 5);
        let initial = vec![0.0; lattice.total_points()];
        // Set a peak in the center
        let center_idx = lattice.index(4, 4, 4);
        let mut initial2 = initial.clone();
        initial2[center_idx] = 1.0;
        let trajectory = solver.solve_diffusion(&lattice, 0.1, &initial2);
        assert_eq!(trajectory.len(), 6);
    }

    #[test]
    fn test_field_integrator_gaussian() {
        let lattice = make_test_lattice();
        let lag = E8Lagrangian::new();
        let solver = PDESolver::default();
        let integrator = E8FieldIntegrator::new(lattice, lag, solver);
        let gaussian = integrator.gaussian_initial((2.0, 2.0, 2.0), 1.0, 1.0);
        assert_eq!(gaussian.len(), 512);
        let max_val = gaussian.iter().fold(0.0_f64, |a, &b| a.max(b));
        assert!((max_val - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_field_integrator_evolve() {
        let lattice = make_test_lattice();
        let lag = E8Lagrangian::new();
        let solver = PDESolver::new(0.01, 5);
        let integrator = E8FieldIntegrator::new(lattice, lag, solver);
        let initial = integrator.gaussian_initial((2.0, 2.0, 2.0), 0.5, 0.5);
        let velocity = integrator.zero_velocity();
        let trajectory = integrator.evolve(&initial, &velocity);
        assert_eq!(trajectory.len(), 6);
    }

    #[test]
    fn test_density_trajectory() {
        let lattice = make_test_lattice();
        let lag = E8Lagrangian::new();
        let integrator =
            E8FieldIntegrator::new(lattice.clone(), lag.clone(), PDESolver::new(0.01, 3));
        let initial = integrator.gaussian_initial((2.0, 2.0, 2.0), 0.5, 0.5);
        let velocity = integrator.zero_velocity();
        let trajectory = integrator.evolve(&initial, &velocity);
        let densities = integrator.compute_density_trajectory(&trajectory);
        assert_eq!(densities.len(), trajectory.len());
    }

    #[test]
    fn test_e8_interaction() {
        let lattice = make_test_lattice();
        let lag = E8Lagrangian::new();
        let fields = vec![vec![0.5; lattice.total_points()]; 4];
        let interaction = lag.e8_interaction(&fields, &lattice);
        assert_eq!(interaction.len(), lattice.total_points());
    }

    #[test]
    fn test_eom_residual() {
        let lattice = make_test_lattice();
        let lag = E8Lagrangian::new();
        let field = vec![0.1; lattice.total_points()];
        let residual = lag.eom_residual(&lattice, &field, 1, 1, 1);
        assert!(residual.abs() > 0.0);
    }
}
