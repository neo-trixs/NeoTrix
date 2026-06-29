use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use sha3::{Digest, Keccak256};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WalletChain {
    Evm,
    Solana,
    Bitcoin,
}

impl fmt::Display for WalletChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletChain::Evm => write!(f, "evm"),
            WalletChain::Solana => write!(f, "solana"),
            WalletChain::Bitcoin => write!(f, "bitcoin"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CryptoWallet {
    pub chain: WalletChain,
    pub address: String,
    pub label: String,
    key_bytes: Vec<u8>,
}

impl CryptoWallet {
    pub fn generate_evm(label: &str) -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let encoded = verifying_key.to_encoded_point(false);
        let public_key = encoded.as_bytes();
        let hash = Keccak256::digest(&public_key[1..]);
        let address = format!("0x{}", hex::encode(&hash[12..]));

        Self {
            chain: WalletChain::Evm,
            address,
            label: label.to_string(),
            key_bytes: signing_key.to_bytes().to_vec(),
        }
    }

    pub fn import_evm(private_key_hex: &str, label: &str) -> Result<Self, String> {
        let hex_str = private_key_hex
            .strip_prefix("0x")
            .unwrap_or(private_key_hex);
        let bytes = hex::decode(hex_str).map_err(|e| format!("invalid hex: {}", e))?;
        let signing_key =
            SigningKey::from_slice(&bytes).map_err(|e| format!("invalid key: {}", e))?;
        let verifying_key = signing_key.verifying_key();
        let encoded = verifying_key.to_encoded_point(false);
        let public_key = encoded.as_bytes();
        let hash = Keccak256::digest(&public_key[1..]);
        let address = format!("0x{}", hex::encode(&hash[12..]));

        Ok(Self {
            chain: WalletChain::Evm,
            address,
            label: label.to_string(),
            key_bytes: signing_key.to_bytes().to_vec(),
        })
    }

    pub fn private_key_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.key_bytes))
    }

    pub fn private_key_bytes(&self) -> &[u8] {
        &self.key_bytes
    }

    pub fn address_short(&self) -> String {
        if self.address.len() > 10 {
            format!(
                "{}...{}",
                &self.address[..6],
                &self.address[self.address.len() - 4..]
            )
        } else {
            self.address.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct WalletManager {
    wallets: Vec<CryptoWallet>,
    active: Option<usize>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            wallets: Vec::new(),
            active: None,
        }
    }

    pub fn add_wallet(&mut self, wallet: CryptoWallet) {
        self.active = Some(self.wallets.len());
        self.wallets.push(wallet);
    }

    pub fn active_wallet(&self) -> Option<&CryptoWallet> {
        self.active.and_then(|i| self.wallets.get(i))
    }

    pub fn active_wallet_mut(&mut self) -> Option<&mut CryptoWallet> {
        self.active.and_then(|i| self.wallets.get_mut(i))
    }

    pub fn set_active(&mut self, index: usize) -> Result<(), String> {
        if index < self.wallets.len() {
            self.active = Some(index);
            Ok(())
        } else {
            Err(format!(
                "wallet index {} out of bounds (max {})",
                index,
                self.wallets.len().saturating_sub(1)
            ))
        }
    }

    pub fn wallets(&self) -> &[CryptoWallet] {
        &self.wallets
    }

    pub fn wallet_count(&self) -> usize {
        self.wallets.len()
    }

    pub fn remove_wallet(&mut self, index: usize) -> Option<CryptoWallet> {
        if index < self.wallets.len() {
            let w = self.wallets.remove(index);
            if self.active == Some(index) {
                self.active = if self.wallets.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
            Some(w)
        } else {
            None
        }
    }

    pub fn total_balance_usd(&self) -> f64 {
        0.0
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_evm_wallet() {
        let wallet = CryptoWallet::generate_evm("test");
        assert_eq!(wallet.chain, WalletChain::Evm);
        assert!(wallet.address.starts_with("0x"));
        assert_eq!(wallet.address.len(), 42);
        assert_eq!(wallet.label, "test");
    }

    #[test]
    fn test_import_evm_wallet() {
        let wallet = CryptoWallet::generate_evm("original");
        let pk = wallet.private_key_hex();
        let imported = CryptoWallet::import_evm(&pk, "imported").unwrap();
        assert_eq!(wallet.address, imported.address);
        assert_eq!(imported.label, "imported");
    }

    #[test]
    fn test_wallet_manager() {
        let mut mgr = WalletManager::new();
        assert_eq!(mgr.wallet_count(), 0);
        mgr.add_wallet(CryptoWallet::generate_evm("wallet-1"));
        mgr.add_wallet(CryptoWallet::generate_evm("wallet-2"));
        assert_eq!(mgr.wallet_count(), 2);
        assert!(mgr.active_wallet().is_some());
        assert_eq!(mgr.active_wallet().unwrap().label, "wallet-2");
        mgr.set_active(0).unwrap();
        assert_eq!(mgr.active_wallet().unwrap().label, "wallet-1");
    }

    #[test]
    fn test_remove_wallet() {
        let mut mgr = WalletManager::new();
        mgr.add_wallet(CryptoWallet::generate_evm("w1"));
        mgr.add_wallet(CryptoWallet::generate_evm("w2"));
        assert!(mgr.remove_wallet(0).is_some());
        assert_eq!(mgr.wallet_count(), 1);
    }

    #[test]
    fn test_address_short() {
        let wallet = CryptoWallet::generate_evm("test");
        let short = wallet.address_short();
        assert!(short.contains("..."));
        assert!(short.len() < wallet.address.len());
    }
}
