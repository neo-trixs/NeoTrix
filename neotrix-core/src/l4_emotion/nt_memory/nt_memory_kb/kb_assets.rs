//! kb_assets — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_unify;

impl KnowledgeBase {
    pub fn asset_store(
        &self,
        namespace: &str,
        name: &str,
        data: &[u8],
        mime_type: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::asset_store(&conn, namespace, name, data, mime_type, metadata)
    }

    pub fn asset_load(
        &self,
        id: &str,
    ) -> Result<Option<(Vec<u8>, String, String, Option<String>, i64, Option<String>)>, String>
    {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::asset_load(&conn, id)
    }

    pub fn asset_list(
        &self,
        namespace: &str,
    ) -> Result<
        Vec<(
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            Option<String>,
        )>,
        String,
    > {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::asset_list(&conn, namespace)
    }

    pub fn migrate_from_files(&self) -> nt_memory_unify::MigrationReport {
        match self.conn.lock() {
            Ok(conn) => nt_memory_unify::migrate_from_files(&conn),
            Err(e) => {
                let mut report = nt_memory_unify::MigrationReport::default();
                report.errors.push(("lock".into(), format!("Mutex: {}", e)));
                report
            }
        }
    }

    pub fn store_stats(&self) -> Result<std::collections::HashMap<String, usize>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::store_stats(&conn)
    }

    // ── User memory persistence ──
}
