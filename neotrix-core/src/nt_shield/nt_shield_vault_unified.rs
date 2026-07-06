use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::neotrix::l1_body_impl::nt_l1_error::{L1Error, L1Result};

const ENC_PREFIX: &str = "enc:";
const NONCE_LEN: usize = 12;
const STORAGE_PATH: &str = "~/.neotrix/unified_vault.json";
const VAULT_KEY_ENV: &str = "NEOTRIX_VAULT_KEY";

/// Master encryption key source: OS keychain, env var, or deterministic machine-id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// Retrieved from OS keychain (keyring crate, feature-gated)
    Keyring,
    /// Read from `NEOTRIX_VAULT_KEY` env var
    EnvVar,
    /// Derived from machine-id (deterministic per-machine)
    MachineDerived,
    /// Generated fresh (ephemeral, printed to stderr)
    Generated,
}

/// Unified AES-256-GCM vault — thread-safe, file-backed encrypted key store.
pub struct UnifiedVault {
    path: PathBuf,
    cipher: Aes256Gcm,
    key_source: KeySource,
    entries: Mutex<HashMap<String, String>>,
    dirty: Mutex<bool>,
}

/// A single encrypted entry as stored on disk (value is `enc:` format).
#[derive(Debug, Serialize, Deserialize)]
struct VaultFile {
    entries: HashMap<String, String>,
}

impl UnifiedVault {
    /// Open or create the unified vault.
    ///
    /// Master key resolution follows this priority:
    /// 1. OS keychain (`keyring` feature)
    /// 2. `NEOTRIX_VAULT_KEY` env var
    /// 3. Deterministic machine-derived key
    pub fn new() -> L1Result<Self> {
        let (key_bytes, key_source) = load_master_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| L1Error::Config(format!("AES-256-GCM init error: {}", e)))?;

        let path = shellexpand::tilde(STORAGE_PATH)
            .parse::<PathBuf>()
            .map_err(|_| L1Error::Config("Failed to expand vault path".into()))?;

        let entries = if path.exists() {
            let data = std::fs::read_to_string(&path)
                .map_err(|e| L1Error::Io(e.to_string()))?;
            let vf: VaultFile = serde_json::from_str(&data)
                .map_err(|e| L1Error::Serde(e.to_string()))?;
            vf.entries
        } else {
            HashMap::new()
        };

