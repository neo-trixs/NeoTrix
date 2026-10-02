//! 会话消息 —— 聊天轮的持久化（`messages` 表）。
//!
//! # 为什么单独一张表，不蹭 tasks
//!
//! `tasks` 是任务（有认领/租约/可见性语义），`task_count` 直接画在
//! 会话列表上（`ConvoView.task_count`）。把聊天消息塞进 tasks 会让
//! 「发了 3 句话」显示成「3 个任务」—— 语义错了，界面跟着错。
//! 消息只有四列：谁（role）、说什么（text）、何时（created_at）、
//! 在哪个会话（convo_id）。
//!
//! # 校验（三条，全部在入库前）
//!
//! ① 会话必须存在 —— 往不存在的会话写消息，读侧永远读不到，
//!    等于把消息扔进黑洞，还不报错。
//! ② role 只认 `user` | `assistant` —— 系统提示词不归这里
//!    （那是 `nt_memory` 注入的），多一种 role，前端渲染就要多一个分支。
//! ③ 文本去空 + 限长 100_000 字符 —— 空消息不是消息；
//!    超长拒绝不截断：截断会把「发出去的话」和「存下的话」变成两句话。
//!
//! # last_active 联动
//!
//! 列表的 `last_active` 取 tasks / messages / 创建时间三者之 max
//! （见 `list_conversations_where`）。只写消息不碰任务时，
//! 会话必须照样浮到列表顶部 —— 否则「刚说完话，会话沉底」。

use rusqlite::params;

use crate::nt_error::NtBotError;
use crate::nt_store::NeobotStore;

/// 单条聊天消息。
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub convo_id: String,
    pub role: String,
    pub text: String,
    pub created_at: String,
    /// ⭐ 会话内单调序号（SQLite `rowid`）。**游标分页的游标就是它。**
    ///
    /// ⛔⛔ 为什么不用 `created_at` 当游标：`created_at` 由 `chrono::to_rfc3339()`
    ///    写入，**宽度不定**（亚秒为 0 时不带小数部分）⇒ **字符串比较不是全序**
    ///    ⇒ `WHERE created_at < ?` 在**同秒并列**时不可靠，**且不报错，只是漏消息或重复**。
    ///    全 crate 零定宽时间戳写入（已实测：`to_rfc3339_opts`/`%Y-%m-%dT%H:%M:%S` 零命中）。
    ///
    /// ⭐ 为什么不用「新增 `seq` 列 + `MAX(seq)+1`」：
    ///    ① `MAX()+1` 是「先读后写」两步，而**本 crate 零事务** + 双进程共库
    ///       （`neobot_send` 对同一会话分两次写、每次新开连接）⇒ 有窗口，失败形态是**静默错号**；
    ///    ② `ALTER … DEFAULT 0` 会让**全部历史行 seq=0** ⇒ 存量数据上分页第一页就捞光、之后再捞不到。
    ///    ⭐ `rowid` 已存在、已是整型、单调，且**代码里对 `messages` 零 DELETE/UPDATE**
    ///    （`rg 'DELETE FROM messages|UPDATE messages'` 零命中）⇒ 无重号来源。
    pub seq: i64,
}

/// 单条消息文本上限（字符数）。见模块头 ③。
pub const MESSAGE_MAX_CHARS: usize = 100_000;

