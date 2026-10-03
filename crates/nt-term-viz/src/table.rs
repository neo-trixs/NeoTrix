//! 表格对齐原语 —— **按终端列宽**而非字符数。
//!
//! 本仓 `chars().count()` 出现 145 次，但中文占 2 列 ⇒ 用字符数算 padding
//! 会让所有中文表格错位。本模块把「按列宽对齐」收进原语，
//! 避免每个站点各写一遍（且各写错一遍）。
use crate::char_width;

/// 对齐方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// 左对齐（尾部补空格）
    Left,
    /// 右对齐（头部补空格）
    Right,
}

/// 把 `s` 按**列宽**补齐到 `cols` 列。
///
/// # 与「按字符数补齐」的区别（本模块存在的理由）
///
/// ```
/// use nt_term_viz::{pad_to, Align};
/// // 「中」1 个字符但占 2 列
/// assert_eq!(pad_to("中", 4, Align::Left), "中  ");
/// assert_eq!("中".chars().count(), 1);          // 字符数只有 1
/// ```
///
/// # 边界（刻意固定，避免各调用方自己判断）
/// * `s` 列宽已 ≥ `cols` ⇒ **原样返回，不截断**（截断请用 [`pad_trunc`]）。
///   理由：对齐与截断是两种意图，混在一起会出现「静默丢数据」。
/// * `cols == 0` ⇒ 原样返回。
pub fn pad_to(s: &str, cols: usize, align: Align) -> String {
    let w = crate::display_width(s);
    if cols == 0 || w >= cols {
        return s.to_string();
    }
    let fill = " ".repeat(cols - w);
    match align {
        Align::Left => format!("{s}{fill}"),
        Align::Right => format!("{fill}{s}"),
    }
}

/// 按**列宽**截断到 `cols`（超长时以 `…` 收尾），不补齐。
///
/// 与 [`pad_to`] 的区别：此函数**只截断不补齐**，
/// 适合单元格；`pad_to` 只补齐不截断。
pub fn pad_trunc(s: &str, cols: usize) -> String {
    crate::truncate_to(s, cols)
}

/// 渲染一张最简单的两列表（`key: value`），自动按最长键右对齐。
///
/// # 例子
///
/// ⚠️ 注意输出里 `层` 后面是**两个空格** —— 因为 `层` 占 **2 列**，
/// 键列宽取 `max(display_width) = 4`（`层`=2 列，故补 2 空格；
/// 而 `类型` 本身就是 4 列）。
///
/// ⛔ 本函数的 doctest 首版写成 `"层:   L0"`（只 1 个空格）——
/// 那是**按字符数**算的期望，在中文下正好错 1 列。
/// 这也是本 crate 存在的理由：**手写列宽几乎必然算错**。
/// ```
/// use nt_term_viz::table::render_pairs;
/// let out = render_pairs(&[("层", "L0"), ("类型", "crate")]);
/// assert_eq!(out, vec!["层  : L0", "类型: crate"]);   // ← "层" 后是 2 个空格
/// ```
pub fn render_pairs(pairs: &[(&str, &str)]) -> Vec<String> {
    let key_w = pairs
        .iter()
        .map(|(k, _)| crate::display_width(k))
        .max()
        .unwrap_or(0);
    pairs
        .iter()
        .map(|(k, v)| {
            let padded = pad_to(k, key_w, Align::Left);
            format!("{padded}: {v}")
        })
        .collect()
}

/// 计算一段文本在终端占用的总列宽（多行取**最大值**）。
///
/// 用于「表格该留多宽」这类需求：取最大值而非求和，
/// 否则多行会被误算成行宽之和。
pub fn max_line_width(text: &str) -> usize {
    text.lines()
        .map(crate::display_width)
        .max()
        .unwrap_or(0)
}

/// 确认一个字符串里没有会让宽度计算失真的控制字符。
///
/// ⚠️ **宽度不可算**：ANSI 转义序列（如 `\x1b[31m`）占 0 列但有实际效果，
/// 含它的字符串算出的 `display_width` 会与终端实际不符。
/// ⇒ 本 crate 不解析 ANSI（那是另一个问题），
/// 但提供此函数让调用方**能自检**，避免静默错位。
pub fn has_ansi_escape(s: &str) -> bool {
    s.contains('\u{1b}')
}

/// 剔除 ANSI 转义序列（仅支持 CSI `ESC[...m` 这类 SGR）。
///
/// ⛔ 这是**尽力而为**的最小实现：只处理 `ESC [` … 终���符。
/// 遇到未识别的转义构造时**保持原样**（不猜测），宁可保守。
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // 期待 '['，则跳到终止字节
            if chars.peek() == Some(&'[') {
                chars.next();
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() || c2 == '~' {
                        break;
                    }
                }
                continue;
            }
            // 未识别的两字节转义：原样保留 ESC
            out.push(c);
            continue;
        }
        if c.is_control() && c != '\n' && c != '\t' {
            continue; // 丢弃裸控制字符（宽度不可算）
        }
        out.push(c);
    }
    out
}

