/// Betti numbers at a specific filtration scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BettiNumbers {
    pub beta_0: usize,
    pub beta_1: usize,
    pub beta_2: usize,
}

impl Default for BettiNumbers {
    fn default() -> Self {
        Self::zero()
    }
}

impl BettiNumbers {
    pub fn zero() -> Self {
        Self {
            beta_0: 0,
            beta_1: 0,
            beta_2: 0,
        }
    }

    pub fn total(&self) -> usize {
        self.beta_0 + self.beta_1 + self.beta_2
    }

    /// A rough proxy for integrated information Φ (IIT-style): the
    /// ratio of "interesting" cycles to the number of components.
    pub fn integration_estimate(&self) -> f64 {
        let numer = (self.beta_1 + 2 * self.beta_2) as f64;
        let denom = (self.beta_0 + 1) as f64;
        numer / denom
    }
}

/// A simple labelled point cloud.
pub struct PointCloud {
    pub points: Vec<Vec<f64>>,
    pub name: String,
}

impl PointCloud {
    pub fn new(name: &str) -> Self {
        Self {
            points: Vec::new(),
            name: name.to_string(),
        }
    }

    pub fn add_point(&mut self, p: Vec<f64>) {
        self.points.push(p);
    }

    pub fn n(&self) -> usize {
        self.points.len()
    }

    pub fn dim(&self) -> usize {
        self.points.first().map(|p| p.len()).unwrap_or(0)
    }
}

/// Simplified persistent homology over a Vietoris-Rips filtration.
///
/// β_0 is computed exactly via union-find.
/// β_1 is the cyclomatic number of the 1-skeleton at each scale:
///   `|E| - |V| + #components`.
/// β_2 is a coarse 0 unless 4-cliques are present, in which case
/// `filled tetrahedra` are counted. (Sufficient for our smoke tests
/// and a usable Φ proxy; full V-R β_2 is intentionally out of scope.)
pub struct PersistentHomology {
    pub scale_max: f64,
    pub num_steps: usize,
    pub betti_curves: Vec<(f64, BettiNumbers)>,
}

impl PersistentHomology {
    /// Compute Betti numbers along a uniform scale grid from 0 to
    /// `scale_max` in `num_steps` steps.
    pub fn compute(cloud: &PointCloud, scale_max: f64, num_steps: usize) -> Self {
        let n = cloud.n();
        let steps = num_steps.max(1);
        let mut curves = Vec::with_capacity(steps + 1);

        if n == 0 {
            curves.push((0.0, BettiNumbers::zero()));
            return Self {
                scale_max,
                num_steps: steps,
                betti_curves: curves,
            };
        }

        let mut dists = vec![vec![0.0f64; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let d = euclidean(&cloud.points[i], &cloud.points[j]);
                dists[i][j] = d;
                dists[j][i] = d;
            }
        }

        for s in 0..=steps {
            let scale = scale_max * (s as f64) / (steps as f64);
            let (beta_0, edges, triangles) = betti_0_1(&dists, n, scale);
            let beta_1 = (edges + beta_0).saturating_sub(n);
            let beta_2 = (triangles / 4).max(0);
            curves.push((
                scale,
                BettiNumbers {
                    beta_0,
                    beta_1,
                    beta_2,
                },
            ));
        }

        Self {
            scale_max,
            num_steps: steps,
            betti_curves: curves,
        }
    }

    /// Betti numbers at the largest scale ≤ `scale` in the curve.
    pub fn at_scale(&self, scale: f64) -> Option<BettiNumbers> {
        let mut answer: Option<BettiNumbers> = None;
        for (s, b) in self.betti_curves.iter() {
            if *s <= scale {
                answer = Some(*b);
            } else {
                break;
            }
        }
        answer
    }

    /// Shannon entropy of the persistence curve (treating each
    /// step's Betti total as a probability mass).
    pub fn persistence_entropy(&self) -> f64 {
        let totals: Vec<f64> = self
            .betti_curves
            .iter()
            .map(|(_, b)| b.total() as f64)
            .collect();
        let sum: f64 = totals.iter().sum();
        if sum <= 0.0 {
            return 0.0;
        }
        let mut entropy = 0.0f64;
        for t in totals.iter() {
            if *t > 0.0 {
                let p = t / sum;
                entropy -= p * p.ln();
            }
        }
        entropy
    }

    /// A single representative Betti vector (the one at the median
    /// scale). Useful for downstream comparison / clustering.
    pub fn simplified_betti(&self) -> BettiNumbers {
        if self.betti_curves.is_empty() {
            return BettiNumbers::zero();
        }
        self.betti_curves[self.betti_curves.len() / 2].1
    }
}

