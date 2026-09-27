use std::sync::LazyLock;

use log::{error, warn};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use rusqlite::Connection;
use sha2::{Digest, Sha256};

pub const NONCE_LEN: usize = 12;

// D3 架构倒置: kv 原语下沉至 core (nt_core_kb_primitives), 此处 re-export
// 保持 `nt_memory_unify::kv_*` 调用方路径不变。实现单一事实源在 core。
pub use crate::l0_substrate::nt_core_kb_primitives::{
    is_compressed_value, kv_delete, kv_exists, kv_get, kv_list, kv_list_namespaces,
    kv_purge_namespace, kv_set, VALUE_COMPRESSED_MAGIC,
};

pub fn now() -> i64 {
    crate::l0_substrate::nt_core_kb_primitives::now()
}

// ─── Config Entries ─────────────────────────────────────────────────────────

pub fn config_get(conn: &Connection, section: &str, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM config_entries WHERE section=?1 AND key=?2")
        .map_err(|e| format!("config_get prepare: {}", e))?;
    match stmt.query_row(rusqlite::params![section, key], |row| {
        row.get::<_, String>(0)
    }) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("config_get query: {}", e)),
    }
}

pub fn config_set(
    conn: &Connection,
    section: &str,
    key: &str,
    value: &str,
    is_secret: bool,
) -> Result<(), String> {
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

pub fn config_list_section(
    conn: &Connection,
    section: &str,
) -> Result<Vec<(String, String, bool)>, String> {
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
            warn!(
                "[neotrix] NEOTRIX_VAULT_KEY not set. Generated ephemeral key: {}",
                hex_key
            );
            key
        }
    }
}

static CIPHER: LazyLock<Aes256Gcm> = LazyLock::new(|| {
    let key = load_master_key();
    match Aes256Gcm::new_from_slice(&key) {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "[neotrix] AES-256-GCM key init failed: {}. Using zero key (encryption will fail).",
                e
            );
            // Zero key will still produce a valid cipher; encrypt/decrypt will return errors at call time
            Aes256Gcm::new_from_slice(&[0u8; 32]).unwrap_or_else(|_| {
                error!("[neotrix] FATAL: cannot create AES-256-GCM cipher even with zero key");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_schema;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_kv_store_roundtrip() {
        let conn = test_conn();
        kv_set(&conn, "test", "key1", "value1").unwrap();
        assert_eq!(
            kv_get(&conn, "test", "key1").unwrap(),
            Some("value1".into())
        );
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
        assert_eq!(
            config_get(&conn, "llm", "model").unwrap(),
            Some("claude-4".into())
        );
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
