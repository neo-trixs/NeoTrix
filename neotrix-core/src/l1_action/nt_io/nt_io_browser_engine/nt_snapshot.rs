//! nt_snapshot — EVO-04 步骤1：原子 DOM 快照表（jev-ultrafast 思想）。
//!
//! 把 [`PageSnapshot`] 压成 `[id] type name/value` 文本表：链接与表单字段
//! 统一编号（`L<n>` 链接，`F<fi>#<li>` 字段），行数上限 [`MAX_ROWS`]，
//! 只发可见文本（标题＋表），无截图默认环。`resolve` 把 id 映回
//! selector 语义（供步骤2 单 RTT `{snapshot_id,operation,target_id}` 用）。
//! 同步纯逻辑，无 IO。

use super::types::{FormSpec, LinkRef, PageSnapshot};

/// 快照表行数上限。
pub const MAX_ROWS: usize = 200;

/// 快照表（文本＋id 索引）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotTable {
    /// 渲染文本（`[id] type name/value` 每行一条）。
    pub text: String,
    /// id 顺序表（行号即 id 序号）。
    pub ids: Vec<String>,
}

impl SnapshotTable {
    pub fn row_count(&self) -> usize {
        self.ids.len()
    }

    /// 按 id 取回元素定位（链接 href / 字段 `form#field`）。
    pub fn resolve(&self, id: &str) -> Option<String> {
        if self.ids.iter().any(|i| i == id) {
            Some(id.to_owned())
        } else {
            None
        }
    }
}

fn push_row(text: &mut String, ids: &mut Vec<String>, id: String, kind: &str, name: &str) {
    ids.push(id.clone());
    text.push('[');
    text.push_str(&id);
    text.push_str("] ");
    text.push_str(kind);
    text.push(' ');
    text.push_str(name);
    text.push('\n');
}

/// 链接行名：text 为空时回落 href（截 60 字）。
fn link_name(link: &LinkRef) -> String {
    let t = link.text.trim();
    if t.is_empty() {
        link.href.chars().take(60).collect()
    } else {
        t.chars().take(60).collect()
    }
}

/// 字段行名：`form[action] name=value`（各截 40 字）。
fn field_name(form: &FormSpec, fi: usize, name: &str, value: &str) -> String {
    let act: String = form.action.chars().take(40).collect();
    let name: String = name.chars().take(40).collect();
    let value: String = value.chars().take(40).collect();
    format!("F{fi}[{act}] {name}={value}")
}

/// 从快照构建元素表（链接优先，字段随后；超限截断）。
pub fn build_table(snap: &PageSnapshot) -> SnapshotTable {
    let mut text = String::new();
    let mut ids = Vec::new();
    for (i, link) in snap.links.iter().enumerate() {
        if ids.len() >= MAX_ROWS {
            break;
        }
        push_row(&mut text, &mut ids, format!("L{i}"), "link", &link_name(link));
    }
    for (fi, form) in snap.forms.iter().enumerate() {
        for (li, field) in form.fields.iter().enumerate() {
            if ids.len() >= MAX_ROWS {
                break;
            }
            push_row(
                &mut text,
                &mut ids,
                format!("F{fi}#{li}"),
                "field",
                &field_name(form, fi, &field.name, &field.value),
            );
        }
    }
    SnapshotTable { text, ids }
}

// ============================================================================
// EVO-04 步骤2：单 RTT 操作包（数据契约；引擎映射在浏览器窗 e2e 时接入）
// ============================================================================

/// 单请求操作包：一次往返只带一个 op（禁多 op 打包）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleOp {
    /// 快照序号（执行前与引擎侧序号比对，不一致即 stale）。
    pub snapshot_seq: u64,
    /// 操作动词（click/type/select/press…，与 BrowserAction 动作名同义）。
    pub operation: String,
    /// 目标元素 id（`L<n>` / `F<fi>#<li>`）。
    pub target_id: String,
}

impl SingleOp {
    /// 构造（空动词/空目标拒绝，防无头请求）。
    pub fn new(snapshot_seq: u64, operation: impl Into<String>, target_id: impl Into<String>) -> Option<Self> {
        let operation = operation.into();
        let target_id = target_id.into();
        if operation.is_empty() || target_id.is_empty() {
            return None;
        }
        Some(Self { snapshot_seq, operation, target_id })
    }

    /// 绑定快照表校验（目标必须在表中，否则疑似过期 DOM）。
    pub fn bind(&self, table: &SnapshotTable, engine_seq: u64) -> Result<(), SnapshotError> {
        if self.snapshot_seq != engine_seq {
            return Err(SnapshotError::Stale);
        }
        if table.resolve(&self.target_id).is_none() {
            return Err(SnapshotError::Occluded);
        }
        Ok(())
    }
}

