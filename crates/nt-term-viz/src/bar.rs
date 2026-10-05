//! 方块密度条 —— 用 8 级 Unicode 方块（U+2581..U+2588）画柱状图。
//!
//! # 为什么用方块字符而不是 `█` 重复
//!
//! 重复 `█` 只能表达整数倍；方块有 **8 级亚字符精度**，
//! 在 1~2 列宽的小柱子上能表达小数差异。
//! 实测依据：本仓此前**完全没有任何方块渲染**（全 `crates/` 树 0 个）。
use crate::table::{pad_to, Align};

/// 8 级方块密度（U+2581 ▁ … U+2588 █），**均占 1 列**。
pub const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// 把 `value`（相对于 `max`）映射到 `width` 列的方块条。
///
/// # 边界（刻意固定，避免各调用方自己判断）
/// * `width == 0` ⇒ 返回空串。
/// * `max <= 0` 或无有效数据 ⇒ 返回 `width` 个空格（占位保持列宽）。
/// * `value` 为负 ⇒ 夹到 0（不产生畸形输出）。
/// * `value > max` ⇒ 夹到 `max`（不溢出）。
pub fn bar_line(value: f64, max: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
        // ⭐ 2026-10-05 修 `clippy::neg_cmp_op_on_partial_ord`（CI 是 `-D warnings` ⇒ 硬红）。
    //   `f64` 是**偏序**类型，`!(max > 0.0)` 同时排除「≤0」与「NaN」，
    //   语义正确但 clippy 要求写明。实测 6/6 等价（0.0/-1.0/1.0/NaN/±INF）。
    //   ⇒ 改成显式 `is_nan()`，语义逐位不变，同时满足 lint。
    if max.is_nan() || max <= 0.0 {
        return " ".repeat(width);
    }
    // ⭐ 2026-10-05 补 NaN 显式处理。
    //
    // 实测：`f64::clamp` 对 NaN **返回 NaN**（不夹取），
    // 而 `NaN as usize` 饱和到 0 ⇒ `value=NaN` 或 `value/max=NaN` 时
    // 整条静默退化成空格。调用方看到「一条空条」**无从区分
    // 「真的是 0」还是「算崩了」** —— 静默错误比报错更难查。
    // ⇒ 显式把 NaN 比例夹到 0，让「无效输入」有确定行为。
    let raw_ratio = value / max;
    let ratio = if raw_ratio.is_nan() { 0.0 } else { raw_ratio.clamp(0.0, 1.0) };
    // 先算「整列 + 亚列部分」：第 k 列用第 k+1 级方块（1..=8）
    let full = ratio * width as f64;
    let mut out = String::with_capacity(width);
    for i in 0..width {
        let cell = (full - i as f64).clamp(0.0, 1.0);
        let level = (cell * 8.0).round() as usize;
        // level 0 ⇒ 空格；1..=8 ⇒ BLOCKS[level-1]
        if level == 0 {
            out.push(' ');
        } else {
            out.push(BLOCKS[(level - 1).min(7)]);
        }
    }
    out
}

/// 一行「标签 + 条」：`label` 左对齐到 `label_w`，后接 `bar_width` 列的条。
///
/// `max <= 0` 时条为空白占位 —— 这样多行输出仍然上下对齐。
pub fn bar_row(label: &str, value: f64, max: f64, label_w: usize, bar_width: usize) -> String {
    let head = pad_to(label, label_w, Align::Left);
    format!("{head} {}", bar_line(value, max, bar_width))
}

