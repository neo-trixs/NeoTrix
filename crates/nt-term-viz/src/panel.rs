//! 面板原语 —— 带**左右边框**的定宽块。
//!
//! # 为什么需要（实测依据，2026-10-02）
//!
//! `neotrix-core/src/entry/` 下 7 个入口（brain / browse / clean / consciousness /
//! headless / proxy_cmd / standalone）都是手写面板：
//! ```text
//! println!("╭─ NeoTrix 意识核心 (ConsciousnessCore) ───────────────╮");
//! println!("│ 周期      {:>54}", cycle);
//! println!("│ 相位(Φ)   {:>53.4}", phi);
//! ```
//!
//! **实测错位**：Rust 的 `{:>N}` 按**字符数**补足，而中文占 2 列 ⇒
//! 各行错位量**不同**：
//!
//! | 字面 | 字符数 | 视觉宽 | 错位 |
//! |---|---|---|---|
//! | `│ 周期      ` | 10 | 12 | **+2 列** |
//! | `│ 相位(Φ)   ` | 10 | 12 | **+2 列** |
//! | `│ 相干性    ` | 9 | 12 | **+3 列** |
//! | `│ 谐振周期  ` | 8 | 12 | **+4 列** |
//!
//! ⇒ **右边框参差不齐**。本模块提供按**列宽**计算的面板，
//! 使每一行的 `│` 都落在同一终端列。
//!
//! # 与 [`crate::table`] 的分工
//! * [`crate::table`]：单元格对齐（表格、键值对）
//! * 本模块：**整体外框**（顶边/底边/带左边框的内容行）
use crate::table::{pad_to_visible, visible_width, Align};

/// 面板内容区的列宽（**不含**左右边框 `│ ` 与 `│`）。
///
/// # 边界
/// * `content_width == 0` ⇒ 内容被完全裁掉，只剩边框（仍返回合法字符串）。
///
/// # ⚠️ ANSI 安全性（2026-10-04 修正）
///
/// 首版用 `pad_to`，而 `pad_to` 内部 `display_width` **把 ANSI 计入列宽**
/// ⇒ 彩色内容命中 `w >= cols` 分支 ⇒ **一个空格都不补** ⇒ 行短一截、边框参差。
/// 现改用 [`pad_to_visible`]：**测宽剥 ANSI、输出保留染色**。
pub fn content_line(content: &str, content_width: usize) -> String {
    format!("│ {} │", pad_to_visible(content, content_width, Align::Left))
}

/// 面板顶边：`╭─ 标题 ────╮`，总宽 = `content_width + 4`。
///
/// 标题过长时截断并保底 `╭─╮`（3 列）。
pub fn panel_top(title: &str, content_width: usize) -> String {
    let total = content_width + 4;
    // ⚠️ 2026-10-02 修正（**下溢**）：预算需要 `total >= 6`
    //（`╭─ ` 3 列 + ` ─╮` 3 列）。首版守卫写成 `total <= 3`
    // ⇒ `total` 在 4~5 时 `total - 6` **usize 下溢 panic**。
    if total < 6 {
        // 退化情形：直接用与 `panel_bottom` **对称**的纯边框，
        // 保证「顶边与底边等宽」这个核心不变量在所有宽度下都成立。
        // ⚠️ 首版这里返回 `panel_bottom(..)`（即 `╰──╯`）⇒ 顶边用了底边的字符，
        //    明显不合理 —— 那是为绕开下溢临时借的。
        if total <= 3 {
            return "╭─╮".to_string();
        }
        return format!("╭{}╮", "─".repeat(total - 2));
    }
    // 预算：total - 「╭─ 」(3) - 「 ─╮」(3)
    let budget = total - 6;
    // ⚠️ 2026-10-04 修正（**标题含 ANSI 时错位**）：`truncate_to` 与 `pad_to`
    // 都按 `display_width` 算，而它**把 ANSI 计入列宽** ⇒ 带染色的标题
    // 会算成超宽 ⇒ 明明放得下也被截断，或补齐量算错。
    // ⇒ 判「是否超预算」用 `visible_width`；**放得下就原样保留颜色**，
    //    只有真超了才剥色截断（保守：宁可少个颜色，也不要框线错位）。
    let padded = if visible_width(title) <= budget {
        pad_to_visible(title, budget, Align::Left)
    } else {
        // 剥色后截断，保证截断量按真实列宽计算。
        let plain = crate::table::strip_ansi(title);
        crate::table::pad_to(&crate::truncate_to(&plain, budget), budget, Align::Left)
    };
    format!("╭─ {padded} ─╮")
}