/// 快照错误（步骤3 校验双检）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotError {
    /// 序号漂移（页面已变）。
    Stale,
    /// 目标不在表（遮挡/移除）。
    Occluded,
}

// ============================================================================
// EVO-04 步骤3：按需等待（无截图默认环；等待只在 combobox/异步处加）
// ============================================================================

/// combobox 展开等待（ms）。
pub const COMBOBOX_WAIT_MS: u64 = 200;
/// 其余操作等待上限（ms）。
pub const DEFAULT_WAIT_MS: u64 = 50;

/// 按需等待量（combobox 200ms，其余 50ms 上限；纯函数）。
pub fn wait_for(operation: &str) -> u64 {
    if operation.eq_ignore_ascii_case("select") || operation.eq_ignore_ascii_case("combobox") {
        COMBOBOX_WAIT_MS
    } else {
        DEFAULT_WAIT_MS
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{FormField, FormSpec, LinkRef, PageSnapshot};
    use super::*;

    fn snap() -> PageSnapshot {
        PageSnapshot {
            url: "https://x.test/".to_owned(),
            title: Some("t".to_owned()),
            text: "hi".to_owned(),
            raw_html: String::new(),
            http_status: Some(200),
            links: vec![
                LinkRef { index: 0, text: "go".to_owned(), href: "https://x.test/a".to_owned() },
                LinkRef { index: 1, text: String::new(), href: "https://x.test/b".to_owned() },
            ],
            forms: vec![FormSpec {
                index: 0,
                action: "/s".to_owned(),
                method: "get".to_owned(),
                fields: vec![FormField {
                    name: "q".to_owned(),
                    kind: "text".to_owned(),
                    value: String::new(),
                }],
            }],
        }
    }

    #[test]
    fn table_rows_and_ids_stable() {
        let t = build_table(&snap());
        assert_eq!(t.row_count(), 3);
        assert!(t.text.contains("[L0] link go"));
        assert!(t.text.contains("[L1] link https://x.test/b"));
        assert!(t.text.contains("[F0#0] field"));
        assert_eq!(t.resolve("L0"), Some("L0".to_owned()));
        assert_eq!(t.resolve("Z9"), None);
    }

    #[test]
    fn empty_snapshot_empty_table() {
        let s = PageSnapshot {
            url: String::new(),
            title: None,
            text: String::new(),
            links: Vec::new(),
            forms: Vec::new(),
            raw_html: String::new(),
            http_status: None,
        };
        let t = build_table(&s);
        assert_eq!(t.row_count(), 0);
        assert!(t.text.is_empty());
    }

    #[test]
    fn rows_truncate_at_max() {
        let mut s = snap();
        s.links = (0..300)
            .map(|i| LinkRef { index: i, text: format!("l{i}"), href: format!("https://x.test/{i}") })
            .collect();
        let t = build_table(&s);
        assert_eq!(t.row_count(), MAX_ROWS);
    }

    #[test]
    fn long_names_truncated() {
        let s = PageSnapshot {
            url: String::new(),
            title: None,
            text: String::new(),
            links: vec![LinkRef { index: 0, text: "a".repeat(100), href: "h".to_owned() }],
            forms: Vec::new(),
            raw_html: String::new(),
            http_status: None,
        };
        let t = build_table(&s);
        let line = t.text.lines().next().unwrap_or("");
        assert!(line.len() < "[L0] link ".len() + 61);
    }

    #[test]
    fn single_op_rejects_empty() {
        assert!(SingleOp::new(1, "", "L0").is_none());
        assert!(SingleOp::new(1, "click", "").is_none());
        assert!(SingleOp::new(1, "click", "L0").is_some());
    }

    #[test]
    fn bind_checks_seq_then_target() {
        let t = build_table(&snap());
        let op = SingleOp::new(7, "click", "L0").unwrap_or(SingleOp {
            snapshot_seq: 0,
            operation: String::new(),
            target_id: String::new(),
        });
        assert_eq!(op.bind(&t, 8), Err(SnapshotError::Stale));
        let ghost = SingleOp::new(7, "click", "L9").unwrap_or(SingleOp {
            snapshot_seq: 0,
            operation: String::new(),
            target_id: String::new(),
        });
        assert_eq!(ghost.bind(&t, 7), Err(SnapshotError::Occluded));
        assert!(op.bind(&t, 7).is_ok());
    }

    #[test]
    fn wait_policy_on_demand() {
        assert_eq!(wait_for("select"), COMBOBOX_WAIT_MS);
        assert_eq!(wait_for("ComboBox"), COMBOBOX_WAIT_MS);
        assert_eq!(wait_for("click"), DEFAULT_WAIT_MS);
    }
}