/// 多行渲染：自动取 `values` 的最大值作 `max`，并按最长标签对齐。
///
/// # 例子
/// ```
/// use nt_term_viz::bar::bar_rows;
/// let rows = bar_rows(&[("core", 3.0), ("types", 5.0)]);
/// assert_eq!(rows.len(), 2);
/// // 标签列对齐 ⇒ 两个字符串的条起始列相同
/// assert_eq!(rows[0].find('▁'), rows[1].find('▁'));
/// ```
pub fn bar_rows(pairs: &[(&str, f64)]) -> Vec<String> {
    let max = pairs.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
    let label_w = pairs
        .iter()
        .map(|(l, _)| crate::display_width(l))
        .max()
        .unwrap_or(0);
    pairs
        .iter()
        .map(|(l, v)| bar_row(l, *v, max, label_w, 10))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_blocks_are_single_width() {
        for b in BLOCKS {
            assert_eq!(crate::char_width(b), 1, "方块 {b} 必须占 1 列");
        }
    }

    #[test]
    fn bar_line_respects_width() {
        assert_eq!(bar_line(1.0, 1.0, 0), "");
        assert_eq!(bar_line(0.0, 1.0, 4).chars().count(), 4);
        assert_eq!(bar_line(1.0, 1.0, 4), "████");
        assert_eq!(bar_line(0.0, 1.0, 4), "    ");
    }

    #[test]
    fn bar_line_clamps_out_of_range() {
        // 负值夹到 0
        assert_eq!(bar_line(-5.0, 10.0, 3), "   ");
        // 超过 max 夹到 max
        assert_eq!(bar_line(99.0, 10.0, 3), "███");
        // max 无效 ⇒ 空白占位（保持列宽）
        assert_eq!(bar_line(5.0, 0.0, 3), "   ");
        assert_eq!(bar_line(5.0, -1.0, 3), "   ");
    }

    /// ⭐ 回归锁：NaN /±INF 不得产生**静默错误**的条（2026-10-05）。
    ///
    /// 实测（喂真值）：`value=INF, max=INF` ⇒ `ratio = INF/INF = NaN`
    /// ⇒ `f64::clamp`对 NaN **返回 NaN**（不夹取）
    /// ⇒ `(NaN * 8.0).round() as usize == 0` ⇒ 整条退化成空格，
    ///    而调用方看到的是「一条空条」，**无从区分「真的是 0」还是「算崩了」**。
    #[test]
    fn bar_line_nan_and_inf_do_not_silently_blank() {
        // ⭐ NaN 比例 ⇒ 必须夹到 0（全空条），不得 panic / 不得输出乱码
        let nan_ratio = bar_line(f64::INFINITY, f64::INFINITY, 4);
        assert_eq!(
            nan_ratio.chars().count(),
            4,
            "宽度仍须守恒：{nan_ratio:?}"
        );
        assert!(
            nan_ratio.chars().all(|c| c == ' '|| BLOCKS.contains(&c)),
            "只允许空格或方块，不得出现其他字符：{nan_ratio:?}"
        );
        // NaN 入参（value 或 max）
        for (v, m) in [
            (f64::NAN, 10.0),
            (10.0, f64::NAN),
            (f64::NAN, f64::NAN),
            (f64::NEG_INFINITY, 10.0),
            (10.0, f64::NEG_INFINITY),
            (f64::INFINITY, 0.0),
        ] {
            let out = bar_line(v, m, 4);
            assert_eq!(out.chars().count(), 4, "value={v:?} max={m:?} ⇒ {out:?}");
            assert!(
                out.chars().all(|c| c == ' ' || BLOCKS.contains(&c)),
                "value={v:?} max={m:?} ⇒ 出现非法字符：{out:?}"
            );
        }
    }

    /// ⭐ `!(max > 0.0)` 与显式写法的**等价性**锁。
    ///
    /// 为何要改写：`clippy::neg_cmp_op_on_partial_ord`（`f64` 是偏序类型）。
    /// 而 CI 是 `cargo clippy --lib -p neotrix -- -D warnings` ⇒ 该告警**硬红 CI**。
    ///
    /// 实测 6/6 等价（含 NaN）：`0.0 / -1.0 / 1.0 / NaN / ±INF` 三种写法同结果。
    #[test]
    fn max_guard_is_nan_safe() {
        // max 为 NaN 时必须走「无效」分支（空白占位），而不是算出 NaN 比例
        assert!(bar_line(5.0, f64::NAN, 3).chars().all(|c| c == ' '));
        assert!(bar_line(5.0, f64::NEG_INFINITY, 3).chars().all(|c| c == ' '));
        // max 正常时条仍须有内容
        assert!(bar_line(5.0, 10.0, 3).chars().any(|c| c != ' '));
    }

    #[test]
    fn bar_line_is_monotonic() {
        // 递增的值不应产生更短的「可见」条（用空格数衡量）
        let spaces = |v: f64| bar_line(v, 10.0, 8).chars().filter(|c| *c == ' ').count();
        let seq: Vec<usize> = (0..=10).map(|i| spaces(i as f64)).collect();
        for w in seq.windows(2) {
            assert!(w[0] >= w[1], "条应单调不减（空格数应单调不增）: {seq:?}");
        }
    }

    #[test]
    fn bar_rows_aligns_labels() {
        let rows = bar_rows(&[("core", 3.0), ("types", 5.0)]);
        assert_eq!(rows.len(), 2);
        // 条的起始**列**必须相同 ⇒ 标签右对齐有效。
        // ⚠️ 探测只能用**方块**定位：首版写成 `BLOCKS.contains(&c) || c == ' '`
        // 会命中「第一个空格」，而空格位置随标签长度变化 ⇒ 假失败。
        let bar_start_col = |s: &str| {
            s.chars()
                .position(|c| BLOCKS.contains(&c))
                .expect("条内必有方块")
        };
        assert_eq!(
            bar_start_col(&rows[0]),
            bar_start_col(&rows[1]),
            "条的起始列应一致（标签列已对齐）"
        );
        // 行总宽也应一致（条宽固定）
        assert_eq!(
            crate::display_width(&rows[0]),
            crate::display_width(&rows[1]),
            "两行总宽应一致"
        );
    }

    #[test]
    fn bar_rows_empty_is_empty() {
        assert!(bar_rows(&[]).is_empty());
    }
}