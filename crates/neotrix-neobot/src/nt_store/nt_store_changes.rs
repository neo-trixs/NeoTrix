//! `nt_store_changes` — 本轮文件改动账（侧边栏「本轮文件」视角的底账）。
//!
//! 吸收 `dsh-better-sidebar` 的「本轮文件」视角：`steps` 只记
//! `wrote 812 bytes` 这种**结果摘要**，看不出改了哪个文件、改前改后是什么，
//! 渲染不出 diff。本表补上这条链：`task_id` → `path` → `kind` → `before/after`。
//!
//! 体积律：`before`/`after` 各有 `CHANGE_CONTENT_CAP` 上限（`nt_changes` 里定），
//! 超限则**不存内容、只存元信息**并置 `content_omitted`，UI 据此说
//! 「内容过大，请打开文件对比」——**不截断存半份**：半份 diff 比没有 diff 更骗人。
//!
//! 留存律：`prune_changes` 按任务保留最近 N 条 + 按时间切掉老账，
//! 与 `prune_audit` 同律（append-only 但要留得住也放得掉）。

use rusqlite::{params, OptionalExtension, Row};

use crate::nt_error::NtBotError;
use crate::nt_store::{FileChange, FileChangeView, NeobotStore, PathTally};

fn row_to_change(row: &Row<'_>) -> rusqlite::Result<FileChange> {
    Ok(FileChange {
        id: row.get(0)?,
        task_id: row.get(1)?,
        at: row.get(2)?,
        path: row.get(3)?,
        kind: row.get(4)?,
        bytes: row.get(5)?,
        content_omitted: row.get::<_, i64>(6)? != 0,
    })
}

const CHANGE_COLS: &str = "id, task_id, at, path, kind, bytes, content_omitted";