        Ok(Self {
            path,
            cipher,
            key_source,
            entries: Mutex::new(entries),
            dirty: Mutex::new(false),
        })
    }

    /// Encrypt plaintext bytes under the given key_id.
    ///
    /// The key_id is a label for the stored value. Returns `enc:<base64(nonce+ciphertext)>`.
    pub fn encrypt(&self, data: &[u8], key_id: &str) -> L1Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| L1Error::Crypto(format!("Encryption failed: {}", e)))?;

        let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        let encoded = format!("{}{}", ENC_PREFIX, base64::engine::general_purpose::STANDARD.encode(&combined));

        let mut entries = self.entries.lock().expect("vault lock poisoned");
        entries.insert(key_id.to_string(), encoded);
        *self.dirty.lock().expect("dirty lock poisoned") = true;

        Ok(combined)
    }

    /// Decrypt a value previously stored under `key_id`.
    ///
    /// Looks up the encrypted entry by `key_id` from the in-memory store.
    /// If not found, falls back to treating `data` as raw `nonce+ciphertext` bytes.
    pub fn decrypt(&self, data: &[u8], key_id: &str) -> L1Result<Vec<u8>> {
        let entries = self.entries.lock().expect("vault lock poisoned");

        let raw = match entries.get(key_id) {
            Some(stored) if stored.starts_with(ENC_PREFIX) => {
                let b64 = &stored[ENC_PREFIX.len()..];
                base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .map_err(|e| L1Error::Crypto(format!("Base64 decode failed: {}", e)))?
            }
            Some(_stored) => {
                return Err(L1Error::Crypto("Invalid entry format: missing enc: prefix".into()));
            }
            None => {
                // Fallback: treat data as raw nonce+ciphertext
                data.to_vec()
            }
        };

        if raw.len() < NONCE_LEN {
            return Err(L1Error::Crypto("Encrypted data too short".into()));
        }

        let (nonce_bytes, ciphertext) = raw.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| L1Error::Crypto(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Store a plaintext value by key_id (encrypted before storage).
    pub fn set(&self, key_id: &str, value: &str) -> L1Result<()> {
        self.encrypt(value.as_bytes(), key_id)?;
        Ok(())
    }

    /// Retrieve and decrypt a stored value by key_id.
    pub fn get(&self, key_id: &str) -> L1Result<Option<String>> {
        let entries = self.entries.lock().expect("vault lock poisoned");
        match entries.get(key_id) {
            Some(encoded) => {
                let raw = if encoded.starts_with(ENC_PREFIX) {
                    let b64 = &encoded[ENC_PREFIX.len()..];
                    base64::engine::general_purpose::STANDARD
                        .decode(b64)
                        .map_err(|e| L1Error::Crypto(format!("Base64 decode failed: {}", e)))?
                } else {
                    return Err(L1Error::Crypto("Invalid entry format: missing enc: prefix".into()));
                };

                if raw.len() < NONCE_LEN {
                    return Err(L1Error::Crypto("Encrypted data too short".into()));
                }
                let (nonce_bytes, ciphertext) = raw.split_at(NONCE_LEN);
                let nonce = Nonce::from_slice(nonce_bytes);
                let plaintext = self
                    .cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| L1Error::Crypto(format!("Decryption failed: {}", e)))?;
                let s = String::from_utf8(plaintext)
                    .map_err(|e| L1Error::Serde(e.to_string()))?;
                Ok(Some(s))
            }
            None => Ok(None),
        }
    }

    /// Delete a stored key.
    pub fn delete(&self, key_id: &str) -> L1Result<()> {
        let mut entries = self.entries.lock().expect("vault lock poisoned");
        entries.remove(key_id);
        *self.dirty.lock().expect("dirty lock poisoned") = true;
        Ok(())
    }

    /// List all stored key identifiers.
    pub fn list_keys(&self) -> Vec<String> {
        let entries = self.entries.lock().expect("vault lock poisoned");
        let mut keys: Vec<String> = entries.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Generate a new random 32-byte AES-256 key and return it as hex.
    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        key
    }

    /// Persist all encrypted entries to disk.
    pub fn save(&self) -> L1Result<()> {
        let mut dirty = self.dirty.lock().expect("dirty lock poisoned");
        if !*dirty {
            return Ok(());
        }
        let entries = self.entries.lock().expect("vault lock poisoned");
        let vf = VaultFile {
            entries: entries.clone(),
        };
        let json = serde_json::to_string_pretty(&vf)
            .map_err(|e| L1Error::Serde(e.to_string()))?;

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, json)?;
        *dirty = false;
        Ok(())
    }

    /// Return the source of the current master key.
    pub fn key_source(&self) -> KeySource {
        self.key_source
    }

    /// Check whether a string value is in `enc:` format.
    pub fn is_encrypted(value: &str) -> bool {
        value.starts_with(ENC_PREFIX)
    }

    /// Number of stored entries.
    pub fn len(&self) -> usize {
        let entries = self.entries.lock().expect("vault lock poisoned");
        entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for UnifiedVault {
    fn drop(&mut self) {
        if let Ok(dirty) = self.dirty.lock() {
            if *dirty {
                if let Err(e) = self.save() {
                    log::error!("[neotrix-unified-vault] failed to auto-save: {}", e);
                }
            }
        }
    }
}

// ---- Key loading (adapted from key_encryption.rs) ----

/// Load the 32-byte master key, returning it and the source description.
fn load_master_key() -> L1Result<([u8; 32], KeySource)> {
    #[cfg(feature = "keyring")]
    {
        match get_keyring_key() {
            Ok(key) => return Ok((key, KeySource::Keyring)),
            Err(e) => {
                eprintln!(
                    "[neotrix] OS keychain unavailable ({}); falling back to env var",
                    e
                );
            }
        }
    }

    if let Ok(key_str) = std::env::var(VAULT_KEY_ENV) {
        let trimmed = key_str.trim().to_string();
        if let Ok(decoded) = hex::decode(&trimmed) {
            if decoded.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&decoded);
                return Ok((key, KeySource::EnvVar));
            }
        }
        let hash = Sha256::digest(trimmed.as_bytes());
        return Ok((hash.into(), KeySource::EnvVar));
    }

    match derive_machine_key() {
        Ok(key) => Ok((key, KeySource::MachineDerived)),
        Err(e) => {
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            let hex_key = hex::encode(key);
            eprintln!("[neotrix-unified-vault] {} not set.", VAULT_KEY_ENV);
            eprintln!("[neotrix-unified-vault] No machine-derived key available ({}).", e);
            eprintln!("[neotrix-unified-vault] Generated new temporary key (save this!):");
            eprintln!("[neotrix-unified-vault]   export {}={}", VAULT_KEY_ENV, hex_key);
            Ok((key, KeySource::Generated))
        }
    }
}

