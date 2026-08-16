//! KB 存储原语 — NT-MEMORY 持久化原语下沉至 core (D3 架构倒置)。
//!
//! 背景: e8_predictor / consciousness_core 等 core 层模块直接调用
//! `l3_memory_impl::nt_memory_kb::*` 的 kv/schema/count 自由函数, 构成
//! core → l3 反向依赖。这些函数是**纯 SQL 原语** (仅依赖 rusqlite 连接,
//! 无 NT-MEMORY 领域状态), 可安全下沉至 core 作为单一事实源; NT-MEMORY
//! 侧 `nt_memory_unify` / `nt_memory_schema` / `nt_memory_store` re-export,
//! 调用方路径全部保持不变。
//!
//! 语义保持与 l3 原实现逐字一致 (VALUE_COMPRESSED_MAGIC / busy 语义 /
//! schema DDL)。任何 schema 演进必须在 core 单点修改。

use rusqlite::Connection;

pub const NONCE_LEN: usize = 12;

/// kv_store value 透明压缩魔数 (neotrix-experience 的 _VALUE_MAGIC, 与旧 Python 版兼容)。
/// 魔数前缀 + zlib 表示 value 已压缩。Rust 侧读到该前缀时
/// 视为压缩数据: 不尝试解码为明文 (避免乱码), 按无数据跳过, 配合 neotrix-experience 完整解压读取。
pub const VALUE_COMPRESSED_MAGIC: &[u8] = b"NTZ1";