impl NeobotStore {
    /// 追记一条消息（校验见模块头）。返回落库行。
    pub fn append_message(
        &self,
        convo_id: &str,
        role: &str,
        text: &str,
    ) -> Result<ChatMessage, NtBotError> {
        if self.get_conversation(convo_id)?.is_none() {
            return Err(NtBotError::Store(format!("no such conversation '{convo_id}'")));
        }
        if role != "user" && role != "assistant" {
            return Err(NtBotError::Invalid(format!("bad message role '{role}'")));
        }
        let text = text.trim();
        if text.is_empty() {
            return Err(NtBotError::Invalid("message text is empty".to_owned()));
        }
        if text.chars().count() > MESSAGE_MAX_CHARS {
            return Err(NtBotError::Invalid(format!(
                "message text exceeds {MESSAGE_MAX_CHARS} chars"
            )));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO messages(id,convo_id,role,text,created_at) VALUES(?1,?2,?3,?4,?5)",
            params![id, convo_id, role, text, now],
        )?;
        // ⭐ 取刚落库这行的 `rowid` 当 `seq` 游标。
        //    `last_insert_rowid` 读的是**本连接**最后一次 INSERT 的 rowid
        //    ⇒ 不需要 `MAX(seq)+1` 那种「先读后写」的两步，**无竞态窗口**。
        // ⛔ 不可用 `SELECT rowid WHERE id=?` 回查：多一次查询，
        //    且并发写下那条查询可能已被别的 INSERT 影响语义。
        let seq = self.conn.last_insert_rowid();
        self.touch_conversation(convo_id)?;
        Ok(ChatMessage {
            id,
            convo_id: convo_id.to_owned(),
            role: role.to_owned(),
            text: text.to_owned(),
            created_at: now,
            seq,
        })
    }

    /// 读一个会话的**全部**消息（时间正序）。会话不存在即报错 ——
    /// 「空会话」与「不存在的会话」对调用方是两种处境（见模块头 ①）。
    ///
    /// ⭐ **薄封装**：转调 `list_messages_page` 且不设游标、不限条数。
    ///    ⛔ **刻意不改签名** —— 本函数是唯一的生产读侧入口（IPC `neobot_convo_messages`）
    ///    且被 3 处测试断言依赖；Rust 无默认参数，若加 `limit` 参数会让**全部调用点编译失败**。
    ///    ⭐ 分页能力走新增的 `list_messages_page`，本函数行为**逐字不变**。
    pub fn list_messages(&self, convo_id: &str) -> Result<Vec<ChatMessage>, NtBotError> {
        self.list_messages_page(convo_id, None, i64::MAX)
    }

