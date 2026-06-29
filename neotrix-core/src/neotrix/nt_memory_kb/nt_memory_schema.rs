use rusqlite::Connection;

pub const SCHEMA_VERSION: i32 = 1;

pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        );

        CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL,
            title TEXT NOT NULL,
            summary TEXT,
            content TEXT,
            url TEXT,
            domain TEXT,
            language TEXT DEFAULT 'en',
            confidence REAL DEFAULT 1.0,
            importance REAL DEFAULT 0.5,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            access_count INTEGER DEFAULT 0,
            metadata TEXT,
            version INTEGER NOT NULL DEFAULT 1,
            superseded_by TEXT
        );

        CREATE TABLE IF NOT EXISTS edges (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            target_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            relation_type TEXT NOT NULL,
            weight REAL DEFAULT 1.0,
            description TEXT,
            created_at INTEGER NOT NULL,
            metadata TEXT,
            version INTEGER NOT NULL DEFAULT 1,
            superseded_by TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
        CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
        CREATE INDEX IF NOT EXISTS idx_edges_type ON edges(relation_type);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_edges_unique ON edges(source_id, target_id, relation_type);

        CREATE TABLE IF NOT EXISTS embeddings (
            node_id TEXT PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
            vector BLOB NOT NULL,
            dimension INTEGER NOT NULL,
            model TEXT DEFAULT 'text-embedding-3-small'
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
            title, summary, content, domain,
            tokenize='porter unicode61'
        );

        CREATE TABLE IF NOT EXISTS crawl_queue (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL UNIQUE,
            depth INTEGER DEFAULT 0,
            domain TEXT,
            priority INTEGER DEFAULT 0,
            status TEXT DEFAULT 'pending',
            discovered_at INTEGER NOT NULL,
            last_attempt INTEGER,
            retry_count INTEGER DEFAULT 0,
            error_message TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_crawl_status ON crawl_queue(status);
        CREATE INDEX IF NOT EXISTS idx_crawl_priority ON crawl_queue(priority, status);

        CREATE TABLE IF NOT EXISTS ingest_log (
            id TEXT PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_url TEXT,
            node_id TEXT REFERENCES nodes(id),
            status TEXT NOT NULL,
            items_count INTEGER DEFAULT 0,
            started_at INTEGER NOT NULL,
            completed_at INTEGER,
            error TEXT
        );",
    )?;

    let version: i32 = conn
        .query_row("SELECT version FROM schema_version", [], |r| r.get(0))
        .unwrap_or(0);

    if version < SCHEMA_VERSION {
        conn.execute(
            "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
            [SCHEMA_VERSION],
        )?;
    }

    Ok(())
}
