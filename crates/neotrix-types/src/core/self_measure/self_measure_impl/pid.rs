use super::NUM_SUBSYSTEMS;

#[derive(Debug, Clone, Copy)]
pub struct PIDScores {
    pub unique_x: f64,
    pub unique_y: f64,
    pub redundant: f64,
    pub synergy: f64,
    pub total: f64,
    pub synergy_fraction: f64,
}

pub fn spectral_phi(corr: &[[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS], n: usize) -> f64 {
    if n < 2 { return 0.0; }
    let mut a = [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
    let mut deg = [0.0; NUM_SUBSYSTEMS];
    for i in 0..n {
        for j in 0..n {
            if i != j {
                let w = corr[i][j].abs();
                a[i][j] = w;
                deg[i] += w;
            }
        }
    }
    let mut lap = [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
    for i in 0..n {
        for j in 0..n {
            lap[i][j] = if i == j { deg[i] - a[i][j] } else { -a[i][j] };
        }
    }
    let evals = symmetric_qr(&lap, n, 100);
    if evals.len() < 2 {
        return 0.0;
    }
    let mut sorted = evals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted[1].clamp(0.0, 2.0)
}

fn symmetric_qr(matrix: &[[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS], n: usize, max_iter: usize) -> Vec<f64> {
    if n == 0 { return vec![]; }
    let mut a = [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
    for i in 0..n {
        for j in 0..n {
            a[i][j] = matrix[i][j];
        }
    }
    tridiagonalize(&mut a, n);
    qr_tridiagonal(&mut a, n, max_iter)
}

#[allow(clippy::needless_range_loop)]
fn tridiagonalize(a: &mut [[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS], n: usize) {
    for k in 0..(n.saturating_sub(2)) {
        let mut norm = 0.0;
        for i in (k + 1)..n {
            norm += a[i][k] * a[i][k];
        }
        norm = norm.sqrt();
        if norm < 1e-14 { continue; }
        let scale = if a[k + 1][k] >= 0.0 { -norm } else { norm };
        let tau = 2.0 * a[k + 1][k] * scale + scale * scale;
        if tau.abs() < 1e-14 { continue; }
        let u0 = a[k + 1][k] - scale;
        for j in (k + 1)..n {
            let mut s = 0.0;
            for i in (k + 1)..n {
                let ui = if i == k + 1 { u0 } else { a[i][k] };
                s += a[i][j] * ui;
            }
            s *= 2.0 / tau;
            for i in (k + 1)..n {
                let ui = if i == k + 1 { u0 } else { a[i][k] };
                a[i][j] -= s * ui;
                a[j][i] = a[i][j];
            }
        }
        a[k + 1][k] = scale;
        for a_ik in a.iter_mut().take(n).skip(k + 2) {
            a_ik[k] = 0.0;
        }
    }
}

fn qr_tridiagonal(a: &mut [[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS], n: usize, max_iter: usize) -> Vec<f64> {
    let mut d = [0.0; NUM_SUBSYSTEMS];
    let mut e = [0.0; NUM_SUBSYSTEMS];
    for i in 0..n {
        d[i] = a[i][i];
    }
    for i in 0..(n.saturating_sub(1)) {
        e[i] = a[i + 1][i];
    }
    if n > 0 { e[n - 1] = 0.0; }
    for _iter in 0..max_iter {
        let mut m = n.saturating_sub(1);
        while m > 0 && e[m - 1].abs() >= 1e-14 {
            m -= 1;
        }
        if m == 0 { break; }
        let i = n - 2;
        let bb = (d[i + 1] - d[i]) / 2.0;
        let shift = if bb.abs() < 1e-14 {
            d[i + 1] - e[i].abs()
        } else {
            let sign = if bb >= 0.0 { 1.0 } else { -1.0 };
            d[i + 1] - sign * e[i] * e[i] / (bb.abs() + (bb * bb + e[i] * e[i]).sqrt())
        };
        let mut x = d[m] - shift;
        let mut z = e[m];
        for k in m..(n - 1) {
            let denom = (x * x + z * z).sqrt();
            if denom < 1e-14 { continue; }
            let c = x / denom;
            let s = z / denom;
            let dk = d[k];
            let ek = e[k];
            let dk1 = d[k + 1];
            let ek1 = if k < n - 2 { e[k + 1] } else { 0.0 };
            d[k] = c * dk + s * ek;
            e[k] = c * ek - s * dk;
            d[k + 1] = c * dk1 - s * ek1;
            if k < n - 2 {
                e[k + 1] = c * ek1 + s * dk1;
            }
            x = e[k];
            if k < n - 2 {
                z = s * d[k + 1] + c * e[k + 1];
            }
        }
    }
    d[..n].to_vec()
}

pub fn eigenvector_centrality(matrix: &[[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS], n: usize, max_iter: usize) -> Vec<f64> {
    let mut v = vec![1.0 / (n as f64).sqrt(); n];
    for _ in 0..max_iter {
        let mut next = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                next[i] += matrix[i][j] * v[j];
            }
        }
        let norm = next.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < 1e-14 { break; }
        for val in next.iter_mut() {
            *val /= norm;
        }
        v = next;
    }
    let max_val = v.iter().cloned().fold(0.0_f64, f64::max);
    if max_val > 1e-14 {
        for val in v.iter_mut() {
            *val /= max_val;
        }
    }
    v
}