impl NeobotStore {
    /// 记一笔改动。`before`/`after` 传 `None` 即不存内容。
    pub fn record_change(
        &self,
        change: &FileChange,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO file_changes(id,task_id,at,path,kind,bytes,content_omitted,before,after)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                change.id,
                change.task_id,
                change.at,
                change.path,
                change.kind,
                change.bytes,
                i64::from(change.content_omitted),
                before,
                after,
            ],
        )?;
        Ok(())
    }

    /// 列账（`task_id` 为 `None` 即全量；按时间倒序）。
    pub fn list_changes(
        &self,
        task_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<FileChange>, NtBotError> {
        let sql = match task_id {
            Some(_) => format!(
                "SELECT {CHANGE_COLS} FROM file_changes WHERE task_id=?1 ORDER BY at DESC, rowid DESC LIMIT ?2"
            ),
            None => format!(
                "SELECT {CHANGE_COLS} FROM file_changes ORDER BY at DESC, rowid DESC LIMIT ?1"
            ),
        };
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = match task_id {
            Some(task) => stmt
                .query_map(params![task, limit], row_to_change)?
                .collect::<Result<Vec<_>, _>>()?,
            None => stmt
                .query_map(params![limit], row_to_change)?
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(rows)
    }

    /// 取单条账 + 前后内容（渲染 diff 的唯一入口）。
    pub fn get_change(&self, id: &str) -> Result<Option<FileChangeView>, NtBotError> {
        let sql = format!(
            "SELECT {CHANGE_COLS}, before, after FROM file_changes WHERE id=?1"
        );
        let found = self
            .conn
            .query_row(&sql, params![id], |row| {
                Ok(FileChangeView {
                    change: row_to_change(row)?,
                    // `CHANGE_COLS` 占 0..=6，故 before=7 / after=8。
                    before: row.get(7)?,
                    after: row.get(8)?,
                })
            })
            .optional()?;
        Ok(found)
    }

    /// 一任务内按文件分组（读/写/改计数 + 最近一笔）。
    ///
    /// 「最近一笔」用 `MAX(rowid)` 而不是 `MAX(at)`：同秒内多笔时 `at` 撞车，
    /// `rowid` 单调且精确，取出来的才是真·最后一次改动。
    pub fn tally_task_paths(&self, task_id: &str) -> Result<Vec<PathTally>, NtBotError> {
        let sql = "SELECT path,
                          SUM(CASE WHEN kind='read'  THEN 1 ELSE 0 END),
                          SUM(CASE WHEN kind='write' THEN 1 ELSE 0 END),
                          SUM(CASE WHEN kind='edit'  THEN 1 ELSE 0 END),
                          MAX(bytes),
                          MAX(at),
                          MAX(rowid)
                   FROM file_changes WHERE task_id=?1
                   GROUP BY path ORDER BY MAX(rowid) DESC";
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![task_id], |row| {
            let path: String = row.get(0)?;
            let last_rowid: i64 = row.get(6)?;
            Ok((
                path,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                last_rowid,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (path, reads, writes, edits, bytes, last_at, last_rowid) = row?;
            let last_change: String = self.conn.query_row(
                "SELECT id FROM file_changes WHERE rowid=?1",
                params![last_rowid],
                |r| r.get(0),
            )?;
            out.push(PathTally {
                path,
                reads,
                writes,
                edits,
                bytes,
                last_at,
                last_change,
            });
        }
        Ok(out)
    }

    /// 按文件分组，跨**最近若干任务**。
    ///
    /// 为什么不复用 `tally_task_paths`：那个要求调用方**先知道** task_id，
    /// 而侧边栏的「本轮文件」页没有「当前任务」这个概念（任务列表是另一个页签），
    /// 硬传空串会让整页恒为空 —— 看起来能用、实际永远没有数据。
    /// 故这里从「最近 N 个任务」反推，前端只要给 N。
    pub fn tally_recent_task_paths(
        &self,
        task_limit: i64,
    ) -> Result<Vec<PathTally>, NtBotError> {
        let task_limit = task_limit.clamp(1, 50);
        // 最近 N 个任务（有改动的优先在前 —— 改了东西的任务才是用户想看的）。
        let mut stmt = self.conn.prepare(
            "SELECT task_id FROM file_changes GROUP BY task_id
             ORDER BY MAX(rowid) DESC LIMIT ?1",
        )?;
        let task_ids: Vec<String> = stmt
            .query_map(params![task_limit], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }
        // 只留**有内容**的那一笔（overridden 那一列），并按路径分组。
        let placeholders = std::iter::repeat_n("?", task_ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT c.path,
                    SUM(CASE WHEN c.kind='read'  THEN 1 ELSE 0 END),
                    SUM(CASE WHEN c.kind='write' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN c.kind='edit'  THEN 1 ELSE 0 END),
                    MAX(c.bytes), MAX(c.at), MAX(c.rowid)
             FROM file_changes c
             WHERE c.task_id IN ({placeholders})
             GROUP BY c.path ORDER BY MAX(c.rowid) DESC"
        );
        let mut stmt2 = self.conn.prepare(&sql)?;
        let bound: Vec<&dyn rusqlite::ToSql> =
            task_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let rows = stmt2.query_map(bound.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })?;
        let mut out: Vec<PathTally> = Vec::new();
        for row in rows {
            let (path, reads, writes, edits, bytes, last_at, last_rowid) = row?;
            let last_change: String = self.conn.query_row(
                "SELECT id FROM file_changes WHERE rowid=?1",
                params![last_rowid],
                |r| r.get(0),
            )?;
            out.push(PathTally {
                path,
                reads,
                writes,
                edits,
                bytes,
                last_at,
                last_change,
            });
        }
        Ok(out)
    }

    /// 最近有改动的那个任务（`None` = 从没改过文件）。
    ///
    /// 「本轮文件」页的标题要写清是**哪一轮**，不然用户不知道在看什么。
    pub fn latest_changed_task(&self) -> Result<Option<(String, String)>, NtBotError> {
        Ok(self
            .conn
            .query_row(
                "SELECT c.task_id, COALESCE(t.title, c.task_id)
                 FROM file_changes c LEFT JOIN tasks t ON t.id = c.task_id
                 GROUP BY c.task_id ORDER BY MAX(c.rowid) DESC LIMIT 1",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .ok())
    }

    /// 清账：`at < cutoff` 的老账删掉；每任务只留最近 `keep_per_task` 条。
    ///
    /// 先按时间切、再按任务保尾，两步都幂等。返回删掉的行数。
    pub fn prune_changes(&self, cutoff: &str, keep_per_task: i64) -> Result<usize, NtBotError> {
        let mut removed = 0usize;
        if let Ok(n) = self
            .conn
            .execute("DELETE FROM file_changes WHERE at < ?1", params![cutoff])
        {
            removed = removed.saturating_add(n);
        }
        // 每任务保尾：`rowid` 不在「最近 keep 条」里的即删。
        //
        // `rid` 这个别名不是装饰：SQLite 里派生表没有真 `rowid` 列，
        // 直接写 `SELECT rowid FROM (SELECT rowid, ...)` 会让外层那个
        // `rowid` 绑定到**外层**的 `file_changes`，条件退化成
        // `rowid NOT IN (全体 rowid)` = 恒假 = 一行都删不掉。给内层起名
        // `rid` 才能让外层绑到派生表的真列上。
        let sql = "DELETE FROM file_changes WHERE rowid NOT IN (
                     SELECT rid FROM (
                       SELECT rowid AS rid, ROW_NUMBER() OVER (
                         PARTITION BY task_id ORDER BY rowid DESC) AS rn
                       FROM file_changes
                     ) WHERE rn <= ?1
                   )";
        if let Ok(n) = self.conn.execute(sql, params![keep_per_task]) {
            removed = removed.saturating_add(n);
        }
        Ok(removed)
    }

    /// 账目计数（导出 / doctor 用）。
    pub fn count_changes(&self) -> Result<i64, NtBotError> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM file_changes", [], |r| r.get(0))?)
    }
}