#[cfg(feature = "keyring")]
fn get_keyring_key() -> L1Result<[u8; 32]> {
    use keyring::Entry;
    let entry = Entry::new("neotrix", "unified-vault-master")
        .map_err(|e| L1Error::Keyring(format!("keyring entry creation failed: {}", e)))?;

    match entry.get_password() {
        Ok(password) => {
            let trimmed = password.trim().to_string();
            if let Ok(decoded) = hex::decode(&trimmed) {
                if decoded.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&decoded);
                    return Ok(key);
                }
            }
            let hash = Sha256::digest(trimmed.as_bytes());
            Ok(hash.into())
        }
        Err(_) => {
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            let hex_key = hex::encode(key);
            entry
                .set_password(&hex_key)
                .map_err(|e| L1Error::Keyring(format!("failed to store key in OS keychain: {}", e)))?;
            Ok(key)
        }
    }
}

/// Deterministic per-machine key derived from the machine-id file or home directory.
fn derive_machine_key() -> L1Result<[u8; 32]> {
    let source = read_machine_id()
        .or_else(|| {
            dirs::home_dir()
                .map(|h| h.to_string_lossy().to_string())
        })
        .ok_or_else(|| L1Error::Config("Cannot derive machine key: no machine-id or home dir".into()))?;

    let hash = Sha256::digest(source.as_bytes());
    Ok(hash.into())
}

