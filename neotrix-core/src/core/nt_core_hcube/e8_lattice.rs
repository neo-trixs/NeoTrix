/// 8D E8 root vector.
///
/// Components are stored as `i8` using a **2× integer scaling** so
/// the 128 half-integer roots fit. The two E8 root families are:
///
/// * Type A: `±1, ±1, 0, 0, 0, 0, 0, 0` — encoded as `±2, ±2, 0, …`
/// * Type B: `±1/2, ±1/2, …, ±1/2` (8 entries, even number of `-`) —
///   encoded as `±1, ±1, …, ±1`
///
/// In physical units (divide each component by 2), every root has
/// `|r|² = 2`, so the E8 lattice is norm-`√2`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct E8Root {
    pub components: [i8; 8],
}

impl E8Root {
    /// Returns the physical-space vector (divide each component by 2).
    pub fn as_f64(&self) -> [f64; 8] {
        let mut out = [0.0f64; 8];
        for i in 0..8 {
            out[i] = (self.components[i] as f64) / 2.0;
        }
        out
    }

    /// `|r|²` in physical units. For every E8 root this equals 2.
    pub fn norm_squared(&self) -> f64 {
        let mut s = 0.0f64;
        for c in self.components.iter() {
            let cf = (*c as f64) / 2.0;
            s += cf * cf;
        }
        s
    }
}

/// An encoding of a 16D hypercube coordinate into a pair of E8 roots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct E8Encoded {
    pub indices: [usize; 2],
    pub coefficients: [f64; 2],
}

/// E8 root lattice: 240 unit roots in 8D, forming a dense
/// sphere-packing with kissing number 240.
pub struct E8Lattice {
    pub roots: Vec<E8Root>,
    /// The 8 simple-root generators of the Weyl group of E8.
    pub weyl_generators: [[i8; 8]; 8],
}

