use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

use base64::Engine;
use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};

const STORAGE_PATH: &str = "~/.neotrix/secrets.json";
const NONCE_LEN: usize = 12;
const MASTER_KEY_ENV: &str = "NEOTRIX_KEYVAULT_KEY";

fn load_master_key() -> NeoTrixResult<[u8; 32]> {
    match std::env::var(MASTER_KEY_ENV) {
        Ok(key_str) => {
            let key_str = key_str.trim().to_string();
            if let Ok(decoded) = hex::decode(&key_str) {
                if decoded.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&decoded);
                    return Ok(key);
                }
            }
            let hash = Sha256::digest(key_str.as_bytes());
            Ok(hash.into())
        }
        Err(_) => {
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            let hex_key = hex::encode(key);
            eprintln!("[neotrix-keyvault] {} not set.", MASTER_KEY_ENV);
            eprintln!("[neotrix-keyvault] Generated new master key (save this!):");
            eprintln!("[neotrix-keyvault]   export {}={}", MASTER_KEY_ENV, hex_key);
            Ok(key)
        }
    }
}

fn encrypt_value(cipher: &Aes256Gcm, plaintext: &str) -> NeoTrixResult<String> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| NeoTrixError::General { msg: format!("Encryption failed: {}", e), backtrace: None })?;
    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

fn decrypt_value(cipher: &Aes256Gcm, encoded: &str) -> NeoTrixResult<String> {
    let data = base64::engine::general_purpose::STANDARD.decode(encoded)
        .map_err(|e| NeoTrixError::General { msg: format!("Base64 decode failed: {}", e), backtrace: None })?;
    if data.len() < NONCE_LEN {
        return Err(NeoTrixError::from("Encrypted data too short"));
    }
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| NeoTrixError::General { msg: format!("Decryption failed: {}", e), backtrace: None })?;
    String::from_utf8(plaintext)
        .map_err(|e| NeoTrixError::Serde(e.to_string()))
}

struct LocalStore {
    path: PathBuf,
    cipher: Aes256Gcm,
    entries: RefCell<HashMap<String, String>>,
    dirty: RefCell<bool>,
}

impl LocalStore {
    fn new() -> NeoTrixResult<Self> {
        let path = shellexpand::tilde(STORAGE_PATH).parse::<PathBuf>()
            .map_err(|_| NeoTrixError::Config("Failed to expand keyvault path".into()))?;
        let key_bytes = load_master_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| NeoTrixError::Config(format!("AES key init error: {}", e)))?;
        let entries = if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str(&data)
                .map_err(|e| NeoTrixError::Serde(e.to_string()))?
        } else {
            HashMap::new()
        };
        Ok(Self { path, cipher, entries: RefCell::new(entries), dirty: RefCell::new(false) })
    }

    #[cfg(test)]
    fn new_with_path(path: PathBuf, key: &[u8; 32]) -> NeoTrixResult<Self> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| NeoTrixError::Config(format!("AES key init error: {}", e)))?;
        let entries = if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str(&data)
                .map_err(|e| NeoTrixError::Serde(e.to_string()))?
        } else {
            HashMap::new()
        };
        Ok(Self { path, cipher, entries: RefCell::new(entries), dirty: RefCell::new(false) })
    }

    fn set(&self, key: &str, value: &str) -> NeoTrixResult<()> {
        let encrypted = encrypt_value(&self.cipher, value)?;
        let mut entries = self.entries.borrow_mut();
        entries.insert(key.to_string(), encrypted);
        *self.dirty.borrow_mut() = true;
        Ok(())
    }

    fn get(&self, key: &str) -> NeoTrixResult<Option<String>> {
        let entries = self.entries.borrow();
        match entries.get(key) {
            Some(encoded) => Ok(Some(decrypt_value(&self.cipher, encoded)?)),
            None => Ok(None),
        }
    }

    fn delete(&self, key: &str) -> NeoTrixResult<()> {
        let mut entries = self.entries.borrow_mut();
        entries.remove(key);
        *self.dirty.borrow_mut() = true;
        Ok(())
    }

    fn save(&self) -> NeoTrixResult<()> {
        if !*self.dirty.borrow() {
            return Ok(());
        }
        let entries = self.entries.borrow();
        let json = serde_json::to_string(&*entries)
            .map_err(|e| NeoTrixError::Serde(e.to_string()))?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, json)?;
        *self.dirty.borrow_mut() = false;
        Ok(())
    }

    fn list(&self) -> Vec<String> {
        self.entries.borrow().keys().cloned().collect()
    }
}

#[cfg(feature = "keyring")]
struct KeyringStore {
    service_name: String,
}

#[cfg(feature = "keyring")]
impl KeyringStore {
    fn new(service_name: &str) -> Self {
        Self { service_name: service_name.to_string() }
    }

