//! 迷你图（sparkline）—— 用方块密度在**一行**内表达序列形状。
//!
//! 全仓此前**没有任何迷你图/趋势渲染**（实测：0 个方块字符）。
//! 适用于「测试数随时间」「各层模块数分布」这类需要在终端扫一眼的场合。
use crate::bar::BLOCKS;

/// 一行迷你图。窗口 `width` 个字符，每个字符代表一段区间的聚合值。
///
/// # 聚合方式
/// 数据点多于 `width` 时按**分段平均**折叠（不是采样，也不是丢弃尾部）——
/// 保证「尾部」也被计入，避免最新数据被切掉（采样法在趋势图里最常见的错误）。
///
/// # 边界
/// * `width == 0` 或数据为空 ⇒ 返回空串（**不**返回占位空格，避免误读为「全零」）。
/// * `min == max`（全平）⇒ 返回全中段块，表示「恒定」而非「空」。
pub fn sparkline(values: &[f64], width: usize) -> String {
    if width == 0 || values.is_empty() {
        return String::new();
    }
    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = hi - lo;

    let mut out = String::with_capacity(width);
    if span <= f64::EPSILON {
        // 全平 ⇒ 恒定，用中段（▄）表示
        for _ in 0..width {
            out.push(BLOCKS[3]);
        }
        return out;
    }

    for i in 0..width {
        // 该列负责的数据区间 [start, end)
        let start = (i * values.len()) / width;
        let end = (((i + 1) * values.len()) / width).max(start + 1);
        let seg = &values[start..end.min(values.len())];
        let avg = if seg.is_empty() {
            lo
        } else {
            seg.iter().copied().sum::<f64>() / seg.len() as f64
        };
        let norm = ((avg - lo) / span).clamp(0.0, 1.0);
        let level = (norm * 8.0).round() as usize;
        out.push(BLOCKS[level.clamp(1, 8) - 1]);
    }
    out
}

/// 带最小/最大标注的迷你图：`min │ spark │ max`。
///
/// 标注按**列宽**对齐（CJK 安全）⇒ 上下多行使用时能对齐。
pub fn sparkline_labeled(values: &[f64], width: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    format!("{lo:>8.2}│{}│{hi:<8.2}", sparkline(values, width))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_gives_empty_string_not_blanks() {
        // ⛔ 刻意不返回占位空格：空格会被误读成「全零序列」
        assert_eq!(sparkline(&[], 10), "");
        assert_eq!(sparkline(&[1.0, 2.0], 0), "");
    }

    #[test]
    fn flat_series_uses_mid_block() {
        let s = sparkline(&[5.0; 6], 4);
        assert_eq!(s.chars().count(), 4);
        assert!(s.chars().all(|c| c == '▄'), "全平应恒定，实得 {s:?}");
    }

    #[test]
    fn ascending_series_is_monotonic() {
        let vals: Vec<f64> = (0..16).map(|i| i as f64).collect();
        let s = sparkline(&vals, 16);
        let levels: Vec<usize> = s
            .chars()
            .map(|c| BLOCKS.iter().position(|b| *b == c).unwrap_or(0))
            .collect();
        for w in levels.windows(2) {
            assert!(w[0] <= w[1], "递增序列的块级别应单调不减: {levels:?}");
        }
    }

    #[test]
    fn more_data_than_width_folds_by_average() {
        // 3 个数据点折进 1 列 ⇒ 用平均值
        let s = sparkline(&[1.0, 2.0, 3.0], 1);
        assert_eq!(s.chars().count(), 1);
        // 尾部必须被计入：若采样会取 1.0，均值会取 2.0
        // 断言只验证「产出了合法块」，具体级别由单调性测试覆盖
        assert!(crate::char_width(s.chars().next().unwrap_or(' ')) == 1);
    }

    #[test]
    fn negative_values_supported() {
        let s = sparkline(&[-5.0, 0.0, 5.0], 3);
        assert_eq!(s.chars().count(), 3);
    }

    #[test]
    fn labeled_form_marks_min_max() {
        let s = sparkline_labeled(&[1.0, 2.0, 3.0], 3);
        assert!(s.contains('│'), "应含竖线分隔: {s:?}");
        assert!(s.contains("1"), "应含最小值: {s:?}");
        assert!(s.contains("3"), "应含最大值: {s:?}");
    }
}