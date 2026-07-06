use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const NONCE_LEN: usize = 12;

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

// ─── KV Store ───────────────────────────────────────────────────────────────

pub fn kv_get(conn: &Connection, namespace: &str, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM kv_store WHERE namespace=?1 AND key=?2")
        .map_err(|e| format!("kv_get prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![namespace, key], |row| row.get::<_, String>(0))
        .ok();
    Ok(result)
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
        results.push(row.map_err(|e| format!("kv_list row: {}", e))?);
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

// ─── Config Entries ─────────────────────────────────────────────────────────

pub fn config_get(conn: &Connection, section: &str, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM config_entries WHERE section=?1 AND key=?2")
        .map_err(|e| format!("config_get prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![section, key], |row| row.get::<_, String>(0))
        .ok();
    Ok(result)
}

pub fn config_set(conn: &Connection, section: &str, key: &str, value: &str, is_secret: bool) -> Result<(), String> {
    let ts = now();
    conn.execute(
        "INSERT INTO config_entries (section, key, value, is_secret, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(section, key) DO UPDATE SET value=excluded.value, is_secret=excluded.is_secret, updated_at=excluded.updated_at",
        rusqlite::params![section, key, value, is_secret as i32, ts],
    )
    .map_err(|e| format!("config_set: {}", e))?;
    Ok(())
}

pub fn config_delete(conn: &Connection, section: &str, key: &str) -> Result<bool, String> {
    let rows = conn
        .execute(
            "DELETE FROM config_entries WHERE section=?1 AND key=?2",
            rusqlite::params![section, key],
        )
        .map_err(|e| format!("config_delete: {}", e))?;
    Ok(rows > 0)
}

pub fn config_list_section(conn: &Connection, section: &str) -> Result<Vec<(String, String, bool)>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value, is_secret FROM config_entries WHERE section=?1 ORDER BY key")
        .map_err(|e| format!("config_list_section prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![section], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)? != 0,
            ))
        })
        .map_err(|e| format!("config_list_section query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("config_list_section row: {}", e))?);
    }
    Ok(results)
}

pub fn config_all_sections(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT section FROM config_entries ORDER BY section")
        .map_err(|e| format!("config_all_sections prepare: {}", e))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("config_all_sections query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("config_all_sections row: {}", e))?);
    }
    Ok(results)
}

// ─── Secrets ────────────────────────────────────────────────────────────────

fn load_master_key() -> [u8; 32] {
    match std::env::var("NEOTRIX_KEYVAULT_KEY").or_else(|_| std::env::var("NEOTRIX_VAULT_KEY")) {
        Ok(key_str) => {
            let key_str = key_str.trim().to_string();
            if let Ok(decoded) = hex::decode(&key_str) {
                if decoded.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&decoded);
                    return key;
                }
            }
            let hash = Sha256::digest(key_str.as_bytes());
            let mut key = [0u8; 32];
            key.copy_from_slice(&hash);
            key
        }
        Err(_) => {
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            let hex_key = hex::encode(key);
            eprintln!("[neotrix] NEOTRIX_VAULT_KEY not set. Generated ephemeral key: {}", hex_key);
            key
        }
    }
}

static CIPHER: LazyLock<Aes256Gcm> = LazyLock::new(|| {
    let key = load_master_key();
    match Aes256Gcm::new_from_slice(&key) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[neotrix] WARNING: AES-256-GCM key init failed: {}. Using zero key (encryption will fail).", e);
            // Zero key will still produce a valid cipher; encrypt/decrypt will return errors at call time
            Aes256Gcm::new_from_slice(&[0u8; 32]).unwrap_or_else(|_| {
                eprintln!("[neotrix] FATAL: cannot create AES-256-GCM cipher even with zero key");
                std::process::abort();
            })
        }
    }
});

fn secret_encrypt(plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = CIPHER
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    Ok((ciphertext, nonce_bytes.to_vec()))
}