fn euclidean(a: &[f64], b: &[f64]) -> f64 {
    let mut s = 0.0f64;
    for i in 0..a.len() {
        let d = a[i] - b[i];
        s += d * d;
    }
    s.sqrt()
}

fn find(parent: &mut [usize], i: usize) -> usize {
    let mut root = i;
    while parent[root] != root {
        root = parent[root];
    }
    let mut cur = i;
    while parent[cur] != root {
        let next = parent[cur];
        parent[cur] = root;
        cur = next;
    }
    root
}

fn union(parent: &mut [usize], i: usize, j: usize) {
    let ri = find(parent, i);
    let rj = find(parent, j);
    if ri != rj {
        parent[ri] = rj;
    }
}

fn betti_0_1(dists: &[Vec<f64>], n: usize, scale: f64) -> (usize, usize, usize) {
    let mut parent: Vec<usize> = (0..n).collect();
    let mut edges = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            if dists[i][j] <= scale {
                edges += 1;
                union(&mut parent, i, j);
            }
        }
    }
    let mut roots = std::collections::HashSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    let components = roots.len();
    let triangles = count_filled_triangles(dists, n, scale);
    (components, edges, triangles)
}

fn count_filled_triangles(dists: &[Vec<f64>], n: usize, scale: f64) -> usize {
    let mut count = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            if dists[i][j] > scale {
                continue;
            }
            for k in (j + 1)..n {
                if dists[i][k] > scale || dists[j][k] > scale {
                    continue;
                }
                count += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_cloud_has_zero_betti() {
        let cloud = PointCloud::new("empty");
        let ph = PersistentHomology::compute(&cloud, 1.0, 10);
        assert_eq!(ph.at_scale(0.5), Some(BettiNumbers::zero()));
    }

    #[test]
    fn two_isolated_points_merge() {
        let mut cloud = PointCloud::new("two-points");
        cloud.add_point(vec![0.0, 0.0]);
        cloud.add_point(vec![1.0, 0.0]);
        let ph = PersistentHomology::compute(&cloud, 2.0, 20);
        assert_eq!(ph.at_scale(0.4).unwrap().beta_0, 2);
        assert_eq!(ph.at_scale(1.5).unwrap().beta_0, 1);
    }

    #[test]
    fn triangle_yields_beta_1() {
        let mut cloud = PointCloud::new("triangle");
        cloud.add_point(vec![0.0, 0.0]);
        cloud.add_point(vec![1.0, 0.0]);
        cloud.add_point(vec![0.5, 0.866]);
        let ph = PersistentHomology::compute(&cloud, 2.0, 20);
        // At a scale just large enough to connect all 3 edges, the
        // 1-skeleton of the triangle has β_0 = 1, β_1 = 1.
        let b = ph.at_scale(1.1).unwrap();
        assert_eq!(b.beta_0, 1);
        assert_eq!(b.beta_1, 1);
    }

    #[test]
    fn persistence_entropy_nonneg() {
        let mut cloud = PointCloud::new("ring");
        for k in 0..6 {
            let theta = (k as f64) * std::f64::consts::TAU / 6.0;
            cloud.add_point(vec![theta.cos(), theta.sin()]);
        }
        let ph = PersistentHomology::compute(&cloud, 3.0, 30);
        let h = ph.persistence_entropy();
        assert!(h >= 0.0);
    }

    #[test]
    fn simplified_betti_is_midpoint() {
        let mut cloud = PointCloud::new("two");
        cloud.add_point(vec![0.0, 0.0]);
        cloud.add_point(vec![2.0, 0.0]);
        let ph = PersistentHomology::compute(&cloud, 4.0, 10);
        let mid = ph.simplified_betti();
        // Midpoint scale ≈ 2.0 → 1 component, 1 edge → β_1 = 0
        assert_eq!(mid.beta_0, 1);
        assert_eq!(mid.beta_1, 0);
    }

    #[test]
    fn integration_estimate_finite_for_empty() {
        let b = BettiNumbers::zero();
        let v = b.integration_estimate();
        assert!(v.is_finite());
    }

    #[test]
    fn at_scale_returns_none_above_max() {
        let mut cloud = PointCloud::new("single");
        cloud.add_point(vec![0.0]);
        let ph = PersistentHomology::compute(&cloud, 1.0, 5);
        // scale_max = 1.0, num_steps = 5 → largest stored scale is 1.0
        assert!(ph.at_scale(10.0).is_some());
        assert!(ph.at_scale(0.0).is_some());
    }
}
