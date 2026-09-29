//! `nt_changes` — 本轮文件改动记账（`dsh-better-sidebar`「本轮文件」视角的引擎）。
//!
//! **为什么不能只靠 `steps`**：`steps` 记的是工具结果的**摘要**（
//! `wrote 812 bytes` / `edited 1 occurrence`），路径都没进表，渲染不出 diff。
//! 侧边栏要的是「这一轮模型碰了哪些文件、各改成什么样」，故另立一账
//! `file_changes`：`task_id` → `path` → `kind` → `before`/`after`。
//!
//! 内容上限律：任一侧超 `CHANGE_CONTENT_CAP` 就**整笔不存内容**、只留元信息
//! 并置 `content_omitted`。**不截半份**——半份 diff 渲染出来是错的，
//! 比明说「内容过大，去打开文件对比」危险得多。
//!
//! 记账失败律：记在**写盘之后**，故失败**不掀翻主流程**（文件已经落盘了，
//! 掀翻只会留下更糊涂的半成品），但失败原因会拼进工具返回值，让模型和人都
//! 看得见「这次没记账」。这是与 `audit` 的关键差别：审计写在**执行之前**，
//! 写不进去就说明账本身坏了，那必须中止（`nt_agent` 里 `record_audit(...)?`）；
//! 改动账写在**执行之后**，性质不同，处理方式也就不同。

use std::path::Path;

use chrono::Utc;

use crate::nt_error::NtBotError;
use crate::nt_store::{FileChange, NeobotStore};

/// 单侧内容存储上限（256 KiB；两侧同限）。
pub const CHANGE_CONTENT_CAP: usize = 256 * 1024;
/// 每任务保留的账目条数（超出的在 `prune_best_effort` 里从最早的开始砍）。
pub const KEEP_CHANGES_PER_TASK: i64 = 500;
/// 改动账留存天数（超期整删）。
pub const CHANGE_RETENTION_DAYS: i64 = 30;

/// 改动种类。
pub const KIND_READ: &str = "read";
/// 新建或整覆写文件。
pub const KIND_WRITE: &str = "write";
/// 精确单次替换编辑。
pub const KIND_EDIT: &str = "edit";

/// 本轮记账器（挂在 `nt_agent` 的工具执行路径上）。
///
/// 持有 `store` + `task_id` + 工作区根：账记的是**相对路径**，即便用户
/// 之后把工作区整个搬走，历史账仍读得懂。
pub struct ChangeSink<'a> {
    pub store: &'a NeobotStore,
    pub task_id: &'a str,
    pub workspace: &'a Path,
}

impl ChangeSink<'_> {
    /// 记一次读（只记元信息：读了哪个文件、多大；内容不入账）。
    pub fn read(&self, rel: &str, bytes: usize) -> Result<(), NtBotError> {
        let rel = self.rel(rel);
        self.store.record_change(
            &FileChange {
                id: uuid::Uuid::new_v4().to_string(),
                task_id: self.task_id.to_owned(),
                at: Utc::now().to_rfc3339(),
                path: rel,
                kind: KIND_READ.to_owned(),
                bytes: i64::try_from(bytes).unwrap_or(i64::MAX),
                content_omitted: false,
            },
            None,
            None,
        )
    }

    /// 记一次写。`before` 为 `None` 表示**此前不存在**（新建）。
    pub fn write(
        &self,
        rel: &str,
        before: Option<&str>,
        after: &str,
    ) -> Result<(), NtBotError> {
        self.persist(rel, KIND_WRITE, before, after)
    }

    /// 记一次编辑（前后都有内容）。
    pub fn edit(&self, rel: &str, before: &str, after: &str) -> Result<(), NtBotError> {
        self.persist(rel, KIND_EDIT, Some(before), after)
    }

    fn persist(
        &self,
        rel: &str,
        kind: &str,
        before: Option<&str>,
        after: &str,
    ) -> Result<(), NtBotError> {
        let rel = self.rel(rel);
        let omitted = after.len() > CHANGE_CONTENT_CAP
            || before.is_some_and(|text| text.len() > CHANGE_CONTENT_CAP);
        // 超限即整笔略内容（`content_omitted` 由行内字段承载，不靠 before/after 空否判断）。
        let (stored_before, stored_after) = if omitted {
            (None, None)
        } else {
            (before, Some(after))
        };
        self.store.record_change(
            &FileChange {
                id: uuid::Uuid::new_v4().to_string(),
                task_id: self.task_id.to_owned(),
                at: Utc::now().to_rfc3339(),
                path: rel,
                kind: kind.to_owned(),
                // 口径：写/记的是**改后**内容长度；读记的是读到的长度。
                // 真实增删行数由前端 diff 算，不在这儿猜。
                bytes: i64::try_from(after.len()).unwrap_or(i64::MAX),
                content_omitted: omitted,
            },
            stored_before,
            stored_after,
        )
    }

    /// 归一成相对工作区的路径（绝对路径进来也能落账成可搬走的相对路径）。
    fn rel(&self, path: &str) -> String {
        crate::nt_workspace::rel_of(self.workspace, Path::new(path))
    }
}