fn secret_decrypt(ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<String, String> {
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = CIPHER
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 error: {}", e))
}

pub fn secret_set(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    let (ciphertext, nonce) = secret_encrypt(value)?;
    let ts = now();
    conn.execute(
        "INSERT INTO secrets (key, encrypted_value, nonce, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(key) DO UPDATE SET encrypted_value=excluded.encrypted_value, nonce=excluded.nonce, updated_at=excluded.updated_at",
        rusqlite::params![key, ciphertext, nonce, ts, ts],
    )
    .map_err(|e| format!("secret_set: {}", e))?;
    Ok(())
}

pub fn secret_get(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT encrypted_value, nonce FROM secrets WHERE key=?1")
        .map_err(|e| format!("secret_get prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![key], |row| {
            let ct: Vec<u8> = row.get(0)?;
            let nonce: Vec<u8> = row.get(1)?;
            Ok((ct, nonce))
        })
        .ok();
    match result {
        Some((ct, nonce)) => Ok(Some(secret_decrypt(&ct, &nonce)?)),
        None => Ok(None),
    }
}

pub fn secret_delete(conn: &Connection, key: &str) -> Result<bool, String> {
    let rows = conn
        .execute("DELETE FROM secrets WHERE key=?1", rusqlite::params![key])
        .map_err(|e| format!("secret_delete: {}", e))?;
    Ok(rows > 0)
}

pub fn secret_list(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT key FROM secrets ORDER BY key")
        .map_err(|e| format!("secret_list prepare: {}", e))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("secret_list query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("secret_list row: {}", e))?);
    }
    Ok(results)
}

// ─── Session Logs ───────────────────────────────────────────────────────────

pub fn session_log_append(
    conn: &Connection,
    session_id: &str,
    content: &str,
    content_type: &str,
    metadata: Option<&serde_json::Value>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let seq = conn
        .query_row(
            "SELECT COALESCE(MAX(sequence), 0) + 1 FROM session_logs WHERE session_id=?1",
            rusqlite::params![session_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(1);
    let meta_str = metadata.map(|m| m.to_string());
    conn.execute(
        "INSERT INTO session_logs (id, session_id, sequence, content, content_type, created_at, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, session_id, seq, content, content_type, ts, meta_str],
    )
    .map_err(|e| format!("session_log_append: {}", e))?;
    Ok(id)
}

pub fn session_log_get(
    conn: &Connection,
    session_id: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<(i64, String, String, String, Option<String>)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT sequence, content, content_type, created_at, metadata
             FROM session_logs WHERE session_id=?1
             ORDER BY sequence DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| format!("session_log_get prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![session_id, limit as i64, offset as i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?.to_string(),
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|e| format!("session_log_get query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("session_log_get row: {}", e))?);
    }
    Ok(results)
}

pub fn session_log_list_sessions(conn: &Connection) -> Result<Vec<(String, i64, i64)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT session_id, COUNT(*), MAX(created_at)
             FROM session_logs GROUP BY session_id ORDER BY MAX(created_at) DESC",
        )
        .map_err(|e| format!("session_log_list_sessions prepare: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("session_log_list_sessions query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("session_log_list_sessions row: {}", e))?);
    }
    Ok(results)
}

// ─── Cookies ────────────────────────────────────────────────────────────────

pub fn cookie_set(
    conn: &Connection,
    domain: &str,
    name: &str,
    value: &str,
    path: &str,
    secure: bool,
    http_only: bool,
    expiry: Option<i64>,
) -> Result<(), String> {
    let ts = now();
    conn.execute(
        "INSERT INTO cookies (domain, name, value, path, secure, http_only, expiry, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(domain, name, path) DO UPDATE SET
           value=excluded.value, secure=excluded.secure, http_only=excluded.http_only,
           expiry=excluded.expiry, updated_at=excluded.updated_at",
        rusqlite::params![
            domain, name, value, path, secure as i32, http_only as i32, expiry, ts, ts
        ],
    )
    .map_err(|e| format!("cookie_set: {}", e))?;
    Ok(())
}

pub fn cookie_get(
    conn: &Connection,
    domain: &str,
    name: &str,
    path: &str,
) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM cookies WHERE domain=?1 AND name=?2 AND path=?3")
        .map_err(|e| format!("cookie_get prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![domain, name, path], |row| row.get::<_, String>(0))
        .ok();
    Ok(result)
}

pub fn cookie_list_domain(conn: &Connection, domain: &str) -> Result<Vec<(String, String, String, bool, bool, Option<i64>)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT name, value, path, secure, http_only, expiry
             FROM cookies WHERE domain=?1 ORDER BY name",
        )
        .map_err(|e| format!("cookie_list_domain prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![domain], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)? != 0,
                row.get::<_, i32>(4)? != 0,
                row.get::<_, Option<i64>>(5)?,
            ))
        })
        .map_err(|e| format!("cookie_list_domain query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("cookie_list_domain row: {}", e))?);
    }
    Ok(results)
}

pub fn cookie_delete(conn: &Connection, domain: &str, name: &str, path: &str) -> Result<bool, String> {
    let rows = conn
        .execute(
            "DELETE FROM cookies WHERE domain=?1 AND name=?2 AND path=?3",
            rusqlite::params![domain, name, path],
        )
        .map_err(|e| format!("cookie_delete: {}", e))?;
    Ok(rows > 0)
}

pub fn cookie_purge_expired(conn: &Connection) -> Result<usize, String> {
    let now_ts = now();
    let rows = conn
        .execute("DELETE FROM cookies WHERE expiry IS NOT NULL AND expiry < ?1", rusqlite::params![now_ts])
        .map_err(|e| format!("cookie_purge_expired: {}", e))?;
    Ok(rows)
}

// ─── Binary Assets ──────────────────────────────────────────────────────────