/// 若字符串含 ANSI 转义，先剥离再按列宽补齐 —— 否则补齐量会算错。
///
/// 这是「宽度感知对齐」在真实 CLI 里最常见的一环：
/// 带颜色的表格若不先剥离，`display_width` 会把转义序列算成 0 列。
pub fn pad_to_ansi_safe(s: &str, cols: usize, align: Align) -> String {
    pad_to(&strip_ansi(s), cols, align)
}

/// 确保某个字符是单列宽（用于绘制垂直线、方块等）。
///
/// ⛔ 若传入全角字符会破坏对齐 ⇒ 直接判否，让调用方改用已知单宽字符。
pub fn is_single_width(c: char) -> bool {
    char_width(c) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_to_uses_columns_not_chars() {
        // 「中」1 字符 / 2 列 ⇒ 补 2 个空格才是 4 列
        assert_eq!(pad_to("中", 4, Align::Left), "中  ");
        assert_eq!(pad_to("ab", 4, Align::Left), "ab  ");
        assert_eq!(pad_to("中", 4, Align::Right), "  中");
    }

    #[test]
    fn pad_to_does_not_truncate() {
        // 刻意不截断：超长原样返回（截断是另一种意图）
        assert_eq!(pad_to("abcdef", 3, Align::Left), "abcdef");
        assert_eq!(pad_to("abc", 0, Align::Left), "abc");
    }

    #[test]
    fn pad_trunc_truncates_by_columns() {
        assert_eq!(pad_trunc("abcdef", 5), "abcd…");
        // 需要截断时必须为省略号预留 1 列 ⇒ "中"（2 列）+ "…"（1 列）= 3 列
        assert_eq!(pad_trunc("中文中文", 4), "中…");
        assert_eq!(crate::display_width(&pad_trunc("中文中文", 4)), 3);
    }

    #[test]
    fn render_pairs_aligns_cjk_keys() {
        let out = render_pairs(&[("层", "L0"), ("类型", "crate")]);
        // ⚠️ 不断言整串内容：手写期望串极易算错列宽（本轮就错过一次：
        //   "层" 是 2 列不是 1 列 ⇒ 应为 "层  : L0" 而非 "层:   L0"）。
        // ⇒ 断言**真正要保证的性质**：两个冒号落在同一终端列。
        // ⛔⛔ **必须按列宽比，不能按字节偏移比** ——
        // `find(':')` 返回的是**字节**下标：`层`(3B)+2空格=5，
        // 而 `类型`(6B)=6 ⇒ 字节不同但**列数都是 4、视觉是对齐的**。
        // 首版正是踩了这个坑（left: 5, right: 6）—— 恰是本 crate 要防的那类错。
        let colon_col = |line: &str| {
            let byte = line.find(':').expect("应含冒号");
            crate::display_width(&line[..byte])
        };
        assert_eq!(
            colon_col(&out[0]),
            colon_col(&out[1]),
            "冒号应对齐（按**列宽**比，不是字节偏移）"
        );
        // 且键列确实被补齐到 4 列（"层" 2 列 + 2 空格）
        assert_eq!(colon_col(&out[0]), 4);
        assert_eq!(colon_col(&out[1]), 4);
        // 值未被破坏
        assert!(out[0].ends_with("L0"));
        assert!(out[1].ends_with("crate"));
    }

    #[test]
    fn max_line_width_takes_max_not_sum() {
        assert_eq!(max_line_width("ab\n中文"), 4);
        assert_eq!(max_line_width(""), 0);
    }

    #[test]
    fn ansi_is_detected_and_stripped() {
        assert!(has_ansi_escape("\u{1b}[31mred\u{1b}[0m"));
        assert!(!has_ansi_escape("plain"));
        assert_eq!(strip_ansi("\u{1b}[31mred\u{1b}[0m"), "red");
        assert_eq!(strip_ansi("plain"), "plain");
        // 带颜色补齐：先剥离再算宽度
        let colored = "\u{1b}[31m中\u{1b}[0m";
        // 实测 11（`\e[31m` 5 + `中` 2 + `\e[0m` 4）
        // ⇒ 正是「转义序列让宽度算错」的实证
        assert_eq!(crate::display_width(colored), 11, "转义序列被计入宽度");
        assert_eq!(pad_to_ansi_safe(colored, 4, Align::Left), "中  ");
    }

    #[test]
    fn single_width_check() {
        assert!(is_single_width('|'));
        assert!(is_single_width('├'));
        assert!(!is_single_width('中'));
    }
}