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
        self.touch_conversation(convo_id)?;
        Ok(ChatMessage {
            id,
            convo_id: convo_id.to_owned(),
            role: role.to_owned(),
            text: text.to_owned(),
            created_at: now,
        })
    }

    /// 读一个会话的全部消息（时间正序）。会话不存在即报错 ——
    /// 「空会话」与「不存在的会话」对调用方是两种处境（见模块头 ①）。
    pub fn list_messages(&self, convo_id: &str) -> Result<Vec<ChatMessage>, NtBotError> {
        if self.get_conversation(convo_id)?.is_none() {
            return Err(NtBotError::Store(format!("no such conversation '{convo_id}'")));
        }
        let mut stmt = self.conn.prepare(
            "SELECT id,convo_id,role,text,created_at FROM messages
             WHERE convo_id=?1 ORDER BY created_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map(params![convo_id], |r| {
            Ok(ChatMessage {
                id: r.get(0)?,
                convo_id: r.get(1)?,
                role: r.get(2)?,
                text: r.get(3)?,
                created_at: r.get(4)?,
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
