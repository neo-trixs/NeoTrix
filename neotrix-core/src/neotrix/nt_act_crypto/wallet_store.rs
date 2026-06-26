use super::cipher;
use super::wallet::{CryptoWallet, WalletManager};
use crate::core::nt_core_util;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const WALLET_DIR: &str = "neotrix/wallets";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WalletFile {
    pub chain: String,
    pub address: String,
    pub label: String,
    pub encrypted_key: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&CryptoWallet> for WalletFile {
    fn from(w: &CryptoWallet) -> Self {
        let pk = w.private_key_hex();
        let encrypted_key = cipher::encrypt_to_hex(&pk).unwrap_or_else(|_| pk.clone());

        Self {
            chain: w.chain.to_string(),
            address: w.address.clone(),
            label: w.label.clone(),
            encrypted_key,
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            updated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        }
    }
}

pub struct WalletStore {
    dir: PathBuf,
}

impl WalletStore {
    pub fn new() -> Self {
        let dir = Self::wallet_dir();
        let _ = std::fs::create_dir_all(&dir);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
        Self { dir }
    }

    pub fn with_dir(dir: &str) -> Self {
        let path = PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&path);
        Self { dir: path }
    }

    fn wallet_dir() -> PathBuf {
        dirs::home_dir()
            .map(|h| h.join(WALLET_DIR))
            .unwrap_or_else(|| nt_core_util::home_dir().join(".neotrix").join("wallets"))
    }

    pub fn wallet_path(&self, label: &str) -> PathBuf {
        let safe_name = label.replace(' ', "_").replace('/', "_");
        self.dir.join(format!("{}.json", safe_name))
    }

    pub fn save_wallet(&self, wallet: &CryptoWallet) -> Result<String, String> {
        let file: WalletFile = wallet.into();
        let path = self.wallet_path(&wallet.label);
        let json = serde_json::to_string_pretty(&file).map_err(|e| format!("serialize: {}", e))?;
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, &json).map_err(|e| format!("write: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
        }
        std::fs::rename(&tmp_path, &path).map_err(|e| format!("rename: {}", e))?;
        Ok(wallet.label.clone())
    }

    pub fn load_wallet(&self, label: &str) -> Result<CryptoWallet, String> {
        let path = self.wallet_path(label);
        if !path.exists() {
            return Err(format!("wallet '{}' not found", label));
        }
        let json = std::fs::read_to_string(&path).map_err(|e| format!("read: {}", e))?;
        let file: WalletFile = serde_json::from_str(&json).map_err(|e| format!("parse: {}", e))?;

        let pk = if file.encrypted_key.starts_with("0x") && file.encrypted_key.len() > 42 {
            cipher::decrypt_from_hex(&file.encrypted_key).map_err(|e| format!("decrypt: {}", e))?
        } else {
            file.encrypted_key.clone()
        };

        let wallet = CryptoWallet::import_evm(&pk, &file.label)?;
        Ok(wallet)
    }

    pub fn list_wallets(&self) -> Result<Vec<WalletInfo>, String> {
        let mut wallets = Vec::new();
        if !self.dir.exists() {
            return Ok(wallets);
        }
        let entries = std::fs::read_dir(&self.dir).map_err(|e| format!("read dir: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("entry: {}", e))?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(file) = serde_json::from_str::<WalletFile>(&json) {
                        wallets.push(WalletInfo {
                            label: file.label,
                            address: file.address,
                            chain: file.chain,
                            created_at: file.created_at,
                            encrypted: file.encrypted_key.starts_with("0x")
                                && file.encrypted_key.len() > 42,
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
        wallets.sort_by(|a, b| a.label.cmp(&b.label));
        Ok(wallets)
    }

    pub fn delete_wallet(&self, label: &str) -> Result<(), String> {
        let path = self.wallet_path(label);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("delete: {}", e))
        } else {
            Err(format!("wallet '{}' not found", label))
        }
    }

    pub fn wallet_exists(&self, label: &str) -> bool {
        self.wallet_path(label).exists()
    }

    pub fn load_all(&self) -> Result<WalletManager, String> {
        let mut manager = WalletManager::new();
        let wallets = self.list_wallets()?;
        for info in &wallets {
            if let Ok(wallet) = self.load_wallet(&info.label) {
                manager.add_wallet(wallet);
            }
        }
        Ok(manager)
    }

    pub fn dir_path(&self) -> &Path {
        &self.dir
    }

    pub fn wallet_count(&self) -> usize {
        self.list_wallets().map(|w| w.len()).unwrap_or(0)
    }
}

impl Default for WalletStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct WalletInfo {
    pub label: String,
    pub address: String,
    pub chain: String,
    pub created_at: String,
    pub encrypted: bool,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_store() -> (WalletStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        let store = WalletStore::with_dir(tmp.path().to_str().unwrap());
        (store, tmp)
    }

    #[test]
    fn test_save_and_load_wallet() {
        let (store, _tmp) = test_store();
        let wallet = CryptoWallet::generate_evm("test-wallet");
        store.save_wallet(&wallet).unwrap();
        let loaded = store.load_wallet("test-wallet").unwrap();
        assert_eq!(wallet.address, loaded.address);
        assert_eq!(wallet.private_key_hex(), loaded.private_key_hex());
    }

    #[test]
    fn test_list_wallets() {
        let (store, _tmp) = test_store();
        let w1 = CryptoWallet::generate_evm("wallet-a");
        let w2 = CryptoWallet::generate_evm("wallet-b");
        store.save_wallet(&w1).unwrap();
        store.save_wallet(&w2).unwrap();
        let list = store.list_wallets().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_delete_wallet() {
        let (store, _tmp) = test_store();
        let wallet = CryptoWallet::generate_evm("delete-me");
        store.save_wallet(&wallet).unwrap();
        assert!(store.wallet_exists("delete-me"));
        store.delete_wallet("delete-me").unwrap();
        assert!(!store.wallet_exists("delete-me"));
    }

    #[test]
    fn test_load_all() {
        let (store, _tmp) = test_store();
        store
            .save_wallet(&CryptoWallet::generate_evm("w1"))
            .unwrap();
        store
            .save_wallet(&CryptoWallet::generate_evm("w2"))
            .unwrap();
        let manager = store.load_all().unwrap();
        assert_eq!(manager.wallet_count(), 2);
    }

    #[test]
    fn test_encrypted_storage() {
        let (store, _tmp) = test_store();
        let wallet = CryptoWallet::generate_evm("encrypted-test");
        store.save_wallet(&wallet).unwrap();
        let list = store.list_wallets().unwrap();
        assert!(list[0].encrypted, "private key should be encrypted");
        let loaded = store.load_wallet("encrypted-test").unwrap();
        assert_eq!(wallet.address, loaded.address);
    }

    #[test]
    fn test_nonexistent_wallet() {
        let (store, _tmp) = test_store();
        let result = store.load_wallet("nonexistent");
        assert!(result.is_err());
    }
}
