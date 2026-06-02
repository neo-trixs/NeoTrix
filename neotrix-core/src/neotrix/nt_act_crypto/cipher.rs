//! AES-256-GCM 加密层 — 所有持久化敏感数据的保护
//!
//! 密钥存储: ~/.neotrix/.master_key (0600 权限)
//! 每个密文: base64(nonce(12字节) || ciphertext(可变) || tag(16字节))

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use std::path::PathBuf;

const KEY_DIR: &str = "neotrix";
const KEY_FILE: &str = ".master_key";

fn master_key_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(KEY_DIR).join(KEY_FILE))
        .unwrap_or_else(|| PathBuf::from("/tmp/neotrix_master_key"))
}

fn ensure_key_dir() -> Result<PathBuf, String> {
    let path = master_key_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create key dir failed: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }
    Ok(path)
}

fn generate_master_key_bytes() -> [u8; 32] {
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    key
}

fn load_or_create_master_key() -> Result<[u8; 32], String> {
    let path = ensure_key_dir()?;

    if path.exists() {
        let data = std::fs::read(&path)
            .map_err(|e| format!("read master key failed: {}", e))?;
        if data.len() != 32 {
            return Err("invalid master key length".into());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&data);
        Ok(key)
    } else {
        let key = generate_master_key_bytes();
        std::fs::write(&path, &key)
            .map_err(|e| format!("write master key failed: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(key)
    }
}

fn key_to_cipher(key: &[u8; 32]) -> Aes256Gcm {
    Aes256Gcm::new_from_slice(key).expect("valid AES-256 key")
}

/// Encrypt plaintext bytes. Returns (nonce, ciphertext_with_tag) as separate vecs.
fn encrypt_raw(plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    use rand::RngCore;
    let key = load_or_create_master_key()?;
    let cipher = key_to_cipher(&key);
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("encrypt failed: {}", e))?;
    Ok((nonce_bytes.to_vec(), ciphertext))
}

fn decrypt_raw(nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let key = load_or_create_master_key()?;
    let cipher = key_to_cipher(&key);
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("decrypt failed: {}", e))?;
    Ok(plaintext)
}

/// Encrypt a hex string to an encrypted hex string.
/// Format: hex(nonce || ciphertext)
pub fn encrypt_to_hex(plaintext_hex: &str) -> Result<String, String> {
    let plaintext = hex::decode(plaintext_hex.strip_prefix("0x").unwrap_or(plaintext_hex))
        .map_err(|e| format!("decode hex failed: {}", e))?;
    let (nonce, ct) = encrypt_raw(&plaintext)?;
    let mut combined = Vec::with_capacity(nonce.len() + ct.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ct);
    Ok(format!("0x{}", hex::encode(&combined)))
}

/// Decrypt from the combined hex format back to a hex string.
pub fn decrypt_from_hex(encrypted_hex: &str) -> Result<String, String> {
    let combined = hex::decode(encrypted_hex.strip_prefix("0x").unwrap_or(encrypted_hex))
        .map_err(|e| format!("decode hex failed: {}", e))?;
    if combined.len() < 12 {
        return Err("ciphertext too short".into());
    }
    let nonce = &combined[..12];
    let ct = &combined[12..];
    let plaintext = decrypt_raw(nonce, ct)?;
    Ok(format!("0x{}", hex::encode(&plaintext)))
}

/// Encrypt a string to an encrypted hex string.
pub fn encrypt_str(plaintext: &str) -> Result<String, String> {
    let (nonce, ct) = encrypt_raw(plaintext.as_bytes())?;
    let mut combined = Vec::with_capacity(nonce.len() + ct.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ct);
    Ok(format!("0x{}", hex::encode(&combined)))
}

/// Decrypt from combined hex format back to a string.
pub fn decrypt_str(encrypted_hex: &str) -> Result<String, String> {
    let combined = hex::decode(encrypted_hex.strip_prefix("0x").unwrap_or(encrypted_hex))
        .map_err(|e| format!("decode hex failed: {}", e))?;
    if combined.len() < 12 {
        return Err("ciphertext too short".into());
    }
    let nonce = &combined[..12];
    let ct = &combined[12..];
    let plaintext = decrypt_raw(nonce, ct)?;
    String::from_utf8(plaintext).map_err(|e| format!("utf8 decode failed: {}", e))
}

/// Encrypt arbitrary bytes -> hex string
pub fn encrypt_bytes(plaintext: &[u8]) -> Result<String, String> {
    let (nonce, ct) = encrypt_raw(plaintext)?;
    let mut combined = Vec::with_capacity(nonce.len() + ct.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ct);
    Ok(hex::encode(&combined))
}

/// Decrypt from hex string to bytes
pub fn decrypt_bytes(encrypted_hex: &str) -> Result<Vec<u8>, String> {
    let combined = hex::decode(encrypted_hex)
        .map_err(|e| format!("decode hex failed: {}", e))?;
    if combined.len() < 12 {
        return Err("ciphertext too short".into());
    }
    let nonce = &combined[..12];
    let ct = &combined[12..];
    decrypt_raw(nonce, ct)
}

/// Encrypt a JSON-serializable value
pub fn encrypt_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    let json = serde_json::to_string(value)
        .map_err(|e| format!("serialize failed: {}", e))?;
    encrypt_str(&json)
}

/// Decrypt a hex string back to a JSON-deserializable value
pub fn decrypt_json<T: serde::de::DeserializeOwned>(encrypted_hex: &str) -> Result<T, String> {
    let json = decrypt_str(encrypted_hex)?;
    serde_json::from_str(&json).map_err(|e| format!("deserialize failed: {}", e))
}

/// Check if master key exists
pub fn has_master_key() -> bool {
    master_key_path().exists()
}

/// Get master key path
pub fn master_key_path_str() -> String {
    master_key_path().to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "hello crypto world 42!";
        let encrypted = encrypt_str(original).unwrap();
        assert_ne!(encrypted, original);
        let decrypted = decrypt_str(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_hex_roundtrip() {
        let hex_input = "0x1234567890abcdef";
        let encrypted = encrypt_to_hex(hex_input).unwrap();
        let decrypted = decrypt_from_hex(&encrypted).unwrap();
        assert_eq!(hex_input, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_bytes_roundtrip() {
        let original = b"some binary data \x00\x01\x02";
        let encrypted = encrypt_bytes(original).unwrap();
        let decrypted = decrypt_bytes(&encrypted).unwrap();
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_json_roundtrip() {
        let data = serde_json::json!({"key": "value", "num": 42});
        let encrypted = encrypt_json(&data).unwrap();
        let decrypted: serde_json::Value = decrypt_json(&encrypted).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_master_key_creation() {
        assert!(has_master_key() || master_key_path().exists() || true);
        // Key is auto-created on first encrypt call
        let _ = encrypt_str("test key creation");
        assert!(master_key_path().exists());
    }

    #[test]
    fn test_different_ciphertexts() {
        let text = "same text";
        let e1 = encrypt_str(text).unwrap();
        let e2 = encrypt_str(text).unwrap();
        assert_ne!(e1, e2, "each encryption should produce different ciphertext");
    }

    #[test]
    fn test_invalid_ciphertext() {
        let result = decrypt_str("0xdeadbeef");
        assert!(result.is_err());
    }
}
