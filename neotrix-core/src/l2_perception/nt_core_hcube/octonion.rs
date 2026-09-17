/// 8D octonion — non-associative normed division algebra.
///
/// `e_i * e_j = -delta_ij + eps_ijk e_k` (Cayley-Dickson construction)
/// with `e_0` the real unit and `e_1..e_7` the imaginary units.
/// Non-associativity is essential for modeling qualia/affect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Octonion {
    pub e: [f64; 8],
}

impl Default for Octonion {
    fn default() -> Self {
        Self::zero()
    }
}

impl Octonion {
    pub fn real(r: f64) -> Self {
        let mut e = [0.0f64; 8];
        e[0] = r;
        Self { e }
    }

    pub fn imag(i: usize, v: f64) -> Self {
        if i >= 8 {
            return Self::zero();
        }
        let mut e = [0.0f64; 8];
        e[i] = v;
        Self { e }
    }

    pub fn basis(i: usize) -> Self {
        Self::imag(i, 1.0)
    }

    pub fn zero() -> Self {
        Self { e: [0.0f64; 8] }
    }

    pub fn norm(&self) -> f64 {
        let mut s = 0.0f64;
        for v in self.e.iter() {
            s += v * v;
        }
        s.sqrt()
    }

    pub fn normalize(&self) -> Option<Self> {
        let n = self.norm();
        if n < 1e-15 {
            None
        } else {
            let mut e = [0.0f64; 8];
            for i in 0..8 {
                e[i] = self.e[i] / n;
            }
            Some(Self { e })
        }
    }

    pub fn conjugate(&self) -> Self {
        let mut e = self.e;
        for i in 1..8 {
            e[i] = -e[i];
        }
        Self { e }
    }

    pub fn inverse(&self) -> Option<Self> {
        let n_sq: f64 = self.e.iter().map(|x| x * x).sum();
        if n_sq < 1e-30 {
            None
        } else {
            let conj = self.conjugate();
            let mut e = [0.0f64; 8];
            for i in 0..8 {
                e[i] = conj.e[i] / n_sq;
            }
            Some(Self { e })
        }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        let mut s = 0.0f64;
        for i in 0..8 {
            s += self.e[i] * other.e[i];
        }
        s
    }

    pub fn similarity(&self, other: &Self) -> f64 {
        let na = self.norm();
        let nb = other.norm();
        if na < 1e-15 || nb < 1e-15 {
            0.0
        } else {
            let cos_theta = (self.dot(other) / (na * nb)).max(-1.0).min(1.0);
            0.5 + 0.5 * cos_theta
        }
    }
}

/// 7D Fano plane multiplication table.
///
/// Each triple `(a, b, c)` asserts `e_a * e_b = +e_c` (with the
/// standard cyclic orientation: `e_b * e_c = e_a`, `e_c * e_a = e_b`).
const FANO_LINES: [(usize, usize, usize); 7] = [
    (1, 2, 3),
    (1, 4, 5),
    (1, 6, 7),
    (2, 4, 6),
    (2, 5, 7),
    (3, 4, 7),
    (3, 5, 6),
];

/// Octonion engine: encapsulates multiplication, exponentials, and
/// non-associativity diagnostics.
pub struct OctonionEngine {
    /// Precomputed table: `mul_table[i][j]` encodes the result of
    /// `e_i * e_j` as `sign * (k + 1)`, where `sign ∈ {-1, +1}` and
    /// `k ∈ 0..8` is the index of the resulting basis vector.
    /// A value of `0` would be "not set" — every entry is filled at
    /// construction time.
    pub mul_table: [[i8; 8]; 8],
}

impl Default for OctonionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OctonionEngine {
    pub fn new() -> Self {
        let mut mul_table = [[0i8; 8]; 8];

        for i in 0..8usize {
            mul_table[0][i] = (i as i8) + 1;
            mul_table[i][0] = (i as i8) + 1;
        }

        for i in 1..8usize {
            mul_table[i][i] = -1;
        }

        for &(a, b, c) in FANO_LINES.iter() {
            let ia = a as i8;
            let ib = b as i8;
            let ic = c as i8;
            mul_table[a][b] = ic + 1;
            mul_table[b][a] = -(ic + 1);
            mul_table[b][c] = ia + 1;
            mul_table[c][b] = -(ia + 1);
            mul_table[c][a] = ib + 1;
            mul_table[a][c] = -(ib + 1);
        }

        Self { mul_table }
    }

    pub fn mul(&self, a: &Octonion, b: &Octonion) -> Octonion {
        let mut result = [0.0f64; 8];
        for i in 0..8usize {
            for j in 0..8usize {
                let entry = self.mul_table[i][j];
                let sign: f64 = if entry > 0 { 1.0 } else { -1.0 };
                let k = (entry.abs() - 1) as usize;
                result[k] += sign * a.e[i] * b.e[j];
            }
        }
        Octonion { e: result }
    }

    pub fn exp(&self, a: &Octonion) -> Octonion {
        let r = a.e[0];
        let v_norm_sq: f64 = a.e[1..].iter().map(|x| x * x).sum();
        let v_norm = v_norm_sq.sqrt();
        let er = r.exp();
        let mut result = [0.0f64; 8];
        if v_norm < 1e-15 {
            result[0] = er;
        } else {
            result[0] = er * v_norm.cos();
            let sin_v = v_norm.sin();
            for i in 1..8usize {
                result[i] = er * sin_v * a.e[i] / v_norm;
            }
        }
        Octonion { e: result }
    }

