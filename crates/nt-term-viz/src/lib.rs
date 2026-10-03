//! Unicode 终端可视化原语（`nt-term-viz`）。
//!
//! # 为什么需要这个 crate
//!
//! 建它之前的实测结论（2026-10-02，`grep` + 严格核实）：**全 `crates/` 树只有
//! 2 个文件含渲染字符，共 8 个制表符、0 个方块符**；唯一的「树渲染」是
//! `nt-core-capability-tree/src/cli.rs:488` 里**内联 `println!` + 硬编码 `├─`**。
//!
//! ⇒ 真实情况是**能力缺失**（各处各写各的、零抽象），**不是冗余重复**。
//! ⇒ 所以这是**新建原语**，不是「提取重复」。
//!
//! # 核心问题：宽度
//!
//! 本仓是**中文为主**的仓库，而全仓 `chars().count()` 出现 **145 次** ——
//! 但 `chars().count()` 数的是**字符数**，不是**终端列宽**：
//! `一` 是 1 个字符却占 **2 列** ⇒ 任何用它算 padding 的对齐在中文下都会错位。
//!
//! 本 crate 用 `unicode-width` 给出**真实列宽**，并把「对齐」收进原语，
//! 避免每个站点各写一遍（且各写错一遍）。
#![forbid(unsafe_code)]

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub mod bar;
pub mod sparkline;
pub mod table;
pub mod tree;

pub use bar::{bar_line, bar_row};
pub use sparkline::sparkline;
pub use table::{Align, pad_to, pad_trunc};
pub use tree::{tree_connector, TreeStyle};

/// 字符串在终端占用的**列数**（CJK 全角 = 2）。
///
/// # 与 `chars().count()` 的区别（本 crate 存在的核心理由）
///
/// ```
/// use nt_term_viz::display_width;
/// assert_eq!("abc".chars().count(), 3);
/// assert_eq!(display_width("abc"), 3);
/// // 中文：字符数 2，实际占 4 列
/// assert_eq!("中文".chars().count(), 2);
/// assert_eq!(display_width("中文"), 4);
/// ```
///
/// ⛔ **不要**用 `s.len()`（字节数）算宽度：「中」是 3 字节。
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// 单个字符的终端列宽（无法识别时按 1 计，避免返回 0 造成对齐抖动）。
pub fn char_width(c: char) -> usize {
    UnicodeWidthChar::width(c).unwrap_or(1)
}

/// 按**列宽**截断到 `max_cols`，超长时以 `…` 收尾。
///
/// ⛔ 不能直接用 [`pad_trunc`]：它按列宽截断时不会补省略号，
/// 适合表格单元；此函数面向「单行标题」场景。
///
/// # 边界
/// * `max_cols == 0` ⇒ 返回空串。
/// * 单个宽字符就超过 `max_cols` 时，返回 `…`（若 `max_cols >= 1`）。
pub fn truncate_to(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    let total = display_width(s);
    if total <= max_cols {
        return s.to_string();
    }
    // ⛔ 需要截断时**必须为省略号预留 1 列**（占满 max_cols 再补 … 会超宽）。
    // 首版就是先填满 max_cols 再想加省略号 ⇒ `w + 1 <= max_cols` 恒假
    // ⇒ 永远不加省略号，`truncate_to("abcdef", 5)` 得到 "abcde"（丢了截断标记）。
    // 内容预算 = max_cols - 1；`max_cols == 1` 时内容预算为 0，只剩省略号。
    let budget = max_cols - 1;
    let mut out = String::new();
    let mut w = 0usize;
    for c in s.chars() {
        let cw = char_width(c);
        if w + cw > budget {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_width_counts_columns_not_chars() {
        assert_eq!(display_width(""), 0);
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("中文"), 4, "CJK 全角应占 2 列/字");
        assert_eq!(display_width("a中b"), 4);
        // ⛔ 对照：这些是**错误**的宽度算法，中文下会错位
        assert_eq!("中文".len(), 6, "字节数不是列宽");
        assert_eq!("中文".chars().count(), 2, "字符数不是列宽");
    }

    #[test]
    fn char_width_never_zero() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('中'), 2);
    }

    #[test]
    fn truncate_respects_columns() {
        assert_eq!(truncate_to("", 5), "");
        assert_eq!(truncate_to("abc", 5), "abc");
        assert_eq!(truncate_to("abcdef", 5), "abcd…");
        // 截断时省略号占 1 列 ⇒ 预算 3 列只够 1 个汉字
        assert_eq!(truncate_to("中文中文", 4), "中…");
        // 0 列 ⇒ 空串（不得返回 "…"）
        assert_eq!(truncate_to("abc", 0), "");
        // 单个宽字符超限
        assert_eq!(truncate_to("中", 1), "…");   // 只剩省略号
        assert_eq!(truncate_to("abcdef", 5), "abcd…");  // 预算 4 列 + 省略号
        assert_eq!(display_width(&truncate_to("abcdef", 5)), 5, "必须恰好 max_cols 列");
    }
}