    fn set(&self, key: &str, value: &str) -> NeoTrixResult<()> {
        let entry = keyring::Entry::new(&self.service_name, key)
            .map_err(|e| NeoTrixError::General(format!("Keyring entry error: {}", e)))?;
        entry.set_password(value)
            .map_err(|e| NeoTrixError::General(format!("Keyring set failed: {}", e)))?;
        Ok(())
    }

    fn get(&self, key: &str) -> NeoTrixResult<Option<String>> {
        let entry = match keyring::Entry::new(&self.service_name, key) {
            Ok(e) => e,
            Err(_) => return Ok(None),
        };
        match entry.get_password() {
            Ok(val) => Ok(Some(val)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(NeoTrixError::General(format!("Keyring get failed: {}", e))),
        }
    }

    fn delete(&self, key: &str) -> NeoTrixResult<()> {
        let entry = keyring::Entry::new(&self.service_name, key)
            .map_err(|e| NeoTrixError::General(format!("Keyring entry error: {}", e)))?;
        entry.delete_password()
            .map_err(|e| NeoTrixError::General(format!("Keyring delete failed: {}", e)))?;
        Ok(())
    }
}

pub struct KeyVault {
    local_store: LocalStore,
    #[cfg(feature = "keyring")]
    keyring: KeyringStore,
}

impl KeyVault {
    pub fn new(_service_name: &str) -> NeoTrixResult<Self> {
        let local_store = LocalStore::new()?;
        Ok(Self {
            local_store,
            #[cfg(feature = "keyring")]
            keyring: KeyringStore::new(service_name),
        })
    }

    pub fn set(&self, key: &str, value: &str) -> NeoTrixResult<()> {
        self.local_store.set(key, value)?;
        self.local_store.save()?;
        #[cfg(feature = "keyring")]
        self.keyring.set(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> NeoTrixResult<Option<String>> {
        #[cfg(feature = "keyring")]
        {
            if let Some(val) = self.keyring.get(key)? {
                return Ok(Some(val));
            }
        }
        self.local_store.get(key)
    }

    pub fn delete(&self, key: &str) -> NeoTrixResult<()> {
        self.local_store.delete(key)?;
        self.local_store.save()?;
        #[cfg(feature = "keyring")]
        self.keyring.delete(key)?;
        Ok(())
    }

    pub fn list(&self) -> NeoTrixResult<Vec<String>> {
        Ok(self.local_store.list())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        key.copy_from_slice(
            &hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
                .expect("valid hex test key"),
        );
        key
    }

    #[test]
    fn test_local_store_set_get() {
        let dir = tempdir().expect("tempdir creation");
        let path = dir.path().join("secrets.json");
        let key = test_key();
        let store = LocalStore::new_with_path(path.clone(), &key).expect("new_with_path");

        store.set("api_key", "sk-test123").expect("store.set");
        assert_eq!(store.get("api_key").expect("store.get"), Some("sk-test123".to_string()));

        store.save().expect("store.save");

        let store2 = LocalStore::new_with_path(path, &key).expect("new_with_path");
        assert_eq!(store2.get("api_key").expect("store.get"), Some("sk-test123".to_string()));
    }

    #[test]
    fn test_local_store_get_missing() {
        let dir = tempdir().expect("tempdir creation");
        let path = dir.path().join("secrets.json");
        let key = test_key();
        let store = LocalStore::new_with_path(path, &key).expect("new_with_path");

        assert_eq!(store.get("nonexistent").expect("store.get"), None);
    }

    #[test]
    fn test_local_store_delete() {
        let dir = tempdir().expect("tempdir creation");
        let path = dir.path().join("secrets.json");
        let key = test_key();
        let store = LocalStore::new_with_path(path, &key).expect("new_with_path");

        store.set("a", "1").expect("store.set a");
        store.set("b", "2").expect("store.set b");
        assert_eq!(store.list().len(), 2);

        store.delete("a").expect("store.delete a");
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.get("a").expect("store.get a"), None);
        assert_eq!(store.get("b").expect("store.get b"), Some("2".to_string()));
    }

    #[test]
    fn test_local_store_list() {
        let dir = tempdir().expect("tempdir creation");
        let path = dir.path().join("secrets.json");
        let key = test_key();
        let store = LocalStore::new_with_path(path, &key).expect("new_with_path");

        assert!(store.list().is_empty());

        store.set("alpha", "x").expect("store.set alpha");
        store.set("beta", "y").expect("store.set beta");
        let mut keys = store.list();
        keys.sort();
        assert_eq!(keys, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_local_store_empty() {
        let dir = tempdir().expect("tempdir creation");
        let path = dir.path().join("secrets.json");
        let key = test_key();
        let store = LocalStore::new_with_path(path, &key).expect("new_with_path");

        assert!(store.list().is_empty());
        assert_eq!(store.get("anything").expect("store.get"), None);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("AES key init");
        let original = "sensitive-data-42";

        let encrypted = encrypt_value(&cipher, original).expect("encrypt_value");
        assert_ne!(encrypted, original);

        let decrypted = decrypt_value(&cipher, &encrypted).expect("decrypt_value");
        assert_eq!(decrypted, original);
    }
}
