//! # nt_tui_theme — TUI 配色集中点（v1 + v2 共用）
//!
//! 立项原因（2026-10-05 实测）：`nt_tui_app.rs`（v2）有**11 处**裸
//! `Style::default()` / `Color::Xxx`，`nt_dialogue_tui.rs`（v1）另有 5 处，
//! 全仓**不存在**任何集中定义的 ratatui 配色（`rg '->\s*Style\b'` 只命中
//! 旧的 `demand_style` 一个）。⇒ 配色语义只存在于散落的字面量里，
//! 改配色要逐处 `rg`，漏一处就不一致。
//!
//! ## 为什么不塞进 `nt_tui_app.rs`
//!
//! v1 与 v2 **都要**用（v1 是活路径：`bin/ntcode.rs` 在 `run_tui_session`
//! 失败时回退到 `NtTuiHuman`）。若主题放在 `nt_tui_app` 里让 v1 反向导入，
//! 就形成 `nt_dialogue_tui → nt_tui_app → nt_dialogue_tui` 的逻辑环。
//! ⇒ 叶子模块，零回边。
//!
//! ## 层门约束（改本文件前必读）
//!
//! ⛔ 本模块**只允许**两条 import：
//! - `ratatui::style::{Color, Style}`
//! - `crate::l1_action::nt_action_facade::NtDemandKind`
//!
//! 理由：`scripts/layer-deps-baseline.txt:8` 已登记
//! `l5_cognition → nt_tui_app.rs`（它直取 L5 的 `nt_crystal_core`），
//! 而 `nt_dialogue_tui.rs` **不在** baseline，正因为它走
//! `nt_action_facade`（`nt_action_facade.rs:49` 是 `NtDemandKind` 的
//! sanctioned 转出点）。本模块若直写 `crate::l5_cognition::…`，
//! `check-layer-deps.sh --strict` 会报 `FAIL: 1 new`。
//!
//! # 一色一名
//!
//! ⚠️ 收编前实测发现**同色多义**，故命名按**语义**而非按色值：
//! - `Yellow` 同时表示「进行中」（工作区/spinner）与「选中」（picker）
//!   ⇒ 拆成 `active()` 与 `selected()` 两个函数，色值相同但语义可分辨。
//! - `Cyan` 同时表示「元信息」（subtask_id/耗时条）与「状态行」
//!   ⇒ `meta()` 与 `status()`。
//! - `demand()` 里 `RecheckMinority` 也是 Cyan ⇒ 与 `meta()` 同色属
//!   既有事实（见 `nt_dialogue_tui.rs::demand_style` 原实现），本模块
//!   **照搬不改动**，以免改变已被测试钉住的行为。
//!
//! # Safety
//! - 纯函数 + const，无 unsafe (R-P1)；生产代码无 `unwrap`/`expect`/`panic!`。
#![forbid(unsafe_code)]

use crate::l1_action::nt_action_facade::NtDemandKind;
use ratatui::style::{Color, Style};

/// 进行中（工作区 Block / spinner）。
pub const FG_ACTIVE: Color = Color::Yellow;

/// 状态行底色。
pub const FG_STATUS: Color = Color::Cyan;

/// 元信息（subtask_id / 耗时条）。
pub const FG_META: Color = Color::Cyan;

/// 弱化文本（流式尾行 / placeholder）。
pub const FG_DIM: Color = Color::DarkGray;

/// 光标前景（反白块）。
pub const FG_CURSOR: Color = Color::White;

/// 光标背景。
pub const CURSOR_BG: Color = Color::DarkGray;

/// 「进行中」：工作区 Block 与 spinner。
pub fn active() -> Style {
    Style::default().fg(FG_ACTIVE)
}

/// 「元信息」：subtask_id 与耗时条。
pub fn meta() -> Style {
    Style::default().fg(FG_META)
}

/// 「状态行」整行。
pub fn status() -> Style {
    Style::default().fg(FG_STATUS)
}

/// 「弱化」：流式答案尾行与输入框 placeholder。
pub fn dim() -> Style {
    Style::default().fg(FG_DIM)
}

/// 光标反白块（bg + fg 同行两个色，故不可拆成两个函数）。
pub fn cursor() -> Style {
    Style::default().bg(CURSOR_BG).fg(FG_CURSOR)
}

/// picker 选中项。
pub fn selected() -> Style {
    Style::default().fg(FG_ACTIVE)
}

/// 未选中项：**显式命名「无样式」**，避免再散落裸 `Style::default()`。
///
/// 与 `selected()` 分开的原因：旧代码两处都写 `Style::default()`，
/// 一个带 `.fg(Yellow)` 一个不带，肉眼极易看漏哪个分支是哪个。
pub fn normal() -> Style {
    Style::default()
}

/// 需求单配色（kind 即 JEV 式紧急度语义）。
///
/// 复核/低置信复核=黄（待审），失败重试=红（失败），矛盾裁决=品红（冲突），
/// 少数派核查=青（求证），跳过确认=蓝（待定），其他=灰。
///
/// ⛔ 本函数是 `nt_dialogue_tui.rs::demand_style` 的**平移**，色值逐字照搬。
/// 该映射被 `nt_dialogue_tui.rs` 的测试钉死（断言各分支 fg），
/// 改色即改行为，故此处不做任何"统一"式重排。
pub fn demand(kind: NtDemandKind) -> Style {
    let fg = match kind {
        NtDemandKind::ReviewFusion | NtDemandKind::RecheckLowConf => Color::Yellow,
        NtDemandKind::RetryFailed => Color::Red,
        NtDemandKind::Adjudicate => Color::Magenta,
        NtDemandKind::RecheckMinority => Color::Cyan,
        NtDemandKind::ConfirmSkipped => Color::Blue,
        NtDemandKind::Other => Color::Gray,
    };
    Style::default().fg(fg)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 钉住调色板。搬进本模块是因为「配色真源在此」，
    /// 断言留在 `nt_dialogue_tui.rs` 只会让人误以为那是真源。
    #[test]
    fn demand_palette_is_stable() {
        assert_eq!(demand(NtDemandKind::ReviewFusion).fg, Some(Color::Yellow));
        assert_eq!(demand(NtDemandKind::RecheckLowConf).fg, Some(Color::Yellow));
        assert_eq!(demand(NtDemandKind::RetryFailed).fg, Some(Color::Red));
        assert_eq!(demand(NtDemandKind::Adjudicate).fg, Some(Color::Magenta));
        assert_eq!(demand(NtDemandKind::RecheckMinority).fg, Some(Color::Cyan));
        assert_eq!(demand(NtDemandKind::ConfirmSkipped).fg, Some(Color::Blue));
        assert_eq!(demand(NtDemandKind::Other).fg, Some(Color::Gray));
    }

    /// 收编的核心不变量：`selected()` 与 `active()` **同色但不同函数**。
    /// 若哪天有人把两者合并成一个，本测试提醒那是语义退化。
    #[test]
    fn same_color_different_semantics() {
        assert_eq!(selected().fg, active().fg);
        assert_eq!(meta().fg, status().fg);
        // 弱化与光标背景同色，但一个只设 fg、一个设 bg。
        assert_eq!(dim().fg, Some(FG_DIM));
        assert_eq!(cursor().bg, Some(CURSOR_BG));
        assert_eq!(cursor().fg, Some(FG_CURSOR));
        // normal 必须真的是「无样式」，否则 selected/normal 二分会退化。
        assert_eq!(normal().fg, None);
    }
}