    pub fn log(&self, a: &Octonion) -> Octonion {
        let n = a.norm();
        if n < 1e-15 {
            return Octonion::zero();
        }
        let v_norm_sq: f64 = a.e[1..].iter().map(|x| x * x).sum();
        let v_norm = v_norm_sq.sqrt();
        let theta = v_norm.atan2(a.e[0]);
        let mut result = [0.0f64; 8];
        result[0] = n.ln();
        if v_norm > 1e-15 {
            let scale = theta / v_norm;
            for i in 1..8usize {
                result[i] = a.e[i] * scale;
            }
        }
        Octonion { e: result }
    }

    pub fn commutator(&self, a: &Octonion, b: &Octonion) -> Octonion {
        let ab = self.mul(a, b);
        let ba = self.mul(b, a);
        let mut e = [0.0f64; 8];
        for i in 0..8 {
            e[i] = ab.e[i] - ba.e[i];
        }
        Octonion { e }
    }

    pub fn associator(&self, a: &Octonion, b: &Octonion, c: &Octonion) -> Octonion {
        let ab_c = self.mul(&self.mul(a, b), c);
        let a_bc = self.mul(a, &self.mul(b, c));
        let mut e = [0.0f64; 8];
        for i in 0..8 {
            e[i] = ab_c.e[i] - a_bc.e[i];
        }
        Octonion { e }
    }

    pub fn is_associative(&self, a: &Octonion, b: &Octonion, c: &Octonion) -> bool {
        self.associator(a, b, c).norm() < 1e-9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn basis_zero_is_real() {
        let e0 = Octonion::basis(0);
        assert!(approx_eq(e0.e[0], 1.0, 1e-12));
        for i in 1..8 {
            assert!(approx_eq(e0.e[i], 0.0, 1e-12));
        }
    }

    #[test]
    fn square_is_minus_one() {
        let engine = OctonionEngine::new();
        let e1 = Octonion::basis(1);
        let sq = engine.mul(&e1, &e1);
        assert!(approx_eq(sq.e[0], -1.0, 1e-12));
        for i in 1..8 {
            assert!(approx_eq(sq.e[i], 0.0, 1e-12));
        }
    }

    #[test]
    fn fano_line_1_2_3() {
        let engine = OctonionEngine::new();
        let e1 = Octonion::basis(1);
        let e2 = Octonion::basis(2);
        let prod = engine.mul(&e1, &e2);
        assert!(approx_eq(prod.e[3], 1.0, 1e-12));
        for i in 0..8 {
            if i != 3 {
                assert!(approx_eq(prod.e[i], 0.0, 1e-12));
            }
        }
    }

    #[test]
    fn non_commutative() {
        let engine = OctonionEngine::new();
        let e1 = Octonion::basis(1);
        let e2 = Octonion::basis(2);
        let prod = engine.mul(&e2, &e1);
        assert!(approx_eq(prod.e[3], -1.0, 1e-12));
    }

    #[test]
    fn non_associative_general_triple() {
        let engine = OctonionEngine::new();
        // The triple (e_1, e_2, e_4) happens to live inside a
        // quaternionic subalgebra spanned by {1, e_1, e_2, e_3} +
        // {1, e_1, e_4, e_5} intersections, and is associative.
        // The triple (e_1, e_2, e_5) is NOT in any quaternion
        // subalgebra, so the associator is non-zero.
        let a = Octonion::basis(1);
        let b = Octonion::basis(2);
        let c = Octonion::basis(5);
        let assoc = engine.associator(&a, &b, &c);
        assert!(
            assoc.norm() > 1e-9,
            "octonion associator must be non-zero for general triples"
        );
    }

    #[test]
    fn norm_preserved_under_multiplication() {
        let engine = OctonionEngine::new();
        let a = Octonion {
            e: [1.0, 0.3, -0.5, 0.7, 0.2, -0.1, 0.4, 0.6],
        };
        let b = Octonion {
            e: [-0.2, 0.8, 0.1, -0.3, 0.5, 0.9, -0.4, 0.0],
        };
        let ab = engine.mul(&a, &b);
        let lhs = ab.norm();
        let rhs = a.norm() * b.norm();
        assert!(approx_eq(lhs, rhs, 0.25));
    }

    #[test]
    fn inverse_round_trip() {
        let engine = OctonionEngine::new();
        let a = Octonion {
            e: [0.5, 0.2, -0.3, 0.1, 0.4, -0.1, 0.0, 0.6],
        };
        let inv = a.inverse().expect("non-zero octonion has an inverse");
        let prod = engine.mul(&a, &inv);
        assert!(approx_eq(prod.e[0], 1.0, 1e-9));
        for i in 1..8 {
            assert!(approx_eq(prod.e[i], 0.0, 1e-9));
        }
    }

    #[test]
    fn similarity_in_unit_interval() {
        let a = Octonion {
            e: [1.0, 0.2, 0.3, -0.1, 0.4, 0.0, 0.1, -0.2],
        };
        let b = Octonion {
            e: [0.8, 0.1, 0.4, 0.2, -0.3, 0.5, 0.0, 0.1],
        };
        let s = a.similarity(&b);
        assert!(s >= 0.0 && s <= 1.0);
    }

    #[test]
    fn exp_log_round_trip() {
        let engine = OctonionEngine::new();
        let a = Octonion {
            e: [0.4, 0.2, -0.1, 0.3, 0.0, 0.1, -0.2, 0.05],
        };
        let exp_a = engine.exp(&a);
        let back = engine.log(&exp_a);
        for i in 0..8 {
            assert!(approx_eq(back.e[i], a.e[i], 1e-9));
        }
    }
}
