//! 树连线原语 —— 取代各站点内联的硬编码 `├─`。
//!
//! 现状（实测）：`nt-core-capability-tree/src/cli.rs:488` 唯一一处树渲染是
//! **内联 `println!("    ├─ {} ...")`**，无法调整层级/样式，也无法对齐。
//!
//! 本模块只提供**连线前缀**（不含缩进宽度计算），保持职责单一：
//! 缩进宽度由调用方按层级自行决定。
use crate::char_width;

/// 树连线风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeStyle {
    /// ASCII 兜底（`|--` / ``--``）—— 供不支持 Unicode 的终端使用
    Ascii,
    /// Unicode 制表符（`├──` / `└──`）—— 默认
    Unicode,
}

/// 取某个深度的连线前缀。
///
/// * `depth == 0` ⇒ 空串（根节点前无连线）。
/// * `is_last == true` ⇒ 用「末项」符号（`└──` / ``--``）。
/// * 否则用「中间项」符号（`├──` / `|--`）。
///
/// ⛔ **不返回缩进**：缩进宽度取决于调用方的层级模型，
/// 在本模块里猜会导致调用方重复计算。
///
/// # 例子
/// ```
/// use nt_term_viz::tree::{tree_connector, TreeStyle};
/// assert_eq!(tree_connector(0, false, TreeStyle::Unicode), "");
/// assert_eq!(tree_connector(1, true, TreeStyle::Unicode), "└── ");
/// assert_eq!(tree_connector(1, false, TreeStyle::Unicode), "├── ");
/// assert_eq!(tree_connector(2, true, TreeStyle::Ascii), "`-- ");
/// ```
pub fn tree_connector(depth: usize, is_last: bool, style: TreeStyle) -> String {
    if depth == 0 {
        return String::new();
    }
    let branch = match (style, is_last) {
        (TreeStyle::Ascii, true) => "`-- ",
        (TreeStyle::Ascii, false) => "|-- ",
        (TreeStyle::Unicode, true) => "└── ",
        (TreeStyle::Unicode, false) => "├── ",
    };
    String::from(branch)
}

/// 为一层**兄弟节点**生成各自的前缀：`[items.len()]` 个，第 `i` 个据 `is_last` 判定。
///
/// 这样调用方不必自己判断「谁最后」，避免 `i + 1 == len` 这类易错逻辑散落各处。
pub fn sibling_prefixes(len: usize, style: TreeStyle) -> Vec<String> {
    (0..len)
        .map(|i| tree_connector(1, i + 1 == len, style))
        .collect()
}

/// 校验一串连线字符**均为单列宽**。
///
/// ⛔ 若混入全角字符（例如误用 `－` 全角减号而非 `-`），
/// 整棵树会逐级错位。此函数供调用方在建树前自检。
pub fn connectors_single_width(s: &str) -> bool {
    s.chars().all(|c| char_width(c) == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_zero_has_no_connector() {
        assert_eq!(tree_connector(0, true, TreeStyle::Unicode), "");
        assert_eq!(tree_connector(0, false, TreeStyle::Ascii), "");
    }

    #[test]
    fn unicode_and_ascii_styles() {
        assert_eq!(tree_connector(1, true, TreeStyle::Unicode), "└── ");
        assert_eq!(tree_connector(1, false, TreeStyle::Unicode), "├── ");
        assert_eq!(tree_connector(1, true, TreeStyle::Ascii), "`-- ");
        assert_eq!(tree_connector(1, false, TreeStyle::Ascii), "|-- ");
    }

    #[test]
    fn connectors_are_single_width() {
        for style in [TreeStyle::Unicode, TreeStyle::Ascii] {
            for last in [true, false] {
                let c = tree_connector(1, last, style);
                assert!(
                    connectors_single_width(&c),
                    "{style:?}/{last} 的连线 `{c}` 含非单宽字符"
                );
                // 4 个字符（符号3 + 空格1）⇒ 必须占 4 列
                assert_eq!(crate::display_width(&c), 4);
            }
        }
    }

    #[test]
    fn sibling_prefixes_marks_only_last() {
        let p = sibling_prefixes(3, TreeStyle::Unicode);
        assert_eq!(p.len(), 3);
        assert_eq!(p[0], "├── ");
        assert_eq!(p[1], "├── ");
        assert_eq!(p[2], "└── ", "只有最后一个应是末项符号");
        // 单元素时它就是末项
        assert_eq!(sibling_prefixes(1, TreeStyle::Unicode), vec!["└── "]);
        assert!(sibling_prefixes(0, TreeStyle::Unicode).is_empty());
    }

    #[test]
    fn connector_width_is_depth_independent() {
        // ⛔ 缩进不归本模块管 ⇒ 同一层级内宽度恒定，
        //    这正是对齐能成立的前提
        let a = tree_connector(1, false, TreeStyle::Unicode);
        let b = tree_connector(9, false, TreeStyle::Unicode);
        assert_eq!(crate::display_width(&a), crate::display_width(&b));
    }
}