/// 判断 value 是否为 Python 侧压缩存储 (魔数前缀)。压缩值 Rust 侧不解码 (无 zlib),
/// 上层按"无明文数据"处理 — 避免把二进制当 UTF-8 解析产生乱码。
pub fn is_compressed_value(raw: &str) -> bool {
    raw.as_bytes().starts_with(VALUE_COMPRESSED_MAGIC)
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ─── KV Store ───────────────────────────────────────────────────────────────

pub fn kv_get(conn: &Connection, namespace: &str, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM kv_store WHERE namespace=?1 AND key=?2")
        .map_err(|e| format!("kv_get prepare: {}", e))?;
    // 区分真实 SQL 错误与"无行"：无行是正常未命中，错误必须向上传播
    // （否则 schema 漂移/DB 损坏会被静默当成"没有保存过状态"）
    match stmt.query_row(rusqlite::params![namespace, key], |row| row.get::<_, String>(0)) {
        Ok(v) => {
            if is_compressed_value(&v) {
                Ok(None) // 压缩值: Rust 侧不解码, 视为无明文
            } else {
                Ok(Some(v))
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("kv_get query: {}", e)),
    }
}

pub fn kv_set(conn: &Connection, namespace: &str, key: &str, value: &str) -> Result<(), String> {
    let ts = now();
    conn.execute(
        "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(namespace, key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        rusqlite::params![namespace, key, value, ts],
    )
    .map_err(|e| format!("kv_set: {}", e))?;
    Ok(())
}

pub fn kv_delete(conn: &Connection, namespace: &str, key: &str) -> Result<bool, String> {
    let rows = conn
        .execute(
            "DELETE FROM kv_store WHERE namespace=?1 AND key=?2",
            rusqlite::params![namespace, key],
        )
        .map_err(|e| format!("kv_delete: {}", e))?;
    Ok(rows > 0)
}

pub fn kv_list(conn: &Connection, namespace: &str) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM kv_store WHERE namespace=?1 ORDER BY key")
        .map_err(|e| format!("kv_list prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![namespace], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("kv_list query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        let (k, v) = row.map_err(|e| format!("kv_list row: {}", e))?;
        if is_compressed_value(&v) {
            continue; // 压缩值: Rust 侧不解码, 跳过 (Python 侧负责解压读取)
        }
        results.push((k, v));
    }
    Ok(results)
}

pub fn kv_list_namespaces(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT namespace FROM kv_store ORDER BY namespace")
        .map_err(|e| format!("kv_list_namespaces prepare: {}", e))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("kv_list_namespaces query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("kv_list_namespaces row: {}", e))?);
    }
    Ok(results)
}

/// Delete all entries for a given namespace from kv_store (clean slate before re-import)
pub fn kv_purge_namespace(conn: &Connection, namespace: &str) -> Result<usize, String> {
    let rows = conn
        .execute("DELETE FROM kv_store WHERE namespace=?1", rusqlite::params![namespace])
        .map_err(|e| format!("kv_purge_namespace: {}", e))?;
    Ok(rows)
}

// ─── Schema ────────────────────────────────────────────────────────────────

pub const SCHEMA_VERSION: i32 = 8;

/// 初始化 KB schema (全部表 + FTS + 索引 + kv_store + config 等)。
/// 与 NT-MEMORY 原 `nt_memory_schema::initialize` 逐字一致 (D3 下沉)。
pub fn schema_initialize(conn: &Connection) -> rusqlite::Result<()> {
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
            updated_at INTEGER NOT NULL,
            content_hash TEXT
        );

        CREATE TABLE IF NOT EXISTS rkyv_blobs (
            id TEXT PRIMARY KEY,
            namespace TEXT NOT NULL,
            data BLOB NOT NULL,
            checksum TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_rkyv_blobs_ns ON rkyv_blobs(namespace);

        CREATE TABLE IF NOT EXISTS geo_index (
            node_id TEXT PRIMARY KEY,
            lat REAL NOT NULL,
            lng REAL NOT NULL,
            country TEXT DEFAULT '',
            region TEXT DEFAULT '',
            city TEXT DEFAULT '',
            tags TEXT DEFAULT '',
            source TEXT DEFAULT '',
            confidence REAL DEFAULT 0.0,
            updated_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_geo_index_lat ON geo_index(lat);
        CREATE INDEX IF NOT EXISTS idx_geo_index_lng ON geo_index(lng);
        CREATE INDEX IF NOT EXISTS idx_geo_index_country ON geo_index(country);

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
        // ── Migration v8 (UCN Phase 1): skills_index.content_hash ──
        // 写通去重需要内容指纹列。列可能已存在 (新库由上方 CREATE TABLE 直接建列,
        // 或旧 Python 迁移遗留) → 必须先查列存在性, 不得盲 ALTER (R-P20 schema 漂移防护)。
        if version < 8 && !table_column_exists(conn, "skills_index", "content_hash")? {
            conn.execute_batch("ALTER TABLE skills_index ADD COLUMN content_hash TEXT")?;
        }

        conn.execute(
            "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
            [SCHEMA_VERSION],
        )?;
    }

    Ok(())
}

/// 查询表是否已包含某列 (migration 前置守卫)。
fn table_column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&sql)?;
    let cols = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in cols {
        if col? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

// ─── Counts ────────────────────────────────────────────────────────────────

pub fn count_nodes(conn: &Connection) -> rusqlite::Result<usize> {
    conn.query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))
}

pub fn count_edges(conn: &Connection) -> rusqlite::Result<usize> {
    conn.query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("mem conn");
        schema_initialize(&conn).expect("schema init");
        conn
    }

    #[test]
    fn test_kv_roundtrip_persists() {
        let conn = mem_conn();
        kv_set(&conn, "ns", "k", "v1").unwrap();
        assert_eq!(kv_get(&conn, "ns", "k").unwrap(), Some("v1".to_string()));
        assert_eq!(kv_get(&conn, "ns", "missing").unwrap(), None);
    }

    #[test]
    fn test_kv_delete_removes() {
        let conn = mem_conn();
        kv_set(&conn, "ns", "k", "v").unwrap();
        assert!(kv_delete(&conn, "ns", "k").unwrap());
        assert_eq!(kv_get(&conn, "ns", "k").unwrap(), None);
    }

    #[test]
    fn test_schema_version_set() {
        let conn = mem_conn();
        let v: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert!(v >= SCHEMA_VERSION);
    }

    #[test]
    fn test_counts_after_insert() {
        let conn = mem_conn();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at) VALUES ('n1','idea','T',1,1)",
            [],
        )
        .unwrap();
        assert_eq!(count_nodes(&conn).unwrap(), 1);
        assert_eq!(count_edges(&conn).unwrap(), 0);
    }
}