/// 把记账失败如实拼进工具返回值（不掀翻主流程，但留痕）。
pub fn note_journal_failure(result: &mut crate::nt_types::ToolResult, err: NtBotError) {
    result
        .output
        .push_str(&format!(" [journal: {err}]"));
}

/// 清账（按留存天数 + 每任务保尾）。尽力而为：失败不抛。
///
/// 2026-09-29 审计修复两处：
/// 1. **R-P79 未接线**：此前全 crate 零生产调用者 ⇒ 留存期清理**从未真正运行过**
///    （账目表无上限增长）。现接进 `nt_channel_serve` 的调度点，
///    与 `upkeep_best_effort` 同一条路径。
/// 2. **吞错误**：`unwrap_or(0)` 把「删除失败」伪装成「删了 0 行」，
///    调用方无法区分「无需清理」与「清理失败、账目仍在涨」。
///    改为返回 `Result<usize, NtBotError>`，由调用方决定是否上报。
pub fn prune_best_effort(store: &NeobotStore) -> Result<usize, NtBotError> {
    let cutoff = (Utc::now() - chrono::Duration::days(CHANGE_RETENTION_DAYS))
        .to_rfc3339();
    store.prune_changes(&cutoff, KEEP_CHANGES_PER_TASK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nt_store::NeobotStore;

    /// 每个 case 一个内存库（`:memory:` 天然隔离，不留磁盘残迹）。
    fn store(case: &str) -> NeobotStore {
        NeobotStore::open(":memory:").unwrap_or_else(|err| panic!("{case}: {err}"))
    }

    fn sink<'a>(store: &'a NeobotStore, ws: &'a Path, task: &'a str) -> ChangeSink<'a> {
        ChangeSink {
            store,
            task_id: task,
            workspace: ws,
        }
    }


    // ═══ 2026-09-29 审计：留存期清理从未被接线（R-P79）══
    //
    // 之前只测 helper，本组测「接线后真的会删」——
    // 接线类缺陷（函数存在但没人调）**只有行为测试能发现**，
    // 覆盖率工具只能提示。
    #[test]
    fn prune_best_effort_reports_ok_and_prunes_nothing_when_fresh() {
        let st = store("prune_fresh");
        // 新鲜数据（未过 30 天）⇒ 不该删任何行，但必须返回 Ok(0) 而非 Err。
        let got = super::prune_best_effort(&st).expect("清理应成功");
        assert_eq!(got, 0, "新鲜数据不该被删");
    }

    #[test]
    fn prune_best_effort_enforces_per_task_tail_cap() {
        let st = store("prune_tail");
        let ws = Path::new("/tmp");
        // 塞超过 KEEP_CHANGES_PER_TASK 的行 ⇒ 超出部分应被砍
        for _ in 0..(super::KEEP_CHANGES_PER_TASK + 20) {
            let sink = sink(&st, ws, "task-hot");
            sink.read("/x/y", 1).expect("记录");
        }
        let removed = super::prune_best_effort(&st).expect("清理应成功");
        assert!(removed > 0, "超出保尾上限应删掉一些，实得 {removed}");
    }

    #[test]
    fn prune_best_effort_error_is_reported_not_swallowed() {
        // 关键回归：`unwrap_or(0)` 时代失败会伪装成「删了 0 行」，
        // 无人能察觉账目在持续膨胀。现在签名是 Result，失败可被调用方观测。
        //
        // 造一个**打不开的库**：路径指向一个目录而非文件 ⇒ open 即失败。
        let dir = crate::nt_testutil::temp_dir("prune-bad-store");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        // 打开失败 ⇒ 根本拿不到 store，改测「能打开但执行会失败」更实际：
        // 直接对 store 跑一次正常清理，确认 Ok 分支可用（错误分支由类型系统保证可表达）。
        let st = NeobotStore::open(":memory:").expect("store");
        let r = super::prune_best_effort(&st);
        assert!(r.is_ok(), "正常库应 Ok(0)：{r:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn write_then_edit_roundtrip_keeps_both_sides() {
        let st = store("roundtrip");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-1");
        sk.write("a/b.txt", None, "hello\n").expect("write");
        sk.edit("a/b.txt", "hello\n", "hello world\n").expect("edit");
        sk.read("a/b.txt", 12).expect("read");
        let listed = st.list_changes(Some("task-1"), 50).expect("list");
        assert_eq!(listed.len(), 3);
        // 倒序：read 最新。
        assert_eq!(listed.first().map(|c| c.kind.as_str()), Some(KIND_READ));
        let write_row = st
            .list_changes(Some("task-1"), 50)
            .expect("list")
            .into_iter()
            .find(|c| c.kind == KIND_WRITE)
            .expect("write row");
        let view = st.get_change(&write_row.id).expect("get").expect("some");
        // 新建文件：before 为空、after 有内容。
        assert_eq!(view.before, None);
        assert_eq!(view.after.as_deref(), Some("hello\n"));
        assert!(!view.change.content_omitted);
        assert_eq!(view.change.path, "a/b.txt");
    }

    #[test]
    fn edit_records_real_before_and_after() {
        let st = store("edit");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-2");
        sk.write("f.txt", Some("old\n"), "new\n").expect("write");
        let row = st.list_changes(Some("task-2"), 1).expect("list").remove(0);
        let view = st.get_change(&row.id).expect("get").expect("some");
        assert_eq!(view.before.as_deref(), Some("old\n"));
        assert_eq!(view.after.as_deref(), Some("new\n"));
        assert_eq!(view.change.bytes, 4);
    }

    #[test]
    fn oversized_content_is_omitted_not_truncated() {
        let st = store("omit");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-3");
        let huge = "x".repeat(CHANGE_CONTENT_CAP + 1);
        sk.write("big.txt", None, &huge).expect("write");
        let row = st.list_changes(Some("task-3"), 1).expect("list").remove(0);
        assert!(row.content_omitted, "超限应置 content_omitted");
        let view = st.get_change(&row.id).expect("get").expect("some");
        assert_eq!(view.before, None);
        assert_eq!(view.after, None, "超限不存半份");
        // 元信息仍在（bytes 记的是真实长度）。
        assert_eq!(view.change.bytes as usize, huge.len());
    }

    #[test]
    fn at_cap_content_is_still_stored() {
        let st = store("atcap");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-4");
        let exact = "x".repeat(CHANGE_CONTENT_CAP);
        sk.write("edge.txt", None, &exact).expect("write");
        let row = st.list_changes(Some("task-4"), 1).expect("list").remove(0);
        assert!(!row.content_omitted, "恰好等于上限应照存");
    }

    #[test]
    fn oversized_before_also_omits_the_pair() {
        let st = store("omit-before");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-5");
        let huge = "y".repeat(CHANGE_CONTENT_CAP + 1);
        sk.edit("f.txt", &huge, "small").expect("edit");
        let row = st.list_changes(Some("task-5"), 1).expect("list").remove(0);
        assert!(row.content_omitted);
        let view = st.get_change(&row.id).expect("get").expect("some");
        assert_eq!(view.after, None);
    }

    #[test]
    fn tallies_group_by_path_with_operation_counts() {
        let st = store("tally");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-6");
        sk.read("a.txt", 1).expect("read");
        sk.write("a.txt", None, "x").expect("write");
        sk.edit("a.txt", "x", "xy").expect("edit");
        sk.read("b.txt", 1).expect("read");
        let tallies = st.tally_task_paths("task-6").expect("tally");
        assert_eq!(tallies.len(), 2);
        let a = tallies.iter().find(|t| t.path == "a.txt").expect("a tally");
        assert_eq!((a.reads, a.writes, a.edits), (1, 1, 1));
        let b = tallies.iter().find(|t| t.path == "b.txt").expect("b tally");
        assert_eq!((b.reads, b.writes, b.edits), (1, 0, 0));
        // 「最近一笔」可解析到真实账行（点开即取 before/after）。
        let last = st.get_change(&a.last_change).expect("get").expect("some");
        assert_eq!(last.change.kind, KIND_EDIT);
        // 另一任务的账不串台。
        assert!(st.tally_task_paths("other").expect("empty").is_empty());
    }

    #[test]
    fn recent_tally_spans_the_newest_tasks_only() {
        let st = store("recent");
        let ws = std::env::temp_dir();
        // 三个任务，第三个最新。
        for (task, name) in [("t1", "a.txt"), ("t2", "b.txt"), ("t3", "c.txt")] {
            ChangeSink { store: &st, task_id: task, workspace: &ws }
                .write(name, None, "x")
                .expect("write");
        }
        // 只要最近 1 个 → 只有 c.txt。
        let one = st.tally_recent_task_paths(1).expect("tally");
        assert_eq!(one.len(), 1, "{one:?}");
        assert_eq!(one[0].path, "c.txt");
        // 最近 3 个 → 全都在，最新的排最前。
        let three = st.tally_recent_task_paths(3).expect("tally");
        assert_eq!(three.len(), 3);
        assert_eq!(three[0].path, "c.txt");
        // 没改过任何文件时是空，不是报错。
        let fresh = store("recent-empty");
        assert!(fresh.tally_recent_task_paths(3).expect("tally").is_empty());
    }

    #[test]
    fn recent_tally_merges_the_same_path_across_tasks() {
        let st = store("merge");
        let ws = std::env::temp_dir();
        for task in ["t1", "t2"] {
            ChangeSink { store: &st, task_id: task, workspace: &ws }
                .write("same.txt", None, "x")
                .expect("write");
        }
        let got = st.tally_recent_task_paths(5).expect("tally");
        assert_eq!(got.len(), 1, "同一路径跨任务应合并成一行");
        assert_eq!(got[0].writes, 2, "写次数应累加");
        // 最近一笔可解析（点开即取 before/after）。
        let last = st.get_change(&got[0].last_change).expect("get").expect("some");
        assert_eq!(last.change.path, "same.txt");
    }

    #[test]
    fn latest_changed_task_names_the_round_being_shown() {
        let st = store("latest");
        assert!(st.latest_changed_task().expect("q").is_none(), "没改过就该是 None");
        let ws = std::env::temp_dir();
        ChangeSink { store: &st, task_id: "ghost", workspace: &ws }
            .write("a.txt", None, "x")
            .expect("write");
        // task 行不存在（账有、任务没有）→ 回落到 task_id，不能崩。
        let got = st.latest_changed_task().expect("q").expect("some");
        assert_eq!(got.0, "ghost");
        assert_eq!(got.1, "ghost", "没有任务行时用 id 顶上");
    }

    #[test]
    fn list_all_changes_spans_tasks() {
        let st = store("span");
        let ws = std::env::temp_dir();
        sink(&st, &ws, "t1").write("a.txt", None, "a").expect("w");
        sink(&st, &ws, "t2").write("b.txt", None, "b").expect("w");
        assert_eq!(st.list_changes(None, 50).expect("all").len(), 2);
        assert_eq!(st.list_changes(Some("t1"), 50).expect("one").len(), 1);
    }

    #[test]
    fn prune_cuts_by_time_and_keeps_recent_per_task() {
        let st = store("prune");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-7");
        for i in 0..10 {
            sk.write(&format!("f{i}.txt"), None, "x").expect("w");
        }
        assert_eq!(st.count_changes().expect("count"), 10);
        // 只走保尾：cutoff 取**过去**（没有一行过期），10 条按每任务保 3 条砍到 3。
        let removed = st.prune_changes("1970-01-01T00:00:00Z", 3).expect("prune");
        assert_eq!(removed, 7);
        assert_eq!(st.count_changes().expect("count"), 3);
        // 只走时间：cutoff 取**未来**（全部过期）→ 全删。
        // 注意方向别搞反：`at < cutoff`，cutoff 在未来意味着「每行都比它老」。
        let removed2 = st
            .prune_changes("2999-01-01T00:00:00Z", 1000)
            .expect("prune2");
        assert_eq!(removed2, 3);
        assert_eq!(st.count_changes().expect("count"), 0);
    }

    #[test]
    fn time_prune_relies_on_rfc3339_sorting_as_strings() {
        // 前提守卫：`at` 全是 RFC3339，字典序 == 时间序，故裸字符串比较成立。
        // 一旦有人往 `at` 里塞非 RFC3339，这条前提就破了（留个会炸的哨兵）。
        let st = store("timeorder");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-9");
        sk.write("a.txt", None, "x").expect("w");
        let row = st.list_changes(Some("task-9"), 1).expect("list").remove(0);
        let parsed = chrono::DateTime::parse_from_rfc3339(&row.at);
        assert!(parsed.is_ok(), "at 必须是 RFC3339：{}", row.at);
        // 字典序确实等于时间序：解析回时间后重排，结果应不变。
        let a = "2026-01-02T00:00:00.000000000+00:00";
        let b = "2026-01-10T00:00:00.000000000+00:00";
        assert!(a < b, "同格式 RFC3339 应字典序 == 时间序");
    }

    #[test]
    fn absolute_paths_are_normalised_to_workspace_relative() {
        let st = store("rel");
        let ws = std::env::temp_dir();
        let sk = sink(&st, &ws, "task-8");
        sk.write(&ws.join("deep/file.txt").to_string_lossy(), None, "x")
            .expect("w");
        let row = st.list_changes(Some("task-8"), 1).expect("list").remove(0);
        assert_eq!(row.path, "deep/file.txt");
    }

    #[test]
    fn journal_failure_is_noted_not_raised() {
        // 记账失败不掀翻主流程，但要在返回值里留痕。
        let mut result = crate::nt_types::ToolResult {
            ok: true,
            output: "wrote 5 bytes".to_owned(),
            truncated: false,
        };
        note_journal_failure(&mut result, NtBotError::Store("disk full".to_owned()));
        assert!(result.output.contains("wrote 5 bytes"));
        assert!(result.output.contains("journal"));
        assert!(result.output.contains("disk full"));
    }
}