fn read_machine_id() -> Option<String> {
    for path in &["/etc/machine-id", "/var/lib/dbus/machine-id"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            let id = content.trim().to_string();
            if !id.is_empty() {
                return Some(id);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("IOPlatformUUID") {
                    if let Some(uuid) = line.split('"').nth(3) {
                        if !uuid.is_empty() {
                            return Some(uuid.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

// ---- Compatibility adapters ----

/// Adapter: use the unified vault through the `key_encryption`-style free-function API.
///
/// These functions preserve the `enc:<base64(nonce+ciphertext)>` format used by
/// `key_encryption.rs`, providing a drop-in compatible API that uses the same
/// master key resolution chain.
pub mod compat {
    use super::*;

    /// Check whether a value is in `enc:` encrypted format.
    pub fn is_encrypted(value: &str) -> bool {
        UnifiedVault::is_encrypted(value)
    }

    /// Encrypt a plaintext string.  Returns `enc:<base64(nonce+ciphertext)>`.
    ///
    /// No vault storage is involved — the returned string is self-contained and
    /// can be decrypted with [`decrypt`] using only the master key.
    pub fn encrypt(plaintext: &str) -> Result<String, String> {
        let key_bytes = load_master_key().map_err(|e| e.to_string())?.0;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| format!("AES-256-GCM init failed: {}", e))?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(format!("{}{}", ENC_PREFIX, base64::engine::general_purpose::STANDARD.encode(&combined)))
    }

    /// Decrypt a value previously produced by [`encrypt`].
    pub fn decrypt(encrypted: &str) -> Result<String, String> {
        if !is_encrypted(encrypted) {
            return Err("Value does not start with 'enc:' — not encrypted".into());
        }

        let raw = &encrypted[ENC_PREFIX.len()..];
        let data = base64::engine::general_purpose::STANDARD
            .decode(raw)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if data.len() < NONCE_LEN {
            return Err("Encrypted payload too short".into());
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key_bytes = load_master_key().map_err(|e| e.to_string())?.0;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| format!("AES-256-GCM init failed: {}", e))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_key_bytes() -> [u8; 32] {
        let mut key = [0u8; 32];
        key.copy_from_slice(
            &hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
                .expect("valid hex"),
        );
        key
    }

    fn make_vault(dir: &tempfile::TempDir) -> (UnifiedVault, PathBuf) {
        let path = dir.path().join("unified_vault.json");
        let key = test_key_bytes();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("AES init");

        let vault = UnifiedVault {
            path: path.clone(),
            cipher,
            key_source: KeySource::Generated,
            entries: Mutex::new(HashMap::new()),
            dirty: Mutex::new(false),
        };
        (vault, path)
    }

    #[test]
    fn test_set_get_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let (vault, _) = make_vault(&dir);
        vault.set("github_token", "ghp_test123").expect("set");
        let got = vault.get("github_token").expect("get");
        assert_eq!(got, Some("ghp_test123".to_string()));
    }

    #[test]
    fn test_get_missing_key() {
        let dir = tempdir().expect("tempdir");
        let (vault, _) = make_vault(&dir);
        let got = vault.get("nonexistent").expect("get");
        assert_eq!(got, None);
    }

    #[test]
    fn test_delete() {
        let dir = tempdir().expect("tempdir");
        let (vault, _) = make_vault(&dir);
        vault.set("a", "1").expect("set a");
        vault.set("b", "2").expect("set b");
        assert_eq!(vault.len(), 2);
        vault.delete("a").expect("delete a");
        assert_eq!(vault.len(), 1);
        assert_eq!(vault.get("a").expect("get a"), None);
        assert_eq!(vault.get("b").expect("get b"), Some("2".to_string()));
    }

    #[test]
    fn test_list_keys() {
        let dir = tempdir().expect("tempdir");
        let (vault, _) = make_vault(&dir);
        assert!(vault.list_keys().is_empty());
        vault.set("z", "1").expect("set z");
        vault.set("a", "2").expect("set a");
        let keys = vault.list_keys();
        assert_eq!(keys, vec!["a", "z"]);
    }

    #[test]
    fn test_save_and_reload() {
        let dir = tempdir().expect("tempdir");
        let (vault, path) = make_vault(&dir);
        vault.set("api_key", "sk-test").expect("set");
        vault.save().expect("save");
        assert!(!*vault.dirty.lock().expect("lock"));

        let key = test_key_bytes();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("AES init");
        let reloaded = UnifiedVault {
            path: path.clone(),
            cipher,
            key_source: KeySource::Generated,
            entries: Mutex::new(HashMap::new()),
            dirty: Mutex::new(false),
        };
        drop(vault);

        let got = reloaded.get("api_key").expect("get from reloaded");
        assert_eq!(got, Some("sk-test".to_string()));
    }

    #[test]
    fn test_generate_key_length() {
        let key = UnifiedVault::generate_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_is_encrypted() {
        assert!(!UnifiedVault::is_encrypted(""));
        assert!(!UnifiedVault::is_encrypted("sk-test123"));
        assert!(UnifiedVault::is_encrypted("enc:AAAA"));
        assert!(UnifiedVault::is_encrypted("enc:"));
    }

    #[test]
    fn test_is_empty() {
        let dir = tempdir().expect("tempdir");
        let (vault, _) = make_vault(&dir);
        assert!(vault.is_empty());
        vault.set("x", "y").expect("set");
        assert!(!vault.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_raw() {
        let dir = tempdir().expect("tempdir");
        let (vault, _) = make_vault(&dir);
        let data = b"hello unified vault";
        let ct = vault.encrypt(data, "test_key").expect("encrypt");
        assert_ne!(ct, data);
        let pt = vault.decrypt(&ct, "test_key").expect("decrypt");
        assert_eq!(pt, data);
    }

    #[test]
    fn test_compat_is_encrypted() {
        assert!(compat::is_encrypted("enc:AAAA"));
        assert!(!compat::is_encrypted("sk-raw"));
    }

    #[test]
    fn test_key_source_debug() {
        let variants = [
            KeySource::Keyring,
            KeySource::EnvVar,
            KeySource::MachineDerived,
            KeySource::Generated,
        ];
        for vs in &variants {
            let _s = format!("{:?}", vs);
        }
    }
}