pub fn asset_store(
    conn: &Connection,
    namespace: &str,
    name: &str,
    data: &[u8],
    mime_type: Option<&str>,
    metadata: Option<&serde_json::Value>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let checksum = {
        let hash = Sha256::digest(data);
        hex::encode(hash)
    };
    let size = data.len() as i64;
    let meta_str = metadata.map(|m| m.to_string());
    conn.execute(
        "INSERT INTO binary_assets (id, namespace, name, data, mime_type, size, checksum, created_at, updated_at, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![id, namespace, name, data, mime_type, size, checksum, ts, ts, meta_str],
    )
    .map_err(|e| format!("asset_store: {}", e))?;
    Ok(id)
}

pub fn asset_load(conn: &Connection, id: &str) -> Result<Option<(Vec<u8>, String, String, Option<String>, i64, Option<String>)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT data, namespace, name, mime_type, size, checksum FROM binary_assets WHERE id=?1",
        )
        .map_err(|e| format!("asset_load prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .ok();
    Ok(result)
}

pub fn asset_list(
    conn: &Connection,
    namespace: &str,
) -> Result<Vec<(String, String, Option<String>, Option<String>, i64, Option<String>)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, mime_type, checksum, size, metadata
             FROM binary_assets WHERE namespace=?1 ORDER BY name",
        )
        .map_err(|e| format!("asset_list prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![namespace], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|e| format!("asset_list query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("asset_list row: {}", e))?);
    }
    Ok(results)
}

pub fn asset_delete(conn: &Connection, id: &str) -> Result<bool, String> {
    let rows = conn
        .execute("DELETE FROM binary_assets WHERE id=?1", rusqlite::params![id])
        .map_err(|e| format!("asset_delete: {}", e))?;
    Ok(rows > 0)
}

// ─── Skills Index ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source_path: Option<String>,
    pub tags: Option<String>,
    pub is_builtin: bool,
    pub last_indexed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn skill_upsert(conn: &Connection, name: &str, record: &SkillRecord) -> Result<(), String> {
    let ts = now();
    conn.execute(
        "INSERT INTO skills_index (id, name, description, source_path, tags, is_builtin, last_indexed_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(name) DO UPDATE SET
           description=excluded.description, source_path=excluded.source_path,
           tags=excluded.tags, last_indexed_at=excluded.last_indexed_at, updated_at=excluded.updated_at",
        rusqlite::params![
            record.id, name, record.description, record.source_path, record.tags,
            record.is_builtin as i32, record.last_indexed_at, record.created_at, ts,
        ],
    )
    .map_err(|e| format!("skill_upsert: {}", e))?;
    Ok(())
}

pub fn skill_search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<SkillRecord>, String> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, source_path, tags,
                    is_builtin, last_indexed_at, created_at, updated_at
             FROM skills_index
             WHERE name LIKE ?1 OR description LIKE ?1 OR tags LIKE ?1
             ORDER BY last_indexed_at DESC NULLS LAST
             LIMIT ?2",
        )
        .map_err(|e| format!("skill_search prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![pattern, limit as i64], |row| {
            Ok(SkillRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_path: row.get(3)?,
                tags: row.get(4)?,
                is_builtin: row.get::<_, i32>(5)? != 0,
                last_indexed_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("skill_search query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("skill_search row: {}", e))?);
    }
    Ok(results)
}

pub fn skill_list_all(conn: &Connection, limit: usize) -> Result<Vec<SkillRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, source_path, tags,
                    is_builtin, last_indexed_at, created_at, updated_at
             FROM skills_index ORDER BY name LIMIT ?1",
        )
        .map_err(|e| format!("skill_list_all prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![limit as i64], map_skill_row)
        .map_err(|e| format!("skill_list_all query: {}", e))?;
    collect_skills(rows)
}

pub fn skill_delete(conn: &Connection, name: &str) -> Result<bool, String> {
    let rows = conn
        .execute("DELETE FROM skills_index WHERE name=?1", rusqlite::params![name])
        .map_err(|e| format!("skill_delete: {}", e))?;
    Ok(rows > 0)
}

fn map_skill_row(row: &rusqlite::Row) -> rusqlite::Result<SkillRecord> {
    Ok(SkillRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        source_path: row.get(3)?,
        tags: row.get(4)?,
        is_builtin: row.get::<_, i32>(5)? != 0,
        last_indexed_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn collect_skills(rows: impl Iterator<Item = Result<SkillRecord, rusqlite::Error>>) -> Result<Vec<SkillRecord>, String> {
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("skill row: {}", e))?);
    }
    Ok(results)
}

// ─── Rkyv Blobs ─────────────────────────────────────────────────────────────

pub fn rkyv_store(conn: &Connection, namespace: &str, data: &[u8]) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let checksum = {
        let hash = Sha256::digest(data);
        hex::encode(hash)
    };
    conn.execute(
        "INSERT INTO rkyv_blobs (id, namespace, data, checksum, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, namespace, data, checksum, ts],
    )
    .map_err(|e| format!("rkyv_store: {}", e))?;
    Ok(id)
}