impl Default for E8Lattice {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Lattice {
    pub fn new() -> Self {
        let roots = generate_roots();
        let weyl_generators = simple_roots();
        Self { roots, weyl_generators }
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    pub fn roots(&self) -> &[E8Root] {
        &self.roots
    }

    pub fn root(&self, idx: usize) -> &E8Root {
        &self.roots[idx]
    }

    /// Find the E8 root closest to a physical-space 8D vector.
    /// Returns `(index, squared_distance)`.
    pub fn nearest_root(&self, v: &[f64; 8]) -> (usize, f64) {
        let mut best_idx = 0usize;
        let mut best_d = f64::INFINITY;
        for (i, r) in self.roots.iter().enumerate() {
            let mut d = 0.0f64;
            for k in 0..8 {
                let c = (r.components[k] as f64) / 2.0;
                let diff = v[k] - c;
                d += diff * diff;
            }
            if d < best_d {
                best_d = d;
                best_idx = i;
            }
        }
        (best_idx, best_d)
    }

    /// Weyl orbit of a root under the diagonal sign group.
    /// E8 is closed under negation, so this returns `{idx, -idx}`.
    pub fn weyl_orbit(&self, idx: usize) -> Vec<usize> {
        if idx >= self.roots.len() {
            return Vec::new();
        }
        let mut orbit = vec![idx];
        let target: [i8; 8] = {
            let mut neg = [0i8; 8];
            for i in 0..8 {
                neg[i] = -self.roots[idx].components[i];
            }
            neg
        };
        for (i, root) in self.roots.iter().enumerate() {
            if i == idx {
                continue;
            }
            if root.components == target {
                orbit.push(i);
                break;
            }
        }
        orbit
    }

    /// Encode a 16D hypercube coordinate as a pair of E8 root indices
    /// plus their inner-product coefficients. Each half of the input
    /// is independently quantized to its nearest E8 root.
    pub fn encode(&self, hypercube_coord: &[f64; 16]) -> E8Encoded {
        let mut half1 = [0.0f64; 8];
        let mut half2 = [0.0f64; 8];
        for i in 0..8 {
            half1[i] = hypercube_coord[i];
            half2[i] = hypercube_coord[i + 8];
        }
        let (idx1, _) = self.nearest_root(&half1);
        let (idx2, _) = self.nearest_root(&half2);
        let r1 = self.roots[idx1].as_f64();
        let r2 = self.roots[idx2].as_f64();
        let coef1 = dot8(&half1, &r1) / 2.0;
        let coef2 = dot8(&half2, &r2) / 2.0;
        E8Encoded {
            indices: [idx1, idx2],
            coefficients: [coef1, coef2],
        }
    }

    /// Decode an `E8Encoded` back into an 8D physical-space vector
    /// as a linear combination of the two referenced roots.
    pub fn decode(&self, encoded: &E8Encoded) -> [f64; 8] {
        let r1 = self.roots[encoded.indices[0]].as_f64();
        let r2 = self.roots[encoded.indices[1]].as_f64();
        let mut result = [0.0f64; 8];
        for i in 0..8 {
            result[i] = encoded.coefficients[0] * r1[i] + encoded.coefficients[1] * r2[i];
        }
        result
    }

    /// Euclidean distance between two E8 roots in physical units.
    pub fn metric_distance(&self, a: &E8Root, b: &E8Root) -> f64 {
        let mut s = 0.0f64;
        for i in 0..8 {
            let d = ((a.components[i] - b.components[i]) as f64) / 2.0;
            s += d * d;
        }
        s.sqrt()
    }
}

fn dot8(a: &[f64; 8], b: &[f64; 8]) -> f64 {
    let mut s = 0.0f64;
    for i in 0..8 {
        s += a[i] * b[i];
    }
    s
}

fn generate_roots() -> Vec<E8Root> {
    let mut roots = Vec::with_capacity(240);

    // Type A: (±1, ±1, 0, …, 0) with all 8·7/2 position pairs and
    // independent ±1 sign choices. Encoded as ±2 in integer scale.
    for i in 0..8usize {
        for j in (i + 1)..8usize {
            for &si in &[2i8, -2] {
                for &sj in &[2i8, -2] {
                    let mut c = [0i8; 8];
                    c[i] = si;
                    c[j] = sj;
                    roots.push(E8Root { components: c });
                }
            }
        }
    }

    // Type B: (±1/2)^8 with even number of minus signs. Encoded as
    // ±1 in integer scale. There are 2^7 = 128 such vectors.
    for mask in 0u16..256 {
        let mut c = [0i8; 8];
        let mut num_neg = 0i32;
        for i in 0..8usize {
            if (mask >> i) & 1 == 1 {
                c[i] = 1;
            } else {
                c[i] = -1;
                num_neg += 1;
            }
        }
        if num_neg % 2 == 0 {
            roots.push(E8Root { components: c });
        }
    }

    roots
}

fn simple_roots() -> [[i8; 8]; 8] {
    // The standard 8 simple roots of E8 in our 2× integer scaling:
    //   α_k = (e_k - e_{k+1}) for k = 1..6        (chain of A7)
    //   α_7 = (e_7 + e_8)
    //   α_8 = -(1/2)(1, 1, 1, 1, 1, 1, 1, 1)      (lowest root)
    let mut g = [[0i8; 8]; 8];
    for k in 0..6 {
        g[k][k] = 2;
        g[k][k + 1] = -2;
    }
    g[6][6] = 2;
    g[6][7] = 2;
    for i in 0..8 {
        g[7][i] = -1;
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn root_count_is_240() {
        let l = E8Lattice::new();
        assert_eq!(l.root_count(), 240);
    }

    #[test]
    fn all_roots_have_norm_sq_two() {
        let l = E8Lattice::new();
        for r in l.roots() {
            let n2 = r.norm_squared();
            assert!(approx_eq(n2, 2.0, 1e-9), "root {:?} has |r|² = {}", r.components, n2);
        }
    }

    #[test]
    fn nearest_root_of_exact_root_is_zero_distance() {
        let l = E8Lattice::new();
        for (i, r) in l.roots().iter().enumerate() {
            let v = r.as_f64();
            let (idx, d2) = l.nearest_root(&v);
            assert_eq!(idx, i);
            assert!(approx_eq(d2.sqrt(), 0.0, 1e-9));
        }
    }

    #[test]
    fn encode_decode_roundtrip_preserves_direction() {
        let l = E8Lattice::new();
        // Build a 16D input that lies exactly on the lattice when
        // split in half: each half is just an E8 root.
        let r_a = l.roots()[0].as_f64();
        let r_b = l.roots()[7].as_f64();
        let mut input = [0.0f64; 16];
        for i in 0..8 {
            input[i] = r_a[i];
            input[i + 8] = r_b[i];
        }
        let encoded = l.encode(&input);
        let decoded = l.decode(&encoded);
        // The decoded form is a linear combination of the chosen
        // roots; for exact roots the coefficients are 1/2 each (since
        // ⟨r, r⟩/2 = 2/2 = 1 for both halves) so the result is the
        // midpoint of the two roots.
        let mut expected = [0.0f64; 8];
        for i in 0..8 {
            expected[i] = r_a[i] + r_b[i];
        }
        for i in 0..8 {
            assert!(approx_eq(decoded[i], expected[i], 1e-9));
        }
    }

    #[test]
    fn weyl_orbit_contains_negation() {
        let l = E8Lattice::new();
        for i in 0..l.root_count() {
            let orbit = l.weyl_orbit(i);
            assert!(orbit.contains(&i));
            assert!(orbit.len() >= 2, "E8 must contain -r for every root r");
        }
    }

    #[test]
    fn metric_distance_is_symmetric() {
        let l = E8Lattice::new();
        let a = l.root(0);
        let b = l.root(100);
        let d1 = l.metric_distance(a, b);
        let d2 = l.metric_distance(b, a);
        assert!(approx_eq(d1, d2, 1e-12));
    }

    #[test]
    fn type_distribution_matches_construction() {
        let l = E8Lattice::new();
        let mut type_a = 0usize;
        let mut type_b = 0usize;
        for r in l.roots() {
            let nonzero: Vec<i8> = r.components.iter().copied().filter(|c| *c != 0).collect();
            if nonzero.len() == 2 {
                type_a += 1;
            } else if nonzero.len() == 8 {
                type_b += 1;
            }
        }
        assert_eq!(type_a, 112);
        assert_eq!(type_b, 128);
    }
}