/// 面板底边：`╰────╯`，总宽 = `content_width + 4`。
pub fn panel_bottom(content_width: usize) -> String {
    let total = content_width + 4;
    if total <= 3 {
        return "╰─╯".to_string();
    }
    format!("╰{}╯", "─".repeat(total - 2))
}

/// 一次性构造一个完整面板的各行（**不含末尾换行**）。
///
/// # 例子
/// ⚠️ 标题短于预算时会被**补齐**（而非只截断）——
/// 否则顶边会比底边窄，「所有行视觉宽度一致」这个核心不变量就不成立。
/// ```
/// use nt_term_viz::panel::render_panel;
/// let rows = render_panel("状态", &["周期: 1", "相位: 0.9"]);
/// assert_eq!(rows[0], "╭─ 状态    ─╮");   // ← 注意「状态」后是 4 个空格
/// assert!(rows[1].starts_with('│') && rows[1].ends_with('│'));
/// ```
pub fn render_panel(title: &str, lines: &[&str]) -> Vec<String> {
    // 内容宽 = 最长内容的**可见列宽**（ANSI 不计列）
    let content_width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
    let mut out = vec![panel_top(title, content_width)];
    for l in lines {
        out.push(content_line(l, content_width));
    }
    out.push(panel_bottom(content_width));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display_width;

    #[test]
    fn panel_rows_share_one_visual_width() {
        // ⛔ 这正是 consciousness.rs 的真实错位场景：中文标签长度不同
        let rows = render_panel(
            "意识核心",
            &["周期: 1", "相位: 0.95", "相干性: 0.8", "谐振周期: 3"],
        );
        let widths: Vec<usize> = rows.iter().map(|r| display_width(r)).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "面板每行视觉宽度应一致（右边框对齐），实测 {widths:?}"
        );
    }

    #[test]
    fn panel_edges_align_with_content() {
        let rows = render_panel("T", &["中文", "ab"]);
        let w = display_width(&rows[0]);
        for r in &rows {
            assert_eq!(display_width(r), w, "行宽应一致: {r:?}");
        }
        assert!(rows[0].starts_with('╭') && rows[0].ends_with('╮'));
        let last = rows.len() - 1;
        assert!(rows[last].starts_with('╰') && rows[last].ends_with('╯'));
        for r in &rows[1..last] {
            assert!(r.starts_with('│') && r.ends_with('│'), "内容行应有左右边框: {r:?}");
        }
    }

    #[test]
    fn content_line_pads_by_columns() {
        // 「中文」4 列，「ab」2 列 ⇒ 两者补齐后内容区等宽
        let a = content_line("中文", 6);
        let b = content_line("ab", 6);
        assert_eq!(display_width(&a), display_width(&b));
        assert_eq!(display_width(&a), 6 + 4); // │ + space + 6 + space + │
    }

    #[test]
    fn degenerate_widths_are_safe() {
        // 不得 panic
        let _ = render_panel("", &[]);
        let _ = panel_top("很长的标题标题标题标题标题", 0);
        let _ = panel_bottom(0);
        // total = 4 ⇒ 顶/底都是 4 列纯边框，且**对称**
        assert_eq!(panel_top("x", 0), "╭──╮");
        assert_eq!(panel_bottom(0), "╰──╯");
        assert_eq!(
            crate::display_width(&panel_top("x", 0)),
            crate::display_width(&panel_bottom(0))
        );
    }

    #[test]
    fn long_title_is_truncated_not_overflowed() {
        let top = panel_top("这是一个非常非常长的标题需要被截断", 20);
        assert_eq!(display_width(&top), 24, "顶边总宽应为 content_width + 4");
        assert!(top.starts_with("╭─ "));
        assert!(top.ends_with(" ─╮"));
    }

        #[test]
    fn empty_panel_has_no_content_rows() {
        let rows = render_panel("T", &[]);
        assert_eq!(rows.len(), 2, "无内容时只有顶边与底边");
    }

    // ─────────────────────────────────────────────────────────────
    // ⭐ ANSI 反向锁（2026-10-04）
    //
    // 缺陷形状（实测依据）：
    //   `display_width("\e[31m中\e[0m") == 11`（见 table.rs 既有测试）
    //   ⇒ `pad_to` 把 ANSI 计入列宽 ⇒ 命中 `w >= cols` 分支
    //   ⇒ **原样返回、零补齐** ⇒ 行短一截、右边框参差。
    //
    // 为什么必须锁：`content_line` 是 3 个 entry 面板的公共原语，
    // 而 `entry::info()/warn()/err()` **全部产 ANSI**（`.cyan()/.yellow()`）。
    // ⇒ 只要哪个入口把染色内容喂进来就会静默错位，编译器与肉眼都难发现。
    // ─────────────────────────────────────────────────────────────

    /// 彩色内容行必须补齐到与纯文本行**相同的可见宽度**，且**保留染色**。
    #[test]
    fn content_line_is_ansi_safe_and_keeps_color() {
        let plain = content_line("Iteration: 3", 24);
        let colored = content_line("\u{1b}[36mIteration:\u{1b}[0m 3", 24);
        // ⛔ 必须用 `visible_width` 断言，**不能**用 `display_width`：
        //    后者故意把 ANSI 计入 ⇒ 彩色行必然更「宽」，
        //    拿它断言「一致」等于断言一个假命题（首版就这么写错的）。
        assert_eq!(
            visible_width(&plain),
            visible_width(&colored),
            "彩色行与纯文本行的可见宽度应一致（右边框对齐）"
        );
        assert_eq!(visible_width(&colored), 24 + 4, "实测：内容区 24 + `│ ` 2 + `│` 2");
        // 实测 dw=37 / vw=28 ⇒ 差 9 恰为这段 ANSI 的字节数
        assert_eq!(
            crate::display_width(&colored) - visible_width(&colored),
            9,
            "display_width 与 visible_width 之差应恰为 ANSI 字节数"
        );
        // 颜色必须还在 —— 若误用 `pad_to_ansi_safe` 这里会失败
        assert!(
            colored.contains('\u{1b}'),
            "content_line 不得抹掉调用方的染色：{colored:?}"
        );
    }

    /// 整面板：混合彩色/纯文本/CJK，所有行可见宽度一致。
    #[test]
    fn render_panel_is_ansi_safe_across_mixed_lines() {
        let rows = render_panel(
            "\u{1b}[1mBrain\u{1b}[0m",
            &[
                "\u{1b}[36mIteration:\u{1b}[0m 1",
                "相干性: 0.95",
                "\u{1b}[33mCapability Sum:\u{1b}[0m 12.5",
            ],
        );
        // 同上：用 visible_width。实测 vw 全部 = 24；
        // 而 display_width 是 [32, 33, 24, 33, 24] ⇒ 参差全部来自 ANSI。
        let widths: Vec<usize> = rows.iter().map(|r| visible_width(r)).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "彩色面板每行可见宽度应一致，实测 {widths:?}"
        );
        assert!(
            rows.iter().filter(|r| r.contains('\u{1b}')).count() >= 3,
            "染色必须被保留"
        );
    }

    /// 反向锁：若把 `pad_to_visible` 换回 `pad_to`，本测试必须失败。
    #[test]
    fn reverse_lock_pad_to_visible_is_what_fixes_alignment() {
        // 证伪「display_width 会自动处理 ANSI」这一错误前提
        let colored = "\u{1b}[36mIteration:\u{1b}[0m 1";
        // `\e[36m`=5 + `Iteration:`=10 + `\e[0m`=4 + ` 1`=2 = 21；
        // 可见部分只有 `Iteration: 1` = 12。
        assert_eq!(crate::display_width(colored), 21, "前提：display_width 把 ANSI 计入");
        assert_eq!(visible_width(colored), 12, "visible_width 只算可见列");

        // 旧路径（pad_to）：dw=21 ≥ 20 ⇒ 命中「原样返回」⇒ 零补齐，
        // 可见宽仍 12 而非 20 ⇒ 右边框参差的直接来源。
        let old = crate::table::pad_to(colored, 20, Align::Left);
        assert_eq!(visible_width(&old), 12, "旧路径因 ANSI 虚高而放弃补齐");
        // 新路径（pad_to_visible）：按 vw=12 补 8 空格 ⇒ 可见宽 20，保留染色
        let new = pad_to_visible(colored, 20, Align::Left);
        assert_eq!(visible_width(&new), 20, "新路径补齐到 20 可见列");
        assert!(new.contains('\u{1b}'), "新路径保留染色");
    }
}