pub fn rkyv_load(conn: &Connection, id: &str) -> Result<Option<(Vec<u8>, String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT data, namespace, checksum FROM rkyv_blobs WHERE id=?1")
        .map_err(|e| format!("rkyv_load prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .ok();
    Ok(result)
}

pub fn rkyv_list(conn: &Connection, namespace: &str) -> Result<Vec<(String, String, i64)>, String> {
    let mut stmt = conn
        .prepare("SELECT id, checksum, created_at FROM rkyv_blobs WHERE namespace=?1 ORDER BY created_at DESC")
        .map_err(|e| format!("rkyv_list prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![namespace], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("rkyv_list query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("rkyv_list row: {}", e))?);
    }
    Ok(results)
}

pub fn rkyv_delete(conn: &Connection, id: &str) -> Result<bool, String> {
    let rows = conn
        .execute("DELETE FROM rkyv_blobs WHERE id=?1", rusqlite::params![id])
        .map_err(|e| format!("rkyv_delete: {}", e))?;
    Ok(rows > 0)
}

// ─── Migration from Files ───────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct MigrationReport {
    pub total_files_migrated: usize,
    pub kv_entries_created: usize,
    pub config_entries_created: usize,
    pub secrets_migrated: usize,
    pub session_logs_migrated: usize,
    pub cookies_migrated: usize,
    pub assets_migrated: usize,
    pub skills_indexed: usize,
    pub rkyv_blobs_migrated: usize,
    pub errors: Vec<(String, String)>,
}

