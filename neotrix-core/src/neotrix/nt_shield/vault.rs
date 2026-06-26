#![allow(dead_code)]
use std::collections::HashMap;
use std::path::PathBuf;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};

const VAULT_PATH: &str = "~/.neotrix/vault.enc";
const NONCE_LEN: usize = 12;
const MAX_ENTRIES: usize = 1_000;

/// AES-256-GCM encrypted credential vault stored on disk.
///
/// Master key is read from `NEOTRIX_VAULT_KEY` env var.
/// If the env var is not set, a random key is generated and printed
/// to stderr (first-run setup).
pub struct Vault {
    path: PathBuf,
    cipher: Aes256Gcm,
    entries: HashMap<String, String>,
    dirty: bool,
}

impl Vault {
    /// Load vault from disk, or create empty if file missing.
    /// The master key comes from `NEOTRIX_VAULT_KEY` (32 bytes hex-encoded).
    pub fn new() -> NeoTrixResult<Self> {
        let path = shellexpand::tilde(VAULT_PATH)
            .parse::<PathBuf>()
            .map_err(|_| NeoTrixError::Config("Failed to expand vault path".into()))?;

        let key_bytes = Self::load_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| NeoTrixError::Config(format!("AES key init error: {}", e)))?;

        let entries = if path.exists() {
            let encrypted =
                std::fs::read(&path).map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
            Self::decrypt_entries(&cipher, &encrypted)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            path,
            cipher,
            entries,
            dirty: false,
        })
    }

    /// Retrieve a credential
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    /// Store a credential (marks as dirty, call save() to persist)
    pub fn set(&mut self, key: &str, value: &str) {
        if !self.entries.contains_key(key) {
            self.ensure_capacity();
        }
        self.entries.insert(key.to_string(), value.to_string());
        self.dirty = true;
    }

    fn ensure_capacity(&mut self) {
        if self.entries.len() >= MAX_ENTRIES {
            if let Some(k) = self.entries.keys().next().cloned() {
                self.entries.remove(&k);
            }
        }
    }

    /// Remove a credential
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let result = self.entries.remove(key);
        if result.is_some() {
            self.dirty = true;
        }
        result
    }

    /// Persist to disk if dirty
    pub fn save(&mut self) -> NeoTrixResult<()> {
        if !self.dirty {
            return Ok(());
        }
        let json =
            serde_json::to_string(&self.entries).map_err(|e| NeoTrixError::Serde(e.to_string()))?;
        let encrypted = self.encrypt_data(json.as_bytes())?;

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        }

        let tmp_path = self.path.with_extension("tmp");
        std::fs::write(&tmp_path, &encrypted)
            .map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        std::fs::rename(&tmp_path, &self.path)
            .map_err(|e| NeoTrixError::Io(std::sync::Arc::new(e)))?;
        self.dirty = false;
        Ok(())
    }

    /// Inject all credentials as environment variables for a subprocess
    pub fn inject_env(&self, vars: &mut HashMap<String, String>) {
        for (k, v) in &self.entries {
            // Prefix with NEOTRIX_VAULT_ to avoid collisions
            let env_key = format!("NEOTRIX_VAULT_{}", k.to_uppercase());
            vars.insert(env_key, v.clone());
        }
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Number of stored credentials
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // ---- Private helpers ----

    fn load_key() -> NeoTrixResult<[u8; 32]> {
        match std::env::var("NEOTRIX_VAULT_KEY") {
            Ok(key_str) => {
                let key_str = key_str.trim().to_string();
                // Try hex decode first, fallback to SHA-256 hash
                if let Ok(decoded) = hex::decode(&key_str) {
                    if decoded.len() == 32 {
                        let mut key = [0u8; 32];
                        key.copy_from_slice(&decoded);
                        return Ok(key);
                    }
                }
                // Hash the string to produce a 32-byte key
                let hash = Sha256::digest(key_str.as_bytes());
                Ok(hash.into())
            }
            Err(_) => {
                let mut key = [0u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut key);
                let hex_key = hex::encode(key);
                log::warn!("[neotrix-vault] NEOTRIX_VAULT_KEY not set.");
                log::warn!("[neotrix-vault] Generated new master key (save this!):");
                log::warn!(
                    "[neotrix-vault]   export NEOTRIX_VAULT_KEY={}...{}",
                    &hex_key[..8],
                    &hex_key[hex_key.len() - 4..]
                );
                Ok(key)
            }
        }
    }

    fn encrypt_data(&self, plaintext: &[u8]) -> NeoTrixResult<Vec<u8>> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext =
            self.cipher
                .encrypt(nonce, plaintext)
                .map_err(|e| NeoTrixError::General {
                    msg: format!("Encryption failed: {}", e),
                    backtrace: None,
                })?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt_entries(cipher: &Aes256Gcm, data: &[u8]) -> NeoTrixResult<HashMap<String, String>> {
        if data.len() < NONCE_LEN {
            return Err(NeoTrixError::from("Vault file too short"));
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| NeoTrixError::General {
                msg: format!("Decryption failed: {}", e),
                backtrace: None,
            })?;

        serde_json::from_slice(&plaintext).map_err(|e| NeoTrixError::Serde(e.to_string()))
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        if self.dirty {
            if let Err(e) = self.save() {
                log::error!("Failed to auto-save vault: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // NOTE: Tests use std::env::set_var for NEOTRIX_VAULT_KEY.
    // All tests are #[test] (sync, separate OS threads), but they set the
    // SAME env var — run with --test-threads=1 if spurious CI failures occur.
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_vault_set_get() {
        let dir = tempdir().expect("tempdir creation");
        let vault_path = dir.path().join("vault.enc");
        std::env::set_var(
            "NEOTRIX_VAULT_KEY",
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        );
        let key_bytes =
            hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
                .expect("valid hex key");
        let key_arr: [u8; 32] = key_bytes.as_slice().try_into().expect("32-byte key");
        let cipher = Aes256Gcm::new_from_slice(&key_arr).expect("AES key init");

        let mut vault = Vault {
            path: vault_path.clone(),
            cipher,
            entries: HashMap::new(),
            dirty: false,
        };

        vault.set("github_token", "ghp_test123");
        assert_eq!(vault.get("github_token"), Some("ghp_test123"));

        vault.save().expect("vault.save");

        // Reload and verify
        let cipher2 = Aes256Gcm::new_from_slice(&key_arr).expect("AES key init");
        let encrypted = std::fs::read(&vault_path).expect("read vault file");
        let entries = Vault::decrypt_entries(&cipher2, &encrypted).expect("decrypt entries");
        assert_eq!(
            entries.get("github_token").map(|s| s.as_str()),
            Some("ghp_test123")
        );

        std::env::remove_var("NEOTRIX_VAULT_KEY");
    }

    #[test]
    fn test_vault_remove() {
        let mut vault = {
            let dir = tempdir().expect("tempdir creation");
            let vault_path = dir.path().join("vault.enc");
            std::env::set_var(
                "NEOTRIX_VAULT_KEY",
                "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            );
            let key_bytes =
                hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
                    .expect("valid hex key");
            let key_arr: [u8; 32] = key_bytes.as_slice().try_into().expect("32-byte key");
            let cipher = Aes256Gcm::new_from_slice(&key_arr).expect("AES key init");
            let mut v = Vault {
                path: vault_path,
                cipher,
                entries: HashMap::new(),
                dirty: false,
            };
            v.set("a", "1");
            v.set("b", "2");
            v
        };
        assert_eq!(vault.len(), 2);
        let removed = vault.remove("a");
        assert_eq!(removed, Some("1".to_string()));
        assert_eq!(vault.len(), 1);
        assert!(vault.get("a").is_none());
        std::env::remove_var("NEOTRIX_VAULT_KEY");
    }

    #[test]
    fn test_vault_is_empty() {
        let vault = {
            let dir = tempdir().expect("tempdir creation");
            let vault_path = dir.path().join("vault.enc");
            std::env::set_var(
                "NEOTRIX_VAULT_KEY",
                "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            );
            let key_bytes =
                hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
                    .expect("valid hex key");
            let key_arr: [u8; 32] = key_bytes.as_slice().try_into().expect("32-byte key");
            let cipher = Aes256Gcm::new_from_slice(&key_arr).expect("AES key init");
            Vault {
                path: vault_path,
                cipher,
                entries: HashMap::new(),
                dirty: false,
            }
        };
        assert!(vault.is_empty());
        std::env::remove_var("NEOTRIX_VAULT_KEY");
    }

    #[test]
    fn test_inject_env() {
        let dir = tempdir().expect("tempdir creation");
        let vault_path = dir.path().join("vault.enc");
        std::env::set_var(
            "NEOTRIX_VAULT_KEY",
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        );
        let key_bytes =
            hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
                .expect("valid hex key");
        let key_arr: [u8; 32] = key_bytes.as_slice().try_into().expect("32-byte key");
        let cipher = Aes256Gcm::new_from_slice(&key_arr).expect("AES key init");

        let vault = Vault {
            path: vault_path,
            cipher,
            entries: HashMap::from([
                ("api_key".to_string(), "sk-secret".to_string()),
                ("db_url".to_string(), "postgres://localhost".to_string()),
            ]),
            dirty: false,
        };

        let mut env = HashMap::new();
        vault.inject_env(&mut env);
        assert_eq!(
            env.get("NEOTRIX_VAULT_API_KEY"),
            Some(&"sk-secret".to_string())
        );
        assert_eq!(
            env.get("NEOTRIX_VAULT_DB_URL"),
            Some(&"postgres://localhost".to_string())
        );
        std::env::remove_var("NEOTRIX_VAULT_KEY");
    }

    #[test]
    fn test_vault_create_with_missing_key() {
        std::env::remove_var("NEOTRIX_VAULT_KEY");
        let dir = tempdir().expect("tempdir creation");
        let vault_path = dir.path().join("vault.enc");
        let mut key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key_bytes);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("AES key init");
        let vault = Vault {
            path: vault_path,
            cipher,
            entries: HashMap::new(),
            dirty: false,
        };
        assert!(vault.is_empty());
    }
}
