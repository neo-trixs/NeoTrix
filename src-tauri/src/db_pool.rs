use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use anyhow::{Context, Result as AnyhowResult};
use rusqlite::Connection;

/// Shared SQLite connection pool for all domain plugins.
/// Replaces per-call open_db() pattern.
///
/// Uses `std::sync::Mutex` so `get()` is synchronous — suitable for
/// plugins whose helper methods are not async.
#[derive(Clone)]
pub struct DbPool {
    inner: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(db_path: &Path) -> AnyhowResult<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open DB")?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; PRAGMA foreign_keys=ON;")
            .context("Failed to set pragmas")?;
        Ok(Self { inner: Arc::new(Mutex::new(conn)) })
    }

    /// Run a schema-creation SQL block exactly once.
    /// Call this in the plugin constructor after `DbPool::new()`.
    pub fn init_schema(&self, sql: &str) -> AnyhowResult<()> {
        let conn = self.inner.lock().context("Lock poisoned")?;
        conn.execute_batch(sql)
            .context("Schema init failed")
    }

    /// Synchronous access to the underlying `Connection`.
    /// Returns a `MutexGuard` that derefs to `&Connection`.
    pub fn get(&self) -> AnyhowResult<MutexGuard<'_, Connection>> {
        self.inner.lock().context("Lock poisoned")
    }

    // ── convenience methods ──

    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> AnyhowResult<usize> {
        let conn = self.inner.lock().context("Lock poisoned")?;
        conn.execute(sql, params).context("SQL error")
    }

    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], f: F) -> AnyhowResult<T>
    where F: FnOnce(&rusqlite::Row) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static
    {
        let conn = self.inner.lock().context("Lock poisoned")?;
        conn.query_row(sql, params, f).context("SQL query error")
    }

    pub fn query_map<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], f: F) -> AnyhowResult<Vec<T>>
    where F: FnMut(&rusqlite::Row) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static
    {
        let conn = self.inner.lock().context("Lock poisoned")?;
        let mut stmt = conn.prepare(sql).context("Prepare error")?;
        let rows = stmt.query_map(params, f).context("Query map error")?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.context("Row error")?);
        }
        Ok(results)
    }
}