    /// ⭐ 读一个会话的消息**一页**（按 `seq` 正序，可游标续读）。
    ///
    /// * `after_seq` —— **排他**下界：只返回 `seq > after_seq` 的行；`None` 从头读。
    /// * `limit` —— 最多返回几条；`i64::MAX` 表示不限。
    ///
    /// ⛔⛔ **`after_seq` 是 `>` 不是 `<`。** 本函数按 `seq` **正序**返回
    ///    （最旧在前，符合消息列表的阅读方向），调用方一路向**更新的**消息翻页。
    ///    ⭐ 曾误写成 `rowid < ?`（把它当「before」）—— 那是从新往旧回翻，
    ///    与正向翻页方向相反 ⇒ 实测**漏消息**：7 行/每页 3 条只拼出 6 条。
    ///    ⓰ 测试 `游标分页小步长不漏不重` 就是为钉住这个方向而写的。
    ///    ⭐ 游标取**上一页最后一条的 `seq`**（正序 ⇒ 最小的那条），配合 `>` 即排他、不重复。
    ///
    /// ⭐ 用 `seq`（`rowid`）而非 `created_at` 做游标，理由见 `ChatMessage::seq` 的文档。
    ///
    /// ⛔ **代价（诚实记录）：本查询仍走不了索引 ⇒ SQLite 必须额外排序。**
    ///    ⓰ 原因：`rowid` 是**隐式**列，`CREATE INDEX` 看不见它 ⇒ `(convo_id, rowid)`
    ///    这个索引**建不出来**（实测 `no such column: rowid`）。
    ///    ⓰ 现有 `idx_messages_convo(convo_id, created_at)` 也用不上
    ///    （索引不含 `rowid`，而排序第二键是 `rowid`）。
    ///    ⭐ 也就是说：**排序开销在改动前后一样，本次拿到的是「正确性」不是「加速」。**
    ///    ⛔ 要索引覆盖只能新增**真实列** `seq INTEGER`（改 DDL + 回填 + 写入侧赋值），
    ///    ⭐ 那是独立的、更大的一步，本轮**刻意不做**（见 TODO）。
    pub fn list_messages_page(
        &self,
        convo_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<Vec<ChatMessage>, NtBotError> {
        if self.get_conversation(convo_id)?.is_none() {
            return Err(NtBotError::Store(format!("no such conversation '{convo_id}'")));
        }
        // ⛔ `limit` 夹到 [0, i64::MAX]：负数会让 SQLite 的 LIMIT 语义变成「无限制」
        //    （`LIMIT -1` = 不限），那会让调用方的「我只要 20 条」悄悄变成「全给我」。
        let limit = limit.clamp(0, i64::MAX);
        let mut stmt = self.conn.prepare(
            "SELECT id,convo_id,role,text,created_at,rowid FROM messages
             WHERE convo_id=?1 AND (?2 IS NULL OR rowid > ?2)
             ORDER BY rowid ASC LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![convo_id, after_seq, limit], |r| {
            Ok(ChatMessage {
                id: r.get(0)?,
                convo_id: r.get(1)?,
                role: r.get(2)?,
                text: r.get(3)?,
                created_at: r.get(4)?,
                seq: r.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> NeobotStore {
        NeobotStore::open(":memory:").expect("内存库")
    }

    fn convo_with_member(store: &NeobotStore) -> String {
        store.upsert_member("neo", "human").expect("成员");
        store.create_conversation("group", "t", &["neo".into()]).expect("会话")
    }

    /// ⭐ 确定性造出 `created_at` **并列**的一批行（不走 `append_message`，
    ///    因为它用 `Utc::now()`，能不能撞上同一秒取决于运气 ⇒ 测不稳定）。
    ///    然后用**同一个** `created_at` 灌 7 行。
    fn seed_tied_rows(store: &NeobotStore, convo_id: &str, tied_at: &str, n: u32) -> Vec<i64> {
        let mut seqs = Vec::new();
        for i in 0..n {
            store
                .conn
                .execute(
                    "INSERT INTO messages(id,convo_id,role,text,created_at)
                     VALUES(?1,?2,'user',?3,?4)",
                    params![format!("tied-{i}"), convo_id, format!("m{i}"), tied_at],
                )
                .expect("插入并列行");
            seqs.push(store.conn.last_insert_rowid());
        }
        seqs
    }

    /// ⭐⭐ 本次改动的核心断言：`created_at` 并列时，**旧游标口径不可靠、`seq` 可靠**。
    #[test]
    fn created_at并列时seq仍给出唯一全序() {
        let s = open();
        let c = convo_with_member(&s);
        let tied = "2026-01-01T00:00:00Z";
        let seqs = seed_tied_rows(&s, &c, tied, 7);

        let all = s.list_messages(&c).expect("读全部");
        // ⛔ 先证前提：这批行的 `created_at` **真的**全一样
        //    （否则本测试等于没测到并列这个场景）。
        assert!(
            all.iter().all(|m| m.created_at == tied),
            "前提不成立：造出的行 created_at 不全相同"
        );
        // ⭐ 而 `seq` **严格递增且互不相同** ⇒ 它是这批并列行的唯一全序。
        let got: Vec<i64> = all.iter().map(|m| m.seq).collect();
        assert_eq!(got, seqs, "seq 必须与落库顺序一致");
        let mut uniq = got.clone();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(uniq.len(), 7, "seq 必须互不相同，否则游标会歧义");
        // ⭐⭐ 口径对比：以 `created_at` 为游标时，**全部并列 ⇒ 无法分页**；
        //    以 `seq` 为游标时可以。
        let by_time: Vec<&str> = all.iter().map(|m| m.created_at.as_str()).collect();
        assert!(
            by_time.windows(2).all(|w| w[0] == w[1]),
            "预期 created_at 全并列；若已不并列说明 to_rfc3339 行为变了，本测试需重估"
        );
    }

    /// ⭐⭐ **游标分页不漏不重**：小 limit 逐页取完，拼接结果与全量逐字相等。
    #[test]
    fn 游标分页小步长不漏不重() {
        let s = open();
        let c = convo_with_member(&s);
        seed_tied_rows(&s, &c, "2026-01-01T00:00:00Z", 7);

        let full = s.list_messages(&c).expect("全量");
        // ⭐ 逐页取，每页 3 条，游标取「上一页最后一条的 seq」。
        let mut paged: Vec<ChatMessage> = Vec::new();
        let mut cursor: Option<i64> = None;
        loop {
            let page = s.list_messages_page(&c, cursor, 3).expect("取页");
            if page.is_empty() {
                break;
            }
            paged.extend(page.iter().cloned());
            cursor = page.last().map(|m| m.seq);
        }
        assert_eq!(paged.len(), full.len(), "分页拼接条数必须与全量相等（不漏）");
        let pids: Vec<&str> = paged.iter().map(|m| m.id.as_str()).collect();
        let fids: Vec<&str> = full.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(pids, fids, "分页拼接顺序与内容必须与全量逐字相等（不漏不重不乱序）");
        // ⛔ 显式证「不重复」：去重后长度不变。
        let mut d = pids.clone();
        d.sort_unstable();
        d.dedup();
        assert_eq!(d.len(), 7, "分页结果出现重复行");
    }

    /// 游标是**排他下界**（`>`），所以下一页不会重复上一页最后一条。
    #[test]
    fn 游标为排他下界不重复边界行() {
        let s = open();
        let c = convo_with_member(&s);
        let seqs = seed_tied_rows(&s, &c, "2026-01-01T00:00:00Z", 4);
        let first = s.list_messages_page(&c, None, 2).expect("第一页");
        assert_eq!(first.len(), 2);
        let second = s
            .list_messages_page(&c, first.last().map(|m| m.seq), 10)
            .expect("第二页");
        assert_eq!(second.len(), 2, "4 行 / 每页 2 ⇒ 第二页应剩 2 行（漏一条即为方向写反）");
        // ⛔ 边界行 `seqs[1]` 只应出现在第一页。
        assert!(
            first.iter().any(|m| m.seq == seqs[1]) && !second.iter().any(|m| m.seq == seqs[1]),
            "游标必须是排他下界，否则边界行会重复"
        );
    }

    /// ⭐ `limit` 夹取：负数在 SQLite 里等于「**无限制**」⇒ 必须被夹住，
    /// 否则「我要 20 条」会悄悄变成「全给我」。
    #[test]
    fn limit负数被夹住不退化为无限() {
        let s = open();
        let c = convo_with_member(&s);
        seed_tied_rows(&s, &c, "2026-01-01T00:00:00Z", 7);
        assert_eq!(
            s.list_messages_page(&c, None, -1).expect("负数 limit").len(),
            0,
            "负数 limit 必须夹到 0（SQLite 的 LIMIT -1 是『无限制』，不能放行）"
        );
        assert_eq!(s.list_messages_page(&c, None, 3).expect("正常 limit").len(), 3);
        // ⭐ 薄封装行为不变：仍然返回**全部**。
        assert_eq!(s.list_messages(&c).expect("全量").len(), 7);
    }

    #[test]
    fn 写入后能按序读回() {
        let s = open();
        let c = convo_with_member(&s);
        s.append_message(&c, "user", "你好").expect("写");
        s.append_message(&c, "assistant", "在").expect("写");
        let v = s.list_messages(&c).expect("读");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].role, "user");
        assert_eq!(v[1].role, "assistant");
        assert_eq!(v[0].text, "你好");
    }

    #[test]
    fn 不存在的会话读写都拒() {
        let s = open();
        assert!(s.append_message("ghost", "user", "x").is_err());
        assert!(s.list_messages("ghost").is_err());
    }

    #[test]
    fn 非法role与空文本被拒() {
        let s = open();
        let c = convo_with_member(&s);
        assert!(s.append_message(&c, "system", "x").is_err());
        assert!(s.append_message(&c, "user", "   ").is_err());
    }

    #[test]
    fn 写消息后会话浮到顶部() {
        let s = open();
        let c1 = convo_with_member(&s);
        let c2 = convo_with_member(&s);
        // c2 后建，默认在上；给 c1 写句话，c1 必须反超。
        s.append_message(&c1, "user", "hi").expect("写");
        let list = s.list_conversations().expect("列表");
        assert_eq!(list[0].id, c1, "刚说话的会话必须在顶部");
        assert_eq!(list[1].id, c2);
    }
}
