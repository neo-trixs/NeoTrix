use super::{NUM_SUBSYSTEMS, PIDScores};

pub(crate) fn compute_pairwise_pid(
    series: &[Vec<f64>; NUM_SUBSYSTEMS],
    n: usize,
    i: usize,
    j: usize,
) -> PIDScores {
    let xi = &series[i];
    let yj = &series[j];
    let mut target = Vec::with_capacity(n);
    for t in 0..n {
        let mut sum = 0.0;
        for k in 0..NUM_SUBSYSTEMS {
            sum += series[k][t];
        }
        target.push(sum / NUM_SUBSYSTEMS as f64);
    }
    let r_xt = pearson(xi, &target);
    let r_yt = pearson(yj, &target);
    let i_xt = if r_xt.abs() < 1.0 { -0.5 * (1.0 - r_xt * r_xt).ln() } else { 10.0 };
    let i_yt = if r_yt.abs() < 1.0 { -0.5 * (1.0 - r_yt * r_yt).ln() } else { 10.0 };
    let mx = xi.iter().sum::<f64>() / n as f64;
    let my = yj.iter().sum::<f64>() / n as f64;
    let mt = target.iter().sum::<f64>() / n as f64;
    let mut c_xx = 0.0; let mut c_xy = 0.0; let mut c_xt = 0.0;
    let mut c_yy = 0.0; let mut c_yt = 0.0; let mut c_tt = 0.0;
    for k in 0..n {
        let dx = xi[k] - mx;
        let dy = yj[k] - my;
        let dt = target[k] - mt;
        c_xx += dx * dx; c_xy += dx * dy; c_xt += dx * dt;
        c_yy += dy * dy; c_yt += dy * dt; c_tt += dt * dt;
    }
    let nf = n as f64;
    c_xx /= nf - 1.0; c_xy /= nf - 1.0; c_xt /= nf - 1.0;
    c_yy /= nf - 1.0; c_yt /= nf - 1.0; c_tt /= nf - 1.0;
    let det_xy = c_xx * c_yy - c_xy * c_xy;
    let det_xyt = c_xx * (c_yy * c_tt - c_yt * c_yt)
                - c_xy * (c_xy * c_tt - c_yt * c_xt)
                + c_xt * (c_xy * c_yt - c_yy * c_xt);
    let i_xyt = if det_xy > 1e-14 && det_xyt > 1e-14 && c_tt > 1e-14 {
        0.5 * (det_xy * c_tt / det_xyt).ln().max(0.0)
    } else {
        0.0
    };
    let redundant = i_xt.min(i_yt);
    let unique_x = i_xt - redundant;
    let unique_y = i_yt - redundant;
    let synergy = (i_xyt - i_xt.max(i_yt)).max(0.0);
    let total = unique_x + unique_y + redundant + synergy;
    let synergy_fraction = if total > 1e-14 { synergy / total } else { 0.0 };
    PIDScores { unique_x, unique_y, redundant, synergy, total, synergy_fraction }
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n < 3 { return 0.0; }
    let ma = a.iter().sum::<f64>() / n as f64;
    let mb = b.iter().sum::<f64>() / n as f64;
    let mut num = 0.0; let mut sa = 0.0; let mut sb = 0.0;
    for k in 0..n {
        let da = a[k] - ma;
        let db = b[k] - mb;
        num += da * db; sa += da * da; sb += db * db;
    }
    let denom = (sa * sb).sqrt();
    if denom < 1e-14 { 0.0 } else { (num / denom).clamp(-1.0, 1.0) }
}
