use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

const ENC_PREFIX: &str = "enc:";
const NONCE_LEN: usize = 12;
#[cfg_attr(not(feature = "keyring"), allow(dead_code))]
const KEYRING_SERVICE: &str = "neotrix";
#[cfg_attr(not(feature = "keyring"), allow(dead_code))]
const KEYRING_ENTRY: &str = "api-key-master";

/// Check whether a stored value is encrypted (starts with `enc:`).
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENC_PREFIX)
}

/// Encrypt a plaintext API key for storage.
///
/// Returns `enc:<base64(nonce ‖ ciphertext)>`.  Never logs or prints the
/// plaintext.
pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let key = get_or_create_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
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

    Ok(format!("enc:{}", BASE64.encode(&combined)))
}

/// Decrypt a value previously produced by [`encrypt`].
///
/// The plaintext is kept in memory only for the lifetime of the returned
/// `String`.  Callers should `.zeroize()` / drop promptly.
pub fn decrypt(encrypted: &str) -> Result<String, String> {
    if !is_encrypted(encrypted) {
        return Err("Value does not start with 'enc:' — not encrypted".into());
    }

    let raw = &encrypted[ENC_PREFIX.len()..];
    let data = BASE64
        .decode(raw)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if data.len() < NONCE_LEN {
        return Err("Encrypted payload too short".into());
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key = get_or_create_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("AES-256-GCM init failed: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode failed: {}", e))
}

/// Return a 32-byte AES key.
///
/// Priority:
/// 1. OS keychain (keyring crate, feature-gated, 默认启用)
/// 2. `NEOTRIX_VAULT_KEY` env var (same as `vault.rs`)
/// 3. Deterministic machine-derived key — **仅显式 opt-in**：
///    设置 `NEOTRIX_ALLOW_MACHINE_KEY=1` 才使用。机器派生密钥可被
///    本地知情者推导（machine-id/home 非机密），默认拒绝（C-3 加固）。
fn get_or_create_key() -> Result<[u8; 32], String> {
    // 生产环境优先 OS keychain；测试环境禁用（并行测试 keychain 竞争
    // 会导致 set_password 覆盖、decrypt 用错 key），测试走 env key。
    #[cfg(all(feature = "keyring", not(test)))]
    {
        match get_keyring_key() {
            Ok(key) => return Ok(key),
            Err(e) => {
                eprintln!(
                    "[neotrix] OS keychain unavailable ({}); falling back to env/machine key",
                    e
                );
            }
        }
    }

    // Fallback 1: NEOTRIX_VAULT_KEY env var (same scheme as vault.rs)
    if let Ok(key_str) = std::env::var("NEOTRIX_VAULT_KEY") {
        let trimmed = key_str.trim().to_string();
        if let Ok(decoded) = hex::decode(&trimmed) {
            if decoded.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&decoded);
                return Ok(key);
            }
        }
        // Not 32-byte hex — hash it
        let hash = Sha256::digest(trimmed.as_bytes());
        return Ok(hash.into());
    }

    // Fallback 2: machine-derived key — 显式 opt-in，默认拒绝
    if std::env::var("NEOTRIX_ALLOW_MACHINE_KEY").as_deref() == Ok("1") {
        return derive_machine_key();
    }

    Err("No encryption key available: OS keychain unavailable and NEOTRIX_VAULT_KEY not set. \
         Set NEOTRIX_VAULT_KEY (32-byte hex) or enable OS keychain. \
         Machine-derived keys are disabled by default (C-3 security hardening)."
        .into())
}

#[cfg(feature = "keyring")]
fn get_keyring_key() -> Result<[u8; 32], String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ENTRY)
        .map_err(|e| format!("keyring entry creation failed: {}", e))?;

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
            // Stored value isn't 32 hex bytes — hash it
            let hash = Sha256::digest(trimmed.as_bytes());
            Ok(hash.into())
        }
        Err(_) => {
            // Generate + persist a fresh 32-byte key
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            let hex_key = hex::encode(key);
            entry
                .set_password(&hex_key)
                .map_err(|e| format!("failed to store key in OS keychain: {}", e))?;
            Ok(key)
        }
    }
}

/// Deterministic per-machine key derived from the machine-id file (Linux) or
/// the user's home directory (macOS/fallback).
fn derive_machine_key() -> Result<[u8; 32], String> {
    let source = read_machine_id().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let hash = Sha256::digest(source.as_bytes());
    Ok(hash.into())
}

/// Read `/etc/machine-id` (Linux) or IOPlatformUUID (macOS).
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试强制使用确定性 env key（NEOTRIX_VAULT_KEY），避免 keyring
    /// 在并行测试中的 keychain 竞争（set_password 覆盖导致 decrypt 用错 key）。
    /// keyring 已是 default feature，测试必须绕开 OS keychain 才能稳定。
    const TEST_VAULT_KEY: &str = "0123456789abcdef0123456789abcdef";

    fn with_test_key(f: impl FnOnce()) {
        std::env::set_var("NEOTRIX_VAULT_KEY", TEST_VAULT_KEY);
        f();
        // 测试线程独立 env，无需清理（Rust 2021 线程级 env）
    }

    #[test]
    fn test_roundtrip() {
        with_test_key(|| {
            let plaintext = "sk-ant-test123456789";
            let encrypted = encrypt(plaintext).expect("encrypt");
            assert!(is_encrypted(&encrypted));
            assert!(encrypted.starts_with("enc:"));
            let decrypted = decrypt(&encrypted).expect("decrypt");
            assert_eq!(decrypted, plaintext);
        });
    }

    #[test]
    fn test_double_encryption_produces_different_output() {
        with_test_key(|| {
            let plaintext = "sk-test-key";
            let a = encrypt(plaintext).expect("encrypt a");
            let b = encrypt(plaintext).expect("encrypt b");
            assert_ne!(a, b, "nonce must randomize ciphertext");
            assert_eq!(
                decrypt(&a).expect("decrypt a"),
                decrypt(&b).expect("decrypt b")
            );
        });
    }

    #[test]
    fn test_is_encrypted_edge_cases() {
        assert!(!is_encrypted(""));
        assert!(!is_encrypted("sk-test123"));
        assert!(!is_encrypted("ENC:sk-test"));
        assert!(is_encrypted("enc:AAAA"));
        assert!(is_encrypted("enc:"));
    }

    #[test]
    fn test_decrypt_plaintext_fails() {
        let result = decrypt("sk-test123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("enc:"));
    }

    #[test]
    fn test_decrypt_invalid_base64_fails() {
        let result = decrypt("enc:!!!not-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_machine_key_is_deterministic() {
        let a = derive_machine_key().expect("derive a");
        let b = derive_machine_key().expect("derive b");
        assert_eq!(a, b);
    }
}
