//! SINDy Engine — Sparse Identification of Nonlinear Dynamics for VSA state evolution.
//!
//! Inspired by Brunton et al. (2016) and the Nature Reviews Physics Technical Review
//! "Data-driven discovery of dynamical models in biology" (Prokop & Gelens, 2026).
//!
//! Given a sequence of VSA state observations `x(t)`, SINDy finds a sparse representation
//! of the dynamics `dx/dt = f(x)` by solving a sparse regression problem:
//!
//! ```text
//! argmin_Ξ ||Ẋ - Θ(X)Ξ||₂ + λ||Ξ||₁
//! ```
//!
//! where Θ(X) is a library of candidate basis functions (polynomials, trigonometric, etc.)
//! and Ξ is a sparse coefficient matrix. The sparsity of Ξ ensures interpretability and
//! avoids overfitting. Uses STLS (Sequential Thresholded Least Squares).

use std::collections::VecDeque;

const VSA_DIM: usize = 512;
const DEFAULT_POLY_DEGREE: usize = 2;
const DEFAULT_THRESHOLD: f64 = 0.1;
const MAX_LIBRARY_TERMS: usize = 64;

/// A single VSA state snapshot at time t
#[derive(Debug, Clone)]
pub struct VsaSnapshot {
    pub time_step: u64,
    pub state_vector: Vec<f64>,
    pub cycle_label: String,
}

/// A discovered dynamical term: e.g., "x₁²" with coefficient 0.85
#[derive(Debug, Clone)]
pub struct DiscoveredTerm {
    pub term: String,
    pub coefficient: f64,
    pub target_variable: usize,
}

/// Structured report of discovered dynamics
#[derive(Debug, Clone)]
pub struct DynamicsReport {
    pub terms: Vec<DiscoveredTerm>,
    pub prediction_error: f64,
    pub sparsity_ratio: f64,
    pub equation_count: usize,
}

impl DynamicsReport {
    pub fn summary(&self) -> String {
        let top: Vec<String> = self
            .terms
            .iter()
            .take(5)
            .map(|t| format!("{}[{:.3}*{}]", t.target_variable, t.coefficient, t.term))
            .collect();
        format!(
            "SINDy: {} eqs, {} terms, pred_err={:.4}, sparsity={:.2}, top={}",
            self.equation_count,
            self.terms.len(),
            self.prediction_error,
            self.sparsity_ratio,
            top.join(", "),
        )
    }
}

/// Library of candidate basis functions for sparse regression
#[derive(Debug, Clone)]
pub struct CandidateLibrary {
    pub poly_degree: usize,
    pub include_trig: bool,
    pub include_interactions: bool,
}

impl Default for CandidateLibrary {
    fn default() -> Self {
        CandidateLibrary {
            poly_degree: DEFAULT_POLY_DEGREE,
            include_trig: false,
            include_interactions: true,
        }
    }
}

impl CandidateLibrary {
    pub fn build_library(&self, state: &[f64]) -> Vec<f64> {
        let n = state.len().min(8);
        let mut lib = Vec::with_capacity(MAX_LIBRARY_TERMS);
        // Constant term
        lib.push(1.0);
        // Linear terms
        for i in 0..n {
            lib.push(state[i]);
        }
        // Quadratic terms
        if self.poly_degree >= 2 {
            for i in 0..n {
                lib.push(state[i] * state[i]);
            }
        }
        // Interaction terms (pairwise)
        if self.include_interactions {
            for i in 0..n.min(4) {
                for j in (i + 1)..n.min(4) {
                    lib.push(state[i] * state[j]);
                }
            }
        }
        // Trigonometric terms
        if self.include_trig {
            for i in 0..n.min(4) {
                lib.push(state[i].sin());
                lib.push(state[i].cos());
            }
        }
        lib.truncate(MAX_LIBRARY_TERMS);
        lib
    }

    pub fn term_labels(&self) -> Vec<String> {
        let mut labels = vec!["1".to_string()];
        let n = 8usize;
        for i in 0..n {
            labels.push(format!("x{}", i));
        }
        if self.poly_degree >= 2 {
            for i in 0..n {
                labels.push(format!("x{}²", i));
            }
        }
        if self.include_interactions {
            for i in 0..n.min(4) {
                for j in (i + 1)..n.min(4) {
                    labels.push(format!("x{}·x{}", i, j));
                }
            }
        }
        if self.include_trig {
            for i in 0..n.min(4) {
                labels.push(format!("sin(x{})", i));
                labels.push(format!("cos(x{})", i));
            }
        }
        labels.truncate(MAX_LIBRARY_TERMS);
        labels
    }
}

