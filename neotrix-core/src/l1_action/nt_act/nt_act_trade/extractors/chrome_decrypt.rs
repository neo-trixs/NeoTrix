#![forbid(unsafe_code)]

//! Chrome password decryption for macOS.
//!
//! Decrypts Chrome Login Data credentials using the macOS Keychain-derived
//! AES-128-CBC key. Reads directly from the SQLite `Login Data` database.
//!
//! Algorithm:
//! ```text
//! keychain_password = Keychain::get("Chrome Safe Storage", "Chrome")
//! derived_key       = PBKDF2-HMAC-SHA1(keychain_password, b"saltysalt", 1003, 16)
//! iv                = [b' '; 16]
//! plaintext         = AES-128-CBC-Decrypt(derived_key, iv, encrypted_data[3..])
//! ```

use aes::Aes128;
use cbc::Decryptor;
use cipher::{BlockDecryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rusqlite::Connection;
use sha1::Sha1;
use std::fs;
use std::path::{Path, PathBuf};

type Aes128CbcDec = Decryptor<Aes128>;

const PBKDF2_ITERATIONS: u32 = 1003;
const PBKDF2_KEY_LEN: usize = 16;
const SALT: &[u8] = b"saltysalt";
const KEYCHAIN_SERVICE: &str = "Chrome Safe Storage";
const KEYCHAIN_ACCOUNT: &str = "Chrome";
const V10: &[u8] = b"v10";
const V11: &[u8] = b"v11";
const DEFAULT_LOGIN_DATA: &str = "~/Library/Application Support/Google/Chrome/Default/Login Data";

/// Error type for Chrome decryption operations.
#[derive(Debug)]
pub enum ChromeDecryptError {
    /// macOS Keychain access failed.
    Keychain(String),
    /// PBKDF2 key derivation failed.
    KeyDerivation(String),
    /// AES-128-CBC decryption failed.
    Decryption(String),
    /// SQLite database error.
    Database(String),
    /// Encryption prefix not recognized (expected v10 or v11).
    UnknownPrefix(String),
    /// IO error.
    Io(String),
}

impl std::fmt::Display for ChromeDecryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keychain(e) => write!(f, "Keychain error: {e}"),
            Self::KeyDerivation(e) => write!(f, "Key derivation error: {e}"),
            Self::Decryption(e) => write!(f, "Decryption error: {e}"),
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::UnknownPrefix(e) => write!(f, "Unknown encryption prefix: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for ChromeDecryptError {}

impl From<rusqlite::Error> for ChromeDecryptError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<std::io::Error> for ChromeDecryptError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// A single decrypted login entry from Chrome.
#[derive(Debug, Clone)]
pub struct LoginEntry {
    pub origin_url: String,
    pub username: String,
    pub password: String,
}

/// Chrome password decryptor backed by macOS Keychain.
///
/// # Example
/// ```no_run
/// use neotrix::l1_action::nt_act::nt_act_trade::extractors::ChromeDecryptor;
///
/// let decryptor = ChromeDecryptor::new().expect("Keychain access");
/// let entries = decryptor.decrypt_login_data(None).expect("decrypt");
/// for e in &entries {
///     println!("{}: {} / {}", e.origin_url, e.username, e.password);
/// }
/// ```
pub struct ChromeDecryptor {
    derived_key: [u8; PBKDF2_KEY_LEN],
}

impl ChromeDecryptor {
    /// Create a new decryptor by reading the Keychain password and deriving the AES key.
    pub fn new() -> Result<Self, ChromeDecryptError> {
        let keychain_password = Self::get_keychain_password()?;
        let derived_key = Self::derive_key(&keychain_password)?;
        Ok(Self { derived_key })
    }

    /// Create a decryptor with an explicit keychain password (for testing).
    pub fn with_password(keychain_password: &str) -> Result<Self, ChromeDecryptError> {
        let derived_key = Self::derive_key(keychain_password)?;
        Ok(Self { derived_key })
    }

    /// Decrypt Chrome-encrypted bytes (must start with `v10` or `v11` prefix).
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, ChromeDecryptError> {
        let ciphertext = Self::strip_prefix(encrypted)?;
        let iv = [b' '; 16];
        let mut buf = ciphertext.to_vec();

        let decryptor = Aes128CbcDec::new((&self.derived_key).into(), &iv.into());
        decryptor
            .decrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut buf)
            .map_err(|e| ChromeDecryptError::Decryption(e.to_string()))?;

        // Strip PKCS#7 padding (Chrome always uses full blocks, but trim trailing zeros)
        let len = buf.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
        String::from_utf8(buf[..len].to_vec())
            .map_err(|e| ChromeDecryptError::Decryption(e.to_string()))
    }

    /// Decrypt all logins from a Chrome Login Data SQLite database.
    ///
    /// If `db_path` is `None`, uses the default Chrome profile path.
    /// The database is copied to a temp file to avoid lock conflicts.
    pub fn decrypt_login_data(
        &self,
        db_path: Option<&Path>,
    ) -> Result<Vec<LoginEntry>, ChromeDecryptError> {
        let path = match db_path {
            Some(p) => p.to_path_buf(),
            None => {
                let expanded = shellexpand::tilde(DEFAULT_LOGIN_DATA);
                PathBuf::from(expanded.as_ref())
            }
        };

        let tmp =
            tempfile::NamedTempFile::new().map_err(|e| ChromeDecryptError::Io(e.to_string()))?;
        fs::copy(&path, tmp.path())?;

        let conn = Connection::open(tmp.path())?;
        let mut stmt =
            conn.prepare("SELECT origin_url, username_value, password_value FROM logins")?;

        let entries: Vec<LoginEntry> = stmt
            .query_map([], |row| {
                let origin: String = row.get(0)?;
                let username: String = row.get(1)?;
                let encrypted_password: Vec<u8> = row.get(2)?;
                Ok((origin, username, encrypted_password))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(origin, username, enc)| {
                self.decrypt(&enc).ok().map(|password| LoginEntry {
                    origin_url: origin,
                    username,
                    password,
                })
            })
            .collect();

        Ok(entries)
    }

    /// Retrieve the Chrome Safe Storage password from the macOS Keychain.
    fn get_keychain_password() -> Result<String, ChromeDecryptError> {
        let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
            .map_err(|e| ChromeDecryptError::Keychain(e.to_string()))?;
        entry
            .get_password()
            .map_err(|e| ChromeDecryptError::Keychain(e.to_string()))
    }

    /// Derive an AES-128 key using PBKDF2-HMAC-SHA1.
    fn derive_key(password: &str) -> Result<[u8; PBKDF2_KEY_LEN], ChromeDecryptError> {
        let mut key = [0u8; PBKDF2_KEY_LEN];
        pbkdf2_hmac::<Sha1>(password.as_bytes(), SALT, PBKDF2_ITERATIONS, &mut key);
        Ok(key)
    }

    /// Strip the `v10` or `v11` prefix from Chrome encrypted data.
    fn strip_prefix(data: &[u8]) -> Result<&[u8], ChromeDecryptError> {
        if data.starts_with(V10) || data.starts_with(V11) {
            Ok(&data[3..])
        } else {
            Err(ChromeDecryptError::UnknownPrefix(format!(
                "expected v10/v11 prefix, got {:?}",
                &data[..data.len().min(3)]
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decrypt_compiles_with_explicit_password() {
        let dec = ChromeDecryptor::with_password("peanuts");
        assert!(dec.is_ok());

        let dec = dec.unwrap();
        let result = dec.decrypt(b"v10test");
        assert!(result.is_ok());
    }

    #[test]
    fn strip_prefix_v10() {
        let inner = ChromeDecryptor::strip_prefix(b"v10abc").unwrap();
        assert_eq!(inner, b"abc");
    }

    #[test]
    fn strip_prefix_v11() {
        let inner = ChromeDecryptor::strip_prefix(b"v11abc").unwrap();
        assert_eq!(inner, b"abc");
    }

    #[test]
    fn strip_prefix_unknown() {
        let err = ChromeDecryptor::strip_prefix(b"v20abc");
        assert!(err.is_err());
    }
}
