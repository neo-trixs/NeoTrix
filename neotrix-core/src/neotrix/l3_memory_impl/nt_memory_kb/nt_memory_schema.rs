use rusqlite::Connection;

pub const SCHEMA_VERSION: i32 = 7;

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
            data_tier TEXT NOT NULL DEFAULT 'core',
            temporal TEXT,
            supersedes TEXT,
            source_episode TEXT,
            tier TEXT NOT NULL DEFAULT 'warm'
        );

        CREATE TABLE IF NOT EXISTS edges (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            target_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            relation_type TEXT NOT NULL,
            weight REAL DEFAULT 1.0,
            description TEXT,
            created_at INTEGER NOT NULL,
            metadata TEXT
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

        CREATE TABLE IF NOT EXISTS pq_codebook (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            m INTEGER NOT NULL,
            ks INTEGER NOT NULL,
            sub_dim INTEGER NOT NULL,
            codewords BLOB NOT NULL,
            dimension INTEGER NOT NULL,
            model TEXT,
            trained_at INTEGER,
            num_vectors INTEGER
        );

        CREATE TABLE IF NOT EXISTS embeddings_pq (
            node_id TEXT PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
            pq_codes BLOB NOT NULL,
            codebook_id INTEGER REFERENCES pq_codebook(id)
        );

        CREATE INDEX IF NOT EXISTS idx_emb_pq_codebook ON embeddings_pq(codebook_id);

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

        CREATE TABLE IF NOT EXISTS novel_queue (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT,
            genre TEXT,
            sub_genre TEXT,
            book_url TEXT NOT NULL UNIQUE,
            rank INTEGER DEFAULT 0,
            synopsis TEXT,
            tags TEXT,
            word_count TEXT,
            status TEXT DEFAULT 'pending',
            chapter_count INTEGER DEFAULT 0,
            ranking_name TEXT,
            discovered_at INTEGER NOT NULL,
            last_attempt INTEGER,
            retry_count INTEGER DEFAULT 0,
            error_message TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_novel_status ON novel_queue(status);

        CREATE TABLE IF NOT EXISTS procedural_memory (
            id TEXT PRIMARY KEY,
            skill_id TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            e8_sequence TEXT NOT NULL,
            trigger_pattern TEXT NOT NULL,
            success_rate REAL DEFAULT 0.0,
            execution_count INTEGER DEFAULT 0,
            avg_reward REAL DEFAULT 0.0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            tags TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_procedural_memory_skill_id ON procedural_memory(skill_id);
        CREATE INDEX IF NOT EXISTS idx_procedural_memory_success_rate ON procedural_memory(success_rate DESC);

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
        );

        CREATE TABLE IF NOT EXISTS conversation_records (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            task_description TEXT,
            user_intent TEXT,
            strategy_used TEXT,
            e8_mode TEXT,
            specialist_winner TEXT,
            actions_taken TEXT,
            obstacles_encountered TEXT,
            fix_patterns TEXT,
            outcome TEXT,
            effectiveness REAL,
            reasoning_iterations INTEGER DEFAULT 0,
            error_count INTEGER DEFAULT 0,
            timestamp INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_conversation_timestamp ON conversation_records(timestamp DESC);

        CREATE TABLE IF NOT EXISTS evolution_records (
            id TEXT PRIMARY KEY,
            source_conversation_id TEXT,
            pattern_type TEXT NOT NULL,
            description TEXT,
            before_behavior TEXT,
            after_behavior TEXT,
            effectiveness_gain REAL DEFAULT 0.0,
            applied_to TEXT,
            verified INTEGER DEFAULT 0,
            timestamp INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_evolution_timestamp ON evolution_records(timestamp DESC);

        CREATE TABLE IF NOT EXISTS trace_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            data_json TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_trace_created ON trace_data(created_at DESC);

        CREATE TABLE IF NOT EXISTS learning_reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_json TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_learning_created ON learning_reports(created_at DESC);

        CREATE TABLE IF NOT EXISTS discovery_sources (
            source_name TEXT PRIMARY KEY,
            last_run_at INTEGER,
            total_items INTEGER DEFAULT 0,
            status TEXT DEFAULT 'pending',
            error_message TEXT
        );

        CREATE TABLE IF NOT EXISTS kv_store (
            namespace TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (namespace, key)
        );

        CREATE TABLE IF NOT EXISTS config_entries (
            section TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            is_secret INTEGER DEFAULT 0,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (section, key)
        );

        CREATE TABLE IF NOT EXISTS secrets (
            key TEXT PRIMARY KEY,
            encrypted_value BLOB NOT NULL,
            nonce BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS session_logs (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            sequence INTEGER NOT NULL,
            content TEXT NOT NULL,
            content_type TEXT DEFAULT 'markdown',
            created_at INTEGER NOT NULL,
            metadata TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_session_logs_session ON session_logs(session_id);

        CREATE TABLE IF NOT EXISTS cookies (
            domain TEXT NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            path TEXT DEFAULT '/',
            secure INTEGER DEFAULT 0,
            http_only INTEGER DEFAULT 0,
            expiry INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (domain, name, path)
        );

        CREATE TABLE IF NOT EXISTS binary_assets (
            id TEXT PRIMARY KEY,
            namespace TEXT NOT NULL,
            name TEXT NOT NULL,
            data BLOB NOT NULL,
            mime_type TEXT,
            size INTEGER NOT NULL,
            checksum TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            metadata TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_binary_assets_ns ON binary_assets(namespace);

        CREATE TABLE IF NOT EXISTS skills_index (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            source_path TEXT,
            tags TEXT,
            is_builtin INTEGER DEFAULT 0,
            last_indexed_at INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS rkyv_blobs (
            id TEXT PRIMARY KEY,
            namespace TEXT NOT NULL,
            data BLOB NOT NULL,
            checksum TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_rkyv_blobs_ns ON rkyv_blobs(namespace);

        CREATE TABLE IF NOT EXISTS agent_sessions (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            label TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            ended_at INTEGER,
            metadata TEXT NOT NULL DEFAULT '{}'
        );
        CREATE TABLE IF NOT EXISTS agent_memory_entries (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            tier TEXT NOT NULL DEFAULT 'core',
            content TEXT NOT NULL,
            embedding BLOB,
            metadata TEXT NOT NULL DEFAULT '{}',
            created_at INTEGER NOT NULL,
            access_count INTEGER NOT NULL DEFAULT 1,
            superseded INTEGER NOT NULL DEFAULT 0,
            superseded_by TEXT,
            FOREIGN KEY (session_id) REFERENCES agent_sessions(id)
        );
        CREATE INDEX IF NOT EXISTS idx_ame_agent ON agent_memory_entries(agent_id);
        CREATE INDEX IF NOT EXISTS idx_ame_session ON agent_memory_entries(session_id);
        CREATE INDEX IF NOT EXISTS idx_as_agent ON agent_sessions(agent_id);",
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
