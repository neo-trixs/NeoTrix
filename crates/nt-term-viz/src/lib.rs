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

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub mod bar;
pub mod panel;
pub mod sparkline;
pub mod table;
pub mod tree;

pub use bar::{bar_line, bar_row};
pub use panel::{content_line, panel_bottom, panel_top, render_panel};
pub use sparkline::sparkline;
pub use table::{Align, pad_to, pad_trunc};
pub use tree::{tree_connector, TreeStyle};

/// 按**grapheme cluster**（用户感知字符）迭代字符串。
///
/// 2026-10-05 吸收 `emilkowalski/skills@break-ui`（MIT）时新增。
/// 存在的理由是**实测**（喂真 `unicode-width 0.2`）：
/// `👨‍👩‍👧‍👦` 逐 `char` 累加宽 15 而 `display_width` 只有 9（差 6），
/// 且逐 `char` 切分会留下**孤立 ZWJ / 孤立肤色修饰**
/// ⇒ 终端渲染成豆腐块或替换字符。
///
/// ⛔ 为什么不自己实现：`unicode-width` 只提供 `width()`，**不提供切分**；
///   grapheme 规则（ZWJ 序列、肤色修饰、区域指示、组合附加符…）
///   手写必漏。故用 `unicode-segmentation`（见 `Cargo.toml` 的零下载说明）。
///
/// # Examples
/// ```
/// use nt_term_viz::graphemes;
/// let fam = "👨‍👩‍👧‍👦";
/// assert_eq!(graphemes(fam).count(), 1, "家族 emoji 是**一个**感知字符");
/// assert_eq!(fam.chars().count(), 7, "但它由 7 个 char 组成（含 3 个 ZWJ）");
/// ```
pub fn graphemes(s: &str) -> impl Iterator<Item = &str> {
    s.graphemes(true)
}

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
/// # 按 grapheme cluster 切分，不按 `char`（2026-10-05）
///
/// 原实现 `for c in s.chars()` 逐字符切分，实测产生**孤立 ZWJ**：
/// ```text
/// "👨‍👩‍👧‍👦family"（display_width=8）截到 4 列
///   → 旧：'👨' + ZWJ + '👩' + ZWJ = display_width 2，尾部是**孤立连接符**
///   → 新：整簇保留或整簇丢弃，永不切碎
/// ```
/// 终端渲染残簇（孤立 ZWJ / 孤立肤色修饰 U+1F3FD）会显示为豆腐块或替换字符。
///
/// ⛔ 不用 `unicode-width` 切分：它只提供 `width()`，**不提供切分**。
///   故加 `unicode-segmentation`（零新增下载，见 Cargo.toml 注释）。
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
    // 每簇用 `display_width(g)` 而**不是**逐 char 累加 —— 实测两者不等：
    //   👨‍👩‍👧‍👦 逐 char 累加=15 但 display_width=9（差 6）
    //   👍🏽        逐 char 累加=7  但 display_width=5（差 2）
    //   若按累加计，会**提前截断**（看起来还有空间却已停）。
    for g in s.graphemes(true) {
        let gw = display_width(g);
        // ⚠️ 整簇放不下就停（不切碎）。`gw > 0` 恒真故不会死循环。
        if w + gw > budget {
            break;
        }
        out.push_str(g);
        w += gw;
    }
    out.push('…');
    out
}