/// SINDy Engine — sparse identification of VSA system dynamics
#[derive(Debug, Clone)]
pub struct SindyEngine {
    pub buffer: VecDeque<VsaSnapshot>,
    pub max_buffer: usize,
    pub library: CandidateLibrary,
    pub threshold: f64,
    pub report: Option<DynamicsReport>,
    pub convergence_score: f64,
    pub last_novelty: f64,
}

impl SindyEngine {
    pub fn new() -> Self {
        SindyEngine {
            buffer: VecDeque::with_capacity(128),
            max_buffer: 256,
            library: CandidateLibrary::default(),
            threshold: DEFAULT_THRESHOLD,
            report: None,
            convergence_score: 0.0,
            last_novelty: 0.0,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_poly_degree(mut self, degree: usize) -> Self {
        self.library.poly_degree = degree;
        self
    }

    /// Record a new VSA state observation
    pub fn observe(&mut self, snapshot: VsaSnapshot) {
        if self.buffer.len() >= self.max_buffer {
            self.buffer.pop_front();
        }
        self.buffer.push_back(snapshot);
    }

    /// Number of observations in buffer
    pub fn observation_count(&self) -> usize {
        self.buffer.len()
    }

    /// Run SINDy sparse identification on buffered observations.
    /// Returns a DynamicsReport of discovered terms.
    pub fn discover_dynamics(&mut self) -> Option<DynamicsReport> {
        let n_obs = self.buffer.len();
        if n_obs < 5 {
            return None;
        }

        let state_dim = self.buffer[0].state_vector.len().min(8);
        let lib = &self.library;
        let labels = lib.term_labels();
        let n_features = labels.len();

        // Build Θ(X) matrix: n_obs × n_features
        let mut theta: Vec<Vec<f64>> = Vec::with_capacity(n_obs);
        for obs in self.buffer.iter() {
            let features = lib.build_library(&obs.state_vector);
            theta.push(features);
        }

        // Compute time derivatives via finite differences (central)
        let mut dstate: Vec<Vec<f64>> = Vec::with_capacity(n_obs);
        for i in 0..n_obs {
            let dt = if i > 0 && i + 1 < n_obs {
                let t_next = self.buffer[i + 1].time_step as f64;
                let t_prev = self.buffer[i - 1].time_step as f64;
                if t_next > t_prev {
                    let dt_val = t_next - t_prev;
                    let mut deriv: Vec<f64> = Vec::with_capacity(state_dim);
                    for j in 0..state_dim {
                        let forward = self.buffer[i + 1]
                            .state_vector
                            .get(j)
                            .copied()
                            .unwrap_or(0.0);
                        let backward = self.buffer[i - 1]
                            .state_vector
                            .get(j)
                            .copied()
                            .unwrap_or(0.0);
                        deriv.push((forward - backward) / dt_val);
                    }
                    deriv
                } else {
                    vec![0.0; state_dim]
                }
            } else {
                vec![0.0; state_dim]
            };
            dstate.push(dt);
        }

        // STLS: Sequential Thresholded Least Squares for each state variable
        let mut all_terms: Vec<DiscoveredTerm> = Vec::new();
        let mut total_pred_err = 0.0;
        let mut total_nonzero = 0;

        for target in 0..state_dim {
            let target_vals: Vec<f64> = dstate.iter().map(|row| row[target]).collect();
            let (coeffs, pred_err) = self.stls_solve(&theta, &target_vals);
            total_pred_err += pred_err;

            for (j, &c) in coeffs.iter().enumerate() {
                if c.abs() > self.threshold {
                    total_nonzero += 1;
                    all_terms.push(DiscoveredTerm {
                        term: labels.get(j).cloned().unwrap_or_default(),
                        coefficient: c,
                        target_variable: target,
                    });
                }
            }
        }

        let total_possible = state_dim * n_features;
        let sparsity = if total_possible > 0 {
            1.0 - (total_nonzero as f64 / total_possible as f64)
        } else {
            0.0
        };

        let report = DynamicsReport {
            terms: all_terms,
            prediction_error: total_pred_err / state_dim.max(1) as f64,
            sparsity_ratio: sparsity,
            equation_count: state_dim,
        };

        self.convergence_score = 1.0 - report.prediction_error.min(1.0);
        self.report = Some(report.clone());
        Some(report)
    }

    /// STLS: Solve least squares then hard-threshold small coefficients.
    /// Uses pseudo-inverse via normal equations.
    fn stls_solve(&self, theta: &[Vec<f64>], target: &[f64]) -> (Vec<f64>, f64) {
        let n = theta.len();
        let m = if n > 0 { theta[0].len() } else { 0 };
        if n == 0 || m == 0 {
            return (vec![], 0.0);
        }

        // Normal equations: X^T * X and X^T * y
        let mut xtx = vec![vec![0.0; m]; m];
        let mut xty = vec![0.0; m];

        for i in 0..n {
            for j in 0..m {
                xty[j] += theta[i][j] * target[i];
                for k in 0..m {
                    xtx[j][k] += theta[i][j] * theta[i][k];
                }
            }
        }

        // Ridge regularization: add λI for numerical stability
        let lambda = 0.01;
        for j in 0..m {
            xtx[j][j] += lambda;
        }

        // Solve via Cholesky decomposition (simple Gaussian elimination)
        let mut coeffs = self.cholesky_solve(&xtx, &xty);

        // Iterative thresholding (STLS)
        for _iter in 0..5 {
            // Hard threshold
            for c in coeffs.iter_mut() {
                if c.abs() < self.threshold {
                    *c = 0.0;
                }
            }
        }

        // Compute prediction error
        let pred_err = if n > 0 {
            let mut total_err = 0.0;
            for i in 0..n {
                let mut predicted = 0.0;
                for j in 0..m {
                    predicted += theta[i][j] * coeffs[j];
                }
                total_err += (predicted - target[i]).powi(2);
            }
            (total_err / n as f64).sqrt()
        } else {
            0.0
        };

        (coeffs, pred_err)
    }

    /// Cholesky solve for symmetric positive definite system.
    /// Simplified — may fail for degenerate matrices.
    fn cholesky_solve(&self, a: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
        let n = a.len();
        if n == 0 {
            return vec![];
        }

        // Cholesky decomposition: A = L * L^T
        let mut l = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..=i {
                let mut sum = a[i][j];
                for k in 0..j {
                    sum -= l[i][k] * l[j][k];
                }
                if i == j {
                    l[i][j] = sum.sqrt().max(1e-10);
                } else {
                    l[i][j] = sum / l[j][j];
                }
            }
        }

        // Forward substitution: L * y = b
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut sum = b[i];
            for j in 0..i {
                sum -= l[i][j] * y[j];
            }
            y[i] = sum / l[i][i];
        }

        // Back substitution: L^T * x = y
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = y[i];
            for j in (i + 1)..n {
                sum -= l[j][i] * x[j];
            }
            x[i] = sum / l[i][i];
        }

