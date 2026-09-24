//! kb_session — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_unify;

impl KnowledgeBase {
    pub fn config_get(&self, section: &str, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::config_get(&conn, section, key)
    }

    pub fn config_set(
        &self,
        section: &str,
        key: &str,
        value: &str,
        is_secret: bool,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::config_set(&conn, section, key, value, is_secret)
    }

    pub fn secret_set(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::secret_set(&conn, key, value)
    }

    pub fn secret_get(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::secret_get(&conn, key)
    }

    pub fn session_log_append(
        &self,
        session_id: &str,
        content: &str,
        content_type: &str,
        metadata: Option<&serde_json::Value>,
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::session_log_append(&conn, session_id, content, content_type, metadata)
    }

    pub fn session_log_get(
        &self,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<(i64, String, String, String, Option<String>)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::session_log_get(&conn, session_id, limit, offset)
    }

    pub fn session_log_list(&self) -> Result<Vec<(String, i64, i64)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::session_log_list_sessions(&conn)
    }

    pub fn session_log_delete(&self, session_id: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        conn.execute(
            "DELETE FROM session_logs WHERE session_id=?1",
            rusqlite::params![session_id],
        )
        .map_err(|e| format!("session_log_delete: {}", e))
    }
}