impl MigrationReport {
    pub fn summary(&self) -> String {
        format!(
            "Migration complete:\n  Files migrated: {}\n  KV entries: {}\n  Config entries: {}\n  Secrets: {}\n  Session logs: {}\n  Cookies: {}\n  Assets: {}\n  Skills indexed: {}\n  Rkyv blobs: {}\n  Errors: {}",
            self.total_files_migrated, self.kv_entries_created, self.config_entries_created,
            self.secrets_migrated, self.session_logs_migrated, self.cookies_migrated,
            self.assets_migrated, self.skills_indexed, self.rkyv_blobs_migrated, self.errors.len()
        )
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn neotrix_dir() -> PathBuf {
    home_dir().join(".neotrix")
}

fn read_text_file(path: &PathBuf) -> Option<String> {
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

/// Migrate all existing external files into KB. Leaves originals in place.
pub fn migrate_from_files(conn: &Connection) -> MigrationReport {
    let mut report = MigrationReport::default();
    let base = neotrix_dir();

    // ── 1. KV Store: lightweight metadata only (not state snapshots) ──
    // State snapshots (brain/cortex/e8/narrative/whitebox/goals etc.) stay as files.
    // Only track paths and lightweight references.
    let kv_sources: Vec<(&str, &str)> = vec![
        ("data_ingest_metrics", "data_ingest_metrics.json"),
        ("knowledge_v2_snap", "knowledge_v2.snap"),
    ];
    for (ns, filename) in &kv_sources {
        let path = base.join(filename);
        if let Some(content) = read_text_file(&path) {
            report.total_files_migrated += 1;
            if kv_set(conn, ns, "data", &content).is_ok() {
                report.kv_entries_created += 1;
            }
        }
    }

    // Track path to journal_index.db (separate SQLite DB)
    let journal_index_path = base.join("journal_index.db");
    if journal_index_path.exists() {
        report.total_files_migrated += 1;
        if kv_set(conn, "journal_index", "path", &journal_index_path.to_string_lossy()).is_ok() {
            report.kv_entries_created += 1;
        }
    }

    // Track path to exploration_sources.txt
    let exploration_path = base.join("exploration_sources.txt");
    if exploration_path.exists() {
        report.total_files_migrated += 1;
        if kv_set(conn, "exploration", "sources_path", &exploration_path.to_string_lossy()).is_ok() {
            report.kv_entries_created += 1;
        }
    }

    // ── 2. Config entries ──
    // config.toml
    let config_path = base.join("config.toml");
    if let Some(content) = read_text_file(&config_path) {
        report.total_files_migrated += 1;
        if let Ok(table) = content.parse::<toml::Table>() {
            for (section, value) in &table {
                if let Some(val_table) = value.as_table() {
                    for (k, v) in val_table {
                        let val_str = v.to_string();
                        // Strip quotes from string values
                        let val_str = val_str.trim_matches('"').to_string();
                        if config_set(conn, section, k, &val_str, false).is_ok() {
                            report.config_entries_created += 1;
                        }
                    }
                } else {
                    let val_str = value.to_string().trim_matches('"').to_string();
                    if config_set(conn, "root", section, &val_str, false).is_ok() {
                        report.config_entries_created += 1;
                    }
                }
            }
        }
    }

    // profiles.toml
    let profiles_path = base.join("profiles.toml");
    if let Some(content) = read_text_file(&profiles_path) {
        report.total_files_migrated += 1;
        if let Ok(table) = content.parse::<toml::Table>() {
            for (section, value) in &table {
                let val_str = value.to_string();
                if config_set(conn, "profiles", section, &val_str, false).is_ok() {
                    report.config_entries_created += 1;
                }
            }
        }
    }

    // router_config.toml
    let router_cfg_path = base.join("router_config.toml");
    if let Some(content) = read_text_file(&router_cfg_path) {
        report.total_files_migrated += 1;
        if let Ok(table) = content.parse::<toml::Table>() {
            for (section, value) in &table {
                let val_str = value.to_string();
                if config_set(conn, "router", section, &val_str, false).is_ok() {
                    report.config_entries_created += 1;
                }
            }
        }
    }

    // ── 3. Env vars → config entries ──
    let neotrix_vars = [
        "NEOTRIX_MODEL", "NEOTRIX_PROVIDER", "NEOTRIX_BASE_URL", "NEOTRIX_TIMEOUT",
        "NEOTRIX_API_KEY", "NEOTRIX_EMBEDDING_API_KEY", "NEOTRIX_EMBEDDING_BASE_URL",
        "NEOTRIX_EMBEDDING_MODEL", "NEOTRIX_EMBEDDING_DIMENSION", "NEOTRIX_SEARCH_API",
        "NEOTRIX_ZEN_URL", "NEOTRIX_GATEWAY_ADDR", "NEOTRIX_PROXY_SUB_URL",
        "NEOTRIX_SPLIT_ENABLE", "NEOTRIX_HEALTH_FILE", "NEOTRIX_HOME",
        "NEOTRIX_API_TOKEN", "NEOTRIX_SENTRY_DSN",
    ];
    for var_name in &neotrix_vars {
        if let Ok(val) = std::env::var(var_name) {
            let section = "env";
            let is_secret = var_name.contains("API_KEY") || var_name.contains("TOKEN") || var_name.contains("SENTRY");
            if config_set(conn, section, var_name, &val, is_secret).is_ok() {
                report.config_entries_created += 1;
            }
        }
    }

    // ── 4. Secrets from secrets.json ──
    let secrets_path = base.join("secrets.json");
    if let Some(content) = read_text_file(&secrets_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            report.total_files_migrated += 1;
            if let Some(obj) = data.as_object() {
                for (k, v) in obj {
                    let val_str = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                    if secret_set(conn, k, &val_str).is_ok() {
                        report.secrets_migrated += 1;
                    } else {
                        report.errors.push(("secrets.json".into(), format!("secret_set {} failed", k)));
                    }
                }
            }
        }
    }

    // ── 5. Session logs from journal/ ──
    let journal_dir = base.join("journal");
    if journal_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&journal_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Some(content) = read_text_file(&path) {
                        report.total_files_migrated += 1;
                        let session_id = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        if session_log_append(conn, &session_id, &content, "markdown", None).is_ok() {
                            report.session_logs_migrated += 1;
                        }
                    }
                }
            }
        }
    }

    // ── 6. Session logs from session-logs/ ──
    let session_logs_dir = base.join("session-logs");
    if session_logs_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&session_logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(content) = read_text_file(&path) {
                    report.total_files_migrated += 1;
                    let session_id = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    if session_log_append(conn, &format!("session-log-{}", session_id), &content, "log", None).is_ok() {
                        report.session_logs_migrated += 1;
                    }
                }
            }
        }
    }

    // ── 7. Session shares ──
    let shares_dir = base.join("shares");
    if shares_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&shares_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(content) = read_text_file(&path) {
                    report.total_files_migrated += 1;
                    let session_id = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("share")
                        .to_string();
                    if session_log_append(conn, &format!("share-{}", session_id), &content, "json", None).is_ok() {
                        report.session_logs_migrated += 1;
                    }
                }
            }
        }
    }

    // ── 8. Cookies from cookies/ ──
    let cookies_dir = base.join("cookies");
    if cookies_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&cookies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Some(content) = read_text_file(&path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(obj) = data.as_object() {
                                report.total_files_migrated += 1;
                                let domain = path.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown");
                                for (name, value) in obj {
                                    let val_str = if value.is_string() {
                                        value.as_str().map(|s| s.to_string())
                                            .unwrap_or_else(|| value.to_string())
                                    } else {
                                        value.to_string()
                                    };
                                    if cookie_set(conn, domain, name, &val_str, "/", false, false, None).is_ok() {
                                        report.cookies_migrated += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── 9. Binary assets ──
    // NOT stored in KB: avatars/, snapshots/, chrome-profile/ stay as files.
    // Binary blobs in SQLite cause DB bloat and prevent streaming.
    // Only store reference paths if needed.
    for dir_name in &["avatars", "snapshots", "chrome-profile"] {
        let dir = base.join(dir_name);
        if dir.is_dir() {
            report.total_files_migrated += 1;
            let _ = kv_set(conn, "binary_refs", dir_name, &dir.to_string_lossy());
            report.kv_entries_created += 1;
        }
    }

    // ── 10. Skills index (metadata only: name, description, tags, source_path) ──
    // SKILL.md content stays in files; KB only stores searchable metadata.
    let skill_dirs = vec![
        base.join("skills"),
        home_dir().join(".agents").join("skills"),
        home_dir().join(".claude").join("skills"),
    ];
    for skills_path in &skill_dirs {
        if skills_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(skills_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(content) = read_text_file(&skill_md) {
                            report.total_files_migrated += 1;
                            let name = path.file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let description = extract_description(&content);
                            let tags_str = extract_tags_from_content(&content);
                            let record = SkillRecord {
                                id: Uuid::new_v4().to_string(),
                                name,
                                description: Some(description),
                                source_path: Some(skill_md.to_string_lossy().to_string()),
                                tags: tags_str,
                                is_builtin: false,
                                last_indexed_at: Some(now()),
                                created_at: now(),
                                updated_at: now(),
                            };
                            if skill_upsert(conn, &record.name, &record).is_ok() {
                                report.skills_indexed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // ── 11. Rkyv blobs ── stay as files (zero-copy requires mmap).
    // ── 12. journal_index.db ── already tracked via kv_store section above.

    report
}

fn extract_description(content: &str) -> String {
    // Try to get first non-empty line that isn't YAML frontmatter
    for line in content.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with("---") && !line.starts_with('#') && !line.starts_with(':') {
            return line.to_string();
        }
    }
    "No description".to_string()
}

fn extract_tags_from_content(content: &str) -> Option<String> {
    // Try to find YAML frontmatter and extract tags from it
    let trimmed = content.trim();
    if trimmed.starts_with("---") {
        if let Some(end) = trimmed[3..].find("\n---") {
            let yaml = &trimmed[3..3 + end];
            for line in yaml.lines() {
                if line.trim().starts_with("tags:") {
                    let tags = line.trim_start_matches("tags:").trim();
                    if tags.starts_with('[') && tags.ends_with(']') {
                        let inner = tags.trim_start_matches('[').trim_end_matches(']');
                        return Some(inner.split(',').map(|t| t.trim().trim_matches('"').trim_matches('\'')).collect::<Vec<_>>().join(","));
                    }
                    return Some(tags.to_string());
                }
            }
        }
    }
    None
}

// ─── Cleanup helpers ────────────────────────────────────────────────────────

/// Delete all entries for a given namespace from kv_store (clean slate before re-import)
pub fn kv_purge_namespace(conn: &Connection, namespace: &str) -> Result<usize, String> {
    let rows = conn
        .execute("DELETE FROM kv_store WHERE namespace=?1", rusqlite::params![namespace])
        .map_err(|e| format!("kv_purge_namespace: {}", e))?;
    Ok(rows)
}

/// Get total size stats for the unified store
pub fn store_stats(conn: &Connection) -> Result<HashMap<String, usize>, String> {
    let tables = [
        "kv_store", "config_entries", "secrets", "session_logs",
        "cookies", "binary_assets", "skills_index", "rkyv_blobs",
    ];
    let mut stats = HashMap::new();
    for table in &tables {
        let sql = format!("SELECT COUNT(*) FROM {}", table);
        if let Ok(count) = conn.query_row(&sql, [], |row| row.get::<_, i64>(0)) {
            stats.insert(table.to_string(), count as usize);
        }
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema;

    fn split_yaml_frontmatter(content: &str) -> (Option<String>, String) {
        let trimmed = content.trim();
        if trimmed.starts_with("---") {
            if let Some(end) = trimmed[3..].find("\n---") {
                let yaml = trimmed[3..3 + end].to_string();
                let body = trimmed[3 + end + 4..].trim().to_string();
                return (Some(yaml), body);
            }
        }
        (None, content.to_string())
    }

    fn mime_for_extension(ext: Option<&str>) -> Option<String> {
        let mime = match ext.unwrap_or("") {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "json" => "application/json",
            "xml" => "application/xml",
            "yaml" | "yml" => "application/yaml",
            "pdf" => "application/pdf",
            "md" | "markdown" => "text/markdown",
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "csv" => "text/csv",
            "toml" => "application/toml",
            "rs" => "text/x-rust",
            "py" => "text/x-python",
            "js" => "application/javascript",
            "ts" => "application/typescript",
            _ => "application/octet-stream",
        };
        Some(mime.to_string())
    }

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_kv_store_roundtrip() {
        let conn = test_conn();
        kv_set(&conn, "test", "key1", "value1").unwrap();
        assert_eq!(kv_get(&conn, "test", "key1").unwrap(), Some("value1".into()));
    }

    #[test]
    fn test_kv_store_overwrite() {
        let conn = test_conn();
        kv_set(&conn, "test", "k", "v1").unwrap();
        kv_set(&conn, "test", "k", "v2").unwrap();
        assert_eq!(kv_get(&conn, "test", "k").unwrap(), Some("v2".into()));
    }

    #[test]
    fn test_kv_store_delete() {
        let conn = test_conn();
        kv_set(&conn, "test", "k", "v").unwrap();
        assert!(kv_delete(&conn, "test", "k").unwrap());
        assert_eq!(kv_get(&conn, "test", "k").unwrap(), None);
        assert!(!kv_delete(&conn, "test", "k").unwrap());
    }

    #[test]
    fn test_kv_store_list() {
        let conn = test_conn();
        kv_set(&conn, "ns1", "a", "1").unwrap();
        kv_set(&conn, "ns1", "b", "2").unwrap();
        let entries = kv_list(&conn, "ns1").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_kv_list_namespaces() {
        let conn = test_conn();
        kv_set(&conn, "ns_a", "k", "v").unwrap();
        kv_set(&conn, "ns_b", "k", "v").unwrap();
        let namespaces = kv_list_namespaces(&conn).unwrap();
        assert_eq!(namespaces.len(), 2);
        assert!(namespaces.contains(&"ns_a".to_string()));
        assert!(namespaces.contains(&"ns_b".to_string()));
    }

    #[test]
    fn test_config_roundtrip() {
        let conn = test_conn();
        config_set(&conn, "llm", "model", "claude-4", false).unwrap();
        assert_eq!(config_get(&conn, "llm", "model").unwrap(), Some("claude-4".into()));
    }

    #[test]
    fn test_config_list_section() {
        let conn = test_conn();
        config_set(&conn, "llm", "a", "1", false).unwrap();
        config_set(&conn, "llm", "b", "2", true).unwrap();
        let entries = config_list_section(&conn, "llm").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_secret_encrypt_decrypt() {
        let conn = test_conn();
        secret_set(&conn, "test_key", "sensitive_value").unwrap();
        let val = secret_get(&conn, "test_key").unwrap();
        assert_eq!(val, Some("sensitive_value".into()));
    }

    #[test]
    fn test_secret_delete() {
        let conn = test_conn();
        secret_set(&conn, "k", "v").unwrap();
        assert!(secret_delete(&conn, "k").unwrap());
        assert_eq!(secret_get(&conn, "k").unwrap(), None);
    }

    #[test]
    fn test_session_log_append_and_retrieve() {
        let conn = test_conn();
        let id1 = session_log_append(&conn, "session1", "hello", "markdown", None).unwrap();
        let id2 = session_log_append(&conn, "session1", "world", "markdown", None).unwrap();
        assert_ne!(id1, id2);
        let logs = session_log_get(&conn, "session1", 10, 0).unwrap();
        assert_eq!(logs.len(), 2);
        // Returns in DESC sequence, so "world" first
        assert!(logs[0].1.contains("world"));
        assert!(logs[1].1.contains("hello"));
    }

    #[test]
    fn test_session_log_list_sessions() {
        let conn = test_conn();
        session_log_append(&conn, "s1", "a", "text", None).unwrap();
        session_log_append(&conn, "s2", "b", "text", None).unwrap();
        let sessions = session_log_list_sessions(&conn).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_cookie_roundtrip() {
        let conn = test_conn();
        cookie_set(&conn, "example.com", "session", "abc123", "/", true, true, None).unwrap();
        assert_eq!(cookie_get(&conn, "example.com", "session", "/").unwrap(), Some("abc123".into()));
    }

    #[test]
    fn test_cookie_list_domain() {
        let conn = test_conn();
        cookie_set(&conn, "example.com", "a", "1", "/", false, false, None).unwrap();
        cookie_set(&conn, "example.com", "b", "2", "/", false, false, None).unwrap();
        let cookies = cookie_list_domain(&conn, "example.com").unwrap();
        assert_eq!(cookies.len(), 2);
    }

    #[test]
    fn test_cookie_delete() {
        let conn = test_conn();
        cookie_set(&conn, "ex.com", "k", "v", "/", false, false, None).unwrap();
        assert!(cookie_delete(&conn, "ex.com", "k", "/").unwrap());
        assert_eq!(cookie_get(&conn, "ex.com", "k", "/").unwrap(), None);
    }

    #[test]
    fn test_cookie_purge_expired() {
        let conn = test_conn();
        cookie_set(&conn, "ex.com", "valid", "ok", "/", false, false, Some(now() + 86400)).unwrap();
        cookie_set(&conn, "ex.com", "expired", "old", "/", false, false, Some(now() - 86400)).unwrap();
        let purged = cookie_purge_expired(&conn).unwrap();
        assert_eq!(purged, 1);
    }

    #[test]
    fn test_asset_store_load_delete() {
        let conn = test_conn();
        let data = b"hello world";
        let id = asset_store(&conn, "test", "hello.txt", data, Some("text/plain"), None).unwrap();
        let loaded = asset_load(&conn, &id).unwrap();
        assert!(loaded.is_some());
        let (loaded_data, ns, name, mime, size, _) = loaded.unwrap();
        assert_eq!(loaded_data, data);
        assert_eq!(ns, "test");
        assert_eq!(name, "hello.txt");
        assert_eq!(mime, Some("text/plain".to_string()));
        assert_eq!(size, 11);
        assert!(asset_delete(&conn, &id).unwrap());
        assert!(asset_load(&conn, &id).unwrap().is_none());
    }

    #[test]
    fn test_skill_upsert_search() {
        let conn = test_conn();
        let record = SkillRecord {
            id: "id1".into(),
            name: "test-skill".into(),
            description: Some("A test skill".into()),
            source_path: Some("/path/to/skill".into()),
            tags: Some("test,example".into()),
            is_builtin: false,
            last_indexed_at: Some(now()),
            created_at: now(),
            updated_at: now(),
        };
        skill_upsert(&conn, "test-skill", &record).unwrap();
        let results = skill_search(&conn, "test", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test-skill");
    }

    #[test]
    fn test_rkyv_store_load() {
        let conn = test_conn();
        let data = b"binary data for rkyv";
        let id = rkyv_store(&conn, "test_ns", data).unwrap();
        let loaded = rkyv_load(&conn, &id).unwrap();
        assert!(loaded.is_some());
        let (loaded_data, ns, _) = loaded.unwrap();
        assert_eq!(loaded_data, data);
        assert_eq!(ns, "test_ns");
    }

    #[test]
    fn test_rkyv_list() {
        let conn = test_conn();
        rkyv_store(&conn, "ns1", b"a").unwrap();
        rkyv_store(&conn, "ns1", b"b").unwrap();
        let entries = rkyv_list(&conn, "ns1").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_kv_purge_namespace() {
        let conn = test_conn();
        kv_set(&conn, "purge_test", "k1", "v1").unwrap();
        kv_set(&conn, "purge_test", "k2", "v2").unwrap();
        kv_set(&conn, "other", "k3", "v3").unwrap();
        assert_eq!(kv_purge_namespace(&conn, "purge_test").unwrap(), 2);
        assert_eq!(kv_list(&conn, "purge_test").unwrap().len(), 0);
        assert_eq!(kv_list(&conn, "other").unwrap().len(), 1);
    }

    #[test]
    fn test_store_stats() {
        let conn = test_conn();
        kv_set(&conn, "ns", "k", "v").unwrap();
        config_set(&conn, "sec", "k", "v", false).unwrap();
        let stats = store_stats(&conn).unwrap();
        assert_eq!(*stats.get("kv_store").unwrap_or(&0), 1);
        assert_eq!(*stats.get("config_entries").unwrap_or(&0), 1);
    }

    #[test]
    fn test_yaml_frontmatter_split() {
        let content = "---\ntitle: Test\ntags: [a, b]\n---\n\nBody text";
        let (yaml, body) = split_yaml_frontmatter(content);
        assert!(yaml.is_some());
        assert!(yaml.as_ref().unwrap().contains("title: Test"));
        assert!(body.contains("Body text"));
    }

    #[test]
    fn test_yaml_no_frontmatter() {
        let content = "Just body text";
        let (yaml, body) = split_yaml_frontmatter(content);
        assert!(yaml.is_none());
        assert_eq!(body, "Just body text");
    }

    #[test]
    fn test_extract_description() {
        let content = "---\ntitle: X\n---\n\nThis is the description.\nMore text.";
        let desc = extract_description(content);
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_extract_tags() {
        let tags = extract_tags_from_content("---\ntags: [rust, testing, ai]\n---\ncontent here");
        assert!(tags.is_some());
        let t = tags.unwrap();
        assert!(t.contains("rust"));
    }

    #[test]
    fn test_mime_for_extension() {
        assert_eq!(mime_for_extension(Some("png")), Some("image/png".into()));
        assert_eq!(mime_for_extension(Some("json")), Some("application/json".into()));
        assert_eq!(mime_for_extension(Some("unknown_ext")), Some("application/octet-stream".into()));
        assert_eq!(mime_for_extension(None), Some("application/octet-stream".into()));
    }

    #[test]
    fn test_secret_list() {
        let conn = test_conn();
        secret_set(&conn, "key_a", "val_a").unwrap();
        secret_set(&conn, "key_b", "val_b").unwrap();
        let keys = secret_list(&conn).unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key_a".to_string()));
    }

    #[test]
    fn test_config_delete() {
        let conn = test_conn();
        config_set(&conn, "test", "k", "v", false).unwrap();
        assert!(config_delete(&conn, "test", "k").unwrap());
        assert_eq!(config_get(&conn, "test", "k").unwrap(), None);
    }

    #[test]
    fn test_empty_kv_get() {
        let conn = test_conn();
        assert_eq!(kv_get(&conn, "nonexistent", "key").unwrap(), None);
    }
}
