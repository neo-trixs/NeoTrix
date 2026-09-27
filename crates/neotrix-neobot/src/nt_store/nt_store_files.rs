//! `nt_store_files` — 附件行（本体落盘，行落库）。

use super::{Attachment, NeobotStore};
use crate::nt_error::NtBotError;
use rusqlite::{OptionalExtension, params};

impl NeobotStore {
    // ---- attachments（作曲区文图文件视频；本体落盘，行落库） ----

    pub fn add_attachment(
        &self,
        convo_id: &str,
        kind: &str,
        name: &str,
        path: &str,
        size: i64,
    ) -> Result<String, NtBotError> {
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
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO attachments(id,convo_id,kind,name,path,size,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![id, convo_id, kind, name, path, size.max(0), now],
        )?;
        Ok(id)
    }

    pub fn list_attachments(&self, convo_id: &str) -> Result<Vec<Attachment>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,convo_id,kind,name,path,size,created_at FROM attachments
             WHERE convo_id=?1 ORDER BY created_at,rowid",
        )?;
        let rows = stmt.query_map(params![convo_id], |r| {
            Ok(Attachment {
                id: r.get(0)?,
                convo_id: r.get(1)?,
                kind: r.get(2)?,
                name: r.get(3)?,
                path: r.get(4)?,
                size: r.get(5)?,
                created_at: r.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 删附件行 + 尽力删文件（文件删失败不报错，行已删即达目的）。
    pub fn remove_attachment(&self, id: &str) -> Result<(), NtBotError> {
        let path: Option<String> = self
            .conn
            .query_row("SELECT path FROM attachments WHERE id=?1", params![id], |r| {
                r.get(0)
            })
            .optional()?;
        let Some(path) = path else {
            return Err(NtBotError::Store(format!("no such attachment '{id}'")));
        };
        self.conn.execute("DELETE FROM attachments WHERE id=?1", params![id])?;
        let _removed = std::fs::remove_file(path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;

    #[test]
    fn attachment_crud_and_classify() {
        use crate::nt_store::classify_attachment;
        assert_eq!(classify_attachment("a.PNG"), "image");
        assert_eq!(classify_attachment("b.mp4"), "video");
        assert_eq!(classify_attachment("c.md"), "text");
        assert_eq!(classify_attachment("d.bin"), "file");
        let store = NeobotStore::open(":memory:").expect("open memory db");
        assert!(store
            .add_attachment("nope", "file", "a", "/tmp/a", 1)
            .is_err());
        let gid = store
            .create_conversation("group", "t", &[])
            .expect("convo");
        let id = store
            .add_attachment(&gid, "image", "a.png", "/tmp/a.png", 10)
            .expect("add");
        assert_eq!(store.list_attachments(&gid).expect("list").len(), 1);
        assert!(store.remove_attachment("nope").is_err());
        store.remove_attachment(&id).expect("remove");
        assert!(store.list_attachments(&gid).expect("list").is_empty());
    }

}