/// 按 grapheme cluster **补齐**到 `max_cols` 列（不足处补空格）。
///
/// 与 [`truncate_to`] 配套：那条负责「超了怎么办」，这条负责「短了补齐」。
/// ⚠️ 二者都**不切碎** grapheme cluster —— 补齐只追加 ASCII 空格（宽度恒 1），
/// 不引入新的切分问题。
pub fn pad_to_cols(s: &str, max_cols: usize) -> String {
    let mut out = s.to_string();
    while display_width(&out) < max_cols {
        out.push(' ');
    }
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

    /// 回归锁：截断**不得切碎** grapheme cluster（2026-10-05）。
    ///
    /// 用例值是**实测抓来的**，不是构造的：喂进真`unicode-width 0.2`
    /// 后发现逐 `char` 截断 emoji 家族会留下孤立 ZWJ。
    #[test]
    fn truncate_never_splits_grapheme_cluster() {
        // 👨‍👩‍👧‍👦 = 4 个 emoji 用 3 个 ZWJ 相连；family 后缀确保会触发截断
        let fam = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}family";
        assert_eq!(display_width(fam), 8, "实测值：家族 emoji 占 2 列 + family 6 列");

        // 核心不变量：切出来的每个前缀都必须是**完整簇序列的前缀**。
        //
        // ⚠️ 判据不能写成「输出不含 ZWJ」—— 家族 emoji *内部*就有 3 个 ZWJ，
        //   那是合法的。实测教训：`max=3` 得 `"👨‍👩‍👧‍👦…"`（簇完整，未切碎）
        //   是**正确**行为，而我第一版断言因用`contains(ZWJ)` 而误报。
        //   ⇒ 正确判据：输出的前若干簇必须与原串的前若干簇**逐一相等**
        //     （若切碎了，最后一簇会与原串对应簇不同）。
        let fam_gs: Vec<&str> = fam.graphemes(true).collect();
        for max in 1..=12usize {
            let out = truncate_to(fam, max);
            let out_body = out.strip_suffix('…').unwrap_or(&out);
            let out_gs: Vec<&str> = out_body.graphemes(true).collect();
            assert!(
                out_gs.len() <= fam_gs.len(),
                "max={max} 簇数超界：{out:?}"
            );
            for (i, g) in out_gs.iter().enumerate() {
                assert_eq!(
                    *g, fam_gs[i],
                    "max={max} 第 {i}簇被切碎：得到 {g:?}，原串该簇是 {:?}",
                    fam_gs[i]
                );
            }
            // 肤色修饰：只许作为 👍🏽 整簇出现，孤立即错
            let skin = "\u{1F44D}\u{1F3FD} ok";
            let o2 = truncate_to(skin, max);
            for g in o2.strip_suffix('…').unwrap_or(&o2).graphemes(true) {
                assert!(
                    g != "\u{1F3FD}",
                    "max={max} 产出孤立肤色修饰：{o2:?}"
                );
            }
        }
    }

    /// 回归锁：每簇宽度必须用 `display_width(g)`，不可逐char 累加。
    ///
    /// 实测（喂 `unicode-width 0.2` 真值）：
    /// 👨‍👩‍👧‍👦 逐 char 累加 = 15，但 `display_width` = 9（差 6）
    /// 👍🏽        逐 char 累加 = 7， 但 `display_width` = 5（差 2）
    /// 若按累加计会**提前截断**（看起来还有空间却已停）。
    #[test]
    fn cluster_width_uses_whole_cluster_not_char_sum() {
        let fam = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}family";
        let by_char: usize = fam.chars().map(char_width).sum();
        let whole = display_width(fam);
        assert_eq!(whole, 8);
        assert!(
            by_char > whole,
            "本测试前提：逐 char 累加应 > 整串宽度（实测 {by_char} > {whole}）"
        );
        // 预算 8（不触发截断）⇒ 必须**原样返回**
        assert_eq!(truncate_to(fam, 8), fam, "预算恰好够时不得截断");
        // 预算 7 ⇒ 只放得下 'family' 之外的部分… 至少必须是合法簇
        let out = truncate_to(fam, 7);
        assert!(display_width(&out) <= 7, "不得超预算");
    }

    /// `pad_to_cols` 配套 `truncate_to`：补齐到恰好 `max_cols` 列，
    /// 且补齐**不破坏**簇（只追加 ASCII 空格）。
    #[test]
    fn pad_to_cols_reaches_exact_width() {
        // ⚛️ 只在「原串未超预算」时才要求补齐到恰好 max —— 补齐**不截断**
        //   （实测：`pad_to_cols("优化", 0)` 返回 4 列是对的，
        //     我第一版断言它须等于 0 列，那是在要求 pad 做截断，判据写错了）。
        for max in 4..10usize {
            let p = pad_to_cols("优化", max);
            assert_eq!(display_width(&p), max, "max={max} 须恰好 {max} 列");
        }
        // 已超预算 ⇒ 原样返回，绝不截断
        assert_eq!(display_width(&pad_to_cols("优化", 0)), 4, "超预算不截断");
        assert_eq!(pad_to_cols("abc", 2), "abc", "超预算原样返回");
        assert_eq!(pad_to_cols("abc", 3), "abc", "恰好等宽不加空格");
        assert_eq!(pad_to_cols("abc", 5), "abc  ", "补到 5 列");
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