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
use crate::table::{pad_to, Align};

/// 面板内容区的列宽（**不含**左右边框 `│ ` 与 `│`）。
///
/// # 边界
/// * `content_width == 0` ⇒ 内容被完全裁掉，只剩边框（仍返回合法字符串）。
pub fn content_line(content: &str, content_width: usize) -> String {
    format!("│ {} │", pad_to(content, content_width, Align::Left))
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
    let t = crate::truncate_to(title, budget);
    // ⚠️ 修正（**宽度不一致**）：标题短于预算时必须**补齐**，
    // 否则顶边比底边窄 ⇒ `panel_edges_align_with_content` 会失败。
    // 首版只截断不补齐 ⇒ 「行宽一致」这个核心不变量不成立。
    let padded = crate::table::pad_to(&t, budget, crate::table::Align::Left);
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
    // 内容宽 = 最长内容的**列宽**
    let content_width = lines
        .iter()
        .map(|l| crate::display_width(l))
        .max()
        .unwrap_or(0);
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
}