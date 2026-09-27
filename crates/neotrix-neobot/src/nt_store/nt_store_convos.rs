//! `nt_store_convos` — 会话容器 + 成员 + 已读水位（`NeobotStore` 的 impl 片段）。

use super::{Conversation, NeobotStore};
use crate::nt_error::NtBotError;
use rusqlite::{OptionalExtension, params};

impl NeobotStore {
    // ---- conversations（IM 语义：会话是容器，发送即追加） ----

    /// 建会话（kind: dm|group；title 1..=80；成员必须已登记，空成员允许）。
    /// 返回新会话 id。
    pub fn create_conversation(
        &self,
        kind: &str,
        title: &str,
        members: &[String],
    ) -> Result<String, NtBotError> {
        if kind != "dm" && kind != "group" {
            return Err(NtBotError::Invalid(format!("bad conversation kind '{kind}'")));
        }
        let title = title.trim();
        if title.is_empty() {
            return Err(NtBotError::Invalid("conversation title is empty".to_owned()));
        }
        if title.chars().count() > 80 {
            return Err(NtBotError::Invalid("conversation title exceeds 80 chars".to_owned()));
        }
        for member in members {
            if self
                .list_members()?
                .iter()
                .all(|(id, _, _)| id != member)
            {
                return Err(NtBotError::Store(format!("no such member '{member}'")));
            }
        }
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO conversations(id,kind,title,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?4)",
            params![id, kind, title, now],
        )?;
        for member in members {
            self.conn.execute(
                "INSERT OR IGNORE INTO conversation_members(convo_id,member_id) VALUES(?1,?2)",
                params![id, member],
            )?;
        }
        Ok(id)
    }

    /// 会话列表（按最后活跃倒序；含成员、任务数、免打扰、未读数）。
    pub fn list_conversations(&self) -> Result<Vec<Conversation>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id,c.kind,c.title,c.created_at,c.muted,
               (SELECT COUNT(*) FROM tasks t WHERE t.conversation_id=c.id),
               COALESCE((SELECT MAX(t.created_at) FROM tasks t WHERE t.conversation_id=c.id), c.created_at),
               (SELECT COUNT(*) FROM tasks t WHERE t.conversation_id=c.id
                 AND t.created_at > COALESCE((SELECT last_read_at FROM read_marks WHERE convo_id=c.id), c.created_at))
              FROM conversations c ORDER BY 7 DESC, c.created_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, i64>(7)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, kind, title, created_at, muted, task_count, last_active, unread) = row?;
            let members = self.conversation_members(&id)?;
            out.push(Conversation {
                id,
                kind,
                title,
                created_at,
                members,
                task_count,
                last_active,
                muted: muted != 0,
                unread,
            });
        }
        Ok(out)
    }

    /// 免打扰开关（免打扰会话不亮未读）。
    pub fn set_conversation_muted(&self, id: &str, muted: bool) -> Result<(), NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE conversations SET muted=?1, updated_at=?2 WHERE id=?3",
            params![i64::from(muted), now, id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such conversation '{id}'")));
        }
        Ok(())
    }

    /// 标已读（打开会话/看完本轮后调；水位=现在）。
    pub fn mark_conversation_read(&self, id: &str) -> Result<(), NtBotError> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM conversations WHERE id=?1", params![id], |r| {
                r.get(0)
            })?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such conversation '{id}'")));
        }
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO read_marks(convo_id,last_read_at) VALUES(?1,?2)
             ON CONFLICT(convo_id) DO UPDATE SET last_read_at=excluded.last_read_at",
            params![id, now],
        )?;
        Ok(())
    }

    pub fn conversation_members(&self, convo_id: &str) -> Result<Vec<String>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT member_id FROM conversation_members WHERE convo_id=?1 ORDER BY member_id",
        )?;
        let rows = stmt.query_map(params![convo_id], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn rename_conversation(&self, id: &str, title: &str) -> Result<(), NtBotError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(NtBotError::Invalid("conversation title is empty".to_owned()));
        }
        if title.chars().count() > 80 {
            return Err(NtBotError::Invalid("conversation title exceeds 80 chars".to_owned()));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE conversations SET title=?1, updated_at=?2 WHERE id=?3",
            params![title, now, id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such conversation '{id}'")));
        }
        Ok(())
    }

    /// 删会话（级联删其任务 + steps + 成员行；general 也可删，
    /// 删后不再自动复活——回填只在有无归属任务时发生）。
    pub fn delete_conversation(&self, id: &str) -> Result<(), NtBotError> {
        let exists: Option<String> = self
            .conn
            .query_row("SELECT id FROM conversations WHERE id=?1", params![id], |r| {
                r.get(0)
            })
            .optional()?;
        if exists.is_none() {
            return Err(NtBotError::Store(format!("no such conversation '{id}'")));
        }
        self.conn.execute(
            "DELETE FROM steps WHERE task_id IN (SELECT id FROM tasks WHERE conversation_id=?1)",
            params![id],
        )?;
        self.conn.execute("DELETE FROM tasks WHERE conversation_id=?1", params![id])?;
        self.conn.execute("DELETE FROM conversation_members WHERE convo_id=?1", params![id])?;
        self.conn.execute("DELETE FROM conversations WHERE id=?1", params![id])?;
        Ok(())
    }

    /// 加成员（须已登记；幂等）。
    pub fn add_conversation_member(&self, convo_id: &str, member: &str) -> Result<(), NtBotError> {
        if self
            .list_members()?
            .iter()
            .all(|(id, _, _)| id != member)
        {
            return Err(NtBotError::Store(format!("no such member '{member}'")));
        }
        let n = self.conn.execute(
            "INSERT OR IGNORE INTO conversation_members(convo_id,member_id) VALUES(?1,?2)",
            params![convo_id, member],
        )?;
        if n == 0 {
            // 行已存在或会话不存在：区分之
            let exists: Option<String> = self
                .conn
                .query_row(
                    "SELECT id FROM conversations WHERE id=?1",
                    params![convo_id],
                    |r| r.get(0),
                )
                .optional()?;
            if exists.is_none() {
                return Err(NtBotError::Store(format!("no such conversation '{convo_id}'")));
            }
        }
        self.touch_conversation(convo_id)?;
        Ok(())
    }

    /// 减成员（不许删空到 0 人）。
    pub fn remove_conversation_member(&self, convo_id: &str, member: &str) -> Result<(), NtBotError> {
        self.conn.execute(
            "DELETE FROM conversation_members WHERE convo_id=?1 AND member_id=?2",
            params![convo_id, member],
        )?;
        if self.conversation_members(convo_id)?.is_empty() {
            // 回滚这次删除
            self.conn.execute(
                "INSERT INTO conversation_members(convo_id,member_id) VALUES(?1,?2)",
                params![convo_id, member],
            )?;
            return Err(NtBotError::Store("conversation must keep at least 1 member".to_owned()));
        }
        self.touch_conversation(convo_id)?;
        Ok(())
    }

    fn touch_conversation(&self, convo_id: &str) -> Result<(), NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE conversations SET updated_at=?1 WHERE id=?2",
            params![now, convo_id],
        )?;
        Ok(())
    }

    /// 默认个人对话：含我的 DM 按活跃取首个；没有则建“我的私聊”
    /// （单成员 DM，自己跟自己说话，Telegram 纸飞机思想）。
    pub fn ensure_default_dm(&self, me: &str) -> Result<String, NtBotError> {
        let me = me.trim();
        if me.is_empty() {
            return Err(NtBotError::Invalid("member name is empty".to_owned()));
        }
        self.upsert_member(me, "human")?;
        let mut mine: Vec<Conversation> = self
            .list_conversations()?
            .into_iter()
            .filter(|c| c.kind == "dm" && c.members.iter().any(|m| m == me))
            .collect();
        mine.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        if let Some(first) = mine.first() {
            return Ok(first.id.clone());
        }
        self.create_conversation("dm", "我的私聊", &[me.to_owned()])
    }

    /// 取或建两人 DM（成员集合精确 {me, peer} 即复用；标题取 peer 名）。
    pub fn get_or_create_dm(&self, me: &str, peer: &str) -> Result<String, NtBotError> {
        if me.trim().is_empty() || peer.trim().is_empty() {
            return Err(NtBotError::Invalid("dm members must be non-empty".to_owned()));
        }
        if me == peer {
            return Err(NtBotError::Invalid("cannot dm yourself".to_owned()));
        }
        for convo in self.list_conversations()? {
            if convo.kind != "dm" {
                continue;
            }
            let mut members = convo.members.clone();
            members.sort();
            let mut want = vec![me.to_owned(), peer.to_owned()];
            want.sort();
            if members == want {
                return Ok(convo.id);
            }
        }
        self.create_conversation("dm", peer, &[me.to_owned(), peer.to_owned()])
    }

    /// 任务改属会话（会话须存在）。
    pub fn set_task_conversation(&self, task_id: &str, convo_id: &str) -> Result<(), NtBotError> {
        let exists: Option<String> = self
            .conn
            .query_row("SELECT id FROM conversations WHERE id=?1", params![convo_id], |r| {
                r.get(0)
            })
            .optional()?;
        if exists.is_none() {
            return Err(NtBotError::Store(format!("no such conversation '{convo_id}'")));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE tasks SET conversation_id=?1, updated_at=?2 WHERE id=?3",
            params![convo_id, now, task_id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such task '{task_id}'")));
        }
        Ok(())
    }

    // ---- members（单管理员座位：首成员即 owner） ----

    /// 成员登记（kind: human|agent；幂等 upsert）。
    pub fn upsert_member(&self, id: &str, kind: &str) -> Result<(), NtBotError> {
        if id.trim().is_empty() {
            return Err(NtBotError::Invalid("member id is empty".to_owned()));
        }
        if kind != "human" && kind != "agent" {
            return Err(NtBotError::Invalid(format!("bad member kind '{kind}'")));
        }
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO members(id,kind,created_at) VALUES(?1,?2,?3)
             ON CONFLICT(id) DO UPDATE SET kind=excluded.kind",
            params![id.trim(), kind, now],
        )?;
        Ok(())
    }

    /// 删成员：owner 本人不可删（先转交——转交面远期，此处直接拒绝）。
    pub fn remove_member(&self, id: &str) -> Result<(), NtBotError> {
        if let Some(owner) = self.owner_name()? {
            if owner == id {
                return Err(NtBotError::Store(format!(
                    "cannot remove owner '{id}' (transfer ownership first)"
                )));
            }
        }
        let n = self
            .conn
            .execute("DELETE FROM members WHERE id=?1", params![id])?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such member '{id}'")));
        }
        Ok(())
    }

    pub fn list_members(&self) -> Result<Vec<(String, String, String)>, NtBotError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id,kind,created_at FROM members ORDER BY created_at,rowid")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 首成员即 owner（单用户起点：owner 即全员起点）。
    /// `NEOBOT_OWNER` 环境变量优先（不落库，运维直配）。
    pub fn owner_name(&self) -> Result<Option<String>, NtBotError> {
        if let Ok(name) = std::env::var("NEOBOT_OWNER") {
            if !name.trim().is_empty() {
                return Ok(Some(name.trim().to_owned()));
            }
        }
        let owner: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM members ORDER BY created_at,rowid LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(owner)
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;
    use crate::nt_types::{AgentTask, TaskStatus};

    #[test]
    fn members_owner_and_control_takeover() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        assert!(store.owner_name().expect("owner").is_none());
        store.upsert_member("alice", "human").expect("add");
        store.upsert_member("bot1", "agent").expect("add");
        assert!(store.upsert_member("x", "weird").is_err());
        assert_eq!(
            store.owner_name().expect("owner").as_deref(),
            Some("alice")
        );
        // owner 不可删
        assert!(store.remove_member("alice").is_err());
        store.remove_member("bot1").expect("remove");
        assert!(store.remove_member("bot1").is_err());
        // take-the-wheel：具名拒绝 + 交接审计
        store.take_control("alice").expect("take");
        assert_eq!(
            store.control_holder().expect("holder").as_deref(),
            Some("alice")
        );
        let err = store.take_control("bob").expect_err("takeover must fail");
        assert!(err.to_string().contains("alice"), "holder named: {err}");
        assert!(store.release_control("bob").is_err());
        store.release_control("alice").expect("release");
        assert!(store.control_holder().expect("holder").is_none());
        let audits = store.list_audit(10).expect("audits");
        assert_eq!(
            audits
                .iter()
                .filter(|e| e.tool == "control" && e.rule.as_deref() == Some("take-the-wheel"))
                .count(),
            2
        );
        // visibility owner 门：无 owner 配置但有成员时，非 owner 转 private 被拒
        store.upsert_member("alice", "human").expect("re-add");
        store.save_task(&AgentTask {
            id: "v1".to_owned(),
            title: "v".to_owned(),
            status: TaskStatus::Pending,
            created_at: "2026-09-24T00:00:00Z".to_owned(),
            updated_at: "2026-09-24T00:00:00Z".to_owned(),
            claimed_by: None,
            claimed_at: None,
            visibility: crate::nt_types::default_visibility(),
            lease_id: None,
            lease_until: None,
            attempts: 0,
            error: None,
            conversation_id: None,
        }).expect("save");
        assert!(store.set_task_visibility_as("v1", "private", "bob").is_err());
        store
            .set_task_visibility_as("v1", "private", "alice")
            .expect("owner private ok");
    }


    #[test]
    fn conversation_lifecycle_and_backfill() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        // 新库无全体（只在有无归属任务时回填）。
        assert!(store.list_conversations().expect("list").is_empty());
        store.upsert_member("alice", "human").expect("add");
        store.upsert_member("bot1", "agent").expect("add");
        // 建群 + DM 幂等
        let gid = store
            .create_conversation("group", "小队", &["alice".to_owned(), "bot1".to_owned()])
            .expect("create");
        assert!(store.create_conversation("weird", "x", &[]).is_err());
        assert!(store
            .create_conversation("group", "坏成员", &["ghost".to_owned()])
            .is_err());
        let dm1 = store.get_or_create_dm("alice", "bot1").expect("dm");
        let dm2 = store.get_or_create_dm("bot1", "alice").expect("dm again");
        assert_eq!(dm1, dm2);
        assert!(store.get_or_create_dm("alice", "alice").is_err());
        // 任务归属 + 按会话列
        store
            .set_task_conversation("nope", &gid)
            .expect_err("missing task must fail");
        // 改名/成员/删除
        store.rename_conversation(&gid, "小队2").expect("rename");
        assert!(store.rename_conversation(&gid, "  ").is_err());
        store.remove_conversation_member(&gid, "bot1").expect("rm member");
        assert!(store.remove_conversation_member(&gid, "alice").is_err());
        store.add_conversation_member(&gid, "bot1").expect("re-add");
        assert!(store.delete_conversation("nope").is_err());
        store.delete_conversation(&gid).expect("delete");
        assert!(store.list_conversations().expect("list").iter().all(|c| c.id != gid));
        // 全体可删：建一个叫全体的群再删（删后不复活）。
        let g2 = store
            .create_conversation("group", "全体", &[])
            .expect("general-like");
        store.delete_conversation(&g2).expect("delete general-like");
        assert!(store.delete_conversation(&g2).is_err());
        // 默认个人对话：已有含我的 DM 取最近；幂等。
        let d1 = store.ensure_default_dm("alice").expect("default dm");
        let again = store.ensure_default_dm("alice").expect("default dm again");
        assert_eq!(d1, again);
        assert!(store.ensure_default_dm("  ").is_err());
    }


    #[test]
    fn convo_mute_and_read_watermark() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let gid = store.create_conversation("group", "小队", &[]).expect("create");
        let mk = |id: &str, at: &str| {
            store
                .save_task(&AgentTask {
                    id: id.to_owned(),
                    title: "t".to_owned(),
                    status: TaskStatus::Done,
                    created_at: at.to_owned(),
                    updated_at: at.to_owned(),
                    claimed_by: None,
                    claimed_at: None,
                    visibility: crate::nt_types::default_visibility(),
                    lease_id: None,
                    lease_until: None,
                    attempts: 0,
                    error: None,
                    conversation_id: Some(gid.clone()),
                })
                .expect("save task")
        };
        // 会话创建前的老任务不算未读（默认水位=会话创建时间）。
        mk("old", "2000-01-01T00:00:00Z");
        let c = store.list_conversations().expect("list");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].unread, 0);
        assert!(!c[0].muted);
        // 未标已读时水位之后的新任务算未读；标已读后清零；之后的新任务再计。
        let now = || chrono::Utc::now().to_rfc3339();
        mk("t1", &now());
        assert_eq!(store.list_conversations().expect("list")[0].unread, 1);
        store.mark_conversation_read(&gid).expect("mark");
        assert_eq!(store.list_conversations().expect("list")[0].unread, 0);
        mk("t2", &now());
        assert_eq!(store.list_conversations().expect("list")[0].unread, 1);
        store.mark_conversation_read(&gid).expect("mark again");
        assert_eq!(store.list_conversations().expect("list")[0].unread, 0);
        // 免打扰开关 + 不存在的会话报错。
        store.set_conversation_muted(&gid, true).expect("mute");
        assert!(store.list_conversations().expect("list")[0].muted);
        store.set_conversation_muted(&gid, false).expect("unmute");
        assert!(!store.list_conversations().expect("list")[0].muted);
        assert!(store.set_conversation_muted("nope", true).is_err());
        assert!(store.mark_conversation_read("nope").is_err());
    }

}
