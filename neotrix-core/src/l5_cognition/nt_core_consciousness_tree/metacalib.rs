//! Phase 5 元认知校准: 期望校准误差 (ECE) 与 Brier 分数, 防 D15 健康虚高。
//! 纯函数, 由 ConsciousnessTreeImpl::calibrate_branch_health 调用 (T3 生产接线)。

/// 期望校准误差 (Expected Calibration Error): 按置信度分箱, 各箱 |精度-置信| 加权平均。
/// `samples`: (模型置信度 0..1, 是否正确)。空样本返回 0.0。
pub fn expected_calibration_error(samples: &[(f32, bool)], bins: usize) -> f32 {
    if samples.is_empty() || bins == 0 {
        return 0.0;
    }
    let bins = bins.max(1);
    let mut bin_conf_sum = vec![0.0f32; bins];
    let mut bin_acc_sum = vec![0.0f32; bins];
    let mut bin_count = vec![0u32; bins];
    for &(conf, correct) in samples {
        let c = conf.clamp(0.0, 0.999999);
        let b = ((c * bins as f32) as usize).min(bins - 1);
        bin_conf_sum[b] += conf;
        bin_acc_sum[b] += if correct { 1.0 } else { 0.0 };
        bin_count[b] += 1;
    }
    let mut ece = 0.0f32;
    let mut total = 0u32;
    for i in 0..bins {
        let n = bin_count[i];
        if n == 0 {
            continue;
        }
        let avg_conf = bin_conf_sum[i] / n as f32;
        let avg_acc = bin_acc_sum[i] / n as f32;
        ece += n as f32 * (avg_conf - avg_acc).abs();
        total += n;
    }
    if total == 0 {
        0.0
    } else {
        ece / total as f32
    }
}

/// Brier 分数: 二分类预测均方误差 (越低越好)。空样本返回 0.0。
pub fn brier_score(samples: &[(f32, bool)]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut s = 0.0f32;
    for &(conf, correct) in samples {
        let t = if correct { 1.0f32 } else { 0.0f32 };
        let d = conf.clamp(0.0, 1.0) - t;
        s += d * d;
    }
    s / samples.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ece_perfectly_calibrated_is_near_zero() {
        let s: Vec<(f32, bool)> =
            vec![(0.9, true), (0.9, true), (0.1, false), (0.5, true), (0.5, false)];
        let ece = expected_calibration_error(&s, 5);
        assert!(ece < 0.1, "近似校准 ece 应接近 0, got {ece}");
    }

    #[test]
    fn ece_miscalibrated_is_large() {
        let s: Vec<(f32, bool)> = vec![(0.95, false), (0.95, false), (0.95, false)];
        let ece = expected_calibration_error(&s, 5);
        assert!(ece > 0.8, "高置信全错 ece 应很大, got {ece}");
    }

    #[test]
    fn brier_matches_formula() {
        let s = vec![(1.0, true), (0.0, false), (0.5, true)];
        let b = brier_score(&s);
        assert!((b - 0.25 / 3.0).abs() < 1e-5, "got {b}");
    }

    #[test]
    fn empty_inputs_safe() {
        assert_eq!(expected_calibration_error(&[], 5), 0.0);
        assert_eq!(brier_score(&[]), 0.0);
    }
}