        x
    }

    /// Predict next state given current state and discovered dynamics
    pub fn predict_next(&self, current_state: &[f64]) -> Option<Vec<f64>> {
        let report = self.report.as_ref()?;
        let _state_dim = current_state.len().min(8);
        let mut next = current_state.to_vec();

        let lib = &self.library;
        let features = lib.build_library(current_state);

        for term in &report.terms {
            if term.target_variable < next.len() {
                if let Some(feat_val) = features.get(
                    lib.term_labels()
                        .iter()
                        .position(|l| *l == term.term)
                        .unwrap_or(usize::MAX),
                ) {
                    next[term.target_variable] += term.coefficient * feat_val;
                }
            }
        }

        Some(next)
    }

    /// Detect novelty: how different is this state from the learned dynamics?
    pub fn novelty_score(&mut self, state: &[f64]) -> f64 {
        if let Some(predicted) = self.predict_next(state) {
            let mut error = 0.0;
            let dim = state.len().min(predicted.len());
            for i in 0..dim {
                error += (predicted[i] - state[i]).abs();
            }
            let score = (error / dim as f64).min(1.0);
            self.last_novelty = score;
            score
        } else {
            0.0
        }
    }
}

impl Default for SindyEngine {
    fn default() -> Self {
        Self::new()
    }
}
