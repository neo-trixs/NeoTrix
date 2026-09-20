use std::path::Path;
use std::sync::Arc;
use rusqlite::Connection;
use tokio::sync::Mutex;

/// Shared SQLite connection pool for all domain plugins.
/// Replaces per-call open_db() pattern.
#[derive(Clone)]
pub struct DbPool {
    inner: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open DB: {e}"))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set pragmas: {e}"))?;
        Ok(Self { inner: Arc::new(Mutex::new(conn)) })
    }

    pub async fn execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<usize, String> {
        let conn = self.inner.lock().await;
        conn.execute(sql, params).map_err(|e| format!("SQL error: {e}"))
    }

    pub async fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], f: F) -> Result<T, String>
    where F: FnOnce(&rusqlite::Row) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static
    {
        let conn = self.inner.lock().await;
        conn.query_row(sql, params, f).map_err(|e| format!("SQL query error: {e}"))
    }

    pub async fn query_map<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], f: F) -> Result<Vec<T>, String>
    where F: FnMut(&rusqlite::Row) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static
    {
        let conn = self.inner.lock().await;
        let mut stmt = conn.prepare(sql).map_err(|e| format!("Prepare error: {e}"))?;
        let rows = stmt.query_map(params, f).map_err(|e| format!("Query map error: {e}"))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {e}"))?);
        }
        Ok(results)
    